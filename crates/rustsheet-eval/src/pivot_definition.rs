//! A spec pivot's `definition` (case builder v2, T-4122) as the package parts the case builder
//! writes for it (`tools/src/rustsheet_tools/pivot_definition.py`), so the engine adapter loads it
//! through the xlsx importer and the engine renders it on its load-time refresh, as Excel renders
//! the built file on open (T-4152, SPEC-0032 rollout step 7).
//!
//! It mirrors the builder part for part, attribute for attribute: what Excel opened is what the
//! engine loads. The builder validates a definition (spec2xlsx refuses a bad one, and no case
//! reaches the oracle without it), so a definition it would refuse is only an error here, with
//! its checks not repeated. Keep the two in step: a change to the builder's parts is a change here.

use std::collections::HashMap;
use std::fmt::Write as _;

use quick_xml::escape::escape;
use rustsheet_engine::pivot::Aggregate;
use serde_json::{Map, Value as Json};

use crate::case::{split_qualified, CaseSpec, CellInput, PivotTableSpec, SheetSpec};
use crate::cellref::{parse_range, Addr};
use crate::value::Value;

const MAIN_NS: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
const REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
/// The Σ (data) field on an axis: `<field x="-2"/>`.
const VALUES: &str = "Values";
const VALUES_INDEX: i64 = -2;
/// pivotTableDefinition attributes written unless `table` sets them (the builder's
/// `_TABLE_DEFAULTS`, Excel's own new-pivot values), in its order, each `name=value`.
const TABLE_DEFAULTS: &str = "dataCaption=Values updatedVersion=6 minRefreshableVersion=3 createdVersion=6 applyNumberFormats=0 applyBorderFormats=0 applyFontFormats=0 applyPatternFormats=0 applyAlignmentFormats=0 applyWidthHeightFormats=1 useAutoFormatting=1 itemPrintTitles=1 indent=0 outline=1 outlineData=1";
/// `CT_pivotTableDefinition`'s children after `dataFields`, in schema order: the raw-XML slots
/// (`_SLOTS`), the first four before `pivotTableStyleInfo`.
const SLOTS: [&str; 6] = [
    "formats",
    "conditionalFormats",
    "chartFormats",
    "pivotHierarchies",
    "filters",
    "extLst",
];
/// Most numeric group bins the builder writes (`group_labels`).
const MAX_BINS: f64 = 10_000.0;

/// The three parts of a defined pivot: its cache definition and records, and its table.
pub(crate) struct Parts {
    pub(crate) cache_definition: String,
    pub(crate) cache_records: String,
    pub(crate) table: String,
}

/// A value a source cell contributes to the builder's cache (`cached_value`); `None` is a blank
/// or a value the builder does not know (a formula: Excel computes the cache again on open).
#[derive(Clone, Debug, PartialEq)]
enum Cached {
    Text(String),
    Number(f64),
    Bool(bool),
    Error(String),
}

/// A cache item's identity in the builder's shared items (`_key`): texts that differ only in
/// case are one item.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    Missing,
    Text(String),
    Number(u64),
    Bool(bool),
    Error(String),
}

fn key(value: Option<&Cached>) -> Key {
    match value {
        None => Key::Missing,
        // Python's `casefold`; `to_lowercase` differs from it only past ASCII (`ß`).
        Some(Cached::Text(text)) => Key::Text(text.to_lowercase()),
        Some(Cached::Number(n)) => Key::Number(n.to_bits()),
        Some(Cached::Bool(b)) => Key::Bool(*b),
        Some(Cached::Error(e)) => Key::Error(e.clone()),
    }
}

fn cached(input: Option<&CellInput>) -> Result<Option<Cached>, String> {
    if let Some(CellInput::Str(text)) = input {
        if text.is_empty() || text.starts_with('=') {
            return Ok(None);
        }
    }
    Ok(match input {
        None
        | Some(
            CellInput::Entered { .. }
            | CellInput::EnteredR1c1 { .. }
            | CellInput::Typed(Value::Empty),
        ) => None,
        Some(CellInput::Str(text) | CellInput::Typed(Value::Text(text))) => {
            Some(Cached::Text(text.clone()))
        }
        Some(CellInput::Number(n) | CellInput::Typed(Value::Number(n))) => {
            if !n.is_finite() {
                return Err(format!("pivot source value {n} is not finite"));
            }
            Some(Cached::Number(*n))
        }
        Some(CellInput::Bool(b) | CellInput::Typed(Value::Bool(b))) => Some(Cached::Bool(*b)),
        Some(CellInput::Typed(Value::Error(e))) => Some(Cached::Error(e.clone())),
        Some(CellInput::StoredError(e)) => Some(Cached::Error(e.v.clone())),
    })
}

/// [`cached`] of a value written in the definition (an `order`, `hidden` or page item).
fn cached_json(raw: &Json) -> Result<Option<Cached>, String> {
    if raw.is_null() {
        return Ok(None);
    }
    let input: CellInput =
        serde_json::from_value(raw.clone()).map_err(|e| format!("pivot item {raw}: {e}"))?;
    cached(Some(&input))
}

/// A cached number's text (`number_text`): a whole number below 1e15 without a fraction, else
/// the shortest text that reads back as it.
fn number_text(value: f64) -> String {
    if value == 0.0 {
        "0".to_string()
    } else if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{value:.0}")
    } else {
        format!("{value:?}")
    }
}

fn item_xml(value: Option<&Cached>) -> String {
    match value {
        None => "<m/>".to_string(),
        Some(Cached::Bool(b)) => format!(r#"<b v="{}"/>"#, u8::from(*b)),
        Some(Cached::Number(n)) => format!(r#"<n v="{}"/>"#, number_text(*n)),
        Some(Cached::Error(e)) => format!(r#"<e v="{}"/>"#, escape(e)),
        Some(Cached::Text(t)) => format!(r#"<s v="{}"/>"#, escape(t)),
    }
}

/// An author attribute's text (`_attr_value`).
fn attr_value(value: &Json) -> Result<String, String> {
    match value {
        Json::Bool(b) => Ok(if *b { "1" } else { "0" }.to_string()),
        Json::Number(n) => n
            .as_f64()
            .map(number_text)
            .ok_or_else(|| format!("attribute value {n}")),
        Json::String(s) => Ok(s.clone()),
        other => Err(format!(
            "attribute value {other} is not a string, number or boolean"
        )),
    }
}

/// ` k="v"` for each author attribute of `raw` (`_attrs`).
fn attrs(raw: Option<&Json>) -> Result<String, String> {
    let Some(raw) = raw.filter(|r| !r.is_null()) else {
        return Ok(String::new());
    };
    let object = raw.as_object().ok_or("attributes are an object")?;
    let mut out = String::new();
    for (name, value) in object {
        let _ = write!(out, r#" {name}="{}""#, escape(&attr_value(value)?));
    }
    Ok(out)
}

fn number(value: Option<&Json>) -> f64 {
    value.and_then(Json::as_f64).unwrap_or(0.0)
}

/// One cache field and the pivot's use of it (`_Field`).
struct Field<'a> {
    name: String,
    /// Per record; empty for a calculated field.
    values: Vec<Option<Cached>>,
    formula: Option<String>,
    /// Distinct values, first-seen order (or `order`'s).
    items: Vec<Option<Cached>>,
    /// Its `fields` entry.
    spec: Option<&'a Map<String, Json>>,
    axis: Option<&'static str>,
    data: bool,
}

impl<'a> Field<'a> {
    fn new(name: String, values: Vec<Option<Cached>>, formula: Option<String>) -> Self {
        let mut seen = std::collections::HashSet::new();
        let items = (values.iter())
            .filter(|v| seen.insert(key(v.as_ref())))
            .cloned()
            .collect();
        Field {
            name,
            values,
            formula,
            items,
            spec: None,
            axis: None,
            data: false,
        }
    }

    fn entry(&self, name: &str) -> Option<&'a Json> {
        self.spec.and_then(|s| s.get(name))
    }

    /// Puts the items `order` lists first, in that order (`reorder`).
    fn reorder(&mut self, order: &[Json]) -> Result<(), String> {
        let keys = (order.iter())
            .map(|raw| cached_json(raw).map(|v| key(v.as_ref())))
            .collect::<Result<Vec<_>, _>>()?;
        let mut first = Vec::with_capacity(keys.len());
        for k in &keys {
            let item = (self.items.iter())
                .find(|v| key(v.as_ref()) == *k)
                .ok_or_else(|| format!("an order item is not in field {:?}", self.name))?;
            first.push(item.clone());
        }
        let rest = (self.items.iter()).filter(|v| !keys.contains(&key(v.as_ref())));
        self.items = first.into_iter().chain(rest.cloned()).collect();
        Ok(())
    }

    fn numeric_only(&self) -> bool {
        (self.values.iter()).all(|v| matches!(v, None | Some(Cached::Number(_))))
    }

    fn group(&self) -> Option<&'a Map<String, Json>> {
        self.entry("group").and_then(Json::as_object)
    }

    /// Whether records refer to the shared items by index (`listed`).
    fn listed(&self) -> bool {
        !(self.numeric_only() && (self.axis.is_none() || self.group().is_some()))
    }

    /// `<sharedItems>` (`shared_items`): the flags always, the list when records refer to it.
    fn shared_items(&self) -> String {
        let strings = (self.values.iter()).any(|v| {
            matches!(
                v,
                Some(Cached::Text(_) | Cached::Bool(_) | Cached::Error(_))
            )
        });
        let numbers: Vec<f64> = (self.values.iter())
            .filter_map(|v| match v {
                Some(Cached::Number(n)) => Some(*n),
                _ => None,
            })
            .collect();
        let blank = self.values.iter().any(Option::is_none);
        let mut flags = String::new();
        if !strings && !blank {
            flags.push_str(r#" containsSemiMixedTypes="0""#);
        }
        if !strings {
            flags.push_str(r#" containsString="0""#);
        }
        if strings && !numbers.is_empty() {
            flags.push_str(r#" containsMixedTypes="1""#);
        }
        if !numbers.is_empty() {
            flags.push_str(r#" containsNumber="1""#);
            if numbers.iter().all(|n| n.fract() == 0.0) {
                flags.push_str(r#" containsInteger="1""#);
            }
            let min = numbers.iter().copied().fold(f64::INFINITY, f64::min);
            let max = numbers.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let _ = write!(
                flags,
                r#" minValue="{}" maxValue="{}""#,
                number_text(min),
                number_text(max)
            );
        }
        if blank {
            flags.push_str(r#" containsBlank="1""#);
        }
        if !self.listed() {
            return format!("<sharedItems{flags}/>");
        }
        let items: String = self.items.iter().map(|v| item_xml(v.as_ref())).collect();
        format!(
            r#"<sharedItems{flags} count="{}">{items}</sharedItems>"#,
            self.items.len()
        )
    }

    /// Numeric group items as the builder writes them (`group_labels`); Excel relabels them.
    fn group_labels(&self) -> Result<Vec<String>, String> {
        let group = self.group();
        let get = |name: &str| number(group.and_then(|g| g.get(name)));
        let (start, end, step) = (get("start"), get("end"), get("interval"));
        if step <= 0.0 || end < start || (end - start) / step > MAX_BINS {
            return Err(format!(
                "pivot field {:?} group needs 0 < interval and at most 10,000 bins",
                self.name
            ));
        }
        let mut labels = vec![format!("<{}", number_text(start))];
        let mut low = start;
        while low <= end {
            labels.push(format!("{}-{}", number_text(low), number_text(low + step)));
            low += step;
        }
        labels.push(format!(">{}", number_text(end)));
        Ok(labels)
    }
}

/// A data field (`_data`'s entries).
struct DataEntry<'a> {
    field: usize,
    subtotal: String,
    name: String,
    raw: &'a Map<String, Json>,
}

/// A pivot built from a `definition` (`DefinedPivot`).
struct DefinedPivot<'a> {
    name: &'a str,
    d: &'a Map<String, Json>,
    fields: Vec<Field<'a>>,
    records: usize,
    rows: Vec<i64>,
    columns: Vec<i64>,
    /// Each page field and the key of its selected item (none: `(All)`).
    pages: Vec<(usize, Option<Key>)>,
    data: Vec<DataEntry<'a>>,
    refresh_on_load: bool,
}

impl<'a> DefinedPivot<'a> {
    fn new(
        name: &'a str,
        d: &'a Map<String, Json>,
        header: Vec<String>,
        rows: &[Vec<Option<Cached>>],
    ) -> Result<Self, String> {
        let mut fields: Vec<Field<'a>> = (header.into_iter().enumerate())
            .map(|(i, h)| {
                let values = rows.iter().map(|r| r.get(i).cloned().flatten()).collect();
                Field::new(h, values, None)
            })
            .collect();
        for calc in d
            .get("calculated")
            .and_then(Json::as_array)
            .into_iter()
            .flatten()
        {
            let text = |k: &str| calc.get(k).and_then(Json::as_str).map(str::to_string);
            let (Some(field), Some(formula)) = (text("name"), text("formula")) else {
                return Err(format!(
                    "pivot {name}: a calculated field is {{name, formula}}"
                ));
            };
            fields.push(Field::new(field, Vec::new(), Some(formula)));
        }
        let mut pivot = DefinedPivot {
            name,
            d,
            fields,
            records: rows.len(),
            rows: Vec::new(),
            columns: Vec::new(),
            pages: Vec::new(),
            data: Vec::new(),
            refresh_on_load: d
                .get("refresh_on_load")
                .and_then(Json::as_bool)
                .unwrap_or(true),
        };
        for (field, entry) in d
            .get("fields")
            .and_then(Json::as_object)
            .into_iter()
            .flatten()
        {
            let index = pivot.index(field)?;
            let target = &mut pivot.fields[index];
            target.spec = entry.as_object();
            if let Some(order) = target.entry("order").and_then(Json::as_array) {
                target.reorder(order)?;
            }
        }
        pivot.rows = pivot.axis("rows", "axisRow")?;
        pivot.columns = pivot.axis("columns", "axisCol")?;
        pivot.read_pages()?;
        pivot.read_data()?;
        // Several data fields need the Values field: on the columns unless placed (`_values_field`).
        let placed = pivot.rows.contains(&VALUES_INDEX) || pivot.columns.contains(&VALUES_INDEX);
        if pivot.data.len() > 1 && !placed {
            pivot.columns.push(VALUES_INDEX);
        }
        Ok(pivot)
    }

    fn index(&self, name: &str) -> Result<usize, String> {
        let folded = name.to_lowercase();
        (self.fields.iter())
            .position(|f| f.name.to_lowercase() == folded)
            .ok_or_else(|| format!("pivot {}: no field {name:?}", self.name))
    }

    fn list(&self, key: &str) -> &'a [Json] {
        self.d
            .get(key)
            .and_then(Json::as_array)
            .map_or(&[], Vec::as_slice)
    }

    fn axis(&mut self, key: &str, axis: &'static str) -> Result<Vec<i64>, String> {
        let mut out = Vec::new();
        for name in self.list(key) {
            let name = name.as_str().ok_or("an axis lists field names")?;
            if name == VALUES {
                out.push(VALUES_INDEX);
                continue;
            }
            let i = self.index(name)?;
            self.fields[i].axis = Some(axis);
            out.push(i64::try_from(i).map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    fn read_pages(&mut self) -> Result<(), String> {
        for page in self.list("pages") {
            let field = page
                .get("field")
                .and_then(Json::as_str)
                .ok_or("a page names a field")?;
            let i = self.index(field)?;
            self.fields[i].axis = Some("axisPage");
            let item = (page.get("item"))
                .map(|raw| cached_json(raw).map(|v| key(v.as_ref())))
                .transpose()?;
            self.pages.push((i, item));
        }
        Ok(())
    }

    fn read_data(&mut self) -> Result<(), String> {
        for entry in self.list("data") {
            let raw = entry.as_object().ok_or("a data field is an object")?;
            let field = raw
                .get("field")
                .and_then(Json::as_str)
                .ok_or("a data field names a field")?;
            let i = self.index(field)?;
            let subtotal = raw.get("subtotal").and_then(Json::as_str).unwrap_or("sum");
            let verb = Aggregate::from_consolidate_name(subtotal)
                .ok_or_else(|| format!("subtotal {subtotal:?}"))?
                .word();
            self.fields[i].data = true;
            let name = match raw.get("name").and_then(Json::as_str) {
                Some(name) if !name.is_empty() => name.to_string(),
                _ => format!("{verb} of {}", self.fields[i].name),
            };
            self.data.push(DataEntry {
                field: i,
                subtotal: subtotal.to_string(),
                name,
                raw,
            });
        }
        Ok(())
    }

    fn cache_definition(&self, source_sheet: &str, source_ref: &str) -> Result<String, String> {
        let mut fields = String::new();
        for (index, f) in self.fields.iter().enumerate() {
            let name = escape(&f.name);
            if let Some(formula) = &f.formula {
                let _ = write!(
                    fields,
                    r#"<cacheField name="{name}" numFmtId="0" formula="{}" databaseField="0"/>"#,
                    escape(formula)
                );
                continue;
            }
            let mut group = String::new();
            if let Some(g) = f.group() {
                let labels = f.group_labels()?;
                let items = labels.iter().fold(String::new(), |mut out, label| {
                    let _ = write!(out, r#"<s v="{}"/>"#, escape(label));
                    out
                });
                group = format!(
                    r#"<fieldGroup base="{index}"><rangePr autoStart="0" autoEnd="0" startNum="{}" endNum="{}" groupInterval="{}"/><groupItems count="{}">{items}</groupItems></fieldGroup>"#,
                    number_text(number(g.get("start"))),
                    number_text(number(g.get("end"))),
                    number_text(number(g.get("interval"))),
                    labels.len()
                );
            }
            let _ = write!(
                fields,
                r#"<cacheField name="{name}" numFmtId="0">{}{group}</cacheField>"#,
                f.shared_items()
            );
        }
        let extra = attrs(self.d.get("cache"))?;
        let refresh = if self.refresh_on_load {
            r#" refreshOnLoad="1""#
        } else {
            ""
        };
        Ok(format!(
            r#"<pivotCacheDefinition xmlns="{MAIN_NS}" xmlns:r="{REL_NS}" r:id="rId1"{refresh} createdVersion="6" refreshedVersion="6" minRefreshableVersion="3" recordCount="{}"{extra}><cacheSource type="worksheet"><worksheetSource ref="{source_ref}" sheet="{}"/></cacheSource><cacheFields count="{}">{fields}</cacheFields></pivotCacheDefinition>"#,
            self.records,
            escape(source_sheet),
            self.fields.len()
        ))
    }

    fn cache_records(&self) -> String {
        let database: Vec<&Field<'_>> =
            self.fields.iter().filter(|f| f.formula.is_none()).collect();
        let index: Vec<HashMap<Key, usize>> = (database.iter())
            .map(|f| {
                let keys = f
                    .items
                    .iter()
                    .enumerate()
                    .map(|(k, v)| (key(v.as_ref()), k));
                keys.collect()
            })
            .collect();
        let mut rows = String::new();
        for r in 0..self.records {
            rows.push_str("<r>");
            for (f, at) in database.iter().zip(&index) {
                let value = f.values.get(r).and_then(Option::as_ref);
                if let Some(k) = at.get(&key(value)).filter(|_| f.listed()) {
                    let _ = write!(rows, r#"<x v="{k}"/>"#);
                } else {
                    rows.push_str(&item_xml(value));
                }
            }
            rows.push_str("</r>");
        }
        format!(
            r#"<pivotCacheRecords xmlns="{MAIN_NS}" count="{}">{rows}</pivotCacheRecords>"#,
            self.records
        )
    }

    fn pivot_field(f: &Field<'_>) -> Result<String, String> {
        let extra = attrs(f.entry("attrs"))?;
        let mut head = String::new();
        if let Some(axis) = f.axis {
            let _ = write!(head, r#" axis="{axis}""#);
        }
        if f.data {
            head.push_str(r#" dataField="1""#);
        }
        if f.formula.is_some() {
            head.push_str(r#" dragToRow="0" dragToCol="0" dragToPage="0""#);
        }
        let subtotals: Option<Vec<&str>> = f
            .entry("subtotals")
            .and_then(Json::as_array)
            .map(|s| s.iter().filter_map(Json::as_str).collect());
        if let Some(subtotals) = &subtotals {
            head.push_str(r#" defaultSubtotal="0""#);
            for s in subtotals {
                let _ = write!(head, r#" {s}Subtotal="1""#);
            }
        }
        let hidden = f.entry("hidden").and_then(Json::as_array);
        if hidden.is_some_and(|h| !h.is_empty()) && f.axis == Some("axisPage") {
            head.push_str(r#" multipleItemSelectionAllowed="1""#);
        }
        if f.axis.is_none() {
            return Ok(format!(r#"<pivotField{head} showAll="0"{extra}/>"#));
        }
        let values: Vec<Option<Cached>> = if f.group().is_some() {
            (f.group_labels()?.into_iter())
                .map(|label| Some(Cached::Text(label)))
                .collect()
        } else {
            f.items.clone()
        };
        let keys: HashMap<Key, usize> = (values.iter().enumerate())
            .map(|(k, v)| (key(v.as_ref()), k))
            .collect();
        let mut flags: HashMap<usize, String> = HashMap::new();
        for (kind, attr) in [("hidden", r#" h="1""#), ("collapsed", r#" sd="0""#)] {
            for raw in f.entry(kind).and_then(Json::as_array).into_iter().flatten() {
                let k = keys
                    .get(&key(cached_json(raw)?.as_ref()))
                    .ok_or_else(|| format!("{kind} item {raw} is not in field {:?}", f.name))?;
                flags.entry(*k).or_default().push_str(attr);
            }
        }
        let mut items = (0..values.len()).fold(String::new(), |mut out, k| {
            let flag = flags.get(&k).map_or("", String::as_str);
            let _ = write!(out, r#"<item x="{k}"{flag}/>"#);
            out
        });
        let tails = subtotals.unwrap_or_else(|| vec!["default"]);
        for t in &tails {
            let _ = write!(items, r#"<item t="{t}"/>"#);
        }
        Ok(format!(
            r#"<pivotField{head} showAll="0"{extra}><items count="{}">{items}</items></pivotField>"#,
            values.len() + tails.len()
        ))
    }

    fn page_field(&self, field: usize, item: Option<&Key>) -> Result<String, String> {
        let Some(wanted) = item else {
            return Ok(format!(r#"<pageField fld="{field}" hier="-1"/>"#));
        };
        let k = (self.fields[field].items.iter())
            .position(|v| key(v.as_ref()) == *wanted)
            .ok_or("a page item is not in its field")?;
        Ok(format!(
            r#"<pageField fld="{field}" item="{k}" hier="-1"/>"#
        ))
    }

    fn data_field(&self, e: &DataEntry<'_>) -> Result<String, String> {
        let mut attrs = format!(r#" name="{}" fld="{}""#, escape(&e.name), e.field);
        if e.subtotal != "sum" {
            let _ = write!(attrs, r#" subtotal="{}""#, e.subtotal);
        }
        if let Some(show) = e.raw.get("showDataAs").and_then(Json::as_str) {
            let _ = write!(attrs, r#" showDataAs="{show}""#);
        }
        let base = match e.raw.get("baseField") {
            Some(Json::String(name)) => self.index(name)?.to_string(),
            Some(other) => other.as_i64().unwrap_or(0).to_string(),
            None => "0".to_string(),
        };
        let base_item = e.raw.get("baseItem").and_then(Json::as_i64).unwrap_or(0);
        let _ = write!(attrs, r#" baseField="{base}" baseItem="{base_item}""#);
        if let Some(format) = e.raw.get("numFmtId").and_then(Json::as_i64) {
            let _ = write!(attrs, r#" numFmtId="{format}""#);
        }
        Ok(format!("<dataField{attrs}/>"))
    }

    /// The author's XML for `slot`, as written (`_raw`).
    fn raw(&self, slot: &str) -> String {
        (self.d.get("xml").and_then(|x| x.get(slot)))
            .and_then(Json::as_str)
            .map_or_else(String::new, |text| text.trim().to_string())
    }

    fn table_definition(&self, cache_id: u32, anchor: Addr) -> Result<String, String> {
        let table = self.d.get("table").and_then(Json::as_object);
        let extra = attrs(self.d.get("table"))?;
        let mut head = (TABLE_DEFAULTS.split(' '))
            .filter_map(|pair| pair.split_once('='))
            .filter(|(k, _)| !table.is_some_and(|t| t.contains_key(*k)))
            .fold(String::new(), |mut out, (k, v)| {
                let _ = write!(out, r#" {k}="{v}""#);
                out
            });
        if self.rows.contains(&VALUES_INDEX) {
            head.push_str(r#" dataOnRows="1""#);
        }
        let corner = Addr {
            row: anchor.row + 1,
            col: anchor.col + 1,
        };
        let area = (self.d.get("area").and_then(Json::as_str))
            .map_or_else(|| format!("{anchor}:{corner}"), str::to_string);
        let pages = if self.pages.is_empty() {
            String::new()
        } else {
            format!(r#" rowPageCount="{}" colPageCount="1""#, self.pages.len())
        };
        let mut parts = format!(
            r#"<location ref="{area}" firstHeaderRow="1" firstDataRow="1" firstDataCol="1"{pages}/>"#
        );
        let fields = (self.fields.iter())
            .map(Self::pivot_field)
            .collect::<Result<String, String>>()?;
        let _ = write!(
            parts,
            r#"<pivotFields count="{}">{fields}</pivotFields>"#,
            self.fields.len()
        );
        for (tag, axis, lines) in [
            ("rowFields", &self.rows, "rowItems"),
            ("colFields", &self.columns, "colItems"),
        ] {
            if !axis.is_empty() {
                let fields = axis.iter().fold(String::new(), |mut out, x| {
                    let _ = write!(out, r#"<field x="{x}"/>"#);
                    out
                });
                let _ = write!(parts, r#"<{tag} count="{}">{fields}</{tag}>"#, axis.len());
            }
            parts.push_str(&self.raw(lines));
        }
        if !self.pages.is_empty() {
            let pages = (self.pages.iter())
                .map(|(field, item)| self.page_field(*field, item.as_ref()))
                .collect::<Result<String, String>>()?;
            let _ = write!(
                parts,
                r#"<pageFields count="{}">{pages}</pageFields>"#,
                self.pages.len()
            );
        }
        let data = (self.data.iter())
            .map(|e| self.data_field(e))
            .collect::<Result<String, String>>()?;
        let _ = write!(
            parts,
            r#"<dataFields count="{}">{data}</dataFields>"#,
            self.data.len()
        );
        for slot in &SLOTS[..4] {
            parts.push_str(&self.raw(slot));
        }
        parts.push_str(r#"<pivotTableStyleInfo name="PivotStyleLight16" showRowHeaders="1" showColHeaders="1" showRowStripes="0" showColStripes="0" showLastColumn="1"/>"#);
        for slot in &SLOTS[4..] {
            parts.push_str(&self.raw(slot));
        }
        Ok(format!(
            r#"<pivotTableDefinition xmlns="{MAIN_NS}" name="{}" cacheId="{cache_id}"{head}{extra}>{parts}</pivotTableDefinition>"#,
            escape(self.name)
        ))
    }
}

/// The parts the case builder writes for `pivot` of `sheet` (its `definition`, `definition`),
/// with cache id `cache_id` (`pivot.py` `_defined`).
///
/// # Errors
/// A definition or source the builder would refuse.
pub(crate) fn parts(
    spec: &CaseSpec,
    sheet: &SheetSpec,
    pivot: &PivotTableSpec,
    definition: &Json,
    cache_id: u32,
) -> Result<Parts, String> {
    let name = pivot.name.as_str();
    let d = definition
        .as_object()
        .ok_or_else(|| format!("pivot {name}: a definition is an object"))?;
    let (source_sheet, range) = split_qualified(&pivot.source)
        .unwrap_or_else(|| (sheet.name.clone(), pivot.source.clone()));
    let cells = &(spec.sheets.iter())
        .find(|s| s.name == source_sheet)
        .ok_or_else(|| format!("pivot source sheet {source_sheet:?} is not in the spec"))?
        .cells;
    let (from, to) = parse_range(&range)
        .filter(|(from, to)| from.row < to.row && from.col <= to.col)
        .ok_or_else(|| format!("pivot {name}: source {range} has no data rows"))?;
    let mut rows = Vec::new();
    let mut header = Vec::new();
    for row in from.row..=to.row {
        let mut values = Vec::new();
        for col in from.col..=to.col {
            let cell = cells.get(&Addr { row, col }.to_string());
            if row == from.row {
                match cell {
                    Some(CellInput::Str(text)) if !text.is_empty() => header.push(text.clone()),
                    _ => header.push(String::new()),
                }
            } else {
                values.push(cached(cell)?);
            }
        }
        if row > from.row {
            rows.push(values);
        }
    }
    if let Some(names) = d.get("cache_names").and_then(Json::as_array) {
        header = (names.iter())
            .map(|n| n.as_str().map(str::to_string))
            .collect::<Option<_>>()
            .ok_or_else(|| format!("pivot {name}: cache_names are texts"))?;
    }
    if header.iter().any(String::is_empty) {
        return Err(format!(
            "pivot {name}: the source header has a blank or non-text cell"
        ));
    }
    let anchor = Addr::parse(&pivot.location)
        .ok_or_else(|| format!("pivot {name}: bad location {}", pivot.location))?;
    let built = DefinedPivot::new(name, d, header, &rows)?;
    let source_ref = format!("{from}:{to}");
    Ok(Parts {
        cache_definition: built.cache_definition(&source_sheet, &source_ref)?,
        cache_records: built.cache_records(),
        table: built.table_definition(cache_id, anchor)?,
    })
}

#[cfg(test)]
mod tests;
