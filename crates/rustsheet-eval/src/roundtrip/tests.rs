use std::fmt::Write as _;
use std::io::{Cursor, Write as _};

use super::package::{validate, Package};
use super::*;
use crate::baseline;

const CT: &str = r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/></Types>"#;
const ROOT_RELS: &str = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#;
const R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

fn workbook(sheets: &[(&str, &str)]) -> String {
    let mut body = String::new();
    for (n, (name, rid)) in sheets.iter().enumerate() {
        let _ = write!(
            body,
            r#"<sheet name="{name}" sheetId="{}" r:id="{rid}"/>"#,
            n + 1
        );
    }
    let sheets = body;
    format!(
        r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="{R}"><sheets>{sheets}</sheets></workbook>"#
    )
}

fn rels(entries: &[(&str, &str, &str)]) -> String {
    let mut body = String::new();
    for (id, kind, target) in entries {
        let _ = write!(
            body,
            r#"<Relationship Id="{id}" Type="{R}/{kind}" Target="{target}"/>"#
        );
    }
    format!(
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{body}</Relationships>"#
    )
}

const SHEET: &str =
    r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"/>"#;

fn zip(files: &[(&str, &str)]) -> Vec<u8> {
    let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default();
    for (name, body) in files {
        w.start_file(*name, opts).expect("start");
        w.write_all(body.as_bytes()).expect("write");
    }
    w.finish().expect("finish").into_inner()
}

/// A valid two-sheet package; `sheet2` names the second sheet's part.
fn two_sheets(sheet2: &str) -> Vec<u8> {
    let wb_rels = rels(&[
        ("rId1", "worksheet", "worksheets/sheet1.xml"),
        ("rId2", "worksheet", &format!("worksheets/{sheet2}")),
        ("rId3", "styles", "styles.xml"),
    ]);
    zip(&[
        ("[Content_Types].xml", CT),
        ("_rels/.rels", ROOT_RELS),
        (
            "xl/workbook.xml",
            &workbook(&[("One", "rId1"), ("Two", "rId2")]),
        ),
        ("xl/_rels/workbook.xml.rels", &wb_rels),
        ("xl/worksheets/sheet1.xml", SHEET),
        (&format!("xl/worksheets/{sheet2}"), SHEET),
        ("xl/styles.xml", "<styleSheet/>"),
    ])
}

fn checks(bytes: &[u8]) -> Vec<&'static str> {
    let pkg = Package::read(bytes).expect("package reads");
    validate(&pkg).into_iter().map(|p| p.check).collect()
}

#[test]
fn a_valid_package_passes_every_check() {
    assert_eq!(checks(&two_sheets("sheet2.xml")), Vec::<&str>::new());
}

#[test]
fn part_keys_do_not_depend_on_part_names() {
    let a = Package::read(&two_sheets("sheet2.xml")).expect("a");
    let b = Package::read(&two_sheets("renamed.xml")).expect("b");
    let keys = |p: &Package| p.keyed_parts().keys().cloned().collect::<Vec<_>>();
    assert_eq!(keys(&a), keys(&b));
    assert!(keys(&a).contains(&"/officeDocument[0]/worksheet[Two]".to_string()));
    assert!(part_items("R0", &a, &b)
        .0
        .iter()
        .all(|i| i.verdict == Verdict::Kept));
}

#[test]
fn a_lost_part_and_an_added_part_are_found() {
    let src = Package::read(&two_sheets("sheet2.xml")).expect("src");
    let wb_rels = rels(&[
        ("rId1", "worksheet", "worksheets/sheet1.xml"),
        ("rId2", "worksheet", "worksheets/sheet2.xml"),
        ("rId4", "theme", "theme/theme1.xml"),
    ]);
    let out = Package::read(&zip(&[
        ("[Content_Types].xml", CT),
        ("_rels/.rels", ROOT_RELS),
        (
            "xl/workbook.xml",
            &workbook(&[("One", "rId1"), ("Two", "rId2")]),
        ),
        ("xl/_rels/workbook.xml.rels", &wb_rels),
        ("xl/worksheets/sheet1.xml", SHEET),
        ("xl/worksheets/sheet2.xml", SHEET),
        ("xl/theme/theme1.xml", "<theme/>"),
    ]))
    .expect("out");
    let (items, _) = part_items("R0", &src, &out);
    let verdict = |key: &str| items.iter().find(|i| i.key == key).map(|i| i.verdict);
    assert_eq!(verdict("/officeDocument[0]/styles[0]"), Some(Verdict::Lost));
    assert_eq!(verdict("/officeDocument[0]/theme[0]"), Some(Verdict::Added));
    assert_eq!(
        verdict("/officeDocument[0]/worksheet[One]"),
        Some(Verdict::Kept)
    );
}

#[test]
fn v1_a_part_without_a_content_type_and_a_dangling_override() {
    let ct = CT.replace(
        "</Types>",
        r#"<Override PartName="/xl/missing.xml" ContentType="application/xml"/></Types>"#,
    );
    let mut files = vec![
        ("[Content_Types].xml", ct.as_str()),
        ("_rels/.rels", ROOT_RELS),
        ("xl/workbook.xml", "<workbook/>"),
        ("xl/media/image1.png", "png"),
    ];
    files.sort_unstable();
    let found = checks(&zip(&files));
    assert_eq!(found.iter().filter(|c| **c == "V1").count(), 2, "{found:?}");
}

#[test]
fn v2_a_missing_target_and_v3_a_repeated_id() {
    let wb_rels = rels(&[
        ("rId1", "worksheet", "worksheets/sheet1.xml"),
        ("rId1", "styles", "styles.xml"),
    ]);
    let found = checks(&zip(&[
        ("[Content_Types].xml", CT),
        ("_rels/.rels", ROOT_RELS),
        ("xl/workbook.xml", &workbook(&[("One", "rId1")])),
        ("xl/_rels/workbook.xml.rels", &wb_rels),
        ("xl/worksheets/sheet1.xml", SHEET),
    ]));
    assert!(found.contains(&"V2"), "{found:?}");
    assert!(found.contains(&"V3"), "{found:?}");
}

#[test]
fn v4_a_dangling_relationship_reference() {
    let wb_rels = rels(&[("rId1", "worksheet", "worksheets/sheet1.xml")]);
    let found = checks(&zip(&[
        ("[Content_Types].xml", CT),
        ("_rels/.rels", ROOT_RELS),
        (
            "xl/workbook.xml",
            &workbook(&[("One", "rId1"), ("Two", "rId9")]),
        ),
        ("xl/_rels/workbook.xml.rels", &wb_rels),
        ("xl/worksheets/sheet1.xml", SHEET),
    ]));
    assert!(found.contains(&"V4"), "{found:?}");
}

#[test]
fn v5_two_entries_naming_one_part() {
    let found = checks(&zip(&[
        ("[Content_Types].xml", CT),
        ("_rels/.rels", ROOT_RELS),
        ("xl/workbook.xml", &workbook(&[])),
        ("XL/Workbook.xml", &workbook(&[])),
    ]));
    assert_eq!(found, vec!["V5"]);
}

#[test]
fn v6_repeated_and_invalid_sheet_names() {
    let wb_rels = rels(&[
        ("rId1", "worksheet", "worksheets/sheet1.xml"),
        ("rId2", "worksheet", "worksheets/sheet2.xml"),
        ("rId3", "styles", "styles.xml"),
    ]);
    let found = checks(&zip(&[
        ("[Content_Types].xml", CT),
        ("_rels/.rels", ROOT_RELS),
        (
            "xl/workbook.xml",
            &workbook(&[("One", "rId1"), ("one", "rId2"), ("a[b", "rId3")]),
        ),
        ("xl/_rels/workbook.xml.rels", &wb_rels),
        ("xl/worksheets/sheet1.xml", SHEET),
        ("xl/worksheets/sheet2.xml", SHEET),
        ("xl/styles.xml", "<styleSheet/>"),
    ]));
    // "one" repeats "One"; "a[b" is an invalid name and points at the styles part.
    assert_eq!(found, vec!["V6", "V6", "V6"]);
}

#[test]
fn v8_and_r0_run_on_a_real_workbook() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../eval/suites/fixtures/cases");
    let path = root.join("circular-keeps-cached-values.xlsx");
    let bytes = std::fs::read(&path).expect("fixture");
    let Ok((items, _, _)) = r0(&bytes, Some("circular-keeps-cached-values.xlsx")) else {
        panic!("R0 runs on a fixture");
    };
    assert!(items.iter().all(|i| i.kind != "package"), "{items:?}");
    assert!(items
        .iter()
        .any(|i| i.key == "/officeDocument[0]" && i.verdict == Verdict::Kept));
}

#[test]
fn a_new_loss_in_an_unchanged_case_is_a_regression() {
    let item = |key: &str, verdict| Item {
        scenario: "R0",
        kind: "part",
        key: key.into(),
        verdict,
        detail: None,
        candidate: None,
        weight: super::xmldiff::unit_weight(verdict),
    };
    let case = |items: Vec<Item>| CaseResult {
        id: "rt/a.xlsx".into(),
        suite: "rt".into(),
        status: if items.iter().all(|i| i.verdict.passes()) {
            CaseStatus::Pass
        } else {
            CaseStatus::Fail
        },
        message: None,
        millis: 0,
        checked: items.len(),
        scenarios: std::collections::BTreeMap::from([("R0", items.len())]),
        items,
    };
    let before = case(vec![
        item("/a[0]", Verdict::Kept),
        item("/b[0]", Verdict::Kept),
    ]);
    let after = case(vec![
        item("/a[0]", Verdict::Kept),
        item("/b[0]", Verdict::Lost),
    ]);
    let delta = baseline::diff(
        &baseline(&[&before], BaselineMode::Cells),
        &baseline(&[&after], BaselineMode::Cells),
    );
    assert_eq!(
        delta.regressions,
        vec!["rt/a.xlsx\tR0 part /b[0] lost".to_string()]
    );
    let fixed = baseline::diff(
        &baseline(&[&after], BaselineMode::Cells),
        &baseline(&[&before], BaselineMode::Cells),
    );
    assert_eq!(fixed.fixed.len(), 1);
}

/// A failing case of `checked` items with `(key, weight)` items lost.
fn lossy(checked: usize, lost: &[(&str, usize)]) -> CaseResult {
    CaseResult {
        id: "rt/a.xlsx".into(),
        suite: "rt".into(),
        status: if lost.is_empty() {
            CaseStatus::Pass
        } else {
            CaseStatus::Fail
        },
        message: None,
        millis: 0,
        checked,
        scenarios: std::collections::BTreeMap::from([("R0", checked)]),
        items: lost
            .iter()
            .map(|(key, weight)| Item {
                scenario: "R0",
                kind: "element",
                key: (*key).into(),
                verdict: Verdict::Lost,
                detail: None,
                candidate: None,
                weight: *weight,
            })
            .collect(),
    }
}

fn cases(c: &CaseResult) -> Baseline {
    baseline(&[c], BaselineMode::Cases)
}

#[test]
fn a_cases_baseline_ratchets_each_cases_weight_and_item_set() {
    let two = cases(&lossy(100, &[("/a", 1), ("/b", 1)]));
    let line = two.failing.iter().next().map(|(_, i)| i.clone());
    assert!(
        line.as_deref().is_some_and(|l| l.starts_with("* 2 ")),
        "{line:?}"
    );
    // One more loss.
    let worse = diff(
        &two,
        &cases(&lossy(100, &[("/a", 1), ("/b", 1), ("/c", 1)])),
    );
    assert_eq!(worse.regressions.len(), 1, "{worse:?}");
    // One loss fixed: a fix, listed once.
    let better = diff(&two, &cases(&lossy(100, &[("/a", 1)])));
    assert!(better.regressions.is_empty(), "{better:?}");
    assert_eq!(better.fixed.len(), 1, "{better:?}");
    // A loss traded for a fix: the same n, another set of keys.
    let swapped = diff(&two, &cases(&lossy(100, &[("/a", 1), ("/c", 1)])));
    assert_eq!(swapped.regressions.len(), 1, "{swapped:?}");
    // Thirty changed children, then their parent lost whole: fewer items, more weight.
    let children: Vec<String> = (0..30).map(|n| format!("/p/c[{n}]")).collect();
    let changed: Vec<(&str, usize)> = children.iter().map(|k| (k.as_str(), 1)).collect();
    let collapsed = diff(
        &cases(&lossy(100, &changed)),
        &cases(&lossy(100, &[("/p", 31)])),
    );
    assert_eq!(collapsed.regressions.len(), 1, "{collapsed:?}");
    // A passing case that fails, and a failing one that passes.
    assert_eq!(diff(&cases(&lossy(100, &[])), &two).regressions.len(), 1);
    let fixed = diff(&two, &cases(&lossy(100, &[])));
    assert!(fixed.regressions.is_empty() && fixed.fixed.len() == 1);
}

#[test]
fn a_round_trip_that_compares_less_or_stops_running_regresses() {
    let two = cases(&lossy(100, &[("/a", 1), ("/b", 1)]));
    // The output became unreadable: one V8 item, one item compared.
    let unreadable = diff(&two, &cases(&lossy(1, &[("V8", 0)])));
    assert!(
        unreadable
            .regressions
            .iter()
            .any(|r| r.contains("checked fell from 100 to 1")),
        "{unreadable:?}"
    );
    // The case no longer runs.
    let errored = CaseResult {
        status: CaseStatus::EngineError,
        ..lossy(100, &[])
    };
    for mode in [BaselineMode::Cases, BaselineMode::Cells] {
        let before = baseline(&[&lossy(100, &[("/a", 1)])], mode);
        let after = baseline(&[&errored], mode);
        let delta = diff(&before, &after);
        assert_eq!(
            delta.regressions,
            vec!["rt/a.xlsx\t!EngineError".to_string()],
            "{mode:?}"
        );
    }
}

#[test]
fn the_kept_rate_weighs_lost_subtrees_and_skips_cases_that_did_not_run() {
    let mut case = lossy(200, &[("/x", 1), ("/tree", 9)]);
    case.items.push(Item {
        verdict: Verdict::Added,
        weight: 0,
        ..case.items[0].clone()
    });
    let errored = CaseResult {
        status: CaseStatus::Timeout,
        ..lossy(0, &[])
    };
    let rate = KeptRate::of(&[&case, &errored]);
    assert_eq!((rate.checked, rate.not_kept, rate.unscored), (200, 10, 1));
    assert!((rate.percent() - 95.0).abs() < 1e-9);
    assert!((KeptRate::default().percent() - 100.0).abs() < 1e-9);
}

#[test]
fn a_lost_subtree_and_the_truncated_rest_carry_their_weight() {
    let src = "<r><keep/><gone><a/><b/></gone></r>";
    let d = super::xmldiff::diff(src, "<r><keep/></r>").unwrap_or_default();
    let lost: Vec<usize> = d.differences.iter().map(|x| x.weight).collect();
    assert_eq!(lost, vec![3]);
    let many = (0..600).fold(String::new(), |mut acc, n| {
        let _ = write!(acc, "<i n=\"{n}\"/>");
        acc
    });
    let d = super::xmldiff::diff(&format!("<r>{many}</r>"), "<r/>").unwrap_or_default();
    assert_eq!(d.differences.len() + d.truncated, 600);
    // Each lost `<i n=".."/>` is an element and an attribute.
    assert_eq!(d.truncated_weight, 2 * d.truncated);
}

mod xml {
    use std::fmt::Write as _;

    use super::super::xmldiff::{candidate, diff};

    const NS: &str = r#"xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006""#;

    fn sheet(body: &str, extra: &str) -> String {
        format!("<worksheet {NS} {extra}>{body}</worksheet>")
    }

    fn found(a: &str, b: &str) -> Vec<(String, &'static str)> {
        let d = diff(a, b).expect("both parse");
        d.differences
            .into_iter()
            .map(|x| (x.path, x.verdict.label()))
            .collect()
    }

    #[test]
    fn identity_finds_nothing() {
        let a = sheet(
            r#"<sheetViews><sheetView workbookViewId="0"/></sheetViews><extLst><ext uri="{X}"><q/></ext></extLst>"#,
            "",
        );
        assert_eq!(found(&a, &a), vec![]);
    }

    #[test]
    fn prefixes_order_defaults_and_booleans_are_invariant() {
        let a = r#"<x:worksheet xmlns:x="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:p="http://schemas.microsoft.com/office/spreadsheetml/2009/9/ac" mc:Ignorable="p"><x:sheetViews><x:sheetView workbookViewId="0" tabSelected="1"/></x:sheetViews><x:mergeCells><x:mergeCell ref="A1:B1"/><x:mergeCell ref="C1:D1"/></x:mergeCells><x:extLst><x:ext uri="{A}"/><x:ext uri="{B}"/></x:extLst></x:worksheet>"#.to_string();
        let b = sheet(
            r#"<sheetViews><sheetView workbookViewId="0" tabSelected="true" showGridLines="1" zoomScale="100"/></sheetViews><mergeCells><mergeCell ref="C1:D1"/><mergeCell ref="A1:B1"/></mergeCells><extLst><ext uri="{B}"/><ext uri="{A}"/></extLst>"#,
            r#"xmlns:x14ac="http://schemas.microsoft.com/office/spreadsheetml/2009/9/ac" mc:Ignorable="x14ac""#,
        );
        assert_eq!(found(&a, &b), vec![]);
    }

    #[test]
    fn a_lost_ext_element_and_attribute_are_found() {
        let a = sheet(
            r#"<sheetPr codeName="S1"><tabColor rgb="FFFF0000"/></sheetPr><dataValidations count="1"><dataValidation sqref="A1"/></dataValidations><extLst><ext uri="{NEW}"><future/></ext></extLst>"#,
            "",
        );
        let b = sheet("<sheetPr/>", "");
        assert_eq!(
            found(&a, &b),
            vec![
                ("/worksheet/dataValidations[0]".to_string(), "lost"),
                ("/worksheet/extLst[0]".to_string(), "lost"),
                ("/worksheet/sheetPr[0]@codeName".to_string(), "lost"),
                ("/worksheet/sheetPr[0]/tabColor[0]".to_string(), "lost"),
            ]
        );
        let lost_ext = sheet(
            r#"<extLst><ext uri="{NEW}"/><ext uri="{OLD}"/></extLst>"#,
            "",
        );
        let kept = sheet(r#"<extLst><ext uri="{OLD}"/></extLst>"#, "");
        assert_eq!(
            found(&lost_ext, &kept),
            vec![("/worksheet/extLst[0]/ext[{NEW}]".to_string(), "lost")]
        );
    }

    #[test]
    fn booleans_only_where_the_schema_says_and_cols_keep_their_order() {
        // `name` is a string: "1" and "true" differ. `tabSelected` is a boolean: they do not.
        let a = sheet(
            r#"<sheetViews><sheetView workbookViewId="0" tabSelected="1"/></sheetViews><customSheetViews><customSheetView name="1"/></customSheetViews>"#,
            "",
        );
        let b = sheet(
            r#"<sheetViews><sheetView workbookViewId="0" tabSelected="true"/></sheetViews><customSheetViews><customSheetView name="true"/></customSheetViews>"#,
            "",
        );
        assert_eq!(
            found(&a, &b),
            vec![(
                "/worksheet/customSheetViews[0]/customSheetView[0]@name".to_string(),
                "changed"
            )]
        );
        // `<col>` order is not proven irrelevant (SPEC-0033 D4.2), so a reorder is seen.
        let a = sheet(
            r#"<cols><col min="1" max="1"/><col min="2" max="2"/></cols>"#,
            "",
        );
        let b = sheet(
            r#"<cols><col min="2" max="2"/><col min="1" max="1"/></cols>"#,
            "",
        );
        assert_eq!(found(&a, &b).len(), 4);
    }

    #[test]
    fn hostile_names_and_wide_trees_stay_bounded() {
        let long = "n".repeat(300);
        let bad = format!("<{long}/>");
        assert!(
            diff(&bad, &bad).is_err(),
            "an over-long name makes the part unparsable"
        );
        let dup = r#"<worksheet a="1" a="2"/>"#;
        assert!(
            diff(dup, dup).is_err(),
            "a repeated attribute makes the part unparsable"
        );
        // Deep, long-named ancestors over many children: linear, not quadratic.
        let name = "n".repeat(200);
        let depth = 16;
        let mut xml = String::new();
        for _ in 0..depth {
            let _ = write!(xml, "<{name}>");
        }
        xml.push_str(&"<a/>".repeat(200_000));
        for _ in 0..depth {
            let _ = write!(xml, "</{name}>");
        }
        let start = std::time::Instant::now();
        let d = diff(&xml, &xml).expect("parses");
        assert!(d.differences.is_empty());
        assert!(start.elapsed().as_secs() < 20, "took {:?}", start.elapsed());
    }

    #[test]
    fn cell_data_is_left_to_the_semantic_model() {
        let a = sheet(
            r#"<sheetData><row r="1"><c r="A1"><v>1</v></c></row></sheetData>"#,
            "",
        );
        let b = sheet(
            r#"<sheetData><row r="1"><c r="A1"><v>2</v></c></row></sheetData>"#,
            "",
        );
        assert_eq!(found(&a, &b), vec![]);
        let d = diff(&a, &b).expect("parse");
        assert!(d.checked >= 4, "the skipped cells still count as checked");
    }

    #[test]
    fn candidates_are_tagged_by_key() {
        let cases = [
            ("/officeDocument[0]/calcChain[0]", Some("N1")),
            ("/officeDocument[0]:/workbook/calcPr[0]@calcId", Some("N4")),
            (
                "/officeDocument[0]/worksheet[S]:/worksheet/sheetData[0]/row[0]@spans",
                Some("N8"),
            ),
            (
                "/core-properties[0]:/coreProperties/dcterms:modified[0]",
                Some("N2"),
            ),
            (
                "/officeDocument[0]/styles[0]:/styleSheet/fonts[0]@count",
                Some("N10"),
            ),
            (
                "/officeDocument[0]/worksheet[S]:/worksheet/sheetPr[0]@codeName",
                None,
            ),
        ];
        for (key, want) in cases {
            assert_eq!(candidate(key), want, "{key}");
        }
    }
}

#[test]
fn every_fixture_part_diffs_empty_against_itself() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../eval/suites/fixtures/cases");
    let mut parts = 0;
    for entry in std::fs::read_dir(&root).expect("fixtures") {
        let path = entry.expect("entry").path();
        if path.extension().is_none_or(|e| e != "xlsx") {
            continue;
        }
        let pkg = Package::read(&std::fs::read(&path).expect("read")).expect("package");
        for (name, xml) in &pkg.xml {
            let d = super::xmldiff::diff(xml, xml).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(
                d.differences.is_empty(),
                "{}: {name}: {:?}",
                path.display(),
                d.differences
            );
            parts += 1;
        }
    }
    assert!(parts > 100, "{parts} parts");
}

mod semantic_items {
    use std::io::{Cursor, Read as _, Write as _};

    use super::super::semantic::{
        compare, load_model, model_facts, stored_facts, validate_indexes, Facts,
    };
    use crate::xlsx_snapshot::read_snapshot;

    /// A1 "hello", A2 "world", A3 `=1+1`, as the export writes them.
    fn base() -> Vec<u8> {
        let mut m = rustsheet_engine::Model::new_empty("w", "en", "UTC", "en").expect("model");
        m.set_user_input(0, 1, 1, "hello".into()).expect("A1");
        m.set_user_input(0, 2, 1, "world".into()).expect("A2");
        m.set_user_input(0, 3, 1, "=1+1".into()).expect("A3");
        m.evaluate();
        rustsheet_xlsx::export::save_xlsx_to_writer(&m, Cursor::new(Vec::new()))
            .expect("save")
            .into_inner()
    }

    /// `bytes` with each `(part, from, to)` text replacement applied (each must match).
    fn edit(bytes: &[u8], edits: &[(&str, &str, &str)]) -> Vec<u8> {
        let mut src = zip::ZipArchive::new(Cursor::new(bytes)).expect("zip");
        let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for i in 0..src.len() {
            let mut f = src.by_index(i).expect("entry");
            let name = f.name().expect("entry name").to_string();
            if name.ends_with('/') {
                continue;
            }
            let mut text = String::new();
            f.read_to_string(&mut text).expect("utf-8");
            for (part, from, to) in edits.iter().filter(|e| e.0 == name) {
                assert!(text.contains(from), "{part}: {from}");
                text = text.replacen(from, to, 1);
            }
            out.start_file(name, zip::write::SimpleFileOptions::default())
                .expect("start");
            out.write_all(text.as_bytes()).expect("write");
        }
        out.finish().expect("finish").into_inner()
    }

    fn facts(bytes: &[u8]) -> Facts {
        let mut f = model_facts(&load_model(bytes, "w").expect("loads"));
        f.extend(stored_facts(&read_snapshot(bytes).expect("snapshot")));
        f
    }

    const SST: &str = "xl/sharedStrings.xml";
    const SHEET: &str = "xl/worksheets/sheet1.xml";
    const STYLES: &str = "xl/styles.xml";
    const XF: &str = r#"<xf xfId="0" borderId="0" fillId="0" fontId="0" numFmtId="0"/>"#;

    #[test]
    fn shuffled_strings_and_renumbered_styles_are_kept() {
        let a = base();
        let b = edit(
            &a,
            &[
                (
                    SST,
                    "<si><t>hello</t></si><si><t>world</t></si>",
                    "<si><t>world</t></si><si><t>hello</t></si>",
                ),
                (
                    SHEET,
                    r#"<c r="A1" t="s"><v>0</v>"#,
                    r#"<c r="A1" t="s"><v>1</v>"#,
                ),
                (
                    SHEET,
                    r#"<c r="A2" t="s"><v>1</v>"#,
                    r#"<c r="A2" t="s"><v>0</v>"#,
                ),
                (
                    STYLES,
                    &format!(r#"<cellXfs count="1">{XF}"#),
                    &format!(r#"<cellXfs count="2">{XF}{XF}"#),
                ),
                (SHEET, r#"<c r="A3">"#, r#"<c r="A3" s="1">"#),
            ],
        );
        let (diffs, n) = compare(&facts(&a), &facts(&b));
        assert!(n > 0);
        assert_eq!(diffs, vec![]);
    }

    #[test]
    fn a_changed_value_formula_and_style_are_found() {
        let a = base();
        let date = r#"<xf xfId="0" borderId="0" fillId="0" fontId="0" numFmtId="14" applyNumberFormat="1"/>"#;
        let b = edit(
            &a,
            &[
                (SST, "<t>world</t>", "<t>wor</t>"),
                (SHEET, "<f>1+1</f>", "<f>1+2</f>"),
                (
                    STYLES,
                    &format!(r#"<cellXfs count="1">{XF}"#),
                    &format!(r#"<cellXfs count="2">{XF}{date}"#),
                ),
                (SHEET, r#"<c r="A1" t="s">"#, r#"<c r="A1" s="1" t="s">"#),
            ],
        );
        let (diffs, _) = compare(&facts(&a), &facts(&b));
        let keys: Vec<&str> = diffs.iter().map(|d| d.0.as_str()).collect();
        assert_eq!(
            keys,
            vec![
                "cell Sheet1!A1 style",
                "cell Sheet1!A2 input",
                "cell Sheet1!A3 formula"
            ]
        );
    }

    #[test]
    fn a_needless_xlfn_prefix_is_no_difference_and_a_needed_one_is() {
        // Also proves the static pattern compiles (its initializer's `unreachable!` cannot fire).
        let a = base();
        let prefixed = edit(&a, &[(SHEET, "<f>1+1</f>", "<f>_xlfn.SUM(1,1)</f>")]);
        let bare = edit(&a, &[(SHEET, "<f>1+1</f>", "<f>SUM(1,1)</f>")]);
        assert_eq!(compare(&facts(&prefixed), &facts(&bare)).0, vec![]);
        let needed = edit(&a, &[(SHEET, "<f>1+1</f>", "<f>_xlfn.XLOOKUP(1,1,1)</f>")]);
        let dropped = edit(&a, &[(SHEET, "<f>1+1</f>", "<f>XLOOKUP(1,1,1)</f>")]);
        assert_eq!(compare(&facts(&needed), &facts(&dropped)).0.len(), 1);
    }

    #[test]
    fn v7_an_out_of_range_shared_string_index() {
        // An out-of-range style index is refused by the importer itself, so V8 already fails it.
        let bad_style = edit(
            &base(),
            &[(SHEET, r#"<c r="A1" t="s">"#, r#"<c r="A1" s="9" t="s">"#)],
        );
        assert!(load_model(&bad_style, "w").is_err());
        let bad_string = edit(
            &base(),
            &[(
                SHEET,
                r#"<c r="A1" t="s"><v>0</v>"#,
                r#"<c r="A1" t="s"><v>9</v>"#,
            )],
        );
        match load_model(&bad_string, "w") {
            Ok(model) => assert_eq!(validate_indexes(&model).len(), 1, "V7 names the string"),
            Err(e) => panic!("{e}"),
        }
    }
}

#[test]
fn flags_map_to_modes_and_regressions_are_refused_by_default() {
    let flags = |check, update, allow_regressions| Flags {
        check,
        update,
        allow_regressions,
    };
    assert_eq!(flags(false, false, false).mode(), Mode::Report);
    assert_eq!(flags(true, false, false).mode(), Mode::Check);
    assert_eq!(
        flags(false, true, false).mode(),
        Mode::Update {
            allow_regressions: false
        }
    );
    assert_eq!(
        flags(false, true, true).mode(),
        Mode::Update {
            allow_regressions: true
        }
    );
}

mod r1_rule {
    use super::super::r1::{violations, Delta};
    use super::super::{Item, Verdict};

    fn item(kind: &'static str, key: &str, verdict: Verdict) -> Item {
        Item {
            scenario: "R0",
            kind,
            key: key.into(),
            verdict,
            detail: None,
            candidate: None,
            weight: super::super::xmldiff::unit_weight(verdict),
        }
    }

    #[test]
    fn only_what_the_edits_explain_may_change() {
        let delta: Delta = ["Sheet1!A1".to_string(), "Sheet1!B2".to_string()].into();
        // Already lost in R0: not the edit's doing.
        let plain = vec![item(
            "element",
            "/officeDocument[0]/theme[0]:/theme/x[0]",
            Verdict::Lost,
        )];
        let edited = vec![
            plain[0].clone(),
            // The edited cell and a cell its value changed.
            item("semantic", "cell Sheet1!A1 input", Verdict::Changed),
            item("semantic", "value Sheet1!B2", Verdict::Changed),
            // Derived counts and range (N9, N10), and an appended cell format.
            item(
                "attribute",
                "/officeDocument[0]/styles[0]:/styleSheet/cellXfs[0]@count",
                Verdict::Changed,
            ),
            item(
                "attribute",
                "/officeDocument[0]/worksheet[Sheet1][0]:/worksheet/dimension[0]@ref",
                Verdict::Changed,
            ),
            item(
                "element",
                "/officeDocument[0]/styles[0]:/styleSheet/cellXfs[0]/xf[9]",
                Verdict::Added,
            ),
            // An unrelated loss the export made because of the edit: a violation.
            item(
                "element",
                "/officeDocument[0]/worksheet[Sheet1][0]:/worksheet/extLst[0]",
                Verdict::Lost,
            ),
            item("semantic", "value Sheet1!C3", Verdict::Changed),
            // A style record lost, not added: a violation too.
            item(
                "element",
                "/officeDocument[0]/styles[0]:/styleSheet/cellXfs[0]/xf[2]",
                Verdict::Lost,
            ),
        ];
        let found = violations("R1/E1", &plain, edited, &delta);
        let keys: Vec<&str> = found.iter().map(|i| i.key.as_str()).collect();
        assert_eq!(
            keys,
            [
                "/officeDocument[0]/worksheet[Sheet1][0]:/worksheet/extLst[0]",
                "value Sheet1!C3",
                "/officeDocument[0]/styles[0]:/styleSheet/cellXfs[0]/xf[2]",
            ]
        );
        assert!(found.iter().all(|i| i.scenario == "R1/E1"));
    }
}

mod exceptions_self_test {
    use std::collections::BTreeMap;

    use super::super::exceptions::{apply, Exception};
    use super::super::{CaseResult, Item, Verdict};
    use crate::runner::CaseStatus;

    fn case(keys: &[&str]) -> CaseResult {
        CaseResult {
            id: "rt/a.xlsx".into(),
            suite: "rt".into(),
            status: CaseStatus::Fail,
            message: None,
            millis: 0,
            checked: 10,
            scenarios: BTreeMap::from([("R0", 10)]),
            items: keys
                .iter()
                .map(|k| Item {
                    scenario: "R0",
                    kind: "element",
                    key: (*k).into(),
                    verdict: Verdict::Lost,
                    detail: None,
                    candidate: None,
                    weight: 1,
                })
                .collect(),
        }
    }

    fn exception(item: &str) -> Exception {
        Exception {
            case: "rt/a.xlsx".into(),
            item: item.into(),
            owner: "T-4514".into(),
        }
    }

    #[test]
    fn an_exception_excuses_its_item_and_a_stale_one_is_reported() {
        let mut r = case(&["/x"]);
        let used = [exception("R0 element /x lost")];
        assert!(apply(&mut [&mut r], &used).is_empty());
        assert!(r.items.is_empty());
        assert_eq!(r.status, CaseStatus::Pass);
        // The item is gone (fixed): the exception now excuses nothing and is stale.
        let mut fixed = case(&[]);
        let stale = apply(&mut [&mut fixed], &used);
        assert_eq!(stale.len(), 1);
    }
}
