//! The structural XML diff of SPEC-0033 D4: `element` and `attribute` fidelity items for every
//! matched XML part pair, so content no feature model knows (unknown `extLst` entries, unknown
//! elements and attributes, new Excel features) is scored, not ignored.
//!
//! Names compare by namespace URI and local name, never by prefix (D4.1). Children compare in
//! order, except at the unordered paths of [`UNORDERED`] and inside `extLst`, which match by key
//! (D4.2). An absent attribute equals its schema default ([`DEFAULTS`], D4.3), and xsd:boolean
//! spellings compare as booleans. Candidate normalizations are tagged, never applied (D4.4).
//! Subtrees the semantic model scores (T-4513) are compared only as present or absent.

use std::collections::{BTreeMap, BTreeSet};

use quick_xml::events::{BytesStart, Event};
use quick_xml::name::{QName, ResolveResult};
use quick_xml::{NsReader, XmlVersion};

use super::Verdict;
use crate::xlsx_snapshot::resolve_entity;

/// The SpreadsheetML main namespace, written without a prefix in item keys.
const MAIN: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
const MC: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";
const XMLNS: &str = "http://www.w3.org/2000/xmlns/";

/// Short names for the namespaces item keys mention most; any other namespace is written as
/// `{uri}`. Keys only: comparison always uses the URI.
const PREFIXES: &[(&str, &str)] = &[
    (
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        "r",
    ),
    (MC, "mc"),
    (
        "http://schemas.microsoft.com/office/spreadsheetml/2009/9/main",
        "x14",
    ),
    (
        "http://schemas.microsoft.com/office/spreadsheetml/2010/11/main",
        "x15",
    ),
    (
        "http://schemas.microsoft.com/office/spreadsheetml/2009/9/ac",
        "x14ac",
    ),
    (
        "http://schemas.microsoft.com/office/spreadsheetml/2014/revision",
        "xr",
    ),
    ("http://schemas.microsoft.com/office/excel/2006/main", "xm"),
    ("http://schemas.openxmlformats.org/drawingml/2006/main", "a"),
    ("http://purl.org/dc/elements/1.1/", "dc"),
    ("http://purl.org/dc/terms/", "dcterms"),
    (
        "http://schemas.openxmlformats.org/package/2006/metadata/core-properties",
        "cp",
    ),
    (
        "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties",
        "ep",
    ),
    (
        "http://schemas.openxmlformats.org/package/2006/content-types",
        "ct",
    ),
];

/// Element paths (from the part root, local names) whose children the semantic model of
/// SPEC-0033 D3 scores (T-4513): the diff checks the element itself, not its children.
const SEMANTIC: &[&str] = &["/worksheet/sheetData", "/sst"];

/// Unordered collections (D4.2): `(parent path, child local name, key attributes)`. Exactly
/// SPEC-0033 D3.2's collections: merge ranges, hyperlinks, data-validation and conditional-format
/// rules, and defined names, which match by their natural key. `extLst` children always match by
/// `uri` (`CT_ExtensionList`). Any other collection compares in order, so a writer that reorders
/// one is seen.
const UNORDERED: &[(&str, &str, &[&str])] = &[
    // CT_MergeCells, SPEC-0033 D3.2 merge ranges
    ("/worksheet/mergeCells", "mergeCell", &["ref"]),
    // CT_Hyperlinks, D3.2 hyperlinks
    ("/worksheet/hyperlinks", "hyperlink", &["ref"]),
    // CT_DataValidations, D3.2 DV rules
    ("/worksheet/dataValidations", "dataValidation", &["sqref"]),
    // CT_ConditionalFormatting, D3.2 CF rules
    ("/worksheet", "conditionalFormatting", &["sqref"]),
    // CT_DefinedNames, D3.2 defined names
    (
        "/workbook/definedNames",
        "definedName",
        &["name", "localSheetId"],
    ),
];

/// Schema defaults (D4.3): `(element local name, "attribute=default ...")`. Rows are keyed by local
/// name in the SpreadsheetML main namespace only. Each row was checked to mean the same type
/// wherever that name occurs in a part the differ reads; a row whose name has a second meaning
/// (`sheetView` is also `CT_ChartsheetView`, `i` is also the pivot `CT_I`) must stay correct for
/// both, or key on the parent path. An absent attribute
/// equals its default. Each is the `default=` of the attribute in ECMA-376 Part 1's sml.xsd:
/// `CT_Cell` (`t`, `s`), `CT_Row`, `CT_Col`, `CT_SheetView`, `CT_Pane`, `CT_Selection`, `CT_Sheet`,
/// `CT_BookView`, `CT_SheetFormatPr` and `CT_BooleanProperty` (`val`, as in `<b/>`).
const DEFAULTS: &[(&str, &str)] = &[
    // CT_Cell
    ("c", "t=n s=0"),
    // CT_Row
    (
        "row",
        "hidden=false customHeight=false customFormat=false outlineLevel=0 collapsed=false",
    ),
    // CT_Col
    (
        "col",
        "hidden=false customWidth=false bestFit=false outlineLevel=0 collapsed=false style=0",
    ),
    // CT_SheetView
    (
        "sheetView",
        "tabSelected=false showGridLines=true rightToLeft=false zoomScale=100 view=normal",
    ),
    // CT_Pane
    ("pane", "xSplit=0 ySplit=0 activePane=topLeft state=split"),
    // CT_Selection
    ("selection", "pane=topLeft activeCellId=0"),
    // CT_Sheet
    ("sheet", "state=visible"),
    // CT_BookView
    (
        "workbookView",
        "activeTab=0 firstSheet=0 visibility=visible",
    ),
    // CT_SheetFormatPr
    (
        "sheetFormatPr",
        "customHeight=false zeroHeight=false outlineLevelRow=0 outlineLevelCol=0",
    ),
    // CT_BooleanProperty
    ("b", "val=true"),
    // CT_BooleanProperty
    ("i", "val=true"),
    // CT_BooleanProperty
    ("strike", "val=true"),
];

/// Deepest element nesting a part may have (directive 5); deeper parts are reported as unparsable.
const MAX_DEPTH: usize = 256;

/// Most elements kept per part (directive 5). Subtrees under [`SEMANTIC`] are only counted.
const MAX_NODES: usize = 1_000_000;

/// Longest element or attribute local name, and longest namespace URI, a part may use (directive
/// 5): names are copied into every node and path, so their length must be bounded, not only
/// their count. Excel's longest names are a few dozen bytes.
const MAX_NAME: usize = 256;
const MAX_NAMESPACE: usize = 512;

/// Longest element path (`/worksheet/sheetViews/sheetView`, local names) a part may nest to.
const MAX_PATH: usize = 4096;

/// Most items one part pair reports; the rest are counted in one `truncated` item.
pub(crate) const MAX_ITEMS: usize = 500;

/// A namespace-resolved name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Name {
    ns: String,
    local: String,
}

impl Name {
    fn label(&self) -> String {
        if self.ns.is_empty() || self.ns == MAIN {
            return self.local.clone();
        }
        match PREFIXES.iter().find(|(uri, _)| *uri == self.ns) {
            Some((_, p)) => format!("{p}:{}", self.local),
            None => format!("{{{}}}{}", self.ns, self.local),
        }
    }
}

/// One element of a parsed part.
#[derive(Debug, Clone)]
struct Node {
    name: Name,
    attrs: BTreeMap<Name, String>,
    children: Vec<Node>,
    text: String,
    /// Elements below this one that were counted, not kept ([`SEMANTIC`]).
    skipped: usize,
}

impl Node {
    fn size(&self) -> usize {
        1 + self.skipped + self.children.iter().map(Node::size).sum::<usize>()
    }

    /// The source items the subtree holds as [`compare`] counts them: each element, its
    /// attributes, and the semantic-owned elements below it.
    fn items(&self) -> usize {
        1 + self.attrs.len() + self.skipped + self.children.iter().map(Node::items).sum::<usize>()
    }
}

/// One difference: the item's path, what it is, and its verdict label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Difference {
    /// `element` or `attribute`.
    pub(crate) kind: &'static str,
    /// The element path (`/worksheet/sheetViews/sheetView[0]`), with `@attr` for an attribute.
    pub(crate) path: String,
    /// `Lost`, `Added` or `Changed`.
    pub(crate) verdict: Verdict,
    pub(crate) detail: Option<String>,
    /// Source items the difference covers that were not kept: the subtree's size for a lost
    /// element, 1 for a change or a lost attribute, 0 for an addition.
    pub(crate) weight: usize,
}

/// The result of diffing one part pair.
#[derive(Debug, Default)]
pub(crate) struct PartDiff {
    /// Source elements and attributes compared (the case's checked count, SPEC-0033 D8).
    pub(crate) checked: usize,
    pub(crate) differences: Vec<Difference>,
    /// Differences beyond [`MAX_ITEMS`].
    pub(crate) truncated: usize,
    /// The source items those truncated differences cover ([`Difference::weight`]).
    pub(crate) truncated_weight: usize,
}

/// Diffs two XML parts. A part that does not parse is `Err` (the validator reports the output's).
///
/// # Errors
/// When either part is not well-formed XML or exceeds [`MAX_DEPTH`] or [`MAX_NODES`].
pub(crate) fn diff(source: &str, output: &str) -> Result<PartDiff, String> {
    let a = parse(source)?;
    let b = parse(output)?;
    let mut d = PartDiff::default();
    if a.name == b.name {
        let mut path = format!("/{}", a.name.local);
        let mut local = path.clone();
        compare(&a, &b, &mut path, &mut local, &mut d);
    } else {
        d.checked = a.items();
        let mut root = element(
            format!("/{}", a.name.label()),
            Verdict::Changed,
            Some(format!("root became {}", b.name.label())),
        );
        root.weight = a.items();
        push(&mut d, root);
    }
    Ok(d)
}

/// At most 200 characters of `text`, for a detail line.
pub(crate) fn clip(text: &str) -> String {
    match text.char_indices().nth(200) {
        Some((at, _)) => format!("{}…", &text[..at]),
        None => text.to_string(),
    }
}

fn element(path: String, verdict: Verdict, detail: Option<String>) -> Difference {
    Difference {
        kind: "element",
        path,
        verdict,
        detail,
        weight: unit_weight(verdict),
    }
}

/// The weight of a difference that covers one source item at most: 1 unless nothing of the
/// source is missing.
pub(crate) fn unit_weight(verdict: Verdict) -> usize {
    usize::from(matches!(verdict, Verdict::Changed | Verdict::Lost))
}

fn push(d: &mut PartDiff, diff: Difference) {
    if d.differences.len() < MAX_ITEMS {
        d.differences.push(diff);
    } else {
        d.truncated += 1;
        d.truncated_weight += diff.weight;
    }
}

/// The source items of a well-formed part (elements and attributes), as [`diff`] counts a lost
/// subtree.
///
/// # Errors
/// As [`diff`]'s parse.
pub(crate) fn element_count(xml: &str) -> Result<usize, String> {
    parse(xml).map(|n| n.items())
}

/// Compares two matched elements at `path` (the item path, with ordinals and keys) and `local`
/// (the same path with local names only, which the unordered table matches): attributes, text,
/// then children. Both are buffers each level extends and truncates again, so building them costs
/// one segment per element, not a whole path (directive 5).
fn compare(a: &Node, b: &Node, path: &mut String, local: &mut String, d: &mut PartDiff) {
    d.checked += 1 + a.attrs.len();
    compare_attrs(a, b, path, d);
    if a.children.is_empty() && b.children.is_empty() && a.text != b.text {
        push(
            d,
            element(
                path.clone(),
                Verdict::Changed,
                Some(format!(
                    "text {:?} became {:?}",
                    clip(&a.text),
                    clip(&b.text)
                )),
            ),
        );
    }
    if a.skipped > 0 || b.skipped > 0 {
        // A semantic-owned subtree: T-4513 scores its contents.
        d.checked += a.skipped;
        return;
    }
    let (ka, kb) = {
        let local_path = local.as_str();
        let unordered = |child: &Node| {
            if child.name.local == "ext" && a.name.local == "extLst" {
                return Some(&["uri"][..]);
            }
            UNORDERED
                .iter()
                .find(|(parent, name, _)| *parent == local_path && *name == child.name.local)
                .map(|(_, _, keys)| *keys)
        };
        (
            keyed(&a.children, &unordered),
            keyed(&b.children, &unordered),
        )
    };
    for ((name, key), nodes) in &ka {
        let others = kb.get(&(name.clone(), key.clone()));
        for (n, node) in nodes.iter().enumerate() {
            let (path_len, local_len) = (path.len(), local.len());
            path.push('/');
            path.push_str(&child_label(name, key, n));
            if let Some(other) = others.and_then(|o| o.get(n)) {
                local.push('/');
                local.push_str(&name.local);
                compare(node, other, path, local, d);
            } else if d.differences.len() >= MAX_ITEMS {
                // Past the cap, count the loss without building its path.
                d.checked += node.items();
                d.truncated += 1;
                d.truncated_weight += node.items();
            } else {
                d.checked += node.items();
                let below = node.size() - 1;
                let detail = (below > 0).then(|| format!("with {below} elements below it"));
                let mut lost = element(path.clone(), Verdict::Lost, detail);
                lost.weight = node.items();
                push(d, lost);
            }
            path.truncate(path_len);
            local.truncate(local_len);
        }
    }
    for ((name, key), nodes) in &kb {
        let have = ka.get(&(name.clone(), key.clone())).map_or(0, Vec::len);
        for n in have..nodes.len() {
            if d.differences.len() >= MAX_ITEMS {
                d.truncated += 1;
                continue;
            }
            push(
                d,
                element(
                    format!("{path}/{}", child_label(name, key, n)),
                    Verdict::Added,
                    None,
                ),
            );
        }
    }
}

/// A child's path segment: its name, then its key at an unordered path or its ordinal.
fn child_label(name: &Name, key: &str, n: usize) -> String {
    if key.is_empty() {
        format!("{}[{n}]", name.label())
    } else {
        format!("{}[{key}]", name.label())
    }
}

/// Children grouped by name and, at an unordered path, by their key attributes.
fn keyed<'n>(
    nodes: &'n [Node],
    unordered: &dyn Fn(&Node) -> Option<&'static [&'static str]>,
) -> BTreeMap<(Name, String), Vec<&'n Node>> {
    let mut out: BTreeMap<(Name, String), Vec<&Node>> = BTreeMap::new();
    for n in nodes {
        let key = match unordered(n) {
            Some(attrs) => attrs
                .iter()
                .map(|k| local_attr(n, k).unwrap_or_default())
                .collect::<Vec<_>>()
                .join(","),
            None => String::new(),
        };
        out.entry((n.name.clone(), key)).or_default().push(n);
    }
    out
}

fn compare_attrs(src: &Node, out: &Node, path: &str, diffs: &mut PartDiff) {
    let names: BTreeSet<&Name> = src.attrs.keys().chain(out.attrs.keys()).collect();
    for name in names {
        let default = default_of(&src.name, name);
        let was = src.attrs.get(name).map(String::as_str).or(default);
        let now = out.attrs.get(name).map(String::as_str).or(default);
        let verdict = match (was, now) {
            (Some(old), Some(new)) if same_value(name, default, old, new) => continue,
            (Some(_), Some(_)) => Verdict::Changed,
            (Some(_), None) => Verdict::Lost,
            (None, Some(_)) => Verdict::Added,
            (None, None) => continue,
        };
        let detail = (verdict == Verdict::Changed).then(|| format!("{was:?} became {now:?}"));
        push(
            diffs,
            Difference {
                kind: "attribute",
                path: format!("{path}@{}", name.label()),
                verdict,
                detail,
                weight: unit_weight(verdict),
            },
        );
    }
}

/// The schema default of an attribute in no namespace on `element` (D4.3).
fn default_of(element: &Name, attr: &Name) -> Option<&'static str> {
    if !attr.ns.is_empty() || element.ns != MAIN {
        return None;
    }
    let (_, row) = DEFAULTS.iter().find(|(e, _)| *e == element.local)?;
    row.split(' ')
        .filter_map(|pair| pair.split_once('='))
        .find(|(a, _)| *a == attr.local)
        .map(|(_, v)| v)
}

/// Whether two attribute values are equal: `mc:Ignorable` as a set of namespace URIs (already
/// resolved at parse time), an attribute whose schema default is a boolean as an xsd:boolean
/// (`1` equals `true`), anything else as text.
fn same_value(name: &Name, default: Option<&str>, a: &str, b: &str) -> bool {
    if name.ns == MC && name.local == "Ignorable" {
        let set = |v: &str| {
            v.split_whitespace()
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        };
        return set(a) == set(b);
    }
    let boolean = |v: &str| match v {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    };
    let is_boolean = default.is_some_and(|d| d == "true" || d == "false");
    a == b || (is_boolean && matches!((boolean(a), boolean(b)), (Some(x), Some(y)) if x == y))
}

fn local_attr(n: &Node, local: &str) -> Option<String> {
    n.attrs
        .iter()
        .find(|(k, _)| k.ns.is_empty() && k.local == local)
        .map(|(_, v)| v.clone())
}

fn resolved(r: &ResolveResult<'_>) -> String {
    match r {
        ResolveResult::Bound(ns) => ns.as_ref().to_string(),
        _ => String::new(),
    }
}

/// Parses a part into a tree, resolving every name to its namespace URI and `mc:Ignorable`'s
/// prefixes to URIs, dropping namespace declarations, and only counting [`SEMANTIC`] subtrees.
fn parse(xml: &str) -> Result<Node, String> {
    let mut reader = NsReader::from_str(xml);
    let mut stack: Vec<Node> = Vec::new();
    // The local-name path of the open elements, extended and truncated in place.
    let mut path = String::new();
    let mut lens: Vec<usize> = Vec::new();
    let mut skip_depth = 0usize;
    let mut nodes = 0usize;
    loop {
        let event = reader.read_event().map_err(|e| e.to_string())?;
        match event {
            Event::Start(ref e) | Event::Empty(ref e) => {
                let empty = matches!(event, Event::Empty(_));
                if skip_depth > 0 {
                    if let Some(top) = stack.last_mut() {
                        top.skipped += 1;
                    }
                    if !empty {
                        skip_depth += 1;
                    }
                    continue;
                }
                nodes += 1;
                if nodes > MAX_NODES || stack.len() >= MAX_DEPTH {
                    return Err(format!(
                        "more than {MAX_NODES} elements or {MAX_DEPTH} levels"
                    ));
                }
                let node = start_node(&reader, e)?;
                lens.push(path.len());
                path.push('/');
                path.push_str(&node.name.local);
                if path.len() > MAX_PATH {
                    return Err(format!("an element path longer than {MAX_PATH} bytes"));
                }
                let semantic = SEMANTIC.contains(&path.as_str());
                stack.push(node);
                if empty {
                    path.truncate(lens.pop().unwrap_or_default());
                    if stack.len() == 1 {
                        return stack.pop().ok_or_else(|| "no root".to_string());
                    }
                    close(&mut stack);
                } else if semantic {
                    skip_depth = 1;
                }
            }
            Event::End(_) => {
                if skip_depth > 1 {
                    skip_depth -= 1;
                    continue;
                }
                skip_depth = 0;
                path.truncate(lens.pop().unwrap_or_default());
                if stack.len() == 1 {
                    return stack.pop().ok_or_else(|| "no root".to_string());
                }
                close(&mut stack);
            }
            Event::Text(t) if skip_depth == 0 => {
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(&t.xml_content(XmlVersion::Implicit1_0));
                }
            }
            Event::CData(t) if skip_depth == 0 => {
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(&t.into_inner());
                }
            }
            Event::GeneralRef(r) if skip_depth == 0 => {
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(&resolve_entity(&r));
                }
            }
            Event::Eof => {
                return match stack.pop() {
                    Some(root) if stack.is_empty() => Ok(root),
                    _ => Err("the part has no complete root element".into()),
                }
            }
            _ => {}
        }
    }
}

/// The node for a start tag: its resolved name and attributes, without namespace declarations.
///
/// # Errors
/// On a malformed or repeated attribute, or a name or namespace past [`MAX_NAME`] or
/// [`MAX_NAMESPACE`]: such a part is unparsable, never silently half-read.
fn start_node(reader: &NsReader<&[u8]>, e: &BytesStart<'_>) -> Result<Node, String> {
    let (ns, local) = reader.resolver().resolve_element(e.name());
    let checked_name = |ns: String, local: &str| {
        if local.len() > MAX_NAME || ns.len() > MAX_NAMESPACE {
            return Err(format!(
                "a name longer than {MAX_NAME} bytes or a namespace longer than {MAX_NAMESPACE}"
            ));
        }
        Ok(Name {
            ns,
            local: local.to_string(),
        })
    };
    let element_name = checked_name(resolved(&ns), local.as_ref())?;
    let mut attrs = BTreeMap::new();
    for a in e.attributes() {
        let a = a.map_err(|e| e.to_string())?;
        let key = a.key.as_ref();
        if key == "xmlns" || key.starts_with("xmlns:") {
            continue;
        }
        let (ans, alocal) = reader.resolver().resolve_attribute(a.key);
        let name = checked_name(resolved(&ans), alocal.as_ref())?;
        if name.ns == XMLNS {
            continue;
        }
        let mut value = a
            .normalized_value(XmlVersion::Implicit1_0)
            .map(std::borrow::Cow::into_owned)
            .unwrap_or_default();
        if name.ns == MC && name.local == "Ignorable" {
            // Prefixes are local to the part: compare the namespaces they stand for.
            value = value
                .split_whitespace()
                .map(|p| {
                    let probe = format!("{p}:x");
                    resolved(&reader.resolver().resolve_element(QName(&probe)).0)
                })
                .collect::<Vec<_>>()
                .join(" ");
        }
        attrs.insert(name, value);
    }
    Ok(Node {
        name: element_name,
        attrs,
        children: Vec::new(),
        text: String::new(),
        skipped: 0,
    })
}

/// Pops the finished top node into its parent. Whitespace-only text of an element that has
/// children is formatting, not content (D4.7).
fn close(stack: &mut Vec<Node>) {
    if let Some(mut node) = stack.pop() {
        if !node.children.is_empty() && node.text.trim().is_empty() {
            node.text.clear();
        }
        if let Some(parent) = stack.last_mut() {
            parent.children.push(node);
        }
    }
}

/// The candidate normalization (SPEC-0033 D4.4) an item key falls under, if any. The key is a
/// part key, then `:` and an element path for element and attribute items.
#[must_use]
pub(crate) fn candidate(key: &str) -> Option<&'static str> {
    let (part, path) = key.split_once(':').unwrap_or((key, ""));
    let last = part.rsplit('/').next().unwrap_or(part);
    let attr = path.rsplit_once('@').map(|(_, a)| a);
    let in_core = last.starts_with("core-properties[");
    let in_app = last.starts_with("extended-properties[");
    let n = if path.is_empty() && last.starts_with("calcChain[") {
        "N1"
    } else if in_core
        && (path.ends_with("/dcterms:modified[0]") || path.ends_with("/cp:lastModifiedBy[0]"))
    {
        "N2"
    } else if in_app
        && [
            "Application",
            "AppVersion",
            "DocSecurity",
            "TitlesOfParts",
            "HeadingPairs",
        ]
        .iter()
        .any(|e| path.contains(&format!("/ep:{e}[")))
    {
        "N3"
    } else if path.contains("/calcPr[0]") && attr == Some("calcId") {
        "N4"
    } else if path.contains("/fileVersion[0]")
        && matches!(
            attr,
            Some("lastEdited" | "lowestEdited" | "rupBuild" | "appName")
        )
    {
        "N5"
    } else if path.contains("xr:revisionPtr")
        || attr.is_some_and(|a| a.ends_with(":uid") || a.ends_with("}uid"))
    {
        "N6"
    } else if attr == Some("x14ac:dyDescent") {
        "N7"
    } else if attr == Some("spans") {
        "N8"
    } else if path.ends_with("/dimension[0]@ref") {
        "N9"
    } else if matches!(attr, Some("count" | "uniqueCount")) {
        "N10"
    } else if path.contains("/workbookView[")
        && matches!(
            attr,
            Some("xWindow" | "yWindow" | "windowWidth" | "windowHeight")
        )
    {
        "N11"
    } else if path.contains("mc:AlternateContent") {
        "N12"
    } else {
        return None;
    };
    Some(n)
}
