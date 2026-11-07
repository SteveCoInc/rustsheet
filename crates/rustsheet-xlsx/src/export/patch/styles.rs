//! The styles part of a patched save (SPEC-0034 D3.3): a style table that only grew, every
//! record the source had still at its index, is written by appending the new records to the
//! source's `styles.xml` and updating each section's `count`. Every existing byte is kept, so
//! every `s=` index in the package still names the record it named.

use quick_xml::events::Event;
use quick_xml::Reader;
use rustsheet_engine::types::{Border, CellXfs, Fill, Font, NumFmt, Styles};

use super::super::styles::{cell_xf_xml, font_xml, num_fmt_xml};
use super::super::styles_util::{get_border_xml, get_fill_xml};
use super::tag::{attr_span, set_attr};
use super::MAX_DEPTH;

/// The records the live style table appended to the pristine one, section by section.
pub(super) struct Appended<'a> {
    num_fmts: &'a [NumFmt],
    fonts: &'a [Font],
    fills: &'a [Fill],
    borders: &'a [Border],
    cell_xfs: &'a [CellXfs],
}

/// The live table's tail after the pristine one, when the live table starts with it.
fn tail<'a, T: PartialEq>(pristine: &[T], live: &'a [T]) -> Option<&'a [T]> {
    live.starts_with(pristine)
        .then(|| live.get(pristine.len()..))
        .flatten()
}

/// What `live` appended to `pristine`, or `None` when it is not a prefix-stable extension: a record
/// the source had changed or moved, or a cell style, named style or dxf changed (none of which
/// the patch writes).
pub(super) fn appended<'a>(pristine: &Styles, live: &'a Styles) -> Option<Appended<'a>> {
    let same = live.cell_style_xfs == pristine.cell_style_xfs
        && live.cell_styles == pristine.cell_styles
        && live.dxfs == pristine.dxfs;
    if !same {
        return None;
    }
    Some(Appended {
        num_fmts: tail(&pristine.num_fmts, &live.num_fmts)?,
        fonts: tail(&pristine.fonts, &live.fonts)?,
        fills: tail(&pristine.fills, &live.fills)?,
        borders: tail(&pristine.borders, &live.borders)?,
        cell_xfs: tail(&pristine.cell_xfs, &live.cell_xfs)?,
    })
}

/// One section of `styles.xml`: its record element, the records the source holds, the records
/// to append, already written.
struct Section {
    name: &'static str,
    record: &'static str,
    had: usize,
    new: Vec<String>,
}

/// One section as the source stores it: where its start tag is, where its records end (before
/// `</name>`, or the whole `<name/>` element when empty), how many records it holds, and its
/// `count` when it has one.
struct Found {
    start: (usize, usize),
    end: Option<usize>,
    records: usize,
    count: Option<String>,
}

/// `xml` (the source's `styles.xml`) with `added` appended. `pristine` is the source's table as
/// the importer read it: each section must hold as many records as it has, or the patch would
/// misnumber what it appends.
///
/// # Errors
/// A fallback reason: a section with new records that the source lacks (`numFmts` excepted, which
/// is created), records the importer did not count the same way, a new fill at index 0 or 1, a
/// prefixed or nested root, or a part that does not parse.
pub(super) fn patch(xml: &str, pristine: &Styles, added: &Appended<'_>) -> Result<String, String> {
    // Fills 0 and 1 are the reserved `none` and `gray125` patterns (ECMA-376 Part 1 §18.8.21): a
    // new fill there would not be drawn as written.
    if !added.fills.is_empty() && pristine.fills.len() < 2 {
        return Err("styles.xml has fewer than the two reserved fills".into());
    }
    // quick-xml skips a leading byte-order mark without counting it in its offsets: the scan runs
    // on the text after it, and the mark is put back.
    let (bom, xml) = match xml.strip_prefix('\u{feff}') {
        Some(body) => ("\u{feff}", body),
        None => ("", xml),
    };
    let sections = [
        Section {
            name: "numFmts",
            record: "numFmt",
            had: pristine.num_fmts.len(),
            new: added.num_fmts.iter().map(num_fmt_xml).collect(),
        },
        Section {
            name: "fonts",
            record: "font",
            had: pristine.fonts.len(),
            new: added.fonts.iter().map(font_xml).collect(),
        },
        Section {
            name: "fills",
            record: "fill",
            had: pristine.fills.len(),
            new: added.fills.iter().map(get_fill_xml).collect(),
        },
        Section {
            name: "borders",
            record: "border",
            had: pristine.borders.len(),
            new: added.borders.iter().map(get_border_xml).collect(),
        },
        Section {
            name: "cellXfs",
            record: "xf",
            had: pristine.cell_xfs.len(),
            new: added.cell_xfs.iter().map(cell_xf_xml).collect(),
        },
    ];
    let (root_end, found) = scan(xml, &sections)?;
    // (offset, length to remove, text), applied from the end.
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for (section, at) in sections.iter().zip(&found) {
        if section.new.is_empty() {
            continue;
        }
        let total = section.had + section.new.len();
        let records: String = section.new.concat();
        match at {
            Some(f) => {
                if f.records != section.had {
                    return Err(format!(
                        "styles.xml holds {} {} records, the importer read {}",
                        f.records, section.record, section.had
                    ));
                }
                let (start, end) = f.start;
                let tag = xml
                    .get(start..end)
                    .ok_or("styles.xml: a tag off a char boundary")?;
                match f.end {
                    Some(close) => {
                        edits.push((close, 0, records));
                        edits.push((start, end - start, set_attr(tag, "count", |_| total)));
                    }
                    None => {
                        // `<name .../>`: written out with its records.
                        let open =
                            set_attr(tag.trim_end_matches("/>").trim_end(), "count", |_| total);
                        edits.push((
                            start,
                            end - start,
                            format!("{open}>{records}</{}>", section.name),
                        ));
                    }
                }
            }
            // A workbook with only built-in number formats has no `numFmts`: it is the first
            // child of `styleSheet` (ECMA-376 Part 1 §18.8.39).
            None if section.name == "numFmts" => edits.push((
                root_end,
                0,
                format!(r#"<numFmts count="{total}">{records}</numFmts>"#),
            )),
            None => return Err(format!("styles.xml has no {} to append to", section.name)),
        }
    }
    // From the end, so earlier offsets hold. At one offset a replacement goes first and an
    // insertion after it: a new `numFmts` lands before the start tag it shares an offset with.
    edits.sort_by_key(|e| (std::cmp::Reverse(e.0), std::cmp::Reverse(e.1)));
    let mut out = xml.to_string();
    for (at, remove, text) in edits {
        let end = at + remove;
        if !(out.is_char_boundary(at) && out.is_char_boundary(end)) {
            return Err("styles.xml: an edit off a char boundary".into());
        }
        out.replace_range(at..end, &text);
    }
    // Each section the patch appended to must hold exactly the records it meant to, and say so in
    // its `count` when it has one, or the save regenerates. Sections it left alone are the
    // source's, whatever they hold.
    let (_, again) = scan(&out, &sections)?;
    for (section, at) in sections.iter().zip(&again) {
        if section.new.is_empty() {
            continue;
        }
        let want = section.had + section.new.len();
        let got = at.as_ref().map_or(0, |f| f.records);
        if got != want {
            return Err(format!(
                "the patched styles.xml holds {got} {} records, not {want}",
                section.record
            ));
        }
        let count = at.as_ref().and_then(|f| f.count.as_deref());
        if count.is_some_and(|c| c != want.to_string()) {
            return Err(format!("the patched {} count is not {want}", section.name));
        }
    }
    Ok(format!("{bom}{out}"))
}

/// The root's start tag end, and each section's position, scanning only the root's children.
fn scan(xml: &str, sections: &[Section]) -> Result<(usize, Vec<Option<Found>>), String> {
    let mut reader = Reader::from_str(xml);
    let mut depth = 0usize;
    let mut root_end = None;
    let mut found: Vec<Option<Found>> = sections.iter().map(|_| None).collect();
    let mut open: Option<usize> = None;
    let offset = |p: u64| usize::try_from(p).map_err(|_| "styles.xml is too large".to_string());
    loop {
        let start = offset(reader.buffer_position())?;
        let event = reader
            .read_event()
            .map_err(|e| format!("styles.xml: {e}"))?;
        let end = offset(reader.buffer_position())?;
        match &event {
            Event::Start(e) | Event::Empty(e) => {
                let empty = matches!(event, Event::Empty(_));
                let level = depth + 1;
                let local = e.local_name();
                let local = local.as_ref();
                if level == 1 {
                    if local != "styleSheet" || e.name().as_ref() != local || empty {
                        return Err("styles.xml has no plain styleSheet root".into());
                    }
                    root_end = Some(end);
                } else if level == 2 {
                    if let Some(i) = sections.iter().position(|s| s.name == local) {
                        if e.name().as_ref() != local || found[i].is_some() {
                            return Err(format!("styles.xml has a prefixed or second {local}"));
                        }
                        let tag = xml
                            .get(start..end)
                            .ok_or("styles.xml: a tag off a char boundary")?;
                        found[i] = Some(Found {
                            start: (start, end),
                            end: None,
                            records: 0,
                            count: attr_span(tag, "count").map(|(s, e)| tag[s..e].to_string()),
                        });
                        if !empty {
                            open = Some(i);
                        }
                    }
                } else if level == 3 {
                    if let Some(i) = open {
                        if let Some(f) = found[i].as_mut() {
                            f.records += usize::from(local == sections[i].record);
                        }
                    }
                }
                if !empty {
                    depth = level;
                    if depth > MAX_DEPTH {
                        return Err("styles.xml is nested too deep".into());
                    }
                }
            }
            Event::End(_) => {
                if depth == 2 {
                    if let Some(i) = open.take() {
                        if let Some(f) = found[i].as_mut() {
                            f.end = Some(start);
                        }
                    }
                }
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            _ => {}
        }
    }
    let root_end = root_end.ok_or("styles.xml has no root")?;
    Ok((root_end, found))
}
