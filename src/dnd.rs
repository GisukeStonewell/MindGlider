//! Freeplane-style node drop zones; previews and commits share this validation.
use crate::model::Document;
use anyhow::{Result, ensure};
use eframe::egui::{Pos2, Rect};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Left,
    Right,
    Before,
    After,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drop {
    pub source: String,
    pub target: String,
    pub zone: Zone,
    pub parent: String,
    pub index: usize,
    pub left: bool,
}
pub fn zone(rect: Rect, point: Pos2, root: bool) -> Zone {
    let x = (point.x - rect.left()) / rect.width();
    let y = (point.y - rect.top()) / rect.height();
    if root {
        return if x < 0.5 { Zone::Left } else { Zone::Right };
    }
    if x < 1. / 6. {
        Zone::Left
    } else if x >= 5. / 6. {
        Zone::Right
    } else if y < 1. / 6. {
        Zone::Before
    } else if y >= 5. / 6. {
        Zone::After
    } else if x < 0.25 {
        Zone::Left
    } else if x >= 0.75 {
        Zone::Right
    } else if y < 0.5 {
        Zone::Before
    } else {
        Zone::After
    }
}
pub fn destination(doc: &Document, source: &str, target: &str, zone: Zone) -> Result<Drop> {
    doc.ensure_branch_editable(source)?;
    doc.ensure_editable(target)?;
    ensure!(
        source != doc.root && !doc.subtree(source).contains(target),
        "{}",
        crate::i18n::tr("dnd.invalid")
    );
    let n = &doc.nodes[target];
    let (parent, mut index, left) = match zone {
        Zone::Left | Zone::Right => {
            let left = zone == Zone::Left;
            ensure!(
                target == doc.root || n.left == left,
                "{}",
                crate::i18n::tr("dnd.invalid")
            );
            (target.to_string(), n.children.len(), left)
        }
        Zone::Before | Zone::After => {
            let parent = n
                .parent
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("{}", crate::i18n::tr("dnd.invalid")))?;
            let i = doc.nodes[parent]
                .children
                .iter()
                .position(|id| id == target)
                .unwrap()
                + usize::from(zone == Zone::After);
            (parent.clone(), i, n.left)
        }
    };
    doc.ensure_editable(&parent)?;
    let old = doc.nodes[source].parent.as_ref().unwrap();
    doc.ensure_editable(old)?;
    if old == &parent
        && let Some(i) = doc.nodes[&parent]
            .children
            .iter()
            .position(|id| id == source)
        && i < index
    {
        index -= 1;
    }
    Ok(Drop {
        source: source.into(),
        target: target.into(),
        zone,
        parent,
        index,
        left,
    })
}
pub fn apply(doc: &mut Document, drop: &Drop) -> Result<()> {
    let drop = destination(doc, &drop.source, &drop.target, drop.zone)?;
    doc.move_node(&drop.source, &drop.parent, drop.index)?;
    for id in doc.subtree(&drop.source) {
        doc.nodes.get_mut(&id).unwrap().left = drop.left;
    }
    doc.nodes.get_mut(&drop.parent).unwrap().folded = false;
    Ok(())
}
/// Region highlighted when hovering a drop target.
pub fn highlight(rect: Rect, zone: Zone) -> Rect {
    match zone {
        Zone::Left => Rect::from_min_max(rect.min, Pos2::new(rect.center().x, rect.bottom())),
        Zone::Right => Rect::from_min_max(Pos2::new(rect.center().x, rect.top()), rect.max),
        Zone::Before => Rect::from_min_max(rect.min, Pos2::new(rect.right(), rect.center().y)),
        Zone::After => Rect::from_min_max(Pos2::new(rect.left(), rect.center().y), rect.max),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Document, History};
    use eframe::egui::{pos2, vec2};
    fn sample() -> (Document, String, String, String) {
        let mut d = Document::default();
        let root = d.root.clone();
        let a = d.add(&root, "a").unwrap();
        let b = d.add(&root, "b").unwrap();
        let child = d.add(&a, "child").unwrap();
        (d, a, b, child)
    }
    #[test]
    fn root_accepts_both_sides_and_undo_restores_the_whole_branch() {
        let (d, a, _, child) = sample();
        let mut h = History::new(d);
        let before = h.doc.clone();
        let root = h.doc.root.clone();
        let p = destination(&h.doc, &a, &root, Zone::Left).unwrap();
        h.edit("left", |d| apply(d, &p)).unwrap();
        assert!(h.doc.nodes[&a].left && h.doc.nodes[&child].left);
        h.undo();
        assert_eq!(h.doc, before);
        h.redo();
        let p = destination(&h.doc, &a, &root, Zone::Right).unwrap();
        h.edit("right", |d| apply(d, &p)).unwrap();
        assert!(!h.doc.nodes[&a].left && !h.doc.nodes[&child].left);
    }
    #[test]
    fn left_branch_accepts_children_on_left_and_rejects_inward_drop() {
        let (mut d, a, b, child) = sample();
        let root = d.root.clone();
        let p = destination(&d, &a, &root, Zone::Left).unwrap();
        apply(&mut d, &p).unwrap();
        assert!(destination(&d, &b, &a, Zone::Right).is_err());
        let p = destination(&d, &b, &a, Zone::Left).unwrap();
        apply(&mut d, &p).unwrap();
        assert_eq!(d.nodes[&b].parent.as_deref(), Some(a.as_str()));
        assert!(d.nodes[&b].left && d.nodes[&child].left);
        let scene = crate::layout::LayoutCache::default().build(&d, None, |_, _| vec2(40., 20.));
        let get = |id: &str| {
            scene
                .nodes
                .iter()
                .find(|p| p.id == id)
                .unwrap()
                .rect
                .center()
                .x
        };
        assert!(get(&b) < get(&a) && get(&a) < get(&root));
    }
    #[test]
    fn sibling_reordering_accounts_for_removing_original_index() {
        let (mut d, a, b, _) = sample();
        let root = d.root.clone();
        let c = d.add(&root, "c").unwrap();
        let p = destination(&d, &a, &b, Zone::After).unwrap();
        apply(&mut d, &p).unwrap();
        assert_eq!(
            d.nodes[&root].children,
            vec![b.clone(), a.clone(), c.clone()]
        );
        let p = destination(&d, &c, &a, Zone::Before).unwrap();
        apply(&mut d, &p).unwrap();
        assert_eq!(d.nodes[&root].children, vec![b, c, a]);
    }
    #[test]
    fn invalid_drop_does_not_change_protected_or_cyclic_tree() {
        let (mut d, a, b, child) = sample();
        let before = d.clone();
        assert!(destination(&d, &a, &child, Zone::Right).is_err());
        assert!(destination(&d, &d.root, &b, Zone::Right).is_err());
        assert_eq!(d, before);
        d.nodes.get_mut(&a).unwrap().protected = true;
        assert!(destination(&d, &a, &b, Zone::Right).is_err());
        assert!(destination(&d, &b, &a, Zone::Right).is_err());
    }
    #[test]
    fn hit_zones_and_highlight_follow_pointer_position() {
        let r = Rect::from_min_size(Pos2::ZERO, vec2(120., 30.));
        for (point, expected) in [
            (pos2(4., 15.), Zone::Left),
            (pos2(116., 15.), Zone::Right),
            (pos2(60., 3.), Zone::Before),
            (pos2(60., 27.), Zone::After),
        ] {
            assert_eq!(zone(r, point, false), expected);
            assert!(highlight(r, expected).contains(point));
        }
        assert_eq!(zone(r, pos2(30., 15.), true), Zone::Left);
        assert_eq!(zone(r, pos2(90., 15.), true), Zone::Right);
    }
}
