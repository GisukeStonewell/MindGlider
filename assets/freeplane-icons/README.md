# Freeplane icon resources

These images are derived from the cloned Freeplane repository at commit
`25f7e70756c7d93b11ee9061a4134f61324be873`.

- Source: `freeplane/src/viewer/resources/images/icons/**/*.svg` (201 SVG files).
- `original/` contains unmodified images, including author and license metadata.
- `simplified/` contains MindGlider derivatives: opaque gradients become flat colors, transparency gradients are preserved,
  filter effects and editor metadata are removed, SVG IDs are namespaced, and
  the original paths, silhouettes and BUILTIN identifiers are preserved.
- `manifest.json` records the source paths, IDs, and SHA-256 hashes of originals.
- `Freeplane-LICENSE.txt` contains the upstream project's GPL v2 license text.
  Preserve attribution and any per-image notices in the original SVG metadata.
- Display names and ordering are imported from Freeplane's icon group properties
  and Japanese/English translation resources. MindGlider translations remain editable.

Reproduce from the workspace clone with `python scripts/import_freeplane_icons.py`
followed by `cargo fmt --all`. No network or font downloads are used.
