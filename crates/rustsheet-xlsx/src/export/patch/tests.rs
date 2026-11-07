use std::collections::BTreeSet;

use rustsheet_engine::expressions::token::Error;
use rustsheet_engine::types::{Cell, FormulaValue};

use super::strings::{append_strings, count_si, set_count};
use super::tag::set_attr;
use super::{index_sheet, patch_value, rich_value};

const SHEET: &str = r#"<worksheet><sheetData><row r="1" spans="1:3"><c r="A1"><v>1</v></c><c r="C1" s="2"><f>A1*2</f><v>0</v></c></row><row r="3"><c r="B3" t="s"><v>0</v></c></row></sheetData><extLst><ext uri="{X}"/></extLst></worksheet>"#;

#[test]
fn a_new_cached_value_keeps_the_formula_and_every_other_byte() {
    let index = index_sheet(SHEET).expect("indexes");
    let c = &index.rows[0].cells[1];
    let old = &SHEET[c.start..c.end];
    let new = r#"<c r="C1" s="2"><f>A1*2</f><v>2</v></c>"#;
    assert_eq!(
        patch_value(old, new, false).as_deref(),
        Some(r#"<c r="C1" s="2"><f>A1*2</f><v>2</v></c>"#)
    );
    // The value becomes text: `t` is set, the stored formula text is kept as it was.
    let stored = r#"<c r="C1" s="2"><f> A1 * 2 </f><v>0</v></c>"#;
    let text = r#"<c r="C1" t="str" s="2"><f>A1*2</f><v>x</v></c>"#;
    assert_eq!(
        patch_value(stored, text, false).as_deref(),
        Some(r#"<c r="C1" s="2" t="str"><f> A1 * 2 </f><v>x</v></c>"#)
    );
}

#[test]
fn the_index_refuses_what_it_will_not_guess_at() {
    for (xml, why) in [
        (
            r#"<worksheet><sheetData><row r="2"/><row r="1"/></sheetData></worksheet>"#,
            "rows out of order",
        ),
        (
            r#"<worksheet><sheetData><row r="1"><c r="B1"/><c r="A1"/></row></sheetData></worksheet>"#,
            "cells out of order",
        ),
        (
            r#"<worksheet><sheetData><row r="1"><c/></row></sheetData></worksheet>"#,
            "a cell without r",
        ),
    ] {
        assert_eq!(index_sheet(xml).err().as_deref(), Some(why), "{xml}");
    }
    let deep = format!(
        "<worksheet>{}{}</worksheet>",
        "<a>".repeat(300),
        "</a>".repeat(300)
    );
    assert!(index_sheet(&deep).is_err());
}

#[test]
fn the_index_finds_rows_cells_and_shared_formulas() {
    let xml = r#"<worksheet><sheetData><row r="1"><c r="A1"><f t="shared" ref="A1:A2" si="0">B1</f><v>1</v></c></row><row r="2"><c r="A2"><f t="shared" si="0"/><v>1</v></c></row></sheetData></worksheet>"#;
    let index = index_sheet(xml).expect("indexes");
    let rows: BTreeSet<i32> = index.rows.iter().map(|r| r.r).collect();
    assert_eq!(rows, BTreeSet::from([1, 2]));
    assert!(index.rows.iter().all(|r| r.cells.iter().all(|c| c.shared)));
}

#[test]
fn new_strings_are_appended_with_both_counts() {
    let sst =
        r#"<sst xmlns="x" count="3" uniqueCount="2"><si><t>a</t></si><si><t>b</t></si></sst>"#;
    let added = ["c <&>".to_string(), " d".to_string()];
    assert_eq!(
        append_strings(sst, &added, 4, (5, 3)).as_deref(),
        Some(
            r#"<sst xmlns="x" count="5" uniqueCount="4"><si><t>a</t></si><si><t>b</t></si><si><t>c &lt;&amp;&gt;</t></si><si><t xml:space="preserve"> d</t></si></sst>"#
        )
    );
}

#[test]
fn only_sheet_data_rows_and_cells_count_and_prefixes_fall_back() {
    // A `row` and a `c` inside an extension are not the sheet's.
    let xml = r#"<worksheet><sheetData><row r="1"><c r="A1"><v>1</v></c></row></sheetData><extLst><ext><row r="1"><c r="Z9"/></row></ext></extLst></worksheet>"#;
    let index = index_sheet(xml).expect("indexes");
    assert_eq!(index.rows.len(), 1);
    assert_eq!(index.rows[0].cells.len(), 1);
    let prefixed =
        r#"<x:worksheet xmlns:x="m"><x:sheetData><x:row r="1"/></x:sheetData></x:worksheet>"#;
    assert_eq!(
        index_sheet(prefixed).err().as_deref(),
        Some("a prefixed worksheet")
    );
}

#[test]
fn a_single_quoted_attribute_is_requoted_and_a_rich_error_keeps_only_its_own_kind() {
    let old = r#"<c r='A1' x='a"b'><f>1</f><v>0</v></c>"#;
    let new = r#"<c r="A1"><f>1</f><v>1</v></c>"#;
    assert_eq!(
        patch_value(old, new, false).as_deref(),
        Some(r#"<c r="A1" x="a&quot;b"><f>1</f><v>1</v></c>"#)
    );
    // The source's `vm` is kept as written; the export's number is never copied in.
    let kept = r#"<c r="A1" t="e" vm="3"><f>1</f><v>#VALUE!</v></c>"#;
    let export = r#"<c r="A1" t="e" vm="1"><f>1</f><v>#VALUE!</v></c>"#;
    assert_eq!(patch_value(kept, export, false).as_deref(), Some(kept));
    // `vm="1"` in both, but the source's names #CALC! and the export's #SPILL!: a new kind of
    // rich error falls back (T-4561), as does a rich error the source cell did not have.
    let formula = |ei| Cell::CellFormula {
        f: 0,
        s: 0,
        v: FormulaValue::Error {
            ei,
            o: String::new(),
            m: String::new(),
        },
    };
    let (calc, spill) = (formula(Error::CALC), formula(Error::SPILL));
    let source = r#"<c r="A1" t="e" vm="1"><f>1</f><v>#VALUE!</v></c>"#;
    assert!(rich_value(source, export, &calc, &spill).is_err());
    assert!(rich_value(source, export, &calc, &calc).is_ok());
    assert!(rich_value(old, export, &calc, &spill).is_err());
    assert!(rich_value(old, new, &calc, &spill).is_ok());
}

#[test]
fn an_attribute_is_found_in_either_quote_and_never_inside_another_value() {
    assert_eq!(
        set_attr("<fonts count='2'", "count", |n| n + 1),
        "<fonts count='3'"
    );
    assert_eq!(
        set_attr("<fonts\tcount = \"2\"", "count", |_| 5),
        "<fonts\tcount = \"5\""
    );
    assert_eq!(
        set_attr(r#"<fonts x='a count="9"' count="2""#, "count", |_| 3),
        r#"<fonts x='a count="9"' count="3""#
    );
    assert_eq!(
        set_attr(r#"<fonts recount="2""#, "count", |_| 3),
        r#"<fonts recount="2""#
    );
    assert_eq!(
        set_attr(r#"<fonts count="x""#, "count", |_| 3),
        r#"<fonts count="x""#
    );
    assert_eq!(
        set_attr(r#"<fonts é="1" count="2""#, "count", |_| 3),
        r#"<fonts é="1" count="3""#
    );
}

#[test]
fn counts_are_set_in_the_sst_tag_only() {
    let sst = r#"<sst uniqueCount="1"><si><t>x count="7"</t></si>"#;
    assert_eq!(set_count(sst, "count", |n| n + 1), sst);
    assert_eq!(
        set_count(sst, "uniqueCount", |_| 2),
        r#"<sst uniqueCount="2"><si><t>x count="7"</t></si>"#
    );
    assert_eq!(
        count_si(r"<sst><si><t>a</t></si><si/><si><r><t>b</t></r></si></sst>"),
        Ok(3)
    );
}

mod styles {
    use rustsheet_engine::types::{Border, Dxf, NumFmt, Styles};

    use super::super::styles::{appended, patch};

    #[test]
    fn only_a_prefix_stable_table_is_patched() {
        let pristine = Styles::default();
        let mut live = pristine.clone();
        live.num_fmts.push(NumFmt {
            num_fmt_id: 164,
            format_code: "0.000".into(),
        });
        live.cell_xfs.push(live.cell_xfs[0].clone());
        assert!(appended(&pristine, &live).is_some());
        // A record the source had, changed in place: every `s=` naming it would change meaning.
        let mut moved = live.clone();
        moved.cell_xfs[0].num_fmt_id = 164;
        assert!(appended(&pristine, &moved).is_none());
        // A named style, a cell style xf or a dxf: none of them is written by the patch.
        let mut named = live.clone();
        named.cell_styles.clear();
        assert!(appended(&pristine, &named).is_none());
        let mut style_xf = live.clone();
        style_xf
            .cell_style_xfs
            .push(style_xf.cell_style_xfs[0].clone());
        assert!(appended(&pristine, &style_xf).is_none());
        let mut dxf = live.clone();
        dxf.dxfs.push(Dxf::default());
        assert!(appended(&pristine, &dxf).is_none());
    }

    #[test]
    fn records_are_appended_with_their_counts_and_a_missing_num_fmts_is_created() {
        let mut pristine = Styles::default();
        pristine.num_fmts.clear();
        let mut live = pristine.clone();
        live.num_fmts.push(NumFmt {
            num_fmt_id: 164,
            format_code: "0.000".into(),
        });
        live.cell_xfs.push(live.cell_xfs[0].clone());
        let xfs = pristine.cell_xfs.len();
        let fonts = pristine.fonts.len();
        let source = format!(
            r#"<styleSheet xmlns="m"><fonts count="{fonts}">{}</fonts><fills count="0"/><borders count="0"/><cellXfs count="{xfs}">{}</cellXfs></styleSheet>"#,
            "<font/>".repeat(fonts),
            "<xf/>".repeat(xfs)
        );
        let added = appended(&pristine, &live).expect("prefix-stable");
        let out = patch(&source, &pristine, &added).expect("patches");
        assert!(out.starts_with(r#"<styleSheet xmlns="m"><numFmts count="1"><numFmt numFmtId="164" formatCode="0.000"/></numFmts>"#), "{out}");
        assert!(
            out.contains(&format!(r#"<cellXfs count="{}">"#, xfs + 1)),
            "{out}"
        );
        // A section that holds other than what the importer read is refused.
        let short = source.replacen("<xf/>", "", 1);
        assert!(patch(&short, &pristine, &added).is_err());
    }

    /// A source `styles.xml` whose sections hold the pristine table's records, `borders` as given.
    fn source(pristine: &Styles, bom: &str, borders: &str) -> String {
        format!(
            r#"{bom}<styleSheet xmlns="m"><numFmts count="1"><numFmt numFmtId="164" formatCode="€ 0"/></numFmts><fonts count="{}">{}</fonts><fills count="{}">{}</fills>{borders}<cellXfs count="{}">{}</cellXfs></styleSheet>"#,
            pristine.fonts.len(),
            "<font/>".repeat(pristine.fonts.len()),
            pristine.fills.len(),
            "<fill/>".repeat(pristine.fills.len()),
            pristine.cell_xfs.len(),
            "<xf/>".repeat(pristine.cell_xfs.len()),
        )
    }

    #[test]
    fn fills_and_borders_append_after_a_bom_into_an_empty_single_quoted_section() {
        let pristine = Styles {
            num_fmts: vec![NumFmt {
                num_fmt_id: 164,
                format_code: "€ 0".into(),
            }],
            borders: Vec::new(),
            ..Styles::default()
        };
        let mut live = pristine.clone();
        live.fills.push(live.fills[0].clone());
        live.borders.push(Border::default());
        live.num_fmts.push(NumFmt {
            num_fmt_id: 165,
            format_code: "€ 0.0".into(),
        });
        let source = source(&pristine, "\u{feff}", "<borders count='0'/>");
        let added = appended(&pristine, &live).expect("prefix-stable");
        let out = patch(&source, &pristine, &added).expect("patches");
        assert!(out.starts_with("\u{feff}<styleSheet"), "{out}");
        assert!(
            out.contains(r#"<numFmts count="2"><numFmt numFmtId="164" formatCode="€ 0"/><numFmt numFmtId="165""#),
            "{out}"
        );
        let fills = pristine.fills.len();
        assert!(
            out.contains(&format!(r#"<fills count="{}">"#, fills + 1)),
            "{out}"
        );
        assert!(out.contains("<borders count='1'><border>"), "{out}");
        // Every source byte before the first appended record is kept.
        let first = source.find("</numFmts>").expect("numFmts");
        assert_eq!(
            out[..first].replace("count=\"2\"", "count=\"1\""),
            source[..first]
        );
    }

    #[test]
    fn a_new_fill_in_a_reserved_slot_regenerates() {
        let mut pristine = Styles::default();
        pristine.fills.truncate(1);
        let mut live = pristine.clone();
        live.fills.push(live.fills[0].clone());
        let source = source(&pristine, "", "<borders count=\"1\"><border/></borders>");
        let added = appended(&pristine, &live).expect("prefix-stable");
        assert!(patch(&source, &pristine, &added).is_err());
    }
}
