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
    let Ok((items, _)) = r0(&bytes, Some("circular-keeps-cached-values.xlsx")) else {
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
    let delta = baseline::diff(&baseline(&[&before]), &baseline(&[&after]));
    assert_eq!(
        delta.regressions,
        vec!["rt/a.xlsx\tR0 part /b[0] lost".to_string()]
    );
    let fixed = baseline::diff(&baseline(&[&after]), &baseline(&[&before]));
    assert_eq!(fixed.fixed.len(), 1);
}

mod xml {
    use super::super::xmldiff::{candidate, diff};

    const NS: &str = r#"xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006""#;

    fn sheet(body: &str, extra: &str) -> String {
        format!("<worksheet {NS} {extra}>{body}</worksheet>")
    }

    fn found(a: &str, b: &str) -> Vec<(String, &'static str)> {
        let d = diff(a, b).expect("both parse");
        d.differences
            .into_iter()
            .map(|x| (x.path, x.verdict))
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
        let a = format!(
            r#"<x:worksheet xmlns:x="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:p="http://schemas.microsoft.com/office/spreadsheetml/2009/9/ac" mc:Ignorable="p"><x:sheetViews><x:sheetView workbookViewId="0" tabSelected="1"/></x:sheetViews><x:mergeCells><x:mergeCell ref="A1:B1"/><x:mergeCell ref="C1:D1"/></x:mergeCells><x:extLst><x:ext uri="{{A}}"/><x:ext uri="{{B}}"/></x:extLst></x:worksheet>"#
        );
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
        let b = sheet(r#"<sheetPr/>"#, "");
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
        assert_eq!(candidate("/officeDocument[0]/calcChain[0]"), Some("N1"));
        assert_eq!(
            candidate("/officeDocument[0]:/workbook/calcPr[0]@calcId"),
            Some("N4")
        );
        assert_eq!(
            candidate("/officeDocument[0]/worksheet[S]:/worksheet/sheetData[0]/row[0]@spans"),
            Some("N8")
        );
        assert_eq!(
            candidate("/core-properties[0]:/coreProperties/dcterms:modified[0]"),
            Some("N2")
        );
        assert_eq!(
            candidate("/officeDocument[0]/styles[0]:/styleSheet/fonts[0]@count"),
            Some("N10")
        );
        assert_eq!(
            candidate("/officeDocument[0]/worksheet[S]:/worksheet/sheetPr[0]@codeName"),
            None
        );
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
