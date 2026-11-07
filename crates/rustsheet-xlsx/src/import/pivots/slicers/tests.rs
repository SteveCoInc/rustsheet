#![allow(clippy::unwrap_used)]

use super::super::tests::{caches, package, parts, with, workbook_rels, REL};
use super::super::{load_pivots, PivotSheet};
use super::*;

const X14: &str = "http://schemas.microsoft.com/office/spreadsheetml/2009/9/main";
const X15: &str = "http://schemas.microsoft.com/office/spreadsheetml/2010/11/main";

fn read(xml: &str) -> Result<SlicerCache, String> {
    read_slicer_cache(
        xml.as_bytes(),
        &ImportLimits::default(),
        DateSystem::Excel1900,
    )
    .map_err(|Skip(reason)| reason)
}

/// The slicer cache probes/t4767-pivot-slicers SM writes: East and West selected, North `nd`.
#[test]
fn a_tabular_slicer_reads_its_pivots_and_items() {
    let cache = read(&format!(
        r#"<slicerCacheDefinition xmlns="{X14}" name="Slicer_Region" sourceName="Region"><pivotTables><pivotTable tabId="3" name="P_SM"/></pivotTables><data><tabular pivotCacheId="3" sortOrder="descending"><items count="3"><i x="0" s="1"/><i x="1" s="1"/><i x="2" nd="1"/></items></tabular></data></slicerCacheDefinition>"#
    ))
    .unwrap();
    assert_eq!(
        cache,
        SlicerCache {
            source: "Region".into(),
            pivots: vec![(3, "P_SM".into())],
            selection: Selection::Items {
                unselected: vec![0b100],
                any_selected: true,
            },
        }
    );
}

/// probes/t4767-pivot-timelines TL, and TN without a selection.
#[test]
fn a_timeline_reads_its_selection_as_serials() {
    let timeline = |selection: &str| {
        read(&format!(
            r#"<timelineCacheDefinition xmlns="{X15}" name="NativeTimeline_Date" sourceName="Date"><pivotTables><pivotTable tabId="2" name="P_TL"/></pivotTables><state minimalRefreshVersion="6" lastRefreshVersion="6" pivotCacheId="1" filterType="dateBetween">{selection}<bounds startDate="2019-01-01T00:00:00" endDate="2021-01-01T00:00:00"/></state></timelineCacheDefinition>"#
        ))
        .unwrap()
    };
    let selected =
        timeline(r#"<selection startDate="2019-01-01T00:00:00" endDate="2019-03-31T00:00:00"/>"#);
    assert_eq!(selected.source, "Date");
    assert_eq!(selected.pivots, vec![(2, "P_TL".to_string())]);
    assert_eq!(
        selected.selection,
        Selection::Dates(Some((43466.0, 43555.0)))
    );
    assert_eq!(timeline("").selection, Selection::Dates(None));
}

#[test]
fn what_is_not_modelled_is_skipped() {
    let olap = format!(
        r#"<slicerCacheDefinition xmlns="{X14}" name="S" sourceName="[Region]"><data><olap pivotCacheId="1"/></data></slicerCacheDefinition>"#
    );
    assert!(read(&olap).unwrap_err().contains("OLAP"));
    let table = format!(
        r#"<slicerCacheDefinition xmlns="{X14}" xmlns:x="m" name="S" sourceName="Region"><extLst><x:ext uri="u" xmlns:x15="{X15}"><x15:tableSlicerCache tableId="1" column="1"/></x:ext></extLst></slicerCacheDefinition>"#
    );
    assert!(read(&table).unwrap_err().contains("table slicer"));
    let filter = format!(
        r#"<timelineCacheDefinition xmlns="{X15}" name="T" sourceName="Date"><state/><timelinePivotFilter fld="2"/></timelineCacheDefinition>"#
    );
    assert!(read(&filter).unwrap_err().contains("timelinePivotFilter"));
    assert!(read(r#"<worksheet xmlns="m"/>"#).is_err());
    let nameless = format!(r#"<slicerCacheDefinition xmlns="{X14}" name="S"/>"#);
    assert!(read(&nameless).unwrap_err().contains("sourceName"));
    let bad_item = format!(
        r#"<slicerCacheDefinition xmlns="{X14}" name="S" sourceName="R"><data><tabular pivotCacheId="1"><items><i x="-1" s="1"/></items></tabular></data></slicerCacheDefinition>"#
    );
    assert!(read(&bad_item).is_err());
    let bad_date = format!(
        r#"<timelineCacheDefinition xmlns="{X15}" name="T" sourceName="Date"><state><selection startDate="soon" endDate="2019-03-31T00:00:00"/></state></timelineCacheDefinition>"#
    );
    assert!(read(&bad_date).is_err());
}

/// `parts` with a slicer cache on Region (`selected`: the `s` of East and West) and a timeline
/// cache on Product, both driving `PivotSum` on sheet 7, listed in the workbook relationships as
/// `rIdS1` and `rIdT1` (T-4188).
fn with_slicers(selected: [bool; 2], timeline_pivot: &str) -> Vec<(String, String)> {
    const MS: &str = "http://schemas.microsoft.com/office";
    let workbook_rels = format!(
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdC1" Type="{REL}/pivotCacheDefinition" Target="pivotCache/pivotCacheDefinition1.xml"/><Relationship Id="rIdS1" Type="{MS}/2007/relationships/slicerCache" Target="slicerCaches/slicerCache1.xml"/><Relationship Id="rIdT1" Type="{MS}/2011/relationships/timelineCache" Target="timelineCaches/timelineCache2.xml"/></Relationships>"#
    );
    let s = |on: bool| if on { r#" s="1""# } else { "" };
    let slicer = format!(
        r#"<slicerCacheDefinition xmlns="{X14}" name="Slicer_Region" sourceName="Region"><pivotTables><pivotTable tabId="7" name="PivotSum"/></pivotTables><data><tabular pivotCacheId="1"><items count="2"><i x="0"{}/><i x="1"{}/></items></tabular></data></slicerCacheDefinition>"#,
        s(selected[0]),
        s(selected[1])
    );
    let timeline = format!(
        r#"<timelineCacheDefinition xmlns="{X15}" name="NativeTimeline_Product" sourceName="Product"><pivotTables><pivotTable tabId="7" name="{timeline_pivot}"/></pivotTables><state pivotCacheId="1" filterType="dateBetween"><selection startDate="2024-01-01T00:00:00" endDate="2024-03-31T00:00:00"/><bounds startDate="2024-01-01T00:00:00" endDate="2025-01-01T00:00:00"/></state></timelineCacheDefinition>"#
    );
    let parts = with(parts(), "xl/_rels/workbook.xml.rels", workbook_rels);
    let parts = with(parts, "xl/slicerCaches/slicerCache1.xml", slicer);
    with(parts, "xl/timelineCaches/timelineCache2.xml", timeline)
}

fn load_slicers(parts: &[(String, String)], slicer_caches: &[&str]) -> Vec<PivotTable> {
    let sheets = [PivotSheet {
        sheet_id: 7,
        path: "xl/worksheets/sheet1.xml".into(),
    }];
    let rids: Vec<String> = slicer_caches.iter().map(|r| (*r).to_string()).collect();
    let (_, pivots) = load_pivots(
        &mut package(parts),
        &workbook_rels(parts),
        &caches(&[("1", "rIdC1")]),
        &rids,
        &sheets,
        DateSystem::Excel1900,
    );
    pivots
}

/// probes/t4767-pivot-slicers SX: a slicer selecting every item shows the item the pivot hides.
#[test]
fn a_slicer_cache_sets_its_fields_selection_over_the_hidden_items() {
    let pivots = load_slicers(&with_slicers([true, true], "PivotSum"), &["rIdS1", "rIdT1"]);
    let region = &pivots[0].fields[0];
    assert!(region.is_hidden(1), "item@h stays what the export writes");
    assert_eq!(region.slicer, Some(vec![0]));
    assert!(!region.excludes(1));
    // Only West selected: East is left out.
    let pivots = load_slicers(&with_slicers([false, true], "PivotSum"), &["rIdS1"]);
    let region = &pivots[0].fields[0];
    assert!(region.excludes(0) && !region.excludes(1));
    assert_eq!(pivots[0].fields[1].timeline, None, "rIdT1 is not listed");
}

#[test]
fn a_timeline_cache_sets_its_fields_span() {
    let pivots = load_slicers(&with_slicers([true, true], "PivotSum"), &["rIdT1"]);
    assert_eq!(
        pivots[0].fields[1].timeline,
        Some(rustsheet_engine::pivot::DateSpan {
            first: 45292,
            last: 45382
        })
    );
    assert_eq!(pivots[0].fields[0].slicer, None);
}

/// A cache naming no loaded pivot, an unknown relationship or a part that is not a slicer
/// cache changes nothing, and the pivot still loads.
#[test]
fn a_slicer_cache_that_drives_nothing_changes_nothing() {
    let pivots = load_slicers(
        &with_slicers([true, false], "Other"),
        &["rIdT1", "rIdX", "rIdC1"],
    );
    assert_eq!(pivots.len(), 1);
    assert_eq!(pivots[0].fields[1].timeline, None);
    assert_eq!(pivots[0].fields[0].slicer, None);
}

/// A part that lists one pivot a thousand times beside many items is read as one pivot, and an
/// item index past every cache's items allocates nothing (directive 5).
#[test]
fn a_repeated_pivot_entry_is_kept_once() {
    let entries = r#"<pivotTable tabId="7" name="PivotSum"/>"#.repeat(1_000);
    let mut items = String::new();
    for x in 2..10_000 {
        items.push_str(r#"<i x=""#);
        items.push_str(&x.to_string());
        items.push_str(r#""/>"#);
    }
    let cache = read(&format!(
        r#"<slicerCacheDefinition xmlns="{X14}" name="S" sourceName="Region"><pivotTables>{entries}</pivotTables><data><tabular pivotCacheId="1"><items><i x="0"/><i x="1" s="1"/>{items}<i x="4294967295"/></items></tabular></data></slicerCacheDefinition>"#
    ))
    .unwrap();
    assert_eq!(cache.pivots, vec![(7, "PivotSum".to_string())]);
    let words = match &cache.selection {
        Selection::Items { unselected, .. } => unselected.len(),
        Selection::Dates(_) => 0,
    };
    assert_eq!(words, 10_000_usize.div_ceil(64));
    // Applied to a two-item field: cut to its items, East (0) out, West (1) shown.
    let parts = with_slicers([true, true], "PivotSum");
    let (caches, mut tables) = {
        let sheets = [PivotSheet {
            sheet_id: 7,
            path: "xl/worksheets/sheet1.xml".into(),
        }];
        load_pivots(
            &mut package(&parts),
            &workbook_rels(&parts),
            &caches(&[("1", "rIdC1")]),
            &[],
            &sheets,
            DateSystem::Excel1900,
        )
    };
    let by_name = pivots_by_name(&tables);
    apply_slicer_cache(&cache, &mut tables, &by_name, &caches);
    assert_eq!(tables[0].fields[0].slicer, Some(vec![0b01]));
}
