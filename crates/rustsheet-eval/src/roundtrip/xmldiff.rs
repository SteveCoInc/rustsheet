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

/// Unordered collections (D4.2): `(parent path, child local name, key attributes)`. The schema
/// gives these children no order meaning (ECMA-376 Part 1 CT_MergeCells, CT_Hyperlinks,
/// CT_DataValidations, CT_DefinedNames, CT_Cols; Part 2 §10.1.2.2 for content types), so they
/// match by key. `extLst` children always match by `uri` (CT_ExtensionList).
const UNORDERED: &[(&str, &str, &[&str])] = &[
    ("/worksheet/mergeCells", "mergeCell", &["ref"]),
    ("/worksheet/hyperlinks", "hyperlink", &["ref"]),
    ("/worksheet/dataValidations", "dataValidation", &["sqref"]),
    ("/worksheet", "conditionalFormatting", &["sqref"]),
    ("/worksheet/cols", "col", &["min"]),
    (
        "/workbook/definedNames",
        "definedName",
        &["name", "localSheetId"],
    ),
    ("/Types", "Override", &["PartName"]),
    ("/Types", "Default", &["Extension"]),
];

/// Schema defaults (D4.3): `(element local name, "attribute=default ...")`. An absent attribute
/// equals its default. Each is the `default=` of the attribute in ECMA-376 Part 1's sml.xsd:
/// CT_Cell (`t`, `s`), CT_Row, CT_Col, CT_SheetView, CT_Pane, CT_Selection, CT_Sheet,
/// CT_BookView, CT_SheetFormatPr and CT_BooleanProperty (`val`, as in `<b/>`).
const DEFAULTS: &[(&str, &str)] = &[
    ("c", "t=n s=0"),
    (
        "row",
        "hidden=false customHeight=false customFormat=false outlineLevel=0 collapsed=false",
    ),
    (
        "col",
        "hidden=false customWidth=false bestFit=false outlineLevel=0 collapsed=false style=0",
    ),
    (
        "sheetView",
        "tabSelected=false showGridLines=true rightToLeft=false zoomScale=100 view=normal",
    ),
    ("pane", "xSplit=0 ySplit=0 activePane=topLeft state=split"),
    ("selection", "pane=topLeft activeCellId=0"),
    ("sheet", "state=visible"),
    (
        "workbookView",
        "activeTab=0 firstSheet=0 visibility=visible",
    ),
    (
        "sheetFormatPr",
        "customHeight=false zeroHeight=false outlineLevelRow=0 outlineLevelCol=0",
    ),
    ("b", "val=true"),
    ("i", "val=true"),
    ("strike", "val=true"),
];

/// Deepest element nesting a part may have (directive 5); deeper parts are reported as unparsable.
const MAX_DEPTH: usize = 256;

/// Most elements kept per part (directive 5). Subtrees under [`SEMANTIC`] are only counted.
const MAX_NODES: usize = 1_000_000;

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
}

/// One difference: the item's path, what it is, and its verdict label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Difference {
    /// `element` or `attribute`.
    pub(crate) kind: &'static str,
    /// The element path (`/worksheet/sheetViews/sheetView[0]`), with `@attr` for an attribute.
    pub(crate) path: String,
    /// `lost`, `added` or `changed`.
    pub(crate) verdict: &'static str,
    pub(crate) detail: Option<String>,
}

/// The result of diffing one part pair.
#[derive(Debug, Default)]
pub(crate) struct PartDiff {
    /// Source elements and attributes compared (the case's checked count, SPEC-0033 D8).
    pub(crate) checked: usize,
    pub(crate) differences: Vec<Difference>,
    /// Differences beyond [`MAX_ITEMS`].
    pub(crate) truncated: usize,
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
        let path = format!("/{}", a.name.local);
        compare(&a, &b, &path, &mut d);
    } else {
        d.checked = a.size();
        push(
            &mut d,
            element(
                format!("/{}", a.name.label()),
                "changed",
                Some(format!("root became {}", b.name.label())),
            ),
        );
    }
    Ok(d)
}

fn element(path: String, verdict: &'static str, detail: Option<String>) -> Difference {
    Difference {
        kind: "element",
        path,
        verdict,
        detail,
    }
}

fn push(d: &mut PartDiff, diff: Difference) {
    if d.differences.len() < MAX_ITEMS {
        d.differences.push(diff);
    } else {
        d.truncated += 1;
    }
}

/// Compares two matched elements at `path`: attributes, text, then children.
fn compare(a: &Node, b: &Node, path: &str, d: &mut PartDiff) {
    d.checked += 1 + a.attrs.len();
    compare_attrs(a, b, path, d);
    if a.children.is_empty() && b.children.is_empty() && a.text != b.text {
        push(
            d,
            element(
                path.to_string(),
                "changed",
                Some(format!("text {:?} became {:?}", a.text, b.text)),
            ),
        );
    }
    if a.skipped > 0 || b.skipped > 0 {
        // A semantic-owned subtree: T-4513 scores its contents.
        d.checked += a.skipped;
        return;
    }
    let local_path = strip_ordinals(path);
    let unordered = |child: &Node| {
        if child.name.local == "ext" && a.name.local == "extLst" {
            return Some(&["uri"][..]);
        }
        UNORDERED
            .iter()
            .find(|(parent, name, _)| *parent == local_path && *name == child.name.local)
            .map(|(_, _, keys)| *keys)
    };
    let (ka, kb) = (
        keyed(&a.children, &unordered),
        keyed(&b.children, &unordered),
    );
    for ((name, key), nodes) in &ka {
        let others = kb.get(&(name.clone(), key.clone()));
        for (n, node) in nodes.iter().enumerate() {
            let label = if key.is_empty() {
                format!("{}[{n}]", name.label())
            } else {
                format!("{}[{key}]", name.label())
            };
            let child_path = format!("{path}/{label}");
            match others.and_then(|o| o.get(n)) {
                Some(other) => compare(node, other, &child_path, d),
                None => {
                    d.checked += node.size();
                    let below = node.size() - 1;
                    let detail = (below > 0).then(|| format!("with {below} elements below it"));
                    push(d, element(child_path, "lost", detail));
                }
            }
        }
    }
    for ((name, key), nodes) in &kb {
        let have = ka.get(&(name.clone(), key.clone())).map_or(0, Vec::len);
        for n in have..nodes.len() {
            let label = if key.is_empty() {
                format!("{}[{n}]", name.label())
            } else {
                format!("{}[{key}]", name.label())
            };
            push(d, element(format!("{path}/{label}"), "added", None));
        }
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

fn compare_attrs(a: &Node, b: &Node, path: &str, d: &mut PartDiff) {
    let names: BTreeSet<&Name> = a.attrs.keys().chain(b.attrs.keys()).collect();
    for name in names {
        let default = default_of(&a.name, name);
        let va = a.attrs.get(name).map(String::as_str).or(default);
        let vb = b.attrs.get(name).map(String::as_str).or(default);
        let verdict = match (va, vb) {
            (Some(x), Some(y)) if same_value(name, x, y) => continue,
            (Some(_), Some(_)) => "changed",
            (Some(_), None) => "lost",
            (None, Some(_)) => "added",
            (None, None) => continue,
        };
        let detail = (verdict == "changed").then(|| format!("{va:?} became {vb:?}"));
        push(
            d,
            Difference {
                kind: "attribute",
                path: format!("{path}@{}", name.label()),
                verdict,
                detail,
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
/// resolved at parse time), xsd:boolean spellings as booleans, anything else as text.
fn same_value(name: &Name, a: &str, b: &str) -> bool {
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
    a == b || matches!((boolean(a), boolean(b)), (Some(x), Some(y)) if x == y)
}

fn local_attr(n: &Node, local: &str) -> Option<String> {
    n.attrs
        .iter()
        .find(|(k, _)| k.ns.is_empty() && k.local == local)
        .map(|(_, v)| v.clone())
}

/// `/worksheet/cols[0]/col[3]` → `/worksheet/cols/col`.
fn strip_ordinals(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut depth = 0usize;
    for c in path.chars() {
        match c {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

fn resolved(r: ResolveResult<'_>) -> String {
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
    let mut path: Vec<String> = Vec::new();
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
                let node = start_node(&reader, e);
                path.push(node.name.local.clone());
                let semantic = SEMANTIC.contains(&format!("/{}", path.join("/")).as_str());
                stack.push(node);
                if empty {
                    path.pop();
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
                path.pop();
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
fn start_node(reader: &NsReader<&[u8]>, e: &BytesStart<'_>) -> Node {
    let (ns, local) = reader.resolver().resolve_element(e.name());
    let mut attrs = BTreeMap::new();
    for a in e.attributes().flatten() {
        let key = a.key.as_ref();
        if key == "xmlns" || key.starts_with("xmlns:") {
            continue;
        }
        let (ans, alocal) = reader.resolver().resolve_attribute(a.key);
        let name = Name {
            ns: resolved(ans),
            local: alocal.as_ref().to_string(),
        };
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
                    resolved(reader.resolver().resolve_element(QName(&probe)).0)
                })
                .collect::<Vec<_>>()
                .join(" ");
        }
        attrs.insert(name, value);
    }
    Node {
        name: Name {
            ns: resolved(ns),
            local: local.as_ref().to_string(),
        },
        attrs,
        children: Vec::new(),
        text: String::new(),
        skipped: 0,
    }
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
    } else if path.contains("/fileVersion[0]") && attr.is_some() {
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
