//! Utilidades de fecha y formato en español.

use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};

pub const MONTHS: [&str; 12] = [
    "Enero",
    "Febrero",
    "Marzo",
    "Abril",
    "Mayo",
    "Junio",
    "Julio",
    "Agosto",
    "Septiembre",
    "Octubre",
    "Noviembre",
    "Diciembre",
];

pub const MONTHS_SHORT: [&str; 12] = [
    "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic",
];

pub const WEEKDAYS: [&str; 7] = [
    "Domingo",
    "Lunes",
    "Martes",
    "Miércoles",
    "Jueves",
    "Viernes",
    "Sábado",
];

pub const WEEKDAYS_SHORT: [&str; 7] = ["Dom", "Lun", "Mar", "Mié", "Jue", "Vie", "Sáb"];

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

pub fn month_name(month: u32) -> &'static str {
    MONTHS[(month.saturating_sub(1) as usize).min(11)]
}

pub fn month_name_short(month: u32) -> &'static str {
    MONTHS_SHORT[(month.saturating_sub(1) as usize).min(11)]
}

pub fn weekday_name(date: NaiveDate) -> &'static str {
    WEEKDAYS[date.weekday().num_days_from_sunday() as usize]
}

/// Primer día (domingo) de la cuadrícula del mes.
pub fn month_start_cell(year: i32, month: u32) -> NaiveDate {
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("mes válido");
    let offset = first.weekday().num_days_from_sunday() as i64;
    first - Duration::days(offset)
}

/// Las 42 fechas visibles en la vista mensual (6 semanas, empezando en domingo).
pub fn month_grid(year: i32, month: u32) -> [NaiveDate; 42] {
    let start = month_start_cell(year, month);
    std::array::from_fn(|i| start + Duration::days(i as i64))
}

/// Suma (o resta) meses manteniendo el año correcto.
pub fn shift_month(year: i32, month: u32, delta: i32) -> (i32, u32) {
    let zero_based = year as i64 * 12 + month as i64 - 1 + delta as i64;
    let new_year = zero_based.div_euclid(12) as i32;
    let new_month = zero_based.rem_euclid(12) as u32 + 1;
    (new_year, new_month)
}

pub fn format_short(date: NaiveDate) -> String {
    format!(
        "{} {} {}",
        date.day(),
        month_name_short(date.month()),
        date.year()
    )
}

pub fn format_long(date: NaiveDate) -> String {
    format!(
        "{} {} de {} de {}",
        weekday_name(date),
        date.day(),
        month_name(date.month()).to_lowercase(),
        date.year()
    )
}

pub fn format_month_year(year: i32, month: u32) -> String {
    format!("{} {}", month_name(month), year)
}

/// Formato usado por la app original de Firebase: `d-m-aaaa`.
pub fn format_legacy(date: NaiveDate) -> String {
    format!("{}-{}-{}", date.day(), date.month(), date.year())
}

/// Texto relativo para la fecha de un evento.
pub fn relative_label(date: NaiveDate) -> String {
    let today = today();
    let days = (date - today).num_days();
    match days {
        0 => "Hoy".to_string(),
        1 => "Mañana".to_string(),
        -1 => "Ayer".to_string(),
        _ => format_short(date),
    }
}

pub fn is_weekend(date: NaiveDate) -> bool {
    matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
}

/// Interpreta los formatos usados por la app original y por las copias nuevas.
pub fn parse_flexible_date(value: &str) -> Option<NaiveDate> {
    let value = value.trim();
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Some(date);
    }
    let parts: Vec<&str> = value.split(['-', '/']).collect();
    if parts.len() == 3 {
        let day = parts[0].parse::<u32>().ok()?;
        let month = parts[1].parse::<u32>().ok()?;
        let year = parts[2].parse::<i32>().ok()?;
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            return Some(date);
        }
        // También acepta aaaa-mm-dd cuando el separador es "/".
        let year = parts[0].parse::<i32>().ok()?;
        let month = parts[1].parse::<u32>().ok()?;
        let day = parts[2].parse::<u32>().ok()?;
        return NaiveDate::from_ymd_opt(year, month, day);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_starts_on_sunday_and_has_42_days() {
        let grid = month_grid(2026, 10);
        assert_eq!(grid.len(), 42);
        assert_eq!(grid[0].weekday(), Weekday::Sun);
        assert_eq!(grid[0], NaiveDate::from_ymd_opt(2026, 9, 27).unwrap());
        assert_eq!(grid[41], NaiveDate::from_ymd_opt(2026, 11, 7).unwrap());
    }

    #[test]
    fn month_shift_wraps_year() {
        assert_eq!(shift_month(2026, 12, 1), (2027, 1));
        assert_eq!(shift_month(2026, 1, -1), (2025, 12));
        assert_eq!(shift_month(2026, 10, -22), (2024, 12));
    }

    #[test]
    fn parses_legacy_and_iso_dates() {
        let expected = NaiveDate::from_ymd_opt(2026, 10, 21).unwrap();
        assert_eq!(parse_flexible_date("21-10-2026"), Some(expected));
        assert_eq!(parse_flexible_date("2026-10-21"), Some(expected));
        assert_eq!(parse_flexible_date("21/10/2026"), Some(expected));
        assert_eq!(parse_flexible_date("31-02-2026"), None);
    }

    #[test]
    fn formats_spanish_dates() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 21).unwrap();
        assert_eq!(format_long(date), "Miércoles 21 de octubre de 2026");
        assert_eq!(format_short(date), "21 oct 2026");
        assert_eq!(format_legacy(date), "21-10-2026");
    }
}
