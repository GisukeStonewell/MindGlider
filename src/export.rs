use crate::i18n::msg;
use crate::{layout::Scene, model::Document, richtext::escape, storage};
use anyhow::{Result, ensure};
use base64::Engine;
use eframe::egui::{Rect, Vec2};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageSetup {
    pub width_mm: f32,
    pub height_mm: f32,
    pub margin_mm: f32,
    pub tile: bool,
}
impl Default for PageSetup {
    fn default() -> Self {
        Self {
            width_mm: 210.,
            height_mm: 297.,
            margin_mm: 12.,
            tile: false,
        }
    }
}
pub fn outline(doc: &Document, root: &str, format: &str, base: Option<&Path>) -> Result<String> {
    let mut out = String::new();
    fn walk(
        doc: &Document,
        id: &str,
        level: usize,
        format: &str,
        base: Option<&Path>,
        out: &mut String,
    ) -> Result<()> {
        let n = &doc.nodes[id];
        let text = n.text.text();
        match format {
            "txt" => out.push_str(&format!("{}{}\n", "  ".repeat(level), text)),
            "md" => {
                out.push_str(&format!(
                    "{}- {}\n",
                    "  ".repeat(level),
                    text.replace('\n', " ")
                ));
                for body in [&n.details, &n.note] {
                    if !body.text().is_empty() {
                        out.push_str(&format!(
                            "{}{}\n",
                            "  ".repeat(level + 1),
                            body.text().replace('\n', " ")
                        ));
                    }
                }
            }
            "opml" => out.push_str(&format!(
                "<outline text=\"{}\" _note=\"{}\" _details=\"{}\">",
                escape(&text),
                escape(&n.note.text()),
                escape(&n.details.text())
            )),
            "html" => {
                out.push_str(&format!(
                    "<li id=\"{}\"><div class=\"node\">{}</div>",
                    escape(id),
                    n.text.html()
                ));
                if !n.link.is_empty() {
                    out.push_str(&format!(
                        "<p><a href=\"{}\">{}</a></p>",
                        escape(&safe_link(&n.link)),
                        escape(&n.link)
                    ));
                }
                if !n.details.text().is_empty() {
                    out.push_str(&format!(
                        "<div class=\"details\">{}</div>",
                        n.details.html()
                    ));
                }
                if !n.note.text().is_empty() {
                    out.push_str(&format!("<div class=\"note\">{}</div>", n.note.html()));
                }
                if let Some(p) = &n.image {
                    let uri = image_data(&storage::resolve(base, &p.uri))?;
                    out.push_str(&format!(
                        "<img src=\"{uri}\" width=\"{}\" height=\"{}\" alt=\"{}\"/>",
                        p.width,
                        p.height,
                        escape(&text)
                    ));
                }
                if !n.children.is_empty() {
                    out.push_str("<ul>");
                }
            }
            _ => anyhow::bail!("{}", msg("export.219", &[])),
        }
        for c in &n.children {
            walk(doc, c, level + 1, format, base, out)?;
        }
        match format {
            "opml" => out.push_str("</outline>"),
            "html" => {
                if !n.children.is_empty() {
                    out.push_str("</ul>");
                }
                out.push_str("</li>");
            }
            _ => {}
        }
        Ok(())
    }
    walk(doc, root, 0, format, base, &mut out)?;
    Ok(match format {
        "opml" => format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><opml version=\"2.0\"><head><title>{}</title></head><body>{out}</body></opml>",
            escape(&doc.nodes[root].text.text())
        ),
        "html" => format!(
            "<!doctype html><html lang=\"ja\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><title>{}</title><style>body{{font:16px 'Yu Gothic',sans-serif;max-width:1000px;margin:40px auto;padding:20px;color:#243047}}li{{margin:12px 0}}.node{{font-weight:600}}.details{{color:#536273}}.note{{white-space:pre-wrap;padding:8px 16px;border-left:3px solid #8ba8ce}}img{{max-width:100%;object-fit:contain}}</style><body><ul>{out}</ul></body></html>",
            escape(&doc.nodes[root].text.text())
        ),
        _ => out,
    })
}
fn safe_link(s: &str) -> String {
    let l = s.to_lowercase();
    if l.starts_with("javascript:") || l.starts_with("data:") || l.starts_with("vbscript:") {
        String::new()
    } else {
        s.into()
    }
}
pub fn image_data(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    let kind = image::guess_format(&bytes)?;
    let mime = match kind {
        image::ImageFormat::Jpeg => "image/jpeg",
        image::ImageFormat::Gif => "image/gif",
        image::ImageFormat::WebP => "image/webp",
        _ => "image/png",
    };
    Ok(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}
pub fn svg(doc: &Document, scene: &Scene, base: Option<&Path>) -> Result<String> {
    svg_view(doc, scene, base, scene.bounds, scene.bounds.size())
}
pub fn svg_view(
    doc: &Document,
    scene: &Scene,
    base: Option<&Path>,
    view: Rect,
    size: Vec2,
) -> Result<String> {
    ensure!(
        size.x.is_finite() && size.y.is_finite() && size.x > 0. && size.y > 0.,
        "{}",
        msg("export.218", &[])
    );
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"{}\" height=\"{}\" viewBox=\"{} {} {} {}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"white\"/>",
        size.x,
        size.y,
        view.min.x,
        view.min.y,
        view.width(),
        view.height(),
        view.min.x,
        view.min.y,
        view.width(),
        view.height()
    );
    let by_id: BTreeMap<_, _> = scene.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    for (r, c) in &scene.clouds {
        s.push_str(&format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"20\" fill=\"{}\" fill-opacity=\"0.45\"/>",r.min.x,r.min.y,r.width(),r.height(),escape(c)));
    }
    for (a, b) in &scene.edges {
        if let (Some(a), Some(b)) = (by_id.get(a.as_str()), by_id.get(b.as_str())) {
            let sign = if b.rect.center().x < a.rect.center().x {
                -1.
            } else {
                1.
            };
            let x1 = a.rect.center().x + sign * a.rect.width() / 2.;
            let x2 = b.rect.center().x - sign * b.rect.width() / 2.;
            let y1 = crate::layout::branch_anchor(a, sign).y;
            let y2 = crate::layout::branch_anchor(b, -sign).y;
            s.push_str(&format!("<path d=\"M{x1},{y1} C{},{} {},{} {x2},{y2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\"/>",(x1+x2)/2.,y1,(x1+x2)/2.,y2,escape(&b.style.edge_color)));
        }
    }
    for c in &doc.connectors {
        if let (Some(a), Some(b)) = (by_id.get(c.from.as_str()), by_id.get(c.to.as_str())) {
            let a = a.rect.center();
            let b = b.rect.center();
            s.push_str(&format!("<path d=\"M{},{} Q{},{} {},{}\" fill=\"none\" stroke=\"{}\" stroke-dasharray=\"6 4\"/><text x=\"{}\" y=\"{}\" font-size=\"12\">{}</text>",a.x,a.y,(a.x+b.x)/2.,a.y.min(b.y)-40.,b.x,b.y,escape(&c.color),(a.x+b.x)/2.,(a.y+b.y)/2.-10.,escape(&c.label)));
        }
    }
    for p in &scene.nodes {
        let n = &doc.nodes[&p.id];
        let r = p.rect;
        let st = &p.style;
        let pad = crate::layout::padding(st);
        let paint = format!(
            "fill=\"{}\" stroke=\"{}\"",
            escape(&st.background),
            escape(&st.edge_color)
        );
        match st.shape.as_str() {
            "oval" | "circle" => s.push_str(&format!(
                "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" {paint}/>",
                r.center().x,
                r.center().y,
                r.width() / 2.,
                r.height() / 2.
            )),
            "wide_hexagon" | "narrow_hexagon" => {
                let dx = r.width() * 0.12;
                s.push_str(&format!(
                    "<polygon points=\"{},{} {},{} {},{} {},{} {},{} {},{}\" {paint}/>",
                    r.min.x + dx,
                    r.min.y,
                    r.max.x - dx,
                    r.min.y,
                    r.max.x,
                    r.center().y,
                    r.max.x - dx,
                    r.max.y,
                    r.min.x + dx,
                    r.max.y,
                    r.min.x,
                    r.center().y
                ));
            }
            "fork" => {
                let core = crate::layout::core_text_rect(n, p);
                s.push_str(&format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"/><path d=\"M{},{} L{},{}\" fill=\"none\" stroke=\"{}\"/>",core.left(),core.top(),core.width(),core.height(),escape(&st.background),r.left(),r.bottom(),r.right(),r.bottom(),escape(&st.edge_color)));
            }
            _ => {
                let rx = if st.shape == "rectangle" { 2. } else { 18. };
                s.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{rx}\" {paint}/>",
                    r.min.x,
                    r.min.y,
                    r.width(),
                    r.height()
                ));
            }
        }
        let family = format!(
            "{},Segoe UI Variable,Yu Gothic UI,sans-serif",
            escape(&st.font_family)
        );
        let mut y = r.min.y + pad.y + st.font_size;
        let text_x = r.min.x + pad.x + crate::layout::icon_width(crate::layout::icon_count(n));
        let mut x = text_x;
        let visual_icons: Vec<_> = n
            .icons
            .iter()
            .cloned()
            .chain(n.progress.map(|p| format!("{p}%")))
            .collect();
        for (index, icon) in visual_icons.iter().enumerate() {
            s.push_str(&format!(
                "<g transform=\"translate({} {}) scale({})\">{}</g>",
                r.min.x + pad.x + index as f32 * 18.,
                r.min.y + pad.y,
                16. / 24.,
                crate::icons::body(icon)
            ));
        }
        for run in &n.text.runs {
            let size = run.marks.size.unwrap_or(st.font_size);
            for (line_index, line) in run.text.split('\n').enumerate() {
                if line_index > 0 {
                    y += size * 1.3;
                    x = text_x;
                }
                if !line.is_empty() {
                    s.push_str(&format!("<text x=\"{x}\" y=\"{y}\" font-family=\"{family}\" font-size=\"{size}\" font-weight=\"{}\" font-style=\"{}\" text-decoration=\"{}\" fill=\"{}\">{}</text>",if run.marks.bold||st.bold{"bold"}else{"normal"},if run.marks.italic||st.italic{"italic"}else{"normal"},if run.marks.underline{"underline"}else if run.marks.strike{"line-through"}else{"none"},escape(run.marks.color.as_ref().unwrap_or(&st.color)),escape(line)));
                    x += line
                        .chars()
                        .map(|c| if c.is_ascii() { size * 0.56 } else { size })
                        .sum::<f32>();
                }
            }
        }
        if let Some(img) = &n.image {
            let uri = image_data(&storage::resolve(base, &img.uri))?;
            s.push_str(&format!(
                "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" xlink:href=\"{uri}\"/>",
                r.min.x + pad.x,
                r.min.y + pad.y + crate::layout::core_height(n, p.text_height) + 6.,
                img.width,
                img.height
            ));
        }
        if !n.details.text().is_empty() {
            s.push_str(&format!("<text x=\"{}\" y=\"{}\" font-family=\"{family}\" font-size=\"11\" fill=\"#526580\">{}</text>",r.min.x+pad.x,r.max.y-18.,escape(n.details.text().lines().next().unwrap_or_default())));
        }
    }
    s.push_str("</svg>");
    Ok(s)
}
pub fn raster(svg: &str, format: &str) -> Result<Vec<u8>> {
    let mut opts = resvg::usvg::Options::default();
    *opts.fontdb_mut() = crate::fonts::system_database();
    let tree = resvg::usvg::Tree::from_str(svg, &opts)?;
    let size = tree.size().to_int_size();
    ensure!(
        (size.width() as u64) * (size.height() as u64) <= 80_000_000,
        "{}",
        msg("export.217", &[])
    );
    let mut pix = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or_else(|| anyhow::anyhow!("{}", msg("export.216", &[])))?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pix.as_mut(),
    );
    if format == "png" {
        Ok(pix.encode_png()?)
    } else {
        let rgba =
            image::RgbaImage::from_raw(size.width(), size.height(), pix.data().to_vec()).unwrap();
        let rgb = image::DynamicImage::ImageRgba8(rgba).to_rgb8();
        let mut out = vec![];
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 92).encode_image(&rgb)?;
        Ok(out)
    }
}
pub fn page_views(scene: &Scene, setup: &PageSetup) -> Result<Vec<(Rect, Vec2)>> {
    let w = setup.width_mm * 72. / 25.4;
    let h = setup.height_mm * 72. / 25.4;
    let m = setup.margin_mm * 72. / 25.4;
    ensure!(
        w > 2. * m && h > 2. * m && m >= 0.,
        "{}",
        msg("export.215", &[])
    );
    let usable = Vec2::new(w - 2. * m, h - 2. * m);
    if !setup.tile {
        let scale = (usable.x / scene.bounds.width()).min(usable.y / scene.bounds.height());
        let size = Vec2::new(w, h) / scale;
        return Ok(vec![(
            Rect::from_center_size(scene.bounds.center(), size),
            Vec2::new(w, h),
        )]);
    }
    let cols = (scene.bounds.width() / usable.x).ceil() as usize;
    let rows = (scene.bounds.height() / usable.y).ceil() as usize;
    ensure!(cols * rows <= 200, "{}", msg("export.214", &[]));
    let mut pages = vec![];
    for row in 0..rows {
        for col in 0..cols {
            let min =
                scene.bounds.min + Vec2::new(col as f32 * usable.x - m, row as f32 * usable.y - m);
            pages.push((Rect::from_min_size(min, Vec2::new(w, h)), Vec2::new(w, h)));
        }
    }
    Ok(pages)
}
pub fn pdf(
    doc: &Document,
    scene: &Scene,
    base: Option<&Path>,
    setup: &PageSetup,
) -> Result<Vec<u8>> {
    use pdf_writer::{Content, Finish, Name, Pdf, Rect as PdfRect, Ref};
    let pages = page_views(scene, setup)?;
    let mut pdf = Pdf::new();
    let catalog = Ref::new(1);
    let pages_id = Ref::new(2);
    let mut next = 3;
    let mut page_ids = vec![];
    let mut options = svg2pdf::usvg::Options::default();
    *options.fontdb_mut() = crate::fonts::system_database();
    for (view, size) in pages {
        let text = page_svg(doc, scene, base, view, size, setup.margin_mm)?;
        let tree = svg2pdf::usvg::Tree::from_str(&text, &options)?;
        let (chunk, root) = svg2pdf::to_chunk(&tree, svg2pdf::ConversionOptions::default())
            .map_err(|e| anyhow::anyhow!("{}", msg("export.213", &[("e", format!("{e}"))])))?;
        let mut refs = std::collections::HashMap::new();
        let chunk = chunk.renumber(|old| {
            *refs.entry(old).or_insert_with(|| {
                let id = Ref::new(next);
                next += 1;
                id
            })
        });
        let form = refs[&root];
        pdf.extend(&chunk);
        let page = Ref::new(next);
        let stream = Ref::new(next + 1);
        next += 2;
        page_ids.push(page);
        let mut p = pdf.page(page);
        p.parent(pages_id)
            .media_box(PdfRect::new(0., 0., size.x, size.y))
            .contents(stream);
        p.resources().x_objects().pair(Name(b"Map"), form);
        p.finish();
        let mut content = Content::new();
        content
            .save_state()
            .transform([size.x, 0., 0., size.y, 0., 0.])
            .x_object(Name(b"Map"))
            .restore_state();
        pdf.stream(stream, &content.finish());
    }
    pdf.catalog(catalog).pages(pages_id);
    pdf.pages(pages_id)
        .kids(page_ids.iter().copied())
        .count(page_ids.len() as i32);
    Ok(pdf.finish())
}
pub fn page_svg(
    doc: &Document,
    scene: &Scene,
    base: Option<&Path>,
    view: Rect,
    size: Vec2,
    margin_mm: f32,
) -> Result<String> {
    let margin = margin_mm * 72. / 25.4;
    let world_margin = margin * view.width() / size.x;
    let inner = svg_view(
        doc,
        scene,
        base,
        view.shrink(world_margin),
        size - Vec2::splat(2. * margin),
    )?;
    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\"><rect width=\"100%\" height=\"100%\" fill=\"white\"/><g transform=\"translate({margin},{margin})\">{inner}</g></svg>",
        size.x, size.y
    ))
}
