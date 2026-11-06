#![allow(clippy::unwrap_used, clippy::expect_used, reason = "tests")]

//! SPEC-0034 D3, T-4560: a workbook whose cells changed but whose structure did not is saved by
//! patching the source. Only the changed cells' sheet and, for new strings, `sharedStrings.xml`
//! change; every other part is byte-identical. Whatever the patcher will not do exactly
//! regenerates.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read, Write};

use rustsheet_engine::expressions::types::Area;
use rustsheet_engine::Model;
use rustsheet_xlsx::export::{save_preserving, save_xlsx_to_writer, SaveTier};
use rustsheet_xlsx::import::load_from_xlsx_bytes;

const SHEET: &str = "xl/worksheets/sheet1.xml";
const STRINGS: &str = "xl/sharedStrings.xml";

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
    let (out, tier) = save_preserving(&model, &pristine, bytes, Cursor::new(Vec::new())).unwrap();
    (out.into_inner(), tier)
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
    assert!(part(&out, SHEET).contains(r#"<row r="9""#));
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
    // A new style: the style table changed (T-4561).
    regenerated(
        &src,
        |m| {
            m.set_user_input(0, 1, 1, "5".into()).unwrap();
            let mut style = m.get_style_for_cell(0, 1, 1).unwrap();
            style.font.b = true;
            m.set_cell_style(0, 1, 1, &style).unwrap();
        },
        "a new style",
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
