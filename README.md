<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/brand/dpdf-logo-white.svg">
    <img alt="dpdf" src="docs/brand/dpdf-logo-horizontal.svg" width="280">
  </picture>
</p>

<h1 align="center">dpdf</h1>

<p align="center">
  <b>Fast, Private, and Completely Offline PDF Workbench</b><br>
  Read, organize, combine, split, and secure PDFs in a native app, written in pure Rust from the ground up.<br>
  Windows · macOS · Linux
</p>

<p align="center">
  <b>100% Offline · Zero Network Traffic · No Accounts · No Telemetry</b>
</p>

<p align="center">
  <a href="https://github.com/dbp-develop/dpdf/releases/download/v1.1.0/dpdf-1.1.0-setup.exe"><img src="https://img.shields.io/badge/Windows-Setup%20.exe-D13B3B?style=for-the-badge&logo=windows&logoColor=white" alt="Windows Setup .exe"></a>
  <a href="https://github.com/dbp-develop/dpdf/releases/download/v1.1.0/dpdf-1.1.0-windows-x64.msi"><img src="https://img.shields.io/badge/Windows-MSI%20Installer-1E222B?style=for-the-badge&logo=windows&logoColor=white" alt="Windows MSI"></a>
  <a href="https://github.com/dbp-develop/dpdf/releases/download/v1.1.0/dpdf-1.1.0-windows-x64-portable.zip"><img src="https://img.shields.io/badge/Windows-Portable%20.zip-4A5568?style=for-the-badge&logo=windows&logoColor=white" alt="Windows Portable"></a>
  <br>
  <a href="https://github.com/dbp-develop/dpdf/releases/download/v1.1.0/dpdf-1.1.0-macos-universal.dmg"><img src="https://img.shields.io/badge/macOS-Universal%20.dmg-D13B3B?style=for-the-badge&logo=apple&logoColor=white" alt="macOS DMG"></a>
  <a href="https://github.com/dbp-develop/dpdf/releases/download/v1.1.0/dpdf-1.1.0-macos-universal.zip"><img src="https://img.shields.io/badge/macOS-App%20.zip-1E222B?style=for-the-badge&logo=apple&logoColor=white" alt="macOS Zip"></a>
  <a href="https://github.com/dbp-develop/dpdf/releases/latest"><img src="https://img.shields.io/badge/Linux-All%20Versions-2D3748?style=for-the-badge&logo=linux&logoColor=white" alt="Linux Versions"></a>
</p>

---

## 📥 Direct Downloads (v1.1.0 Release)

| Platform | Type | Architecture / Scope | Direct Download Link |
| :--- | :--- | :--- | :--- |
| **Windows** | **GUI Setup (.exe)** | x64 (Dual-Scope User & Machine) | [**⬇️ dpdf-1.1.0-setup.exe**](https://github.com/dbp-develop/dpdf/releases/download/v1.1.0/dpdf-1.1.0-setup.exe) |
| **Windows** | **MSI Package (.msi)** | x64 (WiX Dual-Scope Enterprise) | [**⬇️ dpdf-1.1.0-windows-x64.msi**](https://github.com/dbp-develop/dpdf/releases/download/v1.1.0/dpdf-1.1.0-windows-x64.msi) |
| **Windows** | **Portable (.zip)** | x64 (Zero-install, USB ready) | [**⬇️ dpdf-1.1.0-windows-x64-portable.zip**](https://github.com/dbp-develop/dpdf/releases/download/v1.1.0/dpdf-1.1.0-windows-x64-portable.zip) |
| **macOS** | **Universal DMG (.dmg)** | Apple Silicon (M1–M4) + Intel (Drag to Applications) | [**⬇️ dpdf-1.1.0-macos-universal.dmg**](https://github.com/dbp-develop/dpdf/releases/download/v1.1.0/dpdf-1.1.0-macos-universal.dmg) |
| **macOS** | **Universal App (.zip)** | Apple Silicon + Intel (`dpdf.app`) | [**⬇️ dpdf-1.1.0-macos-universal.zip**](https://github.com/dbp-develop/dpdf/releases/download/v1.1.0/dpdf-1.1.0-macos-universal.zip) |
| **macOS** | **Universal CLI (.zip)** | Apple Silicon + Intel (`dpdf-cli`) | [**⬇️ dpdf-cli-1.1.0-macos-universal.zip**](https://github.com/dbp-develop/dpdf/releases/download/v1.1.0/dpdf-cli-1.1.0-macos-universal.zip) |
| **Linux** | **Releases & Packages** | x86_64, aarch64 | [**⬇️ View Linux Releases**](https://github.com/dbp-develop/dpdf/releases/latest) |

---

## Highlights

<table>
<tr>
<td width="33%" valign="top">

### 🔒 100% Private & Offline
Zero internet access. No analytics, no phone-home pings, no cloud processing, and no accounts required. Everything stays on your local machine.

</td>
<td width="33%" valign="top">

### ⚡ Fast & Native
Built in pure Rust with high-performance rendering. Instant startup, responsive scrolling, and efficient memory usage across desktop platforms.

</td>
<td width="33%" valign="top">

### 🛡️ Fearless Saves
Every save appends your modifications while keeping original bytes intact. Writes are atomic and verified, with deep undo/redo support.

</td>
</tr>
</table>

---

## Features

* **High-Fidelity Reader:** Clean text rendering with complete font metrics, kerning, ligatures, right-to-left scripts, vertical Japanese, and smooth zooming.
* **Organize Pages:** Visual card-like page grid. Rotate, delete, extract, reorder, and insert pages from other PDF files.
* **Combine & Split:** Merge multiple documents into a single PDF with preserved bookmarks, or split documents into parts by page count or ranges.
* **Interactive Forms & Annotations:** Fill form fields, highlight text, add sticky notes, draw markup, and export filled documents.
* **Standard PDF Security:** Open and edit protected documents with full support for 40-bit RC4 through AES-256 encryption and permission handlers.
* **Dual-Scope Windows Installer:**
  * **Current User (No Admin Rights):** Installs directly into your user profile (`%LOCALAPPDATA%\Programs\dpdf`). No UAC elevation required.
  * **System-Wide:** Installs to `C:\Program Files\dpdf` for all computer accounts with administrator approval.
* **Portable Edition:** Extract and run anywhere (e.g. from a USB drive) without installing or needing admin rights. Keeps all data in a local `dpdfData` folder with zero system registry footprint.
* **Headless CLI (`dpdf-cli`):** Batch inspect, extract text, organize, render pages to PNG, and script PDF tasks without opening a window.

---

## Architecture

`dpdf` is organized into clean, modular Rust crates:

| Module | Function |
|---|---|
| `filters` | PDF stream filters (Flate, LZW, ASCII85, RunLength, Predictors) |
| `crypt` | Standard security handler (RC4, AES-128, AES-256, permissions) |
| `cos` | Low-level PDF object model, fault-tolerant parser, incremental writer |
| `organize` | Page manipulation, combining, extraction, and splitting |
| `fonts` | Font metrics and standard 14 font encodings |
| `annot` | PDF annotation appearance streams, highlights, and note threads |
| `forms` | Interactive AcroForms field handling and appearance generation |
| `render` | Page rasterization and reading-order text extraction |
| `engine` | High-level facade for sessions, undo/redo history, and atomic saves |
| `ui-egui` | Cross-platform desktop interface built with egui |

---

## Building from Source

### Prerequisites
* Rust toolchain (stable)
* Platform build tools:
  * **Windows:** Visual Studio C++ Build Tools
  * **macOS:** Xcode Command Line Tools
  * **Linux:** `build-essential`, `libwayland-dev`, `libxkbcommon-dev`

### Compiling
```sh
# Build desktop app
cargo build --release -p dpdf

# Build command-line tool
cargo build --release -p dpdf-cli
```

The resulting binaries will be in `target/release/`.

---

## License & Attribution

`dpdf` is licensed under the **MIT License** or the **Apache License, Version 2.0**, at your option.

Derived from the PdfCraft engine:
Copyright (c) 2026 ArtCraft Team and the PdfCraft contributors.
Portions copyright (c) 2026 Divya and dpdf contributors.

Third-party open-source components:
* **Inter** & **JetBrains Mono** typefaces: SIL Open Font License 1.1
* **Lucide** icons: ISC License
* **Foxit standard font metrics**: BSD-3-Clause

See `NOTICE` and `ATTRIBUTION.md` for complete third-party license texts.
