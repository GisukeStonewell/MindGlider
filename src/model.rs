use crate::i18n::{msg, tr};
use crate::{conditions::Rule, richtext::RichText};
use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use xmltree::Element;

pub fn new_id() -> String {
    format!("ID_{}", uuid::Uuid::new_v4().simple())
}
pub fn icon_label(name: &str) -> String {
    match name {
        "button_ok" => "✓".into(),
        "button_cancel" => "×".into(),
        "idea" => "✦".into(),
        "help" => "?".into(),
        "yes" => "!".into(),
        "bookmark" => "★".into(),
        _ => name.strip_prefix("full-").unwrap_or(name).into(),
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Style {
    pub color: String,
    pub background: String,
    pub edge_color: String,
    pub font_size: f32,
    pub bold: bool,
    pub italic: bool,
    pub shape: String,
    #[serde(default = "default_family")]
    pub font_family: String,
}
fn default_family() -> String {
    "Segoe UI Variable".into()
}
impl Default for Style {
    fn default() -> Self {
        Self {
            color: "#243047".into(),
            background: "#ffffff".into(),
            edge_color: "#8c9bb0".into(),
            font_size: 16.,
            bold: false,
            italic: false,
            shape: "fork".into(),
            font_family: default_family(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Picture {
    pub uri: String,
    pub width: f32,
    pub height: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: String,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub text: RichText,
    pub note: RichText,
    pub details: RichText,
    pub style: Style,
    pub left: bool,
    pub folded: bool,
    pub icons: Vec<String>,
    pub progress: Option<u8>,
    pub link: String,
    pub cloud: Option<String>,
    pub image: Option<Picture>,
    pub protected: bool,
    pub raw: Option<Element>,
}
impl Node {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: new_id(),
            parent: None,
            children: vec![],
            text: RichText::plain(text),
            note: RichText::default(),
            details: RichText::default(),
            style: Style::default(),
            left: false,
            folded: false,
            icons: vec![],
            progress: None,
            link: String::new(),
            cloud: None,
            image: None,
            protected: false,
            raw: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Connector {
    pub from: String,
    pub to: String,
    pub color: String,
    pub label: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    pub root: String,
    pub nodes: BTreeMap<String, Node>,
    pub connectors: Vec<Connector>,
    pub styles: BTreeMap<String, Style>,
    pub rules: Vec<Rule>,
    pub raw: Element,
}
impl Default for Document {
    fn default() -> Self {
        let mut node = Node::new(tr("model.247"));
        node.style.shape = "oval".into();
        Self {
            root: node.id.clone(),
            nodes: BTreeMap::from([(node.id.clone(), node)]),
            connectors: vec![],
            styles: BTreeMap::new(),
            rules: vec![],
            raw: Element::new("map"),
        }
    }
}
impl Document {
    pub fn subtree(&self, id: &str) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        let mut stack = vec![id.to_owned()];
        while let Some(id) = stack.pop() {
            if result.insert(id.clone())
                && let Some(n) = self.nodes.get(&id)
            {
                stack.extend(n.children.iter().cloned());
            }
        }
        result
    }
    pub fn depth(&self, id: &str) -> usize {
        let mut count = 0;
        let mut cur = self.nodes.get(id);
        while let Some(n) = cur {
            if let Some(p) = &n.parent {
                count += 1;
                cur = self.nodes.get(p);
            } else {
                break;
            }
            if count > self.nodes.len() {
                break;
            }
        }
        count
    }
    pub fn ensure_editable(&self, id: &str) -> Result<()> {
        let n = self
            .nodes
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("{}", msg("model.246", &[])))?;
        let mut protected = n.protected;
        let mut parent = n.parent.as_deref();
        while let Some(id) = parent {
            let p = &self.nodes[id];
            protected |= p.protected;
            parent = p.parent.as_deref();
        }
        ensure!(!protected, "{}", msg("model.245", &[]));
        Ok(())
    }
    pub fn ensure_branch_editable(&self, id: &str) -> Result<()> {
        for id in self.subtree(id) {
            self.ensure_editable(&id)?;
        }
        Ok(())
    }
    pub fn add(&mut self, parent: &str, text: impl Into<String>) -> Result<String> {
        self.ensure_editable(parent)?;
        let p = &self.nodes[parent];
        let mut n = Node::new(text);
        n.parent = Some(parent.into());
        n.left = p.left;
        n.style = p.style.clone();
        // The root has its own shape; new branches use the standard fork.
        if parent == self.root {
            n.style.shape = "fork".into();
        }
        let id = n.id.clone();
        self.nodes.insert(id.clone(), n);
        self.nodes
            .get_mut(parent)
            .unwrap()
            .children
            .push(id.clone());
        Ok(id)
    }
    pub fn move_node(&mut self, id: &str, parent: &str, index: usize) -> Result<()> {
        self.ensure_branch_editable(id)?;
        self.ensure_editable(parent)?;
        ensure!(id != self.root, "{}", msg("model.244", &[]));
        ensure!(
            !self.subtree(id).contains(parent),
            "{}",
            msg("model.243", &[])
        );
        let old = self.nodes[id].parent.clone().unwrap();
        self.ensure_editable(&old)?;
        self.nodes
            .get_mut(&old)
            .unwrap()
            .children
            .retain(|c| c != id);
        let p = self.nodes.get_mut(parent).unwrap();
        p.children.insert(index.min(p.children.len()), id.into());
        let left = if parent == self.root {
            self.nodes[id].left
        } else {
            p.left
        };
        self.nodes.get_mut(id).unwrap().parent = Some(parent.into());
        for child in self.subtree(id) {
            self.nodes.get_mut(&child).unwrap().left = left;
        }
        Ok(())
    }
    pub fn delete(&mut self, id: &str) -> Result<()> {
        ensure!(id != self.root, "{}", msg("model.242", &[]));
        self.ensure_branch_editable(id)?;
        let parent = self.nodes[id].parent.clone().unwrap();
        self.ensure_editable(&parent)?;
        let ids = self.subtree(id);
        self.nodes
            .get_mut(&parent)
            .unwrap()
            .children
            .retain(|c| c != id);
        self.nodes.retain(|k, _| !ids.contains(k));
        self.connectors
            .retain(|c| !ids.contains(&c.from) && !ids.contains(&c.to));
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.nodes.contains_key(&self.root),
            "{}",
            msg("model.241", &[])
        );
        ensure!(
            self.nodes[&self.root].parent.is_none(),
            "{}",
            msg("model.240", &[])
        );
        let mut seen = BTreeSet::new();
        let mut stack = vec![(self.root.clone(), 0)];
        while let Some((id, depth)) = stack.pop() {
            ensure!(depth < 512, "{}", msg("model.239", &[]));
            ensure!(seen.insert(id.clone()), "{}", msg("model.238", &[]));
            let n = &self.nodes[&id];
            ensure!(n.id == id, "{}", msg("model.237", &[]));
            validate_text(&n.text.text())?;
            validate_text(&n.note.text())?;
            validate_text(&n.details.text())?;
            ensure!(
                n.style.font_size.is_finite()
                    && n.style.font_size > 0.
                    && n.style.font_size <= 512.,
                "{}",
                msg("model.236", &[])
            );
            if let Some(p) = &n.image {
                ensure!(
                    p.width.is_finite()
                        && p.height.is_finite()
                        && p.width > 0.
                        && p.height > 0.
                        && p.width <= 10000.
                        && p.height <= 10000.,
                    "{}",
                    msg("model.235", &[])
                );
            }
            for c in &n.children {
                let child = self
                    .nodes
                    .get(c)
                    .ok_or_else(|| anyhow::anyhow!("{}", msg("model.234", &[])))?;
                ensure!(
                    child.parent.as_deref() == Some(&id),
                    "{}",
                    msg("model.233", &[])
                );
                stack.push((c.clone(), depth + 1));
            }
        }
        ensure!(seen.len() == self.nodes.len(), "{}", msg("model.232", &[]));
        for c in &self.connectors {
            ensure!(
                self.nodes.contains_key(&c.from) && self.nodes.contains_key(&c.to),
                "{}",
                msg("model.231", &[])
            );
        }
        Ok(())
    }
    pub fn fingerprint(&self, id: &str) -> String {
        let ids = self.subtree(id);
        format!(
            "{:?}{:?}",
            ids.iter().map(|i| &self.nodes[i]).collect::<Vec<_>>(),
            self.connectors
                .iter()
                .filter(|c| ids.contains(&c.from) || ids.contains(&c.to))
                .collect::<Vec<_>>()
        )
    }
    pub fn top_selected(&self, selection: &BTreeSet<String>) -> Vec<String> {
        selection
            .iter()
            .filter(|id| {
                let mut cur = self.nodes.get(*id).and_then(|n| n.parent.as_deref());
                while let Some(parent) = cur {
                    if selection.contains(parent) {
                        return false;
                    }
                    cur = self.nodes.get(parent).and_then(|n| n.parent.as_deref());
                }
                self.nodes.contains_key(*id)
            })
            .cloned()
            .collect()
    }
}
pub struct History {
    pub doc: Document,
    pub revision: u64,
    pub dirty: bool,
    undo: Vec<(String, Document)>,
    redo: Vec<(String, Document)>,
    saved: Document,
}
impl History {
    pub fn restored(doc: Document, saved: Option<Document>) -> Self {
        let saved = saved.unwrap_or_default();
        Self {
            doc,
            saved,
            revision: 0,
            dirty: true,
            undo: vec![],
            redo: vec![],
        }
    }
    pub fn new(doc: Document) -> Self {
        Self {
            saved: doc.clone(),
            doc,
            revision: 0,
            dirty: false,
            undo: vec![],
            redo: vec![],
        }
    }
    pub fn edit(
        &mut self,
        label: &str,
        apply: impl FnOnce(&mut Document) -> Result<()>,
    ) -> Result<()> {
        let mut next = self.doc.clone();
        apply(&mut next)?;
        next.validate()?;
        if next != self.doc {
            self.undo
                .push((label.into(), std::mem::replace(&mut self.doc, next)));
            if self.undo.len() > 100 {
                self.undo.remove(0);
            }
            self.redo.clear();
            self.changed();
        }
        Ok(())
    }
    fn changed(&mut self) {
        self.revision += 1;
        self.dirty = self.doc != self.saved;
    }
    pub fn undo(&mut self) {
        if let Some((l, doc)) = self.undo.pop() {
            self.redo.push((l, std::mem::replace(&mut self.doc, doc)));
            self.changed();
        }
    }
    pub fn redo(&mut self) {
        if let Some((l, doc)) = self.redo.pop() {
            self.undo.push((l, std::mem::replace(&mut self.doc, doc)));
            self.changed();
        }
    }
    pub fn saved(&mut self) {
        self.saved = self.doc.clone();
        self.dirty = false;
    }
    pub fn saved_version(&mut self, doc: Document) {
        self.saved = doc;
        self.dirty = self.doc != self.saved;
    }
    pub fn update_remote(&mut self, doc: Document, saved: Document) {
        self.doc = doc;
        self.undo.clear();
        self.redo.clear();
        self.saved = saved;
        self.changed();
    }
    pub fn undo_label(&self) -> &str {
        self.undo.last().map(|x| x.0.as_str()).unwrap_or("")
    }
}

pub fn validate_text(text: &str) -> Result<()> {
    if text.len() > 1_000_000
        || text
            .chars()
            .any(|c| matches!(c,'\0'..='\u{8}'|'\u{b}'|'\u{c}'|'\u{e}'..='\u{1f}'))
    {
        bail!("{}", msg("model.230", &[]));
    }
    Ok(())
}
