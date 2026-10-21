//! Persistencia local: JSON atómico, copias de seguridad e importación del
//! formato de la app React/Firebase original.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::date_utils::{parse_flexible_date, today};
use crate::model::{
    BG_PROPERTY, CalendarData, Category, DEFAULT_CATEGORY, DEFAULT_CATEGORY_COLOR, Event,
    PERSONAL_PROPERTY, Property, Shortcut, TEXT_PROPERTY, parse_hex_color,
};

const BACKUP_KEEP: usize = 14;

#[derive(Clone, Debug)]
pub struct DataPaths {
    pub data_dir: PathBuf,
    pub data_file: PathBuf,
    pub backups_dir: PathBuf,
}

impl DataPaths {
    pub fn new(override_dir: Option<PathBuf>) -> Self {
        let data_dir = override_dir
            .or_else(|| std::env::var_os("FCALENDAR_DATA_DIR").map(PathBuf::from))
            .or_else(|| {
                directories::ProjectDirs::from("io", "facujarma", "fcalendar")
                    .map(|dirs| dirs.data_dir().to_path_buf())
            })
            .unwrap_or_else(|| PathBuf::from("fcalendar-data"));
        Self::from_dir(data_dir)
    }

    pub fn from_dir(data_dir: PathBuf) -> Self {
        let data_file = data_dir.join("data.json");
        let backups_dir = data_dir.join("backups");
        Self {
            data_dir,
            data_file,
            backups_dir,
        }
    }
}

#[derive(Debug)]
pub struct LoadResult {
    pub data: CalendarData,
    pub notice: Option<String>,
}

pub fn load(paths: &DataPaths) -> LoadResult {
    if !paths.data_file.exists() {
        let data = CalendarData::default();
        let notice = save(paths, &data)
            .err()
            .map(|error| format!("No se pudo crear el archivo de datos: {error}"));
        return LoadResult { data, notice };
    }

    let bytes = match fs::read(&paths.data_file) {
        Ok(bytes) => bytes,
        Err(error) => {
            return LoadResult {
                data: CalendarData::default(),
                notice: Some(format!("No se pudo leer el archivo de datos: {error}")),
            };
        }
    };

    match parse_import(&bytes) {
        Ok(mut data) => {
            data.normalize();
            LoadResult { data, notice: None }
        }
        Err(error) => {
            let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
            let corrupt = paths
                .data_dir
                .join(format!("data-corrupto-{stamp}.json"));
            let _ = fs::copy(&paths.data_file, &corrupt);
            let data = CalendarData::default();
            LoadResult {
                data,
                notice: Some(format!(
                    "No se pudo leer el calendario ({error}). Se conservó una copia en {}.",
                    corrupt.display()
                )),
            }
        }
    }
}

pub fn save(paths: &DataPaths, data: &CalendarData) -> Result<(), String> {
    fs::create_dir_all(&paths.data_dir)
        .map_err(|error| format!("no se pudo crear {}: {error}", paths.data_dir.display()))?;
    fs::create_dir_all(&paths.backups_dir).map_err(|error| {
        format!(
            "no se pudo crear la carpeta de copias {}: {error}",
            paths.backups_dir.display()
        )
    })?;
    backup_existing(paths);
    let bytes = serde_json::to_vec_pretty(data)
        .map_err(|error| format!("no se pudo serializar el calendario: {error}"))?;
    atomic_write(&paths.data_file, &bytes)
}

pub fn export_to(path: &Path, data: &CalendarData) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(data)
        .map_err(|error| format!("no se pudo serializar el calendario: {error}"))?;
    atomic_write(path, &bytes)
}

pub fn import_from(path: &Path) -> Result<CalendarData, String> {
    let bytes = fs::read(path).map_err(|error| format!("no se pudo leer el archivo: {error}"))?;
    let mut data = parse_import(&bytes)?;
    data.normalize();
    Ok(data)
}

pub fn parse_import(bytes: &[u8]) -> Result<CalendarData, String> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("el archivo no es JSON válido: {error}"))?;

    // Formato nativo nuevo.
    if value.get("version").is_some() {
        let mut data: CalendarData = serde_json::from_value(value)
            .map_err(|error| format!("el archivo no tiene el formato de FCalendar: {error}"))?;
        data.normalize();
        return Ok(data);
    }

    // Formato moderno sin campo version (por robustez).
    if value.get("events").is_some_and(Value::is_array) {
        if let Ok(mut data) = serde_json::from_value::<CalendarData>(value.clone()) {
            data.normalize();
            return Ok(data);
        }
    }

    parse_legacy(value)
}

fn parse_legacy(value: Value) -> Result<CalendarData, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "el archivo debe contener un objeto JSON".to_string())?;

    let has_bundle = object.contains_key("events")
        || object.contains_key("categories")
        || object.contains_key("shortcuts");
    let (events_value, categories_value, shortcuts_value) = if has_bundle {
        (
            object.get("events"),
            object.get("categories"),
            object.get("shortcuts"),
        )
    } else if is_date_map(object) {
        (Some(&value), None, None)
    } else {
        return Err(
            "no se reconoce el formato. Se esperaba un archivo de FCalendar o una exportación de Firebase."
                .to_string(),
        );
    };

    let mut data = CalendarData::default();
    data.categories.clear();
    if let Some(categories) = categories_value {
        parse_categories(categories, &mut data);
    }
    if data.categories.is_empty() {
        data.categories.insert(
            DEFAULT_CATEGORY.to_string(),
            Category::new(DEFAULT_CATEGORY_COLOR),
        );
    }

    if let Some(events) = events_value {
        parse_events(events, &mut data)?;
    }

    if let Some(shortcuts) = shortcuts_value {
        parse_shortcuts(shortcuts, &mut data);
    }

    if let Some(profile_name) = object.get("profile_name").and_then(Value::as_str) {
        data.profile_name = profile_name.to_string();
    }

    data.normalize();
    Ok(data)
}

fn is_date_map(object: &Map<String, Value>) -> bool {
    !object.is_empty()
        && object
            .keys()
            .all(|key| parse_flexible_date(key).is_some())
        && object.values().all(Value::is_array)
}

fn parse_categories(value: &Value, data: &mut CalendarData) {
    let Some(object) = value.as_object() else {
        return;
    };
    for (name, category_value) in object {
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        let category = if let Some(color) = category_value.as_str() {
            Category::new(color)
        } else if let Some(object) = category_value.as_object() {
            Category {
                color: object
                    .get("color")
                    .and_then(Value::as_str)
                    .unwrap_or(DEFAULT_CATEGORY_COLOR)
                    .to_string(),
                active: object.get("active").and_then(Value::as_bool).unwrap_or(true),
                focused: object
                    .get("focused")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            }
        } else {
            Category::new(DEFAULT_CATEGORY_COLOR)
        };
        data.categories.insert(name.to_string(), category);
    }
}

fn parse_events(value: &Value, data: &mut CalendarData) -> Result<(), String> {
    if let Some(array) = value.as_array() {
        for item in array {
            let event = parse_event(item, None)?;
            data.push_event(event);
        }
        return Ok(());
    }

    if let Some(object) = value.as_object() {
        if !is_date_map(object) {
            return Err("la sección de eventos no tiene fechas válidas".to_string());
        }
        let mut date_entries: Vec<_> = object.iter().collect();
        date_entries.sort_by_key(|(key, _)| parse_flexible_date(key));
        for (date_key, events_value) in date_entries {
            let Some(date) = parse_flexible_date(date_key) else {
                continue;
            };
            let Some(events) = events_value.as_array() else {
                continue;
            };
            for item in events {
                let event = parse_event(item, Some(date))?;
                data.push_event(event);
            }
        }
    }
    Ok(())
}

fn parse_event(value: &Value, fallback_date: Option<chrono::NaiveDate>) -> Result<Event, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "un evento no es un objeto JSON".to_string())?;

    let title = object
        .get("title")
        .or_else(|| object.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("Sin título")
        .to_string();

    let tags = object
        .get("tags")
        .and_then(Value::as_array)
        .map(|tags| {
            tags.iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();

    let date = object
        .get("date")
        .and_then(Value::as_str)
        .and_then(parse_flexible_date)
        .or(fallback_date)
        .unwrap_or_else(today);

    let mut event = Event::new(0, title, date, tags);
    event.active = object.get("active").and_then(Value::as_bool).unwrap_or(true);

    if let Some(properties) = object.get("properties").and_then(Value::as_object) {
        for (name, property_value) in properties {
            if let Some(property) = parse_property(name, property_value) {
                event.properties.insert(name.clone(), property);
            }
        }
    }
    event.ensure_default_properties();
    Ok(event)
}

fn parse_property(name: &str, value: &Value) -> Option<Property> {
    let required_color = matches!(name, BG_PROPERTY | TEXT_PROPERTY);

    if name == PERSONAL_PROPERTY {
        let flag = value
            .as_bool()
            .or_else(|| value.get("value").and_then(Value::as_bool))
            .unwrap_or(false);
        return Some(Property::Checkbox(flag));
    }

    let (kind, raw): (&str, Value) = if let Some(object) = value.as_object() {
        (
            object.get("type").and_then(Value::as_str).unwrap_or("text"),
            object.get("value").cloned().unwrap_or_else(|| value.clone()),
        )
    } else if value.is_string() && required_color {
        ("color", value.clone())
    } else if value.is_string() {
        ("text", value.clone())
    } else if value.is_number() {
        ("num", value.clone())
    } else if value.is_boolean() {
        ("checkbox", value.clone())
    } else {
        return None;
    };

    match kind {
        "color" => raw
            .as_str()
            .filter(|text| parse_hex_color(text).is_some())
            .map(|text| Property::Color(text.to_string())),
        "num" | "number" => Some(Property::Num(
            raw.as_f64()
                .or_else(|| raw.as_str().and_then(|text| text.parse().ok()))
                .unwrap_or(0.0),
        )),
        "checkbox" | "bool" => Some(Property::Checkbox(
            raw.as_bool()
                .or_else(|| raw.as_str().map(|text| text.eq_ignore_ascii_case("true")))
                .unwrap_or(false),
        )),
        "link" | "url" => raw
            .as_str()
            .map(|text| Property::Link(text.to_string())),
        _ => {
            let text = raw
                .as_str()
                .map(ToString::to_string)
                .or_else(|| raw.as_f64().map(|number| number.to_string()))
                .unwrap_or_default();
            if required_color {
                Some(Property::Color(text))
            } else {
                Some(Property::Text(text))
            }
        }
    }
}

fn parse_shortcuts(value: &Value, data: &mut CalendarData) {
    let mut parsed: Vec<(f64, Shortcut)> = Vec::new();

    if let Some(array) = value.as_array() {
        for (index, item) in array.iter().enumerate() {
            if let Some(shortcut) = shortcut_from_value(item, index as f64) {
                parsed.push(shortcut);
            }
        }
    } else if let Some(object) = value.as_object() {
        let mut entries: Vec<_> = object.iter().collect();
        entries.sort_by(|a, b| {
            a.0.parse::<f64>()
                .unwrap_or(f64::MAX)
                .total_cmp(&b.0.parse::<f64>().unwrap_or(f64::MAX))
        });
        for (index, (key, item)) in entries.iter().enumerate() {
            let priority = key.parse::<f64>().unwrap_or(index as f64);
            if let Some(shortcut) = shortcut_from_value(item, priority) {
                parsed.push(shortcut);
            }
        }
    }

    // Prioridad 0 = se dibuja arriba y gana. El orden de creación original era
    // 0, 1, 2... y la app aplicaba al final el de menor número.
    parsed.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut names = std::collections::HashSet::new();
    for (_, shortcut) in parsed {
        if names.insert(shortcut.name.clone()) {
            data.shortcuts.push(shortcut);
        }
    }
}

fn shortcut_from_value(item: &Value, fallback_priority: f64) -> Option<(f64, Shortcut)> {
    let object = item.as_object()?;
    let name = object.get("name").and_then(Value::as_str)?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let properties = object.get("properties").and_then(Value::as_object);
    let background = object
        .get("background")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            properties
                .and_then(|properties| properties.get(BG_PROPERTY))
                .and_then(property_text)
        })
        .unwrap_or_else(|| "#404040".to_string());
    let text_color = object
        .get("text_color")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            properties
                .and_then(|properties| properties.get(TEXT_PROPERTY))
                .and_then(property_text)
        })
        .unwrap_or_else(|| "#FFFFFF".to_string());
    let priority = object
        .get("shortcutPriority")
        .and_then(Value::as_f64)
        .unwrap_or(fallback_priority);
    Some((priority, Shortcut::new(name, background, text_color)))
}

fn property_text(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(ToString::to_string)
        .or_else(|| value.get("value").and_then(Value::as_str).map(ToString::to_string))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("no se pudo crear {}: {error}", parent.display()))?;
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("data.json");
    let temp = path.with_file_name(format!(".{file_name}.tmp"));

    let write_result = (|| -> Result<(), String> {
        let mut file =
            File::create(&temp).map_err(|error| format!("no se pudo crear el archivo temporal: {error}"))?;
        file.write_all(bytes)
            .map_err(|error| format!("no se pudo escribir el archivo: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("no se pudo sincronizar el archivo: {error}"))?;
        Ok(())
    })();

    if let Err(error) = write_result {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }

    #[cfg(target_os = "windows")]
    if path.exists() {
        fs::remove_file(path).map_err(|error| format!("no se pudo reemplazar el archivo: {error}"))?;
    }

    fs::rename(&temp, path).map_err(|error| {
        let _ = fs::remove_file(&temp);
        format!("no se pudo guardar el archivo: {error}")
    })?;
    Ok(())
}

fn backup_existing(paths: &DataPaths) {
    if !paths.data_file.exists() {
        return;
    }
    let stamp = today().format("%Y-%m-%d").to_string();
    let backup = paths.backups_dir.join(format!("data-{stamp}.json"));
    if !backup.exists() {
        let _ = fs::copy(&paths.data_file, &backup);
    }
    prune_backups(&paths.backups_dir);
}

fn prune_backups(backups_dir: &Path) {
    let Ok(entries) = fs::read_dir(backups_dir) else {
        return;
    };
    let mut backups: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("data-") && name.ends_with(".json"))
        })
        .collect();
    if backups.len() <= BACKUP_KEEP {
        return;
    }
    backups.sort();
    let remove_count = backups.len() - BACKUP_KEEP;
    for path in backups.into_iter().take(remove_count) {
        let _ = fs::remove_file(path);
    }
}

pub fn open_data_folder(paths: &DataPaths) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    let command = "xdg-open";
    #[cfg(target_os = "windows")]
    let command = "explorer";
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    let command = "open";

    std::process::Command::new(command)
        .arg(&paths.data_dir)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("no se pudo abrir la carpeta: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths(name: &str) -> DataPaths {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        DataPaths::from_dir(std::env::temp_dir().join(format!(
            "fcalendar-test-{name}-{}-{stamp}",
            std::process::id()
        )))
    }

    #[test]
    fn saves_and_loads_roundtrip() {
        let paths = temp_paths("roundtrip");
        let mut data = CalendarData::default();
        let id = data.add_event("Prueba", today(), vec![DEFAULT_CATEGORY.into()]);
        save(&paths, &data).unwrap();
        assert!(paths.data_file.exists());
        let loaded = load(&paths);
        assert!(loaded.notice.is_none(), "{:?}", loaded.notice);
        assert_eq!(loaded.data.event(id).unwrap().title, "Prueba");
        let _ = fs::remove_dir_all(&paths.data_dir);
    }

    #[test]
    fn imports_firebase_bundle() {
        let json = r##"{
          "categories": {
            "Trabajo": {"color": "#2F4CB4", "active": true, "focused": false},
            "Personal": "#4d7c0f"
          },
          "events": {
            "21-10-2026": [
              {
                "id": "21-10-2026-1",
                "title": "Reunión",
                "tags": ["Trabajo"],
                "date": "21-10-2026",
                "active": true,
                "properties": {
                  "BackgroundColor": {"type": "text", "value": "#404040"},
                  "TextColor": {"type": "text", "value": "#FFFFFF"},
                  "PriorizePersonal": {"type": "checkbox", "value": false},
                  "Enlace": {"type": "link", "value": "https://example.com"},
                  "Horas": {"type": "num", "value": "2"}
                }
              }
            ],
            "22-10-2026": [
              {
                "title": "Cumpleaños",
                "tags": ["Personal"],
                "active": true,
                "properties": {}
              }
            ]
          },
          "shortcuts": {
            "0": {
              "name": "Trabajo",
              "properties": {"BackgroundColor": "#123456", "TextColor": "#FFFFFF"},
              "shortcutPriority": 0
            }
          }
        }"##;
        let data = parse_import(json.as_bytes()).unwrap();
        assert_eq!(data.events.len(), 2);
        let meeting = data.events.iter().find(|event| event.title == "Reunión").unwrap();
        assert_eq!(meeting.date, chrono::NaiveDate::from_ymd_opt(2026, 10, 21).unwrap());
        assert!(matches!(
            meeting.properties.get("Enlace"),
            Some(Property::Link(_))
        ));
        assert!(matches!(
            meeting.properties.get(BG_PROPERTY),
            Some(Property::Color(color)) if color == "#404040"
        ));
        assert_eq!(data.shortcuts.len(), 1);
        assert_eq!(data.resolve_style(meeting).background, [0x12, 0x34, 0x56]);
    }

    #[test]
    fn imports_raw_firebase_events_map() {
        let json = r##"{
          "21-10-2026": [{
            "title": "Sin categoría",
            "tags": [],
            "properties": {
              "BackgroundColor": {"type": "text", "value": "#404040"},
              "TextColor": {"type": "text", "value": "#FFFFFF"}
            }
          }]
        }"##;
        let data = parse_import(json.as_bytes()).unwrap();
        assert_eq!(data.events.len(), 1);
        assert_eq!(data.events[0].title, "Sin categoría");
        assert!(data.categories.contains_key(DEFAULT_CATEGORY));
    }

    #[test]
    fn corrupt_data_is_preserved_and_replaced() {
        let paths = temp_paths("corrupt");
        fs::create_dir_all(&paths.data_dir).unwrap();
        fs::write(&paths.data_file, b"{ esto no es json").unwrap();
        let loaded = load(&paths);
        assert!(loaded.notice.is_some());
        assert_eq!(loaded.data.active_event_count(), 0);
        let copies: Vec<_> = fs::read_dir(&paths.data_dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("data-corrupto-"))
            .collect();
        assert_eq!(copies.len(), 1);
        let _ = fs::remove_dir_all(&paths.data_dir);
    }

    #[test]
    fn event_without_date_falls_back_to_today() {
        let mut event = Event::new(0, "Hoy", today(), vec![]);
        event.date = today();
        assert_eq!(event.date, today());
    }
}
