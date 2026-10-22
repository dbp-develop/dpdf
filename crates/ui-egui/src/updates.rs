//! Help ▸ Check for updates: ask for the latest release and offer in-app download and update.
//!
//! dpdf operates 100% offline by default. It asks for explicit user consent before initiating
//! any network connections, and connects only on explicit demand.

use std::sync::Arc;

use egui::{Align, Layout};

use crate::{PdfCraftApp, theme, widgets};

/// Where every dpdf release is listed.
pub const RELEASES_PAGE: &str = "https://github.com/dbp-develop/dpdf/releases";

/// The latest published release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    /// Its version tag, such as `v0.2.0`.
    pub version: String,
    /// Its page on [`RELEASES_PAGE`], where the downloads are.
    pub url: String,
    /// Direct installer download URL if available.
    pub download_url: Option<String>,
    /// File name of the installer asset (e.g. `dpdf-1.0.1-setup.exe`).
    pub file_name: Option<String>,
}

impl Release {
    pub fn new(version: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            url: url.into(),
            download_url: None,
            file_name: None,
        }
    }
}

/// Asks for the latest release (blocking; it runs on its own thread).
pub type UpdateSource = Arc<dyn Fn() -> Result<Release, String> + Send + Sync>;

/// Downloads the release asset to a file (blocking; runs on its own thread).
/// Takes (download_url, file_name, progress_bytes, total_bytes).
pub type UpdateDownloader = Arc<
    dyn Fn(&str, &str, Arc<std::sync::atomic::AtomicU64>, Arc<std::sync::atomic::AtomicU64>) -> Result<std::path::PathBuf, String>
        + Send
        + Sync,
>;

/// Launches the installer for an in-app update.
pub type InstallerLauncher = Arc<dyn Fn(&std::path::Path) -> Result<(), String> + Send + Sync>;

/// Whether release `latest` (a tag such as `v0.2.0`) is newer than version `current` (`0.1.1`).
/// Pre-release and build suffixes are ignored; a version that doesn't parse is never newer.
pub fn is_newer(latest: &str, current: &str) -> bool {
    matches!((parse(latest), parse(current)), (Some(l), Some(c)) if l > c)
}

fn parse(v: &str) -> Option<(u64, u64, u64)> {
    let v = v.trim().trim_start_matches(['v', 'V']);
    let core = v.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let mut next = |required: bool| match parts.next() {
        Some(p) => p.parse::<u64>().ok(),
        None if required => None,
        None => Some(0),
    };
    let version = (next(true)?, next(false)?, next(false)?);
    parts.next().is_none().then_some(version)
}

/// Download stage for the in-app updater.
pub(crate) enum DownloadState {
    Idle,
    Downloading {
        progress: Arc<std::sync::atomic::AtomicU64>,
        total: Arc<std::sync::atomic::AtomicU64>,
        rx: std::sync::mpsc::Receiver<Result<std::path::PathBuf, String>>,
    },
    Done(std::path::PathBuf),
    Failed(String),
}

impl Default for DownloadState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Where a check is.
#[derive(Default)]
pub(crate) enum Check {
    #[default]
    Idle,
    #[cfg(not(target_arch = "wasm32"))]
    Running(std::sync::mpsc::Receiver<Result<Release, String>>),
    Done(Result<Release, String>),
}

#[derive(Default)]
pub(crate) struct Updates {
    pub(crate) check: Check,
    pub(crate) download: DownloadState,
    /// Whether the user gave explicit consent to connect to the internet.
    pub(crate) consented: bool,
    /// The Updates dialog is showing.
    pub(crate) open: bool,
}

impl PdfCraftApp {
    /// Help ▸ Check for updates: open the update dialog with a consent prompt first.
    pub fn check_for_updates(&mut self) {
        self.updates.open = true;
        self.updates.consented = false;
        self.updates.check = Check::Idle;
        self.updates.download = DownloadState::Idle;
    }

    /// User consented to connect online: start the background release check thread.
    pub fn start_update_check(&mut self) {
        self.updates.consented = true;
        self.updates.download = DownloadState::Idle;
        let Some(source) = self.update_source.clone() else {
            self.open_url(RELEASES_PAGE);
            return;
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            if matches!(self.updates.check, Check::Running(_)) {
                return;
            }
            let (tx, rx) = std::sync::mpsc::channel();
            let ctx = self.ctx.clone();
            std::thread::spawn(move || {
                let _ = tx.send(source());
                if let Some(ctx) = ctx {
                    ctx.request_repaint();
                }
            });
            self.updates.check = Check::Running(rx);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = source;
            self.open_url(RELEASES_PAGE);
        }
    }

    /// Starts downloading the release installer asset in the background.
    pub fn start_download(&mut self, url: String, file_name: String) {
        let Some(downloader) = self.update_downloader.clone() else {
            self.open_url(&url);
            return;
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            let progress = Arc::new(std::sync::atomic::AtomicU64::new(0));
            let total = Arc::new(std::sync::atomic::AtomicU64::new(0));
            let (tx, rx) = std::sync::mpsc::channel();
            let p_clone = progress.clone();
            let t_clone = total.clone();
            let ctx = self.ctx.clone();
            std::thread::spawn(move || {
                let res = downloader(&url, &file_name, p_clone, t_clone);
                let _ = tx.send(res);
                if let Some(ctx) = ctx {
                    ctx.request_repaint();
                }
            });
            self.updates.download = DownloadState::Downloading { progress, total, rx };
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = downloader;
            self.open_url(&url);
        }
    }

    /// Pick up finished background checks and downloads (each frame).
    pub(crate) fn poll_updates(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Check::Running(rx) = &self.updates.check {
                match rx.try_recv() {
                    Ok(r) => self.updates.check = Check::Done(r),
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.updates.check = Check::Done(Err("the update check stopped unexpectedly".into()));
                    }
                }
            }
            if let DownloadState::Downloading { rx, .. } = &self.updates.download {
                match rx.try_recv() {
                    Ok(Ok(path)) => self.updates.download = DownloadState::Done(path),
                    Ok(Err(e)) => self.updates.download = DownloadState::Failed(e),
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.updates.download = DownloadState::Failed("the download stopped unexpectedly".into());
                    }
                }
            }
        }
    }
}

/// The Updates dialog.
pub(crate) fn dialog(app: &mut PdfCraftApp, ctx: &egui::Context) {
    if !app.updates.open {
        return;
    }
    let t = theme::Tokens::get(ctx);
    let current = env!("CARGO_PKG_VERSION");
    let mut close = false;
    let mut start_check = false;
    let mut start_download: Option<(String, String)> = None;
    let mut open_url: Option<String> = None;
    let mut launch_path: Option<std::path::PathBuf> = None;

    let modal = crate::widgets::show_modal(ctx, "updates", |ui| {
        ui.set_width(440.0);

        // CASE 1: Consent Prompt (User must explicitly agree before any online connection)
        if !app.updates.consented {
            ui.horizontal(|ui| {
                ui.add(crate::icons::image("shield-check", 22.0, t.accent));
                ui.label(egui::RichText::new(tl!("Check for updates")).font(theme::semibold(16.0)));
            });
            ui.add_space(8.0);
            ui.label(egui::RichText::new(tl!("dpdf operates completely offline by default.")).strong());
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(tl!(
                    "Checking for updates will connect to GitHub (github.com/dbp-develop/dpdf) using your internet connection to see if a newer version is available.\n\nNo document content, personal information, or telemetry is ever collected or transmitted."
                ))
                .color(t.text_muted),
            );
            ui.add_space(8.0);
            ui.label(tl!("Do you agree to connect to the internet to check for updates?"));
            ui.add_space(14.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::pill_button(ui, tl!("Agree & Check"), true).clicked() {
                    start_check = true;
                }
                if widgets::pill_button(ui, tl!("Cancel"), false).clicked() {
                    close = true;
                }
            });
            return;
        }

        // CASE 2: Consented - Show Check or Download Progress
        ui.horizontal(|ui| {
            ui.add(crate::icons::image("cloud", 22.0, t.accent));
            ui.label(egui::RichText::new(tl!("Check for updates")).font(theme::semibold(16.0)));
        });
        ui.add_space(8.0);

        match (&app.updates.check, &app.updates.download) {
            // While downloading update installer
            (_, DownloadState::Downloading { progress, total, .. }) => {
                let p = progress.load(std::sync::atomic::Ordering::Relaxed);
                let tot = total.load(std::sync::atomic::Ordering::Relaxed);
                ui.label(egui::RichText::new(tl!("Downloading update…")).strong());
                ui.add_space(6.0);
                let fraction = if tot > 0 { (p as f32 / tot as f32).clamp(0.0, 1.0) } else { 0.0 };
                let mb_p = p as f64 / 1_048_576.0;
                let mb_tot = tot as f64 / 1_048_576.0;
                let text = if tot > 0 {
                    format!("{mb_p:.1} MB / {mb_tot:.1} MB ({:.0}%)", fraction * 100.0)
                } else {
                    format!("{mb_p:.1} MB")
                };
                ui.add(egui::ProgressBar::new(fraction).text(text).animate(true));
                ui.add_space(12.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::pill_button(ui, tl!("Cancel"), false).clicked() {
                        app.updates.download = DownloadState::Idle;
                    }
                });
            }

            // Download succeeded
            (_, DownloadState::Done(path)) => {
                ui.label(egui::RichText::new(tl!("Update downloaded and ready to install.")).strong());
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(tl!("dpdf will close and launch the installer to update the entire application."))
                        .color(t.text_muted),
                );
                ui.add_space(12.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::pill_button(ui, tl!("Install & Restart"), true).clicked() {
                        launch_path = Some(path.clone());
                        close = true;
                    }
                    if widgets::pill_button(ui, tl!("Later"), false).clicked() {
                        close = true;
                    }
                });
            }

            // Download failed
            (_, DownloadState::Failed(err)) => {
                ui.label(egui::RichText::new(tl!("Update download failed.")).strong());
                ui.add_space(4.0);
                ui.label(egui::RichText::new(format!("{err}")).color(t.text_muted));
                ui.add_space(12.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::pill_button(ui, tl!("Open in browser"), true).clicked() {
                        open_url = Some(RELEASES_PAGE.to_string());
                        close = true;
                    }
                    if widgets::pill_button(ui, tl!("Close"), false).clicked() {
                        close = true;
                    }
                });
            }

            // Idle check
            (Check::Idle, DownloadState::Idle) => {
                ui.label(tl!("Ready to check for updates."));
                ui.add_space(12.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::pill_button(ui, tl!("Check now"), true).clicked() {
                        start_check = true;
                    }
                    if widgets::pill_button(ui, tl!("Close"), false).clicked() {
                        close = true;
                    }
                });
            }

            // Running online check
            #[cfg(not(target_arch = "wasm32"))]
            (Check::Running(_), DownloadState::Idle) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(tl!("Checking for a newer version…"));
                });
                ui.add_space(12.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::pill_button(ui, tl!("Cancel"), false).clicked() {
                        close = true;
                    }
                });
            }

            // Check finished: update available
            (Check::Done(Ok(r)), DownloadState::Idle) if is_newer(&r.version, current) => {
                let version = r.version.trim_start_matches(['v', 'V']);
                ui.label(egui::RichText::new(crate::i18n::fmt(tl!("dpdf {v} is available."), &[("v", version)])).strong());
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(crate::i18n::fmt(
                        tl!("You have version {c}. An updated version is available to install."),
                        &[("c", current)],
                    ))
                    .color(t.text_muted),
                );
                ui.add_space(12.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(dl_url) = &r.download_url {
                        let fname = r.file_name.clone().unwrap_or_else(|| format!("dpdf-{version}-setup.exe"));
                        if widgets::pill_button(ui, tl!("Download & Update"), true).clicked() {
                            start_download = Some((dl_url.clone(), fname));
                        }
                    } else if widgets::pill_button(ui, tl!("Download"), true).clicked() {
                        open_url = Some(r.url.clone());
                        close = true;
                    }
                    if widgets::pill_button(ui, tl!("Website"), false).clicked() {
                        open_url = Some(r.url.clone());
                        close = true;
                    }
                    if widgets::pill_button(ui, tl!("Later"), false).clicked() {
                        close = true;
                    }
                });
            }

            // Check finished: up to date
            (Check::Done(Ok(_)), DownloadState::Idle) => {
                ui.label(crate::i18n::fmt(tl!("dpdf {c} is up to date."), &[("c", current)]));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(tl!("You have the latest version installed.")).color(t.text_muted));
                ui.add_space(12.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::pill_button(ui, tl!("Close"), true).clicked() {
                        close = true;
                    }
                });
            }

            // Check finished: error
            (Check::Done(Err(e)), DownloadState::Idle) => {
                ui.label(crate::i18n::fmt(tl!("Couldn't check for updates: {e}"), &[("e", &e.to_string())]));
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(crate::i18n::fmt(
                        tl!("You have version {c}. All releases are listed at {page}."),
                        &[("c", current), ("page", RELEASES_PAGE)],
                    ))
                    .color(t.text_muted),
                );
                ui.add_space(12.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::pill_button(ui, tl!("Open in browser"), false).clicked() {
                        open_url = Some(RELEASES_PAGE.to_string());
                        close = true;
                    }
                    if widgets::pill_button(ui, tl!("Close"), true).clicked() {
                        close = true;
                    }
                });
            }
        }
    });

    if modal.should_close() {
        close = true;
    }
    if start_check {
        app.start_update_check();
    }
    if let Some((url, fname)) = start_download {
        app.start_download(url, fname);
    }
    if let Some(url) = open_url {
        app.open_url(&url);
    }
    if let Some(path) = launch_path {
        if let Some(launcher) = &app.installer_launcher {
            let _ = launcher(&path);
        }
    }
    if close {
        app.updates.open = false;
    }
}
