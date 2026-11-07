#![allow(clippy::unwrap_used, reason = "tests")]
//! The checks a drawing and a chart pass before a regenerated save keeps them (T-4950), and the
//! `c:pivotSource` reading that finds a chart's pivot. The whole save is tested on Excel's file
//! in `crates/rustsheet/tests/pivot_chart_save.rs`.

use std::collections::BTreeSet;

use super::super::pivot_charts::pivot_source;
use super::check_part;

/// A drawing as Excel writes one chart (t4187's `xl/drawings/drawing1.xml`), with `anchor`'s
/// content in place of the chart frame's when given.
fn drawing(frame: &str) -> String {
    format!(
        r#"<xdr:wsDr xmlns:xdr="http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><xdr:twoCellAnchor><xdr:from><xdr:col>4</xdr:col></xdr:from><xdr:to><xdr:col>11</xdr:col></xdr:to>{frame}<xdr:clientData/></xdr:twoCellAnchor></xdr:wsDr>"#
    )
}

const CHART_FRAME: &str = r#"<xdr:graphicFrame macro=""><xdr:nvGraphicFramePr><xdr:cNvPr id="2" name="Chart 1"/><xdr:cNvGraphicFramePr/></xdr:nvGraphicFramePr><xdr:xfrm/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" r:id="rId1"/></a:graphicData></a:graphic></xdr:graphicFrame>"#;

fn ids(ids: &[&str]) -> BTreeSet<String> {
    ids.iter().map(ToString::to_string).collect()
}

#[test]
fn a_drawing_of_chart_frames_passes() {
    assert_eq!(
        check_part(&drawing(CHART_FRAME), &ids(&["rId1"]), true),
        Ok(())
    );
}

#[test]
fn a_picture_shape_or_other_frame_is_refused() {
    let picture = r#"<xdr:pic><xdr:blipFill><a:blip r:embed="rId1"/></xdr:blipFill></xdr:pic>"#;
    let shape = r#"<xdr:sp macro="" textlink=""><xdr:spPr/></xdr:sp>"#;
    let table = CHART_FRAME.replace(
        "drawingml/2006/chart\"><c:chart",
        "drawingml/2006/table\"><c:chart",
    );
    for frame in [picture, shape, table.as_str()] {
        assert!(
            check_part(&drawing(frame), &ids(&["rId1"]), true).is_err(),
            "{frame}"
        );
    }
    let beside = format!("{CHART_FRAME}{shape}");
    assert!(check_part(&drawing(&beside), &ids(&["rId1"]), true).is_err());
}

#[test]
fn a_reference_with_no_relationship_is_refused() {
    assert!(check_part(&drawing(CHART_FRAME), &ids(&["rId2"]), true).is_err());
    // Any prefix bound to the relationships namespace counts, in a chart too.
    let chart = r#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:q="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><c:userShapes q:id="rId3"/></c:chartSpace>"#;
    assert!(check_part(chart, &ids(&[]), false).is_err());
    assert_eq!(check_part(chart, &ids(&["rId3"]), false), Ok(()));
}

#[test]
fn a_document_type_is_refused() {
    let xml = format!("<!DOCTYPE x [<!ENTITY e \"e\">]>{}", drawing(CHART_FRAME));
    assert!(check_part(&xml, &ids(&["rId1"]), true).is_err());
}

#[test]
fn a_part_nested_past_the_cap_is_refused() {
    let deep = format!(
        "<c:chartSpace xmlns:c=\"c\">{}{}</c:chartSpace>",
        "<c:x>".repeat(300),
        "</c:x>".repeat(300)
    );
    assert!(check_part(&deep, &ids(&[]), false).is_err());
}

#[test]
fn the_pivot_source_names_a_sheet_and_a_pivot() {
    let chart = |name: &str| {
        format!(
            r#"<c:chartSpace xmlns:c="c"><c:date1904 val="0"/><c:pivotSource><c:name>{name}</c:name><c:fmtId val="0"/></c:pivotSource><c:chart/></c:chartSpace>"#
        )
    };
    let pair = |s: &str, p: &str| Some((s.to_string(), p.to_string()));
    // t4187's chart1.
    assert_eq!(pivot_source(&chart("[da80.xlsx]C!P_C")), pair("C", "P_C"));
    assert_eq!(
        pivot_source(&chart("Sheet1!PivotTable1")),
        pair("Sheet1", "PivotTable1")
    );
    assert_eq!(
        pivot_source(&chart("[a.xlsx]'It''s &amp; X'!P!1")),
        pair("It's & X", "P!1")
    );
    assert_eq!(pivot_source(&chart("[a.xlsx]NoPivot")), None);
    assert_eq!(pivot_source(&chart("[a.xlsx]'Open!P")), None);
    assert_eq!(
        pivot_source(
            r#"<c:chartSpace xmlns:c="c"><c:chart><c:name>S!P</c:name></c:chart></c:chartSpace>"#
        ),
        None
    );
}
