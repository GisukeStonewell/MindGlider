# MindGlider icon

Original vector artwork for MindGlider: a paper plane with connected nodes as speed trails.
The SVG is the editable source; no external artwork or fonts are used.

Regenerate the PNG and Windows ICO with:

```powershell
cargo run --release --example generate_icon
```

The ICO contains 16, 24, 32, 48, 64, 128 and 256 pixel 32-bit PNG frames.
The application window/taskbar use the embedded PNG. `build.rs` invokes the Windows SDK resource compiler to embed the ICO and version metadata in the EXE.
Install the Windows SDK, or set `RC` to the full path of its `rc.exe`, when building on Windows.
Keep `assets/windows.rc` version metadata in sync with Cargo.toml when releasing.

Validate the EXE after building:

```powershell
python scripts/verify_windows_icon.py target/release/mindglider.exe
```
