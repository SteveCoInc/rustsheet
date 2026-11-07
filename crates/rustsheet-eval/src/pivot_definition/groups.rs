//! Discrete (manual) and date-level groups of a defined pivot (`tools/src/rustsheet_tools/
//! pivot_groups.py`, T-4170): the `discrete` and `date_group` field entries, written as Excel saves
//! them (`ClosedXML`'s Excel-authored pivot-table/shared-group-field.xlsx and
//! shared-nested-dategroup.xlsx; corpus-bench 52379 and 59932).
//!
//! A discrete group adds a `databaseField="0"` cache field whose `<fieldGroup base>` names the
//! source field, with a `<discretePr>` entry per source shared item and `<groupItems>` (the
//! ungrouped source items, then the groups); the source field's `<fieldGroup par>` names it. A date
//! group groups its field in place by its first level and adds one field per further level, the
//! source field's `par` naming the last one. Keep this module and `pivot_groups.py` in step: the
//! golden parts in `tools/tests/pivot_parts_golden/` pin both, and both refuse the same
//! definitions (a spec is untrusted input, directive 5: every list is bounded).

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

use rustsheet_engine::formatter::dates::excel_serial_to_ymd;
use serde_json::{Map, Value as Json};

use super::{cached_json, item_xml, key, Cached, DefinedPivot, Field, Key, Role, MAX_LIST};

/// `DATE_GROUPS`: `ST_GroupBy`'s date levels, finest first.
const LEVELS: [&str; 7] = [
    "seconds", "minutes", "hours", "days", "months", "quarters", "years",
];
/// `LEVEL_NAMES`: the name Excel gives the field it adds for each level of [`LEVELS`].
const LEVEL_NAMES: [&str; 7] = [
    "Seconds", "Minutes", "Hours", "Days", "Months", "Quarters", "Years",
];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
/// A leap year's month lengths: a day group always lists `29-Feb`.
const LEAP_MONTH_DAYS: [u32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
/// `_FIRST_SERIAL` (1900-03-01) and `_LAST_SERIAL` (9999-12-31): a date group's bounds.
const FIRST_SERIAL: f64 = 61.0;
const LAST_SERIAL: f64 = 2_958_465.0;
/// `GROUPINGS`: the field entry keys that group a field, one at most.
pub(super) const GROUPINGS: [&str; 3] = ["group", "discrete", "date_group"];
/// `_FIELD_KEYS`: the keys of a source field's entry.
const FIELD_KEYS: [&str; 11] = [
    "attrs",
    "hidden",
    "collapsed",
    "subtotals",
    "order",
    "item_order",
    "item_attrs",
    "xml",
    "group",
    "discrete",
    "date_group",
];
/// `ADDED_FIELD_KEYS`: the keys of an added field's entry.
const ADDED_FIELD_KEYS: [&str; 7] = [
    "attrs",
    "hidden",
    "collapsed",
    "subtotals",
    "item_attrs",
    "xml",
    "item_order",
];

/// `Discrete`: the added field's name, its group items, each source item's group item, and the
/// added pivotField's item order.
struct Discrete {
    name: String,
    labels: Vec<Option<Cached>>,
    groups: Vec<usize>,
    order: Vec<usize>,
}

/// `DateGroup`: its levels (finest first) and its `startDate` and `endDate` serials.
struct DateGroup {
    levels: Vec<usize>,
    start: f64,
    end: f64,
}

/// The year, month and day of a whole serial on the cache's OLE epoch (1899-12-30 is 0), which
/// from 1900-03-01 on is the cell calendar of [`excel_serial_to_ymd`]; below it the OLE date is
/// a day before the 1900 calendar's (serial 1 is 1899-12-31).
fn ole_ymd(days: i64) -> Option<(i32, u32, u32)> {
    match days {
        0 => Some((1899, 12, 30)),
        1 => Some((1899, 12, 31)),
        2..=60 => excel_serial_to_ymd(days - 1),
        _ => excel_serial_to_ymd(days),
    }
}

/// `iso_date`: a date serial as a pivot cache writes it, to the whole second (truncated).
pub(super) fn iso_date(serial: f64) -> Result<String, String> {
    if !(0.0..LAST_SERIAL + 1.0).contains(&serial) {
        return Err(format!(
            "a pivot date serial must be 0 to 9999-12-31: {serial}"
        ));
    }
    // In range, so a whole number of seconds below 2^38: exact as an i64 (Python's `int()`).
    #[allow(clippy::cast_possible_truncation, reason = "bounded above")]
    let seconds = (serial * 86_400.0) as i64;
    let (days, rest) = (seconds / 86_400, seconds % 86_400);
    let (y, m, d) = ole_ymd(days).ok_or_else(|| format!("a pivot date serial {serial}"))?;
    Ok(format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    ))
}

/// `date_shared_items`: a date field's `<sharedItems>`, cached inline.
pub(super) fn date_shared_items(values: &[Option<Cached>]) -> Result<String, String> {
    let numbers: Vec<f64> = (values.iter())
        .filter_map(|v| match v {
            Some(Cached::Number(n)) => Some(*n),
            _ => None,
        })
        .collect();
    if numbers.is_empty() {
        return Err("a date group needs a date".into());
    }
    let min = numbers.iter().copied().fold(f64::INFINITY, f64::min);
    let max = numbers.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let blank = if numbers.len() < values.len() {
        r#" containsBlank="1""#
    } else {
        ""
    };
    Ok(format!(
        r#"<sharedItems containsSemiMixedTypes="0" containsNonDate="0" containsDate="1" containsString="0"{blank} minDate="{}" maxDate="{}"/>"#,
        iso_date(min)?,
        iso_date(max)?
    ))
}

/// `date_labels`: Excel's group items for level `by` from `start` to `end`.
fn date_labels(by: usize, start: f64, end: f64) -> Result<Vec<String>, String> {
    #[allow(clippy::cast_possible_truncation, reason = "a bounded serial")]
    let day = |serial: f64| {
        ole_ymd(serial.trunc() as i64).ok_or_else(|| format!("a date group bound {serial}"))
    };
    let ((fy, fm, fd), (ly, lm, ld)) = (day(start)?, day(end)?);
    let inner: Vec<String> = match LEVELS.get(by).copied() {
        Some("seconds" | "minutes") => (0..60).map(|k| format!(":{k:02}")).collect(),
        Some("hours") => (0..24)
            .map(|h| {
                let half = if h < 12 { "AM" } else { "PM" };
                format!("{} {half}", if h % 12 == 0 { 12 } else { h % 12 })
            })
            .collect(),
        Some("days") => (MONTHS.iter().zip(LEAP_MONTH_DAYS))
            .flat_map(|(month, days)| (1..=days).map(move |d| format!("{d}-{month}")))
            .collect(),
        Some("months") => MONTHS.iter().map(ToString::to_string).collect(),
        Some("quarters") => (1..=4).map(|q| format!("Qtr{q}")).collect(),
        _ => (fy..=ly).map(|y| y.to_string()).collect(),
    };
    let mut labels = vec![format!("<{fm}/{fd}/{fy}")];
    labels.extend(inner);
    labels.push(format!(">{lm}/{ld}/{ly}"));
    Ok(labels)
}

/// An object holding exactly `keys`.
fn exactly<'j>(raw: &'j Json, keys: &[&str]) -> Option<&'j Map<String, Json>> {
    raw.as_object()
        .filter(|o| o.len() == keys.len() && keys.iter().all(|k| o.contains_key(*k)))
}

/// A list of 1 to [`MAX_LIST`] entries.
fn short_list(raw: Option<&Json>) -> Option<&Vec<Json>> {
    raw.and_then(Json::as_array)
        .filter(|l| !l.is_empty() && l.len() <= MAX_LIST)
}

/// `discrete`: the discrete group `raw` over a field whose shared items are `items`.
fn discrete(raw: &Json, items: &[Option<Cached>], place: &str) -> Result<Discrete, String> {
    let bad = |why: &str| format!("{place}: discrete {why}");
    let object = exactly(raw, &["name", "groups"]).ok_or_else(|| bad("is {name, groups}"))?;
    let name = (object.get("name").and_then(Json::as_str))
        .filter(|n| !n.is_empty())
        .ok_or_else(|| bad("name is a non-empty text"))?;
    let groups = short_list(object.get("groups"))
        .ok_or_else(|| bad(&format!("groups is a list of 1 to {MAX_LIST} groups")))?;
    let index: HashMap<Key, usize> = (items.iter().enumerate())
        .map(|(k, v)| (key(v.as_ref()), k))
        .collect();
    let mut owner: HashMap<usize, usize> = HashMap::new();
    let mut names = Vec::with_capacity(groups.len());
    for (g, group) in groups.iter().enumerate() {
        let group =
            exactly(group, &["name", "items"]).ok_or_else(|| bad("group is {name, items}"))?;
        let label = (group.get("name").and_then(Json::as_str))
            .filter(|n| !n.is_empty())
            .ok_or_else(|| bad("group name is a non-empty text"))?;
        let members = short_list(group.get("items")).ok_or_else(|| {
            bad(&format!(
                "group {label:?} items is a list of 1 to {MAX_LIST} items"
            ))
        })?;
        for member in members {
            let k = *(index.get(&key(cached_json(member)?.as_ref())))
                .ok_or_else(|| bad(&format!("item {member} is not in the field")))?;
            if owner.insert(k, g).is_some() {
                return Err(bad(&format!("item {member} is in two groups")));
            }
        }
        names.push(Some(Cached::Text(label.to_string())));
    }
    let loose: Vec<usize> = (0..items.len())
        .filter(|k| !owner.contains_key(k))
        .collect();
    let labels: Vec<Option<Cached>> = (loose.iter())
        .filter_map(|&k| items.get(k).cloned())
        .chain(names)
        .collect();
    let distinct: HashSet<Key> = labels.iter().map(|v| key(v.as_ref())).collect();
    if distinct.len() != labels.len() {
        return Err(bad(
            "group names must differ from each other and from the ungrouped items",
        ));
    }
    let at: HashMap<usize, usize> = loose.iter().enumerate().map(|(n, &k)| (k, n)).collect();
    let of: Vec<usize> = (0..items.len())
        .map(|k| match at.get(&k) {
            Some(&n) => n,
            None => loose.len() + owner.get(&k).copied().unwrap_or_default(),
        })
        .collect();
    let mut seen = HashSet::new();
    let order = of.iter().copied().filter(|g| seen.insert(*g)).collect();
    Ok(Discrete {
        name: name.to_string(),
        labels,
        groups: of,
        order,
    })
}

/// `discrete_xml`: the added field's `<fieldGroup>`.
fn discrete_xml(base: usize, group: &Discrete) -> String {
    let xs = group.groups.iter().fold(String::new(), |mut out, k| {
        let _ = write!(out, r#"<x v="{k}"/>"#);
        out
    });
    let items: String = (group.labels.iter())
        .map(|v| item_xml(v.as_ref()))
        .collect();
    format!(
        r#"<fieldGroup base="{base}"><discretePr count="{}">{xs}</discretePr><groupItems count="{}">{items}</groupItems></fieldGroup>"#,
        group.groups.len(),
        group.labels.len()
    )
}

/// `date_group`: the date group `raw`.
fn date_group(raw: &Json, place: &str) -> Result<DateGroup, String> {
    let bad = |why: &str| format!("{place}: date_group {why}");
    let object =
        exactly(raw, &["levels", "start", "end"]).ok_or_else(|| bad("is {levels, start, end}"))?;
    let levels = (short_list(object.get("levels")))
        .and_then(|l| {
            (l.iter())
                .map(|v| v.as_str().and_then(|s| LEVELS.iter().position(|l| *l == s)))
                .collect::<Option<Vec<usize>>>()
        })
        .ok_or_else(|| bad(&format!("levels lists some of {LEVELS:?}")))?;
    if levels.windows(2).any(|w| w[0] >= w[1]) {
        return Err(bad(
            "levels are distinct and run from the finest to the coarsest",
        ));
    }
    let bound = |name: &str| {
        (object.get(name))
            .filter(|v| v.is_number())
            .and_then(Json::as_f64)
            .ok_or_else(|| bad("start and end are date serials"))
    };
    let (start, end) = (bound("start")?, bound("end")?);
    if !(FIRST_SERIAL <= start && start < end && end <= LAST_SERIAL) {
        return Err(bad(&format!(
            "needs {FIRST_SERIAL} <= start < end <= {LAST_SERIAL} (1900-03-01 to 9999-12-31)"
        )));
    }
    Ok(DateGroup { levels, start, end })
}

/// `date_xml`: one level's `<fieldGroup>`, `par` on the field grouped in place.
fn date_xml(
    by: usize,
    group: &DateGroup,
    base: usize,
    par: Option<usize>,
) -> Result<String, String> {
    let labels = date_labels(by, group.start, group.end)?;
    let items = labels.iter().fold(String::new(), |mut out, label| {
        let _ = write!(out, "{}", item_xml(Some(&Cached::Text(label.clone()))));
        out
    });
    let parent = par.map_or_else(String::new, |p| format!(r#" par="{p}""#));
    Ok(format!(
        r#"<fieldGroup{parent} base="{base}"><rangePr autoStart="0" autoEnd="0" groupBy="{}" startDate="{}" endDate="{}"/><groupItems count="{}">{items}</groupItems></fieldGroup>"#,
        LEVELS.get(by).copied().unwrap_or_default(),
        iso_date(group.start)?,
        iso_date(group.end)?,
        labels.len()
    ))
}

/// Python's truthiness of a definition value (`calculated`, `calculated_items`).
fn truthy(value: Option<&Json>) -> bool {
    match value {
        None | Some(Json::Null | Json::Bool(false)) => false,
        Some(Json::Array(a)) => !a.is_empty(),
        Some(Json::Object(o)) => !o.is_empty(),
        Some(Json::String(s)) => !s.is_empty(),
        Some(Json::Number(n)) => n.as_f64() != Some(0.0),
        Some(Json::Bool(true)) => true,
    }
}

/// A field a group adds, whose items are `labels` (`databaseField="0"`).
fn added_field<'a>(name: String, labels: Vec<Option<Cached>>, group_xml: String) -> Field<'a> {
    let mut field = Field::new(name, Vec::new(), None);
    field.items.clone_from(&labels);
    field.labels = Some(labels);
    field.role = Role::Added;
    field.group_xml = group_xml;
    field
}

impl DefinedPivot<'_> {
    /// `_read_fields`: a source field's entry first (with its `order`), then the groups, then
    /// the entries of the fields they add.
    pub(super) fn read_fields(&mut self) -> Result<(), String> {
        let entries = self.d.get("fields").and_then(Json::as_object);
        if entries.is_some_and(|e| e.len() > MAX_LIST) {
            return Err(format!(
                "pivot {}: more than {MAX_LIST} fields entries",
                self.name
            ));
        }
        let mut added = Vec::new();
        for (field, entry) in entries.into_iter().flatten() {
            let object = (entry.as_object())
                .filter(|o| o.keys().all(|k| FIELD_KEYS.contains(&k.as_str())))
                .ok_or_else(|| {
                    format!(
                        "pivot {}: field {field:?} entry has unknown keys",
                        self.name
                    )
                })?;
            let Some(index) = self.find(field) else {
                added.push((field, object));
                continue;
            };
            let target = &mut self.fields[index];
            target.spec = Some(object);
            if let Some(order) = target.entry("order").and_then(Json::as_array) {
                target.reorder(order)?;
            }
        }
        self.add_groups()?;
        for (field, object) in added {
            let index = self.index(field)?;
            let target = &mut self.fields[index];
            if target.role != Role::Added
                || !object
                    .keys()
                    .all(|k| ADDED_FIELD_KEYS.contains(&k.as_str()))
            {
                return Err(format!(
                    "pivot {}: an added group field's entry takes only {ADDED_FIELD_KEYS:?}: {field:?}",
                    self.name
                ));
            }
            target.spec = Some(object);
        }
        Ok(())
    }

    /// `_add_groups`: each discrete or date group groups its field and appends the fields it
    /// adds, in source-field order.
    fn add_groups(&mut self) -> Result<(), String> {
        for i in 0..self.fields.len() {
            let field = &self.fields[i];
            let kinds: Vec<&str> = (GROUPINGS.iter().copied())
                .filter(|k| field.entry(k).is_some())
                .collect();
            if kinds.len() > 1 {
                return Err(format!(
                    "pivot {}: field {:?} takes one of {GROUPINGS:?}",
                    self.name, field.name
                ));
            }
            let (Some(&kind), Some(raw)) =
                (kinds.first(), kinds.first().and_then(|k| field.entry(k)))
            else {
                continue;
            };
            if kind == "group" {
                continue;
            }
            if truthy(self.d.get("calculated")) || truthy(self.d.get("calculated_items")) {
                return Err(format!(
                    "pivot {}: calculated fields and items do not combine with discrete or date groups",
                    self.name
                ));
            }
            let place = format!("pivot {} field {:?}", self.name, field.name);
            let at = self.fields.len();
            if kind == "discrete" {
                let group = discrete(raw, &field.items, &place)?;
                let added = added_field(
                    group.name.clone(),
                    group.labels.clone(),
                    discrete_xml(i, &group),
                );
                let field = &mut self.fields[i];
                field.role = Role::DiscreteBase;
                field.group_xml = format!(r#"<fieldGroup par="{at}"/>"#);
                self.fields.push(Field {
                    item_order: Some(group.order),
                    ..added
                });
                continue;
            }
            let dates = date_group(raw, &place)?;
            if !field.numeric_only() {
                return Err(format!("{place}: a date group's field holds date serials"));
            }
            let (first, more) = dates.levels.split_first().ok_or("a date group's levels")?;
            let labels = date_labels(*first, dates.start, dates.end)?;
            for &by in more {
                let name = LEVEL_NAMES.get(by).copied().unwrap_or_default().to_string();
                let level = date_labels(by, dates.start, dates.end)?;
                let level = level.into_iter().map(|l| Some(Cached::Text(l))).collect();
                self.fields
                    .push(added_field(name, level, date_xml(by, &dates, i, None)?));
            }
            let par = (!more.is_empty()).then(|| self.fields.len() - 1);
            let field = &mut self.fields[i];
            field.role = Role::Dates;
            field.labels = Some(labels.into_iter().map(|l| Some(Cached::Text(l))).collect());
            field.group_xml = date_xml(*first, &dates, i, par)?;
        }
        let names: HashSet<String> = self.fields.iter().map(|f| f.name.to_lowercase()).collect();
        if names.len() != self.fields.len() {
            return Err(format!(
                "pivot {}: group field names must differ from the other fields",
                self.name
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "groups_tests.rs"]
mod tests;
