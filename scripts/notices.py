"""Collect licenses from the exact Cargo.lock dependency sources already on disk."""
import json
from pathlib import Path
import shutil
import subprocess

root = Path(__file__).resolve().parent.parent
metadata = json.loads(subprocess.check_output(
    ["cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", "x86_64-pc-windows-msvc"],
    cwd=root, encoding="utf-8"))
active = {node["id"] for node in metadata["resolve"]["nodes"]}
packages = [p for p in metadata["packages"] if p["id"] in active and p["name"] != "mindglider"]
out = root / "licenses"
out.mkdir(exist_ok=True)
rows = ["# Third-party components", "", "Versions are pinned in Cargo.lock. License texts supplied by the packages are in licenses/.", "", "Windows system fonts are loaded from the user's Windows installation and are not redistributed.", "", "| Package | Version | Declared license | License files |", "|---|---|---|---|"]
rows[2:2] = ["Freeplane由来のアイコンと翻訳には元の権利表記を保持しています。出典と変更内容は assets/freeplane-icons/README.md を参照してください。", "Freeplane icons and translated labels retain their upstream attribution. See assets/freeplane-icons/README.md.", "", "依存ライブラリのバージョンは Cargo.lock に固定し、取得できたライセンス文書を licenses/ に収録しています。OSのフォントは再配布しません。egui内蔵フォントのライセンスは licenses/epaint_default_fonts-0.31.1/fonts/ にあります。", ""]
for pkg in sorted(packages, key=lambda p: (p["name"], p["version"])):
    if pkg["name"] == "mindglider":
        continue
    source = Path(pkg["manifest_path"]).parent
    target = out / f'{pkg["name"]}-{pkg["version"]}'
    candidates = [p for p in source.iterdir() if p.is_file() and p.name.lower().startswith(("license", "copying", "notice", "copyright"))]
    if pkg.get("license_file"):
        p = source / pkg["license_file"]
        if p.is_file() and p not in candidates:
            candidates.append(p)
    for folder in (source / "LICENSES", source / "licenses"):
        if folder.is_dir():
            candidates.extend(p for p in folder.rglob("*") if p.is_file())
    if pkg["name"] == "epaint_default_fonts":
        candidates.extend(p for p in (source / "fonts").glob("*.txt"))
    names = []
    for p in candidates:
        target.mkdir(exist_ok=True)
        relative = p.relative_to(source)
        dest = target / relative
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(p, dest)
        names.append(str(relative).replace("\\", "/"))
    rows.append(f'| {pkg["name"]} | {pkg["version"]} | {pkg.get("license") or "See upstream"} | {", ".join(names) or "Declared in Cargo manifest"} |')
(root / "THIRD_PARTY.md").write_text("\n".join(rows) + "\n", encoding="utf-8")
print(f"Recorded {len(packages)} dependencies")
