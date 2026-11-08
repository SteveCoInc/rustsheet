#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "tests"
)]
//! Pivot chart data (`rustsheet_engine::pivot::chart_data`, T-4186) against the caches Excel
//! wrote in the chart parts of the two corpus files that hold a pivot chart: POI's
//! `WithChartSheet.xlsx` (the `poi` suite) and LibreOffice's `forum-mso-de-104083.xlsx` (the
//! `pivot-corpus-lo` suite). Each file is imported, its pivot laid out again from its saved
//! definition and cache, and every series' name, categories and values compared with the `c:tx`,
//! `c:cat` and `c:val` caches of the chart whose `c:pivotSource` names it.
//!
//! An optional extra check: the corpus is fetched (`rustsheet-eval fetch --suite poi --suite
//! pivot-corpus-lo`), not committed, and the PR gate does not fetch it, so a file that is not
//! there is skipped with a note. The gated evidence is the transcription of both files in
//! crates/rustsheet-engine/src/pivot/layout/chart/tests.rs.

use std::io::Read;
use std::path::PathBuf;

use quick_xml::events::Event;
use quick_xml::{Reader, XmlVersion};
use rustsheet_engine::pivot::{chart_data, compute, lay_out, PivotChartData, MAX_PIVOT_WORK};
use rustsheet_xlsx::import::load_from_xlsx_bytes;

/// One series of a chart part as its caches hold it.
#[derive(Debug, Default)]
struct CachedSeries {
    name: Vec<String>,
    /// Per level, innermost first, its `(idx, label)`s; one level for a `c:strCache`.
    categories: Vec<Vec<(u32, String)>>,
    category_count: u32,
    values: Vec<(u32, f64)>,
    value_count: u32,
}

/// The `c:pivotSource` name and the series caches of a chart part.
fn read_chart(xml: &str) -> (String, Vec<CachedSeries>) {
    let mut reader = Reader::from_str(xml);
    let (mut path, mut source, mut series) = (Vec::<String>::new(), String::new(), Vec::new());
    let mut current = CachedSeries::default();
    let mut point: Option<u32> = None;
    loop {
        match reader.read_event().unwrap() {
            Event::Start(e) => {
                let name = e.local_name().as_ref().to_string();
                if name == "pt" {
                    let idx = e.try_get_attribute("idx").unwrap().unwrap();
                    point = Some(
                        idx.normalized_value(XmlVersion::Implicit1_0)
                            .unwrap()
                            .parse()
                            .unwrap(),
                    );
                }
                if name == "lvl" {
                    current.categories.push(Vec::new());
                }
                if name == "strCache" && path.iter().any(|p| p == "cat") {
                    current.categories.push(Vec::new());
                }
                if name == "ser" {
                    current = CachedSeries::default();
                }
                path.push(name);
            }
            Event::Empty(e) => {
                if e.local_name().as_ref() == "ptCount" {
                    let count = e.try_get_attribute("val").unwrap().unwrap();
                    let count: u32 = count
                        .normalized_value(XmlVersion::Implicit1_0)
                        .unwrap()
                        .parse()
                        .unwrap();
                    if path.iter().any(|p| p == "cat") {
                        current.category_count = count;
                    } else if path.iter().any(|p| p == "val") {
                        current.value_count = count;
                    }
                }
            }
            Event::Text(t) => {
                let text = t.xml_content(XmlVersion::Implicit1_0).to_string();
                let within = |name: &str| path.iter().any(|p| p == name);
                if path.last().map(String::as_str) == Some("name") && within("pivotSource") {
                    source = text;
                } else if path.last().map(String::as_str) == Some("v") {
                    let idx = point.unwrap();
                    if within("tx") {
                        current.name.push(text);
                    } else if within("cat") {
                        current.categories.last_mut().unwrap().push((idx, text));
                    } else if within("val") {
                        current.values.push((idx, text.parse().unwrap()));
                    }
                }
            }
            Event::End(e) => {
                if e.local_name().as_ref() == "ser" {
                    series.push(std::mem::take(&mut current));
                }
                path.pop();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    (source, series)
}

/// The fetched corpus file `relative` of suite `suite`, if it is there.
fn corpus(suite: &str, relative: &str) -> Option<Vec<u8>> {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "../../eval/.cache/suites",
        suite,
        relative,
    ]
    .iter()
    .collect();
    if let Ok(bytes) = std::fs::read(&path) {
        Some(bytes)
    } else {
        eprintln!("skipped: {} is not fetched", path.display());
        None
    }
}

/// The chart parts of an xlsx package.
fn charts(bytes: &[u8]) -> Vec<String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut out = Vec::new();
    for index in 0..archive.len() {
        let mut part = archive.by_index(index).unwrap();
        if part.name().is_ok_and(|n| n.starts_with("xl/charts/chart")) {
            let mut xml = String::new();
            part.read_to_string(&mut xml).unwrap();
            out.push(xml);
        }
    }
    out
}

/// The chart data of the pivot `[file]Sheet!Name` names, laid out again from the import.
fn pivot_chart(bytes: &[u8], source: &str) -> PivotChartData {
    let (sheet, name) = source.rsplit_once(']').unwrap().1.split_once('!').unwrap();
    let workbook = load_from_xlsx_bytes(bytes, "corpus", "en", "UTC").unwrap();
    let sheet_id = (workbook.worksheets.iter())
        .find(|s| s.name == sheet)
        .unwrap()
        .sheet_id;
    let table = (workbook.pivot_tables.iter())
        .find(|p| p.sheet_id == sheet_id && p.name == name)
        .unwrap();
    let cache = &workbook.pivot_caches[table.cache as usize];
    let mut work = MAX_PIVOT_WORK;
    let result = compute(table, cache, &mut work).unwrap();
    let layout = lay_out(table, cache, &result, &mut work).unwrap();
    let num_fmts = &workbook.styles.num_fmts;
    chart_data(table, cache, &result, &layout, num_fmts, &mut work).unwrap()
}

/// Checks every chart of `bytes` bound to a pivot; returns the points compared.
fn check(file: &str, bytes: &[u8]) -> usize {
    let mut points = 0;
    for xml in charts(bytes) {
        let (source, cached) = read_chart(&xml);
        if source.is_empty() {
            continue;
        }
        let data = pivot_chart(bytes, &source);
        assert_eq!(data.series.len(), cached.len(), "{file}: series");
        let count = u32::try_from(data.categories.len()).unwrap();
        let levels: Vec<Vec<(u32, String)>> = (data.category_levels().into_iter())
            .map(|level| level.into_iter().map(|(i, l)| (i, l.to_string())).collect())
            .collect();
        for (series, excel) in data.series.iter().zip(&cached) {
            // A series name cache holds one point, the name.
            assert_eq!(
                series.name().map(|n| vec![n.to_string()]),
                Some(excel.name.clone()),
                "{file}: name"
            );
            assert_eq!(count, excel.category_count, "{file}: categories");
            assert_eq!(count, excel.value_count, "{file}: values");
            assert_eq!(levels, excel.categories, "{file}: category labels");
            // A blank point has no `c:pt`.
            let mine: Vec<(u32, f64)> = (0u32..)
                .zip(&series.values)
                .filter_map(|(i, v)| v.clone().map(|v| (i, v.unwrap())))
                .collect();
            assert_eq!(mine, excel.values, "{file}: {:?} values", series.name());
            points += mine.len();
        }
    }
    points
}

#[test]
fn with_chart_sheet_chart_caches_are_the_pivots_chart_data() {
    let file = "WithChartSheet.xlsx";
    if let Some(bytes) = corpus("poi", file) {
        assert_eq!(check(file, &bytes), 6);
    }
}

#[test]
fn forum_mso_de_104083_chart_caches_are_the_pivots_chart_data() {
    let file = "xlsx/forum-mso-de-104083.xlsx";
    if let Some(bytes) = corpus("pivot-corpus-lo", file) {
        assert_eq!(check(file, &bytes), 132);
    }
}
