//! Pivot table parts (`xl/pivotTables/pivotTableN.xml`) of a workbook snapshot (T-4123).
//!
//! Excel rewrites a pivot's definition on every save: `<location>`, `<rowItems>` and `<colItems>`
//! are its own layout of the pivot, and the cache's `<groupItems>` its own group labels
//! (docs/research/pivot-tables-2026-11-04/oracle-spike.md). The snapshot keeps them, as Excel
//! stored them, so an oracle sidecar can record Excel's layout beside its cells. The rendered cells
//! themselves are ordinary sheet cells, already in the snapshot.
//!
//! Like the rest of the snapshot this reads untrusted packages: every part goes through the
//! size-capped reader; pivot and cache parts are read once each; the workbook's pivots stop at the
//! importer's `MAX_PIVOTS`, and their layout lines, indices and group items share one budget of the
//! engine's `MAX_RECORD_VALUES`. A pivot whose parts are missing, malformed or past the budget is
//! skipped, never an error for the workbook.

use std::collections::{HashMap, HashSet};

use quick_xml::events::Event;
use quick_xml::Reader;
use rustsheet_engine::pivot::{MAX_FIELDS, MAX_RECORD_VALUES};
use rustsheet_xlsx::import::MAX_PIVOTS;
use serde::{Deserialize, Serialize};

use super::{attr, read_part, relationships, resolve_target, SnapshotError, Zip};
use crate::value::Value;

/// One `<i>` of `<rowItems>` or `<colItems>`: Excel's encoding of one layout line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PivotLine {
    /// `t`: the line's kind (`default`, `grand`, `sum`, `blank`, ...); absent for a data line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
    /// `r`: how many leading fields repeat the line above (0 when absent).
    #[serde(default)]
    pub r: u32,
    /// `i`: the data field the line belongs to (0 when absent).
    #[serde(default)]
    pub i: u32,
    /// The `<x v>` item indices, in order (`v` 0 when absent).
    pub x: Vec<u32>,
}

/// A cache field of the pivot's cache, with Excel's group labels when it is grouped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PivotCacheField {
    /// The cache field's `name` (the source column's header).
    pub name: String,
    /// `<fieldGroup><groupItems>`, as Excel wrote them; empty for an ungrouped field. A `<d>`
    /// (date) item is kept as its ISO text, not converted to a serial: a scorer comparing date
    /// groups (T-4124) converts it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub group_items: Vec<Value>,
}

/// A pivot table as Excel saved it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PivotDef {
    /// The pivot's `name`.
    pub name: String,
    /// Name of the sheet that holds it.
    pub sheet: String,
    /// `<location ref>`, e.g. `A3:E12`.
    pub location: String,
    /// `firstHeaderRow`, `firstDataRow` and `firstDataCol`: offsets inside `location`.
    pub first_header_row: u32,
    /// See `first_header_row`.
    pub first_data_row: u32,
    /// See `first_header_row`.
    pub first_data_col: u32,
    /// `rowPageCount` and `colPageCount`: the page-field area above `location`.
    #[serde(default)]
    pub row_page_count: u32,
    #[serde(default)]
    pub col_page_count: u32,
    /// `<rowItems>`, one line per rendered row of the body.
    pub row_items: Vec<PivotLine>,
    /// `<colItems>`, one line per rendered data column.
    pub col_items: Vec<PivotLine>,
    /// The cache's fields, in cache order; empty when the cache part is missing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cache_fields: Vec<PivotCacheField>,
}

/// What reading a workbook's pivots has used so far: one budget for the whole workbook, so a
/// crafted file cannot multiply the work by repeating parts or pivots (directive 5).
pub(super) struct PivotReader {
    /// Pivot parts already read, by the package's own entry name (`canonical`).
    seen: HashSet<String>,
    /// Cache parts already parsed, by entry name (`None`: missing, malformed or over budget).
    caches: HashMap<String, Option<Vec<PivotCacheField>>>,
    /// Pivot relationships looked at, read or not: at most `MAX_PIVOTS`, so malformed parts count.
    attempts: usize,
    /// What the snapshot may still hold over every pivot and every copy of a shared cache: one
    /// unit per layout line, item index, field and group item, plus one per byte of every string
    /// kept (names, locations, labels).
    values_left: usize,
    /// The pivots read, at most `MAX_PIVOTS`, the importer's own workbook cap.
    pub(super) pivots: Vec<PivotDef>,
}

impl PivotReader {
    pub(super) fn new() -> Self {
        Self {
            seen: HashSet::new(),
            caches: HashMap::new(),
            attempts: 0,
            values_left: MAX_RECORD_VALUES,
            pivots: Vec::new(),
        }
    }
}

/// The budget cost of keeping a copy of `fields`: one per field and item, plus their text bytes.
fn fields_cost(fields: &[PivotCacheField]) -> usize {
    fields
        .iter()
        .map(|f| 1 + f.name.len() + f.group_items.iter().map(value_cost).sum::<usize>())
        .sum()
}

fn value_cost(v: &Value) -> usize {
    1 + match v {
        Value::Text(t) | Value::Error(t) => t.len(),
        _ => 0,
    }
}

/// The workbook budget ran out.
struct OverBudget;

/// `attr(e, key)`, charged its byte length against `budget`.
fn kept_attr(
    e: &quick_xml::events::BytesStart<'_>,
    key: &str,
    budget: &mut usize,
) -> Result<Option<String>, OverBudget> {
    let value = attr(e, key);
    spend(budget, value.as_ref().map_or(0, String::len)).ok_or(OverBudget)?;
    Ok(value)
}

/// The package entry a resolved part path names. The package matches names ignoring case, `\\`
/// against `/` and a leading `/` (`PartNames`), so spellings of one part share one key here.
fn canonical(zip: &Zip, path: &str) -> Option<String> {
    zip.names.entry(path).map(str::to_owned)
}

/// Takes `n` from `budget`, or `None` (leaving it at 0) when it has fewer.
fn spend(budget: &mut usize, n: usize) -> Option<()> {
    if let Some(left) = budget.checked_sub(n) {
        *budget = left;
        Some(())
    } else {
        *budget = 0;
        None
    }
}

/// Reads the pivots of the sheet part at `sheet_path`, found through its relationships, into
/// `reader`. A part already read, a pivot whose parts cannot be read or parsed, and anything past
/// the workbook budget are skipped. Every pivot relationship counts toward `MAX_PIVOTS`.
pub(super) fn read_sheet_pivots(
    zip: &mut Zip,
    sheet_path: &str,
    sheet: &str,
    reader: &mut PivotReader,
) -> Result<(), SnapshotError> {
    let (dir, file) = sheet_path.rsplit_once('/').unwrap_or(("", sheet_path));
    let Some(rels) = read_part(zip, &format!("{dir}/_rels/{file}.rels"))? else {
        return Ok(());
    };
    let Ok(rels) = relationships(&rels, file) else {
        return Ok(());
    };
    for rel in rels.iter().filter(|r| r.kind.ends_with("/pivotTable")) {
        if reader.attempts >= MAX_PIVOTS {
            break;
        }
        reader.attempts += 1;
        let Some(path) = canonical(zip, &resolve_target(dir, &rel.target)) else {
            continue;
        };
        if !reader.seen.insert(path.clone()) {
            continue;
        }
        let Ok(Some(xml)) = read_part(zip, &path) else {
            continue;
        };
        let Some(mut pivot) = parse_pivot(&xml, sheet, &mut reader.values_left) else {
            continue;
        };
        let fields = cache_fields(zip, &path, reader);
        if spend(&mut reader.values_left, fields_cost(&fields)).is_none() {
            continue;
        }
        pivot.cache_fields = fields;
        reader.pivots.push(pivot);
    }
    Ok(())
}

/// The fields of the cache the pivot part at `pivot_path` uses, parsed once per cache part.
fn cache_fields(zip: &mut Zip, pivot_path: &str, reader: &mut PivotReader) -> Vec<PivotCacheField> {
    let Some(path) = cache_path(zip, pivot_path) else {
        return Vec::new();
    };
    if !reader.caches.contains_key(&path) {
        let parsed = read_part(zip, &path)
            .ok()
            .flatten()
            .and_then(|xml| parse_cache_fields(&xml, &mut reader.values_left));
        reader.caches.insert(path.clone(), parsed);
    }
    reader
        .caches
        .get(&path)
        .cloned()
        .flatten()
        .unwrap_or_default()
}

/// The resolved path of the cache definition the pivot part at `pivot_path` relates to.
fn cache_path(zip: &mut Zip, pivot_path: &str) -> Option<String> {
    let (dir, file) = pivot_path.rsplit_once('/').unwrap_or(("", pivot_path));
    let rels = read_part(zip, &format!("{dir}/_rels/{file}.rels")).ok()??;
    let rel = relationships(&rels, file)
        .ok()?
        .into_iter()
        .find(|r| r.kind.ends_with("/pivotCacheDefinition"))?;
    canonical(zip, &resolve_target(dir, &rel.target))
}

/// An attribute as a `u32`, 0 when absent or not a number.
fn number(e: &quick_xml::events::BytesStart<'_>, key: &str) -> u32 {
    attr(e, key).and_then(|v| v.parse().ok()).unwrap_or(0)
}

/// Where the reader is inside a pivot definition.
#[derive(Clone, Copy, PartialEq)]
enum In {
    Other,
    Rows,
    Cols,
}

/// The `<rowItems>` or `<colItems>` list `within` names.
fn lines_mut(pivot: &mut PivotDef, within: In) -> &mut Vec<PivotLine> {
    if within == In::Rows {
        &mut pivot.row_items
    } else {
        &mut pivot.col_items
    }
}

/// The pivot definition, or `None` when the part is malformed, has no `<location>`, or lists more
/// lines or indices than the engine's limits allow.
fn parse_pivot(xml: &str, sheet: &str, budget: &mut usize) -> Option<PivotDef> {
    let mut reader = Reader::from_str(xml);
    let mut pivot: Option<PivotDef> = None;
    let mut within = In::Other;
    loop {
        let event = reader.read_event().ok()?;
        match &event {
            Event::Start(e) | Event::Empty(e) => {
                let empty = matches!(event, Event::Empty(_));
                match e.local_name().as_ref() {
                    "pivotTableDefinition" => {
                        pivot = Some(PivotDef {
                            name: kept_attr(e, "name", budget).ok()?.unwrap_or_default(),
                            sheet: sheet.to_string(),
                            location: String::new(),
                            first_header_row: 0,
                            first_data_row: 0,
                            first_data_col: 0,
                            row_page_count: 0,
                            col_page_count: 0,
                            row_items: Vec::new(),
                            col_items: Vec::new(),
                            cache_fields: Vec::new(),
                        });
                    }
                    "location" => {
                        let p = pivot.as_mut()?;
                        p.location = kept_attr(e, "ref", budget).ok()??;
                        p.first_header_row = number(e, "firstHeaderRow");
                        p.first_data_row = number(e, "firstDataRow");
                        p.first_data_col = number(e, "firstDataCol");
                        p.row_page_count = number(e, "rowPageCount");
                        p.col_page_count = number(e, "colPageCount");
                    }
                    "rowItems" if !empty => within = In::Rows,
                    "colItems" if !empty => within = In::Cols,
                    "i" if within != In::Other => {
                        let lines = lines_mut(pivot.as_mut()?, within);
                        spend(budget, 1)?;
                        lines.push(PivotLine {
                            t: kept_attr(e, "t", budget).ok()?,
                            r: number(e, "r"),
                            i: number(e, "i"),
                            x: Vec::new(),
                        });
                    }
                    "x" if within != In::Other => {
                        let lines = lines_mut(pivot.as_mut()?, within);
                        spend(budget, 1)?;
                        lines.last_mut()?.x.push(number(e, "v"));
                    }
                    _ => {}
                }
            }
            Event::End(e) if matches!(e.local_name().as_ref(), "rowItems" | "colItems") => {
                within = In::Other;
            }
            Event::Eof => break,
            _ => {}
        }
    }
    pivot.filter(|p| !p.location.is_empty())
}

fn parse_cache_fields(xml: &str, budget: &mut usize) -> Option<Vec<PivotCacheField>> {
    let mut reader = Reader::from_str(xml);
    let mut fields: Vec<PivotCacheField> = Vec::new();
    let mut in_group = false;
    loop {
        let event = reader.read_event().ok()?;
        match &event {
            Event::Start(e) | Event::Empty(e) => {
                let empty = matches!(event, Event::Empty(_));
                match e.local_name().as_ref() {
                    "cacheField" => {
                        if fields.len() >= MAX_FIELDS {
                            return None;
                        }
                        spend(budget, 1)?;
                        fields.push(PivotCacheField {
                            name: kept_attr(e, "name", budget).ok()?.unwrap_or_default(),
                            group_items: Vec::new(),
                        });
                    }
                    "groupItems" if !empty => in_group = true,
                    tag if in_group => {
                        spend(budget, 1)?;
                        let v = kept_attr(e, "v", budget).ok()?;
                        let value = match tag {
                            "s" | "d" => Value::Text(v.unwrap_or_default()),
                            "n" => Value::Number(
                                v.and_then(|s| s.parse::<f64>().ok())
                                    .filter(|f| f.is_finite())?,
                            ),
                            "b" => Value::Bool(matches!(v.as_deref(), Some("1" | "true"))),
                            "e" => Value::Error(v.unwrap_or_default()),
                            "m" => Value::Empty,
                            _ => continue,
                        };
                        fields.last_mut()?.group_items.push(value);
                    }
                    _ => {}
                }
            }
            Event::End(e) if e.local_name().as_ref() == "groupItems" => in_group = false,
            Event::Eof => break,
            _ => {}
        }
    }
    Some(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_excel_layout_lines_and_location() {
        // Excel's rewrite of a tabular two-row-field pivot (oracle-spike.md run 4), shortened.
        let xml = r#"<pivotTableDefinition xmlns="m" name="PivotSum" cacheId="1">
            <location ref="A3:E12" firstHeaderRow="1" firstDataRow="2" firstDataCol="2"/>
            <rowItems count="4"><i><x/><x/></i><i r="1"><x v="1"/></i>
              <i t="default"><x/></i><i t="grand"><x/></i></rowItems>
            <colItems count="2"><i><x/></i><i i="1"><x v="1"/></i></colItems>
            </pivotTableDefinition>"#;
        let p = parse_pivot(xml, "P", &mut usize::MAX.clone()).expect("parses");
        assert_eq!(p.location, "A3:E12");
        assert_eq!((p.first_data_row, p.first_data_col), (2, 2));
        assert_eq!(p.row_items.len(), 4);
        assert_eq!(p.row_items[0].x, vec![0, 0]);
        assert_eq!(p.row_items[1].r, 1);
        assert_eq!(p.row_items[1].x, vec![1]);
        assert_eq!(p.row_items[3].t.as_deref(), Some("grand"));
        assert_eq!(p.col_items[1].i, 1);
    }

    #[test]
    fn malformed_or_location_less_pivots_are_skipped() {
        let mut budget = usize::MAX;
        assert!(parse_pivot(
            "<pivotTableDefinition name='x'><rowItems>",
            "P",
            &mut budget
        )
        .is_none());
        assert!(parse_pivot("<pivotTableDefinition name='x'/>", "P", &mut budget).is_none());
        assert!(parse_pivot("<<<", "P", &mut budget).is_none());
    }

    #[test]
    fn reads_group_items_of_grouped_cache_fields() {
        let xml = r#"<pivotCacheDefinition><cacheFields count="2">
            <cacheField name="Region"><sharedItems/></cacheField>
            <cacheField name="Day"><sharedItems/><fieldGroup base="1"><rangePr/>
              <groupItems count="3"><s v="&lt;45000"/><s v="45000-45100"/><n v="3"/></groupItems>
            </fieldGroup></cacheField>
            <cacheField name="Bad"><fieldGroup base="2"><rangePr/>
              <groupItems count="1"><n v="inf"/></groupItems>
            </fieldGroup></cacheField></cacheFields></pivotCacheDefinition>"#;
        // A non-finite number item makes the cache unreadable (the snapshot is JSON).
        assert!(parse_cache_fields(xml, &mut usize::MAX.clone()).is_none());
        let finite = xml.replace(r#"<n v="inf"/>"#, r#"<n v="1"/>"#);
        let fields = parse_cache_fields(&finite, &mut usize::MAX.clone()).expect("parses");
        assert_eq!(fields.len(), 3);
        assert!(fields[0].group_items.is_empty());
        assert_eq!(
            fields[1].group_items,
            vec![
                Value::Text("<45000".into()),
                Value::Text("45000-45100".into()),
                Value::Number(3.0)
            ]
        );
    }

    #[test]
    fn the_workbook_budget_stops_lines_indices_and_group_items() {
        let xml = r#"<pivotTableDefinition name="p"><location ref="A1:B3"/>
            <rowItems><i><x/></i><i><x v="1"/></i></rowItems></pivotTableDefinition>"#;
        // The name `p` and location `A1:B3` cost their 6 bytes, two lines and two indices 4 more.
        let mut budget = 10;
        assert!(parse_pivot(xml, "P", &mut budget).is_some());
        assert_eq!(budget, 0);
        let mut short = 9;
        assert!(parse_pivot(xml, "P", &mut short).is_none());
        assert_eq!(short, 0);
        let cache = r#"<pivotCacheDefinition><cacheField name="n"><fieldGroup><groupItems>
            <s v="a"/><s v="b"/></groupItems></fieldGroup></cacheField></pivotCacheDefinition>"#;
        // One field (1) named `n` (1), two items (2) of one byte each (2).
        assert!(parse_cache_fields(cache, &mut 5).is_none());
        assert!(parse_cache_fields(cache, &mut 6).is_some());
    }

    #[test]
    fn many_pivots_sharing_one_cache_read_it_once_and_stop_at_max_pivots() {
        use std::fmt::Write as _;
        use std::io::{Cursor, Write as _};
        let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let opts = zip::write::SimpleFileOptions::default();
        let mut add = |name: &str, body: &str| {
            w.start_file(name, opts).expect("start");
            w.write_all(body.as_bytes()).expect("write");
        };
        let n = MAX_PIVOTS + 5;
        let mut rels = String::from("<Relationships>");
        for k in 0..n {
            write!(
                rels,
                r#"<Relationship Id="r{k}" Type="x/pivotTable" Target="../pivotTables/p{k}.xml"/>"#
            )
            .expect("formats");
        }
        // One part listed twice too, and spelled differently (the package ignores case and `\`).
        rels.push_str(
            r#"<Relationship Id="dup" Type="x/pivotTable" Target="../pivotTables/p0.xml"/>"#,
        );
        rels.push_str("</Relationships>");
        add("xl/worksheets/_rels/sheet1.xml.rels", &rels);
        for k in 0..n {
            add(
                &format!("xl/pivotTables/p{k}.xml"),
                &format!(
                    r#"<pivotTableDefinition name="p{k}"><location ref="A1:B2"/></pivotTableDefinition>"#
                ),
            );
            add(
                &format!("xl/pivotTables/_rels/p{k}.xml.rels"),
                r#"<Relationships><Relationship Id="c" Type="x/pivotCacheDefinition" Target="../pivotCache/c.xml"/></Relationships>"#,
            );
        }
        add(
            "xl/pivotCache/c.xml",
            r#"<pivotCacheDefinition><cacheField name="Day"><fieldGroup><groupItems><s v="g"/></groupItems></fieldGroup></cacheField></pivotCacheDefinition>"#,
        );
        let bytes = w.finish().expect("finish").into_inner();
        let mut zip = Zip::new(&bytes).expect("zip");
        let mut reader = PivotReader::new();
        read_sheet_pivots(&mut zip, "xl/worksheets/sheet1.xml", "S", &mut reader).expect("reads");
        assert_eq!(reader.pivots.len(), MAX_PIVOTS);
        assert_eq!(reader.caches.len(), 1);
        assert_eq!(
            reader.pivots[7].cache_fields[0].group_items,
            vec![Value::Text("g".into())]
        );
    }

    #[test]
    fn aliases_of_a_malformed_part_are_read_once_and_count_toward_max_pivots() {
        use std::fmt::Write as _;
        use std::io::{Cursor, Write as _};
        let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let opts = zip::write::SimpleFileOptions::default();
        let mut rels = String::from("<Relationships>");
        for k in 0..MAX_PIVOTS * 3 {
            let target = match k % 3 {
                0 => "../pivotTables/bad.xml",
                1 => "../pivotTables/BAD.xml",
                _ => "..\\pivotTables\\bad.XML",
            };
            write!(
                rels,
                r#"<Relationship Id="r{k}" Type="x/pivotTable" Target="{target}"/>"#
            )
            .expect("formats");
        }
        rels.push_str("</Relationships>");
        w.start_file("xl/worksheets/_rels/sheet1.xml.rels", opts)
            .expect("start");
        w.write_all(rels.as_bytes()).expect("write");
        w.start_file("xl/pivotTables/bad.xml", opts).expect("start");
        w.write_all(b"<pivotTableDefinition name='x'/>")
            .expect("write");
        let bytes = w.finish().expect("finish").into_inner();
        let mut zip = Zip::new(&bytes).expect("zip");
        let mut reader = PivotReader::new();
        read_sheet_pivots(&mut zip, "xl/worksheets/sheet1.xml", "S", &mut reader).expect("reads");
        assert!(reader.pivots.is_empty());
        assert_eq!(reader.seen.len(), 1);
        assert_eq!(reader.attempts, MAX_PIVOTS);
    }

    #[test]
    fn copies_of_a_long_named_shared_cache_are_charged_their_bytes() {
        let long = "n".repeat(100_000);
        let fields = vec![PivotCacheField {
            name: long,
            group_items: vec![Value::Text("abc".into())],
        }];
        assert_eq!(fields_cost(&fields), 1 + 100_000 + 1 + 3);
        // A budget the size of the engine's runs out after about a hundred copies, not thousands.
        let mut budget = MAX_RECORD_VALUES;
        let copies = (0..MAX_PIVOTS)
            .take_while(|_| spend(&mut budget, fields_cost(&fields)).is_some())
            .count();
        assert_eq!(copies, MAX_RECORD_VALUES / fields_cost(&fields));
        assert!(copies < 100);
    }
}
