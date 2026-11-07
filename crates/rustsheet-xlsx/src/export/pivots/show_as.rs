//! A data field's x14 show-values-as mode on export (T-4759). The model's mode is the truth: the
//! kept `<dataField>` elements are written back verbatim only when the x14 `pivotShowAs` they name
//! (as the importer reads it, [`data_field_x14_show_as`]) is the model's x14 mode, or both name
//! none. Otherwise the kept `extLst` is rewritten so it does: the first x14 `dataField` takes the
//! mode as its `pivotShowAs` (or loses it, for an ECMA mode or `normal`, which `showDataAs`
//! names), any other loses its own, and a list with no x14 `dataField` gains the extension Excel
//! writes (pivots.md "Show-values-as modes as Excel for the web renders them",
//! [`ShowDataAs::x14_data_field_extension`]) as its last child. A field with no kept list gets a
//! list of its own, `CT_DataField`'s only child. Merging into a kept x14 `dataField` keeps one
//! extension of that uri per list, never two.
//!
//! Removing a stale mode can leave an attribute-less `<x14:dataField/>`, and a prefixed kept list
//! gets a prefixed `<x:ext>`: neither is seen in an Excel-saved file (UNVERIFIED, T-4948).
//!
//! Kept XML the rewrite cannot read to its end (built by hand through the API, as the importer
//! keeps only what it read whole) is written back as kept, and nothing is added: guessing inside
//! it could make the part malformed.

use std::borrow::Cow;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use rustsheet_engine::pivot::{element_name, ShowDataAs};

use crate::export::patch::rewrite_tag;
use crate::import::data_field_x14_show_as;

/// The x14 attribute the mode is.
const SHOW_AS: &str = "pivotShowAs";

/// A `<dataField>`'s elements: `kept`, holding the model's `mode` as described in the module docs.
pub(super) fn data_field_elements<'a>(mode: ShowDataAs, kept: &[&'a str]) -> Vec<Cow<'a, str>> {
    let mut elements: Vec<Cow<'a, str>> = kept.iter().copied().map(Cow::Borrowed).collect();
    let wanted = mode.is_x14().then_some(mode);
    let mut named = None;
    for element in kept {
        match data_field_x14_show_as(element) {
            Ok(Some(mode)) => named = Some(mode),
            Ok(None) => {}
            // Unreadable: as kept, nothing added.
            Err(_) => return elements,
        }
    }
    if named == wanted {
        return elements;
    }
    let mut lists = (elements.iter_mut()).filter(|element| element_name(element) == "extLst");
    match (lists.next(), lists.next()) {
        (Some(list), None) => {
            if let Some(xml) = with_show_as(list, wanted) {
                *list = Cow::Owned(xml);
            }
        }
        (None, _) => {
            if let Some(extension) = mode.x14_data_field_extension("") {
                elements.push(Cow::Owned(format!("<extLst>{extension}</extLst>")));
            }
        }
        // Two lists, which no reader of the part keeps: as kept.
        (Some(_), Some(_)) => {}
    }
    elements
}

/// The kept `extLst` element `list` naming `wanted` as its x14 `pivotShowAs` (none for `None`),
/// every other byte as kept; `None` when it cannot be read to its end, is not one element, or does
/// not read back as `wanted`.
fn with_show_as(list: &str, wanted: Option<ShowDataAs>) -> Option<String> {
    let mut reader = Reader::from_str(list);
    let mut out = String::with_capacity(list.len() + 256);
    let mut depth = 0usize;
    let mut in_ext = false;
    let mut placed = wanted.is_none();
    // The list's qualified name, once its start tag is read; `closed` once its end is.
    let mut root: Option<String> = None;
    let mut closed = false;
    loop {
        let start = usize::try_from(reader.buffer_position()).ok()?;
        let event = reader.read_event().ok()?;
        let end = usize::try_from(reader.buffer_position()).ok()?;
        let source = list.get(start..end)?;
        match &event {
            Event::Start(e) | Event::Empty(e) => {
                let empty = matches!(event, Event::Empty(_));
                let level = depth + 1;
                if level == 1 {
                    if root.is_some() {
                        return None;
                    }
                    let name = e.name().as_ref().to_string();
                    if empty {
                        // `<extLst/>`: reopened around the extension, under its own name.
                        closed = true;
                        if let Some(extension) = extension(wanted, placed, &name) {
                            let open = source.strip_suffix("/>")?.trim_end();
                            out.push_str(&format!("{open}>{extension}</{name}>"));
                            placed = true;
                            root = Some(name);
                            continue;
                        }
                    }
                    root = Some(name);
                } else if level == 2 {
                    in_ext = e.local_name().as_ref() == "ext";
                } else if level == 3 && in_ext && e.local_name().as_ref() == "dataField" {
                    let value = if placed {
                        None
                    } else {
                        wanted.map(ShowDataAs::name)
                    };
                    if has_show_as(e)? || value.is_some() {
                        let name = e.name().as_ref().to_string();
                        out.push_str(&rewrite_tag(e, &name, &[(SHOW_AS, value)])?);
                        out.push_str(if empty { "/>" } else { ">" });
                        placed = true;
                        if !empty {
                            depth = level;
                        }
                        continue;
                    }
                }
                if !empty {
                    depth = level;
                }
            }
            Event::End(_) => {
                if depth == 1 {
                    let name = root.as_deref()?;
                    if let Some(extension) = extension(wanted, placed, name) {
                        out.push_str(&extension);
                        placed = true;
                    }
                    closed = true;
                }
                depth = depth.checked_sub(1)?;
                if depth == 1 {
                    in_ext = false;
                }
            }
            Event::Eof => break,
            _ => {}
        }
        out.push_str(source);
    }
    let reads_back = data_field_x14_show_as(&out).ok()? == wanted;
    (closed && depth == 0 && placed && reads_back).then_some(out)
}

/// The extension still to add to the list named `list` (qualified), in the list's namespace.
fn extension(wanted: Option<ShowDataAs>, placed: bool, list: &str) -> Option<String> {
    let prefix = list.rsplit_once(':').map_or("", |(prefix, _)| prefix);
    let prefix = if prefix.is_empty() {
        String::new()
    } else {
        format!("{prefix}:")
    };
    wanted
        .filter(|_| !placed)?
        .x14_data_field_extension(&prefix)
}

/// Whether `e` has a `pivotShowAs`; `None` for a malformed attribute or a prefixed one, which the
/// importer reads by its local name but [`rewrite_tag`] would not rewrite.
fn has_show_as(e: &BytesStart<'_>) -> Option<bool> {
    let mut has = false;
    for attribute in e.attributes() {
        let attribute = attribute.ok()?;
        if attribute.key.local_name().as_ref() == SHOW_AS {
            if attribute.key.as_ref() != SHOW_AS {
                return None;
            }
            has = true;
        }
    }
    Some(has)
}
