use crate::{
    conditions::{Condition, resolved_style},
    model::{Document, Style},
};
use eframe::egui::{Pos2, Rect, Vec2};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Clone, Debug)]
pub struct Placed {
    pub id: String,
    pub rect: Rect,
    pub style: Style,
    pub text_height: f32,
    pub text_width: f32,
}
#[derive(Clone, Debug)]
pub struct Scene {
    pub nodes: Vec<Placed>,
    pub bounds: Rect,
    pub edges: Vec<(String, String)>,
    pub clouds: Vec<(Rect, String)>,
}
pub fn icon_count(node: &crate::model::Node) -> usize {
    node.icons.len() + usize::from(node.progress.is_some())
}
pub fn icon_width(count: usize) -> f32 {
    count as f32 * 18.
}
pub fn padding(style: &Style) -> Vec2 {
    if style.shape == "fork" {
        Vec2::new(4., 2.)
    } else {
        Vec2::new(16., 8.)
    }
}
pub const ICON_SIZE: f32 = 16.;
pub const SIBLING_GAP: f32 = 4.;
pub const BRANCH_GAP: f32 = 32.;
pub fn core_height(node: &crate::model::Node, text_height: f32) -> f32 {
    text_height.max(if icon_count(node) == 0 { 0. } else { ICON_SIZE })
}
pub fn core_text_rect(node: &crate::model::Node, placed: &Placed) -> Rect {
    let pad = padding(&placed.style);
    Rect::from_min_size(
        placed.rect.min + Vec2::new(pad.x + icon_width(icon_count(node)), pad.y),
        Vec2::new(placed.text_width, placed.text_height),
    )
}
pub fn branch_anchor(p: &Placed, side: f32) -> Pos2 {
    Pos2::new(
        p.rect.center().x + side * p.rect.width() / 2.,
        if p.style.shape == "fork" {
            p.rect.bottom()
        } else {
            p.rect.center().y
        },
    )
}
impl Default for Scene {
    fn default() -> Self {
        Self {
            nodes: vec![],
            bounds: Rect::ZERO,
            edges: vec![],
            clouds: vec![],
        }
    }
}
#[derive(Default)]
pub struct LayoutCache {
    measurements: HashMap<String, Vec2>,
    heights: HashMap<String, (String, f32)>,
    pub scene: Scene,
    pub generation: u64,
}
impl LayoutCache {
    pub fn build(
        &mut self,
        doc: &Document,
        filter: Option<&Condition>,
        mut measure: impl FnMut(&str, f32) -> Vec2,
    ) -> Scene {
        let mut keep = BTreeSet::new();
        for id in doc.nodes.keys() {
            if filter.is_none_or(|c| c.matches(doc, id)) {
                let mut cur = Some(id.clone());
                while let Some(id) = cur {
                    keep.insert(id.clone());
                    cur = doc.nodes[&id].parent.clone();
                }
            }
        }
        keep.insert(doc.root.clone());
        let mut sizes = BTreeMap::new();
        let mut styles = BTreeMap::new();
        for id in &keep {
            let n = &doc.nodes[id];
            let style = resolved_style(doc, id);
            let text = n.text.text();
            let key = format!("{text}:{:?}:{:?}", style, n.text.runs);
            let font_size = n
                .text
                .runs
                .iter()
                .filter_map(|r| r.marks.size)
                .fold(style.font_size, f32::max);
            let mut size = *self
                .measurements
                .entry(key)
                .or_insert_with(|| measure(&text, font_size));
            if style.bold || n.text.runs.iter().any(|r| r.marks.bold) {
                size.x *= 1.1;
            }
            let pad = padding(&style);
            let iw = n.image.as_ref().map(|i| i.width).unwrap_or(0.);
            let ih = n.image.as_ref().map(|i| i.height + 6.).unwrap_or(0.);
            let badge = if n.note.text().is_empty() && !(n.folded && !n.children.is_empty()) {
                0.
            } else {
                16.
            };
            let details = if n.details.text().is_empty() { 0. } else { 16. };
            sizes.insert(
                id.clone(),
                (
                    Vec2::new(
                        (size.x + 2. * pad.x + icon_width(icon_count(n)))
                            .max(iw + 2. * pad.x)
                            .max(24.),
                        core_height(n, size.y) + 2. * pad.y + ih + badge + details,
                    ),
                    size.y,
                    size.x,
                ),
            );
            styles.insert(id.clone(), style);
        }
        fn height(
            id: &str,
            doc: &Document,
            sizes: &BTreeMap<String, (Vec2, f32, f32)>,
            memo: &mut HashMap<String, (String, f32)>,
        ) -> f32 {
            let n = &doc.nodes[id];
            let children: Vec<_> = if n.folded {
                vec![]
            } else {
                n.children
                    .iter()
                    .filter(|c| sizes.contains_key(*c))
                    .collect()
            };
            let hs: Vec<f32> = children
                .iter()
                .map(|c| height(c, doc, sizes, memo))
                .collect();
            let key = format!("{:?}:{:?}:{:?}", sizes[id].0, children, hs);
            if let Some((k, h)) = memo.get(id).filter(|(k, _)| *k == key) {
                let _ = k;
                return *h;
            }
            let total = hs.iter().sum::<f32>() + (hs.len().saturating_sub(1) as f32) * SIBLING_GAP;
            let h = sizes[id].0.y.max(total);
            memo.insert(id.into(), (key, h));
            h
        }
        height(&doc.root, doc, &sizes, &mut self.heights);
        let mut scene = Scene::default();
        // The recursive visitor shares immutable layout inputs and one output scene.
        #[allow(clippy::too_many_arguments)]
        fn place(
            id: &str,
            x: f32,
            y: f32,
            left: bool,
            doc: &Document,
            sizes: &BTreeMap<String, (Vec2, f32, f32)>,
            styles: &BTreeMap<String, Style>,
            heights: &HashMap<String, (String, f32)>,
            scene: &mut Scene,
        ) {
            let (size, th, tw) = sizes[id];
            let rect = Rect::from_center_size(Pos2::new(x, y), size);
            scene.nodes.push(Placed {
                id: id.into(),
                rect,
                style: styles[id].clone(),
                text_height: th,
                text_width: tw,
            });
            let n = &doc.nodes[id];
            if n.folded {
                return;
            }
            for side in [true, false] {
                let children: Vec<_> = n
                    .children
                    .iter()
                    .filter(|c| {
                        sizes.contains_key(*c)
                            && (if id == doc.root {
                                doc.nodes[*c].left == side
                            } else {
                                left == side
                            })
                    })
                    .collect();
                let total = children.iter().map(|c| heights[*c].1).sum::<f32>()
                    + SIBLING_GAP * children.len().saturating_sub(1) as f32;
                let mut top = y - total / 2.;
                for c in children {
                    let h = heights[c].1;
                    let cx = x
                        + (if side { -1. } else { 1. })
                            * (size.x / 2. + BRANCH_GAP + sizes[c].0.x / 2.);
                    scene.edges.push((id.into(), c.clone()));
                    place(
                        c,
                        cx,
                        top + h / 2.,
                        side,
                        doc,
                        sizes,
                        styles,
                        heights,
                        scene,
                    );
                    top += h + SIBLING_GAP;
                }
            }
        }
        place(
            &doc.root,
            0.,
            0.,
            false,
            doc,
            &sizes,
            &styles,
            &self.heights,
            &mut scene,
        );
        scene.bounds = scene
            .nodes
            .iter()
            .map(|n| n.rect)
            .reduce(Rect::union)
            .unwrap_or(Rect::ZERO)
            .expand(30.);
        for p in &scene.nodes {
            if let Some(c) = &doc.nodes[&p.id].cloud {
                let ids = doc.subtree(&p.id);
                let rect = scene
                    .nodes
                    .iter()
                    .filter(|n| ids.contains(&n.id))
                    .map(|n| n.rect)
                    .reduce(Rect::union)
                    .unwrap_or(p.rect)
                    .expand(14.);
                scene.clouds.push((rect, c.clone()));
            }
        }
        self.scene = scene.clone();
        self.generation += 1;
        scene
    }
}
pub fn color(s: &str) -> eframe::egui::Color32 {
    let s = s.trim_start_matches('#');
    if s.len() == 6
        && let Ok(n) = u32::from_str_radix(s, 16)
    {
        return eframe::egui::Color32::from_rgb((n >> 16) as u8, (n >> 8) as u8, n as u8);
    }
    eframe::egui::Color32::GRAY
}
