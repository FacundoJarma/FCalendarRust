//! Vista mensual dibujada a medida para mantener 60 fps y un diseño propio.

use chrono::Datelike;
use egui::{
    Align, Align2, Button, Color32, FontId, Layout, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2,
    vec2,
};

use crate::app::{Action, FCalendarApp};
use crate::date_utils::{self, WEEKDAYS_SHORT};
use crate::icons::{self, Icon};
use crate::model::Event;
use crate::theme::*;

impl FCalendarApp {
    pub(crate) fn calendar_area(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        let month_count = self.data.events_in_month(self.view.year, self.view.month);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(date_utils::month_name(self.view.month))
                        .size(26.0)
                        .strong()
                        .color(TEXT),
                );
                ui.label(
                    RichText::new(format!(
                        "{}  ·  {} evento{} este mes",
                        self.view.year,
                        month_count,
                        if month_count == 1 { "" } else { "s" }
                    ))
                    .size(10.5)
                    .color(TEXT_FAINT),
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if nav_button(ui, Icon::ChevronRight, "Mes siguiente (→)").clicked() {
                    actions.push(Action::NextMonth);
                }
                if nav_button(ui, Icon::ChevronLeft, "Mes anterior (←)").clicked() {
                    actions.push(Action::PrevMonth);
                }
                let today = ui.add_sized(
                    [62.0, 32.0],
                    Button::new(RichText::new("Hoy").size(11.5).color(TEXT))
                        .fill(CARD)
                        .stroke(Stroke::new(1.0, BORDER_SOFT))
                        .corner_radius(8),
                );
                if today.clicked() {
                    actions.push(Action::Today);
                }
                ui.add_space(4.0);
                if ui.input(|input| input.modifiers.command) {
                    ui.label(RichText::new("Ctrl+F para buscar").size(9.5).color(TEXT_FAINT));
                }
            });
        });
        ui.add_space(8.0);

        let width = ui.available_width();
        let (weekday_rect, _) = ui.allocate_exact_size(vec2(width, 24.0), Sense::hover());
        let cell_width = weekday_rect.width() / 7.0;
        for (index, label) in WEEKDAYS_SHORT.iter().enumerate() {
            let center = Pos2::new(
                weekday_rect.left() + cell_width * (index as f32 + 0.5),
                weekday_rect.center().y,
            );
            ui.painter().text(
                center,
                Align2::CENTER_CENTER,
                *label,
                FontId::proportional(10.0),
                TEXT_FAINT,
            );
        }

        let height = ui.available_height().max(260.0);
        let (grid_rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let cell_size = vec2(grid_rect.width() / 7.0, grid_rect.height() / 6.0);
        let grid = date_utils::month_grid(self.view.year, self.view.month);
        let today = date_utils::today();
        let pointer = ui.ctx().input(|input| input.pointer.hover_pos());

        for (index, date) in grid.iter().enumerate() {
            let row = index / 7;
            let column = index % 7;
            let cell_rect = Rect::from_min_size(
                grid_rect.min + vec2(cell_size.x * column as f32, cell_size.y * row as f32),
                cell_size,
            );
            let in_month = date.month() == self.view.month && date.year() == self.view.year;
            let cell_id = ui.id().with(("calendar_day", *date));
            let cell_response = ui.interact(cell_rect, cell_id, Sense::click());
            let hovered = cell_response.hovered()
                || pointer.is_some_and(|position| cell_rect.contains(position));
            if hovered {
                self.view.hovered_day = Some(*date);
            }

            let base = if !in_month {
                Color32::from_rgb(0x19, 0x19, 0x1D)
            } else if date_utils::is_weekend(*date) {
                Color32::from_rgb(0x1E, 0x1E, 0x23)
            } else {
                BG
            };
            let fill = if hovered {
                Color32::from_rgb(0x27, 0x27, 0x2E)
            } else if self.view.selected_date == *date {
                Color32::from_rgb(0x25, 0x25, 0x2C)
            } else {
                base
            };
            ui.painter().rect_filled(cell_rect, 0.0, fill);

            let border = Stroke::new(1.0, BORDER_SOFT);
            if column < 6 {
                ui.painter().line_segment(
                    [cell_rect.right_top(), cell_rect.right_bottom()],
                    border,
                );
            }
            if row < 5 {
                ui.painter().line_segment(
                    [cell_rect.left_bottom(), cell_rect.right_bottom()],
                    border,
                );
            }

            let day_color = if *date == today {
                Color32::WHITE
            } else if !in_month {
                TEXT_FAINT
            } else if hovered {
                ACCENT_HOVER
            } else {
                TEXT_MUTED
            };
            let day_center = Pos2::new(cell_rect.left() + 19.0, cell_rect.top() + 17.0);
            if *date == today {
                ui.painter().circle_filled(day_center, 10.5, TODAY);
            }
            ui.painter().text(
                day_center,
                Align2::CENTER_CENTER,
                date.day().to_string(),
                FontId::proportional(11.0),
                day_color,
            );

            let mut consumed = false;
            if hovered && cell_rect.width() > 54.0 {
                let add_rect = Rect::from_center_size(
                    Pos2::new(cell_rect.right() - 15.0, cell_rect.top() + 16.0),
                    Vec2::splat(19.0),
                );
                let add_response = ui.interact(add_rect, cell_id.with("add"), Sense::click());
                if add_response.hovered() {
                    ui.painter().circle_filled(add_rect.center(), 9.5, CARD_HOVER);
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                } else {
                    ui.painter().circle_filled(add_rect.center(), 9.5, CARD);
                }
                icons::paint_icon(
                    ui.painter(),
                    add_rect.shrink(5.5),
                    Icon::Plus,
                    if add_response.hovered() { TEXT } else { TEXT_MUTED },
                );
                if add_response.clicked() {
                    actions.push(Action::AddEventOn(*date));
                    consumed = true;
                }
            }

            let events: Vec<&Event> = self.data.events_on(*date).collect();
            let slot_height = 17.0;
            let slot_gap = 3.0;
            let events_top = cell_rect.top() + 28.0;
            let events_bottom = cell_rect.bottom() - 5.0;
            let available = (events_bottom - events_top).max(slot_height);
            let max_slots = (((available + slot_gap) / (slot_height + slot_gap)).floor() as usize).max(1);
            let show_more = events.len() > max_slots;
            let shown = if show_more {
                max_slots.saturating_sub(1).max(1)
            } else {
                events.len().min(max_slots)
            };

            for (slot, event) in events.iter().take(shown).enumerate() {
                let chip_rect = Rect::from_min_size(
                    Pos2::new(cell_rect.left() + 5.0, events_top + slot as f32 * (slot_height + slot_gap)),
                    vec2((cell_rect.width() - 10.0).max(20.0), slot_height),
                );
                let event_id = cell_id.with(("event", event.id));
                let response = ui.interact(chip_rect, event_id, Sense::click());
                let style = self.data.resolve_style(event);
                let mut background = color32(style.background);
                if !style.visible {
                    background = with_alpha(background, 34);
                } else if response.hovered() {
                    background = lighten(background, 0.12);
                }
                let rounding = egui::CornerRadius::same(4);
                ui.painter().rect_filled(chip_rect, rounding, background);

                if let Some(tag) = event.tags.first() {
                    if let Some(category) = self.data.categories.get(tag) {
                        let stripe = Rect::from_min_size(
                            chip_rect.min + vec2(2.0, 2.0),
                            vec2(2.5, chip_rect.height() - 4.0),
                        );
                        ui.painter().rect_filled(
                            stripe,
                            1.0,
                            with_alpha(color32_hex(&category.color), if style.visible { 230 } else { 70 }),
                        );
                    }
                }

                if style.focused {
                    ui.painter().rect_stroke(
                        chip_rect.expand(1.0),
                        rounding,
                        Stroke::new(1.0, Color32::from_white_alpha(180)),
                        egui::StrokeKind::Outside,
                    );
                }
                if self.view.selected_event == Some(event.id) {
                    ui.painter().rect_stroke(
                        chip_rect,
                        rounding,
                        Stroke::new(1.5, Color32::WHITE),
                        egui::StrokeKind::Inside,
                    );
                } else if response.hovered() {
                    ui.painter().rect_stroke(
                        chip_rect,
                        rounding,
                        Stroke::new(1.0, Color32::from_white_alpha(70)),
                        egui::StrokeKind::Inside,
                    );
                }

                let mut text_color = color32(style.text);
                if !style.visible {
                    text_color = with_alpha(text_color, 105);
                }
                let text_rect = chip_rect.shrink2(vec2(7.0, 0.0));
                let painter = ui.painter().with_clip_rect(text_rect);
                let max_chars = ((text_rect.width() / 5.7).floor() as usize).max(3);
                painter.text(
                    Pos2::new(text_rect.left(), text_rect.center().y),
                    Align2::LEFT_CENTER,
                    truncate(&event.title, max_chars),
                    FontId::proportional(9.8),
                    text_color,
                );

                let response = response.on_hover_ui(|ui| {
                    ui.set_max_width(240.0);
                    ui.label(RichText::new(&event.title).strong().color(TEXT));
                    ui.label(
                        RichText::new(format!(
                            "{} · {}",
                            date_utils::format_short(event.date),
                            if event.tags.is_empty() {
                                "sin categoría".to_string()
                            } else {
                                event.tags.join(", ")
                            }
                        ))
                        .size(10.0)
                        .color(TEXT_FAINT),
                    );
                    if !style.visible {
                        ui.label(
                            RichText::new("Atenuado: una categoría está oculta")
                                .size(10.0)
                                .color(DANGER),
                        );
                    }
                });
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if response.clicked() {
                    actions.push(Action::SelectEvent(event.id));
                    consumed = true;
                }
            }

            if show_more {
                let more_top = events_top + shown as f32 * (slot_height + slot_gap);
                let more_rect = Rect::from_min_size(
                    Pos2::new(cell_rect.left() + 7.0, more_top.min(events_bottom - 11.0)),
                    vec2((cell_rect.width() - 14.0).max(20.0), 13.0),
                );
                let response = ui.interact(
                    more_rect,
                    cell_id.with("more"),
                    Sense::click(),
                );
                let remaining = events.len().saturating_sub(shown);
                ui.painter().text(
                    Pos2::new(more_rect.left(), more_rect.center().y),
                    Align2::LEFT_CENTER,
                    format!("+{remaining} más"),
                    FontId::proportional(9.2),
                    if response.hovered() { ACCENT_HOVER } else { TEXT_FAINT },
                );
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if response.clicked() {
                    actions.push(Action::OpenDay(*date));
                    consumed = true;
                }
            }

            if self.view.selected_date == *date {
                ui.painter().rect_stroke(
                    cell_rect.shrink(0.5),
                    0.0,
                    Stroke::new(1.0, ACCENT),
                    egui::StrokeKind::Inside,
                );
            }

            if !consumed {
                if cell_response.double_clicked() {
                    actions.push(Action::AddEventOn(*date));
                } else if cell_response.clicked() {
                    actions.push(Action::SelectDate(*date));
                }
            }
        }

        ui.painter().rect_stroke(
            grid_rect,
            0.0,
            Stroke::new(1.0, BORDER),
            egui::StrokeKind::Inside,
        );
    }
}

fn nav_button(ui: &mut Ui, icon: Icon, tooltip: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::click());
    let center = rect.center();
    if response.hovered() {
        ui.painter().circle_filled(center, 16.5, CARD_HOVER);
    } else {
        ui.painter().circle_filled(center, 16.5, CARD);
    }
    icons::paint_icon(
        ui.painter(),
        Rect::from_center_size(center, Vec2::splat(16.0)),
        icon,
        if response.hovered() { TEXT } else { TEXT_MUTED },
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.on_hover_text(tooltip)
}

fn truncate(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }
    let mut result: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    result.push('…');
    result
}
