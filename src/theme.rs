//! Tema visual de FCalendar: oscuro, sobrio y con acentos azules.

use egui::{
    Color32, Context, CornerRadius, FontFamily, FontId, Margin, Shadow, Stroke, TextStyle, Visuals,
    style::WidgetVisuals,
};

pub const BG: Color32 = Color32::from_rgb(0x1B, 0x1B, 0x1F);
pub const PANEL: Color32 = Color32::from_rgb(0x21, 0x21, 0x26);
pub const PANEL_ALT: Color32 = Color32::from_rgb(0x26, 0x26, 0x2C);
pub const CARD: Color32 = Color32::from_rgb(0x2B, 0x2B, 0x32);
pub const CARD_HOVER: Color32 = Color32::from_rgb(0x34, 0x34, 0x3C);
pub const CARD_ACTIVE: Color32 = Color32::from_rgb(0x3B, 0x3B, 0x45);
pub const BORDER: Color32 = Color32::from_rgb(0x38, 0x38, 0x41);
pub const BORDER_SOFT: Color32 = Color32::from_rgb(0x2E, 0x2E, 0x36);
pub const TEXT: Color32 = Color32::from_rgb(0xEE, 0xEE, 0xF2);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0xAA, 0xAA, 0xB4);
pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x74, 0x74, 0x80);
pub const ACCENT: Color32 = Color32::from_rgb(0x4F, 0x6B, 0xE8);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x64, 0x7D, 0xF0);
pub const ACCENT_DARK: Color32 = Color32::from_rgb(0x2F, 0x4C, 0xB4);
pub const ACCENT_SOFT: Color32 = Color32::from_rgba_unmultiplied_const(0x2F, 0x4C, 0xB4, 0x55);
pub const DANGER: Color32 = Color32::from_rgb(0xE0, 0x5C, 0x5C);
pub const DANGER_SOFT: Color32 = Color32::from_rgba_unmultiplied_const(0x7A, 0x2A, 0x2A, 0x60);
pub const GOLD: Color32 = Color32::from_rgb(0xE8, 0xB3, 0x4F);
pub const GREEN: Color32 = Color32::from_rgb(0x63, 0xB5, 0x76);
pub const TODAY: Color32 = Color32::from_rgb(0x5B, 0x7C, 0xFF);

pub const RADIUS_SMALL: u8 = 7;
pub const RADIUS: u8 = 10;
pub const RADIUS_LARGE: u8 = 14;

pub const PALETTE: [(&str, &str); 10] = [
    ("Rojo", "#B42F2F"),
    ("Azul", "#3B5BD9"),
    ("Naranja", "#C2703A"),
    ("Violeta", "#8A47C7"),
    ("Rosa", "#C24B9F"),
    ("Verde", "#5B8F3A"),
    ("Cyan", "#2A9DB8"),
    ("Índigo", "#5B5BD6"),
    ("Amarillo", "#C9A227"),
    ("Gris", "#6B6B78"),
];

pub fn install(ctx: &Context) {
    ctx.set_theme(egui::Theme::Dark);

    let mut visuals = Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = PANEL_ALT;
    visuals.extreme_bg_color = Color32::from_rgb(0x17, 0x17, 0x1B);
    visuals.faint_bg_color = PANEL;
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.window_corner_radius = CornerRadius::same(RADIUS_LARGE);
    visuals.menu_corner_radius = CornerRadius::same(RADIUS);
    visuals.popup_shadow = Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(140),
    };
    visuals.window_shadow = Shadow {
        offset: [0, 12],
        blur: 32,
        spread: 0,
        color: Color32::from_black_alpha(160),
    };
    visuals.hyperlink_color = ACCENT_HOVER;
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.striped = false;

    let inactive = WidgetVisuals {
        bg_fill: CARD,
        weak_bg_fill: CARD,
        bg_stroke: Stroke::new(1.0, BORDER_SOFT),
        fg_stroke: Stroke::new(1.0, TEXT),
        corner_radius: CornerRadius::same(RADIUS_SMALL),
        expansion: 0.0,
    };
    visuals.widgets.noninteractive = WidgetVisuals {
        bg_fill: PANEL,
        weak_bg_fill: PANEL,
        bg_stroke: Stroke::new(1.0, BORDER_SOFT),
        fg_stroke: Stroke::new(1.0, TEXT_MUTED),
        corner_radius: CornerRadius::same(RADIUS_SMALL),
        expansion: 0.0,
    };
    visuals.widgets.inactive = inactive;
    visuals.widgets.hovered = WidgetVisuals {
        bg_fill: CARD_HOVER,
        weak_bg_fill: CARD_HOVER,
        bg_stroke: Stroke::new(1.0, ACCENT),
        fg_stroke: Stroke::new(1.0, TEXT),
        corner_radius: CornerRadius::same(RADIUS_SMALL),
        expansion: 0.0,
    };
    visuals.widgets.active = WidgetVisuals {
        bg_fill: CARD_ACTIVE,
        weak_bg_fill: CARD_ACTIVE,
        bg_stroke: Stroke::new(1.0, ACCENT_HOVER),
        fg_stroke: Stroke::new(1.0, Color32::WHITE),
        corner_radius: CornerRadius::same(RADIUS_SMALL),
        expansion: 0.0,
    };
    visuals.widgets.open = WidgetVisuals {
        bg_fill: CARD_ACTIVE,
        weak_bg_fill: CARD_ACTIVE,
        bg_stroke: Stroke::new(1.0, ACCENT),
        fg_stroke: Stroke::new(1.0, TEXT),
        corner_radius: CornerRadius::same(RADIUS_SMALL),
        expansion: 0.0,
    };
    ctx.set_visuals(visuals);

    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.spacing.interact_size = egui::vec2(30.0, 28.0);
        style.spacing.menu_margin = Margin::same(8);
        style.spacing.window_margin = Margin::same(16);
        style.spacing.combo_width = 180.0;
        style.spacing.indent = 16.0;
        style.animation_time = 0.12;
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(24.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(13.5, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(13.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(11.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(12.0, FontFamily::Monospace),
        );
    });
}

pub fn color32(rgb: [u8; 3]) -> Color32 {
    Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}

pub fn color32_hex(hex: &str) -> Color32 {
    crate::model::parse_hex_color(hex)
        .map(color32)
        .unwrap_or(CARD_HOVER)
}

pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

pub fn lighten(color: Color32, amount: f32) -> Color32 {
    let amount = amount.clamp(0.0, 1.0);
    Color32::from_rgb(
        (color.r() as f32 + (255.0 - color.r() as f32) * amount) as u8,
        (color.g() as f32 + (255.0 - color.g() as f32) * amount) as u8,
        (color.b() as f32 + (255.0 - color.b() as f32) * amount) as u8,
    )
}

pub fn contrast_text(background: [u8; 3]) -> Color32 {
    let luminance = 0.299 * background[0] as f32 + 0.587 * background[1] as f32 + 0.114 * background[2] as f32;
    if luminance > 150.0 {
        Color32::from_rgb(0x18, 0x18, 0x1C)
    } else {
        Color32::WHITE
    }
}

/// Icono de ventana generado en memoria, sin archivos externos.
pub fn app_icon() -> egui::IconData {
    const SIZE: usize = 64;
    let mut rgba = vec![0u8; SIZE * SIZE * 4];
    let accent = [0x4F, 0x6B, 0xE8];
    let panel = [0x21, 0x21, 0x26];
    let white = [0xF4, 0xF4, 0xF8];
    let colors = [[0x4F, 0x6B, 0xE8], [0xE8, 0xB3, 0x4F], [0x63, 0xB5, 0x76]];

    let in_rounded = |x: f32, y: f32, left: f32, top: f32, right: f32, bottom: f32, radius: f32| -> bool {
        if x < left || x > right || y < top || y > bottom {
            return false;
        }
        let radius = radius.min((right - left).min(bottom - top) * 0.5);
        let cx = x.clamp(left + radius, right - radius);
        let cy = y.clamp(top + radius, bottom - radius);
        (x - cx).powi(2) + (y - cy).powi(2) <= radius * radius
    };

    let mut set = |x: usize, y: usize, color: [u8; 3], alpha: u8| {
        let index = (y * SIZE + x) * 4;
        rgba[index] = color[0];
        rgba[index + 1] = color[1];
        rgba[index + 2] = color[2];
        rgba[index + 3] = alpha;
    };

    for y in 0..SIZE {
        for x in 0..SIZE {
            let xf = x as f32 + 0.5;
            let yf = y as f32 + 0.5;
            if in_rounded(xf, yf, 4.0, 4.0, 60.0, 60.0, 14.0) {
                set(x, y, panel, 255);
            }
            if in_rounded(xf, yf, 4.0, 4.0, 60.0, 22.0, 14.0) && yf < 22.0 {
                set(x, y, accent, 255);
            }
            if in_rounded(xf, yf, 13.0, 18.0, 51.0, 55.0, 7.0) {
                set(x, y, white, 255);
            }
        }
    }

    // Anillas y barras de eventos.
    for y in 16..24 {
        for x in 20..24 {
            set(x, y, panel, 255);
        }
        for x in 40..44 {
            set(x, y, panel, 255);
        }
    }
    for (row, color) in colors.iter().enumerate() {
        let top = 27 + row as usize * 8;
        for y in top..top + 4 {
            for x in 20..44 {
                set(x, y, *color, 255);
            }
        }
    }

    egui::IconData {
        rgba,
        width: SIZE as u32,
        height: SIZE as u32,
    }
}
