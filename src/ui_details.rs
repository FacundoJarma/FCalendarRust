//! Panel de detalles: edición directa del evento seleccionado.

use chrono::Datelike;
use egui::{
    Align, Color32, Context, Id, Layout, Margin, RichText, ScrollArea, Sense, Stroke, TextEdit, Ui,
    vec2,
};

use crate::app::{Action, FCalendarApp};
use crate::date_picker::{self, MiniCalendarState};
use crate::date_utils;
use crate::icons::{self, Icon};
use crate::model::{
    self, BG_PROPERTY, Event, PERSONAL_PROPERTY, Property, TEXT_PROPERTY,
};
use crate::theme::*;
use crate::ui_kit::{self, ButtonKind};

impl FCalendarApp {
    pub(crate) fn details_panel(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        ScrollArea::vertical()
            .id_salt("details_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.details_body(ui, actions);
            });
    }

    pub(crate) fn floating_details(&mut self, ctx: &Context) {
        if self.selected_event().is_none() {
            return;
        }

        let mut open = true;
        let mut actions = Vec::new();
        let available_height = (ctx.content_rect().height() - 110.0).max(280.0);
        egui::Window::new("Evento")
            .id(Id::unique("floating_details"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([340.0, available_height])
            .max_height(available_height)
            .default_pos(ctx.content_rect().right_top() + vec2(-370.0, 78.0))
            .frame(
                egui::Frame::new()
                    .fill(PANEL)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(RADIUS_LARGE)
                    .inner_margin(Margin::same(14))
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 10],
                        blur: 30,
                        spread: 0,
                        color: Color32::from_black_alpha(160),
                    }),
            )
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.details_body(ui, &mut actions);
                    });
            });

        self.apply_actions(actions);
        if !open {
            self.view.selected_event = None;
            self.view.editing_title = false;
            self.view.detail_picker = None;
        }
    }

    fn details_body(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        let Some(original) = self.selected_event_cloned() else {
            self.day_overview(ui, actions);
            return;
        };
        let mut event = original.clone();

        ui.horizontal(|ui| {
            ui_kit::section_title(ui, "EVENTO SELECCIONADO");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icons::icon_button(ui, Icon::Close, 20.0, TEXT_FAINT, "Cerrar detalle").clicked() {
                    actions.push(Action::ClearSelection);
                }
            });
        });
        ui.add_space(2.0);

        self.title_editor(ui, &mut event, &original);
        ui.add_space(6.0);
        self.date_editor(ui, &mut event);
        ui.add_space(8.0);

        self.tags_editor(ui, &mut event);
        ui.add_space(6.0);
        ui_kit::divider(ui);

        ui_kit::section_title(ui, "PROPIEDADES");
        ui.add_space(2.0);
        self.required_properties(ui, &mut event);
        ui.add_space(4.0);
        self.custom_properties(ui, &mut event);
        ui.add_space(6.0);

        if ui_kit::full_button(ui, "Añadir propiedad", ButtonKind::Secondary).clicked() {
            self.open_add_property(event.id);
        }
        ui.add_space(8.0);
        if ui_kit::full_button(ui, "Eliminar evento", ButtonKind::Danger).clicked() {
            actions.push(Action::DeleteEvent(event.id));
        }

        if event != original {
            self.data.update_event(event);
            self.touch();
        }
    }

    fn title_editor(&mut self, ui: &mut Ui, event: &mut Event, original: &Event) {
        if self.view.editing_title {
            let response = ui.add_sized(
                [ui.available_width(), 30.0],
                TextEdit::singleline(&mut event.title).hint_text("Título del evento"),
            );
            if self.view.focus_title {
                response.request_focus();
                self.view.focus_title = false;
            }
            if response.lost_focus() || ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                self.view.editing_title = false;
                if event.title.trim().is_empty() {
                    event.title = original.title.clone();
                }
            }
        } else {
            ui.horizontal_wrapped(|ui| {
                if icons::icon_button(ui, Icon::Pencil, 22.0, TEXT_MUTED, "Editar título").clicked() {
                    self.view.editing_title = true;
                    self.view.focus_title = true;
                }
                ui.label(
                    RichText::new(&event.title)
                        .size(19.0)
                        .strong()
                        .color(TEXT),
                );
            });
        }
    }

    fn date_editor(&mut self, ui: &mut Ui, event: &mut Event) {
        if date_picker::date_button(ui, event.date).clicked() {
            self.view.detail_picker = if self.view.detail_picker.is_some() {
                None
            } else {
                Some(MiniCalendarState::from_date(event.date))
            };
        }

        if let Some(state) = self.view.detail_picker.as_mut() {
            let mut date = event.date;
            let changed = egui::Frame::new()
                .fill(PANEL_ALT)
                .stroke(Stroke::new(1.0, BORDER_SOFT))
                .corner_radius(RADIUS)
                .inner_margin(Margin::same(8))
                .show(ui, |ui| date_picker::mini_calendar(ui, state, &mut date))
                .inner;
            if changed {
                event.date = date;
                self.view.selected_date = date;
                self.view.year = date.year();
                self.view.month = date.month();
            }
            ui.add_space(4.0);
        }
    }

    fn tags_editor(&mut self, ui: &mut Ui, event: &mut Event) {
        let categories: Vec<(String, String)> = self
            .data
            .categories
            .iter()
            .map(|(name, category)| (name.clone(), category.color.clone()))
            .collect();

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
            let mut remove = None;
            for tag in &event.tags {
                let rgb = categories
                    .iter()
                    .find(|(name, _)| name == tag)
                    .and_then(|(_, color)| model::parse_hex_color(color))
                    .unwrap_or([0x40, 0x40, 0x40]);
                if ui_kit::chip(ui, tag, color32(rgb), contrast_text(rgb), true).clicked() {
                    remove = Some(tag.clone());
                }
            }
            if let Some(tag) = remove {
                event.tags.retain(|value| value != &tag);
            }

            let available: Vec<&(String, String)> = categories
                .iter()
                .filter(|(name, _)| !event.tags.contains(name))
                .collect();
            if !available.is_empty() {
                let mut chosen = None;
                egui::ComboBox::from_id_salt("details_add_tag")
                    .width(92.0)
                    .selected_text("+ Añadir")
                    .show_ui(ui, |ui| {
                        for (name, _) in &available {
                            if ui.selectable_label(false, name.clone()).clicked() {
                                chosen = Some((*name).clone());
                            }
                        }
                    });
                if let Some(name) = chosen {
                    event.tags.push(name);
                }
            }
        });
    }

    fn required_properties(&mut self, ui: &mut Ui, event: &mut Event) {
        ui.horizontal(|ui| {
            ui.add_sized(
                [80.0, 24.0],
                egui::Label::new(RichText::new("Fondo").size(11.5).color(TEXT_MUTED)),
            );
            let mut rgb = event.background_rgb();
            if ui.color_edit_button_srgb(&mut rgb).changed() {
                event.properties.insert(
                    BG_PROPERTY.to_string(),
                    Property::Color(model::rgb_to_hex(rgb)),
                );
            }
            ui.label(
                RichText::new(model::rgb_to_hex(event.background_rgb()))
                    .size(10.0)
                    .color(TEXT_FAINT),
            );
        });

        ui.horizontal(|ui| {
            ui.add_sized(
                [80.0, 24.0],
                egui::Label::new(RichText::new("Texto").size(11.5).color(TEXT_MUTED)),
            );
            let mut rgb = event.text_rgb();
            if ui.color_edit_button_srgb(&mut rgb).changed() {
                event.properties.insert(
                    TEXT_PROPERTY.to_string(),
                    Property::Color(model::rgb_to_hex(rgb)),
                );
            }
            ui.label(
                RichText::new(model::rgb_to_hex(event.text_rgb()))
                    .size(10.0)
                    .color(TEXT_FAINT),
            );
        });

        let mut personal = event.personal_colors();
        if ui
            .checkbox(&mut personal, RichText::new("Priorizar colores propios").size(11.5))
            .on_hover_text("Ignora el estilo del shortcut y usa los colores de este evento.")
            .changed()
        {
            event.properties.insert(
                PERSONAL_PROPERTY.to_string(),
                Property::Checkbox(personal),
            );
        }

        if let Some(shortcut) = self.data.shortcut_for_event(event) {
            ui.label(
                RichText::new(format!("Estilo aplicado por «{}»", shortcut.name))
                    .size(9.5)
                    .color(TEXT_FAINT),
            );
        } else if event.personal_colors() {
            ui.label(
                RichText::new("Usando los colores propios del evento")
                    .size(9.5)
                    .color(TEXT_FAINT),
            );
        }
    }

    fn custom_properties(&mut self, ui: &mut Ui, event: &mut Event) {
        let custom: Vec<String> = event
            .properties
            .keys()
            .filter(|key| ![BG_PROPERTY, TEXT_PROPERTY, PERSONAL_PROPERTY].contains(&key.as_str()))
            .cloned()
            .collect();

        if custom.is_empty() {
            ui.label(
                RichText::new("Sin propiedades adicionales.")
                    .size(10.5)
                    .color(TEXT_FAINT),
            );
            return;
        }

        for name in custom {
            let mut remove = false;
            ui.horizontal(|ui| {
                ui.add_sized(
                    [92.0, 24.0],
                    egui::Label::new(RichText::new(&name).size(11.0).color(TEXT_MUTED)).truncate(),
                );
                if let Some(property) = event.properties.get_mut(&name) {
                    match property {
                        Property::Text(value) => {
                            ui.add_sized(
                                [(ui.available_width() - 30.0).max(80.0), 24.0],
                                TextEdit::singleline(value),
                            );
                        }
                        Property::Num(value) => {
                            ui.add(egui::DragValue::new(value).speed(0.1));
                        }
                        Property::Checkbox(value) => {
                            ui.checkbox(value, "");
                        }
                        Property::Link(value) => {
                            ui.add_sized(
                                [(ui.available_width() - 90.0).max(80.0), 24.0],
                                TextEdit::singleline(value).hint_text("https://"),
                            );
                            let url = value.trim().to_string();
                            if url.starts_with("http://") || url.starts_with("https://") {
                                ui.hyperlink_to("Abrir", url);
                            }
                        }
                        Property::Color(value) => {
                            let mut rgb = model::parse_hex_color(value).unwrap_or([0x40, 0x40, 0x40]);
                            if ui.color_edit_button_srgb(&mut rgb).changed() {
                                *value = model::rgb_to_hex(rgb);
                            }
                        }
                    }
                }
                if icons::icon_button(ui, Icon::Trash, 22.0, DANGER, "Quitar propiedad").clicked() {
                    remove = true;
                }
            });
            if remove {
                event.properties.shift_remove(&name);
            }
        }
    }

    fn day_overview(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        let date = self.view.selected_date;
        ui_kit::section_title(ui, "DÍA SELECCIONADO");
        ui.label(
            RichText::new(date_utils::format_long(date))
                .size(16.0)
                .strong()
                .color(TEXT),
        );
        ui.add_space(8.0);

        let events: Vec<Event> = self
            .data
            .all_events_on(date)
            .filter(|event| event.active)
            .cloned()
            .collect();

        if events.is_empty() {
            ui.label(
                RichText::new("No hay eventos en este día.")
                    .size(11.5)
                    .color(TEXT_FAINT),
            );
            ui.add_space(10.0);
        } else {
            for event in &events {
                let style = self.data.resolve_style(event);
                let (rect, response) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::click());
                let fill = if response.hovered() { CARD_HOVER } else { CARD };
                ui.painter().rect_filled(rect, RADIUS_SMALL, fill);
                ui.painter().rect_stroke(
                    rect,
                    RADIUS_SMALL,
                    Stroke::new(1.0, BORDER_SOFT),
                    egui::StrokeKind::Inside,
                );
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(
                        rect.min + vec2(5.0, 5.0),
                        vec2(3.0, rect.height() - 10.0),
                    ),
                    1.5,
                    color32(style.background),
                );
                let painter = ui
                    .painter()
                    .with_clip_rect(rect.shrink2(vec2(14.0, 0.0)));
                painter.text(
                    egui::Pos2::new(rect.left() + 14.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    &event.title,
                    egui::FontId::proportional(12.0),
                    if style.visible { TEXT } else { TEXT_FAINT },
                );
                if !event.tags.is_empty() {
                    painter.text(
                        egui::Pos2::new(rect.right() - 14.0, rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        event.tags.join(" · "),
                        egui::FontId::proportional(9.5),
                        TEXT_FAINT,
                    );
                }
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if response.clicked() {
                    actions.push(Action::SelectEvent(event.id));
                }
                ui.add_space(2.0);
            }
            ui.add_space(6.0);
        }

        if ui_kit::full_button(ui, "Nuevo evento", ButtonKind::Primary).clicked() {
            actions.push(Action::AddEventOn(date));
        }

        ui.add_space(16.0);
        ui_kit::divider(ui);
        ui_kit::section_title(ui, "ATAJOS");
        for (keys, description) in [
            ("← / →", "Cambiar de mes"),
            ("T", "Ir a hoy"),
            ("N", "Nuevo evento"),
            ("Ctrl+F", "Buscar"),
            ("Esc", "Cerrar detalle"),
        ] {
            ui.horizontal(|ui| {
                ui_kit::key_hint(ui, keys);
                ui.label(RichText::new(description).size(10.5).color(TEXT_FAINT));
            });
        }
    }
}
