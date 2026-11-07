//! T-4950: a save that regenerates the package keeps the drawings whose closure is pivot charts
//! only, each chart's series rewritten from its pivot (`rewrite_pivot_chart`). The evidence is
//! the file Excel for the web saved for probes/t4187-pivot-chart-caches, committed as
//! eval/roundtrip/excel-saved/cases/t4187-pivot-chart-caches.xlsx: its four pivots are
//! `refreshOnLoad`, so the first recalculation renders them and the save regenerates; Excel's own
//! chart parts are what the rewrite gives back (docs/excel-semantics/pivots.md "Excel for the web
//! rewrites a `PivotChart`'s caches when it renders the pivot").

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::io::{Cursor, Read as _};

use support::rezip;

use rustsheet::{LoadOptions, SaveOptions, Workbook};

type Res = Result<(), Box<dyn Error>>;

/// A relationship: (id, type, target part).
type Rel = (String, String, String);

/// A sheet's drawing part and its chart parts.
type Drawing = (String, Vec<String>);

const SAVED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../eval/roundtrip/excel-saved/cases/t4187-pivot-chart-caches.xlsx"
);

/// Each pivot sheet of the evidence, its worksheet part and its drawing's chart part.
const SHEETS: [(&str, &str, &str); 4] = [
    ("C", "xl/worksheets/sheet2.xml", "xl/charts/chart1.xml"),
    ("CS", "xl/worksheets/sheet3.xml", "xl/charts/chart2.xml"),
    ("F", "xl/worksheets/sheet4.xml", "xl/charts/chart3.xml"),
    ("M", "xl/worksheets/sheet5.xml", "xl/charts/chart4.xml"),
];

const DRAWING_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing";

fn entries(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, Box<dyn Error>> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))?;
    let mut out = BTreeMap::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i)?;
        let name = f.name()?.to_string();
        if name.ends_with('/') {
            continue;
        }
        let mut data = Vec::new();
        f.read_to_end(&mut data)?;
        out.insert(name, data);
    }
    Ok(out)
}

fn text(parts: &BTreeMap<String, Vec<u8>>, name: &str) -> Result<String, Box<dyn Error>> {
    let bytes = parts.get(name).ok_or_else(|| format!("no part {name}"))?;
    Ok(String::from_utf8(bytes.clone())?)
}

/// The value of attribute `key` in the element text `element`.
fn attr<'a>(element: &'a str, key: &str) -> Option<&'a str> {
    let at = element.find(&format!(" {key}=\""))? + key.len() + 3;
    element[at..].split('"').next()
}

/// The `<Relationship>` elements of a relationships part: (id, type, target).
fn relationships(xml: &str) -> Vec<Rel> {
    xml.split("<Relationship ")
        .skip(1)
        .filter_map(|r| {
            let r = format!(" {r}");
            Some((
                attr(&r, "Id")?.to_string(),
                attr(&r, "Type")?.to_string(),
                attr(&r, "Target")?.to_string(),
            ))
        })
        .collect()
}

/// `target` from a part in `folder`, `..` resolved.
fn resolve(folder: &str, target: &str) -> String {
    let mut segments: Vec<&str> = folder.split('/').collect();
    for s in target.split('/') {
        if s == ".." {
            segments.pop();
        } else {
            segments.push(s);
        }
    }
    segments.join("/")
}

fn rels_of(part: &str) -> String {
    let (folder, file) = part.rsplit_once('/').unwrap_or(("", part));
    format!("{folder}/_rels/{file}.rels")
}

/// The parts `part` relates to, by type.
fn related(parts: &BTreeMap<String, Vec<u8>>, part: &str) -> Vec<Rel> {
    let Ok(rels) = text(parts, &rels_of(part)) else {
        return Vec::new();
    };
    let folder = part.rsplit_once('/').map_or("", |p| p.0);
    relationships(&rels)
        .into_iter()
        .map(|(id, kind, target)| (id, kind, resolve(folder, &target)))
        .collect()
}

/// Whether the part `name` ends in `.extension`, ignoring case. Not `Path::extension`, which has
/// none for `_rels/.rels` (a file name that is all extension).
fn has_extension(name: &str, extension: &str) -> bool {
    name.rsplit_once('.')
        .is_some_and(|(_, e)| e.eq_ignore_ascii_case(extension))
}

/// The package's structure is sound where the kept drawings touch it: every internal
/// relationship's target exists, every part has a content type and no override names a missing
/// part, and every `r:id` of a worksheet, drawing or chart resolves in its relationships.
fn assert_valid(parts: &BTreeMap<String, Vec<u8>>) -> Res {
    let types = text(parts, "[Content_Types].xml")?;
    let overrides: BTreeSet<String> = types
        .split("<Override ")
        .skip(1)
        .filter_map(|o| {
            attr(&format!(" {o}"), "PartName").map(|p| p.trim_start_matches('/').to_string())
        })
        .collect();
    for part in &overrides {
        assert!(
            parts.contains_key(part),
            "an override names the missing {part}"
        );
    }
    for name in parts.keys() {
        let defaulted = has_extension(name, "rels") || name == "[Content_Types].xml";
        let xml_default = has_extension(name, "xml") && types.contains(r#"Extension="xml""#);
        assert!(
            defaulted || xml_default || overrides.contains(name),
            "{name} has no content type"
        );
        if name.starts_with("xl/drawings/drawing") || name.starts_with("xl/charts/chart") {
            assert!(overrides.contains(name), "{name} has no override");
        }
        if has_extension(name, "rels") {
            continue;
        }
        let rels = related(parts, name);
        for (id, _, target) in &rels {
            assert!(
                parts.contains_key(target),
                "{name} {id} targets the missing {target}"
            );
        }
        if name.starts_with("xl/worksheets/")
            || name.starts_with("xl/drawings/")
            || name.starts_with("xl/charts/")
        {
            let xml = text(parts, name)?;
            for piece in xml.split(" r:id=\"").skip(1) {
                let id = piece.split('"').next().unwrap_or_default();
                assert!(
                    rels.iter().any(|r| r.0 == id),
                    "{name} r:id {id} has no relationship"
                );
            }
        }
    }
    Ok(())
}

/// Loads `bytes`, recalculates (rendering the `refreshOnLoad` pivots) and saves.
fn round_trip(
    bytes: &[u8],
    edit: impl FnOnce(&mut Workbook) -> Res,
) -> Result<BTreeMap<String, Vec<u8>>, Box<dyn Error>> {
    let mut wb = Workbook::from_xlsx_bytes(
        bytes,
        &LoadOptions {
            recalc: false,
            ..LoadOptions::default()
        },
    )?;
    wb.recalculate();
    edit(&mut wb)?;
    entries(&wb.to_xlsx_bytes(SaveOptions::default())?)
}

/// The worksheet part of the sheet `name` in a saved package.
fn sheet_part(parts: &BTreeMap<String, Vec<u8>>, name: &str) -> Result<String, Box<dyn Error>> {
    let workbook = text(parts, "xl/workbook.xml")?;
    let sheet = workbook
        .split("<sheet ")
        .skip(1)
        .map(|s| format!(" {s}"))
        .find(|s| attr(s, "name") == Some(name))
        .ok_or("no such sheet")?;
    let id = attr(&sheet, "r:id").ok_or("no r:id")?.to_string();
    let rels = related(parts, "xl/workbook.xml");
    Ok(rels
        .into_iter()
        .find(|r| r.0 == id)
        .ok_or("no sheet relationship")?
        .2)
}

/// The drawing of the sheet `name` in a saved package and its charts, if it has one.
fn drawing_of(
    parts: &BTreeMap<String, Vec<u8>>,
    name: &str,
) -> Result<Option<Drawing>, Box<dyn Error>> {
    let sheet = sheet_part(parts, name)?;
    let drawings: Vec<_> = related(parts, &sheet)
        .into_iter()
        .filter(|r| r.1 == DRAWING_REL)
        .collect();
    let xml = text(parts, &sheet)?;
    let Some((id, _, drawing)) = drawings.first() else {
        assert!(
            !xml.contains("<drawing "),
            "{name}: a <drawing> with no relationship"
        );
        return Ok(None);
    };
    assert_eq!(drawings.len(), 1, "{name}");
    assert!(
        xml.contains(&format!("<drawing r:id=\"{id}\"/>")),
        "{name}: no <drawing r:id>"
    );
    let charts = related(parts, drawing).into_iter().map(|r| r.2).collect();
    Ok(Some((drawing.clone(), charts)))
}

#[test]
fn a_regenerated_save_keeps_the_pivot_charts_as_excel_saved_them() -> Res {
    let bytes = std::fs::read(SAVED)?;
    let source = entries(&bytes)?;
    let saved = round_trip(&bytes, |_| Ok(()))?;
    assert!(
        !saved.contains_key("xl/calcChain.xml"),
        "the save did not regenerate"
    );
    assert_valid(&saved)?;
    for (sheet, part, chart) in SHEETS {
        let (drawing, charts) =
            drawing_of(&saved, sheet)?.ok_or(format!("{sheet} lost its drawing"))?;
        let source_drawing = related(&source, part)
            .into_iter()
            .find(|r| r.1 == DRAWING_REL)
            .ok_or("no source drawing")?
            .2;
        assert_eq!(
            saved.get(&drawing),
            source.get(&source_drawing),
            "{sheet}: the drawing changed"
        );
        assert_eq!(charts.len(), 1, "{sheet}");
        // Excel's own saved chart part is a fixed point of the rewrite, so the series are Excel's.
        assert_eq!(
            text(&saved, &charts[0])?,
            text(&source, chart)?,
            "{sheet}: {chart}"
        );
    }
    // The other sheets get no drawing.
    for sheet in ["Data", "Q", "RustSheetCanary"] {
        assert!(drawing_of(&saved, sheet)?.is_none(), "{sheet}");
    }
    Ok(())
}

#[test]
fn a_chart_rewritten_from_stale_series_gets_the_pivots_series() -> Res {
    // chart1's caches as the probe's input held them: one stale point each.
    let excel = std::fs::read(SAVED)?;
    let stale = rezip(
        &excel,
        "xl/charts/chart1.xml",
        "<c:pt idx=\"0\"><c:v>18</c:v></c:pt>",
        "<c:pt idx=\"0\"><c:v>999</c:v></c:pt>",
    )?;
    let saved = round_trip(&stale, |_| Ok(()))?;
    let (_, charts) = drawing_of(&saved, "C")?.ok_or("C lost its drawing")?;
    assert_eq!(
        text(&saved, &charts[0])?,
        text(&entries(&excel)?, "xl/charts/chart1.xml")?
    );
    Ok(())
}

/// `bytes` with the entries `added` appended.
fn with_parts(bytes: &[u8], added: &[(&str, &str)]) -> Result<Vec<u8>, Box<dyn Error>> {
    use std::io::Write as _;
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in entries(bytes)?.into_iter().chain(
        added
            .iter()
            .map(|(n, d)| ((*n).to_string(), d.as_bytes().to_vec())),
    ) {
        out.start_file(name, zip::write::SimpleFileOptions::default())?;
        out.write_all(&data)?;
    }
    Ok(out.finish()?.into_inner())
}

#[test]
fn a_charts_style_parts_are_kept_with_it() -> Res {
    const STYLE: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cs:chartStyle xmlns:cs="http://schemas.microsoft.com/office/drawing/2012/chartStyle" id="201"/>"#;
    const COLORS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cs:colorStyle xmlns:cs="http://schemas.microsoft.com/office/drawing/2012/chartStyle" meth="cycle" id="10"/>"#;
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId2" Type="http://schemas.microsoft.com/office/2011/relationships/chartColorStyle" Target="colors7.xml"/><Relationship Id="rId1" Type="http://schemas.microsoft.com/office/2011/relationships/chartStyle" Target="style7.xml"/></Relationships>"#;
    let bytes = with_parts(
        &std::fs::read(SAVED)?,
        &[
            ("xl/charts/_rels/chart4.xml.rels", rels),
            ("xl/charts/style7.xml", STYLE),
            ("xl/charts/colors7.xml", COLORS),
        ],
    )?;
    let saved = round_trip(&bytes, |_| Ok(()))?;
    assert_valid(&saved)?;
    let (_, charts) = drawing_of(&saved, "M")?.ok_or("M lost its drawing")?;
    let related = related(&saved, &charts[0]);
    assert_eq!(related.len(), 2, "{related:?}");
    for (id, _, target) in &related {
        let expected = if id == "rId1" { STYLE } else { COLORS };
        assert_eq!(text(&saved, target)?, expected, "{id}");
    }
    let types = text(&saved, "[Content_Types].xml")?;
    assert!(types.contains("application/vnd.ms-office.chartstyle+xml"));
    assert!(types.contains("application/vnd.ms-office.chartcolorstyle+xml"));
    Ok(())
}

#[test]
fn a_drawing_holding_anything_but_pivot_charts_is_dropped() -> Res {
    let bytes = std::fs::read(SAVED)?;
    // C's drawing also relates to an image; F's chart is not bound to a pivot; M's drawing holds a
    // shape beside its chart.
    let bytes = rezip(
        &bytes,
        "xl/drawings/_rels/drawing1.xml.rels",
        "</Relationships>",
        r#"<Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.png"/></Relationships>"#,
    )?;
    let bytes = rezip(&bytes, "xl/charts/chart3.xml", "<c:pivotSource>", "<c:x>")?;
    let bytes = rezip(&bytes, "xl/charts/chart3.xml", "</c:pivotSource>", "</c:x>")?;
    let bytes = rezip(
        &bytes,
        "xl/drawings/drawing4.xml",
        "<xdr:clientData/></xdr:twoCellAnchor>",
        "<xdr:clientData/></xdr:twoCellAnchor><xdr:absoluteAnchor><xdr:pos x=\"0\" y=\"0\"/><xdr:ext cx=\"1\" cy=\"1\"/><xdr:sp macro=\"\" textlink=\"\"><xdr:nvSpPr><xdr:cNvPr id=\"3\" name=\"S\"/><xdr:cNvSpPr/></xdr:nvSpPr><xdr:spPr/></xdr:sp><xdr:clientData/></xdr:absoluteAnchor>",
    )?;
    let saved = round_trip(&bytes, |_| Ok(()))?;
    assert_valid(&saved)?;
    for sheet in ["C", "F", "M"] {
        assert!(
            drawing_of(&saved, sheet)?.is_none(),
            "{sheet} kept its drawing"
        );
    }
    assert!(drawing_of(&saved, "CS")?.is_some(), "CS lost its drawing");
    Ok(())
}

#[test]
fn a_drawing_on_a_renamed_restructured_or_moved_sheet_is_dropped() -> Res {
    let bytes = std::fs::read(SAVED)?;
    // C renamed, F's rows changed (any change but to cells counts: decision 0098's structural
    // edits are not told apart from the rest).
    let saved = round_trip(&bytes, |wb| {
        let (c, f) = (wb.sheet_id("C")?, wb.sheet_id("F")?);
        wb.edit(|t| {
            t.rename_sheet(c, "C2")?;
            t.set_row_hidden(f, 20, true)
        })?;
        Ok(())
    })?;
    assert_valid(&saved)?;
    assert!(
        drawing_of(&saved, "C2")?.is_none(),
        "the renamed sheet kept its drawing"
    );
    assert!(
        drawing_of(&saved, "F")?.is_none(),
        "the restructured sheet kept its drawing"
    );
    for sheet in ["CS", "M"] {
        assert!(
            drawing_of(&saved, sheet)?.is_some(),
            "{sheet} lost its drawing"
        );
    }
    // Moving M to the front moves every sheet before its old place one on.
    let saved = round_trip(&bytes, |wb| {
        let m = wb.sheet_id("M")?;
        wb.edit(|t| t.move_sheet(m, 0))?;
        Ok(())
    })?;
    assert_valid(&saved)?;
    for (sheet, _, _) in SHEETS {
        assert!(
            drawing_of(&saved, sheet)?.is_none(),
            "{sheet} kept its drawing"
        );
    }
    Ok(())
}
