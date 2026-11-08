//! The ownership matrix, the closure rule and the coupling groups of SPEC-0034 stage 5 (D5.1,
//! D5.2, D5.6; T-4999): what a regenerated save may keep of the source package, and why each
//! part it does not keep is left out.
//!
//! This module decides. Writing kept parts comes with T-5000 onward, so until then every part this
//! module would keep is still reported `NotPreserved`; what changes with it is that each drop now
//! names its real reason, and that a modelled part the export did not write is found by comparing
//! the output's own inventory.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::io::Cursor;

use crate::import::{
    is_rels, part_key, part_relationships, rel_kind, rels_owner_part, resolve_target,
    zip_entry_names, ImportLimits, Package, XmlNode,
};

mod decide;

pub(super) use decide::{calc_chains, regenerated};

/// What a regenerated save does with a part, by the relationship type that reaches it (D5.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Policy {
    /// Written from the model; the source part is never copied.
    Modelled,
    /// Copied while the package or the workbook exists.
    Replay,
    /// Copied unless an edit invalidated what it is anchored to.
    ReplayIfClean,
    /// Copied only into a macro-enabled file, never interpreted.
    Active,
    /// Never written.
    Drop,
}

/// Where the relationship that reaches a part starts (D5.1's unknown rows).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Reach {
    /// `_rels/.rels`.
    Package,
    /// The workbook part.
    Workbook,
    /// A worksheet part.
    Sheet,
    /// Any other part (a drawing reaching its chart).
    Other,
}

/// D5.1's table: the last segment of each listed relationship type, by policy.
const TABLE: &[(Policy, &[&str])] = &[
    (
        Policy::Modelled,
        &[
            "officeDocument",
            "worksheet",
            "styles",
            "sharedStrings",
            "theme",
            "sheetMetadata",
            "rdRichValue",
            "rdRichValueStructure",
            "rdRichValueTypes",
            "table",
            "pivotCacheDefinition",
            "pivotCacheRecords",
            "pivotTable",
            "externalLink",
            "core-properties",
            "extended-properties",
        ],
    ),
    (
        Policy::Replay,
        &[
            "customXml",
            "customXmlProps",
            "custom-properties",
            "thumbnail",
            "connections",
            "person",
            "webextension",
            "webextensiontaskpanes",
        ],
    ),
    (
        Policy::ReplayIfClean,
        &[
            "drawing",
            "chart",
            "image",
            "chartUserShapes",
            "chartStyle",
            "chartColorStyle",
            "vmlDrawing",
            "comments",
            "threadedComment",
            "ctrlProp",
            "control",
            "activeXControlBinary",
            "oleObject",
            "package",
            "printerSettings",
            "slicer",
            "timeline",
            "queryTable",
            "slicerCache",
            "timelineCache",
        ],
    ),
    (Policy::Active, &["vbaProject", "vbaProjectSignature"]),
    (
        Policy::Drop,
        &[
            "calcChain",
            "origin",
            "certificate",
            "signature",
            "revisionHeaders",
            "revisionLog",
            "usernames",
            "richValueRel",
            "rdArray",
            "rdSupportingPropertyBag",
            "rdSupportingPropertyBagStructure",
            "rdRichValueWebImage",
            "richStyles",
        ],
    ),
];

/// The policy of a part reached by a relationship of type `rel_type` (its last segment) from
/// `reached_from`. A type not in the table is known by its content type when that names VBA or a
/// calcChain; otherwise it is `Replay` from the package or the workbook and `ReplayIfClean` from a
/// sheet or any other part (D5.1's unknown rows; the closure rule then decides, D5.2).
pub(super) fn policy_of(rel_type: &str, content_type: Option<&str>, reached_from: Reach) -> Policy {
    if let Some((policy, _)) = TABLE.iter().find(|(_, kinds)| kinds.contains(&rel_type)) {
        return *policy;
    }
    match content_type {
        Some(ct) if ct.contains("vbaProject") => Policy::Active,
        Some(ct) if ct.contains("calcChain") => Policy::Drop,
        _ => match reached_from {
            Reach::Package | Reach::Workbook => Policy::Replay,
            Reach::Sheet | Reach::Other => Policy::ReplayIfClean,
        },
    }
}

/// The relationship kinds of the workbook's sheets (D5.1: chartsheets, dialogsheets and
/// macrosheets are sheets too).
const SHEET_KINDS: &[&str] = &["worksheet", "chartsheet", "dialogsheet", "xlMacrosheet"];

/// D5.6: the most relationship hops a closure follows.
pub(super) const MAX_CLOSURE_DEPTH: usize = 16;
/// D5.6: the most parts one closure, or one coupling group's closures together, may hold.
pub(super) const MAX_CLOSURE_SIZE: usize = 1_000;

/// One internal relationship of a part.
#[derive(Clone, Debug)]
struct Edge {
    /// Its `Id`.
    id: String,
    /// The last segment of its type.
    kind: String,
    /// The target's part key.
    target: String,
}

/// A package's parts and the relationships between them, read within the load's limits.
#[derive(Debug, Default)]
pub(super) struct Graph {
    /// The content parts (no `.rels`, no `[Content_Types].xml`, no folders), in entry order.
    entries: Vec<String>,
    /// Each part key's relationships, in their `.rels` order; `""` is the package.
    edges: BTreeMap<String, Vec<Edge>>,
    /// Each part key reached by a relationship: the first owner and kind that reach it.
    reached: HashMap<String, (String, String)>,
    /// Each part key's override content type.
    content_types: HashMap<String, String>,
    /// The workbook part's key.
    workbook: Option<String>,
    /// Each workbook `<sheet>`'s relationship id mapped to its `sheetId`: a sheet's identity that
    /// renaming, moving and dropping other sheets all keep (the export writes the model's
    /// `sheetId`, which the importer reads from the source).
    sheet_ids: HashMap<String, String>,
    /// Each workbook `<pivotCache>`'s relationship id mapped to its `cacheId`, which the cache's
    /// pivot tables name.
    cache_ids: HashMap<String, String>,
}

impl Graph {
    /// Reads `bytes` within `limits`. A `.rels` part that does not parse, or passes the budget,
    /// is skipped: the parts only it reaches count as unreached.
    pub(super) fn read(bytes: &[u8], limits: &ImportLimits) -> Result<Graph, String> {
        let names = zip_entry_names(bytes).ok_or("the package is not a readable zip")?;
        let mut package = Package::new(Cursor::new(bytes), *limits).map_err(|e| e.to_string())?;
        let mut graph = Graph::default();
        for rels in names.iter().filter(|n| is_rels(n)) {
            let Some(owner) = rels_owner_part(rels) else {
                continue;
            };
            let base = owner.rsplit_once('/').map_or("", |(folder, _)| folder);
            let Ok(relationships) = part_relationships(&mut package, rels) else {
                continue;
            };
            let owner = part_key(&owner);
            let mut out = Vec::new();
            for (id, rel) in relationships.into_iter().filter(|(_, r)| !r.external) {
                let kind = rel_kind(&rel.rel_type).to_string();
                let target = part_key(&resolve_target(base, &rel.target));
                if owner.is_empty() && kind == "officeDocument" {
                    graph.workbook.get_or_insert_with(|| target.clone());
                }
                graph
                    .reached
                    .entry(target.clone())
                    .or_insert_with(|| (owner.clone(), kind.clone()));
                out.push(Edge { id, kind, target });
            }
            // Two spellings of one `.rels` (a package OPC calls invalid): the first read wins.
            graph.edges.entry(owner).or_insert(out);
        }
        if let Some(workbook) = graph.workbook.clone() {
            if let Ok(doc) = package.open(&workbook).and_then(XmlNode::parse) {
                let ids = |tag: &str, attr: &str| -> HashMap<String, String> {
                    (doc.descendants())
                        .filter(|n| n.has_tag_name(tag))
                        .filter_map(|n| {
                            Some((
                                n.relationship_id()?.to_string(),
                                n.attribute(attr)?.to_string(),
                            ))
                        })
                        .collect()
                };
                graph.sheet_ids = ids("sheet", "sheetId");
                graph.cache_ids = ids("pivotCache", "cacheId");
            }
        }
        if let Ok(types) = package.open("[Content_Types].xml") {
            if let Ok(doc) = XmlNode::parse(types) {
                for o in doc.descendants().filter(|n| n.has_tag_name("Override")) {
                    if let (Some(part), Some(ct)) =
                        (o.attribute("PartName"), o.attribute("ContentType"))
                    {
                        graph.content_types.insert(part_key(part), ct.to_string());
                    }
                }
            }
        }
        graph.entries = names
            .into_iter()
            .filter(|n| !n.ends_with('/') && !is_rels(n))
            .filter(|n| part_key(n) != part_key("[Content_Types].xml"))
            .collect();
        Ok(graph)
    }

    /// Where the relationship that reaches `owner`'s children starts.
    fn reach_of(&self, owner: &str) -> Reach {
        if owner.is_empty() {
            return Reach::Package;
        }
        if self.workbook.as_deref() == Some(owner) {
            return Reach::Workbook;
        }
        match self.reached.get(owner) {
            Some((_, kind)) if SHEET_KINDS.contains(&kind.as_str()) => Reach::Sheet,
            _ => Reach::Other,
        }
    }

    /// The policy of the part `key`, by what reaches it. A part no relationship reaches is
    /// known by its content type alone (D5.1), and is otherwise `Replay`: it is its own root.
    fn policy(&self, key: &str) -> Policy {
        let content_type = self.content_types.get(key).map(String::as_str);
        match self.reached.get(key) {
            Some((owner, kind)) => policy_of(kind, content_type, self.reach_of(owner)),
            None => policy_of("", content_type, Reach::Package),
        }
    }

    /// The parts reachable from `root` by internal relationships, `root` included (D5.2), within
    /// D5.6's depth and size limits; `Err` names the limit passed.
    pub(super) fn closure(&self, root: &str) -> Result<BTreeSet<String>, &'static str> {
        let mut seen = BTreeSet::from([root.to_string()]);
        let mut queue = VecDeque::from([(root.to_string(), 0usize)]);
        while let Some((part, depth)) = queue.pop_front() {
            for edge in self.edges.get(&part).into_iter().flatten() {
                if seen.contains(&edge.target) {
                    continue;
                }
                if depth + 1 > MAX_CLOSURE_DEPTH {
                    return Err("closure depth");
                }
                seen.insert(edge.target.clone());
                if seen.len() > MAX_CLOSURE_SIZE {
                    return Err("closure size");
                }
                queue.push_back((edge.target.clone(), depth + 1));
            }
        }
        Ok(seen)
    }
}

#[cfg(test)]
mod tests;
