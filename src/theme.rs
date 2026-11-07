//! Sand appearance and color presets for the egui renderer.
use eframe::egui::{self, Color32, FontId, Stroke, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    #[default]
    Sand,
    Porcelain,
    Sage,
    Mist,
    Lilac,
    SandNight,
    Graphite,
    Midnight,
    Pine,
    Plum,
}

impl Theme {
    pub const ALL: [Self; 10] = [
        Self::Sand,
        Self::Porcelain,
        Self::Sage,
        Self::Mist,
        Self::Lilac,
        Self::SandNight,
        Self::Graphite,
        Self::Midnight,
        Self::Pine,
        Self::Plum,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Sand => mindglider::i18n::tr("theme.sand"),
            Self::Porcelain => mindglider::i18n::tr("theme.porcelain"),
            Self::Sage => mindglider::i18n::tr("theme.sage"),
            Self::Mist => mindglider::i18n::tr("theme.mist"),
            Self::Lilac => mindglider::i18n::tr("theme.lilac"),
            Self::SandNight => mindglider::i18n::tr("theme.sand-night"),
            Self::Graphite => mindglider::i18n::tr("theme.graphite"),
            Self::Midnight => mindglider::i18n::tr("theme.midnight"),
            Self::Pine => mindglider::i18n::tr("theme.pine"),
            Self::Plum => mindglider::i18n::tr("theme.plum"),
        }
    }

    pub fn dark(self) -> bool {
        matches!(
            self,
            Self::SandNight | Self::Graphite | Self::Midnight | Self::Pine | Self::Plum
        )
    }
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color32,
    pub surface: Color32,
    pub canvas: Color32,
    pub field: Color32,
    pub text: Color32,
    pub secondary: Color32,
    pub border: Color32,
    pub accent: Color32,
    pub branch: Color32,
    pub on_accent: Color32,
    pub hover: Color32,
    pub selected: Color32,
}

pub fn palette(theme: Theme) -> Palette {
    let colors = match theme {
        Theme::Sand => [
            0xeae4d9, 0xf5f1e9, 0xfcfaf5, 0xfffdf8, 0x443e35, 0x6f6557, 0xd8cfc0, 0x89683e,
            0x91816b, 0xffffff,
        ],
        Theme::Porcelain => [
            0xe9edf1, 0xf5f7f9, 0xffffff, 0xffffff, 0x303a45, 0x616c78, 0xd5dce4, 0x4b6f96,
            0x8090a3, 0xffffff,
        ],
        Theme::Sage => [
            0xe0e8dd, 0xf0f4ec, 0xfafcf8, 0xfcfefb, 0x344336, 0x5b6b57, 0xced8c9, 0x54754c,
            0x7f927a, 0xffffff,
        ],
        Theme::Mist => [
            0xdfe8ed, 0xeef4f7, 0xf8fbfd, 0xfcfdff, 0x30434f, 0x556a76, 0xccd9e0, 0x3c758b,
            0x7b939e, 0xffffff,
        ],
        Theme::Lilac => [
            0xe8e2ed, 0xf4f0f7, 0xfcfaff, 0xfefcff, 0x463d50, 0x6f6178, 0xd9cfdf, 0x7a5e8c,
            0x94819f, 0xffffff,
        ],
        Theme::SandNight => [
            0x28241f, 0x302c26, 0x211f1b, 0x38322a, 0xe5dccd, 0xb5a994, 0x494035, 0xc8a66d,
            0x8e806a, 0x191919,
        ],
        Theme::Graphite => [
            0x252729, 0x2c2e31, 0x1d1f21, 0x34373b, 0xe2e5e9, 0xaab0b8, 0x43474c, 0xadb9c8,
            0x858e99, 0x191919,
        ],
        Theme::Midnight => [
            0x202c3e, 0x263347, 0x182232, 0x2d3c53, 0xdce7f5, 0x9fb1ca, 0x3a4a62, 0x8fb6e3,
            0x738bab, 0x191919,
        ],
        Theme::Pine => [
            0x23302a, 0x2a3730, 0x1b2621, 0x324238, 0xdce8df, 0xa1b6a7, 0x3e5145, 0x9ac3a2,
            0x779583, 0x191919,
        ],
        Theme::Plum => [
            0x302735, 0x392f3e, 0x261f2b, 0x45374b, 0xecdfef, 0xbaa7c0, 0x524258, 0xc6a2d2,
            0x9a7ea5, 0x191919,
        ],
    };
    let [
        background,
        surface,
        canvas,
        field,
        text,
        secondary,
        border,
        accent,
        branch,
        on_accent,
    ] = colors.map(|rgb| Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8));
    Palette {
        background,
        surface,
        canvas,
        field,
        text,
        secondary,
        border,
        accent,
        branch,
        on_accent,
        hover: surface.lerp_to_gamma(accent, 0.12),
        selected: surface.lerp_to_gamma(accent, 0.22),
    }
}

pub fn apply(ctx: &egui::Context, theme: Theme) {
    let p = palette(theme);
    let dark = theme.dark();
    let mut v = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    v.override_text_color = Some(p.text);
    v.panel_fill = p.background;
    v.window_fill = p.surface;
    v.window_stroke = Stroke::new(1.0_f32, p.border);
    v.window_corner_radius = egui::CornerRadius::same(8);
    v.menu_corner_radius = egui::CornerRadius::same(8);
    v.window_highlight_topmost = false;
    v.window_shadow = egui::epaint::Shadow {
        offset: [0, 8],
        blur: 32,
        spread: 0,
        color: Color32::from_black_alpha(if dark { 100 } else { 35 }),
    };
    v.popup_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(30),
    };
    v.extreme_bg_color = p.field;
    v.faint_bg_color = p.background;
    v.hyperlink_color = p.accent;
    v.selection.bg_fill = p.selected;
    v.selection.stroke = Stroke::new(1.0_f32, p.accent);
    v.text_cursor.preview = false;
    v.text_cursor.stroke = Stroke::new(1.5_f32, p.accent);
    v.slider_trailing_fill = true;
    v.indent_has_left_vline = false;
    v.collapsing_header_frame = true;
    for (index, w) in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ]
    .into_iter()
    .enumerate()
    {
        w.corner_radius = egui::CornerRadius::same(4);
        w.expansion = 0.;
        w.fg_stroke = Stroke::new(1.0_f32, p.text);
        w.bg_stroke = Stroke::new(1.0_f32, p.border);
        w.bg_fill = match index {
            2 => p.hover,
            3 | 4 => p.selected,
            _ => p.surface,
        };
        w.weak_bg_fill = w.bg_fill;
    }
    v.widgets.noninteractive.bg_fill = p.background;
    v.widgets.noninteractive.weak_bg_fill = Color32::TRANSPARENT;
    ctx.set_visuals(v);
    ctx.data_mut(|data| data.insert_temp(egui::Id::new("active-theme"), theme));
    ctx.style_mut(|s| {
        s.text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(14.));
        s.text_styles
            .insert(egui::TextStyle::Button, FontId::proportional(14.));
        s.text_styles
            .insert(egui::TextStyle::Small, FontId::proportional(12.));
        s.text_styles
            .insert(egui::TextStyle::Heading, FontId::proportional(20.));
        s.spacing.interact_size = Vec2::new(40., 32.);
        s.spacing.button_padding = Vec2::new(12., 6.);
        s.spacing.item_spacing = Vec2::new(8., 8.);
        s.spacing.window_margin = egui::Margin::same(24);
        s.spacing.menu_margin = egui::Margin::same(8);
        s.spacing.indent = 16.;
        s.spacing.icon_width = 16.;
        s.spacing.icon_width_inner = 10.;
        s.spacing.slider_rail_height = 4.;
        s.spacing.scroll.bar_width = 6.;
        s.spacing.scroll.floating_width = 2.;
        s.animation_time = 0.12;
    });
}
pub fn current(ctx: &egui::Context) -> Palette {
    palette(
        ctx.data(|data| data.get_temp(egui::Id::new("active-theme")))
            .unwrap_or_default(),
    )
}

/// No stock centered egui title bar: dialog content has a left-aligned title.
pub fn window<'a>(ctx: &egui::Context, title: &'a str, id: &str) -> egui::Window<'a> {
    egui::Window::new(title)
        .id(egui::Id::new((id, "fluent-v1")))
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .max_width((ctx.screen_rect().width() - 64.).max(320.))
        .frame(egui::Frame::window(&ctx.style()).inner_margin(egui::Margin::same(24)))
}
pub fn dialog_header(ui: &mut egui::Ui, title: &str, open: &mut bool) {
    ui.horizontal(|ui| {
        ui.add(egui::Label::new(egui::RichText::new(title).size(24.)).truncate());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_sized([32., 32.], egui::Button::new("\u{e711}").corner_radius(4))
                .on_hover_text(mindglider::i18n::tr("ui.close_tip"))
                .clicked()
            {
                *open = false;
            }
        });
    });
    ui.add_space(16.);
}
pub fn flat_button(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    selected: bool,
) -> egui::Response {
    let p = current(ui.ctx());
    ui.scope(|ui| {
        let widgets = &mut ui.visuals_mut().widgets;
        widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        widgets.hovered.weak_bg_fill = p.hover;
        widgets.active.weak_bg_fill = p.selected;
        for w in [
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
        ] {
            w.bg_stroke = Stroke::NONE;
        }
        ui.add(
            egui::Button::new(label)
                .selected(selected)
                .corner_radius(4)
                .stroke(Stroke::NONE),
        )
    })
    .inner
}
pub fn tool_button(ui: &mut egui::Ui, id: &str, label: &str) -> egui::Response {
    let paths: &[&[[f32; 2]]] = match id {
        "child" => &[
            &[[-5., -5.], [-5., 4.], [2., 4.]],
            &[[2., 1.], [7., 1.], [7., 7.], [2., 7.], [2., 1.]],
        ],
        "sibling" => &[
            &[[-6., -5.], [-6., 5.]],
            &[[-6., -5.], [1., -5.]],
            &[[-6., 5.], [1., 5.]],
            &[[1., -8.], [7., -8.], [7., -2.], [1., -2.], [1., -8.]],
            &[[1., 2.], [7., 2.], [7., 8.], [1., 8.], [1., 2.]],
        ],
        "fold" => &[
            &[[-7., -4.], [-3., 0.], [-7., 4.]],
            &[[7., -4.], [3., 0.], [7., 4.]],
            &[[0., -6.], [0., 6.]],
        ],
        "undo" => return flat_button(ui, format!("\u{e7a7}  {label}"), false),
        "redo" => return flat_button(ui, format!("\u{e7a6}  {label}"), false),
        _ => return flat_button(ui, format!("\u{e9a6}  {label}"), false),
    };
    let response = flat_button(ui, format!("\u{2003}  {label}"), false);
    let center = response.rect.left_center() + egui::vec2(ui.spacing().button_padding.x + 7., 0.);
    let stroke = Stroke::new(1.3_f32, ui.visuals().text_color());
    for path in paths {
        let points = path
            .iter()
            .map(|[x, y]| center + egui::vec2(*x, *y))
            .collect();
        ui.painter().add(egui::Shape::line(points, stroke));
    }
    response
}

impl Palette {
    pub fn map_style(self, style: &mut mindglider::model::Style) {
        let defaults = mindglider::model::Style::default();
        for (value, default, color) in [
            (&mut style.color, defaults.color, self.text),
            (&mut style.background, defaults.background, self.canvas),
            (&mut style.edge_color, defaults.edge_color, self.branch),
        ] {
            if *value == default {
                *value = format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b());
            }
        }
    }
}

pub fn chevron(ui: &egui::Ui, center: egui::Pos2, angle: f32, color: Color32) {
    let rotation = egui::emath::Rot2::from_angle(angle);
    let points = [
        egui::vec2(-4., -2.),
        egui::vec2(0., 2.),
        egui::vec2(4., -2.),
    ]
    .map(|p| center + rotation * p);
    ui.painter().add(egui::Shape::line(
        points.to_vec(),
        Stroke::new(1.3_f32, color),
    ));
}
pub fn expander(text: impl Into<egui::WidgetText>) -> egui::CollapsingHeader {
    egui::CollapsingHeader::new(text).icon(|ui, openness, response| {
        let angle = egui::lerp(-std::f32::consts::FRAC_PI_2..=0., openness);
        chevron(ui, response.rect.center(), angle, ui.visuals().text_color());
    })
}
pub fn accent_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let p = current(ui.ctx());
    ui.scope(|ui| {
        let widgets = &mut ui.visuals_mut().widgets;
        widgets.inactive.weak_bg_fill = p.accent;
        widgets.hovered.weak_bg_fill = p.accent;
        widgets.active.weak_bg_fill = p.accent;
        ui.add(egui::Button::new(
            egui::RichText::new(label).color(p.on_accent),
        ))
    })
    .inner
}
#[cfg(test)]
mod tests {
    use super::*;
    fn luminance(c: Color32) -> f32 {
        let channel = |v: u8| {
            let x = v as f32 / 255.;
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
    }
    fn contrast(a: Color32, b: Color32) -> f32 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
    #[test]
    fn text_and_accent_buttons_meet_minimum_contrast_in_all_themes() {
        for theme in Theme::ALL {
            let p = palette(theme);
            for surface in [p.background, p.surface, p.canvas] {
                assert!(contrast(p.text, surface) >= 4.5);
                assert!(contrast(p.secondary, surface) >= 4.5);
            }
            assert!(contrast(p.on_accent, p.accent) >= 4.5);
            assert!(contrast(p.branch, p.canvas) >= 3.);
        }
    }
}
