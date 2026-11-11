use anyhow::{Result, ensure};
use mindglider::model::{Document, Node};
use std::collections::BTreeSet;

fn combine<T: Clone + PartialEq>(base: &T, local: &T, remote: &T) -> Result<T> {
    if local == base || local == remote {
        return Ok(remote.clone());
    }
    ensure!(remote == base, "Concurrent changes");
    Ok(local.clone())
}

fn merge_node(base: &Node, local: &Node, remote: &Node) -> Result<Node> {
    let mut node = remote.clone();
    macro_rules! merge {
        ($($field:ident),+ $(,)?) => {
            $(node.$field = combine(&base.$field, &local.$field, &remote.$field)?;)+
        };
    }
    merge!(
        parent, children, text, note, details, style, left, folded, icons, progress, link, cloud,
        image, protected,
    );
    // The latest XML is the serialization template. The existing writer updates known fields.
    ensure!(local.raw == base.raw, "Local XML template changed");
    Ok(node)
}

pub fn merge(base: &Document, local: &Document, remote: &Document) -> Result<Document> {
    ensure!(
        base.root == local.root && base.root == remote.root,
        "Root changed"
    );
    let mut merged = remote.clone();
    merged.connectors = combine(&base.connectors, &local.connectors, &remote.connectors)?;
    merged.styles = combine(&base.styles, &local.styles, &remote.styles)?;
    merged.rules = combine(&base.rules, &local.rules, &remote.rules)?;
    let ids: BTreeSet<_> = base
        .nodes
        .keys()
        .chain(local.nodes.keys())
        .chain(remote.nodes.keys())
        .collect();
    for id in ids {
        let before = base.nodes.get(id);
        let ours = local.nodes.get(id);
        let theirs = remote.nodes.get(id);
        let node = match (before, ours, theirs) {
            (Some(b), Some(l), Some(r)) => Some(merge_node(b, l, r)?),
            _ => combine(&before, &ours, &theirs)?.cloned(),
        };
        if let Some(node) = node {
            merged.nodes.insert(id.clone(), node);
        } else {
            merged.nodes.remove(id);
        }
    }
    merged.validate()?;
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mindglider::mm;

    #[test]
    fn separate_branches_merge_and_same_text_conflicts() {
        let mut base = Document::default();
        let root = base.root.clone();
        let a = base.add(&root, "A").unwrap();
        let b = base.add(&root, "B").unwrap();
        let base = mm::read(&mm::write(&base).unwrap()).unwrap();
        let mut local = base.clone();
        let mut remote = base.clone();
        local.nodes.get_mut(&a).unwrap().text.edit("Local A");
        remote.nodes.get_mut(&b).unwrap().text.edit("Remote B");
        let remote = mm::read(&mm::write(&remote).unwrap()).unwrap();
        let merged = merge(&base, &local, &remote).unwrap();
        let saved = mm::read(&mm::write(&merged).unwrap()).unwrap();
        assert_eq!(saved.nodes[&a].text.text(), "Local A");
        assert_eq!(saved.nodes[&b].text.text(), "Remote B");

        let mut remote = remote;
        remote.nodes.get_mut(&a).unwrap().text.edit("Remote A");
        assert!(merge(&base, &local, &remote).is_err());
        remote.nodes.get_mut(&a).unwrap().text.edit("A");
        remote.delete(&a).unwrap();
        assert!(merge(&base, &local, &remote).is_err());
    }

    #[test]
    fn save_acknowledgement_keeps_edits_made_in_flight() {
        use mindglider::model::History;
        let base = Document::default();
        let root = base.root.clone();
        let mut history = History::new(base);
        history
            .edit("first", |doc| doc.add(&root, "first").map(|_| ()))
            .unwrap();
        let sent = history.doc.clone();
        history
            .edit("second", |doc| doc.add(&root, "second").map(|_| ()))
            .unwrap();
        history.saved_version(sent.clone());
        assert!(history.dirty);
        assert_eq!(history.doc.nodes[&root].children.len(), 2);
        history.undo();
        assert_eq!(history.doc, sent);
        assert!(!history.dirty);
    }
}
