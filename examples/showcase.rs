//! Generate a reproducible map and every export format without calling an AI service.
use mindglider::{
    conditions::{Condition, Field, Operator, Rule},
    export,
    layout::LayoutCache,
    model::*,
    richtext::RichText,
    storage,
};
fn main() -> anyhow::Result<()> {
    let folder = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or("artifacts/showcase".into()),
    );
    std::fs::create_dir_all(&folder)?;
    let folder = folder.canonicalize()?;
    let mut doc = Document::default();
    let root = doc.root.clone();
    doc.nodes.get_mut(&root).unwrap().text.edit("MindGlider");
    let a = doc.add(&root, "日本語で考えを整理")?;
    doc.nodes.get_mut(&a).unwrap().cloud = Some("#dcecf8".into());
    doc.nodes.get_mut(&a).unwrap().style.background = "#e8f3ff".into();
    let b = doc.add(&a, "装飾と長い説明")?;
    let n = doc.nodes.get_mut(&b).unwrap();
    n.text.format(0, 2, |m| {
        m.bold = true;
        m.color = Some("#1976c9".into());
        m.size = Some(20.);
    });
    n.note = RichText::plain("ノートは本文と分けて管理します。\n改行した日本語も出力できます。");
    n.details = RichText::plain("詳細はここに");
    n.icons = vec!["idea".into()];
    n.progress = Some(50);
    let c = doc.add(&root, "CLIでツリーを編集")?;
    doc.nodes.get_mut(&c).unwrap().left = true;
    doc.nodes.get_mut(&c).unwrap().link = "https://example.org/".into();
    doc.add(&c, "検証に成功したら自動反映")?;
    doc.add(&c, "1回のUndoで復元")?;
    doc.connectors.push(Connector {
        from: b,
        to: c,
        color: "#668bb5".into(),
        label: "連携".into(),
    });
    doc.styles.insert(
        "CLI accent".into(),
        Style {
            background: "#e5f5eb".into(),
            ..Default::default()
        },
    );
    doc.rules.push(Rule {
        enabled: true,
        stop: true,
        style: "CLI accent".into(),
        condition: Condition::Match {
            field: Field::Text,
            op: Operator::Contains,
            value: "CLI".into(),
            case_sensitive: false,
        },
    });
    let map = folder.join("showcase.mm");
    storage::atomic_write(&map, &mindglider::mm::write(&doc)?)?;
    let scene = LayoutCache::default().build(&doc, None, |s, size| {
        eframe::egui::vec2(
            s.chars()
                .map(|c| if c.is_ascii() { size * 0.56 } else { size })
                .sum(),
            size * 1.4,
        )
    });
    for f in ["txt", "md", "opml", "html"] {
        storage::atomic_write(
            &folder.join(format!("showcase.{f}")),
            export::outline(&doc, &root, f, Some(&map))?.as_bytes(),
        )?;
    }
    let svg = export::svg(&doc, &scene, Some(&map))?;
    storage::atomic_write(&folder.join("showcase.svg"), svg.as_bytes())?;
    for f in ["png", "jpg"] {
        storage::atomic_write(
            &folder.join(format!("showcase.{f}")),
            &export::raster(&svg, f)?,
        )?;
    }
    storage::atomic_write(
        &folder.join("showcase.pdf"),
        &export::pdf(&doc, &scene, Some(&map), &Default::default())?,
    )?;
    storage::atomic_write(
        &folder.join("showcase-tiled.pdf"),
        &export::pdf(
            &doc,
            &scene,
            Some(&map),
            &export::PageSetup {
                tile: true,
                ..Default::default()
            },
        )?,
    )?;
    Ok(())
}
