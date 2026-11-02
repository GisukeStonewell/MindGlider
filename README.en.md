# MindGlider

English | [日本語](README.md)

**MindGlider is a Windows 11 mind-mapping application ported and reimplemented from Freeplane in Rust with AI assistance, focusing on a selected subset of features.** It uses Rust, egui/eframe and wgpu and does not require Java.

> **This is the first public preview. Even the author who performed the port has not tried every feature or combination of operations.** Bugs and compatibility gaps may remain. Try it on copies of important maps.

## Download

Download `MindGlider-0.9.0-windows-x64.zip` from [GitHub Releases](https://github.com/GisukeStonewell/MindGlider/releases), extract it to a writable directory and run `mindglider.exe`. Neither Java nor Rust is needed to run the application. Verify the ZIP against the accompanying `SHA256SUMS.txt` if desired.

The supported target is **Windows 11 x64**. The initial release is marked as a **Pre-release**. AI CLIs are optional for ordinary mind-map editing.

## Features

- Reads and writes Freeplane-compatible `.mm` files, aiming to preserve unsupported XML information and unedited HTML.
- Node editing, hierarchy and sibling changes, left/right branches, folding, multiple selection, dragging, Undo/Redo, and search/replace.
- Formatting, partial text styling, notes/details, local images, Freeplane-derived icons, connectors and clouds.
- Conditional styles and filters, tabs, session restoration, recovery data and backups.
- TXT, Markdown, OPML, PNG, JPEG, SVG, PDF and HTML export.
- Japanese and English UI, with user-editable external JSON translation files.

**This is not a complete Freeplane port or a guarantee of full round-trip compatibility.** Editing unsupported structures such as clones and encrypted branches is restricted. See the [feature matrix](FEATURES.md) and [English overview](docs/features.en.md). This is an independent project, not an official product or port of Freeplane or FreeMind.

## AI tree editing through local CLIs

MindGlider adds the ability to **invoke a locally installed Codex or Claude Code CLI and update the selected node and its descendants with AI**.

1. Install your preferred CLI separately and sign in using its normal authentication flow.
2. Select the provider, executable, model and other options in MindGlider settings. The fixed prompt is editable.
3. Select a branch, press **F12**, enter instructions, and press Enter to run.
4. A valid result is applied atomically after scope, structure and conflict checks. **One Undo restores the entire change.**

**MindGlider does not implement direct calls to AI-provider APIs.** There is no API-key configuration in MindGlider. Network communication, authentication, service availability and charges are governed by the CLI and its provider.

The selected branch's text, notes, details and your instructions are passed to the CLI. Images, unselected branches and the contents of linked local files are not passed by default. A CLI may read local knowledge files explicitly requested in your instructions. Settings, prompt history and recovery data are stored in `data`; check them for private information before sharing.

[English user guide](docs/usage.en.md) | [CLI options (Japanese)](docs/cli-options.md) | [Feature matrix](FEATURES.md)

## Basic controls

| Action | Shortcut / gesture |
|---|---|
| Add child / sibling | Tab / Enter |
| Edit text on a node | F2 / double-click |
| New line / commit / cancel | Shift+Enter / Enter / Esc |
| Select an icon | Ctrl+F2 |
| AI-edit selected branch | F12 |
| Undo / Redo | Ctrl+Z / Ctrl+Y |
| Save / Save as | Ctrl+S / Ctrl+Shift+S |
| Search / command palette | Ctrl+F / Ctrl+P |
| Pan / zoom | Left-drag background / mouse wheel |

Settings and recovery data live in `data` beside the executable. Override this with `MINDGLIDER_DATA_DIR`. The UI uses installed Windows fonts such as Segoe UI Variable; system font files are not redistributed.

## Build from source

On Windows, install Visual Studio C++ Build Tools and the Windows SDK, the Rust version specified in `rust-toolchain.toml`, and Python 3.11 or newer.

```powershell
cargo test --locked --tests
cargo clippy --locked --all-targets -- -D warnings
./scripts/build.ps1 -Package
```

Packages are written to `dist`. A Freeplane checkout and a JDK are not needed for ordinary builds. They are needed only for reimporting upstream assets or optional checks against the upstream Java implementation.

## License and credits

MindGlider is licensed under **GPL-3.0-or-later**. See [LICENSE.txt](LICENSE.txt) and [NOTICE.md](NOTICE.md). Notices and licenses for Freeplane-derived graphics, translations and test maps, and for Rust dependencies, are retained separately. See [THIRD_PARTY.md](THIRD_PARTY.md) and [icon provenance](assets/freeplane-icons/README.md).

Copyright © 2026 GisukeStonewell and MindGlider contributors.

## Issues and contributions

Report your version, reproduction steps and Windows display scaling in [Issues](https://github.com/GisukeStonewell/MindGlider/issues). Use minimal, sanitized sample maps rather than personal maps, prompt history or CLI credentials. See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md).
