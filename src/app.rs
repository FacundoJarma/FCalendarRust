mod date_picker;
mod date_utils;
mod icons;
mod model;
mod persistence;
mod theme;
mod ui_calendar;
mod ui_details;
mod ui_kit;
mod ui_modals;
mod ui_sidebar;
mod ui_topbar;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use chrono::{Datelike, NaiveDate};
use eframe::egui;
use egui::{Context, Id, Ui};

use crate::date_picker::MiniCalendarState;
use crate::date_utils::today;
use crate::model::{CalendarData, Event, PropertyKind};
use crate::persistence::DataPaths;

const AUTOSAVE_DELAY: Duration = Duration::from_millis(420);
const DETAILS_BREAKPOINT: f32 = 1180.0;
const SIDEBAR_BREAKPOINT: f32 = 1050.0;

pub struct FCalendarApp {
    pub data: CalendarData,
    pub paths: DataPaths,
    pub view: ViewState,
    pub saved: SavedState,
    pub modal: ModalState,
    pub search: SearchState,
    pub toast: Option<Toast>,
}

pub struct ViewState {
    pub year: i32,
    pub month: u32,
    pub selected_date: NaiveDate,
    pub selected_event: Option<u64>,
    pub sidebar_open: bool,
    pub sidebar_user_toggled: bool,
    pub editing_title: bool,
    pub focus_title: bool,
    pub detail_picker: Option<MiniCalendarState>,
    pub categories_open: bool,
    pub hovered_day: Option<NaiveDate>,
}

pub struct SavedState {
    pub dirty: bool,
    pub last_edit: Option<Instant>,
    pub last_saved: Option<Instant>,
    pub error: Option<String>,
}

impl SavedState {
    fn new() -> Self {
        Self {
            dirty: false,
            last_edit: None,
            last_saved: None,
            error: None,
        }
    }
}

#[derive(Default)]
pub struct SearchState {
    pub query: String,
    pub field_rect: Option<egui::Rect>,
    pub popup_rect: Option<egui::Rect>,
    pub focus_requested: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastAction {
    UndoDeleteEvent(u64),
}

pub struct Toast {
    pub kind: ToastKind,
    pub text: String,
    pub action: Option<ToastAction>,
    pub created: Instant,
    pub duration: Duration,
}

pub enum ModalState {
    None,
    AddEvent(AddEventState),
    AddCategory(AddCategoryState),
    AddShortcut(AddShortcutState),
    ManageShortcuts(ManageShortcutsState),
    Settings(SettingsState),
    AddProperty(AddPropertyState),
    Confirm(ConfirmState),
    DayDetail(DayDetailState),
}

impl Default for ModalState {
    fn default() -> Self {
        Self::None
    }
}

pub struct AddEventState {
    pub title: String,
    pub date: NaiveDate,
    pub tags: Vec<String>,
    pub picker: MiniCalendarState,
    pub picker_open: bool,
    pub error: Option<String>,
    pub focus_title: bool,
}

pub struct AddCategoryState {
    pub name: String,
    pub color: String,
    pub custom: [u8; 3],
    pub error: Option<String>,
    pub focus_name: bool,
}

pub struct AddShortcutState {
    pub category: Option<String>,
    pub background: [u8; 3],
    pub text_color: [u8; 3],
    pub error: Option<String>,
}

pub struct ManageShortcutsState {
    pub dragging: Option<usize>,
    pub drag_target: Option<usize>,
}

pub struct SettingsState {
    pub notice: Option<(String, bool)>,
}

pub struct AddPropertyState {
    pub event_id: u64,
    pub name: String,
    pub kind: PropertyKind,
    pub text: String,
    pub number: f64,
    pub checked: bool,
    pub color: [u8; 3],
    pub error: Option<String>,
    pub focus_name: bool,
}

#[derive(Clone, Debug)]
pub enum ConfirmAction {
    DeleteCategory(String),
    DeleteShortcut(String),
    PurgeDeleted,
    ClearEvents,
    ResetAll,
    ImportData(PathBuf),
}

pub struct ConfirmState {
    pub title: String,
    pub message: String,
    pub confirm_label: String,
    pub danger: bool,
    pub action: ConfirmAction,
}

pub struct DayDetailState {
    pub date: NaiveDate,
}

pub enum Action {
    PrevMonth,
    NextMonth,
    Today,
    SelectDate(NaiveDate),
    SelectEvent(u64),
    ClearSelection,
    AddEventOn(NaiveDate),
    OpenAddCategory,
    OpenAddShortcut,
    OpenManageShortcuts,
    OpenSettings,
    ToggleSidebar,
    ToggleCategoryActive(String),
    ToggleCategoryFocus(String),
    RequestDeleteCategory(String),
    OpenDay(NaiveDate),
    DeleteEvent(u64),
}

impl FCalendarApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        paths: DataPaths,
        mut data: CalendarData,
        notice: Option<String>,
    ) -> Self {
        theme::install(&cc.egui_ctx);
        data.normalize();
        let now = today();
        let mut app = Self {
            data,
            paths,
            view: ViewState {
                year: now.year(),
                month: now.month(),
                selected_date: now,
                selected_event: None,
                sidebar_open: true,
                sidebar_user_toggled: false,
                editing_title: false,
                focus_title: false,
                detail_picker: None,
                categories_open: true,
                hovered_day: None,
            },
            saved: SavedState::new(),
            modal: ModalState::None,
            search: SearchState::default(),
            toast: None,
        };
        if let Some(notice) = notice {
            app.notify(ToastKind::Error, notice, None);
        }
        app
    }

    pub fn selected_event(&self) -> Option<&Event> {
        self.view
            .selected_event
            .and_then(|id| self.data.event(id))
            .filter(|event| event.active)
    }

    pub fn selected_event_cloned(&self) -> Option<Event> {
        self.selected_event().cloned()
    }

    pub fn touch(&mut self) {
        self.saved.dirty = true;
        self.saved.last_edit = Some(Instant::now());
        self.saved.error = None;
    }

    pub fn notify(&mut self, kind: ToastKind, text: impl Into<String>, action: Option<ToastAction>) {
        let duration = match kind {
            ToastKind::Error => Duration::from_secs(9),
            ToastKind::Success => Duration::from_secs(4),
            ToastKind::Info => Duration::from_secs(6),
        };
        self.toast = Some(Toast {
            kind,
            text: text.into(),
            action,
            created: Instant::now(),
            duration,
        });
    }

    pub fn go_to_date(&mut self, date: NaiveDate) {
        self.view.year = date.year();
        self.view.month = date.month();
        self.view.selected_date = date;
    }

    fn update_layout(&mut self, ctx: &Context) {
        let width = ctx.content_rect().width();
        if !self.view.sidebar_user_toggled {
            self.view.sidebar_open = width >= SIDEBAR_BREAKPOINT;
        }
    }

    fn handle_shortcuts(&mut self, ctx: &Context) {
        if !matches!(self.modal, ModalState::None) {
            return;
        }
        if ctx.wants_keyboard_input() || egui::Popup::is_any_open(ctx) {
            return;
        }

        let (left, right, go_today, new_event, search) = ctx.input_mut(|input| {
            (
                input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft),
                input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight),
                input.consume_key(egui::Modifiers::NONE, egui::Key::T),
                input.consume_key(egui::Modifiers::NONE, egui::Key::N),
                input.consume_key(egui::Modifiers::COMMAND, egui::Key::F)
                    || input.consume_key(egui::Modifiers::NONE, egui::Key::Slash),
            )
        });

        if left {
            let (year, month) = date_utils::shift_month(self.view.year, self.view.month, -1);
            self.view.year = year;
            self.view.month = month;
        }
        if right {
            let (year, month) = date_utils::shift_month(self.view.year, self.view.month, 1);
            self.view.year = year;
            self.view.month = month;
        }
        if go_today {
            self.go_to_date(today());
        }
        if new_event {
            self.open_add_event(self.view.selected_date);
        }
        if search {
            self.search.focus_requested = true;
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.view.selected_event = None;
            self.view.editing_title = false;
            self.view.detail_picker = None;
            self.search.query.clear();
        }
    }

    fn apply_actions(&mut self, actions: Vec<Action>) {
        for action in actions {
            match action {
                Action::PrevMonth => {
                    let (year, month) = date_utils::shift_month(self.view.year, self.view.month, -1);
                    self.view.year = year;
                    self.view.month = month;
                }
                Action::NextMonth => {
                    let (year, month) = date_utils::shift_month(self.view.year, self.view.month, 1);
                    self.view.year = year;
                    self.view.month = month;
                }
                Action::Today => self.go_to_date(today()),
                Action::SelectDate(date) => {
                    self.view.selected_date = date;
                    if date.month() != self.view.month || date.year() != self.view.year {
                        self.view.year = date.year();
                        self.view.month = date.month();
                    }
                }
                Action::SelectEvent(id) => {
                    if let Some(event) = self.data.event(id).filter(|event| event.active) {
                        let date = event.date;
                        self.view.selected_event = Some(id);
                        self.view.editing_title = false;
                        self.view.detail_picker = None;
                        self.go_to_date(date);
                    }
                }
                Action::ClearSelection => {
                    self.view.selected_event = None;
                    self.view.editing_title = false;
                    self.view.detail_picker = None;
                }
                Action::AddEventOn(date) => self.open_add_event(date),
                Action::OpenAddCategory => self.open_add_category(),
                Action::OpenAddShortcut => self.open_add_shortcut(),
                Action::OpenManageShortcuts => {
                    self.modal = ModalState::ManageShortcuts(ManageShortcutsState {
                        dragging: None,
                        drag_target: None,
                    });
                }
                Action::OpenSettings => {
                    self.modal = ModalState::Settings(SettingsState { notice: None });
                }
                Action::ToggleSidebar => {
                    self.view.sidebar_open = !self.view.sidebar_open;
                    self.view.sidebar_user_toggled = true;
                }
                Action::ToggleCategoryActive(name) => {
                    if let Some(category) = self.data.categories.get(&name) {
                        let active = !category.active;
                        self.data.set_category_active(&name, active);
                        self.touch();
                    }
                }
                Action::ToggleCategoryFocus(name) => {
                    if let Some(category) = self.data.categories.get(&name) {
                        let focused = !category.focused;
                        self.data.set_category_focused(&name, focused);
                        self.touch();
                    }
                }
                Action::RequestDeleteCategory(name) => {
                    self.modal = ModalState::Confirm(ConfirmState {
                        title: "Eliminar categoría".to_string(),
                        message: format!(
                            "Se eliminará «{name}» y se quitará de todos los eventos que la usan. Esta acción no se puede deshacer."
                        ),
                        confirm_label: "Eliminar".to_string(),
                        danger: true,
                        action: ConfirmAction::DeleteCategory(name),
                    });
                }
                Action::OpenDay(date) => {
                    self.modal = ModalState::DayDetail(DayDetailState { date });
                }
                Action::DeleteEvent(id) => self.delete_event_with_undo(id),
            }
        }
    }

    pub fn open_add_event(&mut self, date: NaiveDate) {
        self.view.selected_date = date;
        self.modal = ModalState::AddEvent(AddEventState {
            title: String::new(),
            date,
            tags: Vec::new(),
            picker: MiniCalendarState::from_date(date),
            picker_open: false,
            error: None,
            focus_title: true,
        });
    }

    pub fn open_add_category(&mut self) {
        self.modal = ModalState::AddCategory(AddCategoryState {
            name: String::new(),
            color: theme::PALETTE[1].1.to_string(),
            custom: [0x4F, 0x6B, 0xE8],
            error: None,
            focus_name: true,
        });
    }

    pub fn open_add_shortcut(&mut self) {
        let category = self
            .data
            .categories
            .keys()
            .next()
            .cloned();
        let (background, text_color) = category
            .as_ref()
            .map(|name| self.shortcut_colors_for(name))
            .unwrap_or(([0x40, 0x40, 0x40], [0xFF, 0xFF, 0xFF]));
        self.modal = ModalState::AddShortcut(AddShortcutState {
            category,
            background,
            text_color,
            error: None,
        });
    }

    pub fn open_edit_shortcut(&mut self, name: &str) {
        let (background, text_color) = self.shortcut_colors_for(name);
        self.modal = ModalState::AddShortcut(AddShortcutState {
            category: Some(name.to_string()),
            background,
            text_color,
            error: None,
        });
    }

    pub fn shortcut_colors_for(&self, category: &str) -> ([u8; 3], [u8; 3]) {
        if let Some(shortcut) = self
            .data
            .shortcuts
            .iter()
            .find(|shortcut| shortcut.name == category)
        {
            (
                model::parse_hex_color(&shortcut.background).unwrap_or([0x40, 0x40, 0x40]),
                model::parse_hex_color(&shortcut.text_color).unwrap_or([0xFF, 0xFF, 0xFF]),
            )
        } else {
            (
                self.data
                    .categories
                    .get(category)
                    .and_then(|category| model::parse_hex_color(&category.color))
                    .unwrap_or([0x40, 0x40, 0x40]),
                [0xFF, 0xFF, 0xFF],
            )
        }
    }

    pub fn delete_event_with_undo(&mut self, id: u64) {
        let Some(event) = self.data.event(id).cloned() else {
            return;
        };
        if self.data.delete_event(id) {
            self.touch();
            self.view.selected_event = None;
            self.view.editing_title = false;
            self.notify(
                ToastKind::Info,
                format!("«{}» se movió a la papelera.", event.title),
                Some(ToastAction::UndoDeleteEvent(id)),
            );
        }
    }

    fn apply_toast_action(&mut self, action: ToastAction) {
        match action {
            ToastAction::UndoDeleteEvent(id) => {
                if self.data.restore_event(id) {
                    self.touch();
                    if let Some(event) = self.data.event(id) {
                        let date = event.date;
                        self.view.selected_event = Some(id);
                        self.go_to_date(date);
                    }
                    self.notify(ToastKind::Success, "Evento restaurado.", None);
                }
            }
        }
    }

    pub fn apply_confirm(&mut self, action: ConfirmAction) {
        match action {
            ConfirmAction::DeleteCategory(name) => {
                self.data.delete_category(&name);
                self.touch();
                self.notify(ToastKind::Success, format!("Categoría «{name}» eliminada."), None);
            }
            ConfirmAction::DeleteShortcut(name) => {
                self.data.remove_shortcut(&name);
                self.touch();
                self.notify(ToastKind::Success, "Shortcut eliminado.", None);
            }
            ConfirmAction::PurgeDeleted => {
                let count = self.data.deleted_event_count();
                self.data.purge_deleted_events();
                self.touch();
                self.notify(
                    ToastKind::Success,
                    format!("{count} evento(s) eliminados definitivamente."),
                    None,
                );
            }
            ConfirmAction::ClearEvents => {
                self.data.events.clear();
                self.data.next_id = 1;
                self.data.rebuild_indexes();
                self.view.selected_event = None;
                self.touch();
                self.notify(ToastKind::Success, "Todos los eventos fueron eliminados.", None);
            }
            ConfirmAction::ResetAll => {
                let paths = self.paths.clone();
                self.data = CalendarData::default();
                self.view.selected_event = None;
                self.view.editing_title = false;
                let now = today();
                self.go_to_date(now);
                self.touch();
                if let Err(error) = persistence::save(&paths, &self.data) {
                    self.saved.error = Some(error);
                } else {
                    self.saved.dirty = false;
                    self.saved.last_saved = Some(Instant::now());
                }
                self.notify(ToastKind::Success, "FCalendar fue restablecido.", None);
            }
            ConfirmAction::ImportData(path) => {
                match persistence::import_from(&path) {
                    Ok(data) => {
                        self.data = data;
                        self.view.selected_event = None;
                        self.view.editing_title = false;
                        self.view.detail_picker = None;
                        self.go_to_date(today());
                        self.touch();
                        if let Err(error) = persistence::save(&self.paths, &self.data) {
                            self.saved.error = Some(error);
                        } else {
                            self.saved.dirty = false;
                            self.saved.last_saved = Some(Instant::now());
                        }
                        self.notify(
                            ToastKind::Success,
                            format!("Datos importados: {} eventos.", self.data.active_event_count()),
                            None,
                        );
                    }
                    Err(error) => {
                        self.notify(ToastKind::Error, format!("No se pudo importar: {error}"), None);
                    }
                }
            }
        }
    }

    fn save_now(&mut self) {
        match persistence::save(&self.paths, &self.data) {
            Ok(()) => {
                self.saved.dirty = false;
                self.saved.error = None;
                self.saved.last_saved = Some(Instant::now());
            }
            Err(error) => {
                self.saved.error = Some(error);
                self.saved.last_edit = Some(Instant::now());
            }
        }
    }

    fn autosave(&mut self, ctx: &Context) {
        if self.saved.dirty {
            let elapsed = self
                .saved
                .last_edit
                .map(|instant| instant.elapsed())
                .unwrap_or(AUTOSAVE_DELAY);
            if elapsed >= AUTOSAVE_DELAY {
                self.save_now();
            } else {
                ctx.request_repaint_after(AUTOSAVE_DELAY - elapsed);
            }
        }
        if let Some(toast) = &self.toast {
            let remaining = toast.duration.saturating_sub(toast.created.elapsed());
            if remaining.is_zero() {
                self.toast = None;
            } else {
                ctx.request_repaint_after(remaining.min(Duration::from_millis(250)));
            }
        }
    }

    pub fn data_path_label(&self) -> String {
        self.paths.data_file.display().to_string()
    }

    pub fn is_wide(&self, ctx: &Context) -> bool {
        ctx.content_rect().width() >= DETAILS_BREAKPOINT
    }
}

impl eframe::App for FCalendarApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        theme::BG.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.update_layout(&ctx);
        self.handle_shortcuts(&ctx);

        let mut actions: Vec<Action> = Vec::new();

        let top_frame = egui::Frame::new()
            .fill(theme::PANEL)
            .inner_margin(egui::Margin::symmetric(14, 9))
            .stroke(egui::Stroke::new(1.0, theme::BORDER_SOFT));
        egui::Panel::top("top_bar")
            .exact_size(58.0)
            .frame(top_frame)
            .show(ui, |ui| {
                self.top_bar(ui, &mut actions);
            });

        if self.view.sidebar_open {
            let sidebar_frame = egui::Frame::new()
                .fill(theme::PANEL)
                .inner_margin(egui::Margin::symmetric(12, 12));
            egui::Panel::left("sidebar")
                .default_size(262.0)
                .min_size(238.0)
                .max_size(330.0)
                .resizable(true)
                .frame(sidebar_frame)
                .show(ui, |ui| {
                    self.sidebar(ui, &mut actions);
                });
        }

        let wide = self.is_wide(&ctx);
        if wide {
            let details_frame = egui::Frame::new()
                .fill(theme::PANEL)
                .inner_margin(egui::Margin::symmetric(16, 14));
            egui::Panel::right("details")
                .default_size(340.0)
                .min_size(300.0)
                .max_size(430.0)
                .resizable(true)
                .frame(details_frame)
                .show(ui, |ui| {
                    self.details_panel(ui, &mut actions);
                });
        }

        let central_frame = egui::Frame::new()
            .fill(theme::BG)
            .inner_margin(egui::Margin::same(12));
        egui::CentralPanel::default()
            .frame(central_frame)
            .show(ui, |ui| {
                self.calendar_area(ui, &mut actions);
            });

        self.apply_actions(actions);

        if !wide {
            self.floating_details(&ctx);
        }

        if matches!(self.modal, ModalState::None) {
            self.search_popup(&ctx);
        }

        self.show_modals(&ctx);
        self.show_toast(&ctx);
        self.autosave(&ctx);
    }

    fn on_exit(&mut self) {
        if self.saved.dirty {
            let _ = persistence::save(&self.paths, &self.data);
        }
    }
}
