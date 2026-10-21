//! Pequeños componentes reutilizables para mantener un lenguaje visual uniforme.

use egui::{
    Align2, Button, Color32, FontId, Pos2, Response, RichText, Sense, Stroke, Ui, Vec2, vec2,
};

use crate::icons::{self, Icon};
use crate::theme::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

pub fn action_button(ui: &mut Ui, text: &str, kind: ButtonKind) -> Response {
    let (fill, stroke, color) = match kind {
        ButtonKind::Primary => (ACCENT, Stroke::new(1.0, ACCENT), Color32::WHITE),
        ButtonKind::Secondary => (CARD, Stroke::new(1.0, BORDER_SOFT), TEXT),
        ButtonKind::Ghost => (Color32::TRANSPARENT, Stroke::NONE, TEXT_MUTED),
        ButtonKind::Danger => (DANGER_SOFT, Stroke::new(1.0, with_alpha(DANGER, 90)), DANGER),
    };
    let button = Button::new(RichText::new(text).size(13.0).color(color))
        .fill(fill)
        .stroke(stroke)
        .corner_radius(RADIUS_SMALL);
    let response = ui.add(button);
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

pub fn full_button(ui: &mut Ui, text: &str, kind: ButtonKind) -> Response {
    let width = ui.available_width().max(80.0);
    let height = 36.0;
    ui.add_sized([width, height], button_widget(text, kind))
}

fn button_widget(text: &str, kind: ButtonKind) -> Button<'static> {
    let (fill, stroke, color) = match kind {
        ButtonKind::Primary => (ACCENT, Stroke::new(1.0, ACCENT), Color32::WHITE),
        ButtonKind::Secondary => (CARD, Stroke::new(1.0, BORDER_SOFT), TEXT),
        ButtonKind::Ghost => (Color32::TRANSPARENT, Stroke::NONE, TEXT_MUTED),
        ButtonKind::Danger => (DANGER_SOFT, Stroke::new(1.0, with_alpha(DANGER, 90)), DANGER),
    };
    Button::new(RichText::new(text.to_owned()).size(13.0).color(color))
        .fill(fill)
        .stroke(stroke)
        .corner_radius(RADIUS_SMALL)
}

pub fn icon_text_button(ui: &mut Ui, icon: Icon, text: &str, kind: ButtonKind) -> Response {
    let color = match kind {
        ButtonKind::Primary => Color32::WHITE,
        ButtonKind::Secondary => TEXT,
        ButtonKind::Ghost => TEXT_MUTED,
        ButtonKind::Danger => DANGER,
    };
    let fill = match kind {
        ButtonKind::Primary => ACCENT,
        ButtonKind::Secondary => CARD,
        ButtonKind::Ghost => Color32::TRANSPARENT,
        ButtonKind::Danger => DANGER_SOFT,
    };
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), 34.0),
        Sense::click(),
    );
    let hovered = response.hovered();
    let bg = if kind == ButtonKind::Primary {
        if hovered { ACCENT_HOVER } else { fill }
    } else if hovered {
        CARD_HOVER
    } else {
        fill
    };
    ui.painter().rect_filled(rect, RADIUS_SMALL, bg);
    if kind == ButtonKind::Secondary {
        ui.painter().rect_stroke(
            rect,
            RADIUS_SMALL,
            Stroke::new(1.0, BORDER_SOFT),
            egui::StrokeKind::Inside,
        );
    }
    let icon_rect = egui::Rect::from_center_size(
        Pos2::new(rect.left() + 18.0, rect.center().y),
        Vec2::splat(16.0),
    );
    icons::paint_icon(ui.painter(), icon_rect, icon, color);
    ui.painter().text(
        Pos2::new(rect.left() + 32.0, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        FontId::proportional(13.0),
        color,
    );
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

pub fn section_title(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .size(11.0)
            .color(TEXT_FAINT)
            .strong(),
    );
}

pub fn divider(ui: &mut Ui) {
    ui.add_space(2.0);
    let y = ui.cursor().top();
    let rect = egui::Rect::from_min_max(
        Pos2::new(ui.max_rect().left(), y),
        Pos2::new(ui.max_rect().right(), y + 1.0),
    );
    ui.painter().rect_filled(rect, 0, BORDER_SOFT);
    ui.add_space(2.0 + ui.spacing().item_spacing.y);
}

pub fn dot(ui: &mut Ui, color: Color32, size: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    ui.painter().circle_filled(rect.center(), size * 0.5, color);
    response
}

pub fn chip(ui: &mut Ui, text: &str, background: Color32, foreground: Color32, remove: bool) -> Response {
    let font = FontId::proportional(11.0);
    let galley = ui.fonts(|fonts| fonts.layout_no_wrap(text.to_owned(), font.clone(), foreground));
    let extra = if remove { 15.0 } else { 0.0 };
    let size = vec2(galley.size().x + 18.0 + extra, 21.0);
    let sense = if remove { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    ui.painter().rect_filled(rect, 10.0, background);
    ui.painter().galley(
        Pos2::new(rect.left() + 9.0, rect.center().y - galley.size().y * 0.5),
        galley,
        foreground,
    );
    if remove {
        let center = Pos2::new(rect.right() - 10.0, rect.center().y);
        let color = with_alpha(foreground, 190);
        let stroke = Stroke::new(1.2, color);
        ui.painter().line_segment(
            [center + vec2(-2.5, -2.5), center + vec2(2.5, 2.5)],
            stroke,
        );
        ui.painter().line_segment(
            [center + vec2(2.5, -2.5), center + vec2(-2.5, 2.5)],
            stroke,
        );
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }
    response
}

pub fn key_hint(ui: &mut Ui, text: &str) {
    let font = FontId::monospace(10.0);
    let galley = ui.fonts(|fonts| fonts.layout_no_wrap(text.to_owned(), font, TEXT_MUTED));
    let size = galley.size() + vec2(10.0, 4.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, 4.0, CARD);
    ui.painter().rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, BORDER_SOFT),
        egui::StrokeKind::Inside,
    );
    ui.painter().galley(
        rect.center() - galley.size() * 0.5,
        galley,
        TEXT_MUTED,
    );
}

pub fn empty_state(ui: &mut Ui, icon: Icon, title: &str, description: &str) {
    let available = ui.available_height();
    ui.add_space((available * 0.18).clamp(20.0, 120.0));
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 64.0), Sense::hover());
    let icon_rect = egui::Rect::from_center_size(rect.center(), Vec2::splat(38.0));
    icons::paint_icon(ui.painter(), icon_rect, icon, TEXT_FAINT);
    ui.add_space(6.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(title).size(15.0).color(TEXT_MUTED));
        ui.add_space(4.0);
        ui.label(
            RichText::new(description)
                .size(11.5)
                .color(TEXT_FAINT),
        );
    });
}

pub fn stat(ui: &mut Ui, label: &str, value: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 58.0), Sense::hover());
    ui.painter().rect_filled(rect, RADIUS, CARD);
    ui.painter().rect_stroke(
        rect,
        RADIUS,
        Stroke::new(1.0, BORDER_SOFT),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        Pos2::new(rect.left() + 12.0, rect.top() + 12.0),
        Align2::LEFT_TOP,
        value,
        FontId::proportional(20.0),
        color,
    );
    ui.painter().text(
        Pos2::new(rect.left() + 12.0, rect.bottom() - 11.0),
        Align2::LEFT_BOTTOM,
        label,
        FontId::proportional(10.5),
        TEXT_FAINT,
    );
}

pub fn color_swatch(ui: &mut Ui, color: Color32, size: f32, selected: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    if selected {
        ui.painter()
            .circle_stroke(rect.center(), size * 0.48, Stroke::new(1.5, TEXT));
    }
    ui.painter()
        .circle_filled(rect.center(), size * 0.34, color);
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

pub fn badge(ui: &mut Ui, text: &str, color: Color32) {
    let font = FontId::proportional(10.0);
    let galley = ui.fonts(|fonts| fonts.layout_no_wrap(text.to_owned(), font, color));
    let size = galley.size() + vec2(12.0, 4.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter()
        .rect_filled(rect, 8.0, with_alpha(color, 24));
    ui.painter().galley(
        rect.center() - galley.size() * 0.5,
        galley,
        color,
    );
}
