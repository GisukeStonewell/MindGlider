# MindGlider

English | [日本語](README.md)

**MindGlider is a Windows 11 mind-mapping application ported and reimplemented from Freeplane in Rust with AI assistance, focusing on a selected subset of features.** It uses Rust, egui/eframe and wgpu and does not require Java.

> **This is a public preview.** Bugs and compatibility gaps may remain. Try it on copies of important maps.

## Download

Download `MindGlider-0.11.1-windows-x64.zip` from [GitHub Releases](https://github.com/GisukeStonewell/MindGlider/releases/tag/v0.11.1), extract it to a writable directory and run `mindglider.exe`. Neither Java nor Rust is needed to run the application. Verify the ZIP against the accompanying `SHA256SUMS.txt`.

Choose one of ten palettes under **View → Theme**. Hover over a theme to preview it and click to save your choice. The corresponding source is available through the **Source code** downloads on the same release page.

The supported target is **Windows 11 x64**. AI CLIs are optional for ordinary mind-map editing. [v0.11.1 release notes](docs/releases/v0.11.1.md) | [Updating an existing install](docs/usage.en.md#updating-while-mindglider-is-running)

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
3. Select a node, press **F12**, then choose **Edit subtree** or **Change title only**.
4. With **Edit subtree**, review additions, deletions and moves before applying them together. For a rename candidate, you can switch between renaming the existing node and adding a new one. **Change title only** changes the selected node's text. **One Undo restores the entire change** in either mode.

**MindGlider does not implement direct calls to AI-provider APIs.** There is no API-key configuration in MindGlider. Network communication, authentication, service availability and charges are governed by the CLI and its provider.

**Edit subtree** passes the selected branch's text, notes, details and your instructions to the CLI. **Change title only** provides context from the subtree rooted at the selected node's parent. Images and linked local file contents are omitted from the request. A CLI may read local knowledge files explicitly requested in your instructions. Diagnostic logs record CLI execution and whether changes were applied. [Diagnostic log settings](docs/usage.en.md#ai-diagnostic-logs)

[English user guide](docs/usage.en.md) | [CLI options (Japanese)](docs/cli-options.md) | [Feature matrix](FEATURES.md)

## Basic controls

| Action | Shortcut / gesture |
|---|---|
| Add child / sibling and start editing | Tab / Enter |
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
