use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};
fn resource_compiler() -> PathBuf {
    if let Some(path) = env::var_os("RC") {
        return path.into();
    }
    if let Some(path) = env::var_os("PATH").and_then(|p| {
        env::split_paths(&p)
            .map(|p| p.join("rc.exe"))
            .find(|p| p.is_file())
    }) {
        return path;
    }
    let root = env::var_os("WindowsSdkDir")
        .map(PathBuf::from)
        .map(|p| p.join("bin"))
        .unwrap_or_else(|| {
            PathBuf::from(
                env::var_os("ProgramFiles(x86)").unwrap_or_else(|| "C:/Program Files (x86)".into()),
            )
            .join("Windows Kits/10/bin")
        });
    let mut paths = std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path().join("x64/rc.exe"))
        .filter(|p| p.is_file())
        .collect::<Vec<_>>();
    paths.sort();
    paths.pop().expect("Windows SDK rc.exe was not found. Install the Windows SDK or set RC to rc.exe's full path.")
}
fn main() {
    for path in ["assets/windows.rc", "assets/icons/mindglider.ico"] {
        println!("cargo:rerun-if-changed={path}");
    }
    println!("cargo:rerun-if-env-changed=RC");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    assert_eq!(
        env::var("CARGO_CFG_TARGET_ENV").as_deref(),
        Ok("msvc"),
        "Windows distribution requires the MSVC toolchain"
    );
    let out = Path::new(&env::var_os("OUT_DIR").unwrap()).join("mindglider.res");
    let status = Command::new(resource_compiler())
        .current_dir(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .args(["/nologo", "/fo"])
        .arg(&out)
        .arg("assets/windows.rc")
        .status()
        .expect("Failed to launch Windows resource compiler");
    assert!(
        status.success(),
        "Failed to compile MindGlider's Windows icon resource"
    );
    println!("cargo:rustc-link-arg-bin=mindglider={}", out.display());
}
