use std::collections::BTreeSet;

use super::{append_strings, count_si, index_sheet, patch_value, set_count};

const SHEET: &str = r#"<worksheet><sheetData><row r="1" spans="1:3"><c r="A1"><v>1</v></c><c r="C1" s="2"><f>A1*2</f><v>0</v></c></row><row r="3"><c r="B3" t="s"><v>0</v></c></row></sheetData><extLst><ext uri="{X}"/></extLst></worksheet>"#;

#[test]
fn a_new_cached_value_keeps_the_formula_and_every_other_byte() {
    let index = index_sheet(SHEET).expect("indexes");
    let c = &index.rows[0].cells[1];
    let old = &SHEET[c.start..c.end];
    let new = r#"<c r="C1" s="2"><f>A1*2</f><v>2</v></c>"#;
    assert_eq!(
        patch_value(old, new).as_deref(),
        Some(r#"<c r="C1" s="2"><f>A1*2</f><v>2</v></c>"#)
    );
    // The value becomes text: `t` is set, the stored formula text is kept as it was.
    let stored = r#"<c r="C1" s="2"><f> A1 * 2 </f><v>0</v></c>"#;
    let text = r#"<c r="C1" t="str" s="2"><f>A1*2</f><v>x</v></c>"#;
    assert_eq!(
        patch_value(stored, text).as_deref(),
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
fn a_single_quoted_attribute_is_requoted_and_a_new_vm_is_refused() {
    let old = r#"<c r='A1' x='a"b'><f>1</f><v>0</v></c>"#;
    let new = r#"<c r="A1"><f>1</f><v>1</v></c>"#;
    assert_eq!(
        patch_value(old, new).as_deref(),
        Some(r#"<c r="A1" x="a&quot;b"><f>1</f><v>1</v></c>"#)
    );
    // The export's `vm` names its own metadata layout, not the source's (T-4561).
    let rich = r#"<c r="A1" t="e" vm="2"><f>1</f><v>#CALC!</v></c>"#;
    assert_eq!(patch_value(old, rich), None);
    let same = r#"<c r="A1" t="e" vm="1"><f>1</f><v>#SPILL!</v></c>"#;
    let kept = r#"<c r="A1" t="e" vm="1"><f>1</f><v>#CALC!</v></c>"#;
    assert_eq!(
        patch_value(kept, same).as_deref(),
        Some(r#"<c r="A1" t="e" vm="1"><f>1</f><v>#SPILL!</v></c>"#)
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
        count_si(r#"<sst><si><t>a</t></si><si/><si><r><t>b</t></r></si></sst>"#),
        Ok(3)
    );
}
