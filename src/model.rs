//! Modelo de datos del calendario, independiente de la interfaz.
//!
//! Los eventos se guardan en un `Vec` plano para conservar un orden estable y
//! se indexan por id y por fecha. Así, dibujar un mes cuesta O(eventos del mes)
//! y abrir un evento es O(1), sin recorrer todo el calendario.

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{Datelike, NaiveDate};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::date_utils::{format_legacy, today};

pub const DATA_VERSION: u32 = 1;
pub const DEFAULT_CATEGORY: &str = "Otros";
pub const BG_PROPERTY: &str = "BackgroundColor";
pub const TEXT_PROPERTY: &str = "TextColor";
pub const PERSONAL_PROPERTY: &str = "PriorizePersonal";
pub const DEFAULT_BG: &str = "#404040";
pub const DEFAULT_TEXT: &str = "#FFFFFF";
pub const DEFAULT_CATEGORY_COLOR: &str = "#555555";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum Property {
    Text(String),
    Num(f64),
    Checkbox(bool),
    Link(String),
    Color(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyKind {
    Text,
    Num,
    Checkbox,
    Link,
    Color,
}

impl PropertyKind {
    pub const ALL: [Self; 5] = [
        Self::Text,
        Self::Num,
        Self::Checkbox,
        Self::Link,
        Self::Color,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "Texto",
            Self::Num => "Número",
            Self::Checkbox => "Casilla",
            Self::Link => "Enlace",
            Self::Color => "Color",
        }
    }

    pub fn default_value(self) -> Property {
        match self {
            Self::Text => Property::Text(String::new()),
            Self::Num => Property::Num(0.0),
            Self::Checkbox => Property::Checkbox(false),
            Self::Link => Property::Link("https://".to_string()),
            Self::Color => Property::Color(DEFAULT_BG.to_string()),
        }
    }
}

impl Property {
    pub fn kind(&self) -> PropertyKind {
        match self {
            Self::Text(_) => PropertyKind::Text,
            Self::Num(_) => PropertyKind::Num,
            Self::Checkbox(_) => PropertyKind::Checkbox,
            Self::Link(_) => PropertyKind::Link,
            Self::Color(_) => PropertyKind::Color,
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Checkbox(value) => *value,
            Self::Text(value) => value.eq_ignore_ascii_case("true"),
            Self::Num(value) => *value != 0.0,
            _ => false,
        }
    }

    pub fn color_rgb(&self) -> Option<[u8; 3]> {
        match self {
            Self::Color(value) | Self::Text(value) => parse_hex_color(value),
            _ => None,
        }
    }

    pub fn text_value(&self) -> String {
        match self {
            Self::Text(value) | Self::Link(value) | Self::Color(value) => value.clone(),
            Self::Num(value) => format_number(*value),
            Self::Checkbox(value) => {
                if *value {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
        }
    }
}

pub fn format_number(value: f64) -> String {
    if value.fract().abs() < f64::EPSILON {
        format!("{}", value as i64)
    } else {
        format!("{value:.2}")
    }
}

pub fn parse_hex_color(value: &str) -> Option<[u8; 3]> {
    let value = value.trim().trim_start_matches('#');
    match value.len() {
        3 => {
            let r = u8::from_str_radix(&value[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&value[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&value[2..3].repeat(2), 16).ok()?;
            Some([r, g, b])
        }
        6 => {
            let r = u8::from_str_radix(&value[0..2], 16).ok()?;
            let g = u8::from_str_radix(&value[2..4], 16).ok()?;
            let b = u8::from_str_radix(&value[4..6], 16).ok()?;
            Some([r, g, b])
        }
        _ => None,
    }
}

pub fn rgb_to_hex(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Category {
    #[serde(default = "default_category_color")]
    pub color: String,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default)]
    pub focused: bool,
}

fn default_category_color() -> String {
    DEFAULT_CATEGORY_COLOR.to_string()
}

fn default_true() -> bool {
    true
}

impl Category {
    pub fn new(color: impl Into<String>) -> Self {
        Self {
            color: color.into(),
            active: true,
            focused: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Shortcut {
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_bg")]
    pub background: String,
    #[serde(default = "default_text")]
    pub text_color: String,
}

fn default_bg() -> String {
    DEFAULT_BG.to_string()
}

fn default_text() -> String {
    DEFAULT_TEXT.to_string()
}

impl Shortcut {
    pub fn new(name: impl Into<String>, background: impl Into<String>, text_color: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            background: background.into(),
            text_color: text_color.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Event {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_event_date")]
    pub date: NaiveDate,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default)]
    pub properties: IndexMap<String, Property>,
}

fn default_event_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(1970, 1, 1).expect("fecha válida")
}

impl Event {
    pub fn new(id: u64, title: impl Into<String>, date: NaiveDate, tags: Vec<String>) -> Self {
        let mut properties = IndexMap::new();
        properties.insert(BG_PROPERTY.to_string(), Property::Color(DEFAULT_BG.to_string()));
        properties.insert(
            TEXT_PROPERTY.to_string(),
            Property::Color(DEFAULT_TEXT.to_string()),
        );
        properties.insert(
            PERSONAL_PROPERTY.to_string(),
            Property::Checkbox(false),
        );
        Self {
            id,
            title: title.into(),
            tags,
            date,
            active: true,
            properties,
        }
    }

    pub fn ensure_default_properties(&mut self) {
        self.properties
            .entry(BG_PROPERTY.to_string())
            .or_insert_with(|| Property::Color(DEFAULT_BG.to_string()));
        self.properties
            .entry(TEXT_PROPERTY.to_string())
            .or_insert_with(|| Property::Color(DEFAULT_TEXT.to_string()));
        self.properties
            .entry(PERSONAL_PROPERTY.to_string())
            .or_insert(Property::Checkbox(false));

        // Compatibilidad con la app original, que guardaba los colores como type "text".
        for name in [BG_PROPERTY, TEXT_PROPERTY] {
            if let Some(Property::Text(value)) = self.properties.get(name) {
                let value = value.clone();
                self.properties.insert(name.to_string(), Property::Color(value));
            }
        }
    }

    pub fn personal_colors(&self) -> bool {
        self.properties
            .get(PERSONAL_PROPERTY)
            .is_some_and(Property::as_bool)
    }

    pub fn background_rgb(&self) -> [u8; 3] {
        self.properties
            .get(BG_PROPERTY)
            .and_then(Property::color_rgb)
            .unwrap_or([0x40, 0x40, 0x40])
    }

    pub fn text_rgb(&self) -> [u8; 3] {
        self.properties
            .get(TEXT_PROPERTY)
            .and_then(Property::color_rgb)
            .unwrap_or([0xFF, 0xFF, 0xFF])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedStyle {
    pub background: [u8; 3],
    pub text: [u8; 3],
    pub visible: bool,
    pub focused: bool,
    pub shortcut: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarData {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub profile_name: String,
    #[serde(default)]
    pub categories: IndexMap<String, Category>,
    #[serde(default)]
    pub shortcuts: Vec<Shortcut>,
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub next_id: u64,
    #[serde(skip)]
    by_id: HashMap<u64, usize>,
    #[serde(skip)]
    by_date: BTreeMap<NaiveDate, Vec<usize>>,
}

fn default_version() -> u32 {
    DATA_VERSION
}

impl Default for CalendarData {
    fn default() -> Self {
        let mut categories = IndexMap::new();
        categories.insert(
            DEFAULT_CATEGORY.to_string(),
            Category::new(DEFAULT_CATEGORY_COLOR),
        );
        Self {
            version: DATA_VERSION,
            profile_name: "Mi calendario".to_string(),
            categories,
            shortcuts: Vec::new(),
            events: Vec::new(),
            next_id: 1,
            by_id: HashMap::new(),
            by_date: BTreeMap::new(),
        }
    }
}

impl CalendarData {
    /// Normaliza datos cargados o importados. Es idempotente.
    pub fn normalize(&mut self) {
        self.version = DATA_VERSION;
        if self.profile_name.trim().is_empty() {
            self.profile_name = "Mi calendario".to_string();
        }
        if self.categories.is_empty() {
            self.categories.insert(
                DEFAULT_CATEGORY.to_string(),
                Category::new(DEFAULT_CATEGORY_COLOR),
            );
        }

        // Garantiza que cada etiqueta usada tenga una categoría visible. Así una
        // importación antigua nunca oculta eventos por una categoría borrada.
        let mut missing_categories = Vec::new();
        for event in &self.events {
            for tag in &event.tags {
                if !self.categories.contains_key(tag) && !missing_categories.contains(tag) {
                    missing_categories.push(tag.clone());
                }
            }
        }
        for tag in missing_categories {
            self.categories
                .insert(tag, Category::new(DEFAULT_CATEGORY_COLOR));
        }

        // IDs únicos y estables.
        let mut max_id = self.next_id.saturating_sub(1);
        for event in &self.events {
            max_id = max_id.max(event.id);
        }
        let mut seen = HashSet::new();
        for event in &mut self.events {
            event.ensure_default_properties();
            if event.id == 0 || !seen.insert(event.id) {
                max_id += 1;
                event.id = max_id;
                seen.insert(event.id);
            }
            let mut seen_tags = HashSet::new();
            event.tags.retain(|tag| {
                let tag = tag.trim();
                !tag.is_empty() && seen_tags.insert(tag.to_string())
            });
        }
        self.next_id = max_id.saturating_add(1).max(1);

        // Un shortcut por categoría; el primero de la lista es el de mayor prioridad.
        let mut shortcut_names = HashSet::new();
        self.shortcuts
            .retain(|shortcut| !shortcut.name.trim().is_empty() && shortcut_names.insert(shortcut.name.clone()));

        self.rebuild_indexes();
    }

    pub fn rebuild_indexes(&mut self) {
        self.by_id.clear();
        self.by_date.clear();
        for (index, event) in self.events.iter().enumerate() {
            self.by_id.insert(event.id, index);
            self.by_date.entry(event.date).or_default().push(index);
        }
    }

    pub fn event(&self, id: u64) -> Option<&Event> {
        self.by_id.get(&id).and_then(|index| self.events.get(*index))
    }

    pub fn event_mut(&mut self, id: u64) -> Option<&mut Event> {
        let index = *self.by_id.get(&id)?;
        self.events.get_mut(index)
    }

    pub fn events_on(&self, date: NaiveDate) -> impl Iterator<Item = &Event> {
        self.by_date
            .get(&date)
            .into_iter()
            .flatten()
            .filter_map(|index| self.events.get(*index))
            .filter(|event| event.active)
    }

    pub fn all_events_on(&self, date: NaiveDate) -> impl Iterator<Item = &Event> {
        self.by_date
            .get(&date)
            .into_iter()
            .flatten()
            .filter_map(|index| self.events.get(*index))
    }

    pub fn count_on(&self, date: NaiveDate) -> usize {
        self.events_on(date).count()
    }

    pub fn active_event_count(&self) -> usize {
        self.events.iter().filter(|event| event.active).count()
    }

    pub fn deleted_event_count(&self) -> usize {
        self.events.iter().filter(|event| !event.active).count()
    }

    pub fn events_in_month(&self, year: i32, month: u32) -> usize {
        self.events
            .iter()
            .filter(|event| {
                event.active && event.date.year() == year && event.date.month() == month
            })
            .count()
    }

    pub fn add_event(
        &mut self,
        title: impl Into<String>,
        date: NaiveDate,
        tags: Vec<String>,
    ) -> u64 {
        let id = self.next_id.max(1);
        self.next_id = id.saturating_add(1);
        let event = Event::new(id, title, date, tags);
        self.events.push(event);
        let index = self.events.len() - 1;
        self.by_id.insert(id, index);
        self.by_date.entry(date).or_default().push(index);
        id
    }

    pub fn push_event(&mut self, mut event: Event) -> u64 {
        if event.id == 0 || self.by_id.contains_key(&event.id) {
            event.id = self.next_id.max(1);
        }
        self.next_id = self.next_id.max(event.id).saturating_add(1);
        let id = event.id;
        self.events.push(event);
        let index = self.events.len() - 1;
        self.by_id.insert(id, index);
        self.by_date.entry(self.events[index].date).or_default().push(index);
        id
    }

    pub fn update_event(&mut self, updated: Event) -> bool {
        let Some(index) = self.by_id.get(&updated.id).copied() else {
            return false;
        };
        let old_date = self.events[index].date;
        if old_date != updated.date {
            if let Some(indices) = self.by_date.get_mut(&old_date) {
                indices.retain(|value| *value != index);
                if indices.is_empty() {
                    self.by_date.remove(&old_date);
                }
            }
            self.by_date.entry(updated.date).or_default().push(index);
        }
        self.events[index] = updated;
        true
    }

    pub fn move_event_date(&mut self, id: u64, new_date: NaiveDate) -> bool {
        let Some(event) = self.event(id).cloned() else {
            return false;
        };
        let mut updated = event;
        updated.date = new_date;
        self.update_event(updated)
    }

    pub fn delete_event(&mut self, id: u64) -> bool {
        let Some(event) = self.event_mut(id) else {
            return false;
        };
        event.active = false;
        true
    }

    pub fn restore_event(&mut self, id: u64) -> bool {
        let Some(event) = self.event_mut(id) else {
            return false;
        };
        event.active = true;
        true
    }

    pub fn purge_deleted_events(&mut self) {
        self.events.retain(|event| event.active);
        self.rebuild_indexes();
    }

    pub fn add_category(&mut self, name: &str, color: &str) -> bool {
        let name = name.trim();
        if name.is_empty() || self.categories.contains_key(name) {
            return false;
        }
        self.categories
            .insert(name.to_string(), Category::new(color));
        true
    }

    pub fn delete_category(&mut self, name: &str) {
        self.categories.shift_remove(name);
        self.shortcuts.retain(|shortcut| shortcut.name != name);
        for event in &mut self.events {
            event.tags.retain(|tag| tag != name);
        }
    }

    pub fn set_category_active(&mut self, name: &str, active: bool) {
        if let Some(category) = self.categories.get_mut(name) {
            category.active = active;
        }
    }

    pub fn set_category_focused(&mut self, name: &str, focused: bool) {
        if let Some(category) = self.categories.get_mut(name) {
            category.focused = focused;
        }
    }

    pub fn upsert_shortcut(&mut self, name: &str, background: &str, text_color: &str) -> bool {
        if let Some(existing) = self
            .shortcuts
            .iter_mut()
            .find(|shortcut| shortcut.name == name)
        {
            existing.background = background.to_string();
            existing.text_color = text_color.to_string();
            false
        } else {
            self.shortcuts
                .push(Shortcut::new(name, background, text_color));
            true
        }
    }

    pub fn remove_shortcut(&mut self, name: &str) {
        self.shortcuts.retain(|shortcut| shortcut.name != name);
    }

    pub fn move_shortcut(&mut self, from: usize, to: usize) {
        if from >= self.shortcuts.len() || to >= self.shortcuts.len() || from == to {
            return;
        }
        let shortcut = self.shortcuts.remove(from);
        self.shortcuts.insert(to, shortcut);
    }

    /// Aplica prioridad de shortcuts, colores personales, visibilidad y foco.
    pub fn resolve_style(&self, event: &Event) -> ResolvedStyle {
        let mut visible = true;
        let mut focused = false;
        for tag in &event.tags {
            if let Some(category) = self.categories.get(tag) {
                visible &= category.active;
                focused |= category.focused;
            }
        }

        let mut background = event.background_rgb();
        let mut text = event.text_rgb();
        let mut shortcut_name = None;

        if !event.personal_colors() {
            if let Some(shortcut) = self
                .shortcuts
                .iter()
                .find(|shortcut| event.tags.iter().any(|tag| tag == &shortcut.name))
            {
                if let Some(rgb) = parse_hex_color(&shortcut.background) {
                    background = rgb;
                }
                if let Some(rgb) = parse_hex_color(&shortcut.text_color) {
                    text = rgb;
                }
                shortcut_name = Some(shortcut.name.clone());
            }
        }

        ResolvedStyle {
            background,
            text,
            visible,
            focused,
            shortcut: shortcut_name,
        }
    }

    pub fn search(&self, query: &str) -> Vec<&Event> {
        let query = normalize_for_search(query);
        if query.is_empty() {
            return Vec::new();
        }
        let today = today();
        let mut results: Vec<&Event> = self
            .events
            .iter()
            .filter(|event| event.active)
            .filter(|event| {
                normalize_for_search(&event.title).contains(&query)
                    || event
                        .tags
                        .iter()
                        .any(|tag| normalize_for_search(tag).contains(&query))
                    || event.date.to_string().contains(&query)
                    || format_legacy(event.date).contains(&query)
            })
            .collect();

        results.sort_by_key(|event| {
            let distance = (event.date - today).num_days();
            let group = if distance >= 0 { 0 } else { 1 };
            (group, distance.abs(), event.date)
        });
        results
    }

    pub fn shortcut_for_event(&self, event: &Event) -> Option<&Shortcut> {
        if event.personal_colors() {
            return None;
        }
        self.shortcuts
            .iter()
            .find(|shortcut| event.tags.iter().any(|tag| tag == &shortcut.name))
    }

    pub fn monthly_events(&self, year: i32, month: u32) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|event| {
                event.active && event.date.year() == year && event.date.month() == month
            })
            .collect()
    }
}

pub fn normalize_for_search(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|character| match character {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
    }

    fn sample() -> CalendarData {
        let mut data = CalendarData::default();
        data.add_category("Trabajo", "#2F4CB4");
        data.add_category("Personal", "#4d7c0f");
        data
    }

    #[test]
    fn default_data_has_category_and_no_events() {
        let data = CalendarData::default();
        assert!(data.categories.contains_key(DEFAULT_CATEGORY));
        assert_eq!(data.active_event_count(), 0);
        assert_eq!(data.next_id, 1);
    }

    #[test]
    fn added_events_are_indexed_by_id_and_date() {
        let mut data = sample();
        let id = data.add_event("Reunión", date(21), vec!["Trabajo".into()]);
        let event = data.event(id).unwrap();
        assert_eq!(event.title, "Reunión");
        assert_eq!(data.count_on(date(21)), 1);
        assert_eq!(data.events_on(date(21)).next().unwrap().id, id);
        assert!(data.resolve_style(event).visible);

        assert!(data.move_event_date(id, date(22)));
        assert_eq!(data.count_on(date(21)), 0);
        assert_eq!(data.count_on(date(22)), 1);
    }

    #[test]
    fn shortcuts_follow_priority_and_personal_override() {
        let mut data = sample();
        data.upsert_shortcut("Trabajo", "#111111", "#EEEEEE");
        let id = data.add_event("Informe", date(21), vec!["Trabajo".into()]);
        let event = data.event(id).unwrap();
        let style = data.resolve_style(event);
        assert_eq!(style.background, [0x11, 0x11, 0x11]);
        assert_eq!(style.shortcut.as_deref(), Some("Trabajo"));

        // El primer shortcut de la lista gana.
        data.upsert_shortcut("Personal", "#222222", "#DDDDDD");
        let id = data.add_event("Compras", date(22), vec!["Trabajo".into(), "Personal".into()]);
        let style = data.resolve_style(data.event(id).unwrap());
        assert_eq!(style.background, [0x11, 0x11, 0x11]);

        data.move_shortcut(1, 0);
        let style = data.resolve_style(data.event(id).unwrap());
        assert_eq!(style.background, [0x22, 0x22, 0x22]);

        let mut event = data.event(id).unwrap().clone();
        event
            .properties
            .insert(PERSONAL_PROPERTY.to_string(), Property::Checkbox(true));
        data.update_event(event.clone());
        let style = data.resolve_style(&event);
        assert_eq!(style.background, [0x40, 0x40, 0x40]);
        assert!(style.shortcut.is_none());
    }

    #[test]
    fn inactive_and_focused_categories_are_resolved() {
        let mut data = sample();
        let id = data.add_event("Cita", date(21), vec!["Trabajo".into(), "Personal".into()]);
        data.set_category_active("Trabajo", false);
        data.set_category_focused("Personal", true);
        let style = data.resolve_style(data.event(id).unwrap());
        assert!(!style.visible);
        assert!(style.focused);

        data.set_category_active("Trabajo", true);
        assert!(data.resolve_style(data.event(id).unwrap()).visible);
    }

    #[test]
    fn deleting_category_cleans_tags_and_shortcuts() {
        let mut data = sample();
        data.upsert_shortcut("Trabajo", "#111111", "#FFFFFF");
        let id = data.add_event("Informe", date(21), vec!["Trabajo".into()]);
        data.delete_category("Trabajo");
        assert!(!data.categories.contains_key("Trabajo"));
        assert!(data.shortcuts.is_empty());
        assert!(data.event(id).unwrap().tags.is_empty());
    }

    #[test]
    fn normalize_fixes_missing_categories_and_duplicate_ids() {
        let mut data = CalendarData::default();
        let mut first = Event::new(1, "Uno", date(1), vec!["Nueva".into()]);
        first.id = 4;
        let mut second = Event::new(4, "Dos", date(2), vec!["Nueva".into()]);
        second.id = 4;
        data.events.push(first);
        data.events.push(second);
        data.normalize();
        assert!(data.categories.contains_key("Nueva"));
        assert_eq!(data.event(4).unwrap().title, "Uno");
        assert_eq!(data.events.len(), 2);
        assert_ne!(data.events[0].id, data.events[1].id);
        assert!(data.events.iter().all(|event| event.id != 0));
        assert_eq!(data.next_id, 6);
    }

    #[test]
    fn search_ignores_accents_and_finds_tags_and_dates() {
        let mut data = sample();
        data.add_event("Reunión de diseño", date(21), vec!["Trabajo".into()]);
        data.add_event("Cumpleaños", date(25), vec!["Personal".into()]);
        assert_eq!(data.search("reunion").len(), 1);
        assert_eq!(data.search("personal").len(), 1);
        assert_eq!(data.search("21-10-2026").len(), 1);
        assert!(data.search("no existe").is_empty());
    }

    #[test]
    fn serialization_roundtrip_rebuilds_indexes() {
        let mut data = sample();
        let id = data.add_event("Viaje", date(21), vec!["Personal".into()]);
        let json = serde_json::to_string(&data).unwrap();
        let mut loaded: CalendarData = serde_json::from_str(&json).unwrap();
        loaded.normalize();
        assert_eq!(loaded.event(id).unwrap().title, "Viaje");
        assert_eq!(loaded.count_on(date(21)), 1);
    }

    #[test]
    fn hex_colors_are_flexible() {
        assert_eq!(parse_hex_color("#B42F2F"), Some([0xB4, 0x2F, 0x2F]));
        assert_eq!(parse_hex_color("b42f2f"), Some([0xB4, 0x2F, 0x2F]));
        assert_eq!(parse_hex_color("#abc"), Some([0xAA, 0xBB, 0xCC]));
        assert_eq!(parse_hex_color("#zzzzzz"), None);
    }
}
