#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use mindglider::i18n::msg;
mod app;
mod theme;
mod ui_helpers;
use mindglider::{
    layout::LayoutCache,
    model::{Document, History},
};

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let _ = mindglider::storage::atomic_write(
            &mindglider::storage::data_dir().join("error.log"),
            info.to_string().as_bytes(),
        );
    }));
    if let Err(e) = run() {
        let _ = mindglider::storage::atomic_write(
            &mindglider::storage::data_dir().join("error.log"),
            format!("{e:#}").as_bytes(),
        );
        eprintln!("{e:#}");
        std::process::exit(1);
    }
}
fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "--roundtrip") {
        let doc = mindglider::mm::read(&std::fs::read(&args[2])?)?;
        mindglider::storage::atomic_write(
            std::path::Path::new(&args[3]),
            &mindglider::mm::write(&doc)?,
        )?;
        return Ok(());
    }
    if args.get(1).is_some_and(|a| a == "--benchmark") {
        let mut doc = Document::default();
        for i in 0..20 {
            let p = doc
                .add(&doc.root.clone(), msg("main.222", &[("i", format!("{i}"))]))
                .unwrap();
            for j in 0..49 {
                doc.add(
                    &p,
                    msg("main.221", &[("i", format!("{i}")), ("j", format!("{j}"))]),
                )
                .unwrap();
            }
        }
        let mut history = History::new(doc);
        let id = history
            .doc
            .nodes
            .keys()
            .find(|id| **id != history.doc.root)
            .unwrap()
            .clone();
        let mut times = vec![];
        let mut layout = LayoutCache::default();
        for i in 0..200 {
            let start = std::time::Instant::now();
            history.edit("bench", |d| {
                d.nodes
                    .get_mut(&id)
                    .unwrap()
                    .text
                    .edit(&msg("main.220", &[("i", format!("{i}"))]));
                Ok(())
            })?;
            layout.build(&history.doc, None, |s, size| {
                eframe::egui::vec2(s.chars().count() as f32 * size, size * 1.3)
            });
            times.push(start.elapsed().as_secs_f64() * 1000.);
        }
        times.sort_by(f64::total_cmp);
        let report = serde_json::json!({"nodes":history.doc.nodes.len(),"samples":times.len(),"edit_layout_p95_ms":times[189],"max_ms":times[199]});
        mindglider::storage::atomic_write(
            std::path::Path::new(args.get(2).map(String::as_str).unwrap_or("benchmark.json")),
            &serde_json::to_vec_pretty(&report)?,
        )?;
        return Ok(());
    }
    let start = std::time::Instant::now();
    let icon =
        image::load_from_memory(include_bytes!("../assets/icons/mindglider.png"))?.into_rgba8();
    let icon = eframe::egui::IconData {
        width: icon.width(),
        height: icon.height(),
        rgba: icon.into_raw(),
    };
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        persistence_path: Some(mindglider::storage::data_dir().join("ui")),
        viewport: eframe::egui::ViewportBuilder::default()
            .with_icon(icon)
            .with_inner_size([1320., 860.])
            .with_min_inner_size([900., 600.])
            .with_maximized(args.iter().any(|a| a == "--smoke")),
        ..Default::default()
    };
    eframe::run_native(
        "MindGlider",
        options,
        Box::new(move |cc| Ok(Box::new(app::MindGlider::new(cc, args, start)))),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}
