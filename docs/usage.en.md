# User guide

[日本語](usage.ja.md) | [README](../README.en.md)

## Editing and navigation

Use Tab/Enter for a child/sibling, F2 or double-click to edit on a node, Shift+Enter for a line break, Enter to commit and Esc to cancel. IME confirmation Enter is handled separately. Arrow keys navigate visible nodes, including vertical moves across levels. At an edge, hold the arrow key to scroll. Drag the background with the left mouse button to pan; use the wheel to zoom.

Drag a branch onto a node's outward edge to attach it as a child; use the upper/lower region for insertion before/after a sibling. The root's left/right half chooses the side. The branch follows the pointer translucently; valid/invalid destinations are blue/red. Esc or an invalid drop cancels. One Undo reverts the move.

## Icons and properties

Open/collapse the left icon panel and right properties panel using their edge handles or the View menu. The icon panel starts closed and both states are remembered. Click an icon to toggle it on selected nodes. Ctrl+F2 opens searchable keyboard selection: arrows, Enter and Esc. Node backgrounds also work on fork/underline nodes, covering core text rather than images or notes.

The properties panel provides text styling, details, notes, node/edge style, icons/progress, images, links, clouds and connectors. Rich text is limited to the supported formatting subset. Unsupported HTML can be simplified only through an explicit editing action.

## AI editing

Install/sign in to Codex or Claude Code separately. Choose it in Settings, then select a branch and press F12. Enter instructions in the editable history combo, press Enter to run and Esc to cancel/close. The last ten exact instructions are persisted in `data/ai-history.json`. A validated result is automatically applied as one Undoable transaction. Conflicting edits to the selected branch prevent application.

The fixed prompt is configurable and has localized defaults. Model/effort candidates depend on what the installed CLI exposes through its settings, caches or metadata commands; you can enter a value directly. MindGlider does not call AI APIs directly. Instructions and selected text/notes/details are passed to the CLI; explicitly requested local knowledge files may also be read. Authentication, network behavior and charges depend on the CLI/provider.

## Files and recovery

Save with Ctrl+S or Ctrl+Shift+S. Freeplane-compatible `.mm` is the main format. UTF-8/numeric references and the leading `<map version="…"` header are used for compatibility. Unknown XML and unedited HTML are retained where possible, but full equivalence with every Freeplane feature is not guaranteed. Clones and encrypted branches are edit-protected.

Recovery data are saved separately every 30 seconds. Normal saves retain ten backup generations in `.mindglider-backup`. An externally modified file is not silently overwritten. Settings and GUI state live beside the executable in `data`; use `MINDGLIDER_DATA_DIR` to override the location.

## Languages and export

Settings supports Japanese/English, custom shortcuts and per-field/all-settings resets. Edit `locales/ja.json` or `locales/en.json` and use Reload translations. Preserve keys and interpolation placeholders. Defaults fill omitted keys. `MINDGLIDER_LOCALES_DIR` overrides the catalog directory.

Export the whole map or selected branch to TXT, Markdown, OPML, PNG, JPEG, SVG, PDF or HTML. PDF options include paper, orientation, margins and fit/tiled output. Printing opens the PDF in the default viewer; final printer selection happens there. See [the feature overview](features.en.md) for limits.
