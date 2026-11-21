use mindglider::{
    ai::{self, CliSettings, EditPlan, Operation, Provider, Snapshot},
    model::{Document, History},
    richtext::RichText,
};
use serde_json::{Value, json};
use std::{path::Path, time::Duration};

fn run(provider: Provider, response: &str, snapshot: Snapshot) -> Result<EditPlan, String> {
    let folder = Path::new("artifacts/tmp");
    std::fs::create_dir_all(folder).unwrap();
    let dir = tempfile::tempdir_in(folder).unwrap();
    let work = dir.path().canonicalize().unwrap();
    let cli = env!("CARGO_BIN_EXE_mindglider-cli-fixture");
    let task = ai::spawn(
        CliSettings {
            provider,
            model: response.into(),
            codex: cli.into(),
            claude: cli.into(),
            timeout_secs: 5,
            ..Default::default()
        },
        snapshot,
        "fixture".into(),
        &work,
        eframe::egui::Context::default(),
    );
    task.receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("title CLI fixture must terminate")
}

#[test]
fn both_providers_return_a_title_for_the_fixed_node_only() {
    let mut doc = Document::default();
    let root = doc.root.clone();
    let parent = doc.add(&root, "section").unwrap();
    let target = doc.add(&parent, "観点").unwrap();
    let sibling = doc.add(&parent, "adjacent topic").unwrap();
    doc.add(&sibling, "context from a sibling's child").unwrap();
    doc.add(&target, "existing child of the target").unwrap();
    doc.add(&root, "outside context").unwrap();
    let node = doc.nodes.get_mut(&target).unwrap();
    node.note = RichText::plain("note to retain");
    node.details = RichText::plain("details to retain");

    let snapshot = Snapshot::capture_title(&doc, &target, "Propose a more precise title").unwrap();
    let payload: Value = serde_json::from_str(&snapshot.payload).unwrap();
    let nodes = payload["nodes"].as_array().unwrap();
    let target_index = payload["target"].as_u64().unwrap() as usize;
    let find = |title: &str| nodes.iter().position(|node| node["text"] == title).unwrap();
    let parent_index = find("section");
    let sibling_index = find("adjacent topic");
    assert_eq!(nodes.len(), 5);
    assert_eq!(nodes[target_index]["text"], "観点");
    assert_eq!(nodes[target_index]["parent"], json!(parent_index));
    assert_eq!(nodes[sibling_index]["parent"], json!(parent_index));
    assert_eq!(
        nodes[find("context from a sibling's child")]["parent"],
        json!(sibling_index)
    );
    assert_eq!(
        nodes[find("existing child of the target")]["parent"],
        json!(target_index)
    );
    assert!(nodes[parent_index]["parent"].is_null());
    assert!(
        doc.nodes
            .keys()
            .all(|id| !snapshot.payload.contains(id.as_str()))
    );

    for (provider, response) in [
        (Provider::Codex, "title-ok"),
        (Provider::Claude, "title-ok"),
        (Provider::Claude, "title-result-envelope"),
    ] {
        let plan = run(provider, response, snapshot.clone()).unwrap();
        assert!(matches!(
            plan.operations.as_slice(),
            [Operation::Update { id, text, note: None, details: None }]
                if id == &target && text == "改善後のタイトル"
        ));
        let mut history = History::new(doc.clone());
        assert_eq!(ai::apply(&mut history, &snapshot, &plan).unwrap(), 1);
        let mut expected = doc.clone();
        expected
            .nodes
            .get_mut(&target)
            .unwrap()
            .text
            .edit("改善後のタイトル");
        assert_eq!(history.doc, expected);
        history.undo();
        assert_eq!(history.doc, doc);
    }
}

#[test]
fn title_responses_cannot_supply_an_operation_or_target() {
    let doc = Document::default();
    let snapshot = Snapshot::capture_title(&doc, &doc.root, "New title").unwrap();
    for (provider, response) in [
        (Provider::Codex, "title-extra-control"),
        (Provider::Claude, "title-old-operation"),
        (Provider::Codex, "title-blank"),
        (Provider::Claude, "title-multiline"),
    ] {
        assert!(
            run(provider, response, snapshot.clone()).is_err(),
            "{response}"
        );
    }

    let mut history = History::new(doc);
    let plan = run(Provider::Codex, "title-ok", snapshot.clone()).unwrap();
    let mut changed = history.doc.clone();
    changed
        .nodes
        .get_mut(&snapshot.root)
        .unwrap()
        .text
        .edit("Changed while the AI was running");
    history.doc = changed.clone();
    assert!(ai::apply(&mut history, &snapshot, &plan).is_err());
    assert_eq!(history.doc, changed);
}
