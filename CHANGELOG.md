# Changelog

All notable changes to `dpdf` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [2.0.0] - 2026-10-30 — Major UI Redesign & Enhanced UX

### Added
- **Modern Studio Slate Theme (Dark Mode)**:
  - Deep studio slate pasteboard (`#090D13`) providing high contrast and document pop without black glare.
  - Refined dark slate chrome (`#161B22`) and card elevations (`#1F2633`) with crisp border strokes (`#30363D`).
  - Electric cyan brand accents (`#00A4D6`) across active tools, buttons, selection highlights, and indicators.
  - Layered ambient elevation drop shadows (`blur: 32` for windows, `22` for canvas pages).
- **Nordic Clean Light Theme (Light Mode)**:
  - Soft cool-gray pasteboard (`#EBF0F5`) that eliminates harsh paper glare and eye strain.
  - Crisp white panels and cards (`#FFFFFF` and `#F8FAFC`) with deep slate typography (`#0F172A`).
  - Sky blue accents (`#0284C7`) and soft selection washes (`#E0F2FE`).
- **Floating Frosted-Glass HUD Toolbar**:
  - Floating pill toolbar anchored at the bottom-center of the document canvas.
  - Provides instant, uncluttered access to essential document controls:
    - **Previous Page** & **Next Page** navigation buttons (`chevron-left`, `chevron-right`).
    - **Page Number Indicator** (`Page X / Y`).
    - **Zoom Controls**: **Zoom Out**, clickable **Zoom %** button (single click resets to 100%), and **Zoom In**.
    - **Fit Toggle**: One-click toggle between **Fit Width** and **Fit Page**.
    - **Continuous Scroll Toggle**: One-click toggle between continuous multi-page scrolling and single-page view.
  - Smart suppression: Automatically hides during file drag-and-drop operations to keep canvas drop zones unobstructed.
- **Enhanced Document Elevation**:
  - Refreshed canvas page rendering with subtle outer boundary strokes and ambient drop shadows for clean document separation.
- **Modernized Tab & Navigation Chrome**:
  - Modern 8px tab corner radii with sleek top accent indicator bars on active tabs.
  - Enhanced window corner radii (12px) and menu corner radii (10px).

### Changed
- Major version bump to `v2.0.0` reflecting the complete user interface redesign and modern design system.

---

## [1.1.0] - 2026-10-30

### Added
- **Continuous View as Default**: New documents now default to Continuous vertical scrolling view instead of single-page snapped stepping. When scrolling down, pages scroll smoothly and continuously.
- **Drag-and-Drop Page Append**: Dragging and dropping a PDF file onto the bottom of the page or document area now automatically appends all of its pages to the end of the currently open document. Includes a visual drop target line and "+ Add to end of PDF" indicator badge.

### Fixed
- **Windows Non-Admin Installation**: Fixed installer configuration (WiX MSI and Inno Setup) allowing users without administrator permissions on corporate/work computers to install into user scope (`LocalAppData`) without installation getting stuck or disabled.
- **Milestone Banner Hidden**: Removed the development "Coming in milestone M7" banner from the "Edit a PDF" panel for a clean, distraction-free release UI.

### Changed
- **Branding & Code References**: Cleaned up remaining legacy upstream references, updated external URLs to point directly to the dpdf GitHub repository, and bumped workspace packages to `1.1.0`.

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
