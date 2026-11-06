//! An OPC package as the round-trip harness sees it (SPEC-0033 D2 `part` items, D6 validator):
//! its parts and content types, its relationship graph, and a key for every part that does not
//! depend on the part's name.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;
use quick_xml::XmlVersion;
use rustsheet::LoadOptions;
use rustsheet_xlsx::import::part_key;
use sha2::{Digest, Sha256};

use crate::xlsx_snapshot::{
    attr, parse_workbook, read_part_bytes_capped, relationships, resolve_target, xml_err, SheetRef,
    SnapshotError, Zip,
};

/// The namespace of `r:id`, `r:embed` and the other relationship-reference attributes.
const REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// Relationship types (last path segment) whose targets are sheets of the workbook: their key
/// is the sheet's name, not an ordinal.
const SHEET_KINDS: &[&str] = &[
    "worksheet",
    "chartsheet",
    "dialogsheet",
    "macrosheet",
    "xlMacrosheet",
];

/// The content-types part, as a normalized part name.
const CONTENT_TYPES: &str = "[content_types].xml";

/// Most parts the harness indexes in one package (directive 5). The importer's own entry limit
/// is the same order of magnitude (`ImportLimits::max_zip_entries`).
const MAX_PARTS: usize = 10_000;

/// Most relationship hops a part key spans (directive 5): keys nest one level per hop, so a
/// chained package must not build ever longer keys. Excel's deepest chains (workbook, sheet,
/// drawing, chart, chart style) are five hops; `GridCraft` caps its closure walk at 16. A part
/// deeper than this is keyed `orphan:` like an unreached one.
const MAX_KEY_DEPTH: usize = 16;

/// One relationship, its target resolved to a normalized part name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Rel {
    pub(crate) id: String,
    /// The type URI's last path segment (`worksheet`, `styles`, ...).
    pub(crate) kind: String,
    /// The normalized target part (`xl/worksheets/sheet1.xml`), or the URI if `external`.
    pub(crate) target: String,
    pub(crate) external: bool,
}

/// A package read for the round trip: nothing in it is trusted.
#[derive(Debug, Default)]
pub(crate) struct Package {
    /// Normalized part name → zip entry name (the first of a clashing group, as the importer
    /// picks it).
    pub(crate) parts: BTreeMap<String, String>,
    /// Extension (ASCII lower case) → content type, from `<Default>`.
    defaults: HashMap<String, String>,
    /// Normalized part name → content type, from `<Override>`.
    overrides: BTreeMap<String, String>,
    /// Source part (normalized; `""` for the package) → its relationships in `.rels` order.
    pub(crate) rels: BTreeMap<String, Vec<Rel>>,
    /// Groups of entries that name one part (V5).
    pub(crate) clashes: Vec<Vec<String>>,
    /// Part → `(attribute local name, relationship id)` of every relationship reference in it,
    /// in document order; XML parts only.
    pub(crate) refs: BTreeMap<String, Vec<(String, String)>>,
    /// XML parts that did not parse, with the error.
    pub(crate) unparsable: BTreeMap<String, String>,
    /// Normalized part name → its key (D2), for every part reached from the package root.
    pub(crate) keys: BTreeMap<String, String>,
    /// The workbook's `<sheet>` entries, as the snapshot reader reads them.
    pub(crate) sheets: Vec<SheetRef>,
    /// The text of every content part whose name ends in `.xml` (D4.1 compares it).
    pub(crate) xml: BTreeMap<String, String>,
    /// The SHA-256 of every other content part (D4.5 compares bytes).
    pub(crate) digests: BTreeMap<String, Vec<u8>>,
}

impl Package {
    /// Reads `bytes` as a package, every part once, through the snapshot reader's bounded part
    /// reader and under one budget for the whole package (`LoadOptions`' default, as the
    /// importer's).
    ///
    /// # Errors
    /// When the bytes are not a zip, the package inflates past its budget or lists more than
    /// [`MAX_PARTS`] parts, or a `.rels` or `[Content_Types].xml` part does not parse.
    pub(crate) fn read(bytes: &[u8]) -> Result<Package, SnapshotError> {
        let mut zip = Zip::new(bytes)?;
        let mut pkg = Package {
            clashes: zip.clashing_entries(),
            ..Package::default()
        };
        for entry in zip.entry_names()? {
            pkg.parts.entry(part_key(&entry)).or_insert(entry);
        }
        if pkg.parts.len() > MAX_PARTS {
            return Err(SnapshotError::Malformed(format!(
                "the package lists more than {MAX_PARTS} parts"
            )));
        }
        // One budget for every byte the package inflates to, as the importer has (directive 5).
        let mut budget = LoadOptions::default().max_uncompressed_bytes;
        let mut take = |zip: &mut Zip, name: &str| -> Result<Option<Vec<u8>>, SnapshotError> {
            let bytes = read_part_bytes_capped(zip, name, budget).map_err(|_| {
                SnapshotError::Malformed(format!(
                    "the package inflates past {budget} more bytes at {name}"
                ))
            })?;
            budget = budget.saturating_sub(bytes.as_ref().map_or(0, |b| b.len() as u64));
            Ok(bytes)
        };
        let text = |bytes: Vec<u8>, name: &str| {
            String::from_utf8(bytes)
                .map_err(|_| SnapshotError::Malformed(format!("part {name} is not UTF-8")))
        };
        if let Some(bytes) = take(&mut zip, "[Content_Types].xml")? {
            pkg.read_content_types(&text(bytes, "[Content_Types].xml")?)?;
        }
        let names: Vec<String> = pkg.parts.keys().cloned().collect();
        for name in names.iter().filter(|n| n.as_str() != CONTENT_TYPES) {
            let Some(bytes) = take(&mut zip, name)? else {
                continue;
            };
            if let Some(source) = rels_source(name) {
                let xml = text(bytes, name)?;
                let dir = source.rsplit_once('/').map_or("", |(d, _)| d);
                let rels = relationships(&xml, name)?
                    .into_iter()
                    .map(|r| Rel {
                        id: r.id,
                        kind: short_kind(&r.kind).to_string(),
                        target: if r.external {
                            r.target
                        } else {
                            part_key(&resolve_target(dir, &r.target))
                        },
                        external: r.external,
                    })
                    .collect();
                pkg.rels.insert(source, rels);
            } else if is_xml_part(name) {
                match String::from_utf8(bytes) {
                    Ok(xml) => {
                        match relationship_refs(&xml) {
                            Ok(refs) => {
                                pkg.refs.insert(name.clone(), refs);
                            }
                            Err(e) => {
                                pkg.unparsable.insert(name.clone(), e);
                            }
                        }
                        pkg.xml.insert(name.clone(), xml);
                    }
                    Err(_) => {
                        pkg.unparsable.insert(name.clone(), "not UTF-8".into());
                    }
                }
            } else {
                pkg.digests
                    .insert(name.clone(), Sha256::digest(&bytes).to_vec());
            }
        }
        if let Some(workbook) = pkg.main_part() {
            if let Some(xml) = pkg.xml.get(&workbook) {
                pkg.sheets = parse_workbook(xml).map(|w| w.sheets).unwrap_or_default();
            }
        }
        pkg.assign_keys();
        Ok(pkg)
    }

    fn read_content_types(&mut self, xml: &str) -> Result<(), SnapshotError> {
        let mut reader = NsReader::from_str(xml);
        loop {
            match reader
                .read_event()
                .map_err(|e| xml_err("[Content_Types].xml", e))?
            {
                Event::Start(e) | Event::Empty(e) => {
                    let ct = attr(&e, "ContentType");
                    match (e.local_name().as_ref(), ct) {
                        ("Default", Some(ct)) => {
                            if let Some(ext) = attr(&e, "Extension") {
                                self.defaults.insert(ext.to_ascii_lowercase(), ct);
                            }
                        }
                        ("Override", Some(ct)) => {
                            if let Some(part) = attr(&e, "PartName") {
                                self.overrides.insert(part_key(&part), ct);
                            }
                        }
                        _ => {}
                    }
                }
                Event::Eof => return Ok(()),
                _ => {}
            }
        }
    }

    /// A part's content type: its `<Override>`, else the `<Default>` of its extension.
    pub(crate) fn content_type(&self, part: &str) -> Option<&str> {
        if let Some(ct) = self.overrides.get(part) {
            return Some(ct);
        }
        let ext = part.rsplit_once('.').map(|(_, e)| e)?;
        self.defaults.get(ext).map(String::as_str)
    }

    /// `<Override>` entries whose part is missing.
    pub(crate) fn dangling_overrides(&self) -> Vec<&str> {
        self.overrides
            .keys()
            .filter(|p| !self.parts.contains_key(*p))
            .map(String::as_str)
            .collect()
    }

    /// The package's main part (the `officeDocument` relationship of `_rels/.rels`).
    pub(crate) fn main_part(&self) -> Option<String> {
        self.rels
            .get("")?
            .iter()
            .find(|r| r.kind == "officeDocument" && !r.external)
            .map(|r| r.target.clone())
    }

    /// Gives every part reached from the package root its key: the parent's key, the
    /// relationship kind, and a label. A sheet of the workbook is labelled by its name; any
    /// other part by its ordinal among the parent's relationships of that kind, ordered by
    /// where the parent first references them, then by relationship id. The key never uses a
    /// part's name, so a writer that renames parts keeps every key (SPEC-0033 D2).
    fn assign_keys(&mut self) {
        let main = self.main_part();
        let mut queue = VecDeque::from([(String::new(), String::new(), 0)]);
        let mut seen = BTreeSet::new();
        while let Some((part, key, depth)) = queue.pop_front() {
            if depth == MAX_KEY_DEPTH {
                continue;
            }
            let Some(rels) = self.rels.get(&part) else {
                continue;
            };
            // Where the parent first references each id: one pass, not one scan per relationship.
            let mut first: HashMap<&str, usize> = HashMap::new();
            for (n, (_, id)) in self.refs.get(&part).into_iter().flatten().enumerate() {
                first.entry(id.as_str()).or_insert(n);
            }
            let first_ref = |id: &str| first.get(id).copied().unwrap_or(usize::MAX);
            let mut by_kind: BTreeMap<&str, Vec<&Rel>> = BTreeMap::new();
            for r in rels.iter().filter(|r| !r.external) {
                by_kind.entry(r.kind.as_str()).or_default().push(r);
            }
            for (kind, mut group) in by_kind {
                group.sort_by(|a, b| (first_ref(&a.id), &a.id).cmp(&(first_ref(&b.id), &b.id)));
                for (n, r) in group.into_iter().enumerate() {
                    let label = (main.as_deref() == Some(part.as_str())
                        && SHEET_KINDS.contains(&kind))
                    .then(|| {
                        self.sheets
                            .iter()
                            .find(|s| s.rid == r.id)
                            .map(|s| s.name.clone())
                    })
                    .flatten()
                    .unwrap_or_else(|| n.to_string());
                    if seen.insert(r.target.clone()) {
                        let child = format!("{key}/{kind}[{label}]");
                        self.keys.insert(r.target.clone(), child.clone());
                        queue.push_back((r.target.clone(), child, depth + 1));
                    }
                }
            }
        }
    }

    /// Every content part (not a `.rels` and not `[Content_Types].xml`) with its key: the D2
    /// key when the part is reached from the root, `orphan:<name>` when it is not.
    pub(crate) fn keyed_parts(&self) -> BTreeMap<String, &str> {
        self.parts
            .keys()
            .filter(|p| rels_source(p).is_none() && p.as_str() != CONTENT_TYPES)
            .map(|p| {
                let key = self
                    .keys
                    .get(p)
                    .cloned()
                    .unwrap_or_else(|| format!("orphan:{p}"));
                (key, p.as_str())
            })
            .collect()
    }
}

/// The source part of a `.rels` part (`""` for the package's own), or `None` for any other part.
fn rels_source(part: &str) -> Option<String> {
    let rest = part.strip_suffix(".rels")?;
    let (dir, name) = rest.rsplit_once('/')?;
    let parent = dir.strip_suffix("_rels")?.trim_end_matches('/');
    Some(if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}/{name}")
    })
}

fn short_kind(uri: &str) -> &str {
    uri.rsplit('/').next().unwrap_or(uri)
}

fn is_xml_part(part: &str) -> bool {
    std::path::Path::new(part)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("xml"))
}

/// Every attribute in the relationships namespace (`r:id`, `r:embed`, ...), in document order,
/// as `(local name, value)`.
fn relationship_refs(xml: &str) -> Result<Vec<(String, String)>, String> {
    let mut reader = NsReader::from_str(xml);
    let mut out = Vec::new();
    loop {
        match reader.read_event().map_err(|e| e.to_string())? {
            Event::Start(e) | Event::Empty(e) => {
                for a in e.attributes() {
                    let a = a.map_err(|e| e.to_string())?;
                    let (ns, local) = reader.resolver().resolve_attribute(a.key);
                    if matches!(ns, ResolveResult::Bound(n) if n.as_ref() == REL_NS) {
                        let value = a
                            .normalized_value(XmlVersion::Implicit1_0)
                            .map_err(|e| e.to_string())?;
                        out.push((local.as_ref().to_string(), value.into_owned()));
                    }
                }
            }
            Event::Eof => return Ok(out),
            _ => {}
        }
    }
}

/// One failed validator check (SPEC-0033 D6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Problem {
    pub(crate) check: &'static str,
    pub(crate) detail: String,
}

/// Runs checks V1–V6 on a package (V7 waits for the semantic model, T-4513; V8, the reload, is
/// the caller's, since it needs the engine).
pub(crate) fn validate(pkg: &Package) -> Vec<Problem> {
    let mut out = Vec::new();
    let mut fail = |check, detail: String| out.push(Problem { check, detail });
    // V1: every part has a content type, and no override names a missing part.
    for part in pkg.parts.keys() {
        if part != CONTENT_TYPES && pkg.content_type(part).is_none() {
            fail("V1", format!("{part} has no content type"));
        }
    }
    for part in pkg.dangling_overrides() {
        fail("V1", format!("an Override names the missing part {part}"));
    }
    for (source, rels) in &pkg.rels {
        let mut ids = BTreeSet::new();
        for r in rels {
            // V2: internal targets exist.
            if !r.external && !pkg.parts.contains_key(&r.target) {
                fail(
                    "V2",
                    format!("{source:?} {} targets the missing part {}", r.id, r.target),
                );
            }
            // V3: ids are unique within one .rels.
            if !ids.insert(r.id.as_str()) {
                fail(
                    "V3",
                    format!("{source:?} repeats the relationship id {}", r.id),
                );
            }
        }
    }
    // V4: every relationship reference resolves in its part's .rels.
    for (part, refs) in &pkg.refs {
        let ids: BTreeSet<&str> = pkg
            .rels
            .get(part)
            .map(|rels| rels.iter().map(|r| r.id.as_str()).collect())
            .unwrap_or_default();
        for (name, id) in refs {
            if !id.is_empty() && !ids.contains(id.as_str()) {
                fail("V4", format!("{part} r:{name}={id:?} has no relationship"));
            }
        }
    }
    for (part, e) in &pkg.unparsable {
        fail("V4", format!("{part} does not parse: {e}"));
    }
    // V5: no two entries name one part.
    for group in &pkg.clashes {
        fail("V5", format!("entries name one part: {}", group.join(", ")));
    }
    validate_sheets(pkg, &mut fail);
    out
}

/// V6: the workbook's sheets have unique ids and names, valid names, and point at sheet parts.
fn validate_sheets(pkg: &Package, fail: &mut impl FnMut(&'static str, String)) {
    let Some(main) = pkg.main_part() else {
        fail(
            "V6",
            "the package has no officeDocument relationship".into(),
        );
        return;
    };
    let rels = pkg.rels.get(&main);
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for SheetRef {
        name,
        rid,
        sheet_id,
    } in &pkg.sheets
    {
        let id = sheet_id.as_deref().unwrap_or_default();
        if !ids.insert(id) {
            fail("V6", format!("sheetId {id:?} repeats"));
        }
        if !names.insert(name.to_lowercase()) {
            fail("V6", format!("sheet name {name:?} repeats"));
        }
        let bad_char = name
            .chars()
            .any(|c| matches!(c, ':' | '\\' | '/' | '?' | '*' | '[' | ']'));
        if name.is_empty() || name.encode_utf16().count() > 31 || bad_char {
            fail(
                "V6",
                format!("sheet name {name:?} is not a valid sheet name"),
            );
        }
        let target = rels.and_then(|rels| rels.iter().find(|r| &r.id == rid));
        if !target.is_some_and(|r| SHEET_KINDS.contains(&r.kind.as_str())) {
            fail(
                "V6",
                format!("sheet {name:?} r:id {rid:?} is not a sheet relationship"),
            );
        }
    }
}
