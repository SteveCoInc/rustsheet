#![allow(clippy::unwrap_used, reason = "tests")]
//! The rewrite against the chart parts Excel for the web saved for
//! probes/t4187-pivot-chart-caches (committed byte for byte as
//! eval/roundtrip/excel-saved/cases/t4187-pivot-chart-caches.xlsx): Excel's own part is a fixed
//! point, and a part with Excel's input series (one series, stale caches, as
//! `tools/src/rustsheet_tools/pivot_chart.py` writes it) becomes Excel's.

use std::io::Read;

use rustsheet_engine::pivot::{
    chart_data, compute, lay_out, PivotChartData, PivotLayout, MAX_PIVOT_WORK,
};
use rustsheet_engine::types::Workbook;

use super::{rewrite_pivot_chart, PivotChartError, PivotChartFrame};
use crate::import::load_from_xlsx_bytes;

const SAVED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../eval/roundtrip/excel-saved/cases/t4187-pivot-chart-caches.xlsx"
);

fn saved() -> Vec<u8> {
    std::fs::read(SAVED).unwrap()
}

/// The part `name` of the package `bytes`.
fn part(bytes: &[u8], name: &str) -> String {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut xml = String::new();
    archive
        .by_name(name)
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml
}

/// The layout and chart data of the pivot `name` on `sheet`, laid out again from the import.
fn plotted(workbook: &Workbook, sheet: &str, name: &str) -> (usize, PivotLayout, PivotChartData) {
    let sheet_id = (workbook.worksheets.iter())
        .find(|s| s.name == sheet)
        .unwrap()
        .sheet_id;
    let index = (workbook.pivot_tables.iter())
        .position(|p| p.sheet_id == sheet_id && p.name == name)
        .unwrap();
    let table = &workbook.pivot_tables[index];
    let cache = &workbook.pivot_caches[table.cache as usize];
    let mut work = MAX_PIVOT_WORK;
    let result = compute(table, cache, &mut work).unwrap();
    let layout = lay_out(table, cache, &result, &mut work).unwrap();
    let data = chart_data(
        table,
        cache,
        &result,
        &layout,
        &workbook.styles.num_fmts,
        &mut work,
    )
    .unwrap();
    (index, layout, data)
}

/// `chart` (a part of the saved file) rewritten from the pivot `name` on `sheet`.
fn rewrite(
    chart: &str,
    sheet: &str,
    name: &str,
    collapsed: bool,
) -> Result<String, PivotChartError> {
    let bytes = saved();
    let workbook = load_from_xlsx_bytes(&bytes, "t4187", "en", "UTC").unwrap();
    let (index, layout, data) = plotted(&workbook, sheet, name);
    let frame = PivotChartFrame {
        sheet,
        table: &workbook.pivot_tables[index],
        layout: &layout,
        data: &data,
        num_fmts: &workbook.styles.num_fmts,
        collapsed,
    };
    rewrite_pivot_chart(chart, &frame)
}

/// The text from the first `<c:ser>` to the end of the last `</c:ser>`.
fn series_span(xml: &str) -> (usize, usize) {
    (
        xml.find("<c:ser>").unwrap(),
        xml.rfind("</c:ser>").unwrap() + "</c:ser>".len(),
    )
}

/// `excel` with its series replaced by one stale series as `pivot_chart.py` writes it: the
/// probe's input references and caches, Excel's formatting children around them.
fn stale(excel: &str, sheet: &str) -> String {
    let (from, to) = series_span(excel);
    let first = &excel[from..to];
    let first = &first[..first.find("</c:ser>").unwrap() + "</c:ser>".len()];
    let (tx, val_end) = (
        first.find("<c:tx>").unwrap(),
        first.find("</c:val>").unwrap(),
    );
    let stale_series = format!(
        "{}<c:tx><c:strRef><c:f>{sheet}!$B$3</c:f><c:strCache><c:ptCount val=\"1\"/><c:pt idx=\"0\">\
         <c:v>Stale</c:v></c:pt></c:strCache></c:strRef></c:tx><c:invertIfNegative val=\"0\"/><c:cat>\
         <c:strRef><c:f>{sheet}!$A$4:$A$5</c:f><c:strCache><c:ptCount val=\"2\"/><c:pt idx=\"0\"><c:v>StaleX\
         </c:v></c:pt><c:pt idx=\"1\"><c:v>StaleY</c:v></c:pt></c:strCache></c:strRef></c:cat><c:val>\
         <c:numRef><c:f>{sheet}!$B$4:$B$5</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount \
         val=\"2\"/><c:pt idx=\"0\"><c:v>999</c:v></c:pt><c:pt idx=\"1\"><c:v>998</c:v></c:pt></c:numCache>\
         </c:numRef></c:val>{}",
        &first[..tx],
        &first[val_end + "</c:val>".len()..],
    );
    format!("{}{stale_series}{}", &excel[..from], &excel[to..])
}

/// The saved charts: (part, sheet, pivot, collapsed). chart2 is CS, whose pivot collapsed to
/// `#SPILL!` (its `blocker` at B5).
const CHARTS: [(&str, &str, &str, bool); 4] = [
    ("xl/charts/chart1.xml", "C", "P_C", false),
    ("xl/charts/chart2.xml", "CS", "P_CS", true),
    ("xl/charts/chart3.xml", "F", "P_F", false),
    ("xl/charts/chart4.xml", "M", "P_M", false),
];

#[test]
fn excels_saved_chart_parts_are_a_fixed_point() {
    let bytes = saved();
    for (name, sheet, pivot, collapsed) in CHARTS {
        let excel = part(&bytes, name);
        assert_eq!(
            rewrite(&excel, sheet, pivot, collapsed).unwrap(),
            excel,
            "{name}"
        );
    }
}

#[test]
fn stale_caches_become_the_ones_excel_wrote() {
    let bytes = saved();
    for (name, sheet, pivot, collapsed) in CHARTS {
        let excel = part(&bytes, name);
        let input = stale(&excel, sheet);
        assert_ne!(input, excel);
        assert_eq!(
            rewrite(&input, sheet, pivot, collapsed).unwrap(),
            excel,
            "{name}"
        );
    }
}

#[test]
fn excels_rewrite_is_what_pivots_md_records() {
    let bytes = saved();
    let caches = |name| {
        let xml = part(&bytes, name);
        let (from, to) = series_span(&xml);
        xml[from..to].to_string()
    };
    let c = caches("xl/charts/chart1.xml");
    assert!(c.contains("<c:f>'C'!$B$3</c:f>"), "{c}");
    assert!(c.contains("<c:f>'C'!$A$4:$A$6</c:f>"));
    assert!(c.contains(
        "<c:formatCode>General</c:formatCode><c:ptCount val=\"2\"/><c:pt idx=\"0\"><c:v>18</c:v>"
    ));
    let cs = caches("xl/charts/chart2.xml");
    assert_eq!(cs.matches("<c:f>CS!$A$3</c:f>").count(), 3);
    assert!(cs.contains("<c:ptCount val=\"2\"/><c:pt idx=\"0\"><c:v>0</c:v></c:pt></c:numCache>"));
    assert!(caches("xl/charts/chart3.xml").contains("<c:formatCode>#,##0.00</c:formatCode>"));
    let m = caches("xl/charts/chart4.xml");
    assert_eq!(m.matches("<c:ser>").count(), 3);
    assert!(m.contains("<c:f>M!$C$3:$C$4</c:f>"));
    // West has no record of product C: no point 1.
    assert!(m.contains("<c:f>M!$D$5:$D$7</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val=\"2\"/><c:pt idx=\"0\"><c:v>3</c:v></c:pt></c:numCache>"));
}

#[test]
fn a_part_without_pivot_source_or_series_is_refused() {
    let bytes = saved();
    let excel = part(&bytes, "xl/charts/chart1.xml");
    let start = excel.find("<c:pivotSource>").unwrap();
    let end = excel.find("</c:pivotSource>").unwrap() + "</c:pivotSource>".len();
    let unbound = format!("{}{}", &excel[..start], &excel[end..]);
    assert_eq!(
        rewrite(&unbound, "C", "P_C", false),
        Err(PivotChartError::NotPivotChart)
    );
    let (from, to) = series_span(&excel);
    let empty = format!("{}{}", &excel[..from], &excel[to..]);
    assert!(matches!(
        rewrite(&empty, "C", "P_C", false),
        Err(PivotChartError::Unsupported(_))
    ));
    assert!(matches!(
        rewrite("<c:chartSpace><c:ser>", "C", "P_C", false),
        Err(PivotChartError::Malformed(_))
    ));
    assert!(matches!(
        rewrite("<a/>", "C", "P_C", false),
        Err(PivotChartError::Malformed(_))
    ));
}

#[test]
fn series_in_two_chart_groups_are_refused() {
    let bytes = saved();
    let excel = part(&bytes, "xl/charts/chart1.xml");
    let (from, to) = series_span(&excel);
    let line = format!("<c:lineChart>{}</c:lineChart>", &excel[from..to]);
    let at = excel.find("</c:barChart>").unwrap() + "</c:barChart>".len();
    let combo = format!("{}{line}{}", &excel[..at], &excel[at..]);
    assert_eq!(
        rewrite(&combo, "C", "P_C", false),
        Err(PivotChartError::Unsupported(
            "series in more than one chart group"
        ))
    );
}

#[test]
fn a_deep_part_is_refused_without_overflow() {
    let deep = format!(
        "<c:chartSpace>{}{}</c:chartSpace>",
        "<a>".repeat(10_000),
        "</a>".repeat(10_000)
    );
    assert!(matches!(
        rewrite(&deep, "C", "P_C", false),
        Err(PivotChartError::Malformed(_))
    ));
}

#[test]
fn labels_and_references_are_escaped() {
    let bytes = saved();
    let excel = part(&bytes, "xl/charts/chart1.xml");
    let workbook = load_from_xlsx_bytes(&bytes, "t4187", "en", "UTC").unwrap();
    let (index, layout, mut data) = plotted(&workbook, "C", "P_C");
    data.categories[0].labels = vec!["A&B <x>".into()];
    let frame = PivotChartFrame {
        sheet: "R&D's",
        table: &workbook.pivot_tables[index],
        layout: &layout,
        data: &data,
        num_fmts: &workbook.styles.num_fmts,
        collapsed: false,
    };
    let out = rewrite_pivot_chart(&excel, &frame).unwrap();
    assert!(out.contains("<c:f>'R&amp;D''s'!$B$3</c:f>"), "{out}");
    assert!(out.contains("<c:v>A&amp;B &lt;x&gt;</c:v>"));
}

#[test]
fn a_large_template_times_many_series_is_refused() {
    use rustsheet_engine::pivot::ChartSeries;
    let bytes = saved();
    let excel = part(&bytes, "xl/charts/chart1.xml");
    // One series carrying a 1 MiB formatting child, over 100 series: 100 MiB copied.
    let padding = format!("<c:spPr><a:ln>{}</a:ln></c:spPr>", " ".repeat(1 << 20));
    let at = excel.find("<c:invertIfNegative").unwrap();
    let large = format!("{}{padding}{}", &excel[..at], &excel[at..]);
    let workbook = load_from_xlsx_bytes(&bytes, "t4187", "en", "UTC").unwrap();
    let (index, layout, mut data) = plotted(&workbook, "C", "P_C");
    let one: ChartSeries = data.series[0].clone();
    data.series = vec![one; 100];
    let frame = PivotChartFrame {
        sheet: "C",
        table: &workbook.pivot_tables[index],
        layout: &layout,
        data: &data,
        num_fmts: &workbook.styles.num_fmts,
        collapsed: false,
    };
    assert_eq!(
        rewrite_pivot_chart(&large, &frame),
        Err(PivotChartError::TooLarge)
    );
    // Under the cap it is written.
    let mut fewer = data.clone();
    fewer.series.truncate(10);
    let frame = PivotChartFrame {
        data: &fewer,
        ..frame
    };
    let out = rewrite_pivot_chart(&large, &frame).unwrap();
    assert!(out.len() > 10 << 20 && out.len() <= super::MAX_CHART_PART_BYTES);
}
