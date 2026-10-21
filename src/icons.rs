//! Iconos vectoriales dibujados con el painter de egui.
//!
//! Evita depender de una fuente de iconos y mantiene el mismo aspecto en
//! cualquier distribución de Linux.

use egui::{Color32, CursorIcon, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, vec2};

use crate::theme::{ACCENT, CARD_HOVER, TEXT};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Plus,
    ChevronLeft,
    ChevronRight,
    Eye,
    EyeOff,
    Star,
    Trash,
    Search,
    Gear,
    Close,
    Pencil,
    Calendar,
    Check,
    Grip,
    ArrowUp,
    ArrowDown,
    Folder,
    Download,
    Upload,
    Menu,
    Tag,
    Clock,
    Inbox,
}

pub fn icon_button(ui: &mut Ui, icon: Icon, size: f32, tint: Color32, tooltip: &str) -> Response {
    icon_button_with_bg(ui, icon, size, tint, None, tooltip)
}

pub fn icon_button_with_bg(
    ui: &mut Ui,
    icon: Icon,
    size: f32,
    tint: Color32,
    background: Option<Color32>,
    tooltip: &str,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let hover = response.hovered() || response.highlighted();
    let bg = background.or_else(|| hover.then_some(CARD_HOVER));
    if let Some(bg) = bg {
        ui.painter()
            .circle_filled(rect.center(), size * 0.5, bg);
    }
    let icon_rect = rect.shrink(size * 0.23);
    paint_icon(ui.painter(), icon_rect, icon, tint);
    if hover {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    if tooltip.is_empty() {
        response
    } else {
        response.on_hover_text(tooltip)
    }
}

pub fn icon_toggle(
    ui: &mut Ui,
    icon: Icon,
    size: f32,
    active: bool,
    active_color: Color32,
    tooltip: &str,
) -> Response {
    let tint = if active { active_color } else { crate::theme::TEXT_FAINT };
    let bg = active.then_some(crate::theme::with_alpha(active_color, 26));
    icon_button_with_bg(ui, icon, size, tint, bg, tooltip)
}

pub fn paint_icon(painter: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    let stroke = Stroke::new((rect.width() * 0.085).clamp(1.2, 2.0), color);
    let point = |x: f32, y: f32| -> Pos2 {
        Pos2::new(
            rect.left() + x / 24.0 * rect.width(),
            rect.top() + y / 24.0 * rect.height(),
        )
    };
    let line = |points: &[(f32, f32)]| {
        painter.add(Shape::line(
            points.iter().map(|(x, y)| point(*x, *y)).collect(),
            stroke,
        ));
    };
    let closed = |points: &[(f32, f32)]| {
        let mut points: Vec<Pos2> = points.iter().map(|(x, y)| point(*x, *y)).collect();
        if let Some(first) = points.first().copied() {
            points.push(first);
        }
        painter.add(Shape::line(points, stroke));
    };
    let dot = |x: f32, y: f32, radius: f32| {
        painter.circle_filled(point(x, y), radius / 24.0 * rect.width(), color);
    };

    match icon {
        Icon::Plus => {
            line(&[(12.0, 5.0), (12.0, 19.0)]);
            line(&[(5.0, 12.0), (19.0, 12.0)]);
        }
        Icon::ChevronLeft => line(&[(14.5, 5.5), (8.0, 12.0), (14.5, 18.5)]),
        Icon::ChevronRight => line(&[(9.5, 5.5), (16.0, 12.0), (9.5, 18.5)]),
        Icon::Eye => {
            painter.circle_stroke(point(12.0, 12.0), rect.width() * 0.29, stroke);
            painter.circle_filled(point(12.0, 12.0), rect.width() * 0.11, color);
            line(&[(4.0, 12.0), (7.0, 8.0)]);
            line(&[(20.0, 12.0), (17.0, 8.0)]);
            line(&[(4.0, 12.0), (7.0, 16.0)]);
            line(&[(20.0, 12.0), (17.0, 16.0)]);
        }
        Icon::EyeOff => {
            painter.circle_stroke(point(12.0, 12.0), rect.width() * 0.28, stroke);
            painter.circle_filled(point(12.0, 12.0), rect.width() * 0.10, color);
            line(&[(5.0, 19.0), (19.0, 5.0)]);
        }
        Icon::Star => {
            let mut points = Vec::new();
            for index in 0..10 {
                let angle = -std::f32::consts::FRAC_PI_2
                    + index as f32 * std::f32::consts::PI / 5.0;
                let radius = if index % 2 == 0 { 8.8 } else { 3.9 };
                points.push((
                    12.0 + radius * angle.cos(),
                    12.0 + radius * angle.sin(),
                ));
            }
            let mut screen_points: Vec<Pos2> =
                points.iter().map(|(x, y)| point(*x, *y)).collect();
            if let Some(first) = screen_points.first().copied() {
                screen_points.push(first);
            }
            painter.add(Shape::line(screen_points, stroke));
        }
        Icon::Trash => {
            line(&[(5.0, 7.0), (19.0, 7.0)]);
            line(&[(9.0, 7.0), (9.0, 4.5), (15.0, 4.5), (15.0, 7.0)]);
            closed(&[(7.0, 8.0), (17.0, 8.0), (16.0, 20.0), (8.0, 20.0)]);
            line(&[(10.5, 11.0), (10.5, 17.0)]);
            line(&[(13.5, 11.0), (13.5, 17.0)]);
        }
        Icon::Search => {
            painter.circle_stroke(point(10.5, 10.5), rect.width() * 0.26, stroke);
            line(&[(15.0, 15.0), (20.0, 20.0)]);
        }
        Icon::Gear => {
            painter.circle_stroke(point(12.0, 12.0), rect.width() * 0.20, stroke);
            for index in 0..8 {
                let angle = index as f32 * std::f32::consts::TAU / 8.0;
                line(&[
                    (12.0 + 6.6 * angle.cos(), 12.0 + 6.6 * angle.sin()),
                    (12.0 + 9.0 * angle.cos(), 12.0 + 9.0 * angle.sin()),
                ]);
            }
        }
        Icon::Close => {
            line(&[(6.0, 6.0), (18.0, 18.0)]);
            line(&[(18.0, 6.0), (6.0, 18.0)]);
        }
        Icon::Pencil => {
            line(&[(5.0, 19.0), (8.0, 18.0), (19.0, 7.0), (17.0, 5.0), (6.0, 16.0), (5.0, 19.0)]);
            line(&[(15.5, 6.5), (17.5, 8.5)]);
        }
        Icon::Calendar => {
            painter.rect_stroke(
                Rect::from_min_max(point(4.0, 6.0), point(20.0, 20.0)),
                egui::CornerRadius::same(2),
                stroke,
                egui::StrokeKind::Middle,
            );
            line(&[(4.0, 10.0), (20.0, 10.0)]);
            line(&[(8.0, 3.5), (8.0, 7.5)]);
            line(&[(16.0, 3.5), (16.0, 7.5)]);
        }
        Icon::Check => line(&[(5.0, 12.5), (10.0, 17.0), (19.0, 7.0)]),
        Icon::Grip => {
            for row in 0..3 {
                for column in 0..2 {
                    dot(9.0 + column as f32 * 5.0, 8.0 + row as f32 * 4.0, 1.15);
                }
            }
        }
        Icon::ArrowUp => line(&[(6.0, 14.0), (12.0, 8.0), (18.0, 14.0)]),
        Icon::ArrowDown => line(&[(6.0, 10.0), (12.0, 16.0), (18.0, 10.0)]),
        Icon::Folder => {
            closed(&[(4.0, 7.0), (4.0, 19.0), (20.0, 19.0), (20.0, 9.0), (12.0, 9.0), (10.0, 6.0)]);
        }
        Icon::Download => {
            line(&[(12.0, 4.0), (12.0, 15.0)]);
            line(&[(7.0, 10.0), (12.0, 15.0), (17.0, 10.0)]);
            line(&[(5.0, 19.0), (19.0, 19.0)]);
        }
        Icon::Upload => {
            line(&[(12.0, 15.0), (12.0, 4.0)]);
            line(&[(7.0, 9.0), (12.0, 4.0), (17.0, 9.0)]);
            line(&[(5.0, 19.0), (19.0, 19.0)]);
        }
        Icon::Menu => {
            line(&[(4.0, 7.0), (20.0, 7.0)]);
            line(&[(4.0, 12.0), (20.0, 12.0)]);
            line(&[(4.0, 17.0), (20.0, 17.0)]);
        }
        Icon::Tag => {
            closed(&[(4.0, 5.0), (12.0, 5.0), (20.0, 13.0), (13.0, 20.0), (4.0, 11.0)]);
            dot(8.0, 9.0, 1.3);
        }
        Icon::Clock => {
            painter.circle_stroke(point(12.0, 12.0), rect.width() * 0.34, stroke);
            line(&[(12.0, 7.0), (12.0, 12.0), (15.5, 14.0)]);
        }
        Icon::Inbox => {
            closed(&[(4.0, 5.0), (20.0, 5.0), (20.0, 19.0), (4.0, 19.0)]);
            line(&[(4.0, 13.0), (9.0, 13.0), (10.5, 16.0), (13.5, 16.0), (15.0, 13.0), (20.0, 13.0)]);
        }
    }
}

pub fn brand_mark(ui: &mut Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    let painter = ui.painter();
    let rounding = egui::CornerRadius::same((size * 0.22) as u8);
    painter.rect_filled(rect, rounding, crate::theme::PANEL_ALT);
    painter.rect_filled(
        Rect::from_min_max(rect.min + vec2(0.0, 0.0), Pos2::new(rect.right(), rect.top() + size * 0.34)),
        rounding,
        ACCENT,
    );
    let body = Rect::from_min_max(
        rect.min + vec2(size * 0.20, size * 0.30),
        rect.max - vec2(size * 0.20, size * 0.16),
    );
    painter.rect_filled(body, egui::CornerRadius::same((size * 0.08) as u8), Color32::from_rgb(0xF2, 0xF2, 0xF7));
    for (index, color) in [ACCENT, crate::theme::GOLD, crate::theme::GREEN]
        .into_iter()
        .enumerate()
    {
        let y = body.top() + size * (0.16 + index as f32 * 0.17);
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(body.left() + size * 0.10, y),
                Pos2::new(body.right() - size * 0.20, y + size * 0.07),
            ),
            egui::CornerRadius::same(2),
            color,
        );
    }
    let _ = TEXT;
}

pub fn small_spinner(ui: &mut Ui, size: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let painter = ui.painter();
    let center = rect.center();
    let radius = size * 0.32;
    for index in 0..8 {
        let angle = index as f32 * std::f32::consts::TAU / 8.0;
        let alpha = 40 + index * 25;
        let color = crate::theme::with_alpha(crate::theme::ACCENT, alpha.min(255) as u8);
        painter.circle_filled(
            center + vec2(radius * angle.cos(), radius * angle.sin()),
            size * 0.045,
            color,
        );
    }
    response
}
