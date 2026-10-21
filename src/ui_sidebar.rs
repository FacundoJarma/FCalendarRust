//! Panel lateral: perfil, categorías y acciones principales.

use egui::{Align, Align2, FontId, Label, Layout, Pos2, RichText, ScrollArea, Sense, Stroke, Ui, Vec2, vec2};

use crate::app::{Action, FCalendarApp};
use crate::icons::{self, Icon};
use crate::model::Category;
use crate::theme::*;
use crate::ui_kit::{self, ButtonKind};

impl FCalendarApp {
    pub(crate) fn sidebar(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        self.profile_card(ui, actions);
        ui.add_space(10.0);
        ui_kit::divider(ui);

        ui.horizontal(|ui| {
            ui_kit::section_title(ui, "CATEGORÍAS");
            ui_kit::badge(
                ui,
                &self.data.categories.len().to_string(),
                ACCENT_HOVER,
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let icon = if self.view.categories_open {
                    Icon::ChevronRight
                } else {
                    Icon::ArrowDown
                };
                if icons::icon_button(
                    ui,
                    icon,
                    20.0,
                    TEXT_FAINT,
                    if self.view.categories_open {
                        "Contraer categorías"
                    } else {
                        "Expandir categorías"
                    },
                )
                .clicked()
                {
                    self.view.categories_open = !self.view.categories_open;
                }
            });
        });

        if self.view.categories_open {
            let max_height = (ui.available_height() - 252.0).max(96.0);
            ScrollArea::vertical()
                .id_salt("categories_scroll")
                .max_height(max_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    for (name, category) in &self.data.categories {
                        category_row(ui, name, category, actions);
                    }
                });
        }

        ui.add_space(4.0);
        ui_kit::divider(ui);

        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.label(
                RichText::new("Datos locales · guardado automático")
                    .size(9.5)
                    .color(TEXT_FAINT),
            )
            .on_hover_text(self.data_path_label());
            
            ui.add_space(2.0);
            if ui_kit::icon_text_button(ui, Icon::Grip, "Administrar shortcuts", ButtonKind::Ghost)
                .clicked()
            {
                actions.push(Action::OpenManageShortcuts);
            }
            if ui_kit::icon_text_button(ui, Icon::Tag, "Añadir shortcut", ButtonKind::Secondary)
                .clicked()
            {
                actions.push(Action::OpenAddShortcut);
            }
            if ui_kit::icon_text_button(ui, Icon::Star, "Añadir categoría", ButtonKind::Secondary)
                .clicked()
            {
                actions.push(Action::OpenAddCategory);
            }
            if ui_kit::icon_text_button(ui, Icon::Plus, "Añadir evento", ButtonKind::Primary)
                .clicked()
            {
                actions.push(Action::AddEventOn(self.view.selected_date));
            }
            ui.add_space(2.0);
        });
    }

    fn profile_card(&self, ui: &mut Ui, actions: &mut Vec<Action>) {
        let (rect, response) = ui.allocate_exact_size(
            vec2(ui.available_width(), 54.0),
            Sense::click(),
        );
        let bg = if response.hovered() { CARD_HOVER } else { CARD };
        ui.painter().rect_filled(rect, RADIUS, bg);
        ui.painter().rect_stroke(
            rect,
            RADIUS,
            Stroke::new(1.0, BORDER_SOFT),
            egui::StrokeKind::Inside,
        );

        let avatar_center = Pos2::new(rect.left() + 26.0, rect.center().y);
        ui.painter()
            .circle_filled(avatar_center, 16.0, ACCENT_DARK);
        let initial = self
            .data
            .profile_name
            .chars()
            .find(|character| !character.is_whitespace())
            .unwrap_or('F')
            .to_uppercase()
            .to_string();
        ui.painter().text(
            avatar_center,
            Align2::CENTER_CENTER,
            initial,
            FontId::proportional(14.0),
            egui::Color32::WHITE,
        );

        let text_left = rect.left() + 50.0;
        ui.painter().text(
            Pos2::new(text_left, rect.center().y - 8.0),
            Align2::LEFT_CENTER,
            truncate_name(&self.data.profile_name, 24),
            FontId::proportional(13.0),
            TEXT,
        );
        ui.painter().text(
            Pos2::new(text_left, rect.center().y + 9.0),
            Align2::LEFT_CENTER,
            "Calendario local",
            FontId::proportional(9.5),
            TEXT_FAINT,
        );
        let gear_rect = egui::Rect::from_center_size(
            Pos2::new(rect.right() - 20.0, rect.center().y),
            Vec2::splat(16.0),
        );
        icons::paint_icon(ui.painter(), gear_rect, Icon::Gear, TEXT_FAINT);
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if response.clicked() {
            actions.push(Action::OpenSettings);
        }
        response.on_hover_text("Perfil y ajustes de la cuenta local");
    }
}

fn category_row(ui: &mut Ui, name: &str, category: &Category, actions: &mut Vec<Action>) {
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), 29.0),
        Sense::hover(),
    );
    if response.hovered() {
        ui.painter().rect_filled(rect, 6.0, CARD);
    }
    if category.focused {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(rect.min + vec2(0.0, 5.0), vec2(2.0, rect.height() - 10.0)),
            1.0,
            GOLD,
        );
    }
    let color = color32_hex(&category.color);
    let dot_color = if category.active { color } else { with_alpha(color, 90) };
    let inner = rect.shrink2(vec2(8.0, 2.0));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::left_to_right(Align::Center)),
    );
    let (dot_rect, _) = child.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
    child
        .painter()
        .circle_filled(dot_rect.center(), 4.0, dot_color);

    let name_width = (child.available_width() - 78.0).max(30.0);
    let label = child.add_sized(
        [name_width, 20.0],
        Label::new(
            RichText::new(name)
                .size(12.0)
                .color(if category.active { TEXT } else { TEXT_FAINT }),
        )
        .truncate(),
    );
    if category.focused {
        label.on_hover_text("Esta categoría está enfocada");
    }

    child.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if icons::icon_button(
            ui,
            Icon::Trash,
            22.0,
            with_alpha(DANGER, 180),
            "Eliminar categoría",
        )
        .clicked()
        {
            actions.push(Action::RequestDeleteCategory(name.to_string()));
        }
        let (eye_icon, eye_color, eye_tip) = if category.active {
            (Icon::Eye, TEXT_FAINT, "Ocultar eventos de esta categoría")
        } else {
            (Icon::EyeOff, DANGER, "Mostrar eventos de esta categoría")
        };
        if icons::icon_button(ui, eye_icon, 22.0, eye_color, eye_tip).clicked() {
            actions.push(Action::ToggleCategoryActive(name.to_string()));
        }
        if icons::icon_toggle(
            ui,
            Icon::Star,
            22.0,
            category.focused,
            GOLD,
            "Enfocar esta categoría",
        )
        .clicked()
        {
            actions.push(Action::ToggleCategoryFocus(name.to_string()));
        }
    });
}

fn truncate_name(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }
    let mut result: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    result.push('…');
    result
}
