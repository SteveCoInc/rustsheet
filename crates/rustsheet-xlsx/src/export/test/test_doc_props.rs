//! Parts Excel for the web refuses to open when they hold an empty element (T-4909;
//! docs/excel-semantics/xlsx-storage.md "An empty `dcterms:created` makes Excel refuse the
//! package" and "An empty `cellStyles` makes Excel refuse the package").

use super::test_export::new_empty_model;
use crate::export::doc_props::get_core_xml;
use crate::export::styles::get_styles_xml;

#[test]
fn an_unknown_creation_date_is_left_out() {
    let mut model = new_empty_model();
    model.workbook.metadata.created = String::new();
    let xml = get_core_xml(&model.workbook, 0).unwrap();
    assert!(!xml.contains("dcterms:created"), "{xml}");
    model.workbook.metadata.created = "2021-09-28T20:02:21Z".into();
    let xml = get_core_xml(&model.workbook, 0).unwrap();
    assert!(
        xml.contains(
            r#"<dcterms:created xsi:type="dcterms:W3CDTF">2021-09-28T20:02:21Z</dcterms:created>"#
        ),
        "{xml}"
    );
}

#[test]
fn the_creator_is_escaped() {
    let mut model = new_empty_model();
    model.workbook.metadata.creator = "A & <B>".into();
    let xml = get_core_xml(&model.workbook, 0).unwrap();
    assert!(
        xml.contains("<dc:creator>A &amp; &lt;B&gt;</dc:creator>"),
        "{xml}"
    );
}

#[test]
fn an_empty_cell_style_table_is_left_out() {
    let mut model = new_empty_model();
    assert!(get_styles_xml(&model.workbook).contains(r#"<cellStyles count="1">"#));
    model.workbook.styles.cell_styles.clear();
    let xml = get_styles_xml(&model.workbook);
    assert!(!xml.contains("cellStyles"), "{xml}");
}
