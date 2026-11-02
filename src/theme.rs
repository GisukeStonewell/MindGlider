//! Fluent / WinUI design tokens for the egui renderer. All sizes are logical pixels.
use eframe::egui::{self, Color32, FontId, Stroke, Vec2};
#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color32,
    pub surface: Color32,
    pub canvas: Color32,
    pub text: Color32,
    pub secondary: Color32,
    pub border: Color32,
    pub accent: Color32,
    pub hover: Color32,
    pub selected: Color32,
}
pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
            background: Color32::from_rgb(32, 32, 32),
            surface: Color32::from_rgb(44, 44, 44),
            canvas: Color32::from_rgb(39, 39, 39),
            text: Color32::from_rgb(255, 255, 255),
            secondary: Color32::from_rgb(190, 190, 190),
            border: Color32::from_white_alpha(20),
            accent: Color32::from_rgb(96, 205, 255),
            hover: Color32::from_rgb(58, 58, 58),
            selected: Color32::from_rgb(49, 65, 74),
        }
    } else {
        Palette {
            background: Color32::from_rgb(243, 243, 243),
            surface: Color32::WHITE,
            canvas: Color32::from_rgb(250, 250, 250),
            text: Color32::from_rgb(26, 26, 26),
            secondary: Color32::from_rgb(96, 96, 96),
            border: Color32::from_black_alpha(18),
            accent: Color32::from_rgb(0, 95, 184),
            hover: Color32::from_rgb(235, 235, 235),
            selected: Color32::from_rgb(229, 241, 251),
        }
    }
}
pub fn apply(ctx: &egui::Context, dark: bool) {
    let p = palette(dark);
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
    v.extreme_bg_color = if dark {
        Color32::from_rgb(35, 35, 35)
    } else {
        Color32::WHITE
    };
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
    let p = palette(ui.visuals().dark_mode);
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
    let dark = ui.visuals().dark_mode;
    let p = palette(dark);
    ui.scope(|ui| {
        let widgets = &mut ui.visuals_mut().widgets;
        widgets.inactive.weak_bg_fill = p.accent;
        widgets.hovered.weak_bg_fill = p.accent.gamma_multiply(0.9);
        widgets.active.weak_bg_fill = p.accent.gamma_multiply(0.8);
        ui.add(egui::Button::new(
            egui::RichText::new(label).color(if dark { Color32::BLACK } else { Color32::WHITE }),
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
    fn text_and_accent_buttons_meet_minimum_contrast_in_both_themes() {
        for dark in [false, true] {
            let p = palette(dark);
            for surface in [p.background, p.surface, p.canvas] {
                assert!(contrast(p.text, surface) >= 7.);
                assert!(contrast(p.secondary, surface) >= 4.5);
            }
            assert!(contrast(if dark { Color32::BLACK } else { Color32::WHITE }, p.accent) >= 4.5);
        }
    }
}
