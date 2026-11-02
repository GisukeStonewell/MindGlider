use mindglider::{
    ai::{self, CliSettings, Provider, Snapshot},
    model::Document,
};
use std::{sync::atomic::Ordering, time::Duration};
fn run(provider: Provider, mode: &str, cancel: bool) -> Result<ai::EditPlan, String> {
    let folder = std::path::Path::new("artifacts/tmp");
    std::fs::create_dir_all(folder).unwrap();
    let dir = tempfile::tempdir_in(folder).unwrap();
    let doc = Document::default();
    let snapshot = Snapshot::capture(&doc, &doc.root, "test fixture").unwrap();
    let mut settings = CliSettings {
        provider,
        model: mode.into(),
        codex: env!("CARGO_BIN_EXE_mindglider-cli-fixture").into(),
        claude: env!("CARGO_BIN_EXE_mindglider-cli-fixture").into(),
        timeout_secs: 1,
        ..Default::default()
    };
    if mode == "custom-prompt" {
        settings.system_prompt =
            "CUSTOM FIXED PROMPT\nRead the reference file if instructed.".into();
    }
    let task = ai::spawn(
        settings,
        snapshot,
        "fixture".into(),
        dir.path(),
        eframe::egui::Context::default(),
    );
    if cancel {
        task.cancel.store(true, Ordering::Relaxed);
    }
    let result = task
        .receiver
        .recv_timeout(Duration::from_secs(8))
        .expect("CLI process must terminate");
    assert_eq!(
        std::fs::read_dir(dir.path()).unwrap().count(),
        0,
        "temporary files must be cleaned"
    );
    result
}
#[test]
fn codex_subprocess() {
    assert!(run(Provider::Codex, "ok", false).is_ok());
}
#[test]
fn claude_subprocess() {
    assert!(run(Provider::Claude, "ok", false).is_ok());
}
#[test]
fn subprocess_auth_failure() {
    assert!(
        run(Provider::Codex, "auth-error", false)
            .unwrap_err()
            .contains("authentication required")
    );
}
#[test]
fn subprocess_invalid_json() {
    assert!(run(Provider::Codex, "invalid", false).is_err());
}
#[test]
fn subprocess_timeout() {
    assert!(
        run(Provider::Codex, "timeout", false)
            .unwrap_err()
            .contains("タイムアウト")
    );
}
#[test]
fn subprocess_cancel() {
    assert!(
        run(Provider::Claude, "timeout", true)
            .unwrap_err()
            .contains("キャンセル")
    );
}

#[test]
fn custom_fixed_prompt_reaches_both_clis() {
    for provider in [Provider::Codex, Provider::Claude] {
        assert!(run(provider, "custom-prompt", false).is_ok());
    }
}

#[test]
fn metadata_handshakes_do_not_start_inference() {
    std::fs::create_dir_all("artifacts/tmp").unwrap();
    let dir = tempfile::tempdir_in("artifacts/tmp").unwrap();
    for provider in [Provider::Codex, Provider::Claude] {
        let settings = CliSettings {
            provider,
            codex: env!("CARGO_BIN_EXE_mindglider-cli-fixture").into(),
            claude: env!("CARGO_BIN_EXE_mindglider-cli-fixture").into(),
            ..Default::default()
        };
        let c = mindglider::cli_catalog::discover(&settings, dir.path());
        assert!(c.warning.is_empty(), "{}", c.warning);
        assert_eq!(c.models.len(), 1);
        assert_eq!(c.models[0].id, "fixture-model");
        assert_eq!(c.models[0].efforts, ["high"]);
    }
}
