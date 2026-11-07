"""Check the public portable distribution, without extracting executable files."""
import hashlib
from pathlib import Path, PurePosixPath
import sys
import zipfile

archive = Path(sys.argv[1])
with zipfile.ZipFile(archive) as bundle:
    names = set(bundle.namelist())
    required = {
        "mindglider.exe", "README.md", "README.en.md", "NOTICE.md", "LICENSE.txt",
        "examples/Example.mm", "assets/freeplane-icons/README.md",
        "licenses/epaint_default_fonts-0.31.1/fonts/OFL.txt",
        "licenses/epaint_default_fonts-0.31.1/fonts/UFL.txt",
    }
    assert required <= names, f"Missing files: {sorted(required - names)}"
    forbidden = {"source", "src", "scripts", "tests", ".cargo", ".github", ".tools", ".git", "target", "data", "artifacts", "__pycache__"}
    for name in names:
        path = PurePosixPath(name)
        assert not path.is_absolute() and ".." not in path.parts, name
        assert not forbidden.intersection(path.parts), f"Private/build directory: {name}"
        assert path.name not in {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml"}, name
        assert path.suffix.lower() not in {".rs", ".ps1", ".py", ".ttf", ".otf", ".pdb", ".pem", ".key"}, name
    assert "Version 3, 29 June 2007" in bundle.read("LICENSE.txt").decode("utf-8")
    example = bundle.read("examples/Example.mm").decode("utf-8")
    # Freeplane detects its native map by the first characters, before XML parsing.
    assert example.startswith('<map version="freeplane ')
    assert "C:\\Users\\" not in example and "file:///C:" not in example
digest = hashlib.file_digest(archive.open("rb"), "sha256").hexdigest()
assert archive.with_name("SHA256SUMS.txt").read_text().strip() == f"{digest}  {archive.name}"
print(f"Verified {len(names)} ZIP entries, absence of application source, licenses and SHA-256")
