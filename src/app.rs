use crate::theme::{self, Theme};
use crate::ui_helpers::*;
use anyhow::{Context, Result, ensure};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use mindglider::i18n::{msg, tr};
use mindglider::{
    ai::{self, CliSettings, Provider},
    conditions::{Condition, Rule},
    export::{self, PageSetup},
    layout::{LayoutCache, Scene, color},
    mm,
    model::*,
    storage,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::atomic::Ordering,
    time::{Duration, Instant, SystemTime},
};

#[derive(Clone)]
struct DragState {
    id: String,
    grab: Vec2,
}
struct Tab {
    id: String,
    history: History,
    path: Option<PathBuf>,
    disk: Option<(SystemTime, u64)>,
    selection: BTreeSet<String>,
    primary: String,
    layout: LayoutCache,
    layout_revision: u64,
    zoom: f32,
    pan: Vec2,
    drag: Option<DragState>,
    canvas_rect: Rect,
    fit_pending: bool,
    ensure_visible: bool,
}
impl Tab {
    fn new(doc: Document, path: Option<PathBuf>) -> Self {
        let root = doc.root.clone();
        let disk = path.as_deref().and_then(stamp);
        Self {
            id: uuid::Uuid::new_v4().simple().to_string(),
            history: History::new(doc),
            path,
            disk,
            selection: BTreeSet::from([root.clone()]),
            primary: root,
            layout: LayoutCache::default(),
            layout_revision: u64::MAX,
            zoom: 1.,
            pan: Vec2::ZERO,
            drag: None,
            canvas_rect: Rect::NOTHING,
            fit_pending: true,
            ensure_visible: false,
        }
    }
    fn name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.history.doc.nodes[&self.history.doc.root].text.text())
    }
    fn select(&mut self, id: String) {
        self.selection = BTreeSet::from([id.clone()]);
        self.primary = id;
    }
    fn normalize_selection(&mut self) {
        self.selection
            .retain(|id| self.history.doc.nodes.contains_key(id));
        if !self.history.doc.nodes.contains_key(&self.primary) {
            self.primary = self.history.doc.root.clone();
        }
        if self.selection.is_empty() {
            self.selection.insert(self.primary.clone());
        }
    }
}
fn stamp(path: &Path) -> Option<(SystemTime, u64)> {
    let m = std::fs::metadata(path).ok()?;
    Some((m.modified().ok()?, m.len()))
}
#[derive(Clone, Serialize, Deserialize)]
struct SavedTab {
    id: String,
    path: Option<PathBuf>,
    recovery: bool,
    zoom: f32,
    pan: [f32; 2],
}
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Settings {
    #[serde(default)]
    version: u32,
    language: String,
    cli: CliSettings,
    shortcuts: BTreeMap<String, String>,
    tabs: Vec<SavedTab>,
    active: usize,
    page: PageSetup,
    theme: Option<Theme>,
    #[serde(default, skip_serializing)]
    dark: bool,
    icon_panel_open: bool,
    #[serde(default = "default_inspector_open")]
    inspector_open: bool,
}
fn default_inspector_open() -> bool {
    true
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 2,
            language: "ja".into(),
            cli: CliSettings::default(),
            shortcuts: commands()
                .iter()
                .filter(|c| !c.2.is_empty())
                .map(|c| (c.0.into(), c.2.into()))
                .collect(),
            tabs: vec![],
            active: 0,
            page: PageSetup::default(),
            theme: None,
            dark: false,
            icon_panel_open: false,
            inspector_open: true,
        }
    }
}
fn commands() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("new", tr("app.169"), "Ctrl+N"),
        ("open", tr("app.212"), "Ctrl+O"),
        ("save", tr("app.211"), "Ctrl+S"),
        ("save_as", tr("app.210"), "Ctrl+Shift+S"),
        ("close", tr("app.170"), "Ctrl+W"),
        ("undo", tr("app.166"), "Ctrl+Z"),
        ("redo", tr("app.165"), "Ctrl+Y"),
        ("child", tr("app.209"), "Tab"),
        ("sibling", tr("app.208"), "Enter"),
        ("parent", tr("app.207"), "Shift+Tab"),
        ("delete", tr("app.206"), "Delete"),
        ("copy", tr("app.205"), "Ctrl+C"),
        ("cut", tr("app.178"), "Ctrl+X"),
        ("paste", tr("app.176"), "Ctrl+V"),
        ("fold", tr("app.132"), "Space"),
        ("side", tr("app.204"), "Ctrl+L"),
        ("move_up", tr("app.203"), "Ctrl+ArrowUp"),
        ("move_down", tr("app.202"), "Ctrl+ArrowDown"),
        ("outdent", tr("app.201"), "Ctrl+ArrowLeft"),
        ("indent", tr("app.200"), "Ctrl+ArrowRight"),
        ("find", tr("app.128"), "Ctrl+F"),
        ("palette", tr("app.107"), "Ctrl+P"),
        ("filter", tr("app.121"), "Ctrl+Shift+F"),
        ("styles", tr("app.117"), ""),
        ("settings", tr("app.105"), "Ctrl+Comma"),
        ("ai", tr("app.199"), "F12"),
        ("export", tr("app.068"), "Ctrl+E"),
        ("fit", tr("app.198"), "Home"),
        ("edit", tr("app.197"), "F2"),
        ("icons", tr("icons.title"), "Ctrl+F2"),
        ("icon_panel", tr("panels.icons"), ""),
        ("inspector", tr("panels.inspector"), ""),
    ]
}
#[derive(Clone, Copy)]
enum Closing {
    Tab(usize),
    Quit,
}
struct StyleDraft {
    tab_id: String,
    styles: BTreeMap<String, Style>,
    rules: Vec<Rule>,
}
struct InlineEdit {
    tab_id: String,
    id: String,
    text: mindglider::richtext::RichText,
    original: mindglider::richtext::RichText,
    focus: bool,
}
#[derive(Default)]
struct MenuState {
    active: bool,
    open: bool,
    selected: usize,
    item: usize,
    theme_item: Option<usize>,
    theme_rows: Vec<(Theme, Rect)>,
    alt_was_down: bool,
    alt_used: bool,
}
#[derive(Default)]
struct IconPicker {
    tab_id: String,
    targets: BTreeSet<String>,
    query: String,
    index: usize,
    focus: bool,
}
pub struct MindGlider {
    icon_picker: Option<IconPicker>,
    menu: MenuState,
    menu_rects: Vec<Rect>,
    about_open: bool,
    icon_filter: String,
    inline: Option<InlineEdit>,
    ime_guard: mindglider::input::ImeGuard,
    tabs: Vec<Tab>,
    active: usize,
    settings: Settings,
    data: PathBuf,
    status: String,
    settings_open: bool,
    settings_page: usize,
    find_open: bool,
    filter_open: bool,
    styles_open: bool,
    style_draft: Option<StyleDraft>,
    export_open: bool,
    ai_open: bool,
    palette_open: bool,
    query: String,
    replacement: String,
    search_case: bool,
    filter: Condition,
    filter_active: bool,
    palette: String,
    ai_prompt: String,
    ai_task: Option<ai::Task>,
    ai_history: Vec<String>,
    ai_was_open: bool,
    ai_focus: bool,
    catalog: mindglider::cli_catalog::Catalog,
    catalog_key: String,
    catalog_task: Option<(
        String,
        std::sync::mpsc::Receiver<mindglider::cli_catalog::Catalog>,
    )>,
    closing: Option<Closing>,
    last_autosave: Instant,
    export_format: String,
    export_branch: bool,
    preview: Option<egui::TextureHandle>,
    preview_page: usize,
    preview_count: usize,
    link_source: Option<String>,
    connector_label: String,
    started: Instant,
    first_frame: bool,
    allow_exit: bool,
    smoke: Option<PathBuf>,
    smoke_drag_mode: Option<String>,
    smoke_drag_source: Option<String>,
    smoke_requested: bool,
    smoke_started: Instant,
    requested_scale: Option<f32>,
    frame_count: u32,
}
impl MindGlider {
    pub fn new(cc: &eframe::CreationContext<'_>, args: Vec<String>, started: Instant) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        crate::ui_helpers::install_fonts(&cc.egui_ctx);
        if let Some(scale) = args
            .iter()
            .position(|s| s == "--ui-scale")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<f32>().ok())
            .filter(|s| s.is_finite() && *s >= 0.75 && *s <= 3.)
        {
            cc.egui_ctx.set_pixels_per_point(scale);
        }
        let data = storage::data_dir();
        let mut status = String::new();
        let mut settings = match std::fs::read(data.join("settings.json")) {
            Ok(b) => match serde_json::from_slice::<Settings>(&b) {
                Ok(s) => s,
                Err(e) => {
                    status = msg("app.051", &[("e", format!("{e}"))]);
                    Settings::default()
                }
            },
            Err(_) => Settings::default(),
        };
        settings.migrate();
        settings.cli.migrate_model();
        if let Some(language) = args
            .iter()
            .position(|s| s == "--language")
            .and_then(|i| args.get(i + 1))
            .filter(|s| matches!(s.as_str(), "ja" | "en"))
        {
            settings.language = language.clone();
        }
        settings.cli.set_language(&settings.language);
        if let Err(e) = mindglider::i18n::initialize(&settings.language) {
            status = msg("app.037", &[("e", format!("{e}"))]);
        }
        theme::apply(&cc.egui_ctx, settings.theme.unwrap_or_default());
        let mut tabs = vec![];
        let smoke = args
            .iter()
            .position(|a| a == "--smoke")
            .and_then(|i| args.get(i + 1))
            .map(PathBuf::from);
        if smoke.is_none() {
            for saved in &settings.tabs {
                if !saved.id.chars().all(|c| c.is_ascii_alphanumeric()) {
                    continue;
                }
                let path = if saved.recovery {
                    Some(data.join(format!("recovery-{}.mm", saved.id)))
                } else {
                    saved.path.clone()
                };
                if let Some(source) = path {
                    match std::fs::read(&source)
                        .map_err(anyhow::Error::from)
                        .and_then(|b| mm::read_at(&b, saved.path.as_deref().unwrap_or(&source)))
                    {
                        Ok(doc) => {
                            let mut tab = Tab::new(doc, saved.path.clone());
                            tab.id = saved.id.clone();
                            if saved.recovery {
                                let on_disk = saved.path.as_ref().and_then(|p| {
                                    std::fs::read(p).ok().and_then(|b| mm::read_at(&b, p).ok())
                                });
                                tab.history = History::restored(tab.history.doc.clone(), on_disk);
                            }
                            tab.zoom = saved.zoom.clamp(0.15, 3.);
                            tab.pan = egui::vec2(saved.pan[0], saved.pan[1]);
                            tab.fit_pending = false;
                            tabs.push(tab);
                            if saved.recovery {
                                status = tr("app.195").into();
                            }
                        }
                        Err(e) => {
                            status = msg("app.050", &[("e", format!("{e}"))]);
                        }
                    }
                }
            }
        }
        let file = args
            .iter()
            .skip(1)
            .find(|a| !a.starts_with('-') && a.to_lowercase().ends_with(".mm"));
        if let Some(file) = file {
            let p = PathBuf::from(file);
            match std::fs::read(&p)
                .map_err(anyhow::Error::from)
                .and_then(|b| mm::read_at(&b, &p))
            {
                Ok(doc) => tabs.push(Tab::new(doc, Some(p))),
                Err(e) => status = msg("app.049", &[("e", format!("{e:#}"))]),
            }
        }
        if tabs.is_empty() {
            let mut doc = Document::default();
            if smoke.is_some() {
                let root = doc.root.clone();
                doc.nodes.get_mut(&root).unwrap().text.edit("MindGlider");
                let a = doc.add(&root, tr("app.194")).unwrap();
                doc.add(&a, tr("app.193")).unwrap();
                doc.add(&a, tr("app.192")).unwrap();
                let b = doc.add(&root, tr("app.191")).unwrap();
                doc.nodes.get_mut(&b).unwrap().left = true;
                doc.nodes.get_mut(&b).unwrap().cloud = Some("#dceef6".into());
                doc.add(&b, tr("app.190")).unwrap();
                doc.add(&b, tr("app.189")).unwrap();
            }
            tabs.push(Tab::new(doc, None));
        }
        let active = settings.active.min(tabs.len() - 1);
        if status.is_empty() {
            status = tr("app.196").into();
        }
        let mut app = Self {
            icon_picker: None,
            menu: Default::default(),
            menu_rects: vec![],
            about_open: false,
            icon_filter: String::new(),
            inline: None,
            ime_guard: Default::default(),
            tabs,
            active,
            settings,
            data,
            status,
            settings_open: false,
            settings_page: 0,
            find_open: false,
            filter_open: false,
            styles_open: false,
            style_draft: None,
            export_open: false,
            ai_open: false,
            palette_open: false,
            query: String::new(),
            replacement: String::new(),
            search_case: false,
            filter: Condition::default(),
            filter_active: false,
            palette: String::new(),
            ai_prompt: String::new(),
            ai_task: None,
            ai_history: Vec::new(),
            ai_was_open: false,
            ai_focus: false,
            catalog: Default::default(),
            catalog_key: String::new(),
            catalog_task: None,
            closing: None,
            last_autosave: Instant::now(),
            export_format: "png".into(),
            export_branch: false,
            preview: None,
            preview_page: 0,
            preview_count: 0,
            link_source: None,
            connector_label: String::new(),
            started,
            first_frame: true,
            allow_exit: false,
            smoke,
            smoke_drag_mode: args
                .iter()
                .position(|a| a == "--smoke-view")
                .and_then(|i| args.get(i + 1))
                .filter(|s| s.starts_with("drag-"))
                .cloned(),
            smoke_drag_source: None,
            smoke_requested: false,
            smoke_started: Instant::now(),
            requested_scale: args
                .iter()
                .position(|s| s == "--ui-scale")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse::<f32>().ok())
                .filter(|s| s.is_finite() && (0.75..=3.).contains(s)),
            frame_count: 0,
        };
        if let Some(view) = args
            .iter()
            .position(|s| s == "--smoke-view")
            .and_then(|i| args.get(i + 1))
        {
            match view.as_str() {
                "settings" => app.settings_open = true,
                "settings-ai" => {
                    app.settings_open = true;
                    app.settings_page = 1;
                }
                "settings-print" => {
                    app.settings_open = true;
                    app.settings_page = 2;
                }
                "settings-shortcuts" => {
                    app.settings_open = true;
                    app.settings_page = 3;
                }
                "ai" => app.ai_open = true,
                "icons" => app.open_icon_picker(),
                "panels" => {
                    app.settings.icon_panel_open = true;
                    app.settings.inspector_open = true;
                }
                "collapsed" => {
                    app.settings.icon_panel_open = false;
                    app.settings.inspector_open = false;
                }
                "inline" => {
                    let _ = app.action("edit", &cc.egui_ctx);
                }
                "menu" => {
                    app.menu.active = true;
                    app.menu.open = true;
                }
                _ => {}
            }
        }
        app
    }
    fn result(&mut self, r: Result<()>) {
        if let Err(e) = r {
            self.status = format!("{e:#}");
        }
    }
    fn open(&mut self, path: PathBuf) -> Result<()> {
        if let Some(i) = self
            .tabs
            .iter()
            .position(|t| t.path.as_ref() == Some(&path))
        {
            self.active = i;
            return Ok(());
        }
        let doc = mm::read_at(&std::fs::read(&path)?, &path)?;
        self.tabs.push(Tab::new(doc, Some(path)));
        self.active = self.tabs.len() - 1;
        self.filter_active = false;
        self.status = tr("app.188").into();
        Ok(())
    }
    fn save_tab(&mut self, index: usize, save_as: bool) -> Result<bool> {
        let old_path = self.tabs[index].path.clone();
        let path = if save_as || old_path.is_none() {
            rfd::FileDialog::new()
                .add_filter(tr("file.mm_filter"), &["mm"])
                .set_file_name(
                    old_path
                        .as_ref()
                        .and_then(|p| p.file_name())
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Mindmap.mm".into()),
                )
                .save_file()
        } else {
            old_path.clone()
        };
        let Some(path) = path else { return Ok(false) };
        if old_path.as_ref() == Some(&path) {
            ensure!(
                self.tabs[index].disk == stamp(&path),
                "{}",
                msg("app.048", &[])
            );
        }
        let tab = &mut self.tabs[index];
        // Relative image paths refer to the map location, including after Save As.
        let mut doc = tab.history.doc.clone();
        if old_path.as_ref().and_then(|p| p.parent()) != path.parent() {
            for n in doc.nodes.values_mut() {
                if let Some(p) = &mut n.image {
                    let resolved = storage::resolve(old_path.as_deref(), &p.uri);
                    if resolved.is_file() {
                        p.uri = url::Url::from_file_path(&resolved)
                            .map(|u| u.to_string())
                            .unwrap_or_else(|_| resolved.to_string_lossy().into_owned());
                    }
                }
                if !n.link.is_empty() && !n.link.starts_with('#') && !n.link.contains(':') {
                    let resolved = storage::resolve(old_path.as_deref(), &n.link);
                    if resolved.exists() {
                        n.link = resolved.to_string_lossy().into_owned();
                    }
                }
            }
        }
        storage::save(&path, &doc)?;
        tab.history.doc = doc;
        tab.path = Some(path.clone());
        tab.disk = stamp(&path);
        tab.history.saved();
        let _ = std::fs::remove_file(self.data.join(format!("recovery-{}.mm", tab.id)));
        self.status = msg("app.047", &[("0", format!("{}", path.display()))]);
        self.save_session()?;
        Ok(true)
    }
    fn save_session(&mut self) -> Result<()> {
        if self.smoke.is_some() {
            return Ok(());
        }
        let mut saved = vec![];
        for t in &self.tabs {
            let recovery = t.history.dirty || t.path.is_none();
            if recovery {
                storage::atomic_write(
                    &self.data.join(format!("recovery-{}.mm", t.id)),
                    &mm::write(&t.history.doc)?,
                )?;
            }
            saved.push(SavedTab {
                id: t.id.clone(),
                path: t.path.clone(),
                recovery,
                zoom: t.zoom,
                pan: [t.pan.x, t.pan.y],
            });
        }
        self.settings.tabs = saved;
        self.settings.active = self.active;
        storage::atomic_write(
            &self.data.join("settings.json"),
            &serde_json::to_vec_pretty(&self.settings)?,
        )?;
        Ok(())
    }
    fn action(&mut self, id: &str, ctx: &egui::Context) -> Result<()> {
        if id != "edit" {
            self.finish_inline(true)?;
        }
        match id {
            "new" => {
                self.tabs.push(Tab::new(Document::default(), None));
                self.active = self.tabs.len() - 1;
                self.filter_active = false;
            }
            "open" => {
                if let Some(p) = rfd::FileDialog::new()
                    .add_filter(tr("file.mm_filter"), &["mm"])
                    .pick_file()
                {
                    self.open(p)?;
                }
            }
            "save" => {
                self.save_tab(self.active, false)?;
            }
            "save_as" => {
                self.save_tab(self.active, true)?;
            }
            "close" => self.closing = Some(Closing::Tab(self.active)),
            "find" => self.find_open = !self.find_open,
            "filter" => self.filter_open = !self.filter_open,
            "styles" => {
                self.styles_open = !self.styles_open;
                self.style_draft = None;
            }
            "about" => self.about_open = true,
            "settings" => self.settings_open = !self.settings_open,
            "ai" => self.ai_open = !self.ai_open,
            "icons" => self.open_icon_picker(),
            "icon_panel" => {
                self.settings.icon_panel_open = !self.settings.icon_panel_open;
                self.save_session()?;
            }
            "inspector" => {
                self.settings.inspector_open = !self.settings.inspector_open;
                self.save_session()?;
            }
            "export" => {
                self.export_open = true;
                self.preview = None;
            }
            "palette" => {
                self.palette_open = true;
                self.palette.clear();
            }
            "copy" => self.copy(false)?,
            "cut" => self.copy(true)?,
            "paste" => self.paste()?,
            "fit" => {
                let t = &mut self.tabs[self.active];
                t.fit_pending = true;
            }
            "edit" => {
                let t = &self.tabs[self.active];
                t.history.doc.ensure_editable(&t.primary)?;
                let n = &t.history.doc.nodes[&t.primary];
                ensure!(!n.text.unsupported, "{}", msg("app.046", &[]));
                self.inline = Some(InlineEdit {
                    tab_id: t.id.clone(),
                    id: t.primary.clone(),
                    text: n.text.clone(),
                    original: n.text.clone(),
                    focus: true,
                });
                ctx.request_repaint();
            }
            _ => {
                let tab = &mut self.tabs[self.active];
                let current = tab.primary.clone();
                match id {
                    "undo" => tab.history.undo(),
                    "redo" => tab.history.redo(),
                    "child" | "sibling" | "parent" => {
                        let mut selected = None;
                        tab.history.edit(tr("app.187"), |d| {
                            let parent = if id == "sibling" {
                                d.nodes[&current]
                                    .parent
                                    .clone()
                                    .unwrap_or_else(|| current.clone())
                            } else {
                                current.clone()
                            };
                            if id == "parent" {
                                let p = d.nodes[&current].parent.clone().context(tr("app.186"))?;
                                let i = d.nodes[&p]
                                    .children
                                    .iter()
                                    .position(|c| c == &current)
                                    .unwrap();
                                let new = d.add(&p, tr("app.185"))?;
                                d.move_node(&new, &p, i)?;
                                d.move_node(&current, &new, 0)?;
                                selected = Some(new);
                            } else {
                                let new = d.add(&parent, tr("app.184"))?;
                                d.nodes.get_mut(&parent).unwrap().folded = false;
                                if id == "sibling" {
                                    let i = d.nodes[&parent]
                                        .children
                                        .iter()
                                        .position(|c| c == &current)
                                        .unwrap_or(0)
                                        + 1;
                                    d.move_node(&new, &parent, i)?;
                                }
                                selected = Some(new);
                            }
                            Ok(())
                        })?;
                        if let Some(id) = selected {
                            tab.select(id);
                        }
                    }
                    "delete" => {
                        let ids = tab.history.doc.top_selected(&tab.selection);
                        tab.history.edit(tr("app.183"), |d| {
                            for id in ids {
                                d.delete(&id)?;
                            }
                            Ok(())
                        })?;
                    }
                    "fold" => {
                        let ids = tab.selection.clone();
                        tab.history.edit(tr("app.130"), |d| {
                            for id in ids {
                                let n = d.nodes.get_mut(&id).unwrap();
                                n.folded = !n.folded;
                            }
                            Ok(())
                        })?;
                    }
                    "side" => {
                        let ids = tab.selection.clone();
                        tab.history.edit(tr("app.182"), |d| {
                            for id in ids {
                                d.ensure_branch_editable(&id)?;
                                let left = !d.nodes[&id].left;
                                for c in d.subtree(&id) {
                                    d.nodes.get_mut(&c).unwrap().left = left;
                                }
                            }
                            Ok(())
                        })?;
                    }
                    "move_up" | "move_down" | "indent" | "outdent" => {
                        tab.history.edit(tr("app.181"), |d| {
                            let parent = d.nodes[&current].parent.clone().context(tr("app.180"))?;
                            let i = d.nodes[&parent]
                                .children
                                .iter()
                                .position(|c| c == &current)
                                .unwrap();
                            match id {
                                "move_up" => d.move_node(&current, &parent, i.saturating_sub(1))?,
                                "move_down" => d.move_node(&current, &parent, i + 1)?,
                                "indent" => {
                                    ensure!(i > 0, "{}", msg("app.045", &[]));
                                    let dest = d.nodes[&parent].children[i - 1].clone();
                                    let end = d.nodes[&dest].children.len();
                                    d.move_node(&current, &dest, end)?;
                                }
                                _ => {
                                    let dest =
                                        d.nodes[&parent].parent.clone().context(tr("app.179"))?;
                                    let pos = d.nodes[&dest]
                                        .children
                                        .iter()
                                        .position(|c| c == &parent)
                                        .unwrap()
                                        + 1;
                                    d.move_node(&current, &dest, pos)?;
                                }
                            }
                            Ok(())
                        })?;
                    }
                    _ => {}
                }
                tab.normalize_selection();
            }
        }
        Ok(())
    }
    fn copy(&mut self, cut: bool) -> Result<()> {
        let tab = &mut self.tabs[self.active];
        let ids = tab.history.doc.top_selected(&tab.selection);
        let mut copy = Document::default();
        let root = copy.root.clone();
        for id in &ids {
            tab.history.doc.ensure_branch_editable(id)?;
            for c in tab.history.doc.subtree(id) {
                copy.nodes
                    .insert(c.clone(), tab.history.doc.nodes[&c].clone());
            }
            copy.nodes.get_mut(id).unwrap().parent = Some(root.clone());
            copy.nodes.get_mut(&root).unwrap().children.push(id.clone());
        }
        let scope: BTreeSet<_> = copy.nodes.keys().cloned().collect();
        copy.connectors = tab
            .history
            .doc
            .connectors
            .iter()
            .filter(|c| scope.contains(&c.from) && scope.contains(&c.to))
            .cloned()
            .collect();
        let text = format!(
            "MINDGLIDER_CLIPBOARD\n{}",
            String::from_utf8(mm::write(&copy)?)?
        );
        arboard::Clipboard::new()?.set_text(text)?;
        if cut {
            tab.history.edit(tr("app.178"), |d| {
                for id in ids {
                    d.delete(&id)?;
                }
                Ok(())
            })?;
            tab.normalize_selection();
        }
        self.status = tr("app.177").into();
        Ok(())
    }
    fn paste(&mut self) -> Result<()> {
        let text = arboard::Clipboard::new()?.get_text()?;
        let tab = &mut self.tabs[self.active];
        let parent = tab.primary.clone();
        tab.history.edit(tr("app.176"), |d| {
            if let Some(xml) = text.strip_prefix("MINDGLIDER_CLIPBOARD\n") {
                let source = mm::read(xml.as_bytes())?;
                let mut ids = BTreeMap::new();
                for id in source.nodes.keys().filter(|i| **i != source.root) {
                    ids.insert(id.clone(), new_id());
                }
                for (old, new) in &ids {
                    let mut n = source.nodes[old].clone();
                    n.id = new.clone();
                    n.parent = Some(if n.parent.as_deref() == Some(&source.root) {
                        parent.clone()
                    } else {
                        ids[n.parent.as_ref().unwrap()].clone()
                    });
                    n.children = n.children.iter().map(|i| ids[i].clone()).collect();
                    if let Some(link) = n.link.strip_prefix('#')
                        && let Some(id) = ids.get(link)
                    {
                        n.link = format!("#{id}");
                    }
                    if let Some(raw) = &mut n.raw {
                        raw.attributes.insert("ID".into(), new.clone());
                        raw.children
                            .retain(|c| !c.as_element().is_some_and(|e| e.name == "arrowlink"));
                    }
                    d.nodes.insert(new.clone(), n);
                }
                d.ensure_editable(&parent)?;
                for c in &source.nodes[&source.root].children {
                    d.nodes
                        .get_mut(&parent)
                        .unwrap()
                        .children
                        .push(ids[c].clone());
                }
                for c in source.connectors {
                    d.connectors.push(Connector {
                        from: ids[&c.from].clone(),
                        to: ids[&c.to].clone(),
                        ..c
                    });
                }
            } else {
                for line in text.lines().filter(|s| !s.trim().is_empty()).take(5000) {
                    d.add(&parent, line)?;
                }
            }
            Ok(())
        })?;
        Ok(())
    }
    fn keyboard(&mut self, ctx: &egui::Context) {
        if self.tabs[self.active].drag.is_some() {
            return;
        }
        if self.icon_picker.is_some() {
            return;
        }
        if self.menu_keys(ctx) {
            return;
        }

        if self.settings_open {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
                self.settings_open = false;
            }
            return;
        }
        if self.ai_open {
            return;
        }
        if self.inline.is_some() {
            let cancel = ctx.input(|i| i.key_pressed(egui::Key::Escape));
            let commit = ctx.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);
            if cancel || commit {
                ctx.input_mut(|i| {
                    i.events.retain(|e| {
                        !matches!(
                            e,
                            egui::Event::Key {
                                key: egui::Key::Enter | egui::Key::Escape,
                                ..
                            }
                        )
                    })
                });
                let result = self.finish_inline(!cancel);
                self.result(result);
                return;
            }
        }
        let typing = ctx
            .memory(|m| m.focused())
            .is_some_and(|id| egui::text_edit::TextEditState::load(ctx, id).is_some());
        let mut bindings: Vec<_> = self.settings.shortcuts.clone().into_iter().collect();
        bindings.sort_by_key(|(_, s)| std::cmp::Reverse(s.matches('+').count()));
        let actions: Vec<String> = ctx.input_mut(|i| {
            let mut actions = vec![];
            for (command, text) in bindings {
                if typing
                    && !matches!(
                        command.as_str(),
                        "save" | "save_as" | "open" | "palette" | "ai" | "settings" | "icons"
                    )
                {
                    continue;
                }
                if let Some(shortcut) = shortcut(&text)
                    && i.consume_shortcut(&shortcut)
                {
                    actions.push(command);
                }
            }
            actions
        });
        for action in actions {
            let r = self.action(&action, ctx);
            self.result(r);
        }
        if !typing
            && !self.palette_open
            && self.inline.is_none()
            && ctx.input(|i| i.focused && i.modifiers.is_none())
        {
            let key = ctx.input(|i| {
                [
                    egui::Key::ArrowUp,
                    egui::Key::ArrowDown,
                    egui::Key::ArrowLeft,
                    egui::Key::ArrowRight,
                ]
                .into_iter()
                .find(|k| i.key_down(*k))
            });
            if let Some(key) = key {
                let tab = &mut self.tabs[self.active];
                if let Some(target) = mindglider::navigation::target(
                    &tab.history.doc,
                    &tab.layout.scene,
                    &tab.primary,
                    key,
                ) {
                    if ctx.input(|i| i.key_pressed(key)) {
                        tab.select(target);
                        tab.ensure_visible = true;
                    }
                } else {
                    let dt = ctx.input(|i| {
                        if i.key_pressed(key) {
                            1. / 30.
                        } else {
                            i.stable_dt
                        }
                    });
                    tab.pan += mindglider::navigation::scroll_delta(key, dt);
                    ctx.request_repaint_after(Duration::from_millis(16));
                }
            }
        }
    }

    fn toolbar(&mut self, ctx: &egui::Context) {
        let mut action = None;
        let mut menu_rects = vec![];
        egui::TopBottomPanel::top("top")
            .frame(
                egui::Frame::NONE
                    .fill(theme::current(ctx).background)
                    .inner_margin(egui::Margin {
                        left: 12,
                        right: 12,
                        top: 0,
                        bottom: 0,
                    }),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 0.;
                ui.spacing_mut().button_padding = egui::vec2(8., 4.);
                ui.spacing_mut().slider_width = if ui.available_width() < 1000. {
                    80.
                } else {
                    100.
                };
                let compact = ui.available_width() < 1120.;
                ui.scope(|ui| {
                    ui.spacing_mut().button_padding = egui::vec2(8., 4.);
                    let bar_start = ui.cursor().min;
                    let width = ui.available_width();
                    let menu_end = ui
                        .horizontal(|ui| {
                            ui.set_min_height(32.);
                            ui.spacing_mut().interact_size.y = 28.;
                            ui.add_space(164.);
                            for (index, label) in [
                                tr("app.175"),
                                tr("app.174"),
                                tr("app.173"),
                                tr("app.172"),
                                tr("app.171"),
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                let text = if self.menu.active && self.menu.selected == index {
                                    egui::RichText::new(label).underline()
                                } else {
                                    egui::RichText::new(label)
                                };
                                let response = crate::theme::flat_button(
                                    ui,
                                    text,
                                    self.menu.active && self.menu.selected == index,
                                );
                                menu_rects.push(response.rect);
                                if response.clicked() {
                                    self.menu.theme_item = None;
                                    let same = self.menu.active
                                        && self.menu.selected == index
                                        && self.menu.open;
                                    self.menu.active = !same;
                                    self.menu.selected = index;
                                    self.menu.open = !same;
                                    self.menu.item = 0;
                                }
                                if self.menu.active
                                    && self.menu.open
                                    && self.menu.selected != index
                                    && response.hovered()
                                    && ctx.input(|i| i.pointer.delta() != Vec2::ZERO)
                                {
                                    self.menu.selected = index;
                                    self.menu.item = 0;
                                    self.menu.theme_item = None;
                                }
                            }
                        })
                        .response
                        .rect
                        .right();
                    self.title_row(
                        ui,
                        Rect::from_min_size(bar_start, egui::vec2(width, 32.)),
                        menu_end,
                    );
                });
                let strip = ui.painter().add(egui::Shape::Noop);
                let strip_rect = ui
                    .horizontal_wrapped(|ui| {
                        ui.set_min_height(32.);
                        ui.spacing_mut().interact_size.y = 30.;
                        for (id, label) in [
                            ("child", tr("app.168")),
                            ("sibling", tr("app.167")),
                            ("fold", tr("app.130")),
                            ("undo", tr("app.166")),
                            ("redo", tr("app.165")),
                            ("fit", tr("app.164")),
                        ] {
                            if theme::tool_button(ui, id, label).clicked() {
                                action = Some(id);
                            }
                        }
                        ui.separator();
                        let t = &mut self.tabs[self.active];
                        ui.add(
                            egui::Slider::new(&mut t.zoom, 0.15..=3.)
                                .logarithmic(true)
                                .show_value(false),
                        );
                        ui.label(format!("{:.0}%", t.zoom * 100.));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if crate::theme::accent_button(
                                ui,
                                &if compact {
                                    "\u{e8bd}".into()
                                } else {
                                    format!("✦  {}", command_label("ai"))
                                },
                            )
                            .on_hover_text(format!(
                                "{} ({})",
                                command_label("ai"),
                                self.settings
                                    .shortcuts
                                    .get("ai")
                                    .map(String::as_str)
                                    .unwrap_or("F12")
                            ))
                            .clicked()
                            {
                                action = Some("ai");
                            }
                        });
                        if self.filter_active && ui.button(tr("app.163")).clicked() {
                            self.filter_active = false;
                            t.layout_revision = u64::MAX;
                        }
                    })
                    .response
                    .rect;
                ui.painter().set(
                    strip,
                    egui::Shape::rect_filled(
                        strip_rect.expand2(egui::vec2(12., 0.)),
                        0.,
                        theme::current(ctx).surface,
                    ),
                );
            });
        self.menu_rects = menu_rects;
        if self.menu.active && self.menu.open {
            let anchor = self
                .menu_rects
                .get(self.menu.selected)
                .map(|r| r.left_bottom())
                .unwrap_or(Pos2::ZERO);
            let choices = menu_commands(self.menu.selected);
            let mut popup_rect = Rect::NOTHING;
            let mut theme_row = None;
            egui::Area::new(egui::Id::new("menu-popup"))
                .order(egui::Order::Foreground)
                .fixed_pos(anchor)
                .show(ctx, |ui| {
                    let popup = egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_min_width(260.);
                        for (index, id) in choices.iter().enumerate() {
                            let is_theme = *id == "theme";
                            let label = if is_theme {
                                tr("menu.theme")
                            } else {
                                command_label(id)
                            };
                            let key = self
                                .settings
                                .shortcuts
                                .get(*id)
                                .cloned()
                                .unwrap_or_default();
                            let (row, response) =
                                ui.allocate_exact_size(egui::vec2(300., 36.), Sense::click());
                            let p = theme::current(ctx);
                            if self.menu.item == index || response.hovered() {
                                ui.painter().rect_filled(row, 4., p.hover);
                            }
                            ui.painter().text(
                                row.left_center() + egui::vec2(12., 0.),
                                Align2::LEFT_CENTER,
                                label,
                                FontId::proportional(14.),
                                p.text,
                            );
                            ui.painter().text(
                                row.right_center() - egui::vec2(12., 0.),
                                Align2::RIGHT_CENTER,
                                if is_theme { "›".into() } else { key },
                                FontId::proportional(12.),
                                p.secondary,
                            );
                            if is_theme {
                                theme_row = Some(row);
                            }
                            if response.hovered() && ctx.input(|i| i.pointer.delta() != Vec2::ZERO)
                            {
                                self.menu.item = index;
                                if is_theme {
                                    self.menu.theme_item.get_or_insert(0);
                                } else {
                                    self.menu.theme_item = None;
                                }
                            }
                            if response.clicked() {
                                if is_theme {
                                    self.menu.theme_item.get_or_insert(0);
                                } else {
                                    action = Some(*id);
                                    self.menu = MenuState::default();
                                }
                            }
                        }
                    });
                    popup_rect = popup.response.rect;
                });
            let submenu_rect = if let (Some(row), Some(index)) = (theme_row, self.menu.theme_item) {
                self.theme_submenu(ctx, popup_rect, row, index)
            } else {
                self.menu.theme_rows.clear();
                Rect::NOTHING
            };
            if ctx.input(|i| i.pointer.any_pressed())
                && ctx.input(|i| i.pointer.interact_pos()).is_some_and(|p| {
                    !popup_rect.contains(p)
                        && !submenu_rect.contains(p)
                        && !self.menu_rects.iter().any(|r| r.contains(p))
                })
            {
                self.menu = MenuState::default();
            }
        }
        if self.menu.theme_item.is_none()
            && theme::current(ctx).canvas
                != theme::palette(self.settings.theme.unwrap_or_default()).canvas
        {
            ctx.request_repaint();
        }
        if let Some(id) = action {
            let result = self.action(id, ctx);
            self.result(result);
        }
    }
    fn title_row(&self, ui: &mut egui::Ui, bar: Rect, menu_end: f32) {
        let controls = Rect::from_min_max(
            egui::pos2(bar.right() - 132., bar.top()),
            bar.right_bottom(),
        );
        let draggable = Rect::from_min_max(egui::pos2(menu_end, bar.top()), controls.left_bottom());
        let drag = ui.interact(
            draggable,
            ui.id().with("title-drag"),
            Sense::click_and_drag(),
        );
        let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        if drag.double_clicked() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
        } else if drag.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        let p = theme::current(ui.ctx());
        egui::Image::new(egui::include_image!("../assets/icons/mindglider.png")).paint_at(
            ui,
            Rect::from_center_size(bar.left_center() + egui::vec2(8., 0.), egui::vec2(16., 16.)),
        );
        ui.painter().text(
            bar.left_center() + egui::vec2(22., 0.),
            Align2::LEFT_CENTER,
            "MindGlider",
            FontId::proportional(14.),
            p.text,
        );
        let title = self.tabs[self.active].name();
        let available = Rect::from_min_max(
            draggable.left_top() + egui::vec2(10., 0.),
            draggable.right_bottom() - egui::vec2(10., 0.),
        );
        let title_room = available.width().clamp(0., 240.) / 2.;
        let title_center = egui::pos2(
            bar.center().x.clamp(
                available.left() + title_room,
                (available.right() - title_room).max(available.left() + title_room),
            ),
            bar.center().y,
        );
        ui.painter().with_clip_rect(available).text(
            title_center,
            Align2::CENTER_CENTER,
            title,
            FontId::proportional(13.),
            p.secondary,
        );
        for (i, (glyph, tip, command)) in [
            (
                "\u{e921}",
                tr("window.minimize"),
                egui::ViewportCommand::Minimized(true),
            ),
            (
                if maximized { "\u{e923}" } else { "\u{e922}" },
                if maximized {
                    tr("window.restore")
                } else {
                    tr("window.maximize")
                },
                egui::ViewportCommand::Maximized(!maximized),
            ),
            ("\u{e8bb}", tr("ui.close_tip"), egui::ViewportCommand::Close),
        ]
        .into_iter()
        .enumerate()
        {
            let rect = Rect::from_min_size(
                controls.min + egui::vec2(i as f32 * 44., 0.),
                egui::vec2(44., 28.),
            );
            if ui
                .put(rect, egui::Button::new(glyph).frame(false))
                .on_hover_text(tip)
                .clicked()
            {
                ui.ctx().send_viewport_cmd(command);
            }
        }
    }

    fn theme_submenu(
        &mut self,
        ctx: &egui::Context,
        parent: Rect,
        row: Rect,
        index: usize,
    ) -> Rect {
        const WIDTH: f32 = 190.;
        const HEIGHT: f32 = 340.;
        let screen = ctx.screen_rect();
        let x = if parent.right() + WIDTH <= screen.right() {
            parent.right() - 1.
        } else {
            parent.left() - WIDTH + 1.
        };
        let y = row.top().min(screen.bottom() - HEIGHT).max(screen.top());
        let mut rows = Vec::with_capacity(Theme::ALL.len());
        let mut picked = None;
        let rect = egui::Area::new(egui::Id::new("theme-submenu"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(x, y))
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 0.;
                        for (i, candidate) in Theme::ALL.into_iter().enumerate() {
                            let (rect, response) =
                                ui.allocate_exact_size(egui::vec2(172., 32.), Sense::click());
                            let p = theme::current(ctx);
                            if response.hovered() || i == index {
                                ui.painter().rect_filled(rect, 4., p.hover);
                            }
                            let swatch = theme::palette(candidate);
                            ui.painter().rect_filled(
                                Rect::from_center_size(
                                    rect.left_center() + egui::vec2(15., 0.),
                                    egui::vec2(15., 15.),
                                ),
                                3.,
                                swatch.accent,
                            );
                            ui.painter().text(
                                rect.left_center() + egui::vec2(30., 0.),
                                Align2::LEFT_CENTER,
                                candidate.name(),
                                FontId::proportional(14.),
                                p.text,
                            );
                            if self.settings.theme.unwrap_or_default() == candidate {
                                ui.painter().text(
                                    rect.right_center() - egui::vec2(8., 0.),
                                    Align2::RIGHT_CENTER,
                                    "✓",
                                    FontId::proportional(14.),
                                    p.text,
                                );
                            }
                            if response.hovered() && ctx.input(|i| i.pointer.delta() != Vec2::ZERO)
                            {
                                self.menu.theme_item = Some(i);
                            }
                            if response.clicked() {
                                picked = Some(candidate);
                            }
                            rows.push((candidate, rect));
                        }
                    })
                    .response
                    .rect
            })
            .inner;
        if let Some(theme) = picked {
            self.select_theme(theme, ctx);
        } else {
            if self.menu.theme_rows != rows {
                ctx.request_repaint();
            }
            self.menu.theme_rows = rows;
        }
        rect
    }

    fn select_theme(&mut self, theme: Theme, ctx: &egui::Context) {
        self.settings.theme = Some(theme);
        self.menu = MenuState::default();
        let result = self.save_session();
        self.result(result);
        ctx.request_repaint();
    }

    fn file_tabs(&mut self, ctx: &egui::Context) {
        let mut action = None;
        let mut change = None;
        let mut close = None;
        egui::TopBottomPanel::bottom("file-tabs")
            .frame(
                egui::Frame::NONE
                    .fill(theme::current(ctx).background)
                    .inner_margin(egui::Margin::symmetric(12, 0)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().interact_size.y = 26.;
                egui::ScrollArea::horizontal()
                    .id_salt("tabs")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.set_min_height(28.);
                            for (index, t) in self.tabs.iter().enumerate() {
                                let title = format!(
                                    "{}{}",
                                    if t.history.dirty { "● " } else { "" },
                                    t.name()
                                );
                                let p = theme::current(ctx);
                                let tab = egui::Frame::new()
                                    .fill(if self.active == index {
                                        p.canvas
                                    } else {
                                        p.background
                                    })
                                    .corner_radius(egui::CornerRadius {
                                        nw: 8,
                                        ne: 8,
                                        sw: 0,
                                        se: 0,
                                    })
                                    .inner_margin(egui::Margin::symmetric(8, 0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            if ui
                                                .add_sized(
                                                    [160., 28.],
                                                    egui::Button::new(title)
                                                        .frame(false)
                                                        .truncate(),
                                                )
                                                .clicked()
                                            {
                                                change = Some(index);
                                            }
                                            if ui
                                                .add_sized(
                                                    [28., 28.],
                                                    egui::Button::new("\u{e711}").frame(false),
                                                )
                                                .on_hover_text(tr("app.170"))
                                                .clicked()
                                            {
                                                close = Some(index);
                                            }
                                        });
                                    });
                                if self.active == index {
                                    ui.painter().hline(
                                        tab.response.rect.x_range(),
                                        tab.response.rect.bottom(),
                                        Stroke::new(2_f32, p.accent),
                                    );
                                }
                            }
                            if ui
                                .add_sized([28., 28.], egui::Button::new("\u{e710}").frame(false))
                                .on_hover_text(tr("app.169"))
                                .clicked()
                            {
                                action = Some("new");
                            }
                        });
                    });
            });
        if let Some(index) = change {
            let result = self.finish_inline(true);
            self.result(result);
            self.active = index;
            self.filter_active = false;
        }
        if let Some(index) = close {
            let result = self.finish_inline(true);
            self.result(result);
            self.closing = Some(Closing::Tab(index));
        }
        if let Some(id) = action {
            let result = self.action(id, ctx);
            self.result(result);
        }
    }

    fn menu_keys(&mut self, ctx: &egui::Context) -> bool {
        let alt = ctx.input(|i| i.modifiers.alt);
        let pressed = ctx.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::Key { pressed: true, .. }))
        });
        if alt && !self.menu.alt_was_down {
            self.menu.alt_used = false;
        }
        if alt && pressed {
            self.menu.alt_used = true;
        }
        if !alt && self.menu.alt_was_down && !self.menu.alt_used {
            self.menu.active = !self.menu.active;
            self.menu.open = false;
            self.menu.selected = 0;
            ctx.request_repaint();
        }
        self.menu.alt_was_down = alt;
        let mnemonic = ctx.input(|i| {
            [
                (egui::Key::F, 0),
                (egui::Key::E, 1),
                (egui::Key::V, 2),
                (egui::Key::T, 3),
                (egui::Key::H, 4),
            ]
            .into_iter()
            .find(|(k, _)| alt && i.key_pressed(*k))
            .map(|(_, n)| n)
        });
        if let Some(index) = mnemonic {
            self.menu.active = true;
            self.menu.open = true;
            self.menu.selected = index;
            self.menu.item = 0;
            self.menu.theme_item = None;
        }
        if !self.menu.active {
            return false;
        }
        if self.menu.selected == 2 && self.menu.open {
            let current = menu_commands(2)[self.menu.item];
            let key = ctx.input(|i| {
                [
                    egui::Key::Escape,
                    egui::Key::ArrowLeft,
                    egui::Key::ArrowRight,
                    egui::Key::ArrowUp,
                    egui::Key::ArrowDown,
                    egui::Key::Enter,
                ]
                .into_iter()
                .find(|key| i.key_pressed(*key))
            });
            if let Some(key) = key {
                if let Some(index) = self.menu.theme_item {
                    match key {
                        egui::Key::Escape | egui::Key::ArrowLeft => self.menu.theme_item = None,
                        egui::Key::ArrowUp => {
                            self.menu.theme_item =
                                Some((index + Theme::ALL.len() - 1) % Theme::ALL.len())
                        }
                        egui::Key::ArrowDown => {
                            self.menu.theme_item = Some((index + 1) % Theme::ALL.len())
                        }
                        egui::Key::Enter => self.select_theme(Theme::ALL[index], ctx),
                        _ => {}
                    }
                    ctx.input_mut(|i| {
                        i.events.retain(|e| {
                            !matches!(e, egui::Event::Key { .. } | egui::Event::Text(_))
                        })
                    });
                    return true;
                }
                if current == "theme" && matches!(key, egui::Key::Enter | egui::Key::ArrowRight) {
                    self.menu.theme_item = Some(0);
                    ctx.input_mut(|i| {
                        i.events.retain(|e| {
                            !matches!(e, egui::Event::Key { .. } | egui::Event::Text(_))
                        })
                    });
                    return true;
                }
            }
        }
        let mut action = None;
        ctx.input_mut(|i| {
            if i.key_pressed(egui::Key::Escape) {
                self.menu.active = false;
                self.menu.open = false;
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                self.menu.selected = (self.menu.selected + 1) % 5;
                self.menu.item = 0;
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                self.menu.selected = (self.menu.selected + 4) % 5;
                self.menu.item = 0;
            }
            let count = menu_commands(self.menu.selected).len();
            if i.key_pressed(egui::Key::ArrowDown) {
                if self.menu.open {
                    self.menu.item = (self.menu.item + 1) % count;
                } else {
                    self.menu.open = true;
                    self.menu.item = 0;
                }
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                self.menu.open = true;
                self.menu.item = (self.menu.item + count - 1) % count;
            }
            if i.key_pressed(egui::Key::Enter) {
                if self.menu.open {
                    action = Some(menu_commands(self.menu.selected)[self.menu.item]);
                } else {
                    self.menu.open = true;
                }
            }
            i.events
                .retain(|e| !matches!(e, egui::Event::Key { .. } | egui::Event::Text(_)));
        });
        if let Some(id) = action {
            self.menu = MenuState::default();
            let result = self.action(id, ctx);
            self.result(result);
        }
        true
    }
    fn icon_sidebar(&mut self, ctx: &egui::Context) {
        if !self.settings.icon_panel_open {
            egui::SidePanel::left("icons-collapsed")
                .frame(egui::Frame::new().fill(theme::current(ctx).background))
                .exact_width(24.)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.spacing_mut().button_padding = Vec2::ZERO;
                    ui.spacing_mut().interact_size = egui::vec2(24., 32.);
                    if panel_handle(ui, true, tr("panels.icons")) {
                        self.settings.icon_panel_open = true;
                        let r = self.save_session();
                        self.result(r);
                    }
                });
        } else {
            let mut picked = None;
            let mut close = false;
            egui::SidePanel::left("icon-sidebar")
                .default_width(88.)
                .min_width(88.)
                .max_width(240.)
                .frame(egui::Frame::side_top_panel(&ctx.style()).fill(theme::current(ctx).surface))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.;
                        ui.label(tr("panels.icons"));
                        if panel_handle(ui, false, tr("panels.close")) {
                            close = true;
                        }
                    });
                    ui.add(
                        egui::TextEdit::singleline(&mut self.icon_filter)
                            .hint_text(tr("panels.search"))
                            .desired_width((ui.available_width() - 8.).max(30.)),
                    );
                    let query = self.icon_filter.to_lowercase();
                    let tab = &self.tabs[self.active];
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let columns = (ui.available_width() / 36.).floor().max(2.) as usize;
                            egui::Grid::new("sidebar-icons")
                                .min_col_width(0.)
                                .num_columns(columns)
                                .spacing([12., 1.])
                                .show(ui, |ui| {
                                    for (index, icon) in mindglider::icons::CATALOG
                                        .iter()
                                        .filter(|i| {
                                            query.is_empty()
                                                || i.id.to_lowercase().contains(&query)
                                                || tr(i.key).to_lowercase().contains(&query)
                                        })
                                        .enumerate()
                                    {
                                        let active = tab.selection.iter().all(|id| {
                                            let n = &tab.history.doc.nodes[id];
                                            n.icons.iter().any(|i| i == icon.id)
                                                || n.progress
                                                    .is_some_and(|p| format!("{p}%") == icon.id)
                                        });
                                        let (rect, response) = ui.allocate_exact_size(
                                            egui::vec2(26., 26.),
                                            Sense::click(),
                                        );
                                        if active {
                                            ui.painter().rect_filled(
                                                rect,
                                                4.,
                                                ui.visuals().selection.bg_fill,
                                            );
                                        }
                                        if ui.is_rect_visible(rect) {
                                            paint_icon(ui, icon.id, rect.shrink(4.));
                                        }
                                        if response
                                            .on_hover_text(format!(
                                                "{} ({})",
                                                tr(icon.key),
                                                icon.id
                                            ))
                                            .clicked()
                                        {
                                            picked = Some(icon.id);
                                        }
                                        if index % columns == columns - 1 {
                                            ui.end_row();
                                        }
                                    }
                                });
                        });
                });
            if close {
                self.settings.icon_panel_open = false;
                let r = self.save_session();
                self.result(r);
            }
            if let Some(icon) = picked {
                let t = &mut self.tabs[self.active];
                let targets = t.selection.clone();
                let r = t.history.edit(tr("icons.title"), |d| {
                    mindglider::icons::toggle(d, &targets, icon)
                });
                self.result(r);
            }
        }
        if !self.settings.inspector_open {
            egui::SidePanel::right("inspector-collapsed")
                .frame(egui::Frame::new().fill(theme::current(ctx).background))
                .exact_width(24.)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.spacing_mut().button_padding = Vec2::ZERO;
                    ui.spacing_mut().interact_size = egui::vec2(24., 32.);
                    if panel_handle(ui, false, tr("panels.inspector")) {
                        self.settings.inspector_open = true;
                        let r = self.save_session();
                        self.result(r);
                    }
                });
        }
    }
    fn inspector(&mut self, ctx: &egui::Context) {
        if !self.settings.inspector_open {
            return;
        }
        let mut close_panel = false;
        let t = &self.tabs[self.active];
        let id = t.primary.clone();
        let Some(original) = t.history.doc.nodes.get(&id).cloned() else {
            return;
        };
        let mut node = original.clone();
        let locked = t.history.doc.ensure_editable(&id).is_err();
        let mut picture = false;
        let mut pick_icon = false;
        let mut follow = false;
        let mut connector = None;
        let mut remove_connector = None;
        let mut connectors = self.tabs[self.active].history.doc.connectors.clone();
        let original_connectors = connectors.clone();
        egui::SidePanel::right("inspector")
            .default_width(300.)
            .min_width(300.)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(theme::current(ctx).surface)
                    .inner_margin(12)
                    .stroke(Stroke::NONE),
            )
            .show(ctx, |ui| {
                ui.add_space(8.);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(tr("app.138")).size(17.).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("›").on_hover_text(tr("panels.close")).clicked() {
                            close_panel = true;
                        }
                    });
                });
                ui.add_space(8.);
                ui.spacing_mut().item_spacing.x = 5.;
                ui.spacing_mut().item_spacing.y = 4.;
                ui.visuals_mut().collapsing_header_frame = false;
                if locked {
                    ui.colored_label(Color32::from_rgb(180, 90, 25), tr("app.162"));
                }
                egui::ScrollArea::vertical()
                    .id_salt("inspector-scroll")
                    .show(ui, |ui| {
                        ui.add_enabled_ui(!locked, |ui| {
                            crate::theme::expander(tr("app.161"))
                                .id_salt(&id)
                                .default_open(true)
                                .show(ui, |ui| {
                                    ui.push_id("node-editor", |ui| {
                                        rich_editor(ui, "text", &mut node.text, 3)
                                    });
                                });
                            crate::theme::expander(tr("app.160"))
                                .show(ui, |ui| rich_editor(ui, "details", &mut node.details, 2));
                            crate::theme::expander(tr("app.159"))
                                .show(ui, |ui| rich_editor(ui, "note", &mut node.note, 5));
                            crate::theme::expander(tr("app.158"))
                                .show(ui, |ui| style_editor(ui, &mut node.style));
                            crate::theme::expander(tr("app.157")).show(ui, |ui| {
                                if ui
                                    .button(format!("{}  Ctrl+F2", tr("icons.title")))
                                    .clicked()
                                {
                                    pick_icon = true;
                                }
                                ui.horizontal_wrapped(|ui| {
                                    for icon in &node.icons {
                                        let (rect, response) = ui.allocate_exact_size(
                                            egui::vec2(26., 26.),
                                            Sense::hover(),
                                        );
                                        paint_icon(ui, icon, rect.shrink(3.));
                                        response.on_hover_text(
                                            mindglider::icons::find(icon)
                                                .map(|i| tr(i.key))
                                                .unwrap_or(icon),
                                        );
                                    }
                                });
                                if ui.button(tr("icons.clear")).clicked() {
                                    node.icons.clear();
                                    node.progress = None;
                                }
                                let mut enabled = node.progress.is_some();
                                if ui.checkbox(&mut enabled, tr("app.155")).changed() {
                                    node.progress = if enabled { Some(0) } else { None };
                                }
                                if let Some(p) = &mut node.progress {
                                    ui.add(egui::Slider::new(p, 0..=100).step_by(25.0).suffix("%"));
                                }
                            });
                            crate::theme::expander(tr("app.139")).show(ui, |ui| {
                                if ui.button(tr("app.154")).clicked() {
                                    picture = true;
                                }
                                let mut remove = false;
                                if let Some(p) = &mut node.image {
                                    ui.label(&p.uri);
                                    ui.horizontal(|ui| {
                                        ui.label(tr("app.153"));
                                        ui.add(
                                            egui::DragValue::new(&mut p.width).range(20.0..=1200.0),
                                        );
                                        ui.label(tr("app.152"));
                                        ui.add(
                                            egui::DragValue::new(&mut p.height)
                                                .range(20.0..=1200.0),
                                        );
                                    });
                                    remove = ui.button(tr("app.151")).clicked();
                                }
                                if remove {
                                    node.image = None;
                                }
                            });
                            crate::theme::expander(tr("app.150")).show(ui, |ui| {
                                ui.text_edit_singleline(&mut node.link);
                                ui.horizontal(|ui| {
                                    if ui.button(tr("app.149")).clicked() {
                                        follow = true;
                                    }
                                    if ui.button(tr("app.148")).clicked()
                                        && let Some(p) = rfd::FileDialog::new().pick_file()
                                    {
                                        node.link = p.to_string_lossy().into();
                                    }
                                });
                                ui.label(tr("app.147"));
                            });
                            crate::theme::expander(tr("app.146")).show(ui, |ui| {
                                let mut enabled = node.cloud.is_some();
                                if ui.checkbox(&mut enabled, tr("app.145")).changed() {
                                    node.cloud = if enabled {
                                        Some("#deebf7".into())
                                    } else {
                                        None
                                    };
                                }
                                if let Some(c) = &mut node.cloud {
                                    color_edit(ui, tr("app.144"), c);
                                }
                            });
                            crate::theme::expander(tr("app.143")).show(ui, |ui| {
                                if ui.button(tr("app.142")).clicked() {
                                    self.link_source = Some(id.clone());
                                }
                                if let Some(source) = &self.link_source {
                                    ui.label(msg("app.044", &[("source", source.to_string())]));
                                    if source != &id && ui.button(tr("app.141")).clicked() {
                                        connector = Some(source.clone());
                                    }
                                }
                                ui.text_edit_singleline(&mut self.connector_label);
                                for (index, c) in connectors
                                    .iter_mut()
                                    .enumerate()
                                    .filter(|(_, c)| c.from == id || c.to == id)
                                {
                                    ui.push_id(index, |ui| {
                                        ui.label(format!("{} → {}", c.from, c.to));
                                        ui.text_edit_singleline(&mut c.label);
                                        color_edit(ui, tr("app.140"), &mut c.color);
                                        if ui.button(tr("app.111")).clicked() {
                                            remove_connector = Some(index);
                                        }
                                    });
                                }
                            });
                        });
                    });
            });
        if close_panel {
            self.settings.inspector_open = false;
            let r = self.save_session();
            self.result(r);
        }
        if pick_icon {
            self.open_icon_picker();
        }
        if picture
            && let Some(p) = rfd::FileDialog::new()
                .add_filter(tr("app.139"), &["png", "jpg", "jpeg", "gif", "webp"])
                .pick_file()
        {
            match image::image_dimensions(&p) {
                Ok((w, h)) => {
                    node.image = Some(Picture {
                        uri: p.to_string_lossy().into_owned(),
                        width: 200.,
                        height: 200. * h as f32 / w.max(1) as f32,
                    })
                }
                Err(e) => self.status = msg("app.043", &[("e", format!("{e}"))]),
            }
        }
        if node != original {
            let r = self.tabs[self.active].history.edit(tr("app.138"), |d| {
                d.ensure_editable(&id)?;
                validate_text(&node.text.text())?;
                d.nodes.insert(id.clone(), node);
                Ok(())
            });
            self.result(r);
        }
        if follow {
            let r = self.follow_link(&id);
            self.result(r);
        }
        if let Some(source) = connector {
            let label = self.connector_label.clone();
            let r = self.tabs[self.active].history.edit(tr("app.137"), |d| {
                d.ensure_editable(&source)?;
                d.ensure_editable(&id)?;
                d.connectors.push(Connector {
                    from: source,
                    to: id.clone(),
                    color: "#6a83a5".into(),
                    label,
                });
                Ok(())
            });
            self.result(r);
            self.link_source = None;
        }
        if let Some(index) = remove_connector {
            connectors.remove(index);
        }
        if connectors != original_connectors {
            let r = self.tabs[self.active].history.edit(tr("app.136"), |d| {
                d.connectors = connectors;
                Ok(())
            });
            self.result(r);
        }
    }
    fn follow_link(&mut self, id: &str) -> Result<()> {
        let link = self.tabs[self.active].history.doc.nodes[id].link.clone();
        if let Some(target) = link.strip_prefix('#') {
            ensure!(
                self.tabs[self.active]
                    .history
                    .doc
                    .nodes
                    .contains_key(target),
                "{}",
                msg("app.042", &[])
            );
            self.tabs[self.active].select(target.into());
            return Ok(());
        }
        if link.starts_with("http://") || link.starts_with("https://") {
            webbrowser::open(&link)?;
        } else {
            ensure!(
                !link.contains("://") && !link.to_lowercase().starts_with("javascript:"),
                "{}",
                msg("app.041", &[])
            );
            let p = storage::resolve(self.tabs[self.active].path.as_deref(), &link);
            ensure!(p.exists(), "{}", msg("app.040", &[]));
            open_file(&p)?;
        }
        Ok(())
    }
    fn finish_inline(&mut self, apply: bool) -> Result<()> {
        let Some(edit) = self.inline.take() else {
            return Ok(());
        };
        if apply && edit.text != edit.original {
            let t = self
                .tabs
                .iter_mut()
                .find(|t| t.id == edit.tab_id)
                .context(tr("app.135"))?;
            t.history.edit(tr("app.134"), |d| {
                d.ensure_editable(&edit.id)?;
                let n = d.nodes.get_mut(&edit.id).context(tr("app.133"))?;
                ensure!(n.text == edit.original, "{}", msg("app.039", &[]));
                validate_text(&edit.text.text())?;
                n.text = edit.text;
                Ok(())
            })?;
        }
        Ok(())
    }
    fn canvas(&mut self, ctx: &egui::Context) {
        let mut inline = self
            .inline
            .take()
            .filter(|e| e.tab_id == self.tabs[self.active].id);
        let mut finish = false;
        let filter = if self.filter_active {
            Some(&self.filter)
        } else {
            None
        };
        let palette = theme::current(ctx);
        let t = &mut self.tabs[self.active];
        if t.layout_revision != t.history.revision {
            t.layout.build(&t.history.doc, filter, |s, size| {
                ctx.fonts(|f| {
                    f.layout_no_wrap(s.into(), FontId::proportional(size), Color32::BLACK)
                        .size()
                })
            });
            t.layout_revision = t.history.revision;
        }
        let mut move_to = None;
        let mut drop_commit = None;
        let mut double_click = false;
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).fill(palette.canvas))
            .show(ctx, |ui| {
                let available = ui.available_rect_before_wrap();
                t.canvas_rect = available;
                if t.fit_pending {
                    t.zoom = ((available.width() - 60.) / t.layout.scene.bounds.width())
                        .min((available.height() - 60.) / t.layout.scene.bounds.height())
                        .clamp(0.15, 1.);
                    t.pan = -t.layout.scene.bounds.center().to_vec2() * t.zoom;
                    if self.frame_count >= 3 {
                        t.fit_pending = false;
                    } else {
                        ctx.request_repaint();
                    }
                }
                let bg = ui.allocate_rect(available, Sense::click_and_drag());
                if t.drag.is_none()
                    && ((bg.dragged_by(egui::PointerButton::Primary)
                        && !ctx.input(|i| i.modifiers.shift))
                        || bg.dragged_by(egui::PointerButton::Middle)
                        || bg.dragged_by(egui::PointerButton::Secondary))
                {
                    t.pan += ctx.input(|i| i.pointer.delta());
                }
                if t.drag.is_none() && ui.rect_contains_pointer(available) {
                    let old = t.zoom;
                    let factor = ctx.input(|i| {
                        let gesture = i.zoom_delta();
                        if (gesture - 1.).abs() > f32::EPSILON {
                            gesture
                        } else {
                            (i.smooth_scroll_delta.y * 0.004).exp()
                        }
                    });
                    let new = (old * factor).clamp(0.15, 3.);
                    if let Some(p) = ctx.input(|i| i.pointer.hover_pos()) {
                        let cursor = p - available.center();
                        t.pan = cursor - (cursor - t.pan) * (new / old);
                    }
                    t.zoom = new;
                }
                if t.ensure_visible {
                    if let Some(p) = t.layout.scene.nodes.iter().find(|p| p.id == t.primary) {
                        let r = Rect::from_center_size(
                            available.center() + t.pan + p.rect.center().to_vec2() * t.zoom,
                            p.rect.size() * t.zoom,
                        );
                        let safe = available.shrink(24.);
                        if r.left() < safe.left() {
                            t.pan.x += safe.left() - r.left();
                        } else if r.right() > safe.right() {
                            t.pan.x -= r.right() - safe.right();
                        }
                        if r.top() < safe.top() {
                            t.pan.y += safe.top() - r.top();
                        } else if r.bottom() > safe.bottom() {
                            t.pan.y -= r.bottom() - safe.bottom();
                        }
                    }
                    t.ensure_visible = false;
                }
                let origin = available.center() + t.pan;
                let zoom = t.zoom;
                let point = |p: Pos2| origin + p.to_vec2() * zoom;
                let rect = |r: Rect| Rect::from_min_max(point(r.min), point(r.max));
                let painter = ui.painter().clone();
                for (r, c) in &t.layout.scene.clouds {
                    let r = rect(*r);
                    if r.intersects(available) {
                        painter.rect_filled(r, 18., color(c).gamma_multiply(0.6));
                    }
                }
                let mut placed = t.layout.scene.nodes.clone();
                for node in &mut placed {
                    palette.map_style(&mut node.style);
                }
                let by_id: BTreeMap<_, _> = placed.iter().map(|n| (n.id.as_str(), n)).collect();
                for (a, b) in &t.layout.scene.edges {
                    if let (Some(a), Some(b)) = (by_id.get(a.as_str()), by_id.get(b.as_str())) {
                        let sign = if b.rect.center().x < a.rect.center().x {
                            -1.
                        } else {
                            1.
                        };
                        let p1 = point(mindglider::layout::branch_anchor(a, sign));
                        let p2 = point(mindglider::layout::branch_anchor(b, -sign));
                        let mid = (p1.x + p2.x) / 2.;
                        painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                            [p1, Pos2::new(mid, p1.y), Pos2::new(mid, p2.y), p2],
                            false,
                            Color32::TRANSPARENT,
                            Stroke::new(1.8_f32 * zoom, color(&b.style.edge_color)),
                        ));
                    }
                }
                for c in &t.history.doc.connectors {
                    if let (Some(a), Some(b)) =
                        (by_id.get(c.from.as_str()), by_id.get(c.to.as_str()))
                    {
                        let a = point(a.rect.center());
                        let b = point(b.rect.center());
                        let direction = (b - a).normalized();
                        let tip = b - direction * 30. * zoom;
                        let normal = egui::vec2(-direction.y, direction.x);
                        painter.line_segment([a, tip], Stroke::new(1.5_f32, color(&c.color)));
                        painter.add(egui::Shape::convex_polygon(
                            vec![
                                tip,
                                tip - direction * 10. * zoom + normal * 4. * zoom,
                                tip - direction * 10. * zoom - normal * 4. * zoom,
                            ],
                            color(&c.color),
                            Stroke::NONE,
                        ));
                        if !c.label.is_empty() {
                            painter.text(
                                a.lerp(b, 0.5),
                                Align2::CENTER_BOTTOM,
                                &c.label,
                                FontId::proportional(12. * zoom),
                                color(&c.color),
                            );
                        }
                    }
                }
                for p in &placed {
                    let r = rect(p.rect);
                    if !r.intersects(available.expand(40.)) {
                        continue;
                    }
                    let n = &t.history.doc.nodes[&p.id];
                    let selected = t.selection.contains(&p.id);
                    let border = if selected {
                        Stroke::new(2.5_f32, palette.accent)
                    } else {
                        Stroke::new(1.0_f32, color(&p.style.edge_color))
                    };
                    draw_map_node(
                        ui,
                        n,
                        p,
                        r,
                        zoom,
                        border,
                        t.path.as_deref(),
                        !inline.as_ref().is_some_and(|e| e.id == p.id),
                    );
                    let response = ui.interact(r, ui.id().with(&p.id), Sense::click_and_drag());
                    if response.clicked() {
                        if ctx.input(|i| i.modifiers.ctrl || i.modifiers.shift) {
                            if !t.selection.insert(p.id.clone()) {
                                t.selection.remove(&p.id);
                            }
                            t.primary = p.id.clone();
                        } else {
                            t.select(p.id.clone());
                        }
                    }
                    if response.double_clicked() {
                        t.select(p.id.clone());
                        double_click = true;
                    }
                    if response.drag_started_by(egui::PointerButton::Primary)
                        && p.id != t.history.doc.root
                        && t.history.doc.ensure_branch_editable(&p.id).is_ok()
                    {
                        let pointer = ctx
                            .input(|i| i.pointer.press_origin())
                            .unwrap_or(r.center());
                        t.drag = Some(DragState {
                            id: p.id.clone(),
                            grab: (pointer - origin) / zoom - p.rect.min.to_vec2(),
                        });
                        t.select(p.id.clone());
                    }
                    response.context_menu(|ui| {
                        if ui.button(tr("app.132")).clicked() {
                            move_to = Some((p.id.clone(), String::new()));
                            ui.close_menu();
                        }
                        if ui.button(tr("app.131")).clicked() {
                            t.select(p.id.clone());
                            ui.close_menu();
                        }
                    });
                }
                if let Some(drag) = t.drag.clone() {
                    let cancel = ctx
                        .input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
                        || !ctx.input(|i| i.focused);
                    let pointer = ctx.input(|i| i.pointer.hover_pos());
                    let mut candidate = None;
                    if !cancel && let Some(pointer) = pointer {
                        if let Some(target) = placed
                            .iter()
                            .find(|p| available.contains(pointer) && rect(p.rect).contains(pointer))
                        {
                            let zone = mindglider::dnd::zone(
                                rect(target.rect),
                                pointer,
                                target.id == t.history.doc.root,
                            );
                            let plan = mindglider::dnd::destination(
                                &t.history.doc,
                                &drag.id,
                                &target.id,
                                zone,
                            );
                            let valid = plan.is_ok();
                            let tint = if valid {
                                palette.accent
                            } else {
                                Color32::from_rgb(196, 43, 28)
                            };
                            let highlight = mindglider::dnd::highlight(rect(target.rect), zone);
                            let mut mesh = egui::Mesh::default();
                            for pos in [
                                highlight.left_top(),
                                highlight.right_top(),
                                highlight.right_bottom(),
                                highlight.left_bottom(),
                            ] {
                                let t = match zone {
                                    mindglider::dnd::Zone::Left => {
                                        (highlight.right() - pos.x) / highlight.width()
                                    }
                                    mindglider::dnd::Zone::Right => {
                                        (pos.x - highlight.left()) / highlight.width()
                                    }
                                    mindglider::dnd::Zone::Before => {
                                        (highlight.bottom() - pos.y) / highlight.height()
                                    }
                                    mindglider::dnd::Zone::After => {
                                        (pos.y - highlight.top()) / highlight.height()
                                    }
                                };
                                mesh.colored_vertex(pos, tint.gamma_multiply(0.05 + 0.3 * t));
                            }
                            mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
                            painter.add(mesh);
                            painter.rect_stroke(
                                highlight,
                                2.,
                                Stroke::new(1.5_f32, tint),
                                StrokeKind::Inside,
                            );
                            candidate = plan.ok();
                        }
                        ctx.set_cursor_icon(if candidate.is_some() {
                            egui::CursorIcon::Grabbing
                        } else {
                            egui::CursorIcon::NotAllowed
                        });
                        if let Some(source) = placed.iter().find(|p| p.id == drag.id) {
                            let delta =
                                (pointer - origin) / zoom - drag.grab - source.rect.min.to_vec2();
                            let branch = t.history.doc.subtree(&drag.id);
                            ui.scope_builder(
                                egui::UiBuilder::new().layer_id(egui::LayerId::new(
                                    egui::Order::Foreground,
                                    egui::Id::new("drag-tree"),
                                )),
                                |ui| {
                                    ui.set_clip_rect(available);
                                    ui.set_opacity(0.55);
                                    let ghost = ui.painter().clone();
                                    for p in placed.iter().filter(|p| branch.contains(&p.id)) {
                                        if let Some(c) = &t.history.doc.nodes[&p.id].cloud {
                                            let descendants = t.history.doc.subtree(&p.id);
                                            if let Some(bounds) = placed
                                                .iter()
                                                .filter(|p| descendants.contains(&p.id))
                                                .map(|p| p.rect)
                                                .reduce(Rect::union)
                                            {
                                                ghost.rect_filled(
                                                    rect(bounds.expand(14.).translate(delta)),
                                                    18.,
                                                    color(c).gamma_multiply(0.6),
                                                );
                                            }
                                        }
                                    }
                                    for c in &t.history.doc.connectors {
                                        if branch.contains(&c.from)
                                            && branch.contains(&c.to)
                                            && let (Some(a), Some(b)) = (
                                                by_id.get(c.from.as_str()),
                                                by_id.get(c.to.as_str()),
                                            )
                                        {
                                            let a = point(a.rect.center() + delta);
                                            let b = point(b.rect.center() + delta);
                                            ghost.line_segment(
                                                [a, b],
                                                Stroke::new(1.5_f32, color(&c.color)),
                                            );
                                            if !c.label.is_empty() {
                                                ghost.text(
                                                    a.lerp(b, 0.5),
                                                    Align2::CENTER_BOTTOM,
                                                    &c.label,
                                                    FontId::proportional(12. * zoom),
                                                    color(&c.color),
                                                );
                                            }
                                        }
                                    }

                                    for (a, b) in &t.layout.scene.edges {
                                        if branch.contains(a)
                                            && branch.contains(b)
                                            && let (Some(a), Some(b)) =
                                                (by_id.get(a.as_str()), by_id.get(b.as_str()))
                                        {
                                            let sign = if b.rect.center().x < a.rect.center().x {
                                                -1.
                                            } else {
                                                1.
                                            };
                                            let p1 = point(
                                                mindglider::layout::branch_anchor(a, sign) + delta,
                                            );
                                            let p2 = point(
                                                mindglider::layout::branch_anchor(b, -sign) + delta,
                                            );
                                            let mid = (p1.x + p2.x) / 2.;
                                            ghost.add(
                                                egui::epaint::CubicBezierShape::from_points_stroke(
                                                    [
                                                        p1,
                                                        Pos2::new(mid, p1.y),
                                                        Pos2::new(mid, p2.y),
                                                        p2,
                                                    ],
                                                    false,
                                                    Color32::TRANSPARENT,
                                                    Stroke::new(
                                                        1.8 * zoom,
                                                        color(&b.style.edge_color),
                                                    ),
                                                ),
                                            );
                                        }
                                    }
                                    for p in placed.iter().filter(|p| branch.contains(&p.id)) {
                                        let r = rect(p.rect.translate(delta));
                                        if r.intersects(available) {
                                            draw_map_node(
                                                ui,
                                                &t.history.doc.nodes[&p.id],
                                                p,
                                                r,
                                                zoom,
                                                Stroke::new(1.0_f32, color(&p.style.edge_color)),
                                                t.path.as_deref(),
                                                true,
                                            );
                                        }
                                    }
                                },
                            );
                        }
                    }
                    if cancel {
                        t.drag = None;
                    } else if ctx.input(|i| i.pointer.button_released(egui::PointerButton::Primary))
                    {
                        drop_commit = candidate;
                        t.drag = None;
                    } else if !ctx.input(|i| i.pointer.primary_down()) {
                        t.drag = None;
                    }
                }
                if let Some(edit) = inline.as_mut()
                    && let Some(p) = placed.iter().find(|p| p.id == edit.id)
                {
                    let mut r = rect(p.rect);
                    let pad = mindglider::layout::padding(&p.style);
                    let node = &t.history.doc.nodes[&edit.id];
                    let icon_width =
                        mindglider::layout::icon_width(mindglider::layout::icon_count(node));
                    let mut measurement = text_job(&edit.text, &p.style, zoom);
                    measurement.wrap.max_width = 400. * zoom;
                    let text_size = ctx.fonts(|f| f.layout_job(measurement).size());
                    r.max.x = r
                        .max
                        .x
                        .max(r.min.x + text_size.x + (2. * pad.x + icon_width) * zoom);
                    r.max.y = r.max.y.max(r.min.y + text_size.y + 2. * pad.y * zoom);
                    paint_node(&painter, r, &p.style, Stroke::new(2.5_f32, palette.accent));
                    if p.style.shape == "fork" {
                        painter.rect_filled(
                            Rect::from_min_size(
                                r.min + egui::vec2(pad.x + icon_width, pad.y) * zoom,
                                text_size,
                            ),
                            0.,
                            color(&p.style.background),
                        );
                    }
                    let node = &t.history.doc.nodes[&edit.id];
                    let visual_icons: Vec<_> = node
                        .icons
                        .iter()
                        .cloned()
                        .chain(node.progress.map(|p| format!("{p}%")))
                        .collect();
                    for (index, icon) in visual_icons.iter().enumerate() {
                        paint_icon(
                            ui,
                            icon,
                            Rect::from_min_size(
                                r.min + egui::vec2(pad.x + index as f32 * 18., pad.y) * zoom,
                                egui::vec2(16., 16.) * zoom,
                            ),
                        );
                    }
                    let editor_id = ui.id().with(("inline-editor", &edit.id));
                    let mut text = edit.text.text();
                    let original = edit.text.clone();
                    let style = p.style.clone();
                    let icons_width = mindglider::layout::icon_width(
                        mindglider::layout::icon_count(&t.history.doc.nodes[&edit.id]),
                    );
                    let min = r.min + egui::vec2(pad.x + icons_width, pad.y) * zoom;
                    let width = (r.width() - (2. * pad.x + icons_width) * zoom)
                        .max(60. * zoom)
                        .min(available.right() - min.x - 4.);
                    let edit_rect = Rect::from_min_size(
                        min,
                        egui::vec2(
                            width,
                            ((text.lines().count() + 1) as f32 * style.font_size * 1.5 * zoom)
                                .max(28. * zoom),
                        ),
                    );
                    let mut layouter = move |ui: &egui::Ui, s: &str, w: f32| {
                        let mut rt = original.clone();
                        rt.edit(s);
                        let mut job = text_job(&rt, &style, zoom);
                        job.wrap.max_width = w;
                        ui.fonts(|f| f.layout_job(job))
                    };
                    let output =
                        ui.scope_builder(egui::UiBuilder::new().max_rect(edit_rect), |ui| {
                            egui::TextEdit::multiline(&mut text)
                                .id(editor_id)
                                .margin(egui::Margin::ZERO)
                                .frame(false)
                                .desired_width(width)
                                .desired_rows(1)
                                .return_key(egui::KeyboardShortcut::new(
                                    egui::Modifiers::SHIFT,
                                    egui::Key::Enter,
                                ))
                                .layouter(&mut layouter)
                                .show(ui)
                        });
                    let mut out = output.inner;
                    if edit.focus {
                        out.response.request_focus();
                        out.state
                            .cursor
                            .set_char_range(Some(egui::text::CCursorRange::two(
                                egui::text::CCursor::new(0),
                                egui::text::CCursor::new(text.chars().count()),
                            )));
                        out.state.store(ctx, editor_id);
                        edit.focus = false;
                        ctx.request_repaint();
                    } else if out.response.lost_focus() {
                        finish = true;
                    }
                    if out.response.changed() {
                        edit.text.edit(&text);
                    }
                }
                if bg.dragged_by(egui::PointerButton::Primary)
                    && ctx.input(|i| i.modifiers.shift)
                    && t.drag.is_none()
                    && let Some(start) = ctx.input(|i| i.pointer.press_origin())
                    && let Some(end) = ctx.input(|i| i.pointer.hover_pos())
                {
                    let sel = Rect::from_two_pos(start, end);
                    painter.rect_stroke(
                        sel,
                        0.,
                        Stroke::new(1.0_f32, palette.accent),
                        StrokeKind::Inside,
                    );
                    t.selection = placed
                        .iter()
                        .filter(|p| rect(p.rect).intersects(sel))
                        .map(|p| p.id.clone())
                        .collect();
                    if let Some(id) = t.selection.first() {
                        t.primary = id.clone();
                    }
                }
                ui.allocate_rect(available, Sense::hover());
            });
        if let Some(drop) = drop_commit {
            t.ensure_visible = true;
            let result = t
                .history
                .edit(tr("app.129"), |d| mindglider::dnd::apply(d, &drop));
            if let Err(e) = result {
                self.status = e.to_string();
            }
        }
        if let Some((id, parent)) = move_to {
            let r = t.history.edit(
                if parent.is_empty() {
                    tr("app.130")
                } else {
                    tr("app.129")
                },
                |d| {
                    if parent.is_empty() {
                        let n = d.nodes.get_mut(&id).unwrap();
                        n.folded = !n.folded;
                        Ok(())
                    } else {
                        let index = d.nodes[&parent].children.len();
                        d.move_node(&id, &parent, index)
                    }
                },
            );
            if let Err(e) = r {
                self.status = e.to_string();
            }
        }
        self.inline = inline;
        if finish {
            let result = self.finish_inline(true);
            self.result(result);
        }
        if double_click {
            let result = self.action("edit", ctx);
            self.result(result);
        }
    }
    fn open_icon_picker(&mut self) {
        let tab = &self.tabs[self.active];
        self.icon_picker = Some(IconPicker {
            tab_id: tab.id.clone(),
            targets: tab.selection.clone(),
            focus: true,
            ..Default::default()
        });
    }
    fn icon_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut picker) = self.icon_picker.take() else {
            return;
        };
        let mut open = true;
        let mut change: Option<String> = None;
        let mut remove = None;
        let mut enter = false;
        let mut navigation = 0isize;
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                open = false;
            }
            enter = i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
            for (key, delta) in [
                (egui::Key::ArrowLeft, -1),
                (egui::Key::ArrowRight, 1),
                (egui::Key::ArrowUp, -8),
                (egui::Key::ArrowDown, 8),
            ] {
                if i.consume_key(egui::Modifiers::NONE, key) {
                    navigation += delta;
                }
            }
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Delete) {
                remove = Some(1);
            }
            if i.consume_key(egui::Modifiers::SHIFT, egui::Key::Delete) {
                remove = Some(2);
            }
        });
        crate::theme::window(ctx, tr("icons.title"), "icons-picker")
            .default_width(480.)
            .show(ctx, |ui| {
                crate::theme::dialog_header(ui, tr("icons.title"), &mut open);
                let search = ui.add(
                    egui::TextEdit::singleline(&mut picker.query)
                        .hint_text(tr("icons.search"))
                        .desired_width(f32::INFINITY),
                );
                if picker.focus {
                    search.request_focus();
                    picker.focus = false;
                }
                if search.changed() {
                    picker.index = 0;
                }
                let query = picker.query.to_lowercase();
                let items: Vec<_> = mindglider::icons::CATALOG
                    .iter()
                    .filter(|icon| {
                        query.is_empty()
                            || icon.id.to_lowercase().contains(&query)
                            || tr(icon.key).to_lowercase().contains(&query)
                    })
                    .collect();
                picker.index = (picker.index as isize + navigation)
                    .clamp(0, items.len().saturating_sub(1) as isize)
                    as usize;
                if enter && let Some(icon) = items.get(picker.index) {
                    change = Some(icon.id.into());
                }
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height((ctx.screen_rect().height() - 280.).clamp(80., 320.))
                    .show(ui, |ui| {
                        if items.is_empty() {
                            ui.label(tr("icons.empty"));
                        }
                        egui::Grid::new("icon-grid")
                            .num_columns(8)
                            .spacing([8., 8.])
                            .show(ui, |ui| {
                                for (index, icon) in items.iter().enumerate() {
                                    let (rect, response) = ui
                                        .allocate_exact_size(egui::vec2(46., 42.), Sense::click());
                                    if index == picker.index {
                                        ui.painter().rect_filled(
                                            rect,
                                            4.,
                                            ui.visuals().selection.bg_fill,
                                        );
                                        ui.painter().rect_stroke(
                                            rect,
                                            4.,
                                            ui.visuals().selection.stroke,
                                            StrokeKind::Inside,
                                        );
                                        if navigation != 0 || search.changed() {
                                            ui.scroll_to_rect(rect, Some(egui::Align::Center));
                                        }
                                    }
                                    paint_icon(
                                        ui,
                                        icon.id,
                                        Rect::from_center_size(rect.center(), egui::vec2(24., 24.)),
                                    );
                                    if response
                                        .on_hover_text(format!("{} ({})", tr(icon.key), icon.id))
                                        .clicked()
                                    {
                                        change = Some(icon.id.into());
                                    }
                                    if index % 8 == 7 {
                                        ui.end_row();
                                    }
                                }
                            });
                    });
                if let Some(icon) = items.get(picker.index) {
                    ui.label(tr(icon.key));
                }
                ui.label(tr("icons.hint"));
                ui.horizontal(|ui| {
                    for (key, action) in [("icons.first", 0), ("icons.last", 1), ("icons.clear", 2)]
                    {
                        if ui.button(tr(key)).clicked() {
                            remove = Some(action);
                        }
                    }
                });
            });
        if change.is_some() || remove.is_some() {
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == picker.tab_id) {
                let result = tab.history.edit(tr("icons.title"), |doc| {
                    for id in &picker.targets {
                        doc.ensure_editable(id)?;
                        let node = doc.nodes.get_mut(id).unwrap();
                        if let Some(icon) = &change {
                            if let Some(value) =
                                icon.strip_suffix('%').and_then(|v| v.parse::<u8>().ok())
                            {
                                node.progress = Some(value);
                            } else {
                                node.icons.push(icon.clone());
                            }
                        }
                        match remove {
                            Some(0) if !node.icons.is_empty() => {
                                node.icons.remove(0);
                            }
                            Some(0) => node.progress = None,
                            Some(1) if node.progress.is_some() => node.progress = None,
                            Some(1) => {
                                node.icons.pop();
                            }
                            Some(2) => {
                                node.icons.clear();
                                node.progress = None;
                            }
                            _ => {}
                        }
                    }
                    Ok(())
                });
                self.result(result);
            }
            open = false;
        }
        if open {
            self.icon_picker = Some(picker);
        }
    }
    fn dialogs(&mut self, ctx: &egui::Context) {
        self.icon_dialog(ctx);
        if self.find_open {
            let mut open = true;
            let mut replace_all = false;
            let mut next = false;
            crate::theme::window(ctx, tr("app.128"), "app.128")
                .default_width(420.)
                .show(ctx, |ui| {
                    crate::theme::dialog_header(ui, tr("app.128"), &mut open);
                    ui.horizontal(|ui| {
                        ui.label(tr("app.127"));
                        ui.text_edit_singleline(&mut self.query);
                    });
                    ui.horizontal(|ui| {
                        ui.label(tr("app.126"));
                        ui.text_edit_singleline(&mut self.replacement);
                    });
                    ui.checkbox(&mut self.search_case, tr("app.125"));
                    ui.horizontal(|ui| {
                        next = ui.button(tr("app.124")).clicked();
                        replace_all = ui.button(tr("app.123")).clicked();
                    });
                    let t = &self.tabs[self.active];
                    let hits: Vec<_> = t
                        .history
                        .doc
                        .nodes
                        .values()
                        .filter(|n| {
                            search_match(
                                &format!(
                                    "{}\n{}\n{}",
                                    n.text.text(),
                                    n.note.text(),
                                    n.details.text()
                                ),
                                &self.query,
                                self.search_case,
                            )
                        })
                        .map(|n| (n.id.clone(), n.text.text()))
                        .collect();
                    ui.label(msg("app.038", &[("0", format!("{}", hits.len()))]));
                    let mut select = None;
                    egui::ScrollArea::vertical()
                        .max_height(220.)
                        .show(ui, |ui| {
                            for (id, label) in &hits {
                                if ui.selectable_label(id == &t.primary, label).clicked() {
                                    select = Some(id.clone());
                                }
                            }
                        });
                    if next {
                        let i = hits
                            .iter()
                            .position(|h| h.0 == t.primary)
                            .map(|i| (i + 1) % hits.len().max(1))
                            .unwrap_or(0);
                        select = hits.get(i).map(|h| h.0.clone());
                    }
                    if let Some(id) = select {
                        self.tabs[self.active].select(id);
                    }
                });
            self.find_open = open;
            if replace_all && !self.query.is_empty() {
                let query = self.query.clone();
                let replacement = self.replacement.clone();
                let case = self.search_case;
                let r = self.tabs[self.active].history.edit(tr("app.122"), |d| {
                    let pattern = regex::RegexBuilder::new(&regex::escape(&query))
                        .case_insensitive(!case)
                        .build()?;
                    for n in d.nodes.values_mut() {
                        if n.protected {
                            continue;
                        }
                        for rich in [&mut n.text, &mut n.note, &mut n.details] {
                            if rich.unsupported {
                                continue;
                            }
                            let text = rich.text();
                            let new = pattern.replace_all(&text, regex::NoExpand(&replacement));
                            if new != text {
                                rich.edit(&new);
                            }
                        }
                    }
                    Ok(())
                });
                self.result(r);
            }
        }
        if self.filter_open {
            let mut open = true;
            let mut apply = false;
            let mut clear = false;
            crate::theme::window(ctx, tr("app.121"), "app.121")
                .default_width(420.)
                .show(ctx, |ui| {
                    crate::theme::dialog_header(ui, tr("app.121"), &mut open);
                    egui::ScrollArea::vertical()
                        .max_height(500.)
                        .show(ui, |ui| condition_editor(ui, &mut self.filter, 0));
                    if let Err(e) = self.filter.validate() {
                        ui.colored_label(Color32::RED, e.to_string());
                    }
                    ui.horizontal(|ui| {
                        apply = ui.button(tr("app.120")).clicked();
                        clear = ui.button(tr("app.119")).clicked();
                    });
                    ui.label(tr("app.118"));
                });
            self.filter_open = open;
            if apply {
                match self.filter.validate() {
                    Ok(()) => {
                        self.filter_active = true;
                        self.tabs[self.active].layout_revision = u64::MAX;
                    }
                    Err(e) => self.status = e.to_string(),
                }
            }
            if clear {
                self.filter_active = false;
                self.tabs[self.active].layout_revision = u64::MAX;
            }
        }
        if self.styles_open {
            let mut open = true;
            let tab = &self.tabs[self.active];
            let draft = self
                .style_draft
                .take()
                .filter(|d| d.tab_id == tab.id)
                .unwrap_or_else(|| StyleDraft {
                    tab_id: tab.id.clone(),
                    styles: tab.history.doc.styles.clone(),
                    rules: tab.history.doc.rules.clone(),
                });
            let mut styles = draft.styles;
            let mut rules = draft.rules;
            let mut apply_styles = false;
            let mut remove = None;
            let mut shift = None;
            crate::theme::window(ctx, tr("app.117"), "app.117")
                .default_width(580.)
                .show(ctx, |ui| {
                    crate::theme::dialog_header(ui, tr("app.117"), &mut open);
                    egui::ScrollArea::vertical()
                        .max_height(620.)
                        .show(ui, |ui| {
                            ui.heading(tr("app.116"));
                            for (name, style) in &mut styles {
                                crate::theme::expander(name).id_salt(name).show(ui, |ui| {
                                    ui.push_id(name, |ui| style_editor(ui, style));
                                });
                            }
                            if ui.button(tr("app.115")).clicked() {
                                let t = &self.tabs[self.active];
                                let mut i = styles.len() + 1;
                                while styles.contains_key(&msg(
                                    "style.default_name",
                                    &[("i", i.to_string())],
                                )) {
                                    i += 1;
                                }
                                styles.insert(
                                    msg("style.default_name", &[("i", i.to_string())]),
                                    t.history.doc.nodes[&t.primary].style.clone(),
                                );
                            }
                            ui.separator();
                            ui.heading(tr("app.114"));
                            for (index, rule) in rules.iter_mut().enumerate() {
                                ui.push_id(index, |ui| {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.checkbox(&mut rule.enabled, tr("app.113"));
                                            ui.checkbox(&mut rule.stop, tr("app.112"));
                                            if ui.button(tr("ui.up_symbol")).clicked() {
                                                shift = Some((index, index.saturating_sub(1)));
                                            }
                                            if ui.button(tr("ui.down_symbol")).clicked() {
                                                shift = Some((index, index + 1));
                                            }
                                            if ui.button(tr("app.111")).clicked() {
                                                remove = Some(index);
                                            }
                                        });
                                        egui::ComboBox::from_id_salt("style-choice")
                                            .icon(|ui, rect, visuals, _, _| {
                                                crate::theme::chevron(
                                                    ui,
                                                    rect.center(),
                                                    0.,
                                                    visuals.fg_stroke.color,
                                                )
                                            })
                                            .selected_text(&rule.style)
                                            .show_ui(ui, |ui| {
                                                for name in styles.keys() {
                                                    ui.selectable_value(
                                                        &mut rule.style,
                                                        name.clone(),
                                                        name,
                                                    );
                                                }
                                            });
                                        condition_editor(ui, &mut rule.condition, 0);
                                        if let Err(e) = rule.condition.validate() {
                                            ui.colored_label(Color32::RED, e.to_string());
                                        }
                                    });
                                });
                            }
                            if ui
                                .add_enabled(!styles.is_empty(), egui::Button::new(tr("app.110")))
                                .clicked()
                            {
                                rules.push(Rule {
                                    enabled: true,
                                    stop: false,
                                    style: styles.keys().next().unwrap().clone(),
                                    condition: Condition::default(),
                                });
                            }
                        });
                    apply_styles = ui
                        .add_enabled(
                            rules.iter().all(|r| r.condition.validate().is_ok()),
                            egui::Button::new(tr("app.109")),
                        )
                        .clicked();
                });
            self.styles_open = open;
            if let Some(i) = remove {
                rules.remove(i);
            }
            if let Some((i, j)) = shift
                && j < rules.len()
            {
                rules.swap(i, j);
            }
            self.style_draft = if open {
                Some(StyleDraft {
                    tab_id: self.tabs[self.active].id.clone(),
                    styles: styles.clone(),
                    rules: rules.clone(),
                })
            } else {
                None
            };
            if apply_styles {
                let r = self.tabs[self.active].history.edit(tr("app.108"), |d| {
                    for r in &rules {
                        r.condition.validate()?;
                    }
                    d.styles = styles;
                    d.rules = rules;
                    Ok(())
                });
                self.result(r);
            }
        }
        if self.settings_open {
            self.settings_dialog(ctx);
        }
        if self.palette_open {
            let mut open = true;
            let mut command = None;
            crate::theme::window(ctx, tr("app.107"), "app.107")
                .default_width(450.)
                .show(ctx, |ui| {
                    crate::theme::dialog_header(ui, tr("app.107"), &mut open);
                    let response = ui.text_edit_singleline(&mut self.palette);
                    if response.gained_focus() || self.palette.is_empty() {
                        response.request_focus();
                    }
                    let query = self.palette.to_lowercase();
                    let matches: Vec<_> = commands()
                        .into_iter()
                        .filter(|c| format!("{} {}", c.0, c.1).to_lowercase().contains(&query))
                        .collect();
                    egui::ScrollArea::vertical()
                        .max_height(360.)
                        .show(ui, |ui| {
                            for (id, label, key) in &matches {
                                if ui.button(format!("{label}    {key}")).clicked() {
                                    command = Some(*id);
                                }
                            }
                        });
                    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        command = matches.first().map(|c| c.0);
                    }
                });
            self.palette_open = open;
            if let Some(c) = command {
                self.palette_open = false;
                let r = self.action(c, ctx);
                self.result(r);
            }
        }
        if self.about_open {
            crate::theme::window(ctx, tr("app.106"), "app.106").show(ctx, |ui| {
                crate::theme::dialog_header(ui, tr("app.106"), &mut self.about_open);
                ui.label(concat!("MindGlider ", env!("CARGO_PKG_VERSION")));
                ui.label(tr("about.stack"));
                ui.label("Copyright © 2026 GisukeStonewell and contributors");
                ui.label(tr("about.license"));
                ui.hyperlink_to(
                    tr("about.source"),
                    "https://github.com/GisukeStonewell/MindGlider",
                );
                ui.hyperlink_to(
                    tr("about.terms"),
                    "https://github.com/GisukeStonewell/MindGlider/blob/main/LICENSE.txt",
                );
            });
        }
        self.ai_dialog(ctx);
        self.export_dialog(ctx);
        self.close_dialog(ctx);
    }
    fn refresh_catalog(&mut self, ctx: &egui::Context) {
        let settings = self.settings.cli.clone();
        let key = format!(
            "{:?}:{}",
            settings.provider,
            match settings.provider {
                Provider::Codex => &settings.codex,
                Provider::Claude => &settings.claude,
            }
        );
        self.catalog = mindglider::cli_catalog::local(settings.provider);
        self.catalog_key = key.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.catalog_task = Some((key, rx));
        let work = self.data.join("cli-catalog");
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = match std::fs::create_dir_all(&work) {
                Ok(()) => mindglider::cli_catalog::discover(&settings, &work),
                Err(e) => mindglider::cli_catalog::Catalog {
                    warning: e.to_string(),
                    ..Default::default()
                },
            };
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
    fn settings_dialog(&mut self, ctx: &egui::Context) {
        if let Some((key, receiver)) = &self.catalog_task
            && let Ok(catalog) = receiver.try_recv()
        {
            if *key == self.catalog_key {
                self.catalog = catalog;
            }
            self.catalog_task = None;
        }
        let key = format!(
            "{:?}:{}",
            self.settings.cli.provider,
            match self.settings.cli.provider {
                Provider::Codex => &self.settings.cli.codex,
                Provider::Claude => &self.settings.cli.claude,
            }
        );
        if key != self.catalog_key && self.settings_page == 1 {
            self.refresh_catalog(ctx);
        }
        let models: Vec<_> = std::iter::once((String::new(), tr("cli.default").to_owned()))
            .chain(
                self.catalog
                    .models
                    .iter()
                    .map(|m| (m.id.clone(), format!("{} ({})", m.label, m.id))),
            )
            .collect();
        let active = self.settings.cli.active_model();
        let model = self
            .catalog
            .models
            .iter()
            .find(|m| m.id == active || (active.is_empty() && m.is_default));
        let efforts: Vec<_> = std::iter::once((String::new(), tr("cli.default").to_owned()))
            .chain(
                model
                    .into_iter()
                    .flat_map(|m| m.efforts.iter().map(|s| (s.clone(), s.clone()))),
            )
            .collect();
        let mut open = true;
        let mut save = false;
        let mut reset_all = false;
        let mut valid_timeout = true;
        let mut reload = false;
        let old_language = self.settings.language.clone();
        let defaults = Settings::default();
        crate::theme::window(ctx, tr("app.105"), "app.105")
            .default_width(850.)
            .show(ctx, |ui| {
                crate::theme::dialog_header(ui, tr("app.105"), &mut open);
                ui.horizontal(|ui| {
                    for (index, label) in [
                        tr("app.104"),
                        tr("ui.cli_heading"),
                        tr("app.088"),
                        tr("app.082"),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let response =
                            crate::theme::flat_button(ui, label, self.settings_page == index);
                        if response.clicked() {
                            self.settings_page = index;
                        }
                        if self.settings_page == index {
                            ui.painter().hline(
                                response.rect.x_range(),
                                response.rect.bottom(),
                                Stroke::new(2.0_f32, theme::current(ctx).accent),
                            );
                        }
                    }
                });
                ui.add_space(16.);
                egui::Frame::new()
                    .fill(theme::current(ctx).background)
                    .corner_radius(8)
                    .inner_margin(16)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        egui::ScrollArea::vertical()
                            .id_salt(("settings-page", self.settings_page))
                            .auto_shrink([false, true])
                            .max_height((ctx.screen_rect().height() - 280.).clamp(200., 520.))
                            .show(ui, |ui| {
                                ui.set_min_height(360.);
                                match self.settings_page {
                                    0 => {
                                        ui.heading(tr("app.104"));
                                        egui::Grid::new("display-preferences")
                                            .num_columns(3)
                                            .spacing([16., 10.])
                                            .show(ui, |ui| {
                                                ui.label(tr("app.103"));
                                                egui::ComboBox::from_id_salt("language")
                                                    .icon(|ui, rect, visuals, _, _| {
                                                        crate::theme::chevron(
                                                            ui,
                                                            rect.center(),
                                                            0.,
                                                            visuals.fg_stroke.color,
                                                        )
                                                    })
                                                    .selected_text(
                                                        if self.settings.language == "ja" {
                                                            tr("app.102")
                                                        } else {
                                                            tr("language.en")
                                                        },
                                                    )
                                                    .show_ui(ui, |ui| {
                                                        ui.selectable_value(
                                                            &mut self.settings.language,
                                                            "ja".into(),
                                                            tr("app.102"),
                                                        );
                                                        ui.selectable_value(
                                                            &mut self.settings.language,
                                                            "en".into(),
                                                            tr("language.en"),
                                                        );
                                                    });
                                                if reset_button(ui) {
                                                    self.settings.language =
                                                        defaults.language.clone();
                                                }
                                                ui.end_row();
                                            });
                                        ui.horizontal(|ui| {
                                            reload = ui.button(tr("app.099")).clicked();
                                            if ui.button(tr("app.098")).clicked() {
                                                let result =
                                                    open_file(&mindglider::i18n::directory());
                                                self.result(result);
                                            }
                                        });
                                        ui.label(tr("app.097"));
                                    }
                                    1 => {
                                        ui.heading(tr("ui.cli_heading"));
                                        egui::Grid::new("cli-preferences")
                                            .num_columns(3)
                                            .spacing([16., 10.])
                                            .show(ui, |ui| {
                                                ui.label(tr("app.096"));
                                                egui::ComboBox::from_id_salt("cli-provider")
                                                    .icon(|ui, rect, visuals, _, _| {
                                                        crate::theme::chevron(
                                                            ui,
                                                            rect.center(),
                                                            0.,
                                                            visuals.fg_stroke.color,
                                                        )
                                                    })
                                                    .selected_text(
                                                        match self.settings.cli.provider {
                                                            Provider::Codex => tr("provider.codex"),
                                                            Provider::Claude => {
                                                                tr("provider.claude")
                                                            }
                                                        },
                                                    )
                                                    .show_ui(ui, |ui| {
                                                        ui.selectable_value(
                                                            &mut self.settings.cli.provider,
                                                            Provider::Codex,
                                                            tr("provider.codex"),
                                                        );
                                                        ui.selectable_value(
                                                            &mut self.settings.cli.provider,
                                                            Provider::Claude,
                                                            tr("provider.claude"),
                                                        );
                                                    });
                                                if reset_button(ui) {
                                                    self.settings.cli.provider =
                                                        defaults.cli.provider;
                                                }
                                                ui.end_row();
                                                let codex =
                                                    self.settings.cli.provider == Provider::Codex;
                                                ui.label(if codex {
                                                    tr("app.095")
                                                } else {
                                                    tr("app.094")
                                                });
                                                let path = if codex {
                                                    &mut self.settings.cli.codex
                                                } else {
                                                    &mut self.settings.cli.claude
                                                };
                                                ui.add_sized(
                                                    [320., 32.],
                                                    egui::TextEdit::singleline(path),
                                                );
                                                if reset_button(ui) {
                                                    *path = if codex {
                                                        defaults.cli.codex.clone()
                                                    } else {
                                                        defaults.cli.claude.clone()
                                                    };
                                                }
                                                ui.end_row();
                                                ui.label(tr("app.093"));
                                                let model = if codex {
                                                    &mut self.settings.cli.codex_options.model
                                                } else {
                                                    &mut self.settings.cli.claude_options.model
                                                };
                                                editable_combo(
                                                    ui,
                                                    "cli-model",
                                                    model,
                                                    &models,
                                                    320.,
                                                );
                                                if reset_button(ui) {
                                                    model.clear();
                                                }
                                                ui.end_row();
                                                ui.label(tr("cli.effort"));
                                                let effort = if codex {
                                                    &mut self.settings.cli.codex_options.effort
                                                } else {
                                                    &mut self.settings.cli.claude_options.effort
                                                };
                                                editable_combo(
                                                    ui,
                                                    "cli-effort",
                                                    effort,
                                                    &efforts,
                                                    320.,
                                                );
                                                if reset_button(ui) {
                                                    effort.clear();
                                                }
                                                ui.end_row();
                                                if codex {
                                                    let opt = &mut self.settings.cli.codex_options;
                                                    preference_text(
                                                        ui,
                                                        tr("cli.service_tier"),
                                                        &mut opt.service_tier,
                                                    );
                                                    preference_check(
                                                        ui,
                                                        tr("cli.web_search"),
                                                        &mut opt.web_search,
                                                    );
                                                    preference_check(
                                                        ui,
                                                        tr("cli.oss"),
                                                        &mut opt.oss,
                                                    );
                                                    if opt.oss {
                                                        preference_text(
                                                            ui,
                                                            tr("cli.local_provider"),
                                                            &mut opt.local_provider,
                                                        );
                                                    }
                                                } else {
                                                    let opt = &mut self.settings.cli.claude_options;
                                                    preference_text(
                                                        ui,
                                                        tr("cli.fallback"),
                                                        &mut opt.fallback_model,
                                                    );
                                                    preference_text(
                                                        ui,
                                                        tr("cli.budget"),
                                                        &mut opt.max_budget_usd,
                                                    );
                                                    preference_text(
                                                        ui,
                                                        tr("cli.add_dirs"),
                                                        &mut opt.add_dirs,
                                                    );
                                                    preference_check(
                                                        ui,
                                                        tr("cli.disable_skills"),
                                                        &mut opt.disable_slash_commands,
                                                    );
                                                }
                                                ui.label(tr("app.092"));
                                                let timeout_id = egui::Id::new("cli-timeout-draft");
                                                let mut timeout = ui
                                                    .ctx()
                                                    .data_mut(|d| d.get_temp::<String>(timeout_id))
                                                    .unwrap_or_else(|| {
                                                        self.settings.cli.timeout_secs.to_string()
                                                    });
                                                if ui
                                                    .add_sized(
                                                        [320., 32.],
                                                        egui::TextEdit::singleline(&mut timeout),
                                                    )
                                                    .changed()
                                                {
                                                    if let Ok(value) = timeout.parse::<u64>()
                                                        && (1..=3600).contains(&value)
                                                    {
                                                        self.settings.cli.timeout_secs = value;
                                                    }
                                                    ui.ctx().data_mut(|d| {
                                                        d.insert_temp(timeout_id, timeout.clone())
                                                    });
                                                }
                                                valid_timeout = timeout
                                                    .parse::<u64>()
                                                    .is_ok_and(|v| (1..=3600).contains(&v));
                                                if reset_button(ui) {
                                                    self.settings.cli.timeout_secs =
                                                        defaults.cli.timeout_secs;
                                                    ui.ctx().data_mut(|d| {
                                                        d.remove::<String>(egui::Id::new(
                                                            "cli-timeout-draft",
                                                        ))
                                                    });
                                                }
                                                ui.end_row();
                                            });
                                        ui.label(tr("cli.discovery_note"));
                                        ui.label(msg(
                                            "cli.source",
                                            &[("source", self.catalog.source.clone())],
                                        ));
                                        if !self.catalog.configured_model.is_empty() {
                                            ui.label(msg(
                                                "cli.configured",
                                                &[
                                                    (
                                                        "model",
                                                        self.catalog.configured_model.clone(),
                                                    ),
                                                    (
                                                        "effort",
                                                        self.catalog.configured_effort.clone(),
                                                    ),
                                                ],
                                            ));
                                        }
                                        if !self.catalog.warning.is_empty() {
                                            ui.colored_label(
                                                Color32::DARK_RED,
                                                &self.catalog.warning,
                                            );
                                        }
                                        if self.catalog_task.is_some() {
                                            ui.horizontal(|ui| {
                                                ui.spinner();
                                                ui.label(tr("cli.loading"));
                                            });
                                        }
                                        if ui
                                            .add_enabled(
                                                self.catalog_task.is_none(),
                                                egui::Button::new(tr("cli.refresh")),
                                            )
                                            .clicked()
                                        {
                                            self.refresh_catalog(ctx);
                                        }
                                        ui.label(tr("cli.path_detection"));
                                        ui.horizontal(|ui| {
                                            ui.label(tr("app.091"));
                                            if reset_button(ui) {
                                                self.settings.cli.system_prompt =
                                                    ai::default_system_prompt_for(
                                                        &self.settings.language,
                                                    );
                                            }
                                        });
                                        ui.add(
                                            egui::TextEdit::multiline(
                                                &mut self.settings.cli.system_prompt,
                                            )
                                            .desired_width(f32::INFINITY)
                                            .desired_rows(9),
                                        );
                                        ui.label(tr("app.090"));
                                        ui.label(tr("app.089"));
                                    }
                                    2 => {
                                        ui.heading(tr("app.088"));
                                        egui::Grid::new("page-preferences")
                                            .num_columns(3)
                                            .spacing([16., 10.])
                                            .show(ui, |ui| {
                                                ui.label(tr("app.087"));
                                                ui.add(
                                                    egui::DragValue::new(
                                                        &mut self.settings.page.width_mm,
                                                    )
                                                    .range(50.0..=1200.0),
                                                );
                                                if reset_button(ui) {
                                                    self.settings.page.width_mm =
                                                        defaults.page.width_mm;
                                                }
                                                ui.end_row();
                                                ui.label(tr("app.086"));
                                                ui.add(
                                                    egui::DragValue::new(
                                                        &mut self.settings.page.height_mm,
                                                    )
                                                    .range(50.0..=1200.0),
                                                );
                                                if reset_button(ui) {
                                                    self.settings.page.height_mm =
                                                        defaults.page.height_mm;
                                                }
                                                ui.end_row();
                                                ui.label(tr("app.085"));
                                                ui.add(
                                                    egui::DragValue::new(
                                                        &mut self.settings.page.margin_mm,
                                                    )
                                                    .range(0.0..=100.0),
                                                );
                                                if reset_button(ui) {
                                                    self.settings.page.margin_mm =
                                                        defaults.page.margin_mm;
                                                }
                                                ui.end_row();
                                                ui.label(tr("app.084"));
                                                ui.checkbox(
                                                    &mut self.settings.page.tile,
                                                    tr("app.083"),
                                                );
                                                if reset_button(ui) {
                                                    self.settings.page.tile = defaults.page.tile;
                                                }
                                                ui.end_row();
                                            });
                                    }
                                    3 => {
                                        ui.heading(tr("app.082"));
                                        for (id, label, key) in commands() {
                                            ui.push_id(id, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.allocate_ui_with_layout(
                                                        egui::vec2(220., 32.),
                                                        egui::Layout::left_to_right(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            ui.set_min_size(egui::vec2(220., 32.));
                                                            ui.add(egui::Label::new(label).wrap());
                                                        },
                                                    );
                                                    let value = self
                                                        .settings
                                                        .shortcuts
                                                        .entry(id.into())
                                                        .or_default();
                                                    let output = shortcut_input(ui, id, value);
                                                    output.response.on_hover_text(value.as_str());
                                                    if reset_button(ui) {
                                                        *value = key.into();
                                                    }
                                                    if !value.is_empty()
                                                        && shortcut(value).is_none()
                                                    {
                                                        ui.colored_label(
                                                            Color32::RED,
                                                            tr("app.081"),
                                                        );
                                                    }
                                                });
                                            });
                                        }
                                    }
                                    _ => {}
                                }
                            });
                    });
                ui.add_space(16.);
                ui.horizontal(|ui| {
                    reset_all = ui.button(tr("app.079")).clicked();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        save = crate::theme::accent_button(ui, tr("app.080")).clicked();
                    });
                });
            });
        self.settings_open = open;
        if reset_all {
            self.settings.reset_preferences();
            ctx.data_mut(|d| d.remove::<String>(egui::Id::new("cli-timeout-draft")));
            self.status = tr("app.078").into();
        }
        if old_language != self.settings.language || reload {
            self.catalog_key.clear();
            self.settings.cli.set_language(&self.settings.language);
            match mindglider::i18n::reload(&self.settings.language) {
                Ok(()) => self.status = tr("app.077").into(),
                Err(e) => {
                    self.settings.language = old_language;
                    self.status = msg("app.037", &[("e", format!("{e}"))]);
                }
            }
        }
        if save && !valid_timeout {
            self.status = tr("cli.timeout_invalid").into();
        } else if save {
            let result = ai::optional_args(&self.settings.cli).and_then(|_| self.save_session());
            self.result(result);
        }
    }
    fn ai_dialog(&mut self, ctx: &egui::Context) {
        if self.ai_open
            && !ctx.memory(|m| m.is_popup_open(combo_popup_id("ai-instruction")))
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            if let Some(task) = &self.ai_task {
                task.cancel.store(true, Ordering::Relaxed);
            }
            self.ai_open = false;
            self.ai_was_open = false;
            return;
        }
        if let Some(task) = &self.ai_task {
            match task.receiver.try_recv() {
                Ok(result) => {
                    let tab_id = task.tab_id.clone();
                    let snapshot = task.snapshot.clone();
                    let cancelled = task.cancel.load(Ordering::Relaxed);
                    self.ai_task = None;
                    match result {
                        Ok(_) if cancelled => self.status = tr("ai.024").into(),
                        Ok(plan) => {
                            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
                                match ai::apply(&mut tab.history, &snapshot, &plan) {
                                    Ok(n) => {
                                        tab.normalize_selection();
                                        self.status = msg("app.036", &[("n", format!("{n}"))]);
                                    }
                                    Err(e) => {
                                        self.status = msg("app.035", &[("e", format!("{e:#}"))])
                                    }
                                }
                            } else {
                                self.status = tr("app.076").into();
                            }
                        }
                        Err(e) => self.status = msg("app.034", &[("e", e.to_string())]),
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.ai_task = None;
                    self.status = tr("app.075").into();
                }
                Err(_) => {}
            }
        }
        if !self.ai_open {
            self.ai_was_open = false;
            return;
        }
        if !self.ai_was_open {
            match mindglider::prompt_history::load(&self.data.join("ai-history.json")) {
                Ok(items) => {
                    self.ai_history = items;
                    self.ai_prompt = self.ai_history.first().cloned().unwrap_or_default();
                }
                Err(e) => self.status = e.to_string(),
            }
            self.ai_focus = true;
            self.ai_was_open = true;
        }
        let combo_open = ctx.memory(|m| m.is_popup_open(combo_popup_id("ai-instruction")));
        let enter = !combo_open && ctx.input_mut(|i| {
            let trigger=i.events.iter().any(|e|matches!(e,egui::Event::Key{key:egui::Key::Enter,pressed:true,repeat:false,modifiers,..} if modifiers.is_none()));
            i.consume_key(egui::Modifiers::NONE,egui::Key::Enter);
            trigger
        });
        let mut open = true;
        let run = enter && self.ai_task.is_none() && !self.ai_prompt.trim().is_empty();
        crate::theme::window(ctx, tr("app.074"), "app.074")
            .default_width(530.)
            .show(ctx, |ui| {
                crate::theme::dialog_header(ui, tr("app.074"), &mut open);
                let t = &self.tabs[self.active];
                ui.label(msg(
                    "app.033",
                    &[
                        ("0", t.history.doc.nodes[&t.primary].text.text().to_string()),
                        ("1", format!("{}", t.history.doc.subtree(&t.primary).len())),
                    ],
                ));
                ui.label(msg(
                    "app.032",
                    &[(
                        "0",
                        match self.settings.cli.provider {
                            Provider::Codex => tr("provider.codex").into(),
                            Provider::Claude => tr("provider.claude").into(),
                        },
                    )],
                ));
                let items: Vec<_> = self
                    .ai_history
                    .iter()
                    .map(|text| (text.clone(), text.clone()))
                    .collect();
                let response = editable_combo(
                    ui,
                    "ai-instruction",
                    &mut self.ai_prompt,
                    &items,
                    ui.available_width(),
                );
                if self.ai_focus {
                    response.request_focus();
                    self.ai_focus = false;
                }
                ui.label(tr("ai.keyboard_hint"));
                ui.label(tr("app.072"));
                ui.label(tr("app.071"));
                if let Some(task) = &self.ai_task {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(msg(
                            "app.031",
                            &[("0", task.started.elapsed().as_secs().to_string())],
                        ));
                    });
                }
            });
        self.ai_open = open;
        if !open {
            self.ai_was_open = false;
        }
        if run && open {
            let t = &self.tabs[self.active];
            match ai::Snapshot::capture(&t.history.doc, &t.primary, &self.ai_prompt) {
                Ok(snapshot) => {
                    if let Err(e) = ai::optional_args(&self.settings.cli) {
                        self.status = e.to_string();
                        return;
                    }
                    match mindglider::prompt_history::record(
                        &self.data.join("ai-history.json"),
                        &self.ai_prompt,
                    ) {
                        Ok(items) => self.ai_history = items,
                        Err(e) => {
                            self.status = e.to_string();
                            return;
                        }
                    }
                    self.ai_task = Some(ai::spawn(
                        self.settings.cli.clone(),
                        snapshot,
                        t.id.clone(),
                        &self.data,
                        ctx.clone(),
                    ));
                    self.status = tr("app.069").into();
                }
                Err(e) => self.status = e.to_string(),
            }
        }
    }
    fn export_scene(&self) -> Scene {
        let t = &self.tabs[self.active];
        let root = if self.export_branch {
            &t.primary
        } else {
            &t.history.doc.root
        };
        let mut doc = t.history.doc.clone();
        for n in doc.nodes.values_mut() {
            n.folded = false;
        }
        let mut layout = LayoutCache::default();
        let mut scene = layout.build(&doc, None, |s, size| {
            egui::vec2(
                s.lines()
                    .map(|l| {
                        l.chars()
                            .map(|c| if c.is_ascii() { size * 0.56 } else { size })
                            .sum::<f32>()
                    })
                    .fold(0., f32::max),
                s.lines().count().max(1) as f32 * size * 1.3,
            )
        });
        if self.export_branch {
            let ids = doc.subtree(root);
            scene.nodes.retain(|p| ids.contains(&p.id));
            scene
                .edges
                .retain(|(a, b)| ids.contains(a) && ids.contains(b));
            scene
                .clouds
                .retain(|(r, _)| scene.nodes.iter().any(|n| r.contains_rect(n.rect)));
            scene.bounds = scene
                .nodes
                .iter()
                .map(|p| p.rect)
                .reduce(Rect::union)
                .unwrap_or(Rect::ZERO)
                .expand(30.);
        }
        scene
    }
    fn export_dialog(&mut self, ctx: &egui::Context) {
        if !self.export_open {
            return;
        }
        let mut open = true;
        let mut save = false;
        let mut preview = false;
        let mut print = false;
        crate::theme::window(ctx, tr("app.068"), "app.068")
            .default_width(560.)
            .show(ctx, |ui| {
                crate::theme::dialog_header(ui, tr("app.068"), &mut open);
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.export_branch, false, tr("app.067"));
                    ui.selectable_value(&mut self.export_branch, true, tr("app.066"));
                });
                egui::ComboBox::from_id_salt("export-format")
                    .icon(|ui, rect, visuals, _, _| {
                        crate::theme::chevron(ui, rect.center(), 0., visuals.fg_stroke.color)
                    })
                    .selected_text(self.export_format.to_uppercase())
                    .show_ui(ui, |ui| {
                        for f in ["txt", "md", "opml", "png", "jpg", "svg", "pdf", "html"] {
                            ui.selectable_value(
                                &mut self.export_format,
                                f.into(),
                                f.to_uppercase(),
                            );
                        }
                    });
                if self.export_format == "pdf" {
                    ui.horizontal(|ui| {
                        ui.label(tr("app.065"));
                        if ui.button(tr("paper.a4")).clicked() {
                            self.settings.page.width_mm = 210.;
                            self.settings.page.height_mm = 297.;
                        }
                        if ui.button(tr("paper.a3")).clicked() {
                            self.settings.page.width_mm = 297.;
                            self.settings.page.height_mm = 420.;
                        }
                        if ui.button(tr("app.064")).clicked() {
                            std::mem::swap(
                                &mut self.settings.page.width_mm,
                                &mut self.settings.page.height_mm,
                            );
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.settings.page.width_mm)
                                .range(50.0..=1200.0)
                                .prefix(tr("app.063"))
                                .suffix("mm"),
                        );
                        ui.add(
                            egui::DragValue::new(&mut self.settings.page.height_mm)
                                .range(50.0..=1200.0)
                                .prefix(tr("app.062"))
                                .suffix("mm"),
                        );
                        ui.add(
                            egui::DragValue::new(&mut self.settings.page.margin_mm)
                                .range(0.0..=100.0)
                                .prefix(tr("app.061"))
                                .suffix("mm"),
                        );
                    });
                    ui.checkbox(&mut self.settings.page.tile, tr("app.060"));
                    ui.horizontal(|ui| {
                        preview = ui.button(tr("app.059")).clicked();
                        print = ui.button(tr("app.058")).clicked();
                        ui.add(
                            egui::DragValue::new(&mut self.preview_page)
                                .range(0..=self.preview_count.saturating_sub(1))
                                .prefix(tr("app.057")),
                        );
                    });
                    if let Some(texture) = &self.preview {
                        ui.add(egui::Image::new(texture).max_size(egui::vec2(420., 400.)));
                    }
                    ui.label(msg("app.030", &[("0", format!("{}", self.preview_count))]));
                }
                save = ui.button(tr("app.056")).clicked();
            });
        self.export_open = open;
        if preview {
            let result = (|| -> Result<()> {
                let scene = self.export_scene();
                let views = export::page_views(&scene, &self.settings.page)?;
                self.preview_count = views.len();
                let (view, size) = views[self.preview_page.min(views.len() - 1)];
                let t = &self.tabs[self.active];
                let svg = export::page_svg(
                    &t.history.doc,
                    &scene,
                    t.path.as_deref(),
                    view,
                    size,
                    self.settings.page.margin_mm,
                )?;
                let png = export::raster(&svg, "png")?;
                let image = image::load_from_memory(&png)?.to_rgba8();
                let color = egui::ColorImage::from_rgba_unmultiplied(
                    [image.width() as usize, image.height() as usize],
                    image.as_raw(),
                );
                self.preview =
                    Some(ctx.load_texture("print-preview", color, egui::TextureOptions::LINEAR));
                Ok(())
            })();
            self.result(result);
        }
        if save || print {
            let path = if print {
                Some(self.data.join("print-preview.pdf"))
            } else {
                rfd::FileDialog::new()
                    .add_filter(&self.export_format, &[self.export_format.as_str()])
                    .set_file_name(format!("Mindmap.{}", self.export_format))
                    .save_file()
            };
            if let Some(path) = path {
                let result = (|| -> Result<()> {
                    let t = &self.tabs[self.active];
                    let root = if self.export_branch {
                        &t.primary
                    } else {
                        &t.history.doc.root
                    };
                    let f = if print { "pdf" } else { &self.export_format };
                    let bytes = match f {
                        "txt" | "md" | "opml" | "html" => {
                            export::outline(&t.history.doc, root, f, t.path.as_deref())?
                                .into_bytes()
                        }
                        _ => {
                            let scene = self.export_scene();
                            match f {
                                "svg" => export::svg(&t.history.doc, &scene, t.path.as_deref())?
                                    .into_bytes(),
                                "pdf" => export::pdf(
                                    &t.history.doc,
                                    &scene,
                                    t.path.as_deref(),
                                    &self.settings.page,
                                )?,
                                _ => export::raster(
                                    &export::svg(&t.history.doc, &scene, t.path.as_deref())?,
                                    f,
                                )?,
                            }
                        }
                    };
                    storage::atomic_write(&path, &bytes)?;
                    if print {
                        open_file(&path)?;
                    }
                    self.status = msg("app.029", &[("0", format!("{}", path.display()))]);
                    Ok(())
                })();
                self.result(result);
            }
        }
    }
    fn close_dialog(&mut self, ctx: &egui::Context) {
        let Some(closing) = self.closing else {
            return;
        };
        let indices: Vec<_> = match closing {
            Closing::Tab(i) => vec![i],
            Closing::Quit => (0..self.tabs.len()).collect(),
        };
        let dirty: Vec<_> = indices
            .iter()
            .copied()
            .filter(|i| self.tabs[*i].history.dirty)
            .collect();
        let mut choice = 0;
        if dirty.is_empty() {
            choice = 2;
        } else {
            crate::theme::window(ctx, tr("app.055"), "app.055").show(ctx, |ui| {
                ui.label(egui::RichText::new(tr("app.055")).size(24.));
                ui.add_space(16.);
                ui.label(msg("app.028", &[("0", format!("{}", dirty.len()))]));
                ui.horizontal(|ui| {
                    if ui.button(tr("app.054")).clicked() {
                        choice = 1;
                    }
                    if ui.button(tr("app.053")).clicked() {
                        choice = 2;
                    }
                    if ui.button(tr("app.052")).clicked() {
                        choice = 3;
                    }
                });
            });
        }
        if choice == 1 {
            for i in dirty {
                match self.save_tab(i, false) {
                    Ok(true) => {}
                    Ok(false) => {
                        self.closing = None;
                        return;
                    }
                    Err(e) => {
                        self.status = e.to_string();
                        self.closing = None;
                        return;
                    }
                }
            }
            choice = 2;
        }
        if choice == 3 {
            self.closing = None;
            return;
        }
        if choice == 2 {
            self.closing = None;
            if let Some(task) = &self.ai_task
                && (matches!(closing, Closing::Quit)
                    || matches!(closing,Closing::Tab(i) if self.tabs[i].id==task.tab_id))
            {
                task.cancel.store(true, Ordering::Relaxed);
            }
            match closing {
                Closing::Tab(i) => {
                    let removed = self.tabs.remove(i);
                    let _ =
                        std::fs::remove_file(self.data.join(format!("recovery-{}.mm", removed.id)));
                    if self.tabs.is_empty() {
                        self.tabs.push(Tab::new(Document::default(), None));
                    }
                    self.active = self.active.min(self.tabs.len() - 1);
                }
                Closing::Quit => {
                    for i in indices {
                        let t = &mut self.tabs[i];
                        let _ =
                            std::fs::remove_file(self.data.join(format!("recovery-{}.mm", t.id)));
                        t.history.dirty = false;
                    }
                    self.settings.tabs = self
                        .tabs
                        .iter()
                        .filter(|t| t.path.is_some())
                        .map(|t| SavedTab {
                            id: t.id.clone(),
                            path: t.path.clone(),
                            recovery: false,
                            zoom: t.zoom,
                            pan: [t.pan.x, t.pan.y],
                        })
                        .collect();
                    let _ = storage::atomic_write(
                        &self.data.join("settings.json"),
                        &serde_json::to_vec_pretty(&self.settings).unwrap_or_default(),
                    );
                    self.allow_exit = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }
}
impl eframe::App for MindGlider {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, input: &mut egui::RawInput) {
        // Deterministic pointer gestures for --smoke-view drag-* verification only.
        let Some(mode) = self
            .smoke_drag_mode
            .clone()
            .filter(|_| self.smoke.is_some())
        else {
            return;
        };
        if self.frame_count < 5 {
            return;
        }
        let tab = &self.tabs[self.active];
        let root = &tab.history.doc.nodes[&tab.history.doc.root];
        if self.smoke_drag_source.is_none() {
            self.smoke_drag_source = root.children.first().cloned();
        }
        let Some(source) = self
            .smoke_drag_source
            .as_ref()
            .and_then(|id| tab.layout.scene.nodes.iter().find(|p| &p.id == id))
        else {
            return;
        };
        let point = |world: Pos2| tab.canvas_rect.center() + tab.pan + world.to_vec2() * tab.zoom;
        let target_id = if mode.contains("invalid") {
            tab.history.doc.nodes[&source.id]
                .children
                .first()
                .unwrap_or(&root.id)
        } else if mode.contains("sibling") || mode.contains("child-left") {
            root.children
                .iter()
                .find(|id| *id != &source.id)
                .unwrap_or(&root.id)
        } else {
            &root.id
        };
        let Some(target) = tab.layout.scene.nodes.iter().find(|p| &p.id == target_id) else {
            return;
        };
        let pos = if self.frame_count == 5 {
            point(source.rect.center())
        } else if mode.contains("sibling") {
            point(egui::pos2(target.rect.center().x, target.rect.top() + 1.))
        } else {
            point(egui::pos2(
                target.rect.left()
                    + target.rect.width() * if mode.contains("left") { 0.12 } else { 0.88 },
                target.rect.center().y,
            ))
        };
        input.focused = true;
        if mode.ends_with("cancel") && self.frame_count == 12 {
            input.events.push(egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            });
        }

        input.events.push(egui::Event::PointerMoved(pos));
        if self.frame_count == 5
            || (mode.ends_with("drop") && self.frame_count == 12)
            || (mode.ends_with("cancel") && self.frame_count == 13)
        {
            input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: self.frame_count == 5,
                modifiers: egui::Modifiers::NONE,
            });
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;
        if self.first_frame
            && let Some(scale) = self.requested_scale
        {
            ctx.set_pixels_per_point(scale);
        }
        ctx.input_mut(|i| {
            if !i.focused {
                self.ime_guard.reset();
            } else {
                self.ime_guard.filter(&mut i.events, i.time);
            }
        });
        if ctx.input(|i| i.viewport().close_requested())
            && self.closing.is_none()
            && !self.allow_exit
            && self.smoke.is_none()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.closing = Some(Closing::Quit);
        }
        self.keyboard(ctx);
        let committed = self.settings.theme.unwrap_or_default();
        let preview = self
            .menu
            .theme_item
            .filter(|_| self.menu.active && self.menu.open && self.menu.selected == 2)
            .and_then(|_| {
                ctx.input(|i| {
                    i.pointer.hover_pos().filter(|_| i.focused).and_then(|pos| {
                        self.menu
                            .theme_rows
                            .iter()
                            .find(|(_, rect)| rect.contains(pos))
                            .map(|(theme, _)| *theme)
                    })
                })
            });
        theme::apply(ctx, preview.unwrap_or(committed));
        self.toolbar(ctx);
        egui::TopBottomPanel::bottom("status")
            .exact_height(24.)
            .frame(
                egui::Frame::NONE
                    .fill(theme::current(ctx).background)
                    .inner_margin(egui::Margin::symmetric(12, 2)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().interact_size.y = 18.;
                ui.horizontal_centered(|ui| {
                    ui.set_min_height(18.);
                    let t = &self.tabs[self.active];
                    let count = msg(
                        "app.027",
                        &[
                            ("0", format!("{}", t.history.doc.nodes.len())),
                            ("1", format!("{}", t.selection.len())),
                        ],
                    );
                    let right = ui.fonts(|f| {
                        f.layout_no_wrap(count.clone(), FontId::proportional(13.), Color32::GRAY)
                            .size()
                            .x
                    }) + 24.;
                    ui.allocate_ui_with_layout(
                        egui::vec2((ui.available_width() - right).max(20.), 18.),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.set_min_size(ui.available_size());
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&self.status)
                                        .size(12.)
                                        .color(theme::current(ctx).secondary),
                                )
                                .truncate(),
                            )
                        },
                    )
                    .inner
                    .on_hover_text(&self.status);
                    ui.separator();
                    ui.label(
                        egui::RichText::new(count)
                            .size(12.)
                            .color(theme::current(ctx).secondary),
                    );
                });
            });
        self.file_tabs(ctx);
        self.icon_sidebar(ctx);
        self.inspector(ctx);
        self.canvas(ctx);
        self.dialogs(ctx);
        if self.first_frame {
            self.first_frame = false;
            let elapsed = self.started.elapsed().as_secs_f64() * 1000.;
            let _=storage::atomic_write(&self.data.join("startup.json"),&serde_json::to_vec_pretty(&serde_json::json!({"first_ui_ms":elapsed,"nodes":self.tabs[self.active].history.doc.nodes.len(),"renderer":"wgpu"})).unwrap_or_default());
        }
        if self.last_autosave.elapsed() > Duration::from_secs(30) {
            let r = self.save_session();
            self.result(r);
            self.last_autosave = Instant::now();
        }
        ctx.request_repaint_after(
            Duration::from_secs(30).saturating_sub(self.last_autosave.elapsed()),
        );
        if self.ai_task.is_some() {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
        if let Some(path) = self.smoke.clone() {
            if self.smoke_drag_mode.is_some() && self.frame_count < 18 {
                ctx.request_repaint();
                return;
            }
            if !self.smoke_requested
                && self.frame_count >= 3
                && self.smoke_started.elapsed() >= Duration::from_millis(350)
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                self.smoke_requested = true;
            }
            for event in ctx.input(|i| i.events.clone()) {
                if let egui::Event::Screenshot { image, .. } = event {
                    if self.smoke_drag_mode.is_some() {
                        let tab = &self.tabs[self.active];
                        let _ = storage::atomic_write(
                            &path.with_extension("mm"),
                            &mm::write(&tab.history.doc).unwrap(),
                        );
                        let _=storage::atomic_write(&path.with_extension("state.json"),&serde_json::to_vec_pretty(&serde_json::json!({"dragging":tab.drag.is_some(),"source":self.smoke_drag_source,"nodes":tab.history.doc.nodes.len()})).unwrap());
                    }

                    let pixels: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
                    let _ = image::save_buffer(
                        &path,
                        &pixels,
                        image.width() as u32,
                        image.height() as u32,
                        image::ColorType::Rgba8,
                    );
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}
fn search_match(text: &str, query: &str, case: bool) -> bool {
    !query.is_empty()
        && if case {
            text.contains(query)
        } else {
            text.to_lowercase().contains(&query.to_lowercase())
        }
}
fn shortcut(text: &str) -> Option<egui::KeyboardShortcut> {
    let mut modifiers = egui::Modifiers::NONE;
    let mut key = None;
    for part in text.split('+') {
        match part.trim() {
            "Ctrl" => modifiers.ctrl = true,
            "Shift" => modifiers.shift = true,
            "Alt" => modifiers.alt = true,
            name => key = egui::Key::from_name(name),
        }
    }
    Some(egui::KeyboardShortcut::new(modifiers, key?))
}
fn open_file(path: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = std::process::Command::new("explorer.exe");
        cmd.arg(path).creation_flags(0x08000000).spawn()?;
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    }
    Ok(())
}

fn menu_commands(index: usize) -> &'static [&'static str] {
    match index {
        0 => &["new", "open", "save", "save_as", "export", "close"],
        1 => &[
            "undo", "redo", "copy", "cut", "paste", "edit", "icons", "child", "sibling", "parent",
            "delete", "side",
        ],
        2 => &[
            "fit",
            "find",
            "filter",
            "styles",
            "icon_panel",
            "inspector",
            "theme",
        ],
        3 => &["ai", "palette", "settings"],
        _ => &["about"],
    }
}
fn command_label(id: &str) -> &'static str {
    commands()
        .into_iter()
        .find(|c| c.0 == id)
        .map(|c| c.1)
        .unwrap_or(tr("app.106"))
}

fn reset_button(ui: &mut egui::Ui) -> bool {
    ui.button(tr("ui.reset"))
        .on_hover_text(tr("ui.reset_tip"))
        .clicked()
}
impl Settings {
    fn migrate(&mut self) {
        self.theme.get_or_insert(if self.dark {
            Theme::SandNight
        } else {
            Theme::Sand
        });
        self.shortcuts
            .entry("icons".into())
            .or_insert_with(|| "Ctrl+F2".into());
        if self.version < 2 {
            if self
                .shortcuts
                .get("ai")
                .is_some_and(|s| s == "Ctrl+Shift+A")
            {
                self.shortcuts.insert("ai".into(), "F12".into());
            }
            self.version = 2;
        }
    }
    fn reset_preferences(&mut self) {
        let defaults = Self::default();
        self.cli = defaults.cli;
        self.shortcuts = defaults.shortcuts;
        self.page = defaults.page;
        self.icon_panel_open = defaults.icon_panel_open;
        self.inspector_open = defaults.inspector_open;
        self.language = defaults.language;
        self.version = defaults.version;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_default_shortcuts_parse() {
        for (id, _, key) in commands() {
            if !key.is_empty() {
                assert!(shortcut(key).is_some(), "{id}: {key}");
            }
        }
    }
    #[test]
    fn button_focus_does_not_block_map_keyboard() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.button("button").request_focus();
            });
        });
        let id = ctx.memory(|m| m.focused()).unwrap();
        assert!(egui::text_edit::TextEditState::load(&ctx, id).is_none());
        let mut text = "test".to_owned();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.text_edit_singleline(&mut text).request_focus();
            });
        });
        let id = ctx.memory(|m| m.focused()).unwrap();
        assert!(egui::text_edit::TextEditState::load(&ctx, id).is_some());
    }
}

#[cfg(test)]
mod preference_tests {
    use super::*;
    #[test]
    fn f12_is_the_default_and_legacy_binding_is_migrated() {
        assert_eq!(Settings::default().shortcuts["ai"], "F12");
        let mut old: Settings = serde_json::from_str(
            r#"{"shortcuts":{"ai":"Ctrl+Shift+A","new":"Alt+N"},"cli":{"provider":"Claude"}}"#,
        )
        .unwrap();
        assert_eq!(old.version, 0);
        old.migrate();
        assert_eq!(old.shortcuts["ai"], "F12");
        assert_eq!(old.shortcuts["new"], "Alt+N");
        assert!(!old.cli.system_prompt.is_empty());
        assert_eq!(old.cli.provider, Provider::Claude);
    }
    #[test]
    fn reset_all_preserves_maps_and_open_tabs() {
        let mut settings = Settings {
            language: "en".into(),
            dark: true,
            ..Default::default()
        };
        settings.cli.system_prompt = "custom".into();
        settings.tabs.push(SavedTab {
            id: "keep".into(),
            path: Some(PathBuf::from("map.mm")),
            recovery: true,
            zoom: 1.2,
            pan: [10., 20.],
        });
        settings.reset_preferences();
        assert_eq!(settings.tabs[0].id, "keep");
        assert!(settings.tabs[0].recovery);
        assert_eq!(settings.language, "ja");
        assert_eq!(settings.cli.system_prompt, ai::default_system_prompt());
        assert_eq!(settings.shortcuts["ai"], "F12");
    }
    #[test]
    fn an_explicit_custom_ai_shortcut_is_not_overwritten() {
        let mut settings = Settings {
            version: 0,
            ..Default::default()
        };
        settings.shortcuts.insert("ai".into(), "Ctrl+F12".into());
        settings.migrate();
        assert_eq!(settings.shortcuts["ai"], "Ctrl+F12");
    }
}
#[cfg(test)]
mod panel_preference_tests {
    use super::*;
    #[test]
    fn panel_visibility_defaults_and_last_state_survive_restart() {
        let defaults: Settings = serde_json::from_str("{}").unwrap();
        assert!(!defaults.icon_panel_open);
        assert!(defaults.inspector_open);
        let mut settings = Settings {
            icon_panel_open: true,
            inspector_open: false,
            ..Default::default()
        };
        settings.migrate();
        let json = serde_json::to_vec(&settings).unwrap();
        let restored: Settings = serde_json::from_slice(&json).unwrap();
        assert!(restored.icon_panel_open);
        assert!(!restored.inspector_open);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_map_node(
    ui: &mut egui::Ui,
    n: &Node,
    p: &mindglider::layout::Placed,
    r: Rect,
    zoom: f32,
    border: Stroke,
    base: Option<&Path>,
    show_text: bool,
) {
    let ctx = ui.ctx().clone();
    let painter = ui.painter().clone();
    let pad = mindglider::layout::padding(&p.style);
    paint_node(&painter, r, &p.style, border);
    if p.style.shape == "fork" {
        let core = mindglider::layout::core_text_rect(n, p);
        let local = Rect::from_min_size(r.min + (core.min - p.rect.min) * zoom, core.size() * zoom);
        painter.rect_filled(local, 0., color(&p.style.background));
    }
    let visual_icons: Vec<_> = n
        .icons
        .iter()
        .cloned()
        .chain(n.progress.map(|p| format!("{p}%")))
        .collect();
    for (index, icon) in visual_icons.iter().enumerate() {
        paint_icon(
            ui,
            icon,
            Rect::from_min_size(
                r.min + egui::vec2(pad.x + index as f32 * 18., pad.y) * zoom,
                egui::vec2(16., 16.) * zoom,
            ),
        );
    }
    if show_text {
        let galley = ctx.fonts(|f| f.layout_job(text_job(&n.text, &p.style, zoom)));
        painter.galley(
            r.min
                + egui::vec2(
                    pad.x + mindglider::layout::icon_width(mindglider::layout::icon_count(n)),
                    pad.y,
                ) * zoom,
            galley,
            color(&p.style.color),
        );
    }
    if let Some(img) = &n.image {
        let path = storage::resolve(base, &img.uri);
        let path = std::path::absolute(&path).unwrap_or(path);
        // egui_extras' file loader expects a literal filesystem path after file://.
        let uri = format!("file://{}", path.to_string_lossy());
        let ir = Rect::from_min_size(
            r.min
                + egui::vec2(
                    pad.x,
                    pad.y + mindglider::layout::core_height(n, p.text_height) + 6.,
                ) * zoom,
            egui::vec2(img.width, img.height) * zoom,
        );
        egui::Image::new(uri).paint_at(ui, ir);
    }
    let mut badge = String::new();
    if !n.note.text().is_empty() {
        badge.push_str("  ▤");
    }
    if n.folded && !n.children.is_empty() {
        badge.push_str(&format!("  ＋{}", n.children.len()));
    }
    if !badge.is_empty() {
        painter.text(
            Pos2::new(r.min.x + pad.x * zoom, r.max.y - 2. * zoom),
            Align2::LEFT_BOTTOM,
            badge,
            FontId::proportional(11. * zoom),
            color(&p.style.color),
        );
    }
    if !n.details.text().is_empty() {
        painter.text(
            Pos2::new(r.min.x + pad.x * zoom, r.max.y - 18. * zoom),
            Align2::LEFT_BOTTOM,
            n.details.text().lines().next().unwrap_or_default(),
            FontId::proportional(11. * zoom),
            color(&p.style.color),
        );
    }
}
