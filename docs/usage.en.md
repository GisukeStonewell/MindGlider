# User guide

[日本語](usage.ja.md) | [README](../README.en.md)

## Updating while MindGlider is running

Extract the new executable from a trusted release into a separate directory, then run the bundled PowerShell script:

```powershell
.\update.ps1 -SourceExe 'C:\Downloads\MindGlider\mindglider.exe' -InstallDir "$env:LOCALAPPDATA\Programs\MindGlider"
```

The script keeps the previous executable in a `backup-...` directory. A running version that supports update detection will show an update notice. Choose **Restart now** to record recovery data for unsaved edits and restore the open maps in the new process. Wait for any active AI task to finish first. Choose **Later** to continue working.

## Editing and navigation

Tab adds a child and Enter adds a sibling. Both open the new node for editing. F2 or double-click edits an existing node. While editing, Shift+Enter inserts a line break, Enter commits, and Esc cancels. IME confirmation Enter is handled separately.

Up and Down prefer visible nodes at the same depth on the current side of the map. If none is available in that direction, the nearest eligible node vertically is selected. At an edge, hold the arrow key to scroll. Delete selects the next sibling, the previous sibling, or the parent, in that order. Drag the background with the left mouse button to pan and use the wheel to zoom.

Drag a branch onto a node's outward edge to attach it as a child; use the upper/lower region for insertion before/after a sibling. The root's left/right half chooses the side. The branch follows the pointer translucently; valid/invalid destinations are blue/red. Esc or an invalid drop cancels. One Undo reverts the move.

## Icons and properties

Open/collapse the left icon panel and right properties panel using their edge handles or the View menu. The icon panel starts closed and both states are remembered. Click an icon to toggle it on selected nodes. Ctrl+F2 opens searchable keyboard selection: arrows, Enter and Esc. Node backgrounds also work on fork/underline nodes, covering core text rather than images or notes.

The properties panel provides text styling, details, notes, node/edge style, icons/progress, images, links, clouds and connectors. Rich text is limited to the supported formatting subset. Unsupported HTML can be simplified only through an explicit editing action.

## AI editing

To rename an existing node, select it, press F12, and choose **Change title only**. Enter runs the request, with optional guidance about the wording. MindGlider uses the subtree rooted at the selected node's parent as context, including siblings and their descendants. For the map root, it uses the root's subtree. The AI returns a new title, and MindGlider applies it to the selected node. One Undo restores the old title.

Use **Edit subtree** to add, delete, move, reorder, or edit multiple nodes. An addition whose ID and parent match an existing node appears as a rename candidate. To keep the existing node and add another, change that candidate to a new addition. MindGlider assigns a fresh ID.

Install and sign in to Codex or Claude Code separately. Choose it in Settings, then select a branch and press F12. Enter instructions in the editable history combo, press Enter to run and Esc to cancel or close. The last ten exact instructions are persisted in `data/ai-history.json`. Review the changes in subtree mode, then confirm to apply them as one Undoable transaction. Changes to the target branch prevent application of an outdated review.

The fixed prompt is configurable and has localized defaults. Model/effort candidates depend on what the installed CLI exposes through its settings, caches or metadata commands. You can enter a value directly. MindGlider does not call AI APIs directly. Instructions and selected text/notes/details are passed to the CLI. The CLI may also read explicitly requested local knowledge files. Authentication, network behavior and charges depend on the CLI/provider.

## AI diagnostic logs

MindGlider writes daily diagnostic logs to `data/logs/mindglider_YYYY-MM-DD.log`. Use `request_id` to follow one F12 request from CLI arguments and response through tree application. Failure records include a stable `code` and any available OS or HTTP code.

Edit the generated `data/logging.json` and restart MindGlider. The defaults are `level: "info"`, `rotation: "daily"`, `retention_days: 3`, and `include_response_content: false`. Supported levels are `off`, `fatal`, `error`, `warning` (or `warn`), `info`, `debug`, and `trace`. Daily rotation uses the local calendar and keeps today and the previous two days by default. Set `rotation` to `never` to append to `mindglider.log` without age-based cleanup.

To record response content, set `include_response_content` to `true` and choose `debug` or `trace`. Each output stream is limited to its first 64 KiB and may include map or instruction text.

## Files and recovery

Save with Ctrl+S or Ctrl+Shift+S. Freeplane-compatible `.mm` is the main format. UTF-8/numeric references and the leading `<map version="…"` header are used for compatibility. Unknown XML and unedited HTML are retained where possible, but full equivalence with every Freeplane feature is not guaranteed. Clones and encrypted branches are edit-protected.

Recovery data are saved separately every 30 seconds. Normal saves retain ten backup generations in `.mindglider-backup`. An externally modified file is not silently overwritten. Settings and GUI state live beside the executable in `data`; use `MINDGLIDER_DATA_DIR` to override the location.

## Languages and export

Settings supports Japanese/English, custom shortcuts and per-field/all-settings resets. Edit `locales/ja.json` or `locales/en.json` and use Reload translations. Preserve keys and interpolation placeholders. Defaults fill omitted keys. `MINDGLIDER_LOCALES_DIR` overrides the catalog directory.

Export the whole map or selected branch to TXT, Markdown, OPML, PNG, JPEG, SVG, PDF or HTML. PDF options include paper, orientation, margins and fit/tiled output. Printing opens the PDF in the default viewer; final printer selection happens there. See [the feature overview](features.en.md) for limits.
