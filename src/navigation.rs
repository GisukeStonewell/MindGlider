//! Directional selection uses only placed nodes (folded/filtered descendants are excluded).
use crate::{layout::Scene, model::Document};
use eframe::egui::{Key, Vec2};
pub fn target(doc: &Document, scene: &Scene, current: &str, key: Key) -> Option<String> {
    let node = doc.nodes.get(current)?;
    let placed = scene.nodes.iter().find(|p| p.id == current)?;
    let visible = |id: &String| scene.nodes.iter().any(|p| &p.id == id);
    if matches!(key, Key::ArrowLeft | Key::ArrowRight) {
        if current == doc.root {
            return node
                .children
                .iter()
                .find(|id| doc.nodes[*id].left == (key == Key::ArrowLeft) && visible(id))
                .cloned();
        }
        let outward = if node.left {
            Key::ArrowLeft
        } else {
            Key::ArrowRight
        };
        return if key == outward {
            node.children.iter().find(|id| visible(id)).cloned()
        } else {
            node.parent.clone().filter(visible)
        };
    }
    let sign = if key == Key::ArrowUp {
        -1.
    } else if key == Key::ArrowDown {
        1.
    } else {
        return None;
    };
    scene
        .nodes
        .iter()
        .filter(|p| p.id != current && (p.rect.center().y - placed.rect.center().y) * sign > 0.5)
        .filter(|p| current == doc.root || p.id == doc.root || doc.nodes[&p.id].left == node.left)
        .min_by(|a, b| {
            let score = |p: &crate::layout::Placed| {
                let d = p.rect.center() - placed.rect.center();
                d.y * d.y + d.x * d.x * 0.25
            };
            score(a).total_cmp(&score(b)).then(a.id.cmp(&b.id))
        })
        .map(|p| p.id.clone())
}
/// Positive viewport scrolling moves the canvas in the opposite direction.
pub fn scroll_delta(key: Key, seconds: f32) -> Vec2 {
    let direction = match key {
        Key::ArrowLeft => Vec2::X,
        Key::ArrowRight => -Vec2::X,
        Key::ArrowUp => Vec2::Y,
        Key::ArrowDown => -Vec2::Y,
        _ => Vec2::ZERO,
    };
    direction * 300. * seconds.clamp(0., 0.05)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{layout::LayoutCache, model::Document};
    use eframe::egui::{Pos2, Rect, vec2};
    #[test]
    fn vertical_navigation_can_cross_levels_and_ignores_hidden_nodes() {
        let mut d = Document::default();
        let root = d.root.clone();
        let a = d.add(&root, "upper parent").unwrap();
        let deep = d.add(&a, "deep leaf").unwrap();
        let b = d.add(&root, "lower ancestor sibling").unwrap();
        let mut scene = LayoutCache::default().build(&d, None, |_, _| vec2(50., 20.));
        for p in &mut scene.nodes {
            let xy = if p.id == deep {
                (220., 0.)
            } else if p.id == a {
                (100., -30.)
            } else if p.id == b {
                (100., 35.)
            } else {
                (0., -30.)
            };
            p.rect = Rect::from_center_size(Pos2::new(xy.0, xy.1), vec2(50., 20.));
        }
        assert_eq!(target(&d, &scene, &deep, Key::ArrowDown), Some(b.clone()));
        assert_eq!(target(&d, &scene, &b, Key::ArrowDown), None);
        scene.nodes.retain(|p| p.id != b);
        assert_eq!(target(&d, &scene, &deep, Key::ArrowDown), None);
    }
    #[test]
    fn horizontal_navigation_follows_the_visual_side() {
        let mut d = Document::default();
        let root = d.root.clone();
        let left = d.add(&root, "left").unwrap();
        d.nodes.get_mut(&left).unwrap().left = true;
        let child = d.add(&left, "left child").unwrap();
        let scene = LayoutCache::default().build(&d, None, |_, _| vec2(50., 20.));
        assert_eq!(
            target(&d, &scene, &root, Key::ArrowLeft),
            Some(left.clone())
        );
        assert_eq!(
            target(&d, &scene, &left, Key::ArrowLeft),
            Some(child.clone())
        );
        assert_eq!(target(&d, &scene, &child, Key::ArrowRight), Some(left));
        assert_eq!(target(&d, &scene, &child, Key::ArrowLeft), None);
    }
    #[test]
    fn edge_scroll_accumulates_without_key_repeat_events() {
        let mut pan = Vec2::ZERO;
        for _ in 0..60 {
            pan += scroll_delta(Key::ArrowRight, 1. / 60.);
        }
        assert!((pan.x + 300.).abs() < 0.01);
        assert_eq!(pan.y, 0.);
        assert!(scroll_delta(Key::ArrowUp, 0.01).y > 0.);
        assert!(scroll_delta(Key::ArrowDown, 0.01).y < 0.);
    }
}
