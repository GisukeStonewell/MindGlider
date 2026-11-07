use eframe::egui::{self, Color32, FontId, TextFormat, text::LayoutJob};
use mindglider::i18n::tr;
use mindglider::{
    conditions::{Condition, Field, Operator},
    layout::color,
    model::Style,
    richtext::RichText,
};

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let db = mindglider::fonts::system_database();
    ctx.data_mut(|d| {
        d.insert_temp(
            egui::Id::new("icon-fontdb"),
            std::sync::Arc::new(db.clone()),
        )
    });
    let fallback = fonts.families[&egui::FontFamily::Proportional].clone();
    let mut ui_fonts = Vec::new();
    for name in [
        "Segoe UI Variable",
        "Segoe UI",
        "Yu Gothic UI",
        "Meiryo",
        "Segoe Fluent Icons",
        "Noto Sans JP",
        "Yu Gothic",
        "MS Gothic",
        "Arial",
        "Consolas",
    ] {
        for (suffix, weight) in [("", 400), (" Bold", 700)] {
            let key = format!("{name}{suffix}");
            let face = db.query(&resvg::usvg::fontdb::Query {
                families: &[resvg::usvg::fontdb::Family::Name(name)],
                weight: resvg::usvg::fontdb::Weight(weight),
                ..Default::default()
            });
            let loaded = face.and_then(|id| {
                db.with_face_data(id, |bytes, index| {
                    let mut data = egui::FontData::from_owned(bytes.to_vec());
                    data.index = index;
                    data
                })
            });
            let mut family = Vec::new();
            if let Some(data) = loaded {
                fonts.font_data.insert(key.clone(), data.into());
                family.push(key.clone());
                if suffix.is_empty()
                    && matches!(
                        name,
                        "Segoe UI Variable"
                            | "Segoe UI"
                            | "Yu Gothic UI"
                            | "Meiryo"
                            | "Segoe Fluent Icons"
                    )
                {
                    ui_fonts.push(key.clone());
                }
            }
            family.extend(ui_fonts.clone());
            family.extend(fallback.clone());
            fonts
                .families
                .insert(egui::FontFamily::Name(key.into()), family);
        }
    }
    for entries in fonts.families.values_mut() {
        for name in &ui_fonts {
            if !entries.contains(name) {
                entries.insert(entries.len().saturating_sub(fallback.len()), name.clone());
            }
        }
    }
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        let entries = fonts.families.entry(family).or_default();
        let mut preferred = ui_fonts.clone();
        preferred.append(entries);
        *entries = preferred;
    }
    ctx.set_fonts(fonts);
}

/// An editable combo: text remains editable even when it is absent from the suggestions.
pub fn editable_combo(
    ui: &mut egui::Ui,
    id: &str,
    text: &mut String,
    items: &[(String, String)],
    width: f32,
) -> egui::Response {
    ui.scope(|ui| {
        let popup_id = combo_popup_id(id);
        let (rect, anchor) = ui.allocate_exact_size(egui::vec2(width, 32.), egui::Sense::hover());
        let background = ui.painter().add(egui::Shape::Noop);
        let input_rect = egui::Rect::from_min_max(rect.min, rect.max - egui::vec2(32., 0.));
        let input = ui.put(
            input_rect,
            egui::TextEdit::singleline(text)
                .frame(false)
                .margin(egui::Margin::symmetric(12, 6))
                .id_salt(id)
                .hint_text(if id == "ai-instruction" {
                    tr("app.073")
                } else {
                    tr("cli.default")
                }),
        );
        let arrow_rect =
            egui::Rect::from_min_max(egui::pos2(rect.right() - 32., rect.top()), rect.max);
        let arrow = ui.interact(
            arrow_rect,
            ui.id().with((id, "arrow")),
            egui::Sense::click(),
        );
        if arrow.clicked()
            || (input.has_focus()
                && ui.input_mut(|i| i.consume_key(egui::Modifiers::ALT, egui::Key::ArrowDown)))
        {
            ui.memory_mut(|m| m.toggle_popup(popup_id));
        }
        let visuals = ui.visuals();
        ui.painter().set(
            background,
            egui::epaint::RectShape::new(
                rect,
                4.,
                visuals.extreme_bg_color,
                if input.has_focus() {
                    visuals.selection.stroke
                } else {
                    visuals.widgets.inactive.bg_stroke
                },
                egui::StrokeKind::Inside,
            ),
        );
        crate::theme::chevron(ui, arrow_rect.center(), 0., visuals.text_color());
        let mut picked = false;
        let selection_id = popup_id.with("selection");
        let mut index = ui
            .data(|d| d.get_temp::<usize>(selection_id))
            .unwrap_or_else(|| {
                items
                    .iter()
                    .position(|(value, _)| value == text)
                    .unwrap_or(0)
            });
        index = index.min(items.len().saturating_sub(1));
        let popup_open = ui.memory(|m| m.is_popup_open(popup_id));
        let mut navigated = false;
        if popup_open {
            ui.input_mut(|i| {
                if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
                    index = (index + 1).min(items.len().saturating_sub(1));
                    navigated = true;
                }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                    index = index.saturating_sub(1);
                    navigated = true;
                }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                    && let Some((value, _)) = items.get(index)
                {
                    *text = value.clone();
                    picked = true;
                }
            });
        }
        egui::popup::popup_below_widget(
            ui,
            popup_id,
            &anchor,
            egui::PopupCloseBehavior::CloseOnClickOutside,
            |ui| {
                ui.set_width((width - 16.).max(40.));
                egui::ScrollArea::vertical()
                    .max_height(280.)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for (row, (value, label)) in items.iter().enumerate() {
                            let response = ui.selectable_label(index == row, label);
                            if row == index && navigated {
                                response.scroll_to_me(Some(egui::Align::Center));
                            }
                            if response.clicked() {
                                *text = value.clone();
                                picked = true;
                            }
                        }
                    });
            },
        );
        ui.data_mut(|d| d.insert_temp(selection_id, index));
        if picked {
            ui.memory_mut(|m| m.close_popup());
            input.request_focus();
        }
        input
    })
    .inner
}

pub fn combo_popup_id(id: &str) -> egui::Id {
    egui::Id::new(("editable-combo-popup", id))
}

/// Rasterize each application-owned vector once; the same paths are used in SVG/PDF export.
pub fn paint_icon(ui: &egui::Ui, name: &str, rect: egui::Rect) {
    let key = egui::Id::new(("node-icon-texture", name));
    let cached = ui.ctx().data(|d| d.get_temp::<egui::TextureHandle>(key));
    let texture = cached.or_else(|| {
        let svg = mindglider::icons::svg(name);
        let db = ui
            .ctx()
            .data(|d| {
                d.get_temp::<std::sync::Arc<resvg::usvg::fontdb::Database>>(egui::Id::new(
                    "icon-fontdb",
                ))
            })
            .unwrap_or_else(|| std::sync::Arc::new(mindglider::fonts::system_database()));
        let options = resvg::usvg::Options {
            fontdb: db,
            ..Default::default()
        };
        let tree = resvg::usvg::Tree::from_str(&svg, &options).ok()?;
        let mut pix = resvg::tiny_skia::Pixmap::new(48, 48)?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(2., 2.),
            &mut pix.as_mut(),
        );
        let image = egui::ColorImage::from_rgba_premultiplied([48, 48], pix.data());
        let texture = ui
            .ctx()
            .load_texture(name, image, egui::TextureOptions::LINEAR);
        ui.ctx().data_mut(|d| d.insert_temp(key, texture.clone()));
        Some(texture)
    });
    if let Some(texture) = texture {
        ui.painter().image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
            egui::Color32::WHITE,
        );
    }
}

/// A fixed-size field avoids a nested horizontal layout being squeezed by Grid.
pub fn shortcut_input(
    ui: &mut egui::Ui,
    id: &str,
    value: &mut String,
) -> egui::text_edit::TextEditOutput {
    ui.scope(|ui| {
        ui.visuals_mut().text_cursor.preview = false;
        egui::TextEdit::singleline(value)
            .id_salt(("shortcut", id))
            .desired_width(216.)
            .min_size(egui::vec2(240., 32.))
            .margin(egui::Margin::symmetric(12, 6))
            .show(ui)
    })
    .inner
}

pub fn preference_text(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.label(label);
    ui.add_sized([320., 32.], egui::TextEdit::singleline(value));
    if ui.button(tr("ui.reset")).clicked() {
        value.clear();
    }
    ui.end_row();
}
pub fn preference_check(ui: &mut egui::Ui, label: &str, value: &mut bool) {
    ui.label(label);
    ui.checkbox(value, "");
    if ui.button(tr("ui.reset")).clicked() {
        *value = false;
    }
    ui.end_row();
}

pub fn hex(c: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}
pub fn color_edit(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut c = color(value);
        if ui.color_edit_button_srgba(&mut c).changed() {
            *value = hex(c);
        }
    });
}
pub fn style_editor(ui: &mut egui::Ui, style: &mut Style) {
    egui::ComboBox::from_id_salt(ui.id().with("font-family"))
        .icon(|ui, rect, visuals, _, _| {
            crate::theme::chevron(ui, rect.center(), 0., visuals.fg_stroke.color)
        })
        .selected_text(&style.font_family)
        .show_ui(ui, |ui| {
            for name in [
                "Segoe UI Variable",
                "Segoe UI",
                "Yu Gothic UI",
                "Noto Sans JP",
                "Yu Gothic",
                "Meiryo",
                "MS Gothic",
                "Arial",
                "Consolas",
            ] {
                ui.selectable_value(&mut style.font_family, name.into(), name);
            }
        });
    ui.horizontal(|ui| {
        ui.checkbox(&mut style.bold, tr("ui_helpers.276"));
        ui.checkbox(&mut style.italic, tr("ui_helpers.275"));
        ui.add(
            egui::DragValue::new(&mut style.font_size)
                .range(8.0..=72.0)
                .suffix(" pt"),
        );
    });
    color_edit(ui, tr("ui_helpers.274"), &mut style.color);
    color_edit(ui, tr("ui_helpers.273"), &mut style.background);
    color_edit(ui, tr("ui_helpers.272"), &mut style.edge_color);
    egui::ComboBox::from_id_salt(ui.id().with("shape"))
        .icon(|ui, rect, visuals, _, _| {
            crate::theme::chevron(ui, rect.center(), 0., visuals.fg_stroke.color)
        })
        .selected_text(tr(&format!("shape.{}", style.shape)))
        .show_ui(ui, |ui| {
            for shape in ["bubble", "fork", "oval", "rectangle", "wide_hexagon"] {
                ui.selectable_value(
                    &mut style.shape,
                    shape.into(),
                    tr(&format!("shape.{shape}")),
                );
            }
        });
}
pub fn text_job(rich: &RichText, style: &Style, scale: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    for run in &rich.runs {
        let name = if matches!(
            style.font_family.as_str(),
            "Segoe UI Variable"
                | "Segoe UI"
                | "Yu Gothic UI"
                | "Noto Sans JP"
                | "Yu Gothic"
                | "Meiryo"
                | "MS Gothic"
                | "Arial"
                | "Consolas"
        ) {
            style.font_family.as_str()
        } else {
            "Segoe UI Variable"
        };
        let family = egui::FontFamily::Name(if run.marks.bold || style.bold {
            format!("{name} Bold").into()
        } else {
            name.into()
        });
        let mut format = TextFormat {
            font_id: FontId::new(run.marks.size.unwrap_or(style.font_size) * scale, family),
            color: color(run.marks.color.as_ref().unwrap_or(&style.color)),
            italics: run.marks.italic || style.italic,
            ..Default::default()
        };
        if run.marks.underline || run.marks.link.is_some() {
            format.underline = egui::Stroke::new(1.0_f32, format.color);
        }
        if run.marks.strike {
            format.strikethrough = egui::Stroke::new(1.0_f32, format.color);
        }
        job.append(&run.text, 0., format);
    }
    job
}
pub fn paint_node(painter: &egui::Painter, r: egui::Rect, style: &Style, stroke: egui::Stroke) {
    let fill = color(&style.background);
    match style.shape.as_str() {
        "oval" | "circle" => {
            painter.add(egui::epaint::EllipseShape::filled(
                r.center(),
                r.size() / 2.,
                fill,
            ));
            painter.add(egui::epaint::EllipseShape::stroke(
                r.center(),
                r.size() / 2.,
                stroke,
            ));
        }
        "wide_hexagon" | "narrow_hexagon" => {
            let dx = r.width() * 0.12;
            let points = vec![
                egui::pos2(r.min.x + dx, r.min.y),
                egui::pos2(r.max.x - dx, r.min.y),
                egui::pos2(r.max.x, r.center().y),
                egui::pos2(r.max.x - dx, r.max.y),
                egui::pos2(r.min.x + dx, r.max.y),
                egui::pos2(r.min.x, r.center().y),
            ];
            painter.add(egui::Shape::convex_polygon(points, fill, stroke));
        }
        "fork" => {
            painter.line_segment([r.left_bottom(), r.right_bottom()], stroke);
        }
        _ => {
            let corner = if style.shape == "rectangle" { 2. } else { 14. };
            painter.rect_filled(r, corner, fill);
            painter.rect_stroke(r, corner, stroke, egui::StrokeKind::Inside);
        }
    }
}
pub fn rich_editor(ui: &mut egui::Ui, id: &str, rich: &mut RichText, rows: usize) {
    let editor_id = ui.id().with(id);
    let range_id = editor_id.with("selection");
    if rich.unsupported {
        ui.colored_label(Color32::from_rgb(170, 100, 20), tr("ui_helpers.271"));
        ui.label(rich.text());
        if ui.button(tr("ui_helpers.270")).clicked() {
            rich.unsupported = false;
        }
        return;
    }
    let range = ui
        .data(|d| d.get_temp::<std::ops::Range<usize>>(range_id))
        .unwrap_or(0..0);
    ui.horizontal_wrapped(|ui| {
        for (label, kind) in [
            (tr("format.bold_short"), 0),
            (tr("format.italic_short"), 1),
            (tr("format.underline_short"), 2),
            (tr("format.strike_short"), 3),
        ] {
            if ui.button(label).clicked() {
                rich.format(range.start, range.end, |m| match kind {
                    0 => m.bold = !m.bold,
                    1 => m.italic = !m.italic,
                    2 => m.underline = !m.underline,
                    _ => m.strike = !m.strike,
                });
            }
        }
        let mut c = ui
            .data(|d| d.get_temp::<Color32>(editor_id.with("color")))
            .unwrap_or_else(|| ui.visuals().text_color());
        if ui.color_edit_button_srgba(&mut c).changed() {
            rich.format(range.start, range.end, |m| m.color = Some(hex(c)));
            ui.data_mut(|d| d.insert_temp(editor_id.with("color"), c));
        }
        let mut size = ui
            .data(|d| d.get_temp::<f32>(editor_id.with("size")))
            .unwrap_or(16.);
        if ui
            .add(
                egui::DragValue::new(&mut size)
                    .range(8.0..=72.0)
                    .suffix("pt"),
            )
            .changed()
        {
            rich.format(range.start, range.end, |m| m.size = Some(size));
            ui.data_mut(|d| d.insert_temp(editor_id.with("size"), size));
        }
    });
    ui.horizontal(|ui| {
        let mut url = ui
            .data(|d| d.get_temp::<String>(editor_id.with("url")))
            .unwrap_or_default();
        ui.add(
            egui::TextEdit::singleline(&mut url)
                .hint_text(tr("ui_helpers.269"))
                .desired_width(180.),
        );
        if ui.button(tr("ui_helpers.268")).clicked() {
            rich.format(range.start, range.end, |m| {
                m.link = if url.is_empty() {
                    None
                } else {
                    Some(url.clone())
                }
            });
        }
        ui.data_mut(|d| d.insert_temp(editor_id.with("url"), url));
    });
    let mut text = rich.text();
    let original = rich.clone();
    let mut layouter = move |ui: &egui::Ui, text: &str, width: f32| {
        let mut r = original.clone();
        if r.text() != text {
            r.edit(text);
        }
        let style = Style {
            color: hex(ui.visuals().text_color()),
            ..Default::default()
        };
        let mut job = text_job(&r, &style, 1.);
        job.wrap.max_width = width;
        ui.fonts(|f| f.layout_job(job))
    };
    let mut out = egui::TextEdit::multiline(&mut text)
        .id(editor_id)
        .desired_rows(rows)
        .desired_width(f32::INFINITY)
        .layouter(&mut layouter)
        .show(ui);
    if id == "text"
        && ui
            .ctx()
            .data_mut(|d| d.remove_temp::<bool>(egui::Id::new("request-node-editor")))
            .unwrap_or(false)
    {
        out.response.request_focus();
        out.state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::two(
                egui::text::CCursor::new(0),
                egui::text::CCursor::new(text.chars().count()),
            )));
        out.state.store(ui.ctx(), editor_id);
    }
    if out.response.changed() {
        rich.edit(&text);
    }
    if let Some(cursor) = out.cursor_range {
        ui.data_mut(|d| d.insert_temp(range_id, cursor.as_sorted_char_range()));
    }
}
pub fn condition_editor(ui: &mut egui::Ui, condition: &mut Condition, depth: usize) {
    if depth > 12 {
        ui.label(tr("ui_helpers.267"));
        return;
    }
    let mut kind = match condition {
        Condition::Match { .. } => 0,
        Condition::And { .. } => 1,
        Condition::Or { .. } => 2,
        Condition::Not { .. } => 3,
    };
    let before = kind;
    egui::ComboBox::from_id_salt(ui.id().with("kind"))
        .icon(|ui, rect, visuals, _, _| {
            crate::theme::chevron(ui, rect.center(), 0., visuals.fg_stroke.color)
        })
        .selected_text(
            [
                tr("ui_helpers.266"),
                tr("logic.and"),
                tr("logic.or"),
                tr("logic.not"),
            ][kind],
        )
        .show_ui(ui, |ui| {
            for (i, k) in [
                tr("ui_helpers.266"),
                tr("logic.and"),
                tr("logic.or"),
                tr("logic.not"),
            ]
            .iter()
            .enumerate()
            {
                ui.selectable_value(&mut kind, i, *k);
            }
        });
    if before != kind {
        let old = condition.clone();
        *condition = match kind {
            1 => Condition::And { items: vec![old] },
            2 => Condition::Or { items: vec![old] },
            3 => Condition::Not {
                item: Box::new(old),
            },
            _ => Condition::default(),
        };
    }
    match condition {
        Condition::Match {
            field,
            op,
            value,
            case_sensitive,
        } => {
            ui.horizontal_wrapped(|ui| {
                egui::ComboBox::from_id_salt(ui.id().with("field"))
                    .icon(|ui, rect, visuals, _, _| {
                        crate::theme::chevron(ui, rect.center(), 0., visuals.fg_stroke.color)
                    })
                    .selected_text(match field {
                        Field::Text => tr("ui_helpers.265"),
                        Field::Note => tr("ui_helpers.264"),
                        Field::Details => tr("ui_helpers.263"),
                        Field::Depth => tr("ui_helpers.262"),
                        Field::Icon => tr("ui_helpers.261"),
                        Field::Progress => tr("ui_helpers.260"),
                        Field::Link => tr("ui_helpers.259"),
                    })
                    .show_ui(ui, |ui| {
                        for (f, label) in [
                            (Field::Text, tr("ui_helpers.265")),
                            (Field::Note, tr("ui_helpers.264")),
                            (Field::Details, tr("ui_helpers.263")),
                            (Field::Depth, tr("ui_helpers.262")),
                            (Field::Icon, tr("ui_helpers.261")),
                            (Field::Progress, tr("ui_helpers.260")),
                            (Field::Link, tr("ui_helpers.259")),
                        ] {
                            ui.selectable_value(field, f, label);
                        }
                    });
                egui::ComboBox::from_id_salt(ui.id().with("op"))
                    .icon(|ui, rect, visuals, _, _| {
                        crate::theme::chevron(ui, rect.center(), 0., visuals.fg_stroke.color)
                    })
                    .selected_text(match op {
                        Operator::Equals => tr("ui_helpers.258"),
                        Operator::Contains => tr("ui_helpers.257"),
                        Operator::StartsWith => tr("ui_helpers.256"),
                        Operator::Regex => tr("ui_helpers.255"),
                        Operator::Greater => tr("ui_helpers.254"),
                        Operator::Less => tr("ui_helpers.253"),
                    })
                    .show_ui(ui, |ui| {
                        for (o, label) in [
                            (Operator::Equals, tr("ui_helpers.258")),
                            (Operator::Contains, tr("ui_helpers.257")),
                            (Operator::StartsWith, tr("ui_helpers.256")),
                            (Operator::Regex, tr("ui_helpers.255")),
                            (Operator::Greater, tr("ui_helpers.254")),
                            (Operator::Less, tr("ui_helpers.253")),
                        ] {
                            ui.selectable_value(op, o, label);
                        }
                    });
            });
            ui.text_edit_singleline(value);
            ui.checkbox(case_sensitive, tr("ui_helpers.252"));
        }
        Condition::And { items } | Condition::Or { items } => {
            let mut remove = None;
            for (i, c) in items.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    ui.group(|ui| {
                        condition_editor(ui, c, depth + 1);
                        if ui.button(tr("ui_helpers.251")).clicked() {
                            remove = Some(i);
                        }
                    });
                });
            }
            if let Some(i) = remove {
                items.remove(i);
            }
            if ui.button(tr("ui_helpers.250")).clicked() {
                items.push(Condition::default());
            }
        }
        Condition::Not { item } => {
            ui.indent("not", |ui| condition_editor(ui, item, depth + 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn japanese_ime_preedit_and_commit_do_not_duplicate_text() {
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        let mut rich = RichText::plain("original");
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("request-node-editor"), true));
        let draw = |ctx: &egui::Context, rich: &mut RichText, events| {
            let input = egui::RawInput {
                events,
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| rich_editor(ui, "text", rich, 3));
            });
        };
        draw(&ctx, &mut rich, vec![]);
        draw(
            &ctx,
            &mut rich,
            vec![
                egui::Event::Ime(egui::ImeEvent::Enabled),
                egui::Event::Ime(egui::ImeEvent::Preedit("にほんご".into())),
            ],
        );
        draw(
            &ctx,
            &mut rich,
            vec![
                egui::Event::Ime(egui::ImeEvent::Commit("日本語".into())),
                egui::Event::Ime(egui::ImeEvent::Disabled),
            ],
        );
        assert_eq!(rich.text(), "日本語");
    }
}

#[cfg(test)]
mod ime_enter_tests {
    use super::*;
    #[test]
    fn commit_enter_does_not_insert_newline_in_a_text_widget() {
        for reversed in [false, true] {
            let ctx = egui::Context::default();
            install_fonts(&ctx);
            let mut guard = mindglider::input::ImeGuard::default();
            let mut rich = RichText::plain("original");
            ctx.data_mut(|d| d.insert_temp(egui::Id::new("request-node-editor"), true));
            let mut time = 0.;
            let mut draw = |rich: &mut RichText, mut events: Vec<egui::Event>| {
                time += 0.03;
                guard.filter(&mut events, time);
                let _ = ctx.run(
                    egui::RawInput {
                        events,
                        time: Some(time),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default()
                            .show(ctx, |ui| rich_editor(ui, "text", rich, 3));
                    },
                );
            };
            draw(&mut rich, vec![]);
            draw(
                &mut rich,
                vec![
                    egui::Event::Ime(egui::ImeEvent::Enabled),
                    egui::Event::Ime(egui::ImeEvent::Preedit("にほん".into())),
                ],
            );
            let mut events = vec![
                egui::Event::Ime(egui::ImeEvent::Commit("日本".into())),
                egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ];
            if reversed {
                events.reverse();
            }
            draw(&mut rich, events);
            assert_eq!(rich.text(), "日本");
        }
    }
}

#[cfg(test)]
mod combo_layout_tests {
    use super::*;
    #[test]
    fn editable_combo_stays_inside_its_requested_width() {
        let ctx = egui::Context::default();
        crate::theme::apply(&ctx, crate::theme::Theme::Sand);
        let mut text = String::new();
        for _ in 0..4 {
            let _ = ctx.run(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let row = ui.allocate_ui_with_layout(
                        egui::vec2(350., 32.),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            editable_combo(ui, "combo-test", &mut text, &[], 350.);
                        },
                    );
                    assert!(
                        row.response.rect.width() <= 350.5,
                        "combo overflow: {}",
                        row.response.rect.width()
                    );
                });
            });
        }
    }
}

#[cfg(test)]
mod shortcut_layout_tests {
    use super::*;
    #[test]
    fn shortcut_text_is_visible_and_hover_cannot_expand_the_row() {
        let ctx = egui::Context::default();
        crate::theme::apply(&ctx, crate::theme::Theme::Sand);
        let mut value = "Ctrl+Shift+Alt+ArrowRight".to_owned();
        let mut pointer = egui::Pos2::ZERO;
        for frame in 0..4 {
            let _ = ctx.run(
                egui::RawInput {
                    events: if frame > 0 {
                        vec![egui::Event::PointerMoved(pointer)]
                    } else {
                        vec![]
                    },
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Command");
                            let field = ui.scope(|ui| shortcut_input(ui, "test", &mut value));
                            let out = field.inner;
                            pointer = out.response.rect.center();
                            assert!(out.text_clip_rect.contains_rect(
                                out.galley.rect.translate(out.galley_pos.to_vec2())
                            ));
                            assert!(out.response.rect.height() <= 32.5);
                            assert!(
                                field.response.rect.width() >= 239.5,
                                "outer field width: {}",
                                field.response.rect.width()
                            );
                            assert!(field.response.rect.height() <= 32.5);
                            assert!(!ui.visuals().text_cursor.preview);
                            ui.button("Reset").on_hover_text("Reset only this setting");
                        });
                    });
                },
            );
        }
    }
}
#[cfg(test)]
mod integrated_combo_tests {
    use super::*;
    #[test]
    fn combo_is_one_grid_cell_and_keyboard_can_choose_history() {
        let ctx = egui::Context::default();
        crate::theme::apply(&ctx, crate::theme::Theme::Sand);
        let mut text = String::new();
        let items = vec![
            ("first".into(), "First".into()),
            ("second".into(), "Second".into()),
        ];
        for frame in 0..5 {
            let key = if frame == 2 {
                Some(egui::Key::ArrowDown)
            } else if frame == 3 {
                Some(egui::Key::Enter)
            } else {
                None
            };
            let input = egui::RawInput {
                events: key
                    .map(|key| egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    })
                    .into_iter()
                    .collect(),
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::Grid::new("combo-grid-test")
                        .num_columns(3)
                        .show(ui, |ui| {
                            ui.label("Model");
                            let combo =
                                editable_combo(ui, "history-grid-test", &mut text, &items, 320.);
                            let reset = ui.button("Reset");
                            assert!(
                                reset.rect.left() - combo.rect.left() < 350.,
                                "reset button advanced an extra grid cell"
                            );
                            ui.end_row();
                        });
                });
                if frame == 1 {
                    ctx.memory_mut(|m| m.open_popup(combo_popup_id("history-grid-test")));
                }
            });
        }
        assert_eq!(text, "second");
        assert!(!ctx.memory(|m| m.is_popup_open(combo_popup_id("history-grid-test"))));
    }
}

/// A small panel handle that never expands a collapsed 24px rail.
pub fn panel_handle(ui: &mut egui::Ui, right: bool, tooltip: &str) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(24., 32.), egui::Sense::click());
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, 4., ui.visuals().widgets.hovered.weak_bg_fill);
    }
    crate::theme::chevron(
        ui,
        rect.center(),
        if right {
            -std::f32::consts::FRAC_PI_2
        } else {
            std::f32::consts::FRAC_PI_2
        },
        ui.visuals().text_color(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), tooltip)
    });
    response.on_hover_text(tooltip).clicked()
}
#[cfg(test)]
mod local_image_tests {
    #[test]
    fn local_image_loader_accepts_spaces_and_japanese_paths() {
        let ctx = eframe::egui::Context::default();
        egui_extras::install_image_loaders(&ctx);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("画像 space.png");
        image::RgbaImage::from_pixel(8, 8, image::Rgba([10, 20, 30, 255]))
            .save(&path)
            .unwrap();
        let image = eframe::egui::Image::new(format!("file://{}", path.to_string_lossy()));
        for _ in 0..200 {
            match image.load_for_size(&ctx, eframe::egui::vec2(8., 8.)) {
                Ok(eframe::egui::load::TexturePoll::Ready { .. }) => return,
                Ok(_) => std::thread::sleep(std::time::Duration::from_millis(5)),
                Err(e) => panic!("Local image failed: {e}"),
            }
        }
        panic!("Local image load timed out");
    }
}
