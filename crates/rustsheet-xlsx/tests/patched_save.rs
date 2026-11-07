#![allow(clippy::unwrap_used, clippy::expect_used, reason = "tests")]

//! SPEC-0034 D3, T-4560: a workbook whose cells changed but whose structure did not is saved by
//! patching the source. Only the changed cells' sheet and, for new strings, `sharedStrings.xml`
//! change; every other part is byte-identical. Whatever the patcher will not do exactly
//! regenerates.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read, Write};

use rustsheet_engine::expressions::types::Area;
use rustsheet_engine::Model;
use rustsheet_xlsx::export::{save_preserving, save_xlsx_to_writer, SaveReport, SaveTier};
use rustsheet_xlsx::import::{load_from_xlsx_bytes, ImportLimits};

const SHEET: &str = "xl/worksheets/sheet1.xml";
const STYLES: &str = "xl/styles.xml";
const STRINGS: &str = "xl/sharedStrings.xml";
const RELS: &str = "xl/_rels/workbook.xml.rels";
const TYPES: &str = "[Content_Types].xml";

/// A1 = 2, A2 = `=A1*3`, A3 = "x", A4 = 7: saved by the export, with every cached value right.
fn source() -> Vec<u8> {
    let mut m = Model::new_empty("m", "en", "UTC", "en").unwrap();
    m.set_user_input(0, 1, 1, "2".into()).unwrap();
    m.set_user_input(0, 2, 1, "=A1*3".into()).unwrap();
    m.set_user_input(0, 3, 1, "x".into()).unwrap();
    m.set_user_input(0, 4, 1, "7".into()).unwrap();
    m.evaluate();
    save_xlsx_to_writer(&m, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner()
}

fn load(bytes: &[u8]) -> Model {
    let wb = load_from_xlsx_bytes(bytes, "m", "en", "UTC").unwrap();
    Model::from_workbook(wb, "en").unwrap()
}

/// Loads `bytes`, applies `edit`, recalculates and saves preserving.
fn save_after(bytes: &[u8], edit: impl FnOnce(&mut Model)) -> (Vec<u8>, SaveTier) {
    let pristine = load(bytes);
    let mut model = load(bytes);
    edit(&mut model);
    model.evaluate();
    let (out, report) = save_preserving(
        &model,
        &pristine,
        bytes,
        &ImportLimits::default(),
        Cursor::new(Vec::new()),
    )
    .unwrap();
    (out.into_inner(), report.tier)
}

/// Every entry of a package, decompressed.
fn entries(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut out = BTreeMap::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).unwrap();
        let mut data = Vec::new();
        f.read_to_end(&mut data).unwrap();
        out.insert(f.name().unwrap().to_string(), data);
    }
    out
}

/// The names of the entries that differ between two packages, which must hold the same names.
fn changed(a: &[u8], b: &[u8]) -> BTreeSet<String> {
    let (a, b) = (entries(a), entries(b));
    assert_eq!(
        a.keys().collect::<Vec<_>>(),
        b.keys().collect::<Vec<_>>(),
        "same parts"
    );
    a.iter()
        .filter(|(k, v)| b.get(*k) != Some(v))
        .map(|(k, _)| k.clone())
        .collect()
}

fn part(bytes: &[u8], name: &str) -> String {
    String::from_utf8(entries(bytes).remove(name).unwrap()).unwrap()
}

fn value(bytes: &[u8], cell: (i32, i32)) -> String {
    load(bytes)
        .get_formatted_cell_value(0, cell.0, cell.1)
        .unwrap()
}

/// `bytes` with one more entry.
fn with_entry(bytes: &[u8], name: &str, content: &str) -> Vec<u8> {
    let mut all = entries(bytes);
    all.insert(name.to_string(), content.as_bytes().to_vec());
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (entry, data) in all {
        out.start_file(entry, zip::write::SimpleFileOptions::default())
            .unwrap();
        out.write_all(&data).unwrap();
    }
    out.finish().unwrap().into_inner()
}

/// `bytes` with one text replacement in `name`.
fn rezip(bytes: &[u8], name: &str, from: &str, to: &str) -> Vec<u8> {
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (entry, data) in entries(bytes) {
        let data = if entry == name {
            let text = String::from_utf8(data).unwrap();
            assert!(text.contains(from), "{from} in {text}");
            text.replacen(from, to, 1).into_bytes()
        } else {
            data
        };
        out.start_file(entry, zip::write::SimpleFileOptions::default())
            .unwrap();
        out.write_all(&data).unwrap();
    }
    out.finish().unwrap().into_inner()
}

fn assert_patched(tier: &SaveTier) {
    assert!(matches!(tier, SaveTier::Patched { .. }), "{tier:?}");
}

/// What an edit is, the edit, a cell it changes and that cell's value after it.
type Edit = (&'static str, fn(&mut Model), (i32, i32), &'static str);

#[test]
fn each_cell_edit_patches_only_its_sheet() {
    let src = source();
    let edits: [Edit; 4] = [
        (
            "a changed constant",
            |m| m.set_user_input(0, 1, 1, "5".into()).unwrap(),
            (2, 1),
            "15",
        ),
        (
            "a changed formula",
            |m| m.set_user_input(0, 2, 1, "=A1*4".into()).unwrap(),
            (2, 1),
            "8",
        ),
        (
            "a new cell in an existing row",
            |m| m.set_user_input(0, 1, 2, "=A1+1".into()).unwrap(),
            (1, 2),
            "3",
        ),
        (
            "a cleared cell",
            |m| {
                let a4 = Area {
                    sheet: 0,
                    row: 4,
                    column: 1,
                    width: 1,
                    height: 1,
                };
                m.range_clear_contents(&a4).unwrap();
            },
            (4, 1),
            "",
        ),
    ];
    for (what, edit, cell, expected) in edits {
        let (out, tier) = save_after(&src, edit);
        assert_patched(&tier);
        assert_eq!(
            changed(&src, &out),
            BTreeSet::from([SHEET.into()]),
            "{what}"
        );
        assert_eq!(value(&out, cell), expected, "{what}");
        assert_eq!(value(&out, (3, 1)), "x", "{what}");
    }
}

#[test]
fn a_new_row_with_a_new_string_appends_it_to_the_shared_strings() {
    let src = source();
    let (out, tier) = save_after(&src, |m| {
        m.set_user_input(0, 9, 3, "new <text>".into()).unwrap();
    });
    assert_patched(&tier);
    assert_eq!(
        changed(&src, &out),
        BTreeSet::from([SHEET.into(), STRINGS.into()])
    );
    let sst = part(&out, STRINGS);
    assert!(sst.contains(r#"count="2""#), "{sst}");
    assert!(sst.contains(r#"uniqueCount="2""#), "{sst}");
    assert!(
        sst.ends_with("<si><t>new &lt;text&gt;</t></si></sst>"),
        "{sst}"
    );
    assert_eq!(value(&out, (9, 3)), "new <text>");
    let sheet = part(&out, SHEET);
    assert!(sheet.contains(r#"<row r="9""#));
    // The dimension grows to cover C9.
    assert!(sheet.contains(r#"<dimension ref="A1:C9"/>"#), "{sheet}");
}

#[test]
fn cells_into_an_empty_row_and_a_new_row_before_it_are_written_once_in_order() {
    let src = rezip(
        &source(),
        SHEET,
        "</sheetData>",
        r#"<row r="6" ht="30" customHeight="1"/></sheetData>"#,
    );
    let (out, tier) = save_after(&src, |m| {
        m.set_user_input(0, 5, 1, "1".into()).unwrap();
        m.set_user_input(0, 6, 2, "2".into()).unwrap();
        m.set_user_input(0, 6, 3, "3".into()).unwrap();
    });
    assert_patched(&tier);
    let sheet = part(&out, SHEET);
    assert_eq!(sheet.matches(r#"<row r="6""#).count(), 1, "{sheet}");
    let (five, six) = (sheet.find(r#"<row r="5""#), sheet.find(r#"<row r="6""#));
    assert!(five.is_some() && five < six, "{sheet}");
    assert!(
        sheet.contains(r#"<row r="6" ht="30" customHeight="1"><c r="B6""#),
        "{sheet}"
    );
    assert_eq!(
        (value(&out, (6, 2)), value(&out, (6, 3))),
        ("2".into(), "3".into())
    );
}

#[test]
fn rows_into_an_empty_sheet_data_are_written_in_one_element() {
    let mut m = Model::new_empty("m", "en", "UTC", "en").unwrap();
    m.evaluate();
    let src = save_xlsx_to_writer(&m, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner();
    let (out, tier) = save_after(&src, |m| {
        m.set_user_input(0, 1, 1, "1".into()).unwrap();
        m.set_user_input(0, 3, 1, "3".into()).unwrap();
    });
    assert!(!matches!(tier, SaveTier::Regenerated { .. }), "{tier:?}");
    let sheet = part(&out, SHEET);
    assert_eq!(sheet.matches("<sheetData").count(), 1, "{sheet}");
    assert_eq!(
        (value(&out, (1, 1)), value(&out, (3, 1))),
        ("1".into(), "3".into())
    );
}

#[test]
fn inline_strings_the_part_lacks_make_a_new_string_regenerate() {
    // The importer interns B3's inline string after the part's `<si>`s.
    let src = rezip(
        &source(),
        SHEET,
        "</row></sheetData>",
        r#"</row><row r="8"><c r="B8" t="inlineStr"><is><t>inline</t></is></c></row></sheetData>"#,
    );
    let (out, tier) = save_after(&src, |m| {
        m.set_user_input(0, 9, 3, "fresh".into()).unwrap();
    });
    assert!(matches!(tier, SaveTier::Regenerated { .. }), "{tier:?}");
    assert_eq!(
        (value(&out, (9, 3)), value(&out, (8, 2))),
        ("fresh".into(), "inline".into())
    );
    // A number needs no string, so it still patches.
    let (_, tier) = save_after(&src, |m| m.set_user_input(0, 1, 1, "5".into()).unwrap());
    assert_patched(&tier);
}

#[test]
fn a_calc_chain_is_kept_while_the_formula_cells_hold_and_dropped_with_its_links_when_they_change() {
    let chain = r#"<calcChain xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><c r="A2" i="1"/></calcChain>"#;
    let src = with_entry(&source(), "xl/calcChain.xml", chain);
    let src = rezip(
        &src,
        RELS,
        "</Relationships>",
        r#"<Relationship Id="rIdCC" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/calcChain" Target="calcChain.xml"/></Relationships>"#,
    );
    let src = rezip(
        &src,
        TYPES,
        "</Types>",
        r#"<Override PartName="/xl/calcChain.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.calcChain+xml"/></Types>"#,
    );
    let (out, tier) = save_after(&src, |m| m.set_user_input(0, 1, 1, "5".into()).unwrap());
    assert_patched(&tier);
    assert_eq!(part(&out, "xl/calcChain.xml"), chain);
    // A2's formula replaced by a constant: the chain would name a cell with no formula, so it goes
    // with its relationship and override (SPEC-0034 E2), and nothing else changes.
    let (out, tier) = save_after(&src, |m| m.set_user_input(0, 2, 1, "9".into()).unwrap());
    assert_patched(&tier);
    assert!(!entries(&out).contains_key("xl/calcChain.xml"));
    assert!(!part(&out, RELS).contains("calcChain"));
    assert!(!part(&out, TYPES).contains("calcChain"));
    assert_eq!(value(&out, (2, 1)), "9");
}

/// `source()` without `xl/metadata.xml`, its relationship and its override, as a workbook from a
/// writer that never needed one.
fn source_without_metadata() -> Vec<u8> {
    let rels = part(&source(), RELS);
    let link = rels
        .split("<Relationship ")
        .find(|r| r.contains("metadata.xml"))
        .map(|r| format!("<Relationship {}", &r[..=r.find("/>").unwrap() + 1]))
        .unwrap();
    let src = rezip(&source(), RELS, &link, "");
    let src = rezip(
        &src,
        TYPES,
        r#"<Override PartName="/xl/metadata.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheetMetadata+xml"/>"#,
        "",
    );
    let mut all = entries(&src);
    all.remove("xl/metadata.xml");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (entry, data) in all {
        out.start_file(entry, zip::write::SimpleFileOptions::default())
            .unwrap();
        out.write_all(&data).unwrap();
    }
    out.finish().unwrap().into_inner()
}

#[test]
fn a_source_without_metadata_takes_the_exports_for_rich_errors_and_dynamic_arrays() {
    let src = source_without_metadata();
    assert!(!entries(&src).contains_key("xl/metadata.xml"));
    // A spill blocked by A7 (#SPILL!, a rich error: `vm`), and one that spills (`cm`).
    let (out, tier) = save_after(&src, |m| {
        m.set_user_input(0, 7, 1, "block".into()).unwrap();
        m.set_user_input(0, 6, 1, "=SEQUENCE(2)".into()).unwrap();
        m.set_user_input(0, 9, 2, "=SEQUENCE(2)".into()).unwrap();
    });
    assert_patched(&tier);
    let all = entries(&out);
    assert!(all.contains_key("xl/metadata.xml"), "{:?}", all.keys());
    assert!(
        all.keys().any(|k| k.starts_with("xl/richData/")),
        "{:?}",
        all.keys()
    );
    let rels = part(&out, RELS);
    assert!(rels.contains(r#"Target="metadata.xml""#), "{rels}");
    assert!(part(&out, TYPES).contains(r#"PartName="/xl/metadata.xml""#));
    assert_eq!(value(&out, (6, 1)), "#SPILL!");
    assert_eq!(
        (value(&out, (9, 2)), value(&out, (10, 2))),
        ("1".into(), "2".into())
    );
    // An untouched cell whose `vm` has no metadata loads as a plain #VALUE!. Under the export's
    // metadata `vm="1"` would name #SPILL!, so that source regenerates instead.
    let dangling = rezip(
        &src,
        SHEET,
        "</sheetData>",
        r#"<row r="12"><c r="A12" t="e" vm="1"><v>#VALUE!</v></c></row></sheetData>"#,
    );
    let (out, tier) = save_after(&dangling, |m| {
        m.set_user_input(0, 7, 1, "block".into()).unwrap();
        m.set_user_input(0, 6, 1, "=SEQUENCE(2)".into()).unwrap();
    });
    assert!(matches!(tier, SaveTier::Regenerated { .. }), "{tier:?}");
    assert_eq!(value(&out, (12, 1)), "#VALUE!");
    // Without any rich value or dynamic array, no part is added.
    let (out, tier) = save_after(&src, |m| m.set_user_input(0, 1, 1, "5".into()).unwrap());
    assert_patched(&tier);
    assert!(!entries(&out).contains_key("xl/metadata.xml"));
}

#[test]
fn what_the_patcher_will_not_do_exactly_regenerates() {
    let src = source();
    let regenerated = |bytes: &[u8], edit: fn(&mut Model), what: &str| {
        let (out, tier) = save_after(bytes, edit);
        assert!(
            matches!(tier, SaveTier::Regenerated { .. }),
            "{what}: {tier:?}"
        );
        assert_eq!(value(&out, (2, 1)), "15", "{what}");
    };
    let edit_a1: fn(&mut Model) = |m| m.set_user_input(0, 1, 1, "5".into()).unwrap();
    // A structural change: a new sheet.
    regenerated(
        &src,
        |m| {
            m.add_sheet("Two").unwrap();
            m.set_user_input(0, 1, 1, "5".into()).unwrap();
        },
        "a new sheet",
    );
    // Rows the source stores out of order.
    let sheet = part(&src, SHEET);
    let (row1, row2) = (row(&sheet, 1), row(&sheet, 2));
    let swapped = rezip(
        &src,
        SHEET,
        &format!("{row1}{row2}"),
        &format!("{row2}{row1}"),
    );
    regenerated(&swapped, edit_a1, "rows out of order");
    // A cell the source stores without `r`.
    let bare = rezip(&src, SHEET, r#"<c r="A4""#, "<c");
    regenerated(&bare, edit_a1, "a cell without r");
}

/// The `<row r="n" ...>...</row>` element of `sheet`.
fn row(sheet: &str, n: i32) -> String {
    let start = sheet.find(&format!(r#"<row r="{n}""#)).unwrap();
    let end = start + sheet[start..].find("</row>").unwrap() + "</row>".len();
    sheet[start..end].to_string()
}

#[test]
fn a_changed_formula_in_a_shared_group_regenerates() {
    // A2 and A5 as one shared-formula group, as Excel stores a filled-down formula.
    let src = source();
    let shared = rezip(
        &rezip(
            &src,
            SHEET,
            "<f>A1*3</f>",
            r#"<f t="shared" ref="A2:A2" si="0">A1*3</f>"#,
        ),
        SHEET,
        "</sheetData>",
        r#"<row r="5"><c r="A5"><f t="shared" ref="A5:A5" si="1">A4*3</f><v>21</v></c></row></sheetData>"#,
    );
    let (out, tier) = save_after(&shared, |m| {
        m.set_user_input(0, 2, 1, "=A1*4".into()).unwrap();
    });
    assert!(matches!(tier, SaveTier::Regenerated { .. }), "{tier:?}");
    assert_eq!(value(&out, (2, 1)), "8");
    // A recalculated value in the group is a value-only patch that keeps the `<f>`.
    let (out, tier) = save_after(&shared, |m| m.set_user_input(0, 1, 1, "5".into()).unwrap());
    assert_patched(&tier);
    assert!(part(&out, SHEET).contains(r#"<f t="shared" ref="A2:A2" si="0">A1*3</f><v>15</v>"#));
}

/// The `count` of the `<name>` section of a styles part, and how many `<record` elements it holds.
fn section(styles: &str, name: &str, record: &str) -> (usize, usize) {
    let start = styles.find(&format!("<{name} ")).unwrap();
    let end = styles[start..].find(&format!("</{name}>")).unwrap() + start;
    let body = &styles[start..end];
    let count = body
        .split("count=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    let records =
        body.matches(&format!("<{record}>")).count() + body.matches(&format!("<{record} ")).count();
    (count.parse().unwrap(), records)
}

#[test]
fn a_new_number_format_and_a_new_font_are_appended_to_the_source_styles() {
    let src = source();
    let before = part(&src, STYLES);
    let (xfs, _) = section(&before, "cellXfs", "xf");
    let (fonts, _) = section(&before, "fonts", "font");
    let (out, tier) = save_after(&src, |m| {
        m.set_user_input(0, 4, 1, "0.5".into()).unwrap();
        let mut style = m.get_style_for_cell(0, 4, 1).unwrap();
        style.num_fmt = "0.000".into();
        m.set_cell_style(0, 4, 1, &style).unwrap();
        let mut bold = m.get_style_for_cell(0, 1, 1).unwrap();
        bold.font.b = true;
        m.set_cell_style(0, 1, 1, &bold).unwrap();
    });
    assert_patched(&tier);
    assert_eq!(
        changed(&src, &out),
        BTreeSet::from([SHEET.into(), STYLES.into()])
    );
    let after = part(&out, STYLES);
    // The source's records keep their indexes, so every `s=` in the package still names its
    // style.
    assert_eq!(
        section(&after, "cellXfs", "xf"),
        (xfs + 2, xfs + 2),
        "{after}"
    );
    assert_eq!(
        section(&after, "fonts", "font"),
        (fonts + 1, fonts + 1),
        "{after}"
    );
    assert!(after.contains(r#"formatCode="0.000""#), "{after}");
    // Each section's source records are its first records, byte for byte.
    for name in ["fonts", "cellXfs"] {
        let body = |s: &str| {
            let open = s.find(&format!("<{name} ")).unwrap();
            let open = s[open..].find('>').unwrap() + open + 1;
            let close = s[open..].find(&format!("</{name}>")).unwrap() + open;
            s[open..close].to_string()
        };
        assert!(body(&after).starts_with(&body(&before)), "{name}: {after}");
    }
    assert_eq!(value(&out, (4, 1)), "0.500");
    let reloaded = load(&out);
    assert!(reloaded.get_style_for_cell(0, 1, 1).unwrap().font.b);
    assert_eq!(value(&out, (3, 1)), "x");
}

/// Saves `bytes` after `edit` as `save_after` does, returning the save report (T-4562).
fn report_after(bytes: &[u8], edit: impl FnOnce(&mut Model)) -> SaveReport {
    let pristine = load(bytes);
    let mut model = load(bytes);
    edit(&mut model);
    model.evaluate();
    save_preserving(
        &model,
        &pristine,
        bytes,
        &ImportLimits::default(),
        Cursor::new(Vec::new()),
    )
    .unwrap()
    .1
}

/// The report's drops as (item, reason name) pairs.
fn drops(report: &SaveReport) -> Vec<(String, &'static str)> {
    (report.dropped.iter())
        .map(|d| (d.item.clone(), d.reason.name()))
        .collect()
}

#[test]
fn the_save_report_names_what_each_tier_left_out() {
    let custom = r#"<root xmlns="urn:x"/>"#;
    let chain = r#"<calcChain xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><c r="A2" i="1"/></calcChain>"#;
    let src = with_entry(&source(), "customXml/item1.xml", custom);
    let src = with_entry(&src, "xl/calcChain.xml", chain);
    let src = rezip(
        &src,
        RELS,
        "</Relationships>",
        r#"<Relationship Id="rIdCC" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/calcChain" Target="calcChain.xml"/><Relationship Id="rIdCX" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml" Target="../customXml/item1.xml"/></Relationships>"#,
    );
    let src = rezip(
        &src,
        TYPES,
        "</Types>",
        r#"<Override PartName="/xl/calcChain.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.calcChain+xml"/></Types>"#,
    );
    // Unchanged: the source, byte for byte, nothing dropped.
    let report = report_after(&src, |_| {});
    assert_eq!(report.tier, SaveTier::Verbatim);
    assert!(report.dropped.is_empty());
    // A cell edit that keeps the formula cells: patched, the chain kept.
    let report = report_after(&src, |m| m.set_user_input(0, 1, 1, "5".into()).unwrap());
    assert!(
        matches!(report.tier, SaveTier::Patched { .. }),
        "{report:?}"
    );
    assert!(report.dropped.is_empty(), "{report:?}");
    // A formula replaced by a constant: patched, the chain dropped by policy.
    let report = report_after(&src, |m| m.set_user_input(0, 2, 1, "9".into()).unwrap());
    assert!(
        matches!(report.tier, SaveTier::Patched { .. }),
        "{report:?}"
    );
    assert_eq!(drops(&report), [("xl/calcChain.xml".to_string(), "Policy")]);
    // A renamed sheet regenerates: the unmodelled custom XML is not preserved, the chain is
    // dropped by policy, and no modelled part is named.
    let report = report_after(&src, |m| m.workbook.worksheets[0].name = "Renamed".into());
    assert!(
        matches!(report.tier, SaveTier::Regenerated { .. }),
        "{report:?}"
    );
    let mut got = drops(&report);
    got.sort();
    assert_eq!(
        got,
        [
            ("customXml/item1.xml".to_string(), "NotPreserved"),
            ("xl/calcChain.xml".to_string(), "Policy"),
        ]
    );
}

/// The report reads the source within the limits it was loaded with (directive 5): over them it
/// says so, rather than reporting that nothing was dropped.
#[test]
fn a_source_the_report_cannot_read_within_the_load_limits_is_reported() {
    let src = source();
    let pristine = load(&src);
    let mut model = load(&src);
    model.workbook.worksheets[0].name = "Renamed".into();
    let tight = ImportLimits {
        max_zip_entries: 1,
        ..ImportLimits::default()
    };
    let (_, report) =
        save_preserving(&model, &pristine, &src, &tight, Cursor::new(Vec::new())).unwrap();
    assert_eq!(drops(&report), [(String::new(), "LimitExceeded")]);
}
