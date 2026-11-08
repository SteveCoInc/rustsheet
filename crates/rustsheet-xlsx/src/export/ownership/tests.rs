//! Tests of the ownership matrix and the decisions of a regenerated save (T-4999).

use std::collections::BTreeSet;
use std::io::{Cursor, Write as _};

use super::decide::regenerated_within;
use super::{policy_of, regenerated, Graph, Policy, Reach, MAX_CLOSURE_DEPTH, MAX_CLOSURE_SIZE};
use crate::export::report::DropReason;
use crate::import::ImportLimits;

const R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";

/// A zip of `files`.
fn zip(files: &[(&str, String)]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, text) in files {
        zip.start_file(*name, zip::write::SimpleFileOptions::default())
            .expect("start");
        zip.write_all(text.as_bytes()).expect("write");
    }
    zip.finish().expect("finish").into_inner()
}

/// A `.rels` part with these (id, kind, target, external) relationships.
fn rels(list: &[(&str, &str, &str, bool)]) -> String {
    let mut out = String::from(
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    );
    for (id, kind, target, external) in list {
        let mode = if *external {
            r#" TargetMode="External""#
        } else {
            ""
        };
        out.push_str(&format!(
            r#"<Relationship Id="{id}" Type="{R}{kind}" Target="{target}"{mode}/>"#
        ));
    }
    out + "</Relationships>"
}

/// A workbook with one sheet; `wb` and `sheet` are extra workbook and sheet relationships,
/// `parts` extra parts.
fn package(
    wb: &[(&str, &str, &str, bool)],
    sheet: &[(&str, &str, &str, bool)],
    parts: &[(&str, String)],
) -> Vec<u8> {
    let mut wb_rels = vec![("rId1", "worksheet", "worksheets/sheet1.xml", false)];
    wb_rels.extend_from_slice(wb);
    let mut files = vec![
        (
            "_rels/.rels",
            rels(&[("rId1", "officeDocument", "xl/workbook.xml", false)]),
        ),
        ("xl/workbook.xml", "<workbook/>".to_string()),
        ("xl/_rels/workbook.xml.rels", rels(&wb_rels)),
        ("xl/worksheets/sheet1.xml", "<worksheet/>".to_string()),
        ("xl/worksheets/_rels/sheet1.xml.rels", rels(sheet)),
    ];
    files.extend(parts.iter().cloned());
    zip(&files)
}

fn limits() -> ImportLimits {
    ImportLimits::default()
}

/// The drops of a regenerated save of `source` whose output is `output`, as (item, reason).
fn drops(source: &[u8], output: &[u8]) -> Vec<(String, DropReason)> {
    regenerated(source, output, &BTreeSet::new(), &limits())
        .expect("decides")
        .into_iter()
        .map(|d| (d.item, d.reason))
        .collect()
}

#[test]
fn each_policy_and_the_unknown_defaults() {
    assert_eq!(
        policy_of("worksheet", None, Reach::Workbook),
        Policy::Modelled
    );
    assert_eq!(
        policy_of("customXml", None, Reach::Workbook),
        Policy::Replay
    );
    assert_eq!(
        policy_of("drawing", None, Reach::Sheet),
        Policy::ReplayIfClean
    );
    assert_eq!(
        policy_of("vbaProject", None, Reach::Workbook),
        Policy::Active
    );
    assert_eq!(policy_of("calcChain", None, Reach::Workbook), Policy::Drop);
    assert_eq!(policy_of("rdArray", None, Reach::Workbook), Policy::Drop);
    assert_eq!(policy_of("certificate", None, Reach::Other), Policy::Drop);
    // Unknown types: by where they hang, or by a content type naming VBA.
    assert_eq!(
        policy_of("futureThing", None, Reach::Package),
        Policy::Replay
    );
    assert_eq!(
        policy_of("futureThing", None, Reach::Workbook),
        Policy::Replay
    );
    assert_eq!(
        policy_of("namedSheetView", None, Reach::Sheet),
        Policy::ReplayIfClean
    );
    assert_eq!(
        policy_of(
            "x",
            Some("application/vnd.ms-office.vbaProject"),
            Reach::Workbook
        ),
        Policy::Active
    );
}

#[test]
fn a_closure_follows_internal_relationships_within_its_limits() {
    // A drawing whose chart points back at a modelled table; an external target is not
    // followed.
    let src = package(
        &[],
        &[
            ("rId1", "drawing", "../drawings/drawing1.xml", false),
            ("rId2", "table", "../tables/table1.xml", false),
        ],
        &[
            ("xl/drawings/drawing1.xml", "<wsDr/>".into()),
            (
                "xl/drawings/_rels/drawing1.xml.rels",
                rels(&[
                    ("rId1", "chart", "../charts/chart1.xml", false),
                    ("rId2", "image", "https://example.com/x.png", true),
                ]),
            ),
            ("xl/charts/chart1.xml", "<chartSpace/>".into()),
            (
                "xl/charts/_rels/chart1.xml.rels",
                rels(&[("rId1", "table", "../tables/table1.xml", false)]),
            ),
            ("xl/tables/table1.xml", "<table/>".into()),
        ],
    );
    let graph = Graph::read(&src, &limits()).expect("reads");
    let closure = graph
        .closure("xl/drawings/drawing1.xml")
        .expect("within limits");
    assert_eq!(
        closure,
        BTreeSet::from([
            "xl/drawings/drawing1.xml".into(),
            "xl/charts/chart1.xml".into(),
            "xl/tables/table1.xml".into()
        ])
    );
    // The output has the table but no drawing: the drawing and its chart reach a modelled
    // part.
    let out = package(
        &[],
        &[("rId2", "table", "../tables/table1.xml", false)],
        &[("xl/tables/table1.xml", "<table/>".into())],
    );
    assert_eq!(
        drops(&src, &out),
        [
            (
                "xl/drawings/drawing1.xml".to_string(),
                DropReason::ClosureReachesRegenerated
            ),
            (
                "xl/charts/chart1.xml".to_string(),
                DropReason::ClosureReachesRegenerated
            ),
        ]
    );
    // A chain deeper than the limit.
    let mut parts = Vec::new();
    for n in 0..=MAX_CLOSURE_DEPTH {
        parts.push((format!("x/p{n}.xml"), "<p/>".to_string()));
        parts.push((
            format!("x/_rels/p{n}.xml.rels"),
            rels(&[("rId1", "next", &format!("p{}.xml", n + 1), false)]),
        ));
    }
    parts.push((
        format!("x/p{}.xml", MAX_CLOSURE_DEPTH + 1),
        "<p/>".to_string(),
    ));
    let parts: Vec<(&str, String)> = parts.iter().map(|(n, t)| (n.as_str(), t.clone())).collect();
    let deep = package(&[("rId9", "next", "../x/p0.xml", false)], &[], &parts);
    let graph = Graph::read(&deep, &limits()).expect("reads");
    assert_eq!(graph.closure("x/p0.xml"), Err("closure depth"));
    // A root fanning out to more parts than the size limit.
    let fan: Vec<(String, String)> = (0..=MAX_CLOSURE_SIZE)
        .map(|n| (format!("x/q{n}.xml"), "<q/>".to_string()))
        .collect();
    let edges: Vec<(String, String)> = (0..=MAX_CLOSURE_SIZE)
        .map(|n| (format!("rId{n}"), format!("q{n}.xml")))
        .collect();
    let edge_refs: Vec<(&str, &str, &str, bool)> = (edges.iter())
        .map(|(id, t)| (id.as_str(), "next", t.as_str(), false))
        .collect();
    let mut parts: Vec<(&str, String)> = fan.iter().map(|(n, t)| (n.as_str(), t.clone())).collect();
    parts.push(("x/root.xml", "<r/>".into()));
    let edge_xml = rels(&edge_refs);
    parts.push(("x/_rels/root.xml.rels", edge_xml));
    let wide = package(&[("rId9", "next", "../x/root.xml", false)], &[], &parts);
    let graph = Graph::read(&wide, &limits()).expect("reads");
    assert_eq!(graph.closure("x/root.xml"), Err("closure size"));
    let got = drops(&wide, &package(&[], &[], &[]));
    assert!(
        got.contains(&(
            "x/root.xml".to_string(),
            DropReason::LimitExceeded {
                limit: "closure size".into()
            }
        )),
        "{:?}",
        &got[..3.min(got.len())]
    );
}

#[test]
fn a_modelled_part_the_output_lacks_is_not_preserved_and_shared_parts_are_decided_once() {
    // Two tables in the source, one written; a custom XML part reached twice.
    let src = package(
        &[
            ("rId7", "customXml", "../customXml/item1.xml", false),
            ("rId8", "customXml", "../customXml/item1.xml", false),
        ],
        &[
            ("rId1", "table", "../tables/table1.xml", false),
            ("rId2", "table", "../tables/table2.xml", false),
        ],
        &[
            ("xl/tables/table1.xml", "<table/>".into()),
            ("xl/tables/table2.xml", "<table/>".into()),
            ("customXml/item1.xml", "<x/>".into()),
            ("xl/calcChain.xml", "<calcChain/>".into()),
        ],
    );
    let out = package(
        &[],
        &[("rId1", "table", "../tables/table1.xml", false)],
        &[("xl/tables/table1.xml", "<table/>".into())],
    );
    let got = drops(&src, &out);
    assert!(
        got.contains(&("xl/tables/table2.xml".to_string(), DropReason::NotPreserved)),
        "{got:?}"
    );
    assert_eq!(
        got.iter()
            .filter(|(i, _)| i == "customXml/item1.xml")
            .count(),
        1,
        "{got:?}"
    );
    // calcChain reached by no relationship here is an orphan, not preserved.
    assert!(
        got.contains(&("xl/calcChain.xml".to_string(), DropReason::NotPreserved)),
        "{got:?}"
    );
}

#[test]
fn coupling_groups_and_pivot_charts_are_decided_as_a_unit() {
    // Notes and a control share the VML; the control's ctrlProp reaches a modelled table, so
    // the whole legacy group goes with it. A chart holding `c:pivotSource` is T-4950's.
    let src = package(
        &[(
            "rId5",
            "slicerCache",
            "slicerCaches/slicerCache1.xml",
            false,
        )],
        &[
            ("rId1", "vmlDrawing", "../drawings/vmlDrawing1.vml", false),
            ("rId2", "comments", "../comments1.xml", false),
            ("rId3", "ctrlProp", "../ctrlProps/ctrlProp1.xml", false),
            ("rId4", "drawing", "../drawings/drawing1.xml", false),
            ("rId6", "slicer", "../slicers/slicer1.xml", false),
        ],
        &[
            ("xl/drawings/vmlDrawing1.vml", "<xml/>".into()),
            ("xl/comments1.xml", "<comments/>".into()),
            ("xl/ctrlProps/ctrlProp1.xml", "<formControlPr/>".into()),
            (
                "xl/ctrlProps/_rels/ctrlProp1.xml.rels",
                rels(&[("rId1", "table", "../tables/table1.xml", false)]),
            ),
            ("xl/tables/table1.xml", "<table/>".into()),
            (
                "xl/drawings/drawing1.xml",
                "<wsDr><a14:compatExt spid=\"_x0000_s1025\"/></wsDr>".into(),
            ),
            (
                "xl/drawings/_rels/drawing1.xml.rels",
                rels(&[("rId1", "chart", "../charts/chart1.xml", false)]),
            ),
            (
                "xl/charts/chart1.xml",
                "<chartSpace><c:pivotSource/></chartSpace>".into(),
            ),
            ("xl/slicers/slicer1.xml", "<slicers/>".into()),
            (
                "xl/slicerCaches/slicerCache1.xml",
                "<slicerCacheDefinition/>".into(),
            ),
        ],
    );
    let out = package(&[], &[], &[]);
    let got = drops(&src, &out);
    for part in [
        "xl/drawings/vmlDrawing1.vml",
        "xl/comments1.xml",
        "xl/ctrlProps/ctrlProp1.xml",
        "xl/drawings/drawing1.xml",
        "xl/charts/chart1.xml",
    ] {
        assert!(
            got.contains(&(part.to_string(), DropReason::ClosureReachesRegenerated)),
            "{part}: {got:?}"
        );
    }
    // The slicer and its cache form the workbook's slicer group; nothing writes them yet.
    for part in ["xl/slicers/slicer1.xml", "xl/slicerCaches/slicerCache1.xml"] {
        assert!(
            got.contains(&(part.to_string(), DropReason::NotPreserved)),
            "{part}: {got:?}"
        );
    }
}

/// A two-sheet book whose second sheet (`sheetId` 2) holds a table; `swapped` lists the sheets
/// in the other order, in `<sheets>` and in the `.rels`, as a save after moving them would.
fn two_sheets(swapped: bool) -> Vec<u8> {
    let r = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let (first, second) = if swapped {
        ("rId2", "rId1")
    } else {
        ("rId1", "rId2")
    };
    let sheets = if swapped {
        format!(
            r#"<sheet name="B" sheetId="2" r:id="{second}"/><sheet name="A" sheetId="1" r:id="{first}"/>"#
        )
    } else {
        format!(
            r#"<sheet name="A" sheetId="1" r:id="{first}"/><sheet name="B" sheetId="2" r:id="{second}"/>"#
        )
    };
    let workbook = format!(r#"<workbook xmlns:r="{r}"><sheets>{sheets}</sheets></workbook>"#);
    let wb_rels = rels(&[
        (first, "worksheet", "worksheets/sheet1.xml", false),
        (second, "worksheet", "worksheets/sheet2.xml", false),
    ]);
    zip(&[
        (
            "_rels/.rels",
            rels(&[("rId1", "officeDocument", "xl/workbook.xml", false)]),
        ),
        ("xl/workbook.xml", workbook),
        ("xl/_rels/workbook.xml.rels", wb_rels),
        ("xl/worksheets/sheet1.xml", "<worksheet/>".into()),
        ("xl/worksheets/sheet2.xml", "<worksheet/>".into()),
        (
            "xl/worksheets/_rels/sheet2.xml.rels",
            rels(&[("rId1", "table", "../tables/table1.xml", false)]),
        ),
        ("xl/tables/table1.xml", "<table/>".into()),
    ])
}

#[test]
fn a_sheet_is_matched_by_its_sheet_id_wherever_it_moved() {
    assert!(drops(&two_sheets(false), &two_sheets(true)).is_empty());
}

/// The drops of `source` saved as an empty book, as (item, reason name), sorted.
fn named(source: &[u8], max_work: usize) -> Vec<(String, String)> {
    let mut got: Vec<(String, String)> = regenerated_within(
        source,
        &package(&[], &[], &[]),
        &BTreeSet::new(),
        &limits(),
        max_work,
    )
    .expect("decides")
    .into_iter()
    .map(|d| {
        let reason = match &d.reason {
            DropReason::LimitExceeded { limit } => format!("LimitExceeded {limit}"),
            other => other.name().to_string(),
        };
        (d.item, reason)
    })
    .collect();
    got.sort();
    got
}

fn pair(item: &str, reason: &str) -> (String, String) {
    (item.to_string(), reason.to_string())
}

#[test]
fn a_part_takes_its_roots_verdict_whatever_its_name_and_a_kept_root_keeps_what_it_shares() {
    // chart1 has no relationship of its own and sorts before its drawing; the drawing reaches a
    // table through chart2. An image shared with a clean second drawing stays keepable.
    let src = package(
        &[],
        &[
            ("rId1", "drawing", "../drawings/drawing1.xml", false),
            ("rId2", "drawing", "../drawings/drawing2.xml", false),
            ("rId3", "table", "../tables/table1.xml", false),
        ],
        &[
            ("xl/drawings/drawing1.xml", "<wsDr/>".into()),
            (
                "xl/drawings/_rels/drawing1.xml.rels",
                rels(&[
                    ("rId1", "chart", "../charts/chart1.xml", false),
                    ("rId2", "chart", "../charts/chart2.xml", false),
                    ("rId3", "image", "../media/image1.png", false),
                ]),
            ),
            ("xl/drawings/drawing2.xml", "<wsDr/>".into()),
            (
                "xl/drawings/_rels/drawing2.xml.rels",
                rels(&[("rId1", "image", "../media/image1.png", false)]),
            ),
            ("xl/charts/chart1.xml", "<chartSpace/>".into()),
            ("xl/charts/chart2.xml", "<chartSpace/>".into()),
            (
                "xl/charts/_rels/chart2.xml.rels",
                rels(&[("rId1", "table", "../tables/table1.xml", false)]),
            ),
            ("xl/tables/table1.xml", r#"<table name="T1"/>"#.into()),
            ("xl/media/image1.png", "png".into()),
        ],
    );
    let got = named(&src, usize::MAX);
    assert_eq!(
        got,
        [
            pair("xl/charts/chart1.xml", "ClosureReachesRegenerated"),
            pair("xl/charts/chart2.xml", "ClosureReachesRegenerated"),
            pair("xl/drawings/drawing1.xml", "ClosureReachesRegenerated"),
            pair("xl/drawings/drawing2.xml", "NotPreserved"),
            pair("xl/media/image1.png", "NotPreserved"),
            pair("xl/tables/table1.xml", "NotPreserved"),
        ]
    );
}

#[test]
fn groups_are_per_sheet_joined_by_their_caches_and_merged_where_they_overlap() {
    // Sheet 1: a slicer on cache A whose drawing also holds a compatExt shape (so its legacy and
    // slicer groups merge) and a pivot chart. Sheet 2: a timeline on cache B, clean.
    let r = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let workbook = format!(
        r#"<workbook xmlns:r="{r}"><sheets><sheet name="A" sheetId="1" r:id="rId1"/><sheet name="B" sheetId="2" r:id="rId2"/></sheets></workbook>"#
    );
    let src = zip(&[
        ("_rels/.rels", rels(&[("rId1", "officeDocument", "xl/workbook.xml", false)])),
        ("xl/workbook.xml", workbook),
        (
            "xl/_rels/workbook.xml.rels",
            rels(&[
                ("rId1", "worksheet", "worksheets/sheet1.xml", false),
                ("rId2", "worksheet", "worksheets/sheet2.xml", false),
                ("rId3", "slicerCache", "slicerCaches/slicerCache1.xml", false),
                ("rId4", "timelineCache", "timelineCaches/timelineCache1.xml", false),
            ]),
        ),
        ("xl/worksheets/sheet1.xml", "<worksheet/>".into()),
        ("xl/worksheets/sheet2.xml", "<worksheet/>".into()),
        (
            "xl/worksheets/_rels/sheet1.xml.rels",
            rels(&[
                ("rId1", "slicer", "../slicers/slicer1.xml", false),
                ("rId2", "drawing", "../drawings/drawing1.xml", false),
                ("rId3", "comments", "../comments1.xml", false),
            ]),
        ),
        ("xl/worksheets/_rels/sheet2.xml.rels", rels(&[("rId1", "timeline", "../timelines/timeline1.xml", false)])),
        ("xl/slicers/slicer1.xml", r#"<slicers><slicer name="s" cache="Slicer_A"/></slicers>"#.into()),
        ("xl/timelines/timeline1.xml", r#"<timelines><timeline name="t" cache="NativeTimeline_B"/></timelines>"#.into()),
        ("xl/slicerCaches/slicerCache1.xml", r#"<slicerCacheDefinition name="Slicer_A"/>"#.into()),
        ("xl/timelineCaches/timelineCache1.xml", r#"<timelineCacheDefinition name="NativeTimeline_B"/>"#.into()),
        ("xl/comments1.xml", "<comments/>".into()),
        (
            "xl/drawings/drawing1.xml",
            r#"<wsDr><a14:compatExt spid="_x0000_s1"/><a:graphicData uri="http://schemas.microsoft.com/office/drawing/2010/slicer"/></wsDr>"#.into(),
        ),
        ("xl/drawings/_rels/drawing1.xml.rels", rels(&[("rId1", "chart", "../charts/chart1.xml", false)])),
        ("xl/charts/chart1.xml", "<chartSpace><c:pivotSource/></chartSpace>".into()),
    ]);
    let got = named(&src, usize::MAX);
    let reason = |item: &str| got.iter().find(|(i, _)| i == item).map(|(_, r)| r.as_str());
    for part in [
        "xl/slicers/slicer1.xml",
        "xl/slicerCaches/slicerCache1.xml",
        "xl/drawings/drawing1.xml",
        "xl/comments1.xml",
        "xl/charts/chart1.xml",
    ] {
        assert_eq!(
            reason(part),
            Some("ClosureReachesRegenerated"),
            "{part}: {got:?}"
        );
    }
    for part in [
        "xl/timelines/timeline1.xml",
        "xl/timelineCaches/timelineCache1.xml",
    ] {
        assert_eq!(reason(part), Some("NotPreserved"), "{part}: {got:?}");
    }
}

#[test]
fn a_missing_table_is_named_by_its_name_not_its_position() {
    let tables = |names: &[&str]| -> Vec<u8> {
        let sheet: Vec<(String, String)> = (names.iter().enumerate())
            .map(|(n, _)| (format!("rId{n}"), format!("../tables/table{n}.xml")))
            .collect();
        let sheet: Vec<(&str, &str, &str, bool)> = (sheet.iter())
            .map(|(id, t)| (id.as_str(), "table", t.as_str(), false))
            .collect();
        let parts: Vec<(String, String)> = (names.iter().enumerate())
            .map(|(n, name)| {
                (
                    format!("xl/tables/table{n}.xml"),
                    format!(r#"<table name="{name}"/>"#),
                )
            })
            .collect();
        let parts: Vec<(&str, String)> =
            parts.iter().map(|(p, t)| (p.as_str(), t.clone())).collect();
        package(&[], &sheet, &parts)
    };
    let got = drops(&tables(&["A", "B", "C"]), &tables(&["A", "C"]));
    assert_eq!(
        got,
        [("xl/tables/table1.xml".to_string(), DropReason::NotPreserved)]
    );
}

#[test]
fn the_work_limit_carries_to_every_remaining_unit_and_an_unreadable_drawing_drops_its_group() {
    // Three custom XML roots; past the work limit, the units after the first are not decided.
    let src = package(
        &[
            ("rId7", "customXml", "../customXml/item1.xml", false),
            ("rId8", "customXml", "../customXml/item2.xml", false),
            ("rId9", "customXml", "../customXml/item3.xml", false),
        ],
        &[],
        &[
            ("customXml/item1.xml", "<x/>".into()),
            ("customXml/item2.xml", "<x/>".into()),
            ("customXml/item3.xml", "<x/>".into()),
        ],
    );
    let got = named(&src, 1);
    assert!(
        got.contains(&pair("customXml/item1.xml", "NotPreserved")),
        "{got:?}"
    );
    assert!(
        got.contains(&pair("customXml/item3.xml", "LimitExceeded closure work")),
        "{got:?}"
    );
    // A drawing that is not UTF-8 cannot be probed: its sheet's legacy group drops on a limit.
    let mut files = vec![
        (
            "_rels/.rels",
            rels(&[("rId1", "officeDocument", "xl/workbook.xml", false)]).into_bytes(),
        ),
        ("xl/workbook.xml", b"<workbook/>".to_vec()),
        (
            "xl/_rels/workbook.xml.rels",
            rels(&[("rId1", "worksheet", "worksheets/sheet1.xml", false)]).into_bytes(),
        ),
        ("xl/worksheets/sheet1.xml", b"<worksheet/>".to_vec()),
        (
            "xl/worksheets/_rels/sheet1.xml.rels",
            rels(&[
                ("rId1", "drawing", "../drawings/drawing1.xml", false),
                ("rId2", "comments", "../comments1.xml", false),
            ])
            .into_bytes(),
        ),
        ("xl/comments1.xml", b"<comments/>".to_vec()),
    ];
    files.push(("xl/drawings/drawing1.xml", vec![0xff, 0xfe, 0x00]));
    let mut zw = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in &files {
        zw.start_file(*name, zip::write::SimpleFileOptions::default())
            .expect("start");
        zw.write_all(data).expect("write");
    }
    let src = zw.finish().expect("finish").into_inner();
    let got = named(&src, usize::MAX);
    for part in ["xl/drawings/drawing1.xml", "xl/comments1.xml"] {
        assert!(
            got.contains(&pair(part, "LimitExceeded part read")),
            "{part}: {got:?}"
        );
    }
}

#[test]
fn a_drawing_past_the_probe_cap_is_unreadable_not_uncoupled() {
    // A compatExt shape past the first MiB: the probe cannot see it, so the group drops on a
    // limit rather than the drawing being decided alone.
    let big = format!(
        "<wsDr>{}<a14:compatExt spid=\"_x0000_s1\"/></wsDr>",
        " ".repeat(1 << 20)
    );
    let src = package(
        &[],
        &[
            ("rId1", "drawing", "../drawings/drawing1.xml", false),
            ("rId2", "comments", "../comments1.xml", false),
        ],
        &[
            ("xl/drawings/drawing1.xml", big),
            ("xl/comments1.xml", "<comments/>".into()),
        ],
    );
    let got = named(&src, usize::MAX);
    for part in ["xl/drawings/drawing1.xml", "xl/comments1.xml"] {
        assert!(
            got.contains(&pair(part, "LimitExceeded part read")),
            "{part}: {got:?}"
        );
    }
}
