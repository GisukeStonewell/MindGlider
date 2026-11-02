//! Opt-in live CLI probe: only generated sample data is sent.
use mindglider::{
    ai::{self, CliSettings, Provider, Snapshot},
    model::{Document, History},
};
fn main() -> anyhow::Result<()> {
    let provider = if std::env::args().nth(1).as_deref() == Some("claude") {
        Provider::Claude
    } else {
        Provider::Codex
    };
    let dir = std::path::Path::new("artifacts/live-cli")
        .canonicalize()
        .or_else(|_| {
            std::fs::create_dir_all("artifacts/live-cli")?;
            std::path::Path::new("artifacts/live-cli").canonicalize()
        })?;
    let doc = Document::default();
    let root = doc.root.clone();
    let knowledge = std::env::args().nth(2).as_deref() == Some("knowledge");
    let instruction = if knowledge {
        let path = dir.join("knowledge-fixture.txt");
        std::fs::write(&path, "LOCAL_KNOWLEDGE_REFERENCE_OK")?;
        format!(
            "Read the local knowledge file at {}. Add exactly one child whose text is exactly the file's content, with a fresh ID. Do not change the root.",
            path.display()
        )
    } else {
        "Add exactly one child node with text 'CLI test' and a fresh ID. Do not modify the root."
            .into()
    };
    let snapshot = Snapshot::capture(&doc, &root, &instruction)?;
    let settings = CliSettings {
        provider,
        timeout_secs: 60,
        ..Default::default()
    };
    let start = std::time::Instant::now();
    let task = ai::spawn(
        settings,
        snapshot.clone(),
        "probe".into(),
        &dir,
        eframe::egui::Context::default(),
    );
    let result = task
        .receiver
        .recv_timeout(std::time::Duration::from_secs(75))?;
    let mut history = History::new(doc.clone());
    let report = match result {
        Ok(plan) => match ai::apply(&mut history, &snapshot, &plan) {
            Ok(n) => {
                let applied = history.doc.nodes.len() > 1;
                let reference_read = knowledge
                    && history
                        .doc
                        .nodes
                        .values()
                        .any(|n| n.text.text() == "LOCAL_KNOWLEDGE_REFERENCE_OK");
                history.undo();
                serde_json::json!({"provider":format!("{provider:?}"),"operations":n,"child_added":applied,"reference_read":reference_read,"undo_restored":history.doc==doc,"seconds":start.elapsed().as_secs_f32()})
            }
            Err(e) => {
                serde_json::json!({"provider":format!("{provider:?}"),"validation_error":format!("{e:#}")})
            }
        },
        Err(e) => {
            serde_json::json!({"provider":format!("{provider:?}"),"error":e,"seconds":start.elapsed().as_secs_f32()})
        }
    };
    let file = dir.join(format!(
        "{provider:?}{}.json",
        if knowledge { "-knowledge" } else { "" }
    ));
    mindglider::storage::atomic_write(&file, &serde_json::to_vec_pretty(&report)?)?;
    println!("{}", report);
    Ok(())
}
