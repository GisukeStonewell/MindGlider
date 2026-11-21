#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use mindglider::i18n::{msg, tr};
mod app;
mod theme;
mod ui_helpers;
mod update;
#[cfg(windows)]
mod windows_frame;
use mindglider::{
    layout::LayoutCache,
    model::{Document, History},
};

fn main() {
    let watch = std::env::current_exe()
        .map_err(anyhow::Error::from)
        .and_then(update::Watch::new);
    let data = mindglider::storage::data_dir();
    let logger = match mindglider::logging::init(&data) {
        Ok(handle) => Some(handle),
        Err(error) => {
            let detail = format!("LOG_INIT_FAILED: {error:#}");
            let _ = mindglider::storage::atomic_write(&data.join("error.log"), detail.as_bytes());
            eprintln!("{detail}");
            None
        }
    };
    log::info!(
        "{}",
        serde_json::json!({
            "event": "app_started", "version": env!("CARGO_PKG_VERSION"), "pid": std::process::id()
        })
    );
    std::panic::set_hook(Box::new(|info| {
        log::error!(target: "mindglider::fatal", "APP_PANIC: {info}");
        let _ = mindglider::storage::atomic_write(
            &mindglider::storage::data_dir().join("error.log"),
            info.to_string().as_bytes(),
        );
    }));
    let watch = watch
        .map_err(|error| log::warn!("UPDATE_WATCH_UNAVAILABLE {error:#}"))
        .ok();
    if let Err(e) = run(watch) {
        log::error!(target: "mindglider::fatal", "APP_START_FAILED: {e:#}");
        let _ = mindglider::storage::atomic_write(
            &mindglider::storage::data_dir().join("error.log"),
            format!("{e:#}").as_bytes(),
        );
        eprintln!("{e:#}");
        if let Some(logger) = logger {
            logger.shutdown();
        }
        std::process::exit(1);
    }
    if let Some(logger) = logger {
        logger.shutdown();
    }
}
fn run(watch: Option<update::Watch>) -> anyhow::Result<()> {
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
            .with_decorations(false)
            .with_inner_size([1320., 860.])
            .with_min_inner_size([900., 600.])
            .with_maximized(args.iter().any(|a| a == "--smoke")),
        ..Default::default()
    };
    let (restart_sender, restart_receiver) = std::sync::mpsc::channel::<update::Restart>();
    eframe::run_native(
        "MindGlider",
        options,
        Box::new(move |cc| {
            #[cfg(windows)]
            windows_frame::install_resize_border(cc)?;
            let mut app = app::MindGlider::new(cc, args, start);
            if let Some(watch) = watch {
                app.watch_updates(watch, restart_sender);
            }
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    if let Ok(restart) = restart_receiver.try_recv()
        && let Err(error) = restart.launch()
    {
        log::error!("UPDATE_RESTART_FAILED {error:#}");
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("MindGlider")
            .set_description(format!("{}\n\n{error}", tr("update.restart_failed")))
            .show();
    }
    Ok(())
}
