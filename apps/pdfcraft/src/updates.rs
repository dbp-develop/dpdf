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

    let mut file = std::fs::File::create(&target_path).map_err(|e| format!("could not create temporary file ({e})"))?;
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

/// Launches the installer and exits the app cleanly.
pub fn launch_installer(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let is_msi = path.extension().and_then(|s| s.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("msi"));
        if is_msi {
            std::process::Command::new("msiexec").arg("/i").arg(path).spawn().map_err(|e| format!("could not launch msi installer ({e})"))?;
        } else {
            std::process::Command::new(path).spawn().map_err(|e| format!("could not launch installer ({e})"))?;
        }
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
}
