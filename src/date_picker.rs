//! Selector de fecha compacto, dibujado con egui (sin dependencias nativas).

use chrono::{Datelike, NaiveDate};
use egui::{Align2, FontId, Pos2, Sense, Stroke, Ui, Vec2, vec2};

use crate::date_utils::{self, WEEKDAYS_SHORT};
use crate::icons::{self, Icon};
use crate::theme::*;

#[derive(Clone, Debug)]
pub struct MiniCalendarState {
    pub year: i32,
    pub month: u32,
}

impl MiniCalendarState {
    pub fn from_date(date: NaiveDate) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
        }
    }
}

/// Dibuja un calendario mensual compacto. Devuelve true si cambió la fecha.
pub fn mini_calendar(ui: &mut Ui, state: &mut MiniCalendarState, selected: &mut NaiveDate) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        if icons::icon_button(ui, Icon::ChevronLeft, 24.0, TEXT_MUTED, "Mes anterior").clicked() {
            let (year, month) = date_utils::shift_month(state.year, state.month, -1);
            state.year = year;
            state.month = month;
        }
        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
            ui.label(
                egui::RichText::new(format!(
                    "{} {}",
                    date_utils::month_name(state.month),
                    state.year
                ))
                .size(13.0)
                .color(TEXT),
            );
        });
        if icons::icon_button(ui, Icon::ChevronRight, 24.0, TEXT_MUTED, "Mes siguiente").clicked() {
            let (year, month) = date_utils::shift_month(state.year, state.month, 1);
            state.year = year;
            state.month = month;
        }
    });

    ui.add_space(4.0);
    let available = ui.available_width().max(180.0);
    let gap = 2.0;
    let cell_width = ((available - gap * 6.0) / 7.0).floor().max(20.0);
    let today = date_utils::today();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for label in WEEKDAYS_SHORT {
            let (rect, _) = ui.allocate_exact_size(vec2(cell_width, 16.0), Sense::hover());
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(9.5),
                TEXT_FAINT,
            );
        }
    });

    let grid = date_utils::month_grid(state.year, state.month);
    for week in 0..6 {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for day in 0..7 {
                let date = grid[week * 7 + day];
                let in_month = date.month() == state.month && date.year() == state.year;
                let (rect, response) =
                    ui.allocate_exact_size(vec2(cell_width, 24.0), Sense::click());
                let is_selected = date == *selected;
                let is_today = date == today;

                if is_selected {
                    ui.painter()
                        .circle_filled(rect.center(), 10.5, ACCENT);
                } else if response.hovered() {
                    ui.painter()
                        .circle_filled(rect.center(), 10.5, CARD_HOVER);
                } else if is_today {
                    ui.painter()
                        .circle_stroke(rect.center(), 9.5, Stroke::new(1.0, ACCENT));
                }

                let color = if is_selected {
                    egui::Color32::WHITE
                } else if !in_month {
                    TEXT_FAINT
                } else if is_today {
                    ACCENT_HOVER
                } else {
                    TEXT
                };
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    date.day().to_string(),
                    FontId::proportional(10.5),
                    color,
                );
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if response.clicked() {
                    *selected = date;
                    state.year = date.year();
                    state.month = date.month();
                    changed = true;
                }
            }
        });
    }

    changed
}

pub fn date_button(ui: &mut Ui, date: NaiveDate) -> egui::Response {
    let label = format!("{}  ·  {}", date_utils::format_short(date), date_utils::weekday_name(date));
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width().min(300.0), 32.0),
        Sense::click(),
    );
    let bg = if response.hovered() { CARD_HOVER } else { CARD };
    ui.painter().rect_filled(rect, RADIUS_SMALL, bg);
    ui.painter().rect_stroke(
        rect,
        RADIUS_SMALL,
        Stroke::new(1.0, BORDER_SOFT),
        egui::StrokeKind::Inside,
    );
    let icon_rect = egui::Rect::from_center_size(
        Pos2::new(rect.left() + 17.0, rect.center().y),
        Vec2::splat(15.0),
    );
    icons::paint_icon(ui.painter(), icon_rect, Icon::Calendar, TEXT_MUTED);
    ui.painter().text(
        Pos2::new(rect.left() + 32.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.0),
        TEXT,
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}
