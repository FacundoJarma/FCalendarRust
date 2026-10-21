//! Modales: alta de eventos, categorías, shortcuts, propiedades, ajustes y avisos.

use egui::{
    Align, Color32, Context, FontId, Layout, Margin, RichText, ScrollArea, Sense, Stroke, TextEdit,
    Ui, vec2,
};

use crate::app::{
    AddCategoryState, AddEventState, AddPropertyState, AddShortcutState, ConfirmAction,
    ConfirmState, DayDetailState, FCalendarApp, ModalState, SettingsState, ToastKind,
};
use crate::date_picker;
use crate::date_utils;
use crate::icons::{self, Icon};
use crate::model::{
    self, BG_PROPERTY, PERSONAL_PROPERTY, PropertyKind, TEXT_PROPERTY,
};
use crate::persistence;
use crate::theme::*;
use crate::ui_kit::{self, ButtonKind};

enum ShortcutOp {
    Move(usize, usize),
    Edit(String),
    Delete(String),
}

impl FCalendarApp {
    pub(crate) fn show_modals(&mut self, ctx: &Context) {
        if matches!(self.modal, ModalState::None) {
            return;
        }

        let mut modal = std::mem::take(&mut self.modal);
        let keep = match &mut modal {
            ModalState::None => false,
            ModalState::AddEvent(state) => self.modal_add_event(ctx, state),
            ModalState::AddCategory(state) => self.modal_add_category(ctx, state),
            ModalState::AddShortcut(state) => self.modal_add_shortcut(ctx, state),
            ModalState::ManageShortcuts => self.modal_manage_shortcuts(ctx),
            ModalState::Settings(state) => self.modal_settings(ctx, state),
            ModalState::AddProperty(state) => self.modal_add_property(ctx, state),
            ModalState::Confirm(state) => self.modal_confirm(ctx, state),
            ModalState::DayDetail(state) => self.modal_day_detail(ctx, state),
        };
        if keep {
            self.modal = modal;
        }
    }

    fn modal_add_event(&mut self, ctx: &Context, state: &mut AddEventState) -> bool {
        let categories: Vec<(String, String)> = self
            .data
            .categories
            .iter()
            .map(|(name, category)| (name.clone(), category.color.clone()))
            .collect();

        let mut submit = false;
        let mut cancel = false;
        let (_, should_close) = modal_shell(ctx, "add_event", "Nuevo evento", 390.0, |ui| {
            ui.label(RichText::new("Título").size(11.0).color(TEXT_MUTED));
            let title = ui.add_sized(
                [ui.available_width(), 30.0],
                TextEdit::singleline(&mut state.title).hint_text("Nombre del evento"),
            );
            if state.focus_title {
                title.request_focus();
                state.focus_title = false;
            }
            if title.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                submit = true;
            }

            ui.add_space(6.0);
            ui.label(RichText::new("Fecha").size(11.0).color(TEXT_MUTED));
            if date_picker::date_button(ui, state.date).clicked() {
                state.picker_open = !state.picker_open;
            }
            if state.picker_open {
                egui::Frame::new()
                    .fill(PANEL)
                    .stroke(Stroke::new(1.0, BORDER_SOFT))
                    .corner_radius(RADIUS)
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        let mut date = state.date;
                        if date_picker::mini_calendar(ui, &mut state.picker, &mut date) {
                            state.date = date;
                        }
                    });
            }

            ui.add_space(6.0);
            ui.label(RichText::new("Categorías").size(11.0).color(TEXT_MUTED));
            ScrollArea::vertical()
                .id_salt("event_categories_scroll")
                .max_height(110.0)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                        for (name, color) in &categories {
                            let selected = state.tags.iter().any(|tag| tag == name);
                            let rgb = model::parse_hex_color(color).unwrap_or([0x40, 0x40, 0x40]);
                            let (fill, text_color) = if selected {
                                (color32(rgb), contrast_text(rgb))
                            } else {
                                (CARD, TEXT_MUTED)
                            };
                            let response = ui.add(
                                egui::Button::new(RichText::new(name).size(11.0).color(text_color))
                                    .fill(fill)
                                    .stroke(Stroke::new(1.0, BORDER_SOFT))
                                    .corner_radius(10.0),
                            );
                            if response.clicked() {
                                if selected {
                                    state.tags.retain(|tag| tag != name);
                                } else {
                                    state.tags.push(name.clone());
                                }
                            }
                        }
                    });
                });

            if let Some(error) = &state.error {
                ui.add_space(4.0);
                ui.label(RichText::new(error).size(10.5).color(DANGER));
            }

            ui.add_space(10.0);
            footer(ui, "Cancelar", "Crear evento", false, &mut cancel, &mut submit);
        });

        if submit {
            let title = state.title.trim().to_string();
            if title.is_empty() {
                state.error = Some("Escribe un título para el evento.".to_string());
            } else {
                let id = self.data.add_event(title, state.date, state.tags.clone());
                self.touch();
                self.view.selected_event = Some(id);
                self.view.editing_title = false;
                self.view.detail_picker = None;
                self.go_to_date(state.date);
                self.notify(ToastKind::Success, "Evento creado.", None);
                return false;
            }
        }
        !(cancel || should_close)
    }

    fn modal_add_category(&mut self, ctx: &Context, state: &mut AddCategoryState) -> bool {
        let mut submit = false;
        let mut cancel = false;
        let (_, should_close) = modal_shell(ctx, "add_category", "Nueva categoría", 370.0, |ui| {
            ui.label(RichText::new("Nombre").size(11.0).color(TEXT_MUTED));
            let name = ui.add_sized(
                [ui.available_width(), 30.0],
                TextEdit::singleline(&mut state.name).hint_text("Ej. Trabajo"),
            );
            if state.focus_name {
                name.request_focus();
                state.focus_name = false;
            }
            if name.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                submit = true;
            }

            ui.add_space(8.0);
            ui.label(RichText::new("Color").size(11.0).color(TEXT_MUTED));
            ui.horizontal_wrapped(|ui| {
                for (label, hex) in PALETTE {
                    let selected = state.color.eq_ignore_ascii_case(hex);
                    if ui_kit::color_swatch(ui, color32_hex(hex), 26.0, selected)
                        .on_hover_text(label)
                        .clicked()
                    {
                        state.color = hex.to_string();
                        state.custom = model::parse_hex_color(hex).unwrap_or([0x4F, 0x6B, 0xE8]);
                    }
                }
            });

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Personalizado").size(11.0).color(TEXT_MUTED));
                if ui.color_edit_button_srgb(&mut state.custom).changed() {
                    state.color = model::rgb_to_hex(state.custom);
                }
                ui.label(
                    RichText::new(&state.color)
                        .size(10.5)
                        .monospace()
                        .color(TEXT_FAINT),
                );
            });

            if let Some(error) = &state.error {
                ui.add_space(4.0);
                ui.label(RichText::new(error).size(10.5).color(DANGER));
            }

            ui.add_space(10.0);
            footer(ui, "Cancelar", "Crear categoría", false, &mut cancel, &mut submit);
        });

        if submit {
            if state.name.trim().is_empty() {
                state.error = Some("Escribe un nombre para la categoría.".to_string());
            } else if !self.data.add_category(&state.name, &state.color) {
                state.error = Some("Ya existe una categoría con ese nombre.".to_string());
            } else {
                let name = state.name.trim().to_string();
                self.touch();
                self.notify(
                    ToastKind::Success,
                    format!("Categoría «{name}» creada."),
                    None,
                );
                return false;
            }
        }
        !(cancel || should_close)
    }

    fn modal_add_shortcut(&mut self, ctx: &Context, state: &mut AddShortcutState) -> bool {
        let categories: Vec<String> = self.data.categories.keys().cloned().collect();
        let mut submit = false;
        let mut cancel = false;
        let (_, should_close) = modal_shell(ctx, "add_shortcut", "Shortcut", 390.0, |ui| {
            ui.label(
                RichText::new(
                    "Aplica un estilo a todos los eventos de una categoría. Si dos coinciden, gana el primero de la lista.",
                )
                .size(10.5)
                .color(TEXT_FAINT),
            );
            ui.add_space(6.0);

            ui.label(RichText::new("Categoría").size(11.0).color(TEXT_MUTED));
            let mut choice = state.category.clone();
            egui::ComboBox::from_id_salt("shortcut_category")
                .width(ui.available_width())
                .selected_text(choice.clone().unwrap_or_else(|| "Selecciona una categoría".into()))
                .show_ui(ui, |ui| {
                    for name in &categories {
                        ui.selectable_value(&mut choice, Some(name.clone()), name);
                    }
                });
            if choice != state.category {
                state.category = choice;
                if let Some(name) = &state.category {
                    let (background, text_color) = self.shortcut_colors_for(name);
                    state.background = background;
                    state.text_color = text_color;
                }
            }

            ui.add_space(6.0);
            color_row(ui, "Fondo", &mut state.background);
            color_row(ui, "Texto", &mut state.text_color);

            ui.add_space(8.0);
            let (preview, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::hover());
            ui.painter()
                .rect_filled(preview, RADIUS_SMALL, color32(state.background));
            ui.painter().text(
                preview.center(),
                egui::Align2::CENTER_CENTER,
                state.category.as_deref().unwrap_or("Vista previa"),
                FontId::proportional(12.0),
                color32(state.text_color),
            );

            if let Some(error) = &state.error {
                ui.add_space(4.0);
                ui.label(RichText::new(error).size(10.5).color(DANGER));
            }

            ui.add_space(10.0);
            footer(ui, "Cancelar", "Guardar shortcut", false, &mut cancel, &mut submit);
        });

        if submit {
            let Some(name) = state.category.clone() else {
                state.error = Some("Selecciona una categoría.".to_string());
                return true;
            };
            if state.original.as_deref() != Some(name.as_str())
                && self.data.shortcuts.iter().any(|shortcut| shortcut.name == name)
            {
                state.error = Some("Ya existe un shortcut para esa categoría.".to_string());
                return true;
            }
            if let Some(original) = state.original.clone() {
                if original != name {
                    if let Some(shortcut) = self
                        .data
                        .shortcuts
                        .iter_mut()
                        .find(|shortcut| shortcut.name == original)
                    {
                        shortcut.name = name.clone();
                    }
                }
            }
            let background = model::rgb_to_hex(state.background);
            let text_color = model::rgb_to_hex(state.text_color);
            self.data.upsert_shortcut(&name, &background, &text_color);
            self.touch();
            self.notify(ToastKind::Success, "Shortcut guardado.", None);
            return false;
        }
        !(cancel || should_close)
    }

    fn modal_manage_shortcuts(&mut self, ctx: &Context) -> bool {
        let shortcuts = self.data.shortcuts.clone();
        let mut op = None;
        let mut add = false;
        let mut close = false;
        let (_, should_close) = modal_shell(ctx, "manage_shortcuts", "Administrar shortcuts", 420.0, |ui| {
            ui.label(
                RichText::new("El orden define la prioridad: el primero gana.")
                    .size(10.5)
                    .color(TEXT_FAINT),
            );
            ui.add_space(6.0);

            if shortcuts.is_empty() {
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Todavía no hay shortcuts.")
                        .size(11.5)
                        .color(TEXT_MUTED),
                );
                ui.add_space(8.0);
            } else {
                ScrollArea::vertical()
                    .id_salt("shortcuts_scroll")
                    .max_height(320.0)
                    .show(ui, |ui| {
                        for (index, shortcut) in shortcuts.iter().enumerate() {
                            shortcut_row(
                                ui,
                                index,
                                shortcuts.len(),
                                &shortcut.name,
                                &shortcut.background,
                                &shortcut.text_color,
                                &mut op,
                            );
                            ui.add_space(3.0);
                        }
                    });
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui_kit::action_button(ui, "Añadir shortcut", ButtonKind::Primary).clicked() {
                    add = true;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui_kit::action_button(ui, "Cerrar", ButtonKind::Ghost).clicked() {
                        close = true;
                    }
                });
            });
        });

        if add {
            self.open_add_shortcut();
            return false;
        }
        if let Some(op) = op {
            match op {
                ShortcutOp::Move(from, to) => {
                    self.data.move_shortcut(from, to);
                    self.touch();
                }
                ShortcutOp::Edit(name) => {
                    self.open_edit_shortcut(&name);
                    return false;
                }
                ShortcutOp::Delete(name) => {
                    self.open_confirm(
                        "Eliminar shortcut",
                        format!(
                            "Se eliminará el shortcut «{name}». Los eventos conservarán sus colores personales."
                        ),
                        "Eliminar",
                        true,
                        ConfirmAction::DeleteShortcut(name),
                    );
                    return false;
                }
            }
        }
        !(close || should_close)
    }

    fn modal_settings(&mut self, ctx: &Context, state: &mut SettingsState) -> bool {
        let mut close = false;
        let (_, should_close) = modal_shell(ctx, "settings", "Ajustes", 470.0, |ui| {
            ScrollArea::vertical()
                .id_salt("settings_scroll")
                .max_height(520.0)
                .show(ui, |ui| {
                    ui_kit::section_title(ui, "PERFIL");
                    let profile = ui.add_sized(
                        [ui.available_width(), 30.0],
                        TextEdit::singleline(&mut self.data.profile_name).hint_text("Nombre del calendario"),
                    );
                    if profile.changed() {
                        self.touch();
                    }

                    ui.add_space(10.0);
                    ui_kit::section_title(ui, "DATOS");
                    ui.label(
                        RichText::new(self.data_path_label())
                            .size(10.0)
                            .monospace()
                            .color(TEXT_FAINT),
                    )
                    .on_hover_text("Archivo local donde se guarda el calendario");
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Guardado automático con copias diarias.")
                            .size(10.0)
                            .color(TEXT_FAINT),
                    );
                    ui.add_space(6.0);

                    ui.horizontal_wrapped(|ui| {
                        if ui_kit::action_button(ui, "Abrir carpeta", ButtonKind::Secondary).clicked() {
                            match persistence::open_data_folder(&self.paths) {
                                Ok(()) => state.notice = Some(("Carpeta abierta.".into(), true)),
                                Err(error) => state.notice = Some((error, false)),
                            }
                        }
                        if ui_kit::action_button(ui, "Exportar…", ButtonKind::Secondary).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Exportar calendario")
                                .add_filter("FCalendar", &["json"])
                                .set_file_name("fcalendar.json")
                                .save_file()
                            {
                                match persistence::export_to(&path, &self.data) {
                                    Ok(()) => {
                                        state.notice = Some((
                                            format!("Exportado en {}", path.display()),
                                            true,
                                        ));
                                    }
                                    Err(error) => {
                                        state.notice =
                                            Some((format!("No se pudo exportar: {error}"), false));
                                    }
                                }
                            }
                        }
                        if ui_kit::action_button(ui, "Importar…", ButtonKind::Secondary).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Importar calendario")
                                .add_filter("FCalendar", &["json"])
                                .pick_file()
                            {
                                self.open_confirm(
                                    "Importar datos",
                                    format!(
                                        "Se reemplazarán los datos actuales por el contenido de «{}». Esta acción no se puede deshacer.",
                                        path.display()
                                    ),
                                    "Importar",
                                    true,
                                    ConfirmAction::ImportData(path),
                                );
                                close = true;
                            }
                        }
                    });

                    ui.add_space(10.0);
                    ui.horizontal_wrapped(|ui| {
                        ui_kit::badge(
                            ui,
                            &format!("{} activos", self.data.active_event_count()),
                            GREEN,
                        );
                        ui_kit::badge(
                            ui,
                            &format!("{} en papelera", self.data.deleted_event_count()),
                            TEXT_FAINT,
                        );
                        ui_kit::badge(
                            ui,
                            &format!("{} categorías", self.data.categories.len()),
                            ACCENT_HOVER,
                        );
                        ui_kit::badge(
                            ui,
                            &format!("{} shortcuts", self.data.shortcuts.len()),
                            GOLD,
                        );
                    });

                    ui.add_space(12.0);
                    ui_kit::section_title(ui, "MANTENIMIENTO");
                    ui.horizontal_wrapped(|ui| {
                        let deleted = self.data.deleted_event_count();
                        if ui_kit::action_button(ui, "Vaciar papelera", ButtonKind::Danger).clicked()
                        {
                            if deleted == 0 {
                                state.notice = Some(("La papelera ya está vacía.".into(), true));
                            } else {
                                self.open_confirm(
                                    "Vaciar papelera",
                                    format!(
                                        "Se eliminarán definitivamente {deleted} evento(s). Esta acción no se puede deshacer."
                                    ),
                                    "Vaciar",
                                    true,
                                    ConfirmAction::PurgeDeleted,
                                );
                                close = true;
                            }
                        }
                        if ui_kit::action_button(ui, "Borrar todos los eventos", ButtonKind::Danger)
                            .clicked()
                        {
                            self.open_confirm(
                                "Borrar todos los eventos",
                                "Se eliminarán todos los eventos del calendario. Las categorías y los shortcuts se conservarán. Esta acción no se puede deshacer.",
                                "Borrar todo",
                                true,
                                ConfirmAction::ClearEvents,
                            );
                            close = true;
                        }
                        if ui_kit::action_button(ui, "Restablecer todo", ButtonKind::Danger).clicked()
                        {
                            self.open_confirm(
                                "Restablecer FCalendar",
                                "Se borrarán todos los eventos, categorías y shortcuts, y se restaurarán los valores iniciales. Esta acción no se puede deshacer.",
                                "Restablecer",
                                true,
                                ConfirmAction::ResetAll,
                            );
                            close = true;
                        }
                    });

                    if let Some((message, ok)) = &state.notice {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(message)
                                .size(10.5)
                                .color(if *ok { GREEN } else { DANGER }),
                        );
                    }
                });

            ui.add_space(10.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui_kit::action_button(ui, "Cerrar", ButtonKind::Primary).clicked() {
                    close = true;
                }
            });
        });

        !(close || should_close)
    }

    fn modal_add_property(&mut self, ctx: &Context, state: &mut AddPropertyState) -> bool {
        let mut submit = false;
        let mut cancel = false;
        let (_, should_close) = modal_shell(ctx, "add_property", "Nueva propiedad", 390.0, |ui| {
            ui.label(RichText::new("Nombre").size(11.0).color(TEXT_MUTED));
            let name = ui.add_sized(
                [ui.available_width(), 30.0],
                TextEdit::singleline(&mut state.name).hint_text("Ej. Horario, Enlace, Precio"),
            );
            if state.focus_name {
                name.request_focus();
                state.focus_name = false;
            }

            ui.add_space(6.0);
            ui.label(RichText::new("Tipo").size(11.0).color(TEXT_MUTED));
            egui::ComboBox::from_id_salt("property_kind")
                .width(ui.available_width())
                .selected_text(state.kind.label())
                .show_ui(ui, |ui| {
                    for kind in PropertyKind::ALL {
                        ui.selectable_value(&mut state.kind, kind, kind.label());
                    }
                });

            ui.add_space(6.0);
            ui.label(RichText::new("Valor inicial").size(11.0).color(TEXT_MUTED));
            match state.kind {
                PropertyKind::Text | PropertyKind::Link => {
                    let hint = if state.kind == PropertyKind::Link {
                        "https://"
                    } else {
                        "Valor"
                    };
                    ui.add_sized(
                        [ui.available_width(), 30.0],
                        TextEdit::singleline(&mut state.text).hint_text(hint),
                    );
                }
                PropertyKind::Num => {
                    ui.add(egui::DragValue::new(&mut state.number).speed(0.1));
                }
                PropertyKind::Checkbox => {
                    ui.checkbox(&mut state.checked, "Marcada por defecto");
                }
                PropertyKind::Color => {
                    color_row(ui, "Color", &mut state.color);
                }
            }

            if let Some(error) = &state.error {
                ui.add_space(4.0);
                ui.label(RichText::new(error).size(10.5).color(DANGER));
            }

            ui.add_space(10.0);
            footer(ui, "Cancelar", "Añadir", false, &mut cancel, &mut submit);
        });

        if submit {
            let name = state.name.trim().to_string();
            let reserved = [BG_PROPERTY, TEXT_PROPERTY, PERSONAL_PROPERTY];
            if name.is_empty() {
                state.error = Some("Escribe un nombre para la propiedad.".to_string());
            } else if reserved.contains(&name.as_str())
                || self
                    .data
                    .event(state.event_id)
                    .is_some_and(|event| event.properties.contains_key(&name))
            {
                state.error = Some("Ya existe una propiedad con ese nombre.".to_string());
            } else {
                let property = match state.kind {
                    PropertyKind::Text => model::Property::Text(state.text.clone()),
                    PropertyKind::Num => model::Property::Num(state.number),
                    PropertyKind::Checkbox => model::Property::Checkbox(state.checked),
                    PropertyKind::Link => model::Property::Link(state.text.clone()),
                    PropertyKind::Color => {
                        model::Property::Color(model::rgb_to_hex(state.color))
                    }
                };
                if let Some(event) = self.data.event_mut(state.event_id) {
                    event.properties.insert(name, property);
                    self.touch();
                    self.notify(ToastKind::Success, "Propiedad añadida.", None);
                }
                return false;
            }
        }
        !(cancel || should_close)
    }

    fn modal_confirm(&mut self, ctx: &Context, state: &ConfirmState) -> bool {
        let mut confirm = false;
        let mut cancel = false;
        let (_, should_close) = modal_shell(ctx, "confirm", &state.title, 400.0, |ui| {
            ui.label(
                RichText::new(&state.message)
                    .size(12.0)
                    .color(TEXT_MUTED),
            );
            ui.add_space(12.0);
            footer(
                ui,
                "Cancelar",
                &state.confirm_label,
                state.danger,
                &mut cancel,
                &mut confirm,
            );
        });

        if confirm {
            self.apply_confirm(state.action.clone());
            return false;
        }
        !(cancel || should_close)
    }

    fn modal_day_detail(&mut self, ctx: &Context, state: &DayDetailState) -> bool {
        let events: Vec<model::Event> = self
            .data
            .all_events_on(state.date)
            .filter(|event| event.active)
            .cloned()
            .collect();
        let mut select = None;
        let mut add = false;
        let mut close = false;

        let (_, should_close) = modal_shell(ctx, "day_detail", "Eventos del día", 420.0, |ui| {
            ui.label(
                RichText::new(date_utils::format_long(state.date))
                    .size(11.5)
                    .color(TEXT_MUTED),
            );
            ui.add_space(8.0);

            if events.is_empty() {
                ui.label(
                    RichText::new("No hay eventos en este día.")
                        .size(11.5)
                        .color(TEXT_FAINT),
                );
                ui.add_space(8.0);
            } else {
                ScrollArea::vertical()
                    .id_salt("day_detail_events")
                    .max_height(320.0)
                    .show(ui, |ui| {
                        for event in &events {
                            let style = self.data.resolve_style(event);
                            let mut label = event.title.clone();
                            if !event.tags.is_empty() {
                                label.push_str("   ·   ");
                                label.push_str(&event.tags.join(" · "));
                            }
                            let response = ui.add_sized(
                                [ui.available_width(), 32.0],
                                egui::Button::new(
                                    RichText::new(label)
                                        .size(11.5)
                                        .color(color32(style.text)),
                                )
                                .fill(color32(style.background))
                                .stroke(Stroke::new(1.0, BORDER_SOFT))
                                .corner_radius(RADIUS_SMALL),
                            );
                            if !style.visible {
                                ui.label(
                                    RichText::new("Una categoría de este evento está oculta")
                                        .size(9.5)
                                        .color(DANGER),
                                );
                            }
                            if response.clicked() {
                                select = Some(event.id);
                            }
                            ui.add_space(3.0);
                        }
                    });
                ui.add_space(6.0);
            }

            ui.horizontal(|ui| {
                if ui_kit::action_button(ui, "Nuevo evento", ButtonKind::Primary).clicked() {
                    add = true;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui_kit::action_button(ui, "Cerrar", ButtonKind::Ghost).clicked() {
                        close = true;
                    }
                });
            });
        });

        if add {
            self.open_add_event(state.date);
            return false;
        }
        if let Some(id) = select {
            self.view.selected_event = Some(id);
            self.view.editing_title = false;
            self.view.detail_picker = None;
            self.go_to_date(state.date);
            return false;
        }
        !(close || should_close)
    }

    pub(crate) fn show_toast(&mut self, ctx: &Context) {
        let Some(toast) = self.toast.as_ref() else {
            return;
        };
        let text = toast.text.clone();
        let action = toast.action;
        let color = match toast.kind {
            ToastKind::Info => ACCENT_HOVER,
            ToastKind::Success => GREEN,
            ToastKind::Error => DANGER,
        };

        let mut run_action = None;
        let mut close = false;
        egui::Area::new(egui::Id::unique("toast"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::RIGHT_BOTTOM, vec2(-18.0, -18.0))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(PANEL_ALT)
                    .stroke(Stroke::new(1.0, color))
                    .corner_radius(RADIUS)
                    .inner_margin(Margin::symmetric(12, 9))
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 8],
                        blur: 24,
                        spread: 0,
                        color: Color32::from_black_alpha(140),
                    })
                    .show(ui, |ui| {
                        ui.set_max_width(400.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&text).size(11.5).color(TEXT));
                            if action.is_some()
                                && ui_kit::action_button(ui, "Deshacer", ButtonKind::Secondary)
                                    .clicked()
                            {
                                run_action = action;
                            }
                            if icons::icon_button(ui, Icon::Close, 20.0, TEXT_FAINT, "Cerrar aviso")
                                .clicked()
                            {
                                close = true;
                            }
                        });
                    });
            });

        if let Some(action) = run_action {
            self.toast = None;
            self.apply_toast_action(action);
        } else if close {
            self.toast = None;
        }
    }
}

fn modal_shell<R>(
    ctx: &Context,
    id: &str,
    title: &str,
    width: f32,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> (R, bool) {
    let response = egui::Modal::new(egui::Id::unique(id))
        .frame(
            egui::Frame::new()
                .fill(PANEL_ALT)
                .stroke(Stroke::new(1.0, BORDER))
                .corner_radius(RADIUS_LARGE)
                .inner_margin(Margin::same(18))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 12],
                    blur: 32,
                    spread: 0,
                    color: Color32::from_black_alpha(170),
                }),
        )
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.label(RichText::new(title).size(18.0).strong().color(TEXT));
            ui.add_space(8.0);
            add_contents(ui)
        });
    let should_close = response.should_close();
    (response.inner, should_close)
}

fn footer(
    ui: &mut Ui,
    cancel_label: &str,
    confirm_label: &str,
    danger: bool,
    cancel: &mut bool,
    confirm: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let kind = if danger {
                ButtonKind::Danger
            } else {
                ButtonKind::Primary
            };
            let confirm_response = ui_kit::action_button(ui, confirm_label, kind);
            if confirm_response.clicked() {
                *confirm = true;
            }
            let cancel_response = ui_kit::action_button(ui, cancel_label, ButtonKind::Ghost);
            if cancel_response.clicked() {
                *cancel = true;
            }
        });
    });
}

fn color_row(ui: &mut Ui, label: &str, color: &mut [u8; 3]) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [70.0, 24.0],
            egui::Label::new(RichText::new(label).size(11.5).color(TEXT_MUTED)),
        );
        ui.color_edit_button_srgb(color);
        ui.label(
            RichText::new(model::rgb_to_hex(*color))
                .size(10.5)
                .monospace()
                .color(TEXT_FAINT),
        );
    });
}

fn shortcut_row(
    ui: &mut Ui,
    index: usize,
    total: usize,
    name: &str,
    background: &str,
    text_color: &str,
    op: &mut Option<ShortcutOp>,
) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
    ui.painter().rect_filled(rect, RADIUS_SMALL, CARD);
    let inner = rect.shrink2(vec2(8.0, 2.0));
    let mut row = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::left_to_right(Align::Center)),
    );

    let grip_rect = egui::Rect::from_center_size(
        egui::Pos2::new(inner.left() + 7.0, inner.center().y),
        egui::Vec2::splat(14.0),
    );
    icons::paint_icon(row.painter(), grip_rect, Icon::Grip, TEXT_FAINT);
    row.add_space(18.0);

    let background_rgb = model::parse_hex_color(background).unwrap_or([0x40, 0x40, 0x40]);
    let text_rgb = model::parse_hex_color(text_color).unwrap_or([0xFF, 0xFF, 0xFF]);
    ui_kit::chip(&mut row, name, color32(background_rgb), color32(text_rgb), false);

    row.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if icons::icon_button(ui, Icon::ArrowDown, 22.0, TEXT_MUTED, "Bajar prioridad").clicked()
            && index + 1 < total
        {
            *op = Some(ShortcutOp::Move(index, index + 1));
        }
        if icons::icon_button(ui, Icon::ArrowUp, 22.0, TEXT_MUTED, "Subir prioridad").clicked()
            && index > 0
        {
            *op = Some(ShortcutOp::Move(index, index - 1));
        }
        if icons::icon_button(ui, Icon::Pencil, 22.0, TEXT_MUTED, "Editar shortcut").clicked() {
            *op = Some(ShortcutOp::Edit(name.to_string()));
        }
        if icons::icon_button(ui, Icon::Trash, 22.0, DANGER, "Eliminar shortcut").clicked() {
            *op = Some(ShortcutOp::Delete(name.to_string()));
        }
    });
}
