use crate::i18n::{msg, tr};
use crate::model::{Document, History, validate_text};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provider {
    Codex,
    Claude,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CliSettings {
    pub codex: String,
    pub claude: String,
    pub provider: Provider,
    pub model: String,
    pub timeout_secs: u64,
    pub system_prompt: String,
    pub codex_options: CodexOptions,
    pub claude_options: ClaudeOptions,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CodexOptions {
    pub model: String,
    pub effort: String,
    pub service_tier: String,
    pub web_search: bool,
    pub oss: bool,
    pub local_provider: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ClaudeOptions {
    pub model: String,
    pub effort: String,
    pub fallback_model: String,
    pub max_budget_usd: String,
    pub add_dirs: String,
    pub disable_slash_commands: bool,
}
impl CliSettings {
    pub fn migrate_model(&mut self) {
        if !self.model.is_empty() {
            let target = match self.provider {
                Provider::Codex => &mut self.codex_options.model,
                Provider::Claude => &mut self.claude_options.model,
            };
            if target.is_empty() {
                *target = self.model.clone();
            }
            self.model.clear();
        }
    }
    pub fn active_model(&self) -> &str {
        let model = match self.provider {
            Provider::Codex => &self.codex_options.model,
            Provider::Claude => &self.claude_options.model,
        };
        if model.is_empty() { &self.model } else { model }
    }
    pub fn set_language(&mut self, language: &str) {
        if ["ja", "en"]
            .iter()
            .any(|lang| self.system_prompt == default_system_prompt_for(lang))
        {
            self.system_prompt = default_system_prompt_for(language);
        }
    }
    pub(crate) fn command(&self) -> Command {
        match self.provider {
            Provider::Codex => {
                #[cfg(windows)]
                if self.codex.eq_ignore_ascii_case("codex.exe") {
                    if let Some(path) = detect("codex") {
                        return Command::new(path);
                    }
                    let path = std::env::var_os("PATH").unwrap_or_default();
                    let default_prefix =
                        std::env::var_os("APPDATA").map(|dir| PathBuf::from(dir).join("npm"));
                    if let Some(path) = std::env::split_paths(&path)
                        .chain(default_prefix)
                        .find_map(|prefix| codex_npm_executable(&prefix))
                    {
                        return Command::new(path);
                    }
                }
                Command::new(&self.codex)
            }
            Provider::Claude => Command::new(&self.claude),
        }
    }
}
impl Default for CliSettings {
    fn default() -> Self {
        Self {
            codex: detect("codex").unwrap_or_else(|| "codex.exe".into()),
            claude: detect("claude").unwrap_or_else(|| "claude.exe".into()),
            provider: Provider::Codex,
            model: String::new(),
            timeout_secs: 300,
            system_prompt: default_system_prompt(),
            codex_options: CodexOptions::default(),
            claude_options: ClaudeOptions::default(),
        }
    }
}
pub fn default_system_prompt() -> String {
    default_system_prompt_for(crate::i18n::language())
}
pub fn default_system_prompt_for(language: &str) -> String {
    (if language == "en" {
        include_str!("../assets/prompts/tree-edit.txt")
    } else {
        include_str!("../assets/prompts/tree-edit-ja.txt")
    })
    .trim()
    .to_owned()
}
/// Optional arguments are separate tokens, never shell command text.
pub fn optional_args(settings: &CliSettings) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let mut value = |flag: &str, text: &str| {
        if !text.trim().is_empty() {
            args.extend([flag.to_owned(), text.trim().to_owned()]);
        }
    };
    value("--model", settings.active_model());
    match settings.provider {
        Provider::Codex => {
            let opt = &settings.codex_options;
            value(
                "-c",
                &format!(
                    "web_search={}",
                    serde_json::to_string(if opt.web_search { "live" } else { "disabled" })?
                ),
            );
            if !opt.effort.is_empty() {
                value(
                    "-c",
                    &format!(
                        "model_reasoning_effort={}",
                        serde_json::to_string(&opt.effort)?
                    ),
                );
            }
            if !opt.service_tier.is_empty() {
                value(
                    "-c",
                    &format!("service_tier={}", serde_json::to_string(&opt.service_tier)?),
                );
            }
            if opt.oss {
                value("--local-provider", &opt.local_provider);
                args.push("--oss".into());
            }
        }
        Provider::Claude => {
            let opt = &settings.claude_options;
            value("--effort", &opt.effort);
            value("--fallback-model", &opt.fallback_model);
            if !opt.max_budget_usd.trim().is_empty() {
                let budget: f64 = opt
                    .max_budget_usd
                    .trim()
                    .parse()
                    .context(tr("cli.budget_invalid"))?;
                ensure!(
                    budget.is_finite() && budget > 0.,
                    "{}",
                    tr("cli.budget_invalid")
                );
                value("--max-budget-usd", &opt.max_budget_usd);
            }
            for dir in opt.add_dirs.lines().filter(|s| !s.trim().is_empty()) {
                value("--add-dir", dir);
            }
            if opt.disable_slash_commands {
                args.push("--disable-slash-commands".into());
            }
        }
    }
    Ok(args)
}
pub fn detect(name: &str) -> Option<String> {
    let file = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.into()
    };
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let p = dir.join(&file);
        if p.is_file() {
            return Some(p.to_string_lossy().into());
        }
    }
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))?;
    let home = PathBuf::from(home);
    for p in [
        home.join(".local/bin").join(&file),
        home.join("AppData/Local/Programs/OpenAI/Codex/bin")
            .join(&file),
        home.join(".cargo/bin").join(&file),
    ] {
        if p.is_file() {
            return Some(p.to_string_lossy().into());
        }
    }
    None
}

#[cfg(windows)]
fn codex_npm_executable(prefix: &Path) -> Option<PathBuf> {
    if !prefix.is_absolute() {
        return None;
    }
    let (platform_package, target) = match std::env::consts::ARCH {
        "x86_64" => ("codex-win32-x64", "x86_64-pc-windows-msvc"),
        "aarch64" => ("codex-win32-arm64", "aarch64-pc-windows-msvc"),
        _ => return None,
    };
    let scope = prefix.join("node_modules/@openai");
    let codex = scope.join("codex");
    if !codex.join("bin/codex.js").is_file() {
        return None;
    }
    // npm can place the platform package inside Codex or alongside it.
    [
        codex.join("node_modules/@openai").join(platform_package),
        scope.join(platform_package),
        codex,
    ]
    .into_iter()
    .map(|package| package.join("vendor").join(target).join("bin/codex.exe"))
    .find(|executable| executable.is_file())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Add {
        id: String,
        parent: String,
        text: String,
    },
    Update {
        id: String,
        text: String,
        note: Option<String>,
        details: Option<String>,
    },
    Delete {
        id: String,
    },
    Move {
        id: String,
        parent: String,
        index: usize,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditPlan {
    pub operations: Vec<Operation>,
}
#[derive(Clone)]
pub struct Snapshot {
    pub root: String,
    pub fingerprint: String,
    pub document_root: String,
    pub payload: String,
    title_only: bool,
}
impl Snapshot {
    pub fn is_title_only(&self) -> bool {
        self.title_only
    }

    pub fn capture(doc: &Document, root: &str, instruction: &str) -> Result<Self> {
        doc.ensure_branch_editable(root)?;
        let ids = doc.subtree(root);
        let nodes:Vec<_>=ids.iter().map(|id|{let n=&doc.nodes[id];serde_json::json!({"id":id,"parent":if id==root{None}else{n.parent.as_deref()},"children":n.children,"text":n.text.text(),"note":n.note.text(),"details":n.details.text()})}).collect();
        Ok(Self {
            root: root.into(),
            fingerprint: doc.fingerprint(root),
            document_root: doc.root.clone(),
            payload: serde_json::to_string(
                &serde_json::json!({"instruction":instruction,"root":root,"nodes":nodes}),
            )?,
            title_only: false,
        })
    }

    pub fn capture_title(doc: &Document, target: &str, instruction: &str) -> Result<Self> {
        doc.ensure_editable(target)?;
        let node = &doc.nodes[target];
        ensure!(
            !node.text.unsupported,
            ApplyError("APPLY_UNSUPPORTED_TEXT", "ai.013")
        );
        let context = node.parent.as_deref().unwrap_or(target);
        let ids = doc.subtree(context);
        let indices: BTreeMap<_, _> = ids
            .iter()
            .enumerate()
            .map(|(index, id)| (id.as_str(), index))
            .collect();
        let nodes: Vec<_> = ids
            .iter()
            .map(|id| {
                let node = &doc.nodes[id];
                json!({
                    "parent": node.parent.as_deref().and_then(|parent| indices.get(parent)),
                    "text": node.text.text(),
                    "note": node.note.text(),
                    "details": node.details.text()
                })
            })
            .collect();
        Ok(Self {
            root: target.into(),
            fingerprint: doc.fingerprint(target),
            document_root: doc.root.clone(),
            payload: serde_json::to_string(&json!({
                "instruction": instruction, "target": indices[target], "nodes": nodes
            }))?,
            title_only: true,
        })
    }
}
fn validate_snapshot(doc: &Document, snapshot: &Snapshot) -> Result<()> {
    ensure!(
        doc.root == snapshot.document_root,
        ApplyError("APPLY_WRONG_DOCUMENT", "ai.021")
    );
    ensure!(
        doc.nodes.contains_key(&snapshot.root),
        ApplyError("APPLY_TARGET_DELETED", "ai.020")
    );
    ensure!(
        doc.fingerprint(&snapshot.root) == snapshot.fingerprint,
        ApplyError("APPLY_TARGET_CHANGED", "ai.019")
    );
    Ok(())
}

fn validate_plan(snapshot: &Snapshot, plan: &EditPlan) -> Result<()> {
    if snapshot.title_only {
        let [
            Operation::Update {
                id,
                text,
                note: None,
                details: None,
            },
        ] = plan.operations.as_slice()
        else {
            bail!(ApplyError(
                "APPLY_TITLE_OPERATION",
                "ai.title_invalid_operation"
            ));
        };
        ensure!(
            id == &snapshot.root,
            ApplyError("APPLY_TITLE_OPERATION", "ai.title_invalid_operation")
        );
        validate_title(text)?;
    }
    ensure!(
        plan.operations.len() <= 5000,
        ApplyError("APPLY_TOO_MANY_OPERATIONS", "ai.018")
    );
    Ok(())
}

pub fn apply(history: &mut History, snapshot: &Snapshot, plan: &EditPlan) -> Result<usize> {
    validate_snapshot(&history.doc, snapshot)?;
    validate_plan(snapshot, plan)?;
    let mut allowed = history.doc.subtree(&snapshot.root);
    let label = if snapshot.title_only {
        "ai.title_history"
    } else {
        "ai.026"
    };
    history.edit(tr(label), |doc| {
        for op in &plan.operations {
            apply_operation(doc, &mut allowed, &snapshot.root, op)?;
        }
        Ok(())
    })?;
    Ok(plan.operations.len())
}

fn apply_operation(
    doc: &mut Document,
    allowed: &mut BTreeSet<String>,
    root: &str,
    op: &Operation,
) -> Result<()> {
    match op {
        Operation::Add { id, parent, text } => {
            ensure!(
                allowed.contains(parent) && doc.nodes.contains_key(parent),
                ApplyError("APPLY_PARENT_OUT_OF_SCOPE", "ai.017")
            );
            ensure!(
                !doc.nodes.contains_key(id) && !allowed.contains(id),
                ApplyError("APPLY_DUPLICATE_NODE_ID", "ai.016")
            );
            ensure!(
                !id.is_empty() && id.len() < 256,
                ApplyError("APPLY_INVALID_NODE_ID", "ai.015")
            );
            validate_text(text)?;
            let generated = doc.add(parent, text)?;
            let mut n = doc.nodes.remove(&generated).unwrap();
            n.id = id.clone();
            doc.nodes
                .get_mut(parent)
                .unwrap()
                .children
                .iter_mut()
                .filter(|c| **c == generated)
                .for_each(|c| *c = id.clone());
            doc.nodes.insert(id.clone(), n);
            allowed.insert(id.clone());
        }
        Operation::Update {
            id,
            text,
            note,
            details,
        } => {
            ensure!(
                allowed.contains(id),
                ApplyError("APPLY_UPDATE_OUT_OF_SCOPE", "ai.014")
            );
            doc.ensure_editable(id)?;
            validate_text(text)?;
            let n = doc.nodes.get_mut(id).unwrap();
            ensure!(
                !n.text.unsupported,
                ApplyError("APPLY_UNSUPPORTED_TEXT", "ai.013")
            );
            if n.text.text() != *text {
                n.text.edit(text);
            }
            if let Some(note) = note {
                validate_text(note)?;
                ensure!(
                    !n.note.unsupported,
                    ApplyError("APPLY_UNSUPPORTED_NOTE", "ai.012")
                );
                if n.note.text() != *note {
                    n.note.edit(note);
                }
            }
            if let Some(details) = details {
                validate_text(details)?;
                ensure!(
                    !n.details.unsupported,
                    ApplyError("APPLY_UNSUPPORTED_DETAILS", "ai.011")
                );
                if n.details.text() != *details {
                    n.details.edit(details);
                }
            }
        }
        Operation::Delete { id } => {
            ensure!(
                allowed.contains(id) && id != root,
                ApplyError("APPLY_DELETE_OUT_OF_SCOPE", "ai.010")
            );
            doc.delete(id)?;
        }
        Operation::Move { id, parent, index } => {
            ensure!(
                allowed.contains(id) && allowed.contains(parent) && id != root,
                ApplyError("APPLY_MOVE_OUT_OF_SCOPE", "ai.009")
            );
            doc.move_node(id, parent, *index)?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Deleted,
    Modified,
}

pub struct TreeNodeView {
    pub node_id: String,
    pub text: String,
    pub parent: Option<String>,
    pub position: Option<usize>,
    pub note: String,
    pub details: String,
}

pub struct TreeChange {
    pub kind: ChangeKind,
    pub before: Option<TreeNodeView>,
    pub after: Option<TreeNodeView>,
    pub moved: bool,
    pub reordered: bool,
}

pub struct RenameCandidate {
    pub operation: usize,
    pub result_node_id: String,
    pub before: String,
    pub after: String,
}

pub struct TreeReview {
    pub changes: Vec<TreeChange>,
    pub renames: Vec<RenameCandidate>,
    plan: EditPlan,
}

pub fn preview_tree(
    history: &History,
    snapshot: &Snapshot,
    plan: &EditPlan,
    add_choices: &BTreeSet<usize>,
) -> Result<TreeReview> {
    ensure!(
        !snapshot.title_only,
        "Tree review requires a tree-edit snapshot"
    );
    validate_snapshot(&history.doc, snapshot)?;
    validate_plan(snapshot, plan)?;
    let scope = history.doc.subtree(&snapshot.root);
    let mut allowed = scope.clone();
    let mut draft = history.doc.clone();
    let mut operations = Vec::with_capacity(plan.operations.len());
    let mut renames = Vec::new();
    for (index, operation) in plan.operations.iter().enumerate() {
        let normalized = if let Operation::Add { id, parent, text } = operation
            && scope.contains(id)
            && allowed.contains(parent)
            && let Some(node) = draft.nodes.get(id)
            && node.parent.as_ref() == Some(parent)
        {
            let as_add = add_choices.contains(&index);
            let result_node_id = if as_add {
                crate::model::new_id()
            } else {
                id.clone()
            };
            renames.push(RenameCandidate {
                operation: index,
                result_node_id: result_node_id.clone(),
                before: node.text.text(),
                after: text.clone(),
            });
            if as_add {
                Operation::Add {
                    id: result_node_id,
                    parent: parent.clone(),
                    text: text.clone(),
                }
            } else {
                Operation::Update {
                    id: result_node_id,
                    text: text.clone(),
                    note: None,
                    details: None,
                }
            }
        } else {
            operation.clone()
        };
        apply_operation(&mut draft, &mut allowed, &snapshot.root, &normalized)?;
        operations.push(normalized);
    }
    draft.validate()?;
    let changes = tree_changes(&history.doc, &draft, &snapshot.root);
    Ok(TreeReview {
        changes,
        renames,
        plan: EditPlan { operations },
    })
}

pub fn apply_review_traced(
    history: &mut History,
    snapshot: &Snapshot,
    review: &TreeReview,
    request_id: &str,
    started: Instant,
) -> Result<usize> {
    apply_traced(history, snapshot, &review.plan, request_id, started)
}

fn tree_order<'a>(doc: &'a Document, root: &'a str) -> Vec<&'a str> {
    let mut ids = Vec::new();
    let mut pending = vec![root];
    while let Some(id) = pending.pop() {
        ids.push(id);
        pending.extend(doc.nodes[id].children.iter().rev().map(String::as_str));
    }
    ids
}

struct SiblingRank {
    position: usize,
    surviving_order: Option<usize>,
}

// Compare the order of surviving siblings, ignoring additions and removed branches.
fn sibling_ranks<'a>(
    doc: &'a Document,
    other: &Document,
    parents: &[&str],
) -> BTreeMap<&'a str, SiblingRank> {
    let mut ranks = BTreeMap::new();
    for parent in parents {
        let mut surviving_order = 0;
        for (index, id) in doc.nodes[*parent].children.iter().enumerate() {
            let retained =
                other.nodes.get(id).and_then(|node| node.parent.as_deref()) == Some(*parent);
            if retained {
                surviving_order += 1;
            }
            ranks.insert(
                id.as_str(),
                SiblingRank {
                    position: index + 1,
                    surviving_order: retained.then_some(surviving_order),
                },
            );
        }
    }
    ranks
}

fn node_view(
    doc: &Document,
    node: &crate::model::Node,
    rank: Option<&SiblingRank>,
) -> TreeNodeView {
    TreeNodeView {
        node_id: node.id.clone(),
        text: node.text.text(),
        parent: node.parent.as_ref().map(|id| doc.nodes[id].text.text()),
        position: rank.map(|rank| rank.position),
        note: node.note.text(),
        details: node.details.text(),
    }
}

fn tree_changes(before: &Document, after: &Document, root: &str) -> Vec<TreeChange> {
    let before_order = tree_order(before, root);
    let after_order = tree_order(after, root);
    let ranks_before = sibling_ranks(before, after, &before_order);
    let ranks_after = sibling_ranks(after, before, &after_order);
    let mut seen = BTreeSet::new();
    before_order
        .into_iter()
        .chain(after_order)
        .filter(|id| seen.insert(*id))
        .filter_map(|id| {
            let old = before.nodes.get(id);
            let new = after.nodes.get(id);
            let old_rank = ranks_before.get(id);
            let new_rank = ranks_after.get(id);
            let (kind, moved, reordered) = match (old, new) {
                (Some(old), Some(new)) => {
                    let moved = old.parent != new.parent;
                    let reordered = !moved
                        && old_rank.and_then(|rank| rank.surviving_order)
                            != new_rank.and_then(|rank| rank.surviving_order);
                    if !moved
                        && !reordered
                        && old.text == new.text
                        && old.note == new.note
                        && old.details == new.details
                    {
                        return None;
                    }
                    (ChangeKind::Modified, moved, reordered)
                }
                (None, Some(_)) => (ChangeKind::Added, false, false),
                (Some(_), None) => (ChangeKind::Deleted, false, false),
                (None, None) => return None,
            };
            Some(TreeChange {
                kind,
                before: old.map(|node| node_view(before, node, old_rank)),
                after: new.map(|node| node_view(after, node, new_rank)),
                moved,
                reordered,
            })
        })
        .collect()
}

#[derive(Debug)]
struct ApplyError(&'static str, &'static str);

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(tr(self.1))
    }
}

impl std::error::Error for ApplyError {}

pub fn failure_code(error: &anyhow::Error) -> &'static str {
    error
        .downcast_ref::<ApplyError>()
        .map_or("APPLY_OPERATION_REJECTED", |reason| reason.0)
}

pub fn apply_traced(
    history: &mut History,
    snapshot: &Snapshot,
    plan: &EditPlan,
    request_id: &str,
    started: Instant,
) -> Result<usize> {
    let result = apply(history, snapshot, plan);
    let elapsed_ms = started.elapsed().as_millis();
    match &result {
        Ok(operations) => log::info!(
            "{}",
            json!({
                "event": "apply_succeeded", "request_id": request_id,
                "elapsed_ms": elapsed_ms, "operations": operations
            })
        ),
        Err(error) => {
            let code = failure_code(error);
            log::error!(
                "{}",
                json!({
                    "event": "apply_failed", "request_id": request_id,
                    "code": code, "elapsed_ms": elapsed_ms, "operations": plan.operations.len(),
                    "reason": error.to_string()
                })
            );
        }
    }
    result
}
pub fn schema() -> serde_json::Value {
    fn object(properties: serde_json::Value, required: &[&str]) -> serde_json::Value {
        serde_json::json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
    }
    let s = serde_json::json!({"type":"string"});
    let optional = serde_json::json!({"type":["string","null"]});
    object(
        serde_json::json!({"operations":{"type":"array","items":{"anyOf":[
            object(serde_json::json!({"op":{"const":"add","type":"string"},"id":s,"parent":s,"text":s}),&["op","id","parent","text"]),
            object(serde_json::json!({"op":{"const":"update","type":"string"},"id":s,"text":s,"note":optional,"details":optional}),&["op","id","text","note","details"]),
            object(serde_json::json!({"op":{"const":"delete","type":"string"},"id":s}),&["op","id"]),
            object(serde_json::json!({"op":{"const":"move","type":"string"},"id":s,"parent":s,"index":{"type":"integer","minimum":0}}),&["op","id","parent","index"])
        ]}}}),
        &["operations"],
    )
}

fn title_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": { "title": { "type": "string" } },
        "required": ["title"],
        "additionalProperties": false
    })
}

fn validate_title(title: &str) -> Result<()> {
    ensure!(
        !title.trim().is_empty()
            && !title.contains(['\r', '\n', '\u{0085}', '\u{2028}', '\u{2029}']),
        ApplyError("TITLE_INVALID_TEXT", "ai.title_invalid_text")
    );
    validate_text(title)
}
pub struct Task {
    pub request_id: String,
    pub snapshot: Snapshot,
    pub tab_id: String,
    pub cancel: Arc<AtomicBool>,
    pub receiver: mpsc::Receiver<Result<EditPlan, String>>,
    pub started: Instant,
}
impl Task {
    pub fn request_cancel(&self, reason: &str) {
        if !self.cancel.swap(true, Ordering::Relaxed) {
            log::info!(
                "{}",
                json!({
                    "event": "ai_cancel_requested", "request_id": self.request_id,
                    "reason": reason, "elapsed_ms": self.started.elapsed().as_millis()
                })
            );
        }
    }
}
pub fn spawn(
    settings: CliSettings,
    snapshot: Snapshot,
    tab_id: String,
    work: &Path,
    ctx: eframe::egui::Context,
) -> Task {
    let request_id = uuid::Uuid::new_v4().simple().to_string();
    let worker_id = request_id.clone();
    let started = Instant::now();
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    let snap = snapshot.clone();
    let work = work.to_owned();
    let (tx, receiver) = mpsc::channel();
    log::info!(
        "{}",
        json!({
            "event": "ai_requested", "request_id": request_id, "provider": settings.provider,
            "tab_id": tab_id, "root_id": snapshot.root, "timeout_secs": settings.timeout_secs,
            "mode": if snapshot.title_only { "title_only" } else { "tree" }
        })
    );
    std::thread::spawn(move || {
        let result = execute(&settings, &snap, &work, &flag, &worker_id, started)
            .map_err(|e| format!("{e:#}"));
        let _ = tx.send(result);
        ctx.request_repaint();
    });
    Task {
        request_id,
        snapshot,
        tab_id,
        cancel,
        receiver,
        started,
    }
}
fn read_capped(mut reader: impl Read) -> Result<Vec<u8>> {
    let mut data = vec![];
    let mut b = [0u8; 8192];
    let mut too_large = false;
    loop {
        let n = reader.read(&mut b)?;
        if n == 0 {
            break;
        }
        if data.len() + n <= 16 * 1024 * 1024 {
            data.extend_from_slice(&b[..n]);
        } else {
            too_large = true;
        }
    }
    ensure!(!too_large, OutputLimit);
    Ok(data)
}
#[derive(Debug)]
struct OutputLimit;

impl std::fmt::Display for OutputLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(tr("ai.008"))
    }
}

impl std::error::Error for OutputLimit {}
fn execute(
    settings: &CliSettings,
    snapshot: &Snapshot,
    work: &Path,
    cancel: &AtomicBool,
    request_id: &str,
    started: Instant,
) -> Result<EditPlan> {
    let folder = work.join(format!("cli-{request_id}"));
    let mut code = "CLI_WORK_DIRECTORY";
    let mut exit_code = None;
    let mut http_status = None;
    let mut diagnostic_hint = None;
    let result = (|| -> Result<EditPlan> {
        std::fs::create_dir_all(&folder)?;
        code = "CLI_SCHEMA_WRITE";
        let schema_file = folder.join("schema.json");
        let response_schema = if snapshot.title_only {
            title_schema()
        } else {
            schema()
        };
        std::fs::write(&schema_file, serde_json::to_vec(&response_schema)?)?;
        let mut cmd = settings.command();
        cmd.current_dir(&folder)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        match settings.provider {
            Provider::Codex => {
                cmd.args([
                    "exec",
                    "--ignore-user-config",
                    "--skip-git-repo-check",
                    "--sandbox",
                    "read-only",
                    "--ephemeral",
                    "--color",
                    "never",
                    "--output-schema",
                ])
                .arg(&schema_file)
                .arg("-");
            }
            Provider::Claude => {
                cmd.args(["-p", "--output-format", "json", "--json-schema"])
                    .arg(response_schema.to_string())
                    .args([
                        "--tools",
                        "Read,Glob,Grep",
                        "--allowedTools",
                        "Read,Glob,Grep",
                        "--strict-mcp-config",
                        "--mcp-config",
                        "{\"mcpServers\":{}}",
                        "--setting-sources",
                        "",
                        "--settings",
                        "{\"disableAllHooks\":true}",
                        "--no-session-persistence",
                    ]);
            }
        }
        code = "CLI_INVALID_ARGUMENTS";
        cmd.args(optional_args(settings)?);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        log::info!(
            "{}",
            json!({
                "event": "cli_starting", "request_id": request_id, "provider": settings.provider,
                "program": cmd.get_program().to_string_lossy(),
                "args": cmd.get_args().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>()
            })
        );
        code = "CLI_SPAWN";
        let mut child = cmd.spawn().context(tr("ai.025"))?;
        log::debug!(
            "{}",
            json!({
                "event": "cli_started", "request_id": request_id, "pid": child.id()
            })
        );
        #[cfg(windows)]
        let _job = {
            code = "CLI_PROCESS_JOB";
            match ProcessJob::attach(&child) {
                Ok(job) => job,
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(e);
                }
            }
        };
        let mut input = child.stdin.take().unwrap();
        let system_prompt = if snapshot.title_only {
            tr("ai.title_prompt")
        } else {
            &settings.system_prompt
        };
        let payload = format!(
            "{}\n\n<user_request_and_subtree_json>\n{}\n</user_request_and_subtree_json>",
            system_prompt, snapshot.payload
        );
        log::trace!(
            "{}",
            json!({
                "event": "cli_input_prepared", "request_id": request_id, "stdin_bytes": payload.len()
            })
        );
        let writer = std::thread::spawn(move || input.write_all(payload.as_bytes()));
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let out = std::thread::spawn(move || read_capped(stdout));
        let err = std::thread::spawn(move || read_capped(stderr));
        let start = Instant::now();
        code = "CLI_WAIT";
        let status = loop {
            if cancel.load(Ordering::Relaxed)
                || start.elapsed() > Duration::from_secs(settings.timeout_secs.max(1))
            {
                #[cfg(windows)]
                drop(_job);
                let _ = child.kill();
                exit_code = child.wait().ok().and_then(|status| status.code());
                let _ = writer.join();
                let output = out.join().ok().and_then(Result::ok).unwrap_or_default();
                let error = err.join().ok().and_then(Result::ok).unwrap_or_default();
                trace_response(request_id, exit_code, false, started, &output, &error);
                if cancel.load(Ordering::Relaxed) {
                    code = "CLI_CANCELLED";
                    bail!(tr("ai.024"));
                } else {
                    code = "CLI_TIMEOUT";
                    bail!(tr("ai.023"));
                }
            }
            if let Some(status) = child.try_wait()? {
                break status;
            }
            std::thread::sleep(Duration::from_millis(100));
        };
        // The CLI may have left helpers holding its pipe handles.
        #[cfg(windows)]
        drop(_job);
        exit_code = status.code();
        let written = writer
            .join()
            .map_err(|_| anyhow::anyhow!("{}", msg("ai.007", &[])))
            .and_then(|result| result.map_err(Into::into));
        code = "CLI_STDOUT";
        let output = out
            .join()
            .map_err(|_| anyhow::anyhow!("{}", msg("ai.006", &[])))??;
        code = "CLI_STDERR";
        let error = err
            .join()
            .map_err(|_| anyhow::anyhow!("{}", msg("ai.005", &[])))??;
        trace_response(
            request_id,
            exit_code,
            status.success(),
            started,
            &output,
            &error,
        );
        if !status.success() {
            code = "CLI_EXIT_NONZERO";
            let detail = if error.is_empty() {
                serde_json::from_slice::<serde_json::Value>(&output)
                    .ok()
                    .and_then(|v| v.get("result").and_then(|s| s.as_str()).map(str::to_owned))
                    .unwrap_or_else(|| String::from_utf8_lossy(&output).into_owned())
            } else {
                String::from_utf8_lossy(&error).into_owned()
            };
            http_status = cli_http_status(&detail);
            diagnostic_hint = cli_failure_hint(&detail, http_status);
            bail!(
                "{}",
                msg(
                    "ai.004",
                    &[(
                        "0",
                        detail.chars().take(2000).collect::<String>().to_string()
                    )]
                )
            );
        }
        code = "CLI_STDIN";
        written?;
        code = if snapshot.title_only {
            "CLI_TITLE_RESPONSE_INVALID"
        } else {
            "CLI_RESPONSE_INVALID"
        };
        let plan = if snapshot.title_only {
            parse_title(settings.provider, &output, &snapshot.root)
        } else {
            parse_output(settings.provider, &output)
        }
        .inspect_err(|error| {
            if let Some(rejection) = error.downcast_ref::<ClaudeResponseError>() {
                code = "CLI_PROVIDER_REJECTED";
                http_status = cli_http_status(&rejection.0);
                diagnostic_hint = cli_failure_hint(&rejection.0, http_status);
            }
        })?;
        log::info!(
            "{}",
            json!({
                "event": "cli_plan_parsed", "request_id": request_id,
                "operations": plan.operations.len()
            })
        );
        Ok(plan)
    })();
    if let Err(error) = &result {
        if error.is::<OutputLimit>() {
            diagnostic_hint = Some("OUTPUT_LIMIT_EXCEEDED");
        }
        let io = error.downcast_ref::<std::io::Error>();
        let json_error = error.downcast_ref::<serde_json::Error>();
        let level = if code == "CLI_CANCELLED" {
            log::Level::Info
        } else {
            log::Level::Error
        };
        log::log!(
            level,
            "{}",
            json!({
                "event": "cli_failed", "request_id": request_id, "code": code,
                "exit_code": exit_code, "http_status": http_status,
                "diagnostic_hint": diagnostic_hint,
                "os_error": io.and_then(std::io::Error::raw_os_error),
                "io_kind": io.map(|error| format!("{:?}", error.kind())),
                "json_error": json_error.map(|error| json!({
                    "category": format!("{:?}", error.classify()), "line": error.line(), "column": error.column()
                })),
                "elapsed_ms": started.elapsed().as_millis()
            })
        );
    }
    let _ = std::fs::remove_file(folder.join("schema.json"));
    let _ = std::fs::remove_dir(&folder);
    result
}
fn trace_response(
    request_id: &str,
    exit_code: Option<i32>,
    success: bool,
    started: Instant,
    output: &[u8],
    error: &[u8],
) {
    log::info!(
        "{}",
        json!({
            "event": "cli_response", "request_id": request_id, "exit_code": exit_code,
            "success": success, "elapsed_ms": started.elapsed().as_millis(),
            "stdout_bytes": output.len(), "stderr_bytes": error.len()
        })
    );
    if crate::logging::include_response_content() && log::log_enabled!(log::Level::Debug) {
        const LIMIT: usize = 64 * 1024;
        log::debug!(
            "{}",
            json!({
                "event": "cli_response_content", "request_id": request_id,
                "stdout": String::from_utf8_lossy(&output[..output.len().min(LIMIT)]),
                "stderr": String::from_utf8_lossy(&error[..error.len().min(LIMIT)]),
                "stdout_truncated": output.len() > LIMIT, "stderr_truncated": error.len() > LIMIT
            })
        );
    }
}
fn cli_http_status(detail: &str) -> Option<u16> {
    static STATUS: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(
            r"(?i)\b(?:http(?:/[\d.]+)?\s+|status(?:\s+code)?\s*[:=]?\s*)([45]\d{2})\b",
        )
        .unwrap()
    });
    STATUS
        .captures_iter(detail)
        .last()?
        .get(1)?
        .as_str()
        .parse()
        .ok()
}
fn cli_failure_hint(detail: &str, http_status: Option<u16>) -> Option<&'static str> {
    let detail = detail.to_ascii_lowercase();
    if matches!(http_status, Some(401 | 403))
        || detail.contains("authentication required")
        || detail.contains("invalid api key")
    {
        Some("AUTHENTICATION_OR_PERMISSION")
    } else if detail.contains("insufficient_quota") || detail.contains("quota exceeded") {
        Some("QUOTA_EXCEEDED")
    } else if http_status == Some(429) || detail.contains("rate limit exceeded") {
        Some("RATE_LIMITED")
    } else if detail.contains("model_not_found") || detail.contains("unsupported_model") {
        Some("MODEL_UNAVAILABLE")
    } else if matches!(http_status, Some(500..=599)) {
        Some("PROVIDER_ERROR")
    } else {
        None
    }
}
pub fn parse_output(provider: Provider, bytes: &[u8]) -> Result<EditPlan> {
    Ok(serde_json::from_value(structured_output(provider, bytes)?)?)
}

fn parse_title(provider: Provider, bytes: &[u8], target: &str) -> Result<EditPlan> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Title {
        title: String,
    }
    let response: Title = serde_json::from_value(structured_output(provider, bytes)?)?;
    let title = response.title.trim();
    validate_title(title)?;
    Ok(EditPlan {
        operations: vec![Operation::Update {
            id: target.into(),
            text: title.into(),
            note: None,
            details: None,
        }],
    })
}

fn structured_output(provider: Provider, bytes: &[u8]) -> Result<serde_json::Value> {
    let json: serde_json::Value = serde_json::from_slice(bytes).context(tr("ai.022"))?;
    if provider == Provider::Claude {
        ensure!(
            !json
                .get("is_error")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            ClaudeResponseError(json.get("result").unwrap_or(&json).to_string())
        );
        if let Some(s) = json.get("structured_output") {
            return Ok(s.clone());
        }
        if let Some(s) = json.get("result").and_then(|v| v.as_str()) {
            return Ok(serde_json::from_str(s)?);
        }
    }
    Ok(json)
}

#[derive(Debug)]
struct ClaudeResponseError(String);

impl std::fmt::Display for ClaudeResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Claude Code: {}", self.0)
    }
}

impl std::error::Error for ClaudeResponseError {}

#[cfg(windows)]
pub(crate) struct ProcessJob(windows_sys::Win32::Foundation::HANDLE);
#[cfg(windows)]
impl ProcessJob {
    pub(crate) fn attach(child: &std::process::Child) -> Result<Self> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::*;
        unsafe {
            let h = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if h.is_null() {
                return Err(std::io::Error::last_os_error()).context(tr("ai.003"));
            }
            let job = Self(h);
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(
                h,
                JobObjectExtendedLimitInformation,
                &info as *const _ as _,
                std::mem::size_of_val(&info) as u32,
            ) == 0
            {
                return Err(std::io::Error::last_os_error()).context(tr("ai.002"));
            }
            if AssignProcessToJobObject(h, child.as_raw_handle() as _) == 0 {
                return Err(std::io::Error::last_os_error()).context(tr("ai.001"));
            }
            Ok(job)
        }
    }
}
#[cfg(windows)]
impl Drop for ProcessJob {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

#[cfg(test)]
mod option_tests {
    use super::*;

    #[cfg(all(windows, target_arch = "x86_64"))]
    #[test]
    fn finds_native_executable_in_nested_and_hoisted_npm_packages() {
        let prefix = tempfile::tempdir().unwrap();
        let scope = prefix.path().join("node_modules/@openai");
        let codex = scope.join("codex");
        std::fs::create_dir_all(codex.join("bin")).unwrap();
        std::fs::write(codex.join("bin/codex.js"), "").unwrap();
        assert_eq!(codex_npm_executable(prefix.path()), None);

        let executable = "codex-win32-x64/vendor/x86_64-pc-windows-msvc/bin/codex.exe";
        let hoisted = scope.join(executable);
        let nested = codex.join("node_modules/@openai").join(executable);
        for path in [hoisted, nested] {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, "").unwrap();
            assert_eq!(codex_npm_executable(prefix.path()), Some(path));
        }
    }

    #[test]
    fn command_preserves_explicit_executable_paths() {
        let settings = CliSettings {
            codex: "C:/Custom CLI/codex.exe".into(),
            claude: "C:/Custom CLI/claude.exe".into(),
            ..Default::default()
        };
        assert_eq!(settings.command().get_program(), settings.codex.as_str());
        let settings = CliSettings {
            provider: Provider::Claude,
            ..settings
        };
        assert_eq!(settings.command().get_program(), settings.claude.as_str());
    }

    #[test]
    fn optional_arguments_are_separate_and_provider_specific() {
        let mut s = CliSettings {
            codex_options: CodexOptions {
                model: "custom model".into(),
                effort: "high".into(),
                service_tier: "fast".into(),
                web_search: true,
                oss: true,
                local_provider: "ollama".into(),
            },
            claude_options: ClaudeOptions {
                model: "sonnet".into(),
                effort: "max".into(),
                fallback_model: "haiku".into(),
                max_budget_usd: "1.25".into(),
                add_dirs: "C:/Knowledge Base".into(),
                disable_slash_commands: true,
            },
            ..Default::default()
        };
        let args = optional_args(&s).unwrap();
        assert!(args.windows(2).any(|a| a == ["--model", "custom model"]));
        assert!(args.contains(&"model_reasoning_effort=\"high\"".into()));
        assert!(args.contains(&"web_search=\"live\"".into()));
        assert!(args.contains(&"--oss".into()));
        assert!(!args.contains(&"--effort".into()));
        s.provider = Provider::Claude;
        let args = optional_args(&s).unwrap();
        assert!(args.windows(2).any(|a| a == ["--model", "sonnet"]));
        assert!(
            args.windows(2)
                .any(|a| a == ["--add-dir", "C:/Knowledge Base"])
        );
        assert!(args.contains(&"--disable-slash-commands".into()));
        assert!(!args.contains(&"--oss".into()));
        for invalid in ["-1", "NaN", "infinity", "invalid"] {
            s.claude_options.max_budget_usd = invalid.into();
            assert!(optional_args(&s).is_err());
        }
    }
    #[test]
    fn localized_defaults_switch_without_overwriting_custom_prompts() {
        let mut s = CliSettings {
            system_prompt: default_system_prompt_for("ja"),
            ..Default::default()
        };
        s.set_language("en");
        assert_eq!(s.system_prompt, default_system_prompt_for("en"));
        s.set_language("ja");
        assert_eq!(s.system_prompt, default_system_prompt_for("ja"));
        s.system_prompt = "User knowledge reference instructions".into();
        s.set_language("en");
        assert_eq!(s.system_prompt, "User knowledge reference instructions");
    }
    #[test]
    fn old_shared_model_migrates_to_selected_provider_only() {
        let mut s = CliSettings {
            model: "existing".into(),
            provider: Provider::Claude,
            ..Default::default()
        };
        s.migrate_model();
        assert_eq!(s.claude_options.model, "existing");
        assert!(s.codex_options.model.is_empty());
        assert!(s.model.is_empty());
    }
}
