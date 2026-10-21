//! Offline update stub: dpdf operates 100% offline with zero network access.

use pdfcraft_ui_egui::updates::Release;

/// In dpdf, network access is completely disabled.
pub fn latest_release() -> Result<Release, String> {
    Err("Offline mode: network access is disabled in dpdf.".into())
}
