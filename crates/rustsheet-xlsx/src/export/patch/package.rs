//! The package-level edits of a patched save (SPEC-0034 D3.4, D3.5, E2): parts added beside the
//! source's, with their relationships and content-type overrides, and calcChain dropped with its
//! own. Every edit is a splice of whole `<Relationship>` and `<Override>` elements; nothing else in
//! either part changes. The zip is read within the import's byte budget and rewritten with every
//! untouched entry copied raw.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read, Seek, Write};

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::import::ImportLimits;

/// The workbook's relationships part, as the importer reads it.
pub(super) const WORKBOOK_RELS: &str = "xl/_rels/workbook.xml.rels";
/// The package's content types.
pub(super) const CONTENT_TYPES: &str = "[Content_Types].xml";

/// A part to add: its name in the package, its content type and its relationship type from the
/// workbook.
pub(super) struct NewPart<'a> {
    pub(super) name: &'a str,
    pub(super) content_type: &'a str,
    pub(super) relationship: &'a str,
}

/// `rels` and `types` with a relationship from the workbook and an override for each part.
/// Relationship ids are fresh: `rIdP1`, `rIdP2`, … past any id the part already uses. `None`
/// when either part has no closing tag.
pub(super) fn add_parts(
    rels: &str,
    types: &str,
    parts: &[NewPart<'_>],
) -> Option<(String, String)> {
    let used = ids(rels)?;
    let mut next = 1usize;
    let mut new_rels = String::new();
    let mut new_types = String::new();
    for part in parts {
        let id = loop {
            let id = format!("rIdP{next}");
            next += 1;
            if !used.contains(&id) {
                break id;
            }
        };
        let target = part.name.strip_prefix("xl/")?;
        new_rels.push_str(&format!(
            r#"<Relationship Id="{id}" Type="{}" Target="{target}"/>"#,
            part.relationship
        ));
        new_types.push_str(&format!(
            r#"<Override PartName="/{}" ContentType="{}"/>"#,
            part.name, part.content_type
        ));
    }
    Some((
        insert_before(rels, "</Relationships>", &new_rels)?,
        insert_before(types, "</Types>", &new_types)?,
    ))
}

/// `rels` without the relationships whose `Type` ends with `/{kind}`, and `types` without the
/// override for `part`. `None` when either part does not parse.
pub(super) fn remove_part(
    rels: &str,
    types: &str,
    kind: &str,
    part: &str,
) -> Option<(String, String)> {
    let suffix = format!("/{kind}");
    let override_name = format!("/{part}");
    let rels = without(rels, "Relationship", |attr| {
        attr("Type").is_some_and(|t| t.ends_with(&suffix))
    })?;
    let types = without(types, "Override", |attr| {
        attr("PartName").is_some_and(|p| p.eq_ignore_ascii_case(&override_name))
    })?;
    Some((rels, types))
}

/// The `Id`s of the relationships in `rels`.
fn ids(rels: &str) -> Option<Vec<String>> {
    let mut out = Vec::new();
    for_each_element(rels, |name, attr, _, _| {
        if name == "Relationship" {
            if let Some(id) = attr("Id") {
                out.push(id);
            }
        }
    })?;
    Some(out)
}

/// `xml` with every empty `<local …/>` element that `drop` selects cut out.
fn without(
    xml: &str,
    local: &str,
    drop: impl Fn(&dyn Fn(&str) -> Option<String>) -> bool,
) -> Option<String> {
    let mut cuts = Vec::new();
    for_each_element(xml, |name, attr, start, end| {
        if name == local && drop(attr) {
            cuts.push((start, end));
        }
    })?;
    let mut out = xml.to_string();
    for (start, end) in cuts.into_iter().rev() {
        out.replace_range(start..end, "");
    }
    Some(out)
}

/// Calls `f` with each empty element's local name, an attribute lookup and its byte range.
fn for_each_element(
    xml: &str,
    mut f: impl FnMut(&str, &dyn Fn(&str) -> Option<String>, usize, usize),
) -> Option<()> {
    let mut reader = Reader::from_str(xml);
    loop {
        let start = usize::try_from(reader.buffer_position()).ok()?;
        let event = reader.read_event().ok()?;
        let end = usize::try_from(reader.buffer_position()).ok()?;
        match event {
            Event::Empty(e) => {
                let local = e.local_name();
                let attr = |key: &str| super::attr(&e, key);
                f(local.as_ref(), &attr, start, end);
            }
            Event::Eof => return Some(()),
            _ => {}
        }
    }
}

/// `xml` with `text` inserted before its last `close` tag.
fn insert_before(xml: &str, close: &str, text: &str) -> Option<String> {
    let at = xml.rfind(close)?;
    let mut out = String::with_capacity(xml.len() + text.len());
    out.push_str(&xml[..at]);
    out.push_str(text);
    out.push_str(&xml[at..]);
    Some(out)
}

/// A part's text, within the import's byte budget: a larger part, a missing one or one that is
/// not UTF-8 is a fallback reason, never a truncated read.
pub(super) fn read_entry<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<String, String> {
    let limit = ImportLimits::default().max_uncompressed_bytes;
    let mut bytes = Vec::new();
    archive
        .by_name(name)
        .map_err(|e| format!("{name}: {e}"))?
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|e| format!("{name}: {e}"))?;
    if u64::try_from(bytes.len()).map_or(true, |n| n > limit) {
        return Err(format!("{name} is over the size limit"));
    }
    String::from_utf8(bytes).map_err(|_| format!("{name} is not UTF-8"))
}

/// The source package with the `replaced` entries' contents swapped in and every other entry
/// copied raw, so it keeps its exact compressed bytes, less the `removed` entries and with the
/// `added` ones last. A repeated or non-UTF-8 entry name is a fallback reason.
pub(super) fn rewrite_zip<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    replaced: &BTreeMap<String, Vec<u8>>,
    removed: &BTreeSet<String>,
    added: &[(String, Vec<u8>)],
) -> Result<Vec<u8>, String> {
    let zip_err = |e: zip::result::ZipError| format!("rewriting the zip: {e}");
    let io_err = |e: std::io::Error| format!("rewriting the zip: {e}");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut seen = BTreeSet::new();
    for i in 0..archive.len() {
        let file = archive.by_index_raw(i).map_err(zip_err)?;
        let name = file.name().map_err(zip_err)?.to_string();
        if !seen.insert(name.clone()) {
            return Err(format!("the zip holds {name} twice"));
        }
        if removed.contains(&name) {
            continue;
        }
        if let Some(bytes) = replaced.get(&name) {
            drop(file);
            out.start_file(name, zip::write::SimpleFileOptions::default())
                .map_err(zip_err)?;
            out.write_all(bytes).map_err(io_err)?;
        } else {
            out.raw_copy_file(file).map_err(zip_err)?;
        }
    }
    for (name, bytes) in added {
        if !seen.insert(name.clone()) {
            return Err(format!("the zip already holds {name}"));
        }
        out.start_file(name.as_str(), zip::write::SimpleFileOptions::default())
            .map_err(zip_err)?;
        out.write_all(bytes).map_err(io_err)?;
    }
    Ok(out.finish().map_err(zip_err)?.into_inner())
}

#[cfg(test)]
mod tests {
    use super::{add_parts, remove_part, NewPart};

    const RELS: &str = r#"<Relationships xmlns="r"><Relationship Id="rId1" Type="t/worksheet" Target="worksheets/sheet1.xml"/><Relationship Id="rIdP1" Type="t/x" Target="x.xml"/><Relationship Id="rId9" Type="http://x/calcChain" Target="calcChain.xml"/></Relationships>"#;
    const TYPES: &str = r#"<Types xmlns="c"><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/calcChain.xml" ContentType="cc"/></Types>"#;

    #[test]
    fn added_parts_get_fresh_ids_and_overrides() {
        let part = NewPart {
            name: "xl/metadata.xml",
            content_type: "md",
            relationship: "http://x/sheetMetadata",
        };
        let (rels, types) = add_parts(RELS, TYPES, &[part]).expect("adds");
        assert!(rels.ends_with(r#"<Relationship Id="rIdP2" Type="http://x/sheetMetadata" Target="metadata.xml"/></Relationships>"#), "{rels}");
        assert!(
            types.ends_with(r#"<Override PartName="/xl/metadata.xml" ContentType="md"/></Types>"#),
            "{types}"
        );
    }

    #[test]
    fn a_removed_part_loses_its_relationship_and_override_only() {
        let (rels, types) =
            remove_part(RELS, TYPES, "calcChain", "xl/calcChain.xml").expect("removes");
        assert!(
            !rels.contains("calcChain") && rels.contains(r#"Id="rIdP1""#),
            "{rels}"
        );
        assert_eq!(
            types,
            r#"<Types xmlns="c"><Default Extension="xml" ContentType="application/xml"/></Types>"#
        );
    }
}
