//! The package-level edits of a patched save (SPEC-0034 D3.1, D3.4, E2): parts added beside the
//! source's, with their relationships and content-type overrides, and calcChain dropped with its
//! own. Every edit is a splice of whole `<Relationship>` and `<Override>` elements found by the
//! parser; nothing else in either part changes. Anything the splice cannot do exactly (a child
//! written with a start and an end tag, a root of another name, an id or part name already taken,
//! a calcChain relationship pointing elsewhere) is `None`, and the caller regenerates. The zip is
//! read within the import's byte budget and rewritten with every untouched entry copied raw.
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::format_push_string, clippy::match_same_arms)]
// LINT-DEBT-END

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read, Seek, Write};

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::import::ImportLimits;

/// A part to add: its name in the package, its content type and its relationship type from the
/// workbook.
pub(super) struct NewPart<'a> {
    pub(super) name: &'a str,
    pub(super) content_type: &'a str,
    pub(super) relationship: &'a str,
}

/// One empty child element of a relationships or content-types part: the attributes the edits
/// read, and its byte range.
struct Child {
    attrs: BTreeMap<String, String>,
    start: usize,
    end: usize,
}

/// The children of a part's root, and the offset of the root's closing tag.
struct Scan {
    children: Vec<Child>,
    close: usize,
}

/// Reads `xml`, whose root must be named `root`. `None` when it is not, when it does not parse,
/// or when a child is written with a start and an end tag (which the splice would misread).
fn scan(xml: &str, root: &str) -> Option<Scan> {
    let mut reader = Reader::from_str(xml);
    let mut depth = 0usize;
    let mut children = Vec::new();
    loop {
        let start = usize::try_from(reader.buffer_position()).ok()?;
        let event = reader.read_event().ok()?;
        let end = usize::try_from(reader.buffer_position()).ok()?;
        match event {
            Event::Start(e) => {
                depth += 1;
                if depth > 1 || e.local_name().as_ref() != root {
                    return None;
                }
            }
            Event::Empty(e) if depth == 1 => {
                let attrs = e
                    .attributes()
                    .map(|a| {
                        let a = a.ok()?;
                        let value = a
                            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                            .ok()?;
                        Some((a.key.local_name().as_ref().to_string(), value.into_owned()))
                    })
                    .collect::<Option<_>>()?;
                children.push(Child { attrs, start, end });
            }
            // An empty root has nowhere to insert.
            Event::Empty(_) => return None,
            Event::End(_) => {
                return (depth == 1).then_some(Scan {
                    children,
                    close: start,
                });
            }
            Event::Eof => return None,
            _ => {}
        }
    }
}

fn attr<'a>(child: &'a Child, key: &str) -> Option<&'a str> {
    child.attrs.get(key).map(String::as_str)
}

/// `rels` and `types` with a relationship from the workbook and an override for each part.
/// Relationship ids are fresh: `rIdP1`, `rIdP2`, … past any id the part already uses. `None`
/// when either part does not scan, or already names a part or relationship type being added
/// (a stale override or link would be duplicated).
pub(super) fn add_parts(
    rels: &str,
    types: &str,
    parts: &[NewPart<'_>],
) -> Option<(String, String)> {
    let (rel_scan, type_scan) = (scan(rels, "Relationships")?, scan(types, "Types")?);
    let used: BTreeSet<&str> = rel_scan
        .children
        .iter()
        .filter_map(|c| attr(c, "Id"))
        .collect();
    let taken = |part: &NewPart<'_>| {
        let name = format!("/{}", part.name);
        rel_scan
            .children
            .iter()
            .any(|c| attr(c, "Type") == Some(part.relationship))
            || type_scan
                .children
                .iter()
                .any(|c| attr(c, "PartName").is_some_and(|p| p.eq_ignore_ascii_case(&name)))
    };
    if parts.iter().any(taken) {
        return None;
    }
    let mut next = 1usize;
    let mut new_rels = String::new();
    let mut new_types = String::new();
    for part in parts {
        let id = loop {
            let id = format!("rIdP{next}");
            next += 1;
            if !used.contains(id.as_str()) {
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
        splice(rels, &[], rel_scan.close, &new_rels),
        splice(types, &[], type_scan.close, &new_types),
    ))
}

/// `rels` without the workbook's relationships whose type ends with `/{kind}` (the transitional
/// and the strict namespaces alike), and `types` without the override for `part`. `None` when either part does not scan, or a relationship of that type is
/// external or targets anything but `part` (dropping it would leave its real target behind).
pub(super) fn remove_part(
    rels: &str,
    types: &str,
    kind: &str,
    part: &str,
) -> Option<(String, String)> {
    let (rel_scan, type_scan) = (scan(rels, "Relationships")?, scan(types, "Types")?);
    let suffix = format!("/{kind}");
    let relative = part.strip_prefix("xl/")?;
    let absolute = format!("/{part}");
    let mut rel_cuts = Vec::new();
    for c in &rel_scan.children {
        if !attr(c, "Type").is_some_and(|t| t.ends_with(&suffix)) {
            continue;
        }
        let target = attr(c, "Target");
        let external = attr(c, "TargetMode") == Some("External");
        if external || !(target == Some(relative) || target == Some(absolute.as_str())) {
            return None;
        }
        rel_cuts.push((c.start, c.end));
    }
    let type_cuts: Vec<(usize, usize)> = type_scan
        .children
        .iter()
        .filter(|c| attr(c, "PartName").is_some_and(|p| p.eq_ignore_ascii_case(&absolute)))
        .map(|c| (c.start, c.end))
        .collect();
    Some((
        splice(rels, &rel_cuts, rel_scan.close, ""),
        splice(types, &type_cuts, type_scan.close, ""),
    ))
}

/// `xml` with the sorted, disjoint `cuts` removed and `insert` written at `at` (after every cut,
/// which all lie inside the root), in one pass.
fn splice(xml: &str, cuts: &[(usize, usize)], at: usize, insert: &str) -> String {
    let mut out = String::with_capacity(xml.len() + insert.len());
    let mut from = 0;
    for &(start, end) in cuts {
        out.push_str(&xml[from..start]);
        from = end;
    }
    out.push_str(&xml[from..at]);
    out.push_str(insert);
    out.push_str(&xml[at..]);
    out
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

    const CALC: &str = "calcChain";
    const RELS: &str = r#"<Relationships xmlns="r"><Relationship Id="rId1" Type="t/worksheet" Target="worksheets/sheet1.xml"/><Relationship Id="rIdP1" Type="t/x" Target="x.xml"/><Relationship Id="rId9" Type="http://x/calcChain" Target="calcChain.xml"/></Relationships>"#;
    const TYPES: &str = r#"<Types xmlns="c"><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/calcChain.xml" ContentType="cc"/></Types>"#;
    const META: NewPart<'static> = NewPart {
        name: "xl/metadata.xml",
        content_type: "md",
        relationship: "http://x/sheetMetadata",
    };

    #[test]
    fn added_parts_get_fresh_ids_and_overrides_before_the_real_closing_tag() {
        // A comment holding a closing tag is not where new elements go.
        let rels = RELS.replace(
            "</Relationships>",
            "</Relationships><!-- </Relationships> -->",
        );
        let (rels, types) = add_parts(&rels, TYPES, &[META]).expect("adds");
        assert!(
            rels.ends_with(r#"<Relationship Id="rIdP2" Type="http://x/sheetMetadata" Target="metadata.xml"/></Relationships><!-- </Relationships> -->"#),
            "{rels}"
        );
        assert!(
            types.ends_with(r#"<Override PartName="/xl/metadata.xml" ContentType="md"/></Types>"#),
            "{types}"
        );
    }

    #[test]
    fn a_taken_name_or_type_or_a_long_form_child_refuses_to_add() {
        let stale = TYPES.replace(
            "</Types>",
            r#"<Override PartName="/XL/metadata.xml" ContentType="md"/></Types>"#,
        );
        assert!(add_parts(RELS, &stale, &[META]).is_none());
        let linked = RELS.replace(
            "</Relationships>",
            r#"<Relationship Id="m" Type="http://x/sheetMetadata" Target="other.xml"/></Relationships>"#,
        );
        assert!(add_parts(&linked, TYPES, &[META]).is_none());
        let long = RELS.replace(r#"Target="x.xml"/>"#, r#"Target="x.xml"></Relationship>"#);
        assert!(add_parts(&long, TYPES, &[META]).is_none());
        assert!(add_parts(RELS, "<Other/>", &[META]).is_none());
    }

    #[test]
    fn a_removed_part_loses_its_relationship_and_override_only() {
        let (rels, types) = remove_part(RELS, TYPES, CALC, "xl/calcChain.xml").expect("removes");
        assert!(
            !rels.contains("calcChain") && rels.contains(r#"Id="rIdP1""#),
            "{rels}"
        );
        assert_eq!(
            types,
            r#"<Types xmlns="c"><Default Extension="xml" ContentType="application/xml"/></Types>"#
        );
        // A calcChain relationship that points elsewhere, or out of the package, is refused.
        let elsewhere = RELS.replace(r#"Target="calcChain.xml""#, r#"Target="chain2.xml""#);
        assert!(remove_part(&elsewhere, TYPES, CALC, "xl/calcChain.xml").is_none());
        let external = RELS.replace(
            r#"Target="calcChain.xml""#,
            r#"Target="calcChain.xml" TargetMode="External""#,
        );
        assert!(remove_part(&external, TYPES, CALC, "xl/calcChain.xml").is_none());
    }
}
