use mindglider::{
    ai::{self, CliSettings, Provider, Snapshot},
    model::{Document, History},
};
use serde_json::{Value, json};
use std::{fs, time::Duration};

#[test]
fn traces_execution_and_apply_results_using_the_same_request_id() {
    fs::create_dir_all("artifacts/tmp").unwrap();
    let dir = tempfile::tempdir_in("artifacts/tmp").unwrap();
    let data = dir.path().canonicalize().unwrap();
    fs::write(
        data.join("logging.json"),
        br#"{"level":"debug","rotation":"daily","retention_days":3,"include_response_content":true}"#,
    )
    .unwrap();
    let logger = mindglider::logging::init(&data).unwrap();
    let mut history = History::new(Document::default());
    let root = history.doc.root.clone();
    let instruction = "private fixture instruction";
    let snapshot = Snapshot::capture(&history.doc, &root, instruction).unwrap();
    let launch = |program: &str, mode: &str| {
        ai::spawn(
            CliSettings {
                codex: program.into(),
                model: mode.into(),
                ..Default::default()
            },
            snapshot.clone(),
            "fixture tab".into(),
            &data,
            eframe::egui::Context::default(),
        )
    };
    let fixture = env!("CARGO_BIN_EXE_mindglider-cli-fixture");
    let task = launch(fixture, "ok");
    let plan = task
        .receiver
        .recv_timeout(Duration::from_secs(8))
        .unwrap()
        .unwrap();
    ai::apply_traced(
        &mut history,
        &snapshot,
        &plan,
        &task.request_id,
        task.started,
    )
    .unwrap();
    let changed = launch(fixture, "ok");
    let plan = changed
        .receiver
        .recv_timeout(Duration::from_secs(8))
        .unwrap()
        .unwrap();
    history.doc.add(&root, "changed in the meantime").unwrap();
    assert!(
        ai::apply_traced(
            &mut history,
            &snapshot,
            &plan,
            &changed.request_id,
            changed.started
        )
        .is_err()
    );
    let invalid = launch(fixture, "invalid");
    assert!(
        invalid
            .receiver
            .recv_timeout(Duration::from_secs(8))
            .unwrap()
            .is_err()
    );
    let rejected = launch(fixture, "auth-error");
    assert!(
        rejected
            .receiver
            .recv_timeout(Duration::from_secs(8))
            .unwrap()
            .is_err()
    );
    let provider_rejection = ai::spawn(
        CliSettings {
            provider: Provider::Claude,
            claude: fixture.into(),
            model: "provider-rejection".into(),
            ..Default::default()
        },
        snapshot.clone(),
        "fixture tab".into(),
        &data,
        eframe::egui::Context::default(),
    );
    assert!(
        provider_rejection
            .receiver
            .recv_timeout(Duration::from_secs(8))
            .unwrap()
            .is_err()
    );
    let missing = launch(data.join("missing.exe").to_str().unwrap(), "ok");
    assert!(
        missing
            .receiver
            .recv_timeout(Duration::from_secs(8))
            .unwrap()
            .is_err()
    );
    logger.shutdown();

    let logs = fs::read_dir(data.join("logs"))
        .unwrap()
        .map(|entry| fs::read_to_string(entry.unwrap().path()).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!logs.contains(instruction));
    let records: Vec<Value> = logs
        .lines()
        .filter_map(|line| {
            line.find('{')
                .and_then(|offset| serde_json::from_str(&line[offset..]).ok())
        })
        .collect();
    let event = |id: &str, name: &str| {
        records
            .iter()
            .find(|record| record["request_id"] == id && record["event"] == name)
            .unwrap_or_else(|| panic!("missing {name} for {id}: {logs}"))
    };
    let start = event(&task.request_id, "cli_starting");
    assert!(
        start["args"]
            .as_array()
            .unwrap()
            .contains(&json!("--output-schema"))
    );
    assert_eq!(event(&task.request_id, "cli_response")["exit_code"], 0);
    assert_eq!(
        serde_json::from_str::<Value>(
            event(&task.request_id, "cli_response_content")["stdout"]
                .as_str()
                .unwrap()
        )
        .unwrap(),
        json!({"operations":[]})
    );
    assert_eq!(event(&task.request_id, "apply_succeeded")["operations"], 0);
    assert_eq!(
        event(&changed.request_id, "apply_failed")["code"],
        "APPLY_TARGET_CHANGED"
    );
    assert_eq!(
        event(&invalid.request_id, "cli_failed")["code"],
        "CLI_RESPONSE_INVALID"
    );
    let failure = event(&rejected.request_id, "cli_failed");
    assert_eq!(failure["code"], "CLI_EXIT_NONZERO");
    assert_eq!(failure["exit_code"], 1);
    assert_eq!(failure["diagnostic_hint"], "AUTHENTICATION_OR_PERMISSION");
    let failure = event(&provider_rejection.request_id, "cli_failed");
    assert_eq!(failure["code"], "CLI_PROVIDER_REJECTED");
    assert_eq!(failure["exit_code"], 0);
    assert_eq!(failure["http_status"], 429);
    let failure = event(&missing.request_id, "cli_failed");
    assert_eq!(failure["code"], "CLI_SPAWN");
    assert_eq!(failure["io_kind"], "NotFound");
}
