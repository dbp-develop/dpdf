# Changelog

All notable changes to `dpdf` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
All versions follow the `v1.0.*` series.

---

## [1.0.1] - 2026-10-27

### Fixed
- **Attachment Panel Autopopup**: Prevented the attachments right-hand panel from automatically popping open upon opening PDF documents (e.g., documents specifying `/PageMode /UseAttachments` or containing file-attachment annotations). This resolves an issue where the inspector panel occupied ~30% of screen real estate and obstructed document viewing upon launch. Attachments remain easily accessible on demand via the paperclip icon in the right-hand vertical menu rail.

### Changed
- Updated version metadata across workspace crates and build manifests to `1.0.1`.
- Updated release packages, download links, and website download hubs for the `v1.0.1` release.

---

## [1.0.0] - 2026-10-22 — Base Release

This is the initial baseline release of **dpdf**, establishing the modern foundation for future updates.

### Added
- **Visual Identity & Rebrand**: Full transition to **dpdf** featuring Ruby Crimson (`#D13B3B`) and Midnight Slate (`#1E222B`) styling, high-resolution vector and raster icons (`dpdf.svg`, `dpdf.ico`, `dpdf.icns`, `dpdf-1024.png`), and document file association icons (`dpdf-document.ico`).
- **Windows Packaging**:
  - Inno Setup graphical installer: `dpdf-1.0.0-setup.exe`.
  - WiX v5 dual-scope MSI package (supporting per-user and per-machine installations): `dpdf-1.0.0-windows-x64.msi`.
  - Zero-installation portable edition: `dpdf-1.0.0-windows-x64-portable.zip`.
- **macOS Universal Packaging**:
  - Universal binary supporting both Apple Silicon (ARM64) and Intel (x86_64).
  - Drag-to-Applications disk image: `dpdf-1.0.0-macos-universal.dmg`.
  - Universal headless CLI archive: `dpdf-cli-1.0.0-macos-universal.zip`.
- **Offline-First In-App Updater**:
  - 100% offline by default: zero telemetry, zero background network pings, and zero automatic connections.
  - Interactive Trust & Privacy Consent warning dialog before any internet connection is made.
  - Integrated in-app download with real-time download progress and percentage tracking.
  - One-click "Install & Restart" to automatically launch the installer and update the entire application.
  - Placed away from distraction in **Preferences ▸ Updates** and **Help ▸ Check for updates…**.
- **Automated CI/CD Workflows**: Multi-platform build and release workflows for Windows and macOS.
- **Direct Download Hub**: Direct platform download buttons on the repository home page and in-app Home tab for Windows (.exe, .msi, portable .zip), macOS (.dmg, .zip), and Linux.

### Changed
- Standardized package versions and workspace dependencies to base version `1.0.0`.
- Streamlined interface for clean, focused PDF reading and editing without extraneous prompts.
