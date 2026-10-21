//! Barra superior y buscador.

use chrono::Datelike;
use egui::{
    Align, Align2, Button, Color32, Context, FontId, Layout, Margin, Pos2, RichText, Sense, Stroke,
    TextEdit, Ui, vec2,
};

use crate::app::{Action, FCalendarApp};
use crate::date_utils;
use crate::icons::{self, Icon};
use crate::theme::*;

impl FCalendarApp {
    pub(crate) fn top_bar(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        let width = ui.ctx().content_rect().width();
        ui.horizontal(|ui| {
            if icons::icon_button(ui, Icon::Menu, 30.0, TEXT_MUTED, "Mostrar u ocultar el panel lateral")
                .clicked()
            {
                actions.push(Action::ToggleSidebar);
            }
            icons::brand_mark(ui, 28.0);
            ui.add_space(2.0);
            ui.label(RichText::new("FCalendar").size(16.0).strong().color(TEXT));
            if width >= 1240.0 {
                ui.label(RichText::new("Calendario personal").size(11.0).color(TEXT_FAINT));
            }

            ui.add_space(14.0);
            if icons::icon_button(ui, Icon::Search, 26.0, TEXT_FAINT, "Buscar (Ctrl+F)").clicked() {
                self.search.focus_requested = true;
            }
            let field_width = if width >= 1200.0 { 250.0 } else { 200.0 };
            let field = ui.add_sized(
                [field_width, 30.0],
                TextEdit::singleline(&mut self.search.query)
                    .hint_text("Buscar eventos…")
                    .margin(Margin::symmetric(8, 5))
                    .frame(
                        egui::Frame::new()
                            .fill(CARD)
                            .stroke(Stroke::new(1.0, BORDER_SOFT))
                            .corner_radius(8),
                    ),
            );
            self.search.field_rect = Some(field.rect);
            if self.search.focus_requested {
                field.request_focus();
                self.search.focus_requested = false;
            }
            if !self.search.query.is_empty()
                && icons::icon_button(ui, Icon::Close, 22.0, TEXT_FAINT, "Limpiar búsqueda").clicked()
            {
                self.search.query.clear();
                self.search.focus_requested = true;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icons::icon_button(ui, Icon::Gear, 30.0, TEXT_MUTED, "Ajustes").clicked() {
                    actions.push(Action::OpenSettings);
                }
                ui.add_space(2.0);
                let today_button = ui.add_sized(
                    [58.0, 30.0],
                    Button::new(RichText::new("Hoy").size(12.0).color(TEXT))
                        .fill(CARD)
                        .stroke(Stroke::new(1.0, BORDER_SOFT))
                        .corner_radius(7),
                );
                if today_button.clicked() {
                    actions.push(Action::Today);
                }
                if width >= 1160.0 {
                    let add = ui.add_sized(
                        [118.0, 30.0],
                        Button::new(RichText::new("Nuevo evento").size(12.0).color(Color32::WHITE))
                            .fill(ACCENT)
                            .stroke(Stroke::new(1.0, ACCENT))
                            .corner_radius(7),
                    );
                    if add.clicked() {
                        actions.push(Action::AddEventOn(self.view.selected_date));
                    }
                } else if icons::icon_button_with_bg(
                    ui,
                    Icon::Plus,
                    30.0,
                    Color32::WHITE,
                    Some(ACCENT),
                    "Nuevo evento (N)",
                )
                .clicked()
                {
                    actions.push(Action::AddEventOn(self.view.selected_date));
                }

                if width >= 1000.0 {
                    ui.add_space(4.0);
                    self.saved_indicator(ui);
                }
            });
        });
    }

    fn saved_indicator(&self, ui: &mut Ui) {
        if let Some(error) = &self.saved.error {
            ui.label(RichText::new("● Error al guardar").size(10.5).color(DANGER))
                .on_hover_text(error);
            return;
        }
        if self.saved.dirty {
            ui.label(RichText::new("● Guardando…").size(10.5).color(TEXT_FAINT));
            return;
        }
        if let Some(saved) = self.saved.last_saved {
            if saved.elapsed().as_secs() < 4 {
                ui.label(RichText::new("● Guardado").size(10.5).color(GREEN))
                    .on_hover_text(format!("Guardado en {}", self.data_path_label()));
            } else {
                ui.label(RichText::new("● Local").size(10.5).color(TEXT_FAINT))
                    .on_hover_text("Todos los datos están en tu equipo. Guardado automático activo.");
            }
        } else {
            ui.label(RichText::new("● Local").size(10.5).color(TEXT_FAINT))
                .on_hover_text("Todos los datos están en tu equipo. Guardado automático activo.");
        }
    }

    pub(crate) fn search_popup(&mut self, ctx: &Context) {
        let query = self.search.query.trim().to_string();
        if query.is_empty() {
            self.search.popup_rect = None;
            return;
        }
        let Some(field_rect) = self.search.field_rect else {
            self.search.popup_rect = None;
            return;
        };
        let results = self.data.search(&query);
        let mut clicked_event = None;
        let screen = ctx.content_rect();
        let mut position = field_rect.left_bottom() + vec2(0.0, 7.0);
        position.x = position.x.min((screen.right() - 390.0).max(12.0));
        position.y = position.y.min((screen.bottom() - 120.0).max(12.0));

        let response = egui::Area::new(egui::Id::unique("search_results"))
            .order(egui::Order::Foreground)
            .fixed_pos(position)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(PANEL_ALT)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(RADIUS)
                    .inner_margin(Margin::same(7))
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 8],
                        blur: 24,
                        spread: 0,
                        color: Color32::from_black_alpha(120),
                    })
                    .show(ui, |ui| {
                        ui.set_width(374.0);
                        ui.label(
                            RichText::new(format!("{} resultado(s)", results.len()))
                                .size(10.0)
                                .color(TEXT_FAINT),
                        );
                        ui.add_space(3.0);
                        if results.is_empty() {
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new("No hay eventos que coincidan.")
                                    .size(12.0)
                                    .color(TEXT_MUTED),
                            );
                            ui.add_space(6.0);
                        } else {
                            for event in results.iter().take(8) {
                                let (rect, response) =
                                    ui.allocate_exact_size(vec2(ui.available_width(), 42.0), Sense::click());
                                let bg = if response.hovered() {
                                    CARD_HOVER
                                } else {
                                    Color32::TRANSPARENT
                                };
                                ui.painter().rect_filled(rect, 7.0, bg);

                                let badge = egui::Rect::from_min_size(
                                    rect.min + vec2(8.0, 7.0),
                                    vec2(50.0, 28.0),
                                );
                                ui.painter().rect_filled(badge, 6.0, CARD);
                                ui.painter().text(
                                    badge.center(),
                                    Align2::CENTER_CENTER,
                                    format!("{} {}", event.date.day(), date_utils::month_name_short(event.date.month())),
                                    FontId::proportional(10.5),
                                    TEXT,
                                );
                                let text_rect = egui::Rect::from_min_max(
                                    Pos2::new(badge.right() + 10.0, rect.top() + 5.0),
                                    Pos2::new(rect.right() - 8.0, rect.bottom() - 4.0),
                                );
                                let painter = ui.painter().with_clip_rect(text_rect);
                                painter.text(
                                    Pos2::new(text_rect.left(), text_rect.top()),
                                    Align2::LEFT_TOP,
                                    truncate(&event.title, 44),
                                    FontId::proportional(12.0),
                                    TEXT,
                                );
                                let subtitle = if event.tags.is_empty() {
                                    date_utils::format_short(event.date)
                                } else {
                                    event.tags.join(" · ")
                                };
                                painter.text(
                                    Pos2::new(text_rect.left(), text_rect.top() + 17.0),
                                    Align2::LEFT_TOP,
                                    truncate(&subtitle, 52),
                                    FontId::proportional(10.0),
                                    TEXT_FAINT,
                                );
                                if response.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if response.clicked() {
                                    clicked_event = Some(event.id);
                                }
                            }
                        }
                        ui.add_space(3.0);
                        ui.label(
                            RichText::new("Haz clic en un resultado para abrirlo")
                                .size(9.5)
                                .color(TEXT_FAINT),
                        );
                    });
            });
        self.search.popup_rect = Some(response.response.rect);

        let pointer = ctx.input(|input| input.pointer.interact_pos());
        let clicked_elsewhere = ctx.input(|input| input.pointer.any_click())
            && pointer.is_some_and(|position| {
                !field_rect.contains(position)
                    && !response.response.rect.contains(position)
            });
        if clicked_elsewhere {
            self.search.query.clear();
            self.search.popup_rect = None;
        }

        if let Some(id) = clicked_event {
            self.search.query.clear();
            self.search.popup_rect = None;
            self.apply_actions(vec![Action::SelectEvent(id)]);
            ctx.request_repaint();
        }
    }
}

fn truncate(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }
    let mut truncated: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    truncated.push('…');
    truncated
}

