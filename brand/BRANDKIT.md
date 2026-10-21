# dpdf Brand Kit

## 1. Identity Overview
* **Application Name:** `dpdf` (or `dPDF`)
* **Tagline:** Fast, private PDF workbench
* **Positioning:** Lightweight, zero-telemetry, pure native PDF reader and editor for personal, family, and internal organizational use.

---

## 2. Color Palette

| Role | Color Name | Hex | RGB | Description / Usage |
| :--- | :--- | :--- | :--- | :--- |
| **Primary Accent** | Ruby Crimson | `#E11D48` | `rgb(225, 29, 72)` | Brand mark, primary action buttons, active tab indicators, PDF highlight |
| **Secondary Accent** | Rose Light | `#F43F5E` | `rgb(244, 63, 94)` | Hover states, gradients, active selection borders |
| **Dark Base (UI)** | Midnight Slate | `#0F172A` | `rgb(15, 23, 42)` | Dark mode background, title bars, text in light mode |
| **Dark Surface** | Deep Charcoal | `#1E293B` | `rgb(30, 41, 59)` | Sidebar panels, tool dialogs, card backgrounds in dark mode |
| **Light Base** | Snow Slate | `#F8FAFC` | `rgb(248, 250, 252)` | Document background canvas, light mode UI base |
| **Borders & Lines** | Steel Gray | `#334155` | `rgb(51, 65, 85)` | Panel splitters, tab dividers, modal borders |
| **Status / Success** | Emerald | `#10B981` | `rgb(16, 185, 129)` | Atomic save confirmations, encryption verified badges |

---

## 3. Typography Selection

PdfCraft already bundles high-grade open-source fonts under the SIL Open Font License 1.1:

1. **User Interface & Headings:**
   * **Font:** **Inter** (Regular, Medium, Semi-Bold, Bold)
   * **Why:** High legibility at small sizes (toolbar labels, tab headers, nested bookmark trees), clean modern geometric sans-serif.
   * **Location in repo:** `assets/fonts/Inter-*.ttf`

2. **Metadata, Hex & Automation:**
   * **Font:** **JetBrains Mono**
   * **Why:** Designed specifically for developers and technical readability. Ideal for PDF object streams, command palette shortcuts, and document properties.
   * **Location in repo:** `assets/fonts/JetBrainsMono-*.ttf`

---

## 4. Logo Assets Generated

The following SVG assets are ready in `brand/`:
* [dpdf-icon.svg](file:///home/antigravity/pdf/brand/dpdf-icon.svg) — 512x512 Master App Icon (Squircle container, folded document sheet, integrated 'd' bookmark ribbon).
* [dpdf-logo-horizontal.svg](file:///home/antigravity/pdf/brand/dpdf-logo-horizontal.svg) — Light background horizontal wordmark.
* [dpdf-logo-white.svg](file:///home/antigravity/pdf/brand/dpdf-logo-white.svg) — Dark background horizontal wordmark.

---

## 5. File Mapping (Where to Swap in `storytold/pdfcraft`)

When you fork the repository, swap these assets:

| Original Upstream File | Replace With | Purpose |
| :--- | :--- | :--- |
| `docs/brand/artcraft-logo.svg` | `brand/dpdf-logo-horizontal.svg` | Light theme header / documentation |
| `docs/brand/artcraft-logo-white.svg` | `brand/dpdf-logo-white.svg` | Dark theme header |
| `docs/brand/artcraft-mark.svg` | `brand/dpdf-icon.svg` | Small mark icon |
| `assets/app-icon/` (`.ico`, `.png`, `.icns`) | Converted from `brand/dpdf-icon.svg` | Operating system desktop app icon |
| `Cargo.toml` (`name`, `package.metadata`) | `dpdf` / `dpdf-cli` | Binary and package names |
