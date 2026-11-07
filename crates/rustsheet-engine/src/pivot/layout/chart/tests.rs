#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! Pivot chart data against the caches Excel wrote for charts bound to pivots, and, for the probe
//! pivots Excel rendered, every plotted point against the cell Excel saved for it.
//!
//! The chart caches are transcribed from two Excel-saved corpus files, because the PR gate does
//! not fetch the corpus (crates/rustsheet-xlsx/tests/pivot_chart_corpus.rs checks the files
//! themselves when they are fetched):
//! - POI `test-data/spreadsheet/WithChartSheet.xlsx` (the `poi` suite, rev `9958a2a2`, sha256
//!   `792b9f21…`; Excel 12): `xl/charts/chart1.xml`, bound to `Sheet2!PivotTable2`
//!   (`xl/pivotTables/pivotTable5.xml`, `cacheId` 1: `pivotCacheDefinition2` and
//!   `pivotCacheRecords2`);
//! - LibreOffice `sc/qa/unit/data/xlsx/forum-mso-de-104083.xlsx` (the `pivot-corpus-lo` suite,
//!   rev `b1653ced`, sha256 `c7182682…`; Excel 16): `xl/charts/chart1.xml`, bound to
//!   `Tabelle1!PivotTable2` (`xl/pivotTables/pivotTable1.xml` over `pivotCacheDefinition1` and
//!   `pivotCacheRecords1`), its 2,303 records reduced to the sums the chart reads.
//!
//! probes/t4130-pivot-chart-cache C's pivot cells are in the committed sidecar, but its chart
//! caches are only in the oracle's cached download (UNVERIFIED, T-4187).

use serde_json::Value as Json;

use super::*;
use crate::pivot::probe_cases::{position, probe_pivot, read};
use crate::pivot::{
    compute, lay_out, Axis, AxisField, CacheBuilder, CacheItem, CellArea, DataField, PivotField,
    PivotItem, RecordValue, MAX_PIVOT_WORK, MAX_RECORD_VALUES,
};
use crate::types::NumFmt;

fn chart(table: &PivotTable, cache: &PivotCache) -> (PivotLayout, PivotChartData) {
    chart_formatted(table, cache, &[])
}

/// [`chart`] with the workbook's number formats `num_fmts`.
fn chart_formatted(
    table: &PivotTable,
    cache: &PivotCache,
    num_fmts: &[NumFmt],
) -> (PivotLayout, PivotChartData) {
    let mut work = MAX_PIVOT_WORK;
    let result = compute(table, cache, &mut work).unwrap();
    let layout = lay_out(table, cache, &result, &mut work).unwrap();
    let data = chart_data(table, cache, &result, &layout, num_fmts, &mut work).unwrap();
    (layout, data)
}

fn labels(data: &PivotChartData) -> Vec<Vec<&str>> {
    (data.categories.iter())
        .map(|c| c.labels.iter().map(String::as_str).collect())
        .collect()
}

fn numbers(series: &ChartSeries) -> Vec<Option<f64>> {
    (series.values.iter())
        .map(|v| v.clone().map(|v| v.unwrap()))
        .collect()
}

/// The pivots' anchor: both corpus pivots are at A3.
const A3: CellArea = CellArea {
    top: 2,
    left: 0,
    bottom: 2,
    right: 0,
};

/// A cache of fields `names` holding `records`, each value interned in record order.
fn cache_of(names: &[&str], records: Vec<Vec<CacheItem>>) -> PivotCache {
    let mut builder = CacheBuilder::new(None, MAX_RECORD_VALUES);
    for name in names {
        builder.add_field((*name).to_string()).unwrap();
    }
    for record in records {
        let values = record.into_iter().map(RecordValue::Inline).collect();
        builder.push_record(values).unwrap();
    }
    builder.finish()
}

/// `pivotCacheRecords2` of WithChartSheet.xlsx, resolved through `pivotCacheDefinition2`'s shared
/// items: Year, Category, Cost, Revenue.
const WITH_CHART_SHEET_RECORDS: [(f64, &str, f64, f64); 12] = [
    (2005.0, "Books", 889_017.0, 1_140_362.5),
    (2005.0, "Electronics", 13_114_909.0, 15_883_893.0),
    (2005.0, "mMovies", 768_187.0, 823_227.875),
    (2005.0, "MMusic", 583_350.0, 626_592.187_5),
    (2006.0, "Books", 1_025_796.0, 1_320_585.375),
    (2006.0, "Electronics", 15_910_763.0, 19_299_870.0),
    (2006.0, "mMovies", 961_323.0, 1_032_390.813),
    (2006.0, "MMusic", 695_300.0, 748_966.0),
    (2007.0, "Books", 1_218_726.0, 1_563_287.125),
    (2007.0, "Electronics", 19_524_378.0, 23_654_030.0),
    (2007.0, "mMovies", 1_241_525.0, 1_333_126.375),
    (2007.0, "MMusic", 875_455.0, 940_136.187_5),
];

/// `Sheet2!PivotTable2` of WithChartSheet.xlsx: tabular (`compact="0" outline="0"` on every
/// field), Year then Values on rows (`dataOnRows`), no column field, classic layout, grand totals
/// on, Sum of Cost and Sum of Revenue.
fn with_chart_sheet() -> (PivotCache, PivotTable) {
    let records = WITH_CHART_SHEET_RECORDS.map(|(year, category, cost, revenue)| {
        vec![
            CacheItem::Number(year),
            CacheItem::Text(category.to_string()),
            CacheItem::Number(cost),
            CacheItem::Number(revenue),
        ]
    });
    let cache = cache_of(&["Year", "Category", "Cost", "Revenue"], records.to_vec());
    let anchor = A3;
    let mut table = PivotTable::new("PivotTable2".to_string(), 1, anchor, 0);
    for (index, field) in (0u32..).zip(&cache.fields) {
        let axis = (index == 0).then_some(Axis::Row);
        let mut pivot_field = PivotField::new(axis, field.items.len());
        pivot_field.show_all = false;
        pivot_field.compact = false;
        pivot_field.outline = false;
        pivot_field.subtotal_top = false;
        if index == 0 {
            pivot_field.items = (0..3).map(PivotItem::data).collect();
        }
        // pivotTable5's Cost field is `numFmtId="6"`; its data fields have none.
        if index == 2 {
            pivot_field.number_format = Some(6);
        }
        table.fields.push(pivot_field);
    }
    table.row_fields = vec![AxisField::Field(0), AxisField::Values];
    table.data_fields = [("Sum of Cost", 2), ("Sum of Revenue", 3)]
        .into_iter()
        .map(|(caption, field)| DataField {
            caption: caption.to_string(),
            field,
            base_field: 0,
            base_item: 0,
            ..DataField::default()
        })
        .collect();
    let options = &mut table.options;
    options.compact = false;
    options.compact_data = false;
    options.outline = false;
    options.outline_data = false;
    options.data_on_rows = true;
    options.grid_drop_zones = true;
    options.data_caption = Some("Data".to_string());
    (cache, table)
}

#[test]
fn with_chart_sheet_plots_its_multi_level_categories_without_grand_totals() {
    let (cache, table) = with_chart_sheet();
    let (layout, data) = chart(&table, &cache);
    // The engine lays the pivot out as Excel saved it: pivotTable5's eight `rowItems`, the last
    // two the grand totals of each data field.
    let lines: Vec<_> = (layout.rows.iter())
        .map(|l| (l.kind, l.repeat, l.data_field, l.items.clone()))
        .collect();
    let data_line = |r, i, x: &[u32]| (ItemType::Data, r, i, x.to_vec());
    let expected = vec![
        data_line(0, 0, &[0, 0]),
        data_line(1, 1, &[1]),
        data_line(0, 0, &[1, 0]),
        data_line(1, 1, &[1]),
        data_line(0, 0, &[2, 0]),
        data_line(1, 1, &[1]),
        (ItemType::Grand, 0, 0, vec![0]),
        (ItemType::Grand, 0, 1, vec![0]),
    ];
    assert_eq!(lines, expected);

    // chart1.xml's `c:cat`: `c:multiLvlStrCache` ptCount 6, the data fields at every point
    // (innermost level first), the years at points 0, 2 and 4.
    let categories = [2005, 2006, 2007]
        .iter()
        .flat_map(|year| {
            let year = year.to_string();
            [
                vec![year.clone(), "Sum of Cost".to_string()],
                vec![year, "Sum of Revenue".to_string()],
            ]
        })
        .collect::<Vec<_>>();
    assert_eq!(labels(&data), categories);
    let cost_revenue = ["Sum of Cost", "Sum of Revenue"];
    let inner: Vec<(u32, &str)> = (0..6).map(|p| (p, cost_revenue[p as usize % 2])).collect();
    let outer = vec![(0, "2005"), (2, "2006"), (4, "2007")];
    assert_eq!(data.category_levels(), vec![inner, outer]);
    assert_eq!(
        data.categories.iter().map(|c| c.line).collect::<Vec<_>>(),
        [0, 1, 2, 3, 4, 5]
    );

    // One series, `c:tx` `Sheet2!$C$3` cached `Total`, and its `c:numCache` ptCount 6.
    assert_eq!(data.series.len(), 1);
    let series = &data.series[0];
    assert_eq!(series.name(), Some(TOTAL_SERIES));
    assert_eq!(series.line, 0);
    let excel = [
        "15355463",
        "18474075.5625",
        "18593182",
        "22401812.188000001",
        "22860084",
        "27490579.6875",
    ];
    let excel: Vec<Option<f64>> = excel.iter().map(|v| Some(v.parse().unwrap())).collect();
    assert_eq!(numbers(series), excel);
}

/// `pivotCacheRecords1` of forum-mso-de-104083.xlsx reduced to what its chart reads: per year,
/// quarter and month (the date's `Jahre`, `Quartale` and month groups) and KW, the sum of Menge
/// (every Menge is an integer, so a sum of sums is exact), as `year quarter month KW=Menge...`.
const FORUM_RECORDS: [&str; 31] = [
    "2015 Qrtl1 Jan 2=98665 3=124327 4=354247 5=192380",
    "2015 Qrtl1 Feb 6=133415 7=125250 8=50230 9=298",
    "2015 Qrtl1 Mrz 10=93301 11=125953 12=117604 13=243775 14=42160",
    "2015 Qrtl2 Apr 14=17326 15=100116 16=169620 17=59392",
    "2015 Qrtl2 Mai 19=6270 20=99764 21=266840 22=194160",
    "2015 Qrtl2 Jun 23=134187 24=220480 25=163211 26=123088 27=57120",
    "2015 Qrtl3 Jul 27=72646 28=59812 29=18630 30=68405 31=139280",
    "2015 Qrtl3 Aug 32=192521 33=93202 34=134397 35=110739 36=60540",
    "2015 Qrtl3 Sep 36=136016 37=57840 38=146631 39=210146 40=92206",
    "2015 Qrtl4 Okt 40=85720 41=55760 42=135930 43=40787 44=6340",
    "2015 Qrtl4 Nov 45=462 46=127000 47=200180 48=176755 49=25040",
    "2015 Qrtl4 Dez 49=102355 50=138018 51=170987 52=79870 53=674",
    "2016 Qrtl1 Jan 1=110126 2=111760 3=50220 4=89037",
    "2016 Qrtl1 Feb 5=218326 6=11690 7=23634 8=6025 9=15800",
    "2016 Qrtl1 Mrz 9=234284 10=237285 11=135453 12=237844 13=6532",
    "2016 Qrtl2 Apr 13=14200 14=408480 15=103885 16=20301 17=112610",
    "2016 Qrtl2 Mai 18=76814 19=83956 20=79041 21=58981 22=47746",
    "2016 Qrtl2 Jun 22=228979 23=163267 24=143923 25=131318 26=4917",
    "2016 Qrtl3 Jul 26=12620 27=482282 28=116923 29=125696 30=99596",
    "2016 Qrtl3 Aug 31=101176 32=44621 33=62138 34=50730 35=27150",
    "2016 Qrtl3 Sep 35=8061 36=47691 37=62920 38=78186 39=229425",
    "2016 Qrtl4 Okt 40=93680 41=52105 42=90886 43=81472",
    "2016 Qrtl4 Nov 44=10248 45=123345 46=111874 47=138014 48=144108",
    "2016 Qrtl4 Dez 48=64290 49=171789 50=64926 51=63546",
    "2017 Qrtl1 Jan 1=16595 2=125050 3=121303 4=6999 5=4210",
    "2017 Qrtl1 Feb 5=39067 6=43035 7=62692 8=54850 9=6504",
    "2017 Qrtl1 Mrz 9=6304 10=32426 11=83262 12=111216 13=153868",
    "2017 Qrtl2 Apr 14=135270 15=41792 16=17853 17=250141",
    "2017 Qrtl2 Mai 18=87612 19=68934 20=50685 21=6000 22=1901",
    "2017 Qrtl2 Jun 22=8 23=35000 24=13600 25=105820 26=98605",
    "2017 Qrtl3 Jul 27=104360 28=161280 29=17008 30=6000",
];

/// The `c:numCache` points of forum-mso-de-104083.xlsx's three series (`c:tx` 2015, 2016,
/// 2017), as `idx=value`; an index with no point is an empty cell.
const FORUM_SERIES: [(&str, &str); 3] = [
    (
        "2015",
        "1=98665 2=124327 3=354247 4=192380 5=133415 6=125250 7=50230 8=298 9=93301 10=125953 11=117604 12=243775 13=59486 14=100116 15=169620 16=59392 18=6270 19=99764 20=266840 21=194160 22=134187 23=220480 24=163211 25=123088 26=129766 27=59812 28=18630 29=68405 30=139280 31=192521 32=93202 33=134397 34=110739 35=196556 36=57840 37=146631 38=210146 39=177926 40=55760 41=135930 42=40787 43=6340 44=462 45=127000 46=200180 47=176755 48=127395 49=138018 50=170987 51=79870 52=674",
    ),
    (
        "2016",
        "0=110126 1=111760 2=50220 3=89037 4=218326 5=11690 6=23634 7=6025 8=250084 9=237285 10=135453 11=237844 12=20732 13=408480 14=103885 15=20301 16=112610 17=76814 18=83956 19=79041 20=58981 21=276725 22=163267 23=143923 24=131318 25=17537 26=482282 27=116923 28=125696 29=99596 30=101176 31=44621 32=62138 33=50730 34=35211 35=47691 36=62920 37=78186 38=229425 39=93680 40=52105 41=90886 42=81472 43=10248 44=123345 45=111874 46=138014 47=208398 48=171789 49=64926 50=63546",
    ),
    (
        "2017",
        "0=16595 1=125050 2=121303 3=6999 4=43277 5=43035 6=62692 7=54850 8=12808 9=32426 10=83262 11=111216 12=153868 13=135270 14=41792 15=17853 16=250141 17=87612 18=68934 19=50685 20=6000 21=1909 22=35000 23=13600 24=105820 25=98605 26=104360 27=161280 28=17008 29=6000",
    ),
];

/// `Tabelle1!PivotTable2` of forum-mso-de-104083.xlsx: KW on rows (`numFmtId` 165, items in
/// numeric order), Jahre, Quartale and the date's months on columns with every year and quarter
/// collapsed (`sd="0"`), Summe (Sum of Menge, `numFmtId` 168), grand totals on, outline form.
fn forum_mso_de_104083() -> (PivotCache, PivotTable) {
    let mut records = Vec::new();
    for line in FORUM_RECORDS {
        let mut words = line.split(' ');
        let mut text = || CacheItem::Text(words.next().unwrap().to_string());
        let (year, quarter, month) = (text(), text(), text());
        for cell in line.split(' ').skip(3) {
            let (week, menge) = cell.split_once('=').unwrap();
            let number = |w: &str| CacheItem::Number(w.parse().unwrap());
            records.push(vec![
                number(week),
                year.clone(),
                quarter.clone(),
                month.clone(),
                number(menge),
            ]);
        }
    }
    let cache = cache_of(&["KW", "Jahre", "Quartale", "Datum", "Menge"], records);
    let anchor = A3;
    let mut table = PivotTable::new("PivotTable2".to_string(), 1, anchor, 0);
    for (index, field) in (0u32..).zip(&cache.fields) {
        let axis = match index {
            0 => Some(Axis::Row),
            1..=3 => Some(Axis::Column),
            _ => None,
        };
        let mut pivot_field = PivotField::new(axis, field.items.len());
        pivot_field.show_all = false;
        pivot_field.subtotal_top = false;
        let count = u32::try_from(field.items.len()).unwrap();
        let mut items: Vec<u32> = (0..count).collect();
        if index == 0 {
            // KW's `<items>` list 1 to 53 in numeric order.
            let value = |i: &u32| match field.items[*i as usize] {
                CacheItem::Number(n) => n,
                _ => panic!("KW is a number"),
            };
            items.sort_by(|a, b| value(a).total_cmp(&value(b)));
            pivot_field.number_format = Some(165);
        }
        pivot_field.items = items.into_iter().map(PivotItem::data).collect();
        // Every year and quarter item is `sd="0"`.
        if matches!(index, 1 | 2) {
            for item in &mut pivot_field.items {
                item.expanded = false;
            }
        }
        table.fields.push(pivot_field);
    }
    table.row_fields = vec![AxisField::Field(0)];
    table.column_fields = vec![
        AxisField::Field(1),
        AxisField::Field(2),
        AxisField::Field(3),
    ];
    table.data_fields = vec![DataField {
        caption: "Summe".to_string(),
        field: 4,
        number_format: Some(168),
        base_field: 0,
        base_item: 0,
        ..DataField::default()
    }];
    (cache, table)
}

#[test]
fn forum_mso_de_104083_plots_collapsed_years_and_formatted_weeks() {
    let (cache, table) = forum_mso_de_104083();
    let num_fmts = [NumFmt {
        num_fmt_id: 165,
        format_code: "\"KW \"00".to_string(),
    }];
    let (layout, data) = chart_formatted(&table, &cache, &num_fmts);
    // pivotTable1's `rowItems`: 53 items and the grand total; `colItems`: three collapsed years
    // and the grand total.
    assert_eq!((layout.rows.len(), layout.columns.len()), (54, 4));
    // `c:cat` `Tabelle1!$A$7:$A$60`: a `c:strCache` of KW 01 to KW 53, the grand total left out.
    let weeks: Vec<String> = (1..=53).map(|w| format!("KW {w:02}")).collect();
    let expected: Vec<Vec<&str>> = weeks.iter().map(|w| vec![w.as_str()]).collect();
    assert_eq!(labels(&data), expected);
    let points: Vec<(u32, &str)> = (0u32..).zip(weeks.iter().map(String::as_str)).collect();
    assert_eq!(data.category_levels(), vec![points]);
    // Three series, each `c:tx` cached as its year alone, ptCount 53, no point for an empty
    // cell; the grand-total column left out.
    assert_eq!(data.series.len(), 3);
    for (series, (year, cached)) in data.series.iter().zip(FORUM_SERIES) {
        assert_eq!(series.name(), Some(year));
        assert_eq!(series.values.len(), 53);
        let mine: Vec<String> = (0u32..)
            .zip(&series.values)
            .filter_map(|(i, v)| v.clone().map(|v| format!("{i}={}", v.unwrap())))
            .collect();
        assert_eq!(mine.join(" "), cached, "{year}");
    }
}

#[test]
fn the_t4130_chart_pivot_plots_its_items_as_one_total_series() {
    // probes/t4130-pivot-chart-cache C: the sidecar's cells give East 18 and West 27 (B4, B5)
    // and the Grand Total 45 (B6); the cached download's chart caches hold East, West, 18, 27
    // and the name `Total` (UNVERIFIED, T-4187: the download is not committed).
    let case = read("t4130-pivot-chart-cache", "case");
    let (cache, table) = probe_pivot(&case, "C");
    let (_, data) = chart(&table, &cache);
    assert_eq!(labels(&data), [["East"], ["West"]]);
    assert_eq!(data.category_levels(), vec![vec![(0, "East"), (1, "West")]]);
    assert_eq!(data.series.len(), 1);
    assert_eq!(data.series[0].name(), Some("Total"));
    assert_eq!(numbers(&data.series[0]), [Some(18.0), Some(27.0)]);
}

/// Checks the chart data of sheet `name`'s pivot of probe `stem` against the cells Excel saved
/// for it: each point is the value cell where its category's row line meets its series' column
/// line (blank when Excel's cell is empty). The plotted lines are checked against the same rule
/// applied to the sidecar's lines, so this shows the engine's lines and values are Excel's pivot
/// cells, not which lines Excel's chart plots (UNVERIFIED, T-4938). Returns the points checked.
fn check(stem: &str, name: &str) -> usize {
    let (case, oracle) = (read(stem, "case"), read(stem, "oracle"));
    let (cache, table) = probe_pivot(&case, name);
    let (layout, data) = chart(&table, &cache);
    let pivot = (oracle["pivots"].as_array().unwrap().iter())
        .find(|p| p["sheet"] == name)
        .unwrap();
    let points = |key: &str| -> Vec<u32> {
        let lines = pivot[key].as_array().unwrap();
        let depth =
            |l: &Json| l["r"].as_u64().unwrap_or(0) + l["x"].as_array().unwrap().len() as u64;
        (0u32..)
            .zip(lines)
            .filter(|(i, l)| {
                let item = l["t"].as_str().unwrap_or("data") == "data";
                let next = lines.get(*i as usize + 1);
                let parent = next.is_some_and(|n| {
                    n["t"].as_str() != Some("grand") && n["r"].as_u64().unwrap_or(0) >= depth(l)
                });
                item && !parent
            })
            .map(|(i, _)| i)
            .collect()
    };
    let (rows, columns) = (points("row_items"), points("col_items"));
    // An item's line in outline form whose subtotal is not on it holds no values; it always has
    // children, so it is never a point.
    assert_eq!(
        data.categories.iter().map(|c| c.line).collect::<Vec<_>>(),
        rows,
        "{stem} {name} categories"
    );
    let series_lines: Vec<u32> = data.series.iter().map(|s| s.line).collect();
    assert_eq!(series_lines, columns, "{stem} {name} series");
    let (top_column, top_row) = position(
        pivot["location"]
            .as_str()
            .unwrap()
            .split(':')
            .next()
            .unwrap(),
    );
    let offset = |key: &str| u32::try_from(pivot[key].as_u64().unwrap()).unwrap();
    let (data_row, data_column) = (
        top_row + offset("first_data_row"),
        top_column + offset("first_data_col"),
    );
    assert_eq!(
        layout.data_origin(),
        (data_row, data_column),
        "{stem} {name}"
    );
    let cells = pivot["cells"].as_object().unwrap();
    let mut checked = 0;
    for series in &data.series {
        for (category, value) in data.categories.iter().zip(&series.values) {
            let at = crate::pivot::probe_cases::address(
                data_column + series.line,
                data_row + category.line,
            );
            // The sidecar writes an empty cell of the capture window as `{"t": "empty"}`.
            let excel = cells.get(&at).filter(|c| c["t"] != "empty");
            match (value, excel) {
                (None, None) => {}
                (Some(Ok(mine)), Some(cell)) if cell["t"] == "n" => {
                    let excel = cell["v"].as_f64().unwrap();
                    let close = (mine - excel).abs() <= 1e-12 * excel.abs().max(1.0);
                    assert!(close, "{stem} {name} {at}: Excel {excel}, chart {mine}");
                }
                (Some(Err(error)), Some(cell)) if cell["t"] == "e" => {
                    assert_eq!(
                        cell["v"].as_str().unwrap(),
                        error.to_string(),
                        "{stem} {name} {at}"
                    );
                }
                (mine, excel) => panic!("{stem} {name} {at}: Excel {excel:?}, chart {mine:?}"),
            }
            checked += 1;
        }
    }
    checked
}

#[test]
fn every_point_of_the_rendered_probe_pivots_is_excels_cell() {
    let sheets: &[(&str, &[&str])] = &[
        // Compact, outline and tabular rows with subtotals, two column fields, Values on rows
        // and on columns, no row field, no axis field.
        (
            "t4138-pivot-geometry",
            &["G1", "G2", "G3", "G4", "G5", "G6", "G7", "G9"],
        ),
        // Show values as: every mode Excel rendered.
        (
            "t4129-pivot-show-as",
            &["N", "DF", "DP", "PC", "PD", "RT", "PR", "PCo", "PT", "IX"],
        ),
        // Collapsed items (compact, tabular), blank rows, custom subtotals.
        (
            "t4168-pivot-layout-options",
            &["SD", "SD2", "BR", "BR2", "F1", "F2", "F3"],
        ),
        // A hidden inner item, with and without its records in the subtotals.
        ("t4164-pivot-filter-totals", &["SH0", "SH1H"]),
        // Two fields on each axis.
        ("t4634-pivot-total-order-4", &["T1", "T2", "T3"]),
    ];
    let mut checked = 0;
    for (stem, names) in sheets {
        for name in *names {
            checked += check(stem, name);
        }
    }
    assert!(checked > 200, "{checked} points");
}

#[test]
fn a_collapsed_item_is_one_category_and_a_subtotal_none() {
    // t4168 SD: Region with one collapsed item, in compact form with subtotals at the top.
    let case = read("t4168-pivot-layout-options", "case");
    let (cache, table) = probe_pivot(&case, "SD");
    let (layout, data) = chart(&table, &cache);
    let depths: Vec<usize> = data.categories.iter().map(|c| c.labels.len()).collect();
    assert!(depths.contains(&1) && depths.contains(&2), "{depths:?}");
    for category in &data.categories {
        let line = &layout.rows[category.line as usize];
        assert_eq!(line.kind, ItemType::Data);
    }
    // The collapsed category has no label at the inner level.
    let levels = data.category_levels();
    assert_eq!(levels.len(), 2);
    assert!(levels[0].len() < data.categories.len());
}

#[test]
fn the_chart_data_is_bounded_by_its_work() {
    let (cache, table) = with_chart_sheet();
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    let layout = lay_out(&table, &cache, &result, &mut work).unwrap();
    let mut little = 3;
    let err = chart_data(&table, &cache, &result, &layout, &[], &mut little);
    assert_eq!(err, Err(PivotComputeError::Limit));
    // A layout that is not over the result (a line's node not in its tree) is malformed.
    let mut broken = layout.clone();
    broken.rows[0].node = u32::MAX;
    let mut work = MAX_PIVOT_WORK;
    let err = chart_data(&table, &cache, &result, &broken, &[], &mut work);
    assert_eq!(err, Err(PivotComputeError::Malformed));
}

#[test]
fn a_series_named_by_several_labels_has_no_name() {
    let series = ChartSeries {
        labels: vec!["East".to_string(), "Sum of Sales".to_string()],
        line: 0,
        values: Vec::new(),
    };
    assert_eq!(series.name(), None);
}

#[test]
fn a_collapsed_items_data_field_lines_plot_with_an_empty_hidden_label() {
    // probes/t4699-pivot-blank-collapsed CV3 and CV4: East collapsed over a hidden Product, with
    // Sum and Count of Val on rows inside it. Each data field line plots, its hidden level an
    // empty label (UNVERIFIED, T-4938).
    let case = read("t4699-pivot-blank-collapsed", "case");
    for sheet in ["CV3", "CV4"] {
        let (cache, table) = probe_pivot(&case, sheet);
        let (_, data) = chart(&table, &cache);
        let labels = labels(&data);
        assert_eq!(labels[0], ["East", "", "Sum of Val"], "{sheet}");
        assert_eq!(labels[1], ["East", "", "Count of Val"], "{sheet}");
        assert_eq!(labels[2], ["West", "A", "Sum of Val"], "{sheet}");
        assert_eq!(data.series.len(), 1, "{sheet}");
        let values = numbers(&data.series[0]);
        assert_eq!(values[..3], [Some(21.0), Some(4.0), Some(7.0)], "{sheet}");
    }
}
