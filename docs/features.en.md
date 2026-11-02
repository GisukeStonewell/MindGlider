# Feature overview

[Japanese feature matrix and IDs](../FEATURES.md) | [English README](../README.en.md)

The implemented subset includes core editing; basic and partial text styles; notes/details; local images; Freeplane icons and progress; connectors and clouds; links; conditional styles/filters; search and replace; shortcut customization; tabs/session restoration; recovery/backups; and text/document/image exports. AI editing through Codex/Claude Code CLIs is an additional MindGlider feature.

Compatibility is partial. Unsupported XML/unedited HTML are preserved where possible, but this is not a complete implementation of every Freeplane behavior. Protected clone/encrypted structures are not silently converted to regular nodes. Unsupported HTML requires explicit simplification before editing. Unknown icon IDs are preserved with a fallback display.

The first public preview has not been manually tested across all features or combinations, including by its author. Automated tests and selected visual checks do not imply complete validation. Read the release notes and work on copies of important maps.
