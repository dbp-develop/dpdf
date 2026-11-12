//! Asks GitHub for the latest dpdf release (Help ▸ Check for updates).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use pdfcraft_ui_egui::updates::{RELEASES_PAGE, Release};

const LATEST: &str = "https://api.github.com/repos/dbp-develop/dpdf/releases/latest";

/// The latest release. The answer is untrusted: its size is capped, and only release pages or
/// official downloads under github.com/dbp-develop/dpdf/releases are ever used.
pub fn latest_release() -> Result<Release, String> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .tls_config(ureq::tls::TlsConfig::builder().root_certs(os_roots()?).build())
        .build()
        .new_agent();
    let mut response = agent
        .get(LATEST)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", concat!("dpdf/", env!("CARGO_PKG_VERSION")))
        .config()
        .http_status_as_error(false)
        .build()
        .call()
        .map_err(|e| format!("couldn't reach GitHub ({e})"))?;
    let status = response.status().as_u16();
    if status == 403 {
        let reset = response.headers().get("x-ratelimit-reset").and_then(|v| v.to_str().ok());
        return Err(rate_limited(reset, now_unix()));
    }
    if !(200..300).contains(&status) {
        return Err(format!("couldn't reach GitHub (http status: {status})"));
    }
    let body = response.body_mut().with_config().limit(1 << 20).read_to_string().map_err(|e| format!("unreadable answer ({e})"))?;
    parse(&body)
}

/// Downloads the release asset to a temporary directory while reporting byte progress.
pub fn download_update(url: &str, file_name: &str, progress: Arc<AtomicU64>, total: Arc<AtomicU64>) -> Result<std::path::PathBuf, String> {
    if !url.starts_with("https://github.com/dbp-develop/dpdf/releases/download/") {
        return Err("untrusted download source".into());
    }
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(300)))
        .tls_config(ureq::tls::TlsConfig::builder().root_certs(os_roots()?).build())
        .build()
        .new_agent();
    let mut response = agent.get(url).call().map_err(|e| format!("download request failed ({e})"))?;
    let status = response.status().as_u16();
    if !(200..300).contains(&status) {
        return Err(format!("download failed (http status: {status})"));
    }
    let content_len = response.headers().get("content-length").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    total.store(content_len, Ordering::Relaxed);

    let temp_dir = std::env::temp_dir();
    let safe_name: String = file_name.chars().filter(|c| c.is_alphanumeric() || matches!(*c, '.' | '-' | '_')).collect();
    if safe_name.is_empty() {
        return Err("invalid download file name".into());
    }
    let target_path = temp_dir.join(&safe_name);

    let (mut file, target_path) = match std::fs::File::create(&target_path) {
        Ok(f) => (f, target_path),
        Err(err) => {
            // If the temp directory is inaccessible (e.g. corporate policy or locked file), try a user-writable fallback
            let fallback_dir = std::env::var_os("USERPROFILE")
                .map(|p| std::path::PathBuf::from(p).join("Downloads"))
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            let fallback_path = fallback_dir.join(&safe_name);
            match std::fs::File::create(&fallback_path) {
                Ok(f) => (f, fallback_path),
                Err(_) => return Err(format!("could not create download file ({err})")),
            }
        }
    };
    let mut reader = response.body_mut().as_reader();
    let mut buffer = [0u8; 32768];
    let mut downloaded = 0u64;

    loop {
        use std::io::{Read, Write};
        let bytes_read = reader.read(&mut buffer).map_err(|e| format!("download read error ({e})"))?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read]).map_err(|e| format!("write error ({e})"))?;
        downloaded += bytes_read as u64;
        progress.store(downloaded, Ordering::Relaxed);
    }
    use std::io::Write;
    file.flush().map_err(|e| format!("file flush error ({e})"))?;
    Ok(target_path)
}

/// Inno Setup AppId, without braces. The uninstall key is `{id}_is1`.
/// This is the same id as `AppId` in `packaging/windows/dpdf.iss`.
#[cfg(any(test, target_os = "windows"))]
const INNO_APP_ID: &str = "D9DF821A-5471-48C1-80A7-0E05B5C58A91";

/// Written by the helper after the installer exits. The next launch reads it once.
const UPDATE_RESULT_FILE: &str = "update-result.txt";
/// Inno's own log (`/LOG=`), overwritten on each attempt.
const SETUP_LOG_FILE: &str = "setup.log";
/// One line per attempt: timestamp and the installer's exit code.
#[cfg(target_os = "windows")]
const UPDATE_LOG_FILE: &str = "update.log";

/// Who the silent installer should write for.
#[cfg(any(test, target_os = "windows"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InstallScope {
    /// Per-user, no administrator. Inno switch `/CURRENTUSER`.
    CurrentUser,
    /// For everyone. Inno switch `/ALLUSERS` (Windows may ask for approval).
    AllUsers,
}

#[cfg(any(test, target_os = "windows"))]
impl InstallScope {
    fn flag(self) -> &'static str {
        match self {
            Self::CurrentUser => "/CURRENTUSER",
            Self::AllUsers => "/ALLUSERS",
        }
    }
}

/// One dpdf install recorded by Inno under the uninstall key.
#[cfg(any(test, target_os = "windows"))]
#[derive(Clone, Debug, PartialEq, Eq)]
struct RegisteredInstall {
    scope: InstallScope,
    /// `InstallLocation`. Empty when the value is missing.
    location: String,
}

/// Arguments for one silent Inno run.
#[cfg(any(test, target_os = "windows"))]
#[derive(Clone, Debug, PartialEq, Eq)]
struct SilentPlan {
    scope: InstallScope,
    /// `/DIR=...` when the running copy is not the registered install.
    /// `None` lets Inno reuse the directory it already recorded for `scope`.
    dir: Option<String>,
}

/// The process the helper starts, and the arguments it passes.
#[cfg(any(test, target_os = "windows"))]
#[derive(Clone, Debug, PartialEq, Eq)]
struct InstallerCommand {
    program: String,
    args: Vec<String>,
}

#[cfg(any(test, target_os = "windows"))]
/// Pick the silent scope from the install that contains `exe_dir`.
///
/// A registered install that does not contain the running program is left alone:
/// the helper restarts this executable, so the files it replaces have to be these
/// files. When nothing registered contains them, `/DIR` names this folder.
fn plan_silent_install(exe_dir: &str, registered: &[RegisteredInstall]) -> SilentPlan {
    let mut owners: Vec<&RegisteredInstall> = registered.iter().filter(|install| dir_owns(&install.location, exe_dir)).collect();
    owners.sort_by_key(|install| std::cmp::Reverse(normalize_dir(&install.location).len()));
    if let Some(owner) = owners.first() {
        return SilentPlan { scope: owner.scope, dir: None };
    }
    let dir = trim_dir(exe_dir);
    SilentPlan { scope: scope_from_path(exe_dir), dir: (!dir.is_empty()).then(|| dir.to_string()) }
}

#[cfg(any(test, target_os = "windows"))]
/// `/VERYSILENT` plus the scope. `/DIR` only when [`plan_silent_install`] set one.
/// `/LOG` is Inno's own record of why a silent run stopped.
fn installer_args(plan: &SilentPlan, log_file: Option<&str>) -> Vec<String> {
    let mut args = vec![
        "/VERYSILENT".to_string(),
        "/SUPPRESSMSGBOXES".to_string(),
        "/NORESTART".to_string(),
        "/CLOSEAPPLICATIONS".to_string(),
        "/FORCECLOSEAPPLICATIONS".to_string(),
        plan.scope.flag().to_string(),
    ];
    if let Some(dir) = plan.dir.as_deref().map(str::trim).filter(|dir| !dir.is_empty()) {
        args.push(format!("/DIR={dir}"));
    }
    if let Some(log) = log_file.map(str::trim).filter(|log| !log.is_empty()) {
        args.push(format!("/LOG={log}"));
    }
    args
}

#[cfg(any(test, target_os = "windows"))]
fn installer_command(installer: &std::path::Path, plan: &SilentPlan, log_file: Option<&str>) -> Result<InstallerCommand, String> {
    let installer = path_text(installer)?;
    if std::path::Path::new(installer).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("msi")) {
        let mut args = vec!["/i".to_string(), installer.to_string(), "/qn".to_string(), "/norestart".to_string()];
        if let Some(log) = log_file.map(str::trim).filter(|log| !log.is_empty()) {
            args.push("/l*v".to_string());
            args.push(log.to_string());
        }
        return Ok(InstallerCommand { program: "msiexec".to_string(), args });
    }
    Ok(InstallerCommand { program: installer.to_string(), args: installer_args(plan, log_file) })
}

/// Inno and Windows Installer codes that mean the update was applied.
/// 3010 and 1641 ask for a reboot; the files are already in place.
pub fn update_succeeded(code: i32) -> bool {
    matches!(code, 0 | 3010 | 1641)
}

#[cfg(any(test, target_os = "windows"))]
fn scope_from_path(exe_dir: &str) -> InstallScope {
    let dir = normalize_dir(exe_dir);
    if dir.contains("\\program files") { InstallScope::AllUsers } else { InstallScope::CurrentUser }
}

/// `location` is the folder Inno recorded, and `exe_dir` is the folder of the running program.
#[cfg(any(test, target_os = "windows"))]
fn dir_owns(location: &str, exe_dir: &str) -> bool {
    let location = normalize_dir(location);
    let exe_dir = normalize_dir(exe_dir);
    if location.is_empty() || exe_dir.is_empty() {
        return false;
    }
    exe_dir == location || exe_dir.starts_with(&(location + "\\"))
}

#[cfg(any(test, target_os = "windows"))]
fn normalize_dir(path: &str) -> String {
    trim_dir(path).replace('/', "\\").to_lowercase()
}

#[cfg(any(test, target_os = "windows"))]
fn trim_dir(path: &str) -> &str {
    path.trim().trim_end_matches(['\\', '/'])
}

#[cfg(any(test, target_os = "windows"))]
fn path_text(path: &std::path::Path) -> Result<&str, String> {
    path.to_str().filter(|text| !text.is_empty()).ok_or_else(|| format!("the path {} is not text the updater can pass on", path.display()))
}

#[cfg(any(test, target_os = "windows"))]
/// The alternate stream Windows attaches to a downloaded file. Removing it lets a
/// silent installer start without a prompt that `/VERYSILENT` cannot show.
fn zone_identifier_stream(path: &std::path::Path) -> Option<std::path::PathBuf> {
    let text = path.to_str()?;
    if text.is_empty() || text.contains(['\n', '\r', '\0']) {
        return None;
    }
    Some(std::path::PathBuf::from(format!("{text}:Zone.Identifier")))
}

#[cfg(target_os = "windows")]
fn clear_download_mark(path: &std::path::Path) {
    let Some(stream) = zone_identifier_stream(path) else {
        return;
    };
    match std::fs::remove_file(&stream) {
        Ok(()) => log::info!("cleared the download mark on {}", path.display()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => log::warn!("could not clear the download mark on {}: {e}", path.display()),
    }
}

/// Where the helper writes the exit code and Inno writes its log.
/// A portable copy keeps that next to its own data; everyone else uses
/// `%LOCALAPPDATA%\dpdf\logs`.
fn update_state_dir() -> std::path::PathBuf {
    if let Some(dir) = pdfcraft_ui_egui::portable::data_dir() {
        return dir.join("logs");
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA").filter(|value| !value.is_empty()) {
        return std::path::PathBuf::from(local).join("dpdf").join("logs");
    }
    std::env::temp_dir().join("dpdf").join("logs")
}

/// Read and remove the helper's exit code. `None` when this launch is not a restart
/// after an update attempt.
pub fn take_update_result() -> Option<i32> {
    consume_result_file(&update_state_dir().join(UPDATE_RESULT_FILE))
}

/// The helper's exit code, read once. A missing file is the normal case.
fn consume_result_file(path: &std::path::Path) -> Option<i32> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            log::warn!("could not read the update result ({}): {e}", path.display());
            return None;
        }
    };
    if let Err(e) = std::fs::remove_file(path) {
        log::warn!("could not remove the update result ({}): {e}", path.display());
    }
    match text.trim().parse() {
        Ok(code) => Some(code),
        Err(_) => {
            log::warn!("the update result in {} was not an exit code", path.display());
            None
        }
    }
}

/// Inno's log from the last attempt. Present after a Windows update, including one that failed.
pub fn setup_log_path() -> std::path::PathBuf {
    update_state_dir().join(SETUP_LOG_FILE)
}

/// PowerShell single-quoted literal. `'` is the only character that is special inside one.
#[cfg(any(test, target_os = "windows"))]
fn ps_single(text: &str) -> String {
    let mut out = String::with_capacity(text.len().saturating_add(2));
    out.push('\'');
    for c in text.chars() {
        if c == '\'' {
            out.push_str("''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

#[cfg(any(test, target_os = "windows"))]
/// A hidden helper: wait until this process has exited, run the installer, record the
/// exit code, then start dpdf again. Waiting in this process cannot work, because the
/// installer has to replace this executable.
fn helper_script(pid: u32, command: &InstallerCommand, relaunch: &str, result_file: &str, log_file: &str) -> Result<String, String> {
    for part in [command.program.as_str(), relaunch, result_file, log_file].into_iter().chain(command.args.iter().map(String::as_str)) {
        if part.chars().any(|c| matches!(c, '\n' | '\r' | '\0')) {
            return Err("the update path contains a character the installer helper cannot quote".into());
        }
    }
    let args = command.args.iter().map(|arg| ps_single(arg)).collect::<Vec<_>>().join(", ");
    Ok(format!(
        "$ErrorActionPreference = 'Continue'\n\
         try {{ Wait-Process -Id {pid} -ErrorAction SilentlyContinue }} catch {{ }}\n\
         $code = 1\n\
         try {{\n\
         \x20 $p = Start-Process -FilePath {program} -ArgumentList @({args}) -Wait -PassThru\n\
         \x20 if ($null -ne $p -and $null -ne $p.ExitCode) {{ $code = [int]$p.ExitCode }}\n\
         }} catch {{ $code = 1 }}\n\
         try {{\n\
         \x20 $dir = Split-Path -Parent {result}\n\
         \x20 if ($dir) {{ New-Item -ItemType Directory -Force -Path $dir | Out-Null }}\n\
         \x20 Set-Content -LiteralPath {result} -Value $code -Encoding ascii\n\
         \x20 $stamp = Get-Date -Format o\n\
         \x20 Add-Content -LiteralPath {log} -Value \"$stamp installer exit $code\"\n\
         }} catch {{ }}\n\
         try {{ Start-Process -FilePath {relaunch} }} catch {{ }}\n",
        program = ps_single(&command.program),
        result = ps_single(result_file),
        log = ps_single(log_file),
        relaunch = ps_single(relaunch),
    ))
}

#[cfg(target_os = "windows")]
fn registered_installs() -> Vec<RegisteredInstall> {
    let subkey = format!("Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{{{INNO_APP_ID}}}_is1");
    let mut found = Vec::new();
    if let Some(location) = uninstall_location(&winsafe::HKEY::CURRENT_USER, &subkey) {
        found.push(RegisteredInstall { scope: InstallScope::CurrentUser, location });
    }
    if let Some(location) = uninstall_location(&winsafe::HKEY::LOCAL_MACHINE, &subkey) {
        found.push(RegisteredInstall { scope: InstallScope::AllUsers, location });
    }
    found
}

/// `Some(location)` when Inno's uninstall key is present. The location may be empty.
#[cfg(target_os = "windows")]
fn uninstall_location(root: &winsafe::HKEY, subkey: &str) -> Option<String> {
    use winsafe::{RegistryValue, co::RRF};
    // UninstallString is written for every Inno install. InstallLocation is the folder.
    // Accept an expandable string too: a plain `REG_SZ` restriction would miss the key.
    if root.RegGetValue(Some(subkey), Some("UninstallString"), RRF::RT_ANY).is_err() {
        return None;
    }
    match root.RegGetValue(Some(subkey), Some("InstallLocation"), RRF::RT_ANY) {
        Ok(RegistryValue::Sz(location) | RegistryValue::ExpandSz(location)) => Some(location),
        _ => Some(String::new()),
    }
}

#[cfg(target_os = "windows")]
fn spawn_helper(script: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    use std::process::Stdio;
    // CREATE_NO_WINDOW: powershell.exe is a console program, and this app has no console.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", script])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("could not start the update helper ({e})"))
}

/// Launches the installer and exits the app.
///
/// On Windows the installer runs only after this process has left, so it can replace
/// `dpdf.exe`. dpdf starts again when the installer has exited. A non-zero exit code is
/// written for the next launch to show. The scope comes from the install that contains
/// this executable (`/CURRENTUSER` or `/ALLUSERS`), which the setup script accepts on
/// the command line.
pub fn launch_installer(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        clear_download_mark(path);
        let current_exe = std::env::current_exe().map_err(|e| format!("could not find dpdf to restart ({e})"))?;
        let relaunch = path_text(&current_exe)?.to_string();
        let exe_dir = current_exe
            .parent()
            .and_then(|dir| dir.to_str())
            .filter(|dir| !dir.is_empty())
            .ok_or("could not find the folder dpdf is running from")?;
        let plan = plan_silent_install(exe_dir, &registered_installs());
        let state_dir = update_state_dir();
        std::fs::create_dir_all(&state_dir).map_err(|e| format!("could not create the update log folder {}: {e}", state_dir.display()))?;
        let setup_log = state_dir.join(SETUP_LOG_FILE);
        let result_file = state_dir.join(UPDATE_RESULT_FILE);
        let update_log = state_dir.join(UPDATE_LOG_FILE);
        let log_text = path_text(&setup_log)?;
        let command = installer_command(path, &plan, Some(log_text))?;
        // A leftover code from an earlier attempt must not be reported if this helper never writes one.
        let _ = std::fs::remove_file(&result_file);
        let script = helper_script(std::process::id(), &command, &relaunch, path_text(&result_file)?, path_text(&update_log)?)?;
        spawn_helper(&script)?;
        log::info!("update helper started for {} ({})", path.display(), plan.scope.flag());
        // The helper waits for this process, then replaces this executable.
        std::process::exit(0);
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn().map_err(|e| format!("could not open installer ({e})"))?;
        std::process::exit(0);
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(path, perms);
        }
        std::process::Command::new(path)
            .spawn()
            .or_else(|_| std::process::Command::new("xdg-open").arg(path).spawn())
            .map_err(|e| format!("could not launch package ({e})"))?;
        std::process::exit(0);
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = path;
        Ok(())
    }
}

/// The certificate authorities the operating system trusts.
fn os_roots() -> Result<ureq::tls::RootCerts, String> {
    let found = rustls_native_certs::load_native_certs();
    let certs: Vec<ureq::tls::Certificate<'static>> = found.certs.iter().map(|c| ureq::tls::Certificate::from_der(c.as_ref()).to_owned()).collect();
    if certs.is_empty() {
        return Err("no trusted certificates found on this system".into());
    }
    Ok(ureq::tls::RootCerts::new_with_certs(&certs))
}

fn now_unix() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn rate_limited(reset: Option<&str>, now: u64) -> String {
    const FALLBACK: &str = "GitHub is limiting update checks for now (shared hourly quota); try again later";
    let wait = reset.and_then(|r| r.trim().parse::<u64>().ok()).and_then(|reset| reset.checked_sub(now)).filter(|&wait| wait > 0);
    match wait {
        Some(wait) => {
            let minutes = wait.div_ceil(60).max(1);
            let plural = if minutes == 1 { "" } else { "s" };
            format!("GitHub is limiting update checks for now (shared hourly quota); try again in about {minutes} minute{plural}")
        }
        None => FALLBACK.into(),
    }
}

fn parse(body: &str) -> Result<Release, String> {
    let v: serde_json::Value = serde_json::from_str(body).map_err(|e| format!("unreadable answer ({e})"))?;
    let version = v["tag_name"].as_str().filter(|t| !t.is_empty() && t.len() <= 64).ok_or("no release found")?.to_string();
    let url = v["html_url"]
        .as_str()
        .filter(|u| u.strip_prefix(RELEASES_PAGE).is_some_and(|rest| rest.starts_with('/') && !rest.contains(['?', '#', '\\'])))
        .unwrap_or(RELEASES_PAGE)
        .to_string();

    let mut download_url = None;
    let mut file_name = None;

    if let Some(assets) = v["assets"].as_array() {
        let mut best_score = 0;
        for asset in assets {
            let name = asset["name"].as_str().unwrap_or_default();
            let browser_download_url = asset["browser_download_url"].as_str().unwrap_or_default();

            if !browser_download_url.starts_with("https://github.com/dbp-develop/dpdf/releases/download/") {
                continue;
            }

            #[cfg(target_os = "windows")]
            let score = if name.ends_with("-setup.exe") || name.ends_with("-installer.exe") {
                3
            } else if name.ends_with(".msi") {
                2
            } else if name.ends_with(".exe") && !name.contains("-cli") {
                1
            } else {
                0
            };

            #[cfg(target_os = "macos")]
            let score = if name.ends_with(".dmg") {
                3
            } else if name.ends_with(".pkg") {
                2
            } else if name.ends_with(".zip") {
                1
            } else {
                0
            };

            #[cfg(target_os = "linux")]
            let score = if name.ends_with(".AppImage") {
                3
            } else if name.ends_with(".deb") {
                2
            } else if name.ends_with(".tar.gz") {
                1
            } else {
                0
            };

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            let score = 0;

            if score > best_score {
                best_score = score;
                download_url = Some(browser_download_url.to_string());
                file_name = Some(name.to_string());
            }
        }
    }

    Ok(Release { version, url, download_url, file_name })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_are_read_and_only_our_release_pages_are_offered() {
        let r = parse(r#"{"tag_name":"v0.2.0","html_url":"https://github.com/dbp-develop/dpdf/releases/tag/v0.2.0"}"#).unwrap();
        assert_eq!(
            r,
            Release {
                version: "v0.2.0".into(),
                url: "https://github.com/dbp-develop/dpdf/releases/tag/v0.2.0".into(),
                download_url: None,
                file_name: None,
            }
        );
        for elsewhere in ["https://example.com/dpdf.exe", "https://github.com/dbp-develop/dpdf/releases.evil/x", "javascript:alert(1)"] {
            let r = parse(&format!(r#"{{"tag_name":"v9.9.9","html_url":"{elsewhere}"}}"#)).unwrap();
            assert_eq!(r.url, RELEASES_PAGE, "{elsewhere}");
        }
        assert!(parse(r#"{"message":"Not Found"}"#).is_err());
        assert!(parse("<html>").is_err());
    }

    #[test]
    fn platform_installer_asset_is_parsed_when_present() {
        let json = r#"{
            "tag_name": "v1.0.1",
            "html_url": "https://github.com/dbp-develop/dpdf/releases/tag/v1.0.1",
            "assets": [
                {
                    "name": "dpdf-1.0.1-windows-x64-portable.zip",
                    "browser_download_url": "https://github.com/dbp-develop/dpdf/releases/download/v1.0.1/dpdf-1.0.1-windows-x64-portable.zip"
                },
                {
                    "name": "dpdf-1.0.1-setup.exe",
                    "browser_download_url": "https://github.com/dbp-develop/dpdf/releases/download/v1.0.1/dpdf-1.0.1-setup.exe"
                },
                {
                    "name": "dpdf_1.0.1_amd64.AppImage",
                    "browser_download_url": "https://github.com/dbp-develop/dpdf/releases/download/v1.0.1/dpdf_1.0.1_amd64.AppImage"
                },
                {
                    "name": "dpdf-1.0.1.dmg",
                    "browser_download_url": "https://github.com/dbp-develop/dpdf/releases/download/v1.0.1/dpdf-1.0.1.dmg"
                }
            ]
        }"#;
        let r = parse(json).unwrap();
        assert_eq!(r.version, "v1.0.1");
        #[cfg(target_os = "windows")]
        {
            assert_eq!(r.file_name.as_deref(), Some("dpdf-1.0.1-setup.exe"));
            assert_eq!(r.download_url.as_deref(), Some("https://github.com/dbp-develop/dpdf/releases/download/v1.0.1/dpdf-1.0.1-setup.exe"));
        }
        #[cfg(target_os = "linux")]
        {
            assert_eq!(r.file_name.as_deref(), Some("dpdf_1.0.1_amd64.AppImage"));
            assert_eq!(r.download_url.as_deref(), Some("https://github.com/dbp-develop/dpdf/releases/download/v1.0.1/dpdf_1.0.1_amd64.AppImage"));
        }
        #[cfg(target_os = "macos")]
        {
            assert_eq!(r.file_name.as_deref(), Some("dpdf-1.0.1.dmg"));
            assert_eq!(r.download_url.as_deref(), Some("https://github.com/dbp-develop/dpdf/releases/download/v1.0.1/dpdf-1.0.1.dmg"));
        }
    }

    #[test]
    fn rate_limit_messages_name_the_wait() {
        let limited = "GitHub is limiting update checks for now (shared hourly quota); try again in about 20 minutes";
        assert_eq!(rate_limited(Some("1800001200"), 1800000000), limited);
        assert_eq!(
            rate_limited(Some("1800000061"), 1800000000),
            "GitHub is limiting update checks for now (shared hourly quota); try again in about 2 minutes"
        );
        assert_eq!(
            rate_limited(Some("1800000060"), 1800000000),
            "GitHub is limiting update checks for now (shared hourly quota); try again in about 1 minute"
        );
        let later = "GitHub is limiting update checks for now (shared hourly quota); try again later";
        for bad in [None, Some(""), Some("soon"), Some("1799999999"), Some("1800000000")] {
            assert_eq!(rate_limited(bad, 1800000000), later);
        }
    }

    fn user_install(location: &str) -> RegisteredInstall {
        RegisteredInstall { scope: InstallScope::CurrentUser, location: location.to_string() }
    }

    fn machine_install(location: &str) -> RegisteredInstall {
        RegisteredInstall { scope: InstallScope::AllUsers, location: location.to_string() }
    }

    #[test]
    fn the_setup_script_accepts_silent_scope_switches() {
        let iss = include_str!("../../../packaging/windows/dpdf.iss");
        assert!(
            iss.contains("PrivilegesRequiredOverridesAllowed=dialog commandline"),
            "a silent per-user update needs /CURRENTUSER on the command line"
        );
        let app_id = format!("{{{{{INNO_APP_ID}}}");
        assert!(iss.contains(&app_id), "AppId must match the updater's uninstall lookup, missing {app_id}");
    }

    #[test]
    fn a_per_user_install_stays_silent_for_the_current_user() {
        let exe = r"C:\Users\Ada\AppData\Local\Programs\dpdf";
        let plan = plan_silent_install(exe, &[user_install(r"C:\Users\Ada\AppData\Local\Programs\dpdf\")]);
        assert_eq!(plan.scope, InstallScope::CurrentUser);
        assert_eq!(plan.dir, None, "Inno already recorded this folder");
        let args = installer_args(&plan, Some(r"C:\Users\Ada\AppData\Local\dpdf\logs\setup.log"));
        assert!(args.iter().any(|arg| arg == "/CURRENTUSER"));
        assert!(args.iter().all(|arg| arg != "/ALLUSERS"));
        assert!(args.iter().all(|arg| !arg.starts_with("/DIR=")));
        assert!(args.iter().any(|arg| arg == r"/LOG=C:\Users\Ada\AppData\Local\dpdf\logs\setup.log"));
        assert!(args.iter().any(|arg| arg == "/VERYSILENT"));
    }

    #[test]
    fn an_all_users_install_uses_that_scope_when_both_exist() {
        let exe = r"C:\Program Files\dpdf";
        let plan = plan_silent_install(exe, &[user_install(r"C:\Users\Ada\AppData\Local\Programs\dpdf"), machine_install(r"C:\Program Files\dpdf")]);
        assert_eq!(plan.scope, InstallScope::AllUsers);
        assert_eq!(plan.dir, None);
        let args = installer_args(&plan, None);
        assert!(args.iter().any(|arg| arg == "/ALLUSERS"));
        assert!(args.iter().all(|arg| arg != "/CURRENTUSER"));
    }

    #[test]
    fn a_running_copy_outside_the_registered_install_is_updated_in_place() {
        let exe = r"D:\portable\dpdf";
        let plan = plan_silent_install(exe, &[user_install(r"C:\Users\Ada\AppData\Local\Programs\dpdf")]);
        assert_eq!(plan.scope, InstallScope::CurrentUser);
        assert_eq!(plan.dir.as_deref(), Some(exe));
        let args = installer_args(&plan, None);
        assert!(args.iter().any(|arg| arg == r"/DIR=D:\portable\dpdf"));
    }

    #[test]
    fn program_files_without_a_registration_is_for_everyone() {
        let exe = r"C:\Program Files (x86)\dpdf";
        let plan = plan_silent_install(exe, &[]);
        assert_eq!(plan.scope, InstallScope::AllUsers);
        assert_eq!(plan.dir.as_deref(), Some(exe));
    }

    #[test]
    fn install_location_matching_ignores_case_and_a_trailing_slash() {
        let plan = plan_silent_install("c:/users/ada/appdata/local/programs/dpdf", &[user_install(r"C:\Users\Ada\AppData\Local\Programs\dpdf\")]);
        assert_eq!(plan.scope, InstallScope::CurrentUser);
        assert!(plan.dir.is_none());
    }

    #[test]
    fn the_helper_waits_for_the_installer_and_quotes_paths() {
        let command = installer_command(
            std::path::Path::new(r"C:\Users\O'Brien\AppData\Local\Temp\dpdf-2.1.0-setup.exe"),
            &SilentPlan { scope: InstallScope::CurrentUser, dir: None },
            Some(r"C:\Users\O'Brien\AppData\Local\dpdf\logs\setup.log"),
        )
        .unwrap();
        assert_eq!(command.program, r"C:\Users\O'Brien\AppData\Local\Temp\dpdf-2.1.0-setup.exe");
        let script = helper_script(
            4242,
            &command,
            r"C:\Users\O'Brien\AppData\Local\Programs\dpdf\dpdf.exe",
            r"C:\Users\O'Brien\AppData\Local\dpdf\logs\update-result.txt",
            r"C:\Users\O'Brien\AppData\Local\dpdf\logs\update.log",
        )
        .unwrap();
        assert!(script.contains("Wait-Process -Id 4242"), "{script}");
        assert!(script.contains("-Wait -PassThru"), "{script}");
        assert!(script.contains("O''Brien"), "{script}");
        assert!(!script.contains("ping "), "{script}");
        let install_at = script.find("-Wait -PassThru").expect("installer wait");
        let restart_at = script.find(r"Start-Process -FilePath 'C:\Users\O''Brien\AppData\Local\Programs\dpdf\dpdf.exe'").expect("relaunch");
        assert!(install_at < restart_at, "dpdf starts again only after the installer returns");
    }

    #[test]
    fn an_msi_is_handed_to_msiexec() {
        let command = installer_command(
            std::path::Path::new(r"C:\Temp\dpdf.msi"),
            &SilentPlan { scope: InstallScope::CurrentUser, dir: None },
            Some(r"C:\Temp\setup.log"),
        )
        .unwrap();
        assert_eq!(command.program, "msiexec");
        assert_eq!(command.args.first().map(String::as_str), Some("/i"));
        assert!(command.args.iter().any(|arg| arg == "/qn"));
        assert!(command.args.iter().any(|arg| arg == r"C:\Temp\setup.log"));
    }

    #[test]
    fn a_path_with_a_newline_is_refused() {
        let command = InstallerCommand { program: "setup.exe\n".into(), args: Vec::new() };
        let err = helper_script(1, &command, "dpdf.exe", "result.txt", "update.log").unwrap_err();
        assert!(err.contains("cannot quote"), "{err}");
    }

    #[test]
    fn the_download_mark_is_the_zone_stream() {
        let path = zone_identifier_stream(std::path::Path::new(r"C:\Temp\dpdf-setup.exe")).unwrap();
        assert_eq!(path, std::path::PathBuf::from(r"C:\Temp\dpdf-setup.exe:Zone.Identifier"));
        assert!(zone_identifier_stream(std::path::Path::new("a\nb.exe")).is_none());
    }

    #[test]
    fn success_codes_include_a_reboot_request() {
        assert!(update_succeeded(0));
        assert!(update_succeeded(3010));
        assert!(update_succeeded(1641));
        assert!(!update_succeeded(1));
        assert!(!update_succeeded(1603));
    }

    #[test]
    fn the_result_file_is_read_once() {
        let dir = std::env::temp_dir().join(format!("dpdf-update-result-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("update-result.txt");
        std::fs::write(&path, "1603\r\n").unwrap();
        assert_eq!(consume_result_file(&path), Some(1603));
        assert!(!path.exists(), "the code is not shown on the launch after that");
        assert_eq!(consume_result_file(&path), None);
        std::fs::write(&path, "nope").unwrap();
        assert_eq!(consume_result_file(&path), None);
        assert!(!path.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
