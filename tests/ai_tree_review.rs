use mindglider::{
    ai::{self, ChangeKind, EditPlan, Operation, Snapshot},
    model::{Document, History},
};
use std::{collections::BTreeSet, time::Instant};

#[test]
fn confirms_large_tree_restructuring_as_one_undo() {
    let mut doc = Document::default();
    let root = doc.root.clone();
    let section = doc.add(&root, "Section").unwrap();
    let reference = doc.add(&section, "First example").unwrap();
    let child = doc.add(&reference, "Example detail").unwrap();
    let deleted = doc.add(&section, "Redundant topic").unwrap();
    doc.add(&deleted, "Redundant detail").unwrap();
    let peer = doc.add(&section, "Other material").unwrap();
    let protected = doc.add(&root, "Protected outside branch").unwrap();
    doc.nodes.get_mut(&protected).unwrap().protected = true;
    let snapshot = Snapshot::capture(&doc, &section, "Reorganize the section").unwrap();
    let plan = EditPlan {
        operations: vec![
            Operation::Add {
                id: reference.clone(),
                parent: section.clone(),
                text: "Case studies".into(),
            },
            Operation::Add {
                id: "ID_new_group".into(),
                parent: section.clone(),
                text: "Evidence".into(),
            },
            Operation::Move {
                id: child.clone(),
                parent: "ID_new_group".into(),
                index: 0,
            },
            Operation::Delete { id: deleted },
            Operation::Move {
                id: peer.clone(),
                parent: section.clone(),
                index: 0,
            },
            Operation::Update {
                id: peer.clone(),
                text: "Recommendations".into(),
                note: Some("Summary".into()),
                details: None,
            },
        ],
    };
    let mut history = History::new(doc.clone());
    let review = ai::preview_tree(&history, &snapshot, &plan, &BTreeSet::new()).unwrap();
    assert_eq!(history.doc, doc);
    assert!(history.undo_label().is_empty());
    assert_eq!(review.renames.len(), 1);
    assert_eq!(review.renames[0].operation, 0);
    assert_eq!(review.renames[0].before, "First example");
    assert_eq!(review.renames[0].after, "Case studies");
    assert_eq!(
        review
            .changes
            .iter()
            .filter(|c| c.kind == ChangeKind::Deleted)
            .count(),
        2
    );
    assert_eq!(
        review
            .changes
            .iter()
            .filter(|c| c.kind == ChangeKind::Added)
            .count(),
        1
    );
    assert!(
        review
            .changes
            .iter()
            .any(|c| c.moved && c.after.as_ref().is_some_and(|n| n.node_id == child))
    );
    assert!(review.changes.iter().any(|c| {
        c.reordered
            && c.after
                .as_ref()
                .is_some_and(|n| n.node_id == peer && n.note == "Summary")
    }));
    assert_eq!(
        ai::apply_review_traced(&mut history, &snapshot, &review, "test", Instant::now()).unwrap(),
        6
    );
    assert_eq!(history.doc.nodes[&reference].text.text(), "Case studies");
    assert_eq!(history.doc.nodes[&protected], doc.nodes[&protected]);
    assert_eq!(
        history.doc.nodes[&child].parent.as_deref(),
        Some("ID_new_group")
    );
    history.undo();
    assert_eq!(history.doc, doc);
    assert!(history.undo_label().is_empty());
}

#[test]
fn choosing_a_new_node_uses_a_distinct_id_and_preserves_the_existing_node() {
    let mut doc = Document::default();
    let root = doc.root.clone();
    let existing = doc.add(&root, "Existing").unwrap();
    let snapshot = Snapshot::capture(&doc, &root, "Add an independent entry").unwrap();
    let plan = EditPlan {
        operations: vec![Operation::Add {
            id: existing.clone(),
            parent: root,
            text: "New".into(),
        }],
    };
    let mut history = History::new(doc.clone());
    let review = ai::preview_tree(&history, &snapshot, &plan, &BTreeSet::from([0])).unwrap();
    assert_eq!(review.changes.len(), 1);
    assert_eq!(review.changes[0].kind, ChangeKind::Added);
    let added = review.changes[0].after.as_ref().unwrap().node_id.clone();
    assert_ne!(added, existing);
    assert!(!doc.nodes.contains_key(&added));
    ai::apply_review_traced(&mut history, &snapshot, &review, "test", Instant::now()).unwrap();
    assert_eq!(history.doc.nodes[&existing], doc.nodes[&existing]);
    assert_eq!(history.doc.nodes[&added].text.text(), "New");
    history.undo();
    assert_eq!(history.doc, doc);
}

#[test]
fn rejects_ambiguous_parents_and_stale_reviews_without_losing_unrelated_edits() {
    let mut doc = Document::default();
    let root = doc.root.clone();
    let section = doc.add(&root, "Section").unwrap();
    let node = doc.add(&section, "Child").unwrap();
    let parent = doc.add(&section, "Other parent").unwrap();
    let outside = doc.add(&root, "Outside").unwrap();
    let snapshot = Snapshot::capture(&doc, &section, "Reorganize").unwrap();
    let mut history = History::new(doc);
    let ambiguous = EditPlan {
        operations: vec![Operation::Add {
            id: node.clone(),
            parent,
            text: "Different branch".into(),
        }],
    };
    let error = ai::preview_tree(&history, &snapshot, &ambiguous, &BTreeSet::new())
        .err()
        .unwrap();
    assert_eq!(ai::failure_code(&error), "APPLY_DUPLICATE_NODE_ID");
    let plan = EditPlan {
        operations: vec![Operation::Add {
            id: node.clone(),
            parent: section,
            text: "Proposed".into(),
        }],
    };
    let review = ai::preview_tree(&history, &snapshot, &plan, &BTreeSet::new()).unwrap();
    history
        .doc
        .nodes
        .get_mut(&outside)
        .unwrap()
        .text
        .edit("Independent edit");
    ai::apply_review_traced(&mut history, &snapshot, &review, "test", Instant::now()).unwrap();
    assert_eq!(history.doc.nodes[&outside].text.text(), "Independent edit");
    history.undo();
    history
        .doc
        .nodes
        .get_mut(&node)
        .unwrap()
        .text
        .edit("Changed during review");
    let doc = history.doc.clone();
    let error = ai::apply_review_traced(&mut history, &snapshot, &review, "test", Instant::now())
        .unwrap_err();
    assert_eq!(ai::failure_code(&error), "APPLY_TARGET_CHANGED");
    assert_eq!(history.doc, doc);
}
