//! Pivot table parts (`xl/pivotTables/pivotTableN.xml`) of a workbook snapshot (T-4123).
//!
//! Excel rewrites a pivot's definition on every save: `<location>`, `<rowItems>` and `<colItems>`
//! are its own layout of the pivot, and the cache's `<groupItems>` its own group labels
//! (docs/research/pivot-tables-2026-11-04/oracle-spike.md). The snapshot keeps them, as Excel
//! stored them, so an oracle sidecar can record Excel's layout beside its cells. The rendered cells
//! themselves are ordinary sheet cells, already in the snapshot.
//!
//! Like the rest of the snapshot this reads untrusted packages: every part goes through the
//! size-capped reader, the lists stop at the engine's pivot limits, and a pivot whose parts are
//! missing or malformed is skipped, never an error for the workbook.

use quick_xml::events::Event;
use quick_xml::Reader;
use rustsheet_engine::pivot::{MAX_FIELDS, MAX_ITEMS};
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
    pub first_header_row: u32,
    pub first_data_row: u32,
    pub first_data_col: u32,
    /// `rowPageCount` and `colPageCount`: the page-field area above `location`.
    #[serde(default)]
    pub row_page_count: u32,
    #[serde(default)]
    pub col_page_count: u32,
    pub row_items: Vec<PivotLine>,
    pub col_items: Vec<PivotLine>,
    /// The cache's fields, in cache order; empty when the cache part is missing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cache_fields: Vec<PivotCacheField>,
}

/// Appends the pivots of the sheet part at `sheet_path`, found through its relationships. A
/// pivot whose part cannot be read or parsed is skipped.
pub(super) fn read_sheet_pivots(
    zip: &mut Zip,
    sheet_path: &str,
    sheet: &str,
    out: &mut Vec<PivotDef>,
) -> Result<(), SnapshotError> {
    let (dir, file) = sheet_path.rsplit_once('/').unwrap_or(("", sheet_path));
    let Some(rels) = read_part(zip, &format!("{dir}/_rels/{file}.rels"))? else {
        return Ok(());
    };
    let Ok(rels) = relationships(&rels, file) else {
        return Ok(());
    };
    // A part listed twice is read once: repeated relationships must not multiply the work.
    let mut seen = std::collections::HashSet::new();
    for rel in rels.iter().filter(|r| r.kind.ends_with("/pivotTable")) {
        let path = resolve_target(dir, &rel.target);
        if !seen.insert(path.clone()) {
            continue;
        }
        let Ok(Some(xml)) = read_part(zip, &path) else {
            continue;
        };
        let Some(mut pivot) = parse_pivot(&xml, sheet) else {
            continue;
        };
        pivot.cache_fields = read_cache_fields(zip, &path).unwrap_or_default();
        out.push(pivot);
    }
    Ok(())
}

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
fn parse_pivot(xml: &str, sheet: &str) -> Option<PivotDef> {
    let mut reader = Reader::from_str(xml);
    let mut pivot: Option<PivotDef> = None;
    let mut within = In::Other;
    let mut indices = 0usize;
    loop {
        let event = reader.read_event().ok()?;
        match &event {
            Event::Start(e) | Event::Empty(e) => {
                let empty = matches!(event, Event::Empty(_));
                match e.local_name().as_ref() {
                    "pivotTableDefinition" => {
                        pivot = Some(PivotDef {
                            name: attr(e, "name").unwrap_or_default(),
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
                        p.location = attr(e, "ref")?;
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
                        if lines.len() >= MAX_ITEMS {
                            return None;
                        }
                        lines.push(PivotLine {
                            t: attr(e, "t"),
                            r: number(e, "r"),
                            i: number(e, "i"),
                            x: Vec::new(),
                        });
                    }
                    "x" if within != In::Other => {
                        let lines = lines_mut(pivot.as_mut()?, within);
                        indices += 1;
                        if indices > MAX_ITEMS * 4 {
                            return None;
                        }
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

/// The fields of the cache the pivot part at `pivot_path` uses, through its relationships; `None`
/// when the cache part is missing or malformed.
fn read_cache_fields(zip: &mut Zip, pivot_path: &str) -> Option<Vec<PivotCacheField>> {
    let (dir, file) = pivot_path.rsplit_once('/').unwrap_or(("", pivot_path));
    let rels = read_part(zip, &format!("{dir}/_rels/{file}.rels")).ok()??;
    let rel = relationships(&rels, file)
        .ok()?
        .into_iter()
        .find(|r| r.kind.ends_with("/pivotCacheDefinition"))?;
    let xml = read_part(zip, &resolve_target(dir, &rel.target)).ok()??;
    parse_cache_fields(&xml)
}

fn parse_cache_fields(xml: &str) -> Option<Vec<PivotCacheField>> {
    let mut reader = Reader::from_str(xml);
    let mut fields: Vec<PivotCacheField> = Vec::new();
    let mut in_group = false;
    let mut items = 0usize;
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
                        fields.push(PivotCacheField {
                            name: attr(e, "name").unwrap_or_default(),
                            group_items: Vec::new(),
                        });
                    }
                    "groupItems" if !empty => in_group = true,
                    tag if in_group => {
                        items += 1;
                        if items > MAX_ITEMS {
                            return None;
                        }
                        let v = attr(e, "v");
                        let value = match tag {
                            "s" | "d" => Value::Text(v.unwrap_or_default()),
                            "n" => Value::Number(
                                v.and_then(|s| s.parse::<f64>().ok())
                                    .filter(|f| f.is_finite())?,
                            ),
                            "b" => Value::Bool(v.as_deref() == Some("1")),
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
        let p = parse_pivot(xml, "P").expect("parses");
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
        assert!(parse_pivot("<pivotTableDefinition name='x'><rowItems>", "P").is_none());
        assert!(parse_pivot("<pivotTableDefinition name='x'/>", "P").is_none());
        assert!(parse_pivot("<<<", "P").is_none());
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
        assert!(parse_cache_fields(xml).is_none());
        let finite = xml.replace(r#"<n v="inf"/>"#, r#"<n v="1"/>"#);
        let fields = parse_cache_fields(&finite).expect("parses");
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
}
