//! The slicer and timeline caches that drive a pivot, and the slicer and timeline parts that show
//! them, as Excel saved them (T-4956), so a claim about how Excel saves a slicer's or timeline's
//! state can cite a committed sidecar.
//!
//! A cache part (`xl/slicerCaches/…`, `xl/timelineCaches/…`) is related from the workbook and
//! names the pivots it drives in `pivotTables/pivotTable` (`tabId`, the sheet's `sheetId`, and the
//! pivot's `name`); a slicers or timelines part is related from a sheet and holds one `<slicer>`
//! or `<timeline>` per shape, naming its cache by `cache` (the cache's `name`). Each cache is kept
//! on every pivot it names, with the shapes that name it. A table slicer's cache names no pivot,
//! so no pivot keeps it.
//!
//! Untrusted input (directive 5): each part goes through the size-capped reader once; at most
//! `MAX_PIVOTS` cache relationships and as many shape-part relationships are looked at, and every
//! kept element, copies included, is charged against the workbook budget of [`PivotReader`].

use rustsheet_xlsx::import::MAX_PIVOTS;
use serde::{Deserialize, Serialize};

use super::xml_tree::{parse_root, XmlElement};
use super::{canonical, spend, PivotReader};
use crate::xlsx_snapshot::{read_part, relationships, resolve_target, SnapshotError, Zip};

/// A slicer or timeline cache that drives a pivot, with the shapes that show it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PivotSlicer {
    /// The cache part's root (`slicerCacheDefinition` or `timelineCacheDefinition`) and
    /// everything below it, as written.
    pub cache: XmlElement,
    /// The `<slicer>` or `<timeline>` elements, from any sheet, whose `cache` names this cache.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shapes: Vec<SlicerShape>,
}

/// One `<slicer>` or `<timeline>` of a sheet's slicers or timelines part.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlicerShape {
    /// The sheet whose part holds it.
    pub sheet: String,
    /// The element as written.
    pub element: XmlElement,
}

/// Whether a relationship type names a sheet's slicers or timelines part.
fn is_shape_part(kind: &str) -> bool {
    kind.ends_with("/slicer") || kind.ends_with("/timeline")
}

/// Whether a relationship type names a workbook's slicer or timeline cache part.
fn is_cache_part(kind: &str) -> bool {
    kind.ends_with("/slicerCache") || kind.ends_with("/timelineCache")
}

/// Reads the slicers and timelines parts the sheet part in `dir` relates to (`rels`) into
/// `reader.shapes`. A part already read, unreadable or past the budget is skipped.
pub(super) fn read_sheet_shapes(
    zip: &mut Zip,
    dir: &str,
    rels: &[crate::xlsx_snapshot::Relationship],
    sheet: &str,
    reader: &mut PivotReader,
) {
    for rel in rels.iter().filter(|r| is_shape_part(&r.kind)) {
        if reader.shape_attempts >= MAX_PIVOTS {
            break;
        }
        reader.shape_attempts += 1;
        let Some(path) = canonical(zip, &resolve_target(dir, &rel.target)) else {
            continue;
        };
        if !reader.seen.insert(path.clone()) {
            continue;
        }
        let Ok(Some(xml)) = read_part(zip, &path) else {
            continue;
        };
        let Some(root) = parse_root(&xml, &mut reader.values_left) else {
            continue;
        };
        let charge = sheet.len() * root.children.len();
        if spend(&mut reader.values_left, charge).is_none() {
            continue;
        }
        reader
            .shapes
            .extend(root.children.into_iter().map(|element| SlicerShape {
                sheet: sheet.to_string(),
                element,
            }));
    }
}

/// Reads the workbook's slicer and timeline caches and keeps each on the pivots it names, with
/// its shapes. `sheets` is every sheet's name and `sheetId`, in workbook order.
pub(crate) fn read_slicer_caches(
    zip: &mut Zip,
    sheets: &[(String, Option<String>)],
    reader: &mut PivotReader,
) -> Result<(), SnapshotError> {
    let Some(rels) = read_part(zip, "xl/_rels/workbook.xml.rels")? else {
        return Ok(());
    };
    let Ok(rels) = relationships(&rels, "workbook.xml.rels") else {
        return Ok(());
    };
    for rel in rels
        .iter()
        .filter(|r| is_cache_part(&r.kind))
        .take(MAX_PIVOTS)
    {
        let Some(path) = canonical(zip, &resolve_target("xl", &rel.target)) else {
            continue;
        };
        if !reader.seen.insert(path.clone()) {
            continue;
        }
        let Ok(Some(xml)) = read_part(zip, &path) else {
            continue;
        };
        let Some(cache) = parse_root(&xml, &mut reader.values_left) else {
            continue;
        };
        attach(cache, sheets, reader);
    }
    Ok(())
}

/// Keeps `cache` on each pivot its `pivotTables` names, with the shapes naming it; each copy past
/// the first is charged its cost, and a copy the budget cannot pay for is not kept.
fn attach(cache: XmlElement, sheets: &[(String, Option<String>)], reader: &mut PivotReader) {
    let timeline = cache.local() == "timelineCacheDefinition";
    if !timeline && cache.local() != "slicerCacheDefinition" {
        return;
    }
    let name = cache.attr("name").unwrap_or_default();
    let shapes: Vec<SlicerShape> = reader
        .shapes
        .iter()
        .filter(|s| s.element.attr("cache") == Some(name))
        .cloned()
        .collect();
    let targets: Vec<(String, String)> = cache
        .children_named("pivotTables")
        .flat_map(|p| p.children_named("pivotTable"))
        .filter_map(|p| {
            let tab = p.attr("tabId")?;
            let (sheet, _) = sheets.iter().find(|(_, id)| id.as_deref() == Some(tab))?;
            Some((sheet.clone(), p.attr("name")?.to_string()))
        })
        .collect();
    let cost = cache.cost()
        + shapes
            .iter()
            .map(|s| s.sheet.len() + s.element.cost())
            .sum::<usize>();
    let record = PivotSlicer { cache, shapes };
    let mut first = true;
    for pivot in reader
        .pivots
        .iter_mut()
        .filter(|p| targets.iter().any(|(s, n)| *s == p.sheet && *n == p.name))
    {
        // The parse paid for the cache once; the shapes and every further copy are paid here.
        let charge = if first {
            cost - record.cache.cost()
        } else {
            cost
        };
        if spend(&mut reader.values_left, charge).is_none() {
            return;
        }
        first = false;
        let list = if timeline {
            &mut pivot.timelines
        } else {
            &mut pivot.slicers
        };
        list.push(record.clone());
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write as _};

    use super::super::{read_sheet_pivots, PivotReader};
    use super::*;

    fn package(parts: &[(&str, &str)]) -> Zip {
        let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let opts = zip::write::SimpleFileOptions::default();
        for (name, body) in parts {
            w.start_file(*name, opts).expect("start");
            w.write_all(body.as_bytes()).expect("write");
        }
        Zip::new(&w.finish().expect("finish").into_inner()).expect("zip")
    }

    #[test]
    fn keeps_each_cache_on_the_pivots_it_names_with_its_shapes() {
        let mut zip = package(&[
            (
                "xl/_rels/workbook.xml.rels",
                r#"<Relationships>
                <Relationship Id="a" Type="x/2007/relationships/slicerCache" Target="slicerCaches/slicerCache1.xml"/>
                <Relationship Id="b" Type="x/2011/relationships/timelineCache" Target="timelineCaches/timelineCache1.xml"/>
                <Relationship Id="c" Type="x/2007/relationships/slicerCache" Target="slicerCaches/slicerCache2.xml"/>
                </Relationships>"#,
            ),
            (
                "xl/slicerCaches/slicerCache1.xml",
                r#"<slicerCacheDefinition xmlns="x14" xmlns:xr10="r" name="Slicer_Region" xr10:uid="{1}" sourceName="Region">
                <pivotTables><pivotTable tabId="7" name="P"/></pivotTables>
                <data><tabular pivotCacheId="1"><items count="2"><i x="0" s="1"/><i x="1"/></items></tabular></data>
                </slicerCacheDefinition>"#,
            ),
            (
                "xl/timelineCaches/timelineCache1.xml",
                r#"<timelineCacheDefinition name="NativeTimeline_Date" sourceName="Date">
                <pivotTables><pivotTable tabId="7" name="P"/></pivotTables></timelineCacheDefinition>"#,
            ),
            // A table slicer's cache names no pivot.
            (
                "xl/slicerCaches/slicerCache2.xml",
                r#"<slicerCacheDefinition name="Slicer_T" sourceName="Region"/>"#,
            ),
            (
                "xl/worksheets/_rels/sheet1.xml.rels",
                r#"<Relationships>
                <Relationship Id="p" Type="x/pivotTable" Target="../pivotTables/p.xml"/>
                <Relationship Id="s" Type="x/2007/relationships/slicer" Target="../slicers/slicer1.xml"/>
                <Relationship Id="t" Type="x/2011/relationships/timeline" Target="../timelines/timeline1.xml"/>
                </Relationships>"#,
            ),
            (
                "xl/pivotTables/p.xml",
                r#"<pivotTableDefinition name="P"><location ref="A1:B2"/></pivotTableDefinition>"#,
            ),
            (
                "xl/slicers/slicer1.xml",
                r#"<slicers><slicer name="Region" cache="Slicer_Region" caption="Region"/><slicer name="T" cache="Slicer_T"/></slicers>"#,
            ),
            (
                "xl/timelines/timeline1.xml",
                r#"<timelines><timeline name="Date" cache="NativeTimeline_Date" scrollPosition="2019-01-01T00:00:00"/></timelines>"#,
            ),
        ]);
        let mut reader = PivotReader::new();
        read_sheet_pivots(&mut zip, "xl/worksheets/sheet1.xml", "S", &mut reader).expect("reads");
        let sheets = [("S".to_string(), Some("7".to_string()))];
        read_slicer_caches(&mut zip, &sheets, &mut reader).expect("reads");
        let p = &reader.pivots[0];
        assert_eq!(p.slicers.len(), 1);
        let slicer = &p.slicers[0];
        assert_eq!(slicer.cache.attr("xr10:uid"), Some("{1}"));
        let items: Vec<_> = slicer.cache.children_named("data").collect();
        assert_eq!(
            items[0].children[0].children[0].children[0].attr("s"),
            Some("1")
        );
        assert_eq!(slicer.shapes.len(), 1);
        assert_eq!(slicer.shapes[0].sheet, "S");
        assert_eq!(slicer.shapes[0].element.attr("caption"), Some("Region"));
        assert_eq!(p.timelines.len(), 1);
        assert_eq!(
            p.timelines[0].shapes[0].element.attr("scrollPosition"),
            Some("2019-01-01T00:00:00")
        );
    }

    #[test]
    fn copies_past_the_budget_are_not_kept() {
        let mut reader = PivotReader::new();
        for name in ["P", "Q"] {
            reader.pivots.push(super::super::PivotDef {
                name: name.into(),
                sheet: "S".into(),
                ..Default::default()
            });
        }
        let cache = parse_root(
            r#"<slicerCacheDefinition name="c"><pivotTables><pivotTable tabId="1" name="P"/><pivotTable tabId="1" name="Q"/></pivotTables></slicerCacheDefinition>"#,
            &mut usize::MAX.clone(),
        )
        .expect("parses");
        // Enough for nothing past the first copy, which the parse already paid for.
        reader.values_left = cache.cost() - 1;
        let sheets = [("S".to_string(), Some("1".to_string()))];
        attach(cache, &sheets, &mut reader);
        assert_eq!(reader.pivots[0].slicers.len(), 1);
        assert!(reader.pivots[1].slicers.is_empty());
        assert_eq!(reader.values_left, 0);
    }
}
