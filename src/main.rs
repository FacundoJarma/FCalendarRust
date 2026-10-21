mod app;
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

use std::sync::Arc;

use crate::app::FCalendarApp;
use crate::persistence::DataPaths;

fn main() -> eframe::Result {
    let paths = DataPaths::new(None);
    let loaded = persistence::load(&paths);

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("FCalendar")
            .with_app_id("io.facujarma.fcalendar")
            .with_inner_size([1240.0, 700.0])
            .with_min_inner_size([900.0, 600.0])
            .with_icon(Arc::new(theme::app_icon())),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "FCalendar",
        options,
        Box::new(move |cc| {
            Ok(Box::new(FCalendarApp::new(
                cc,
                paths,
                loaded.data,
                loaded.notice,
            )))
        }),
    )
}
