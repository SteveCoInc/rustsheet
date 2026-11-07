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

use super::report::{DropReason, Dropped};
use crate::import::{
    is_rels, part_key, part_relationships, rels_owner_part, resolve_target, zip_entry_names,
    ImportLimits, Package,
};

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

/// D5.1's table: the last segment of each listed relationship type and its policy.
const TABLE: &[(&str, Policy)] = &[
    ("officeDocument", Policy::Modelled),
    ("worksheet", Policy::Modelled),
    ("styles", Policy::Modelled),
    ("sharedStrings", Policy::Modelled),
    ("theme", Policy::Modelled),
    ("sheetMetadata", Policy::Modelled),
    ("rdRichValue", Policy::Modelled),
    ("rdRichValueStructure", Policy::Modelled),
    ("rdRichValueTypes", Policy::Modelled),
    ("table", Policy::Modelled),
    ("pivotCacheDefinition", Policy::Modelled),
    ("pivotCacheRecords", Policy::Modelled),
    ("pivotTable", Policy::Modelled),
    ("externalLink", Policy::Modelled),
    ("core-properties", Policy::Modelled),
    ("extended-properties", Policy::Modelled),
    ("customXml", Policy::Replay),
    ("customXmlProps", Policy::Replay),
    ("custom-properties", Policy::Replay),
    ("thumbnail", Policy::Replay),
    ("connections", Policy::Replay),
    ("person", Policy::Replay),
    ("webextension", Policy::Replay),
    ("webextensiontaskpanes", Policy::Replay),
    ("drawing", Policy::ReplayIfClean),
    ("chart", Policy::ReplayIfClean),
    ("image", Policy::ReplayIfClean),
    ("chartUserShapes", Policy::ReplayIfClean),
    ("chartStyle", Policy::ReplayIfClean),
    ("chartColorStyle", Policy::ReplayIfClean),
    ("vmlDrawing", Policy::ReplayIfClean),
    ("comments", Policy::ReplayIfClean),
    ("threadedComment", Policy::ReplayIfClean),
    ("ctrlProp", Policy::ReplayIfClean),
    ("control", Policy::ReplayIfClean),
    ("activeXControlBinary", Policy::ReplayIfClean),
    ("oleObject", Policy::ReplayIfClean),
    ("package", Policy::ReplayIfClean),
    ("printerSettings", Policy::ReplayIfClean),
    ("slicer", Policy::ReplayIfClean),
    ("timeline", Policy::ReplayIfClean),
    ("queryTable", Policy::ReplayIfClean),
    ("slicerCache", Policy::ReplayIfClean),
    ("timelineCache", Policy::ReplayIfClean),
    ("vbaProject", Policy::Active),
    ("vbaProjectSignature", Policy::Active),
    ("calcChain", Policy::Drop),
    ("origin", Policy::Drop),
    ("signature", Policy::Drop),
    ("revisionHeaders", Policy::Drop),
    ("revisionLog", Policy::Drop),
    ("usernames", Policy::Drop),
    ("richValueRel", Policy::Drop),
    ("rdArray", Policy::Drop),
    ("rdSupportingPropertyBag", Policy::Drop),
    ("rdSupportingPropertyBagStructure", Policy::Drop),
    ("rdRichValueWebImage", Policy::Drop),
    ("richStyles", Policy::Drop),
];

/// The policy of a part reached by a relationship of type `rel_type` (its last segment) from
/// `reached_from`. A type not in the table is known by its content type when that names VBA or a
/// calcChain; otherwise it is `Replay` from the package or the workbook and `ReplayIfClean` from a
/// sheet or any other part (D5.1's unknown rows; the closure rule then decides, D5.2).
pub(super) fn policy_of(rel_type: &str, content_type: Option<&str>, reached_from: Reach) -> Policy {
    if let Some((_, policy)) = TABLE.iter().find(|(kind, _)| *kind == rel_type) {
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

/// D5.6: the most relationship hops a closure follows.
pub(super) const MAX_CLOSURE_DEPTH: usize = 16;
/// D5.6: the most parts one closure, or one coupling group's closures together, may hold.
pub(super) const MAX_CLOSURE_SIZE: usize = 1_000;

/// One internal relationship of a part.
#[derive(Clone, Debug)]
struct Edge {
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
            for (_, rel) in relationships.into_iter().filter(|(_, r)| !r.external) {
                let kind = rel
                    .rel_type
                    .rsplit('/')
                    .next()
                    .unwrap_or_default()
                    .to_string();
                let target = part_key(&resolve_target(base, &rel.target));
                if owner.is_empty() && kind == "officeDocument" {
                    graph.workbook.get_or_insert_with(|| target.clone());
                }
                graph
                    .reached
                    .entry(target.clone())
                    .or_insert_with(|| (owner.clone(), kind.clone()));
                out.push(Edge { kind, target });
            }
            graph.edges.insert(owner, out);
        }
        if let Ok(types) = package.open("[Content_Types].xml") {
            if let Ok(doc) = crate::import::XmlNode::parse(types) {
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
            Some((_, kind)) if kind == "worksheet" => Reach::Sheet,
            _ => Reach::Other,
        }
    }

    /// The policy of the part `key`, by what reaches it; `None` for a part no relationship
    /// reaches.
    fn policy(&self, key: &str) -> Option<Policy> {
        let (owner, kind) = self.reached.get(key)?;
        Some(policy_of(
            kind,
            self.content_types.get(key).map(String::as_str),
            self.reach_of(owner),
        ))
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

    /// Each reached part's identity, the same in a source and in a save of it: its owner's
    /// identity, the relationship kind and its ordinal among the owner's relationships of that kind.
    /// A save that writes fewer parts of a kind under one owner lacks the last identities.
    fn identities(&self) -> HashMap<String, String> {
        let mut ids = HashMap::new();
        let mut queue = VecDeque::from([(String::new(), String::new())]);
        let mut seen = BTreeSet::new();
        while let Some((part, id)) = queue.pop_front() {
            let mut ordinal: HashMap<&str, usize> = HashMap::new();
            for edge in self.edges.get(&part).into_iter().flatten() {
                let n = ordinal.entry(edge.kind.as_str()).or_default();
                let child = format!("{id}/{}[{n}]", edge.kind);
                *n += 1;
                if seen.insert(edge.target.clone()) {
                    ids.insert(edge.target.clone(), child.clone());
                    queue.push_back((edge.target.clone(), child));
                }
            }
        }
        ids
    }
}

/// The coupling groups of `source` (D5.2), each a set of part keys kept or dropped as one unit:
/// per sheet, the legacy drawing group, and for the workbook, the slicer and timeline group. A
/// drawing joins a group when its bytes hold the shapes the group shares (`a14:compatExt`, a
/// slicer or timeline frame).
fn coupling_groups(source: &Graph, bytes: &[u8], limits: &ImportLimits) -> Vec<BTreeSet<String>> {
    const LEGACY: &[&str] = &[
        "vmlDrawing",
        "comments",
        "threadedComment",
        "ctrlProp",
        "control",
        "oleObject",
        "package",
    ];
    const SLICERS: &[&str] = &["slicer", "timeline", "slicerCache", "timelineCache"];
    let mut package = Package::new(Cursor::new(bytes), *limits).ok();
    let mut holds = |key: &str, needles: &[&str]| -> bool {
        let Some(package) = package.as_mut() else {
            return false;
        };
        let Ok(mut part) = package.open(key) else {
            return false;
        };
        let mut text = String::new();
        std::io::Read::read_to_string(&mut part, &mut text).is_ok()
            && needles.iter().any(|n| text.contains(n))
    };
    let mut groups = Vec::new();
    let mut slicers = BTreeSet::new();
    for (owner, edges) in &source.edges {
        let sheet = source.reach_of(owner) == Reach::Sheet;
        let workbook = source.reach_of(owner) == Reach::Workbook;
        let mut legacy = BTreeSet::new();
        for edge in edges {
            if sheet && LEGACY.contains(&edge.kind.as_str()) {
                legacy.insert(edge.target.clone());
            }
            if (sheet || workbook) && SLICERS.contains(&edge.kind.as_str()) {
                slicers.insert(edge.target.clone());
            }
            if sheet && edge.kind == "drawing" {
                if holds(&edge.target, &["a14:compatExt", ":compatExt"]) {
                    legacy.insert(edge.target.clone());
                }
                if holds(
                    &edge.target,
                    &["/drawing/2010/slicer", "/drawing/2012/timeslicer"],
                ) {
                    slicers.insert(edge.target.clone());
                }
            }
        }
        if !legacy.is_empty() {
            groups.push(legacy);
        }
    }
    if !slicers.is_empty() {
        groups.push(slicers);
    }
    groups
}

/// Whether `key`, a chart part, is bound to a pivot by its `c:pivotSource` text (D5.2): such a
/// chart's drawing is T-4950's to write from the model.
fn is_pivot_chart(package: &mut Option<Package<Cursor<&[u8]>>>, key: &str) -> bool {
    let Some(package) = package.as_mut() else {
        return false;
    };
    let Ok(mut part) = package.open(key) else {
        return false;
    };
    let mut text = String::new();
    std::io::Read::read_to_string(&mut part, &mut text).is_ok() && text.contains("pivotSource")
}

/// Every source part a regenerated save left out, each with its reason (D5.1, D5.2, D5.6), in
/// the source's entry order. `output` is the saved package, `kept` the source part keys the save
/// wrote anyway (T-4950's pivot drawings). Roots and groups are decided in source part-name order.
pub(super) fn regenerated(
    bytes: &[u8],
    output: &[u8],
    kept: &BTreeSet<String>,
    limits: &ImportLimits,
) -> Result<Vec<Dropped>, String> {
    let source = Graph::read(bytes, limits)?;
    let written = Graph::read(output, limits)?;
    let written_ids: BTreeSet<String> = written.identities().into_values().collect();
    let source_ids = source.identities();
    let mut package = Package::new(Cursor::new(bytes), *limits).ok();
    // The reason of each part by key; parts not in it are written or kept.
    let mut reasons: BTreeMap<String, DropReason> = BTreeMap::new();
    let mut decide_unit = |unit: &BTreeSet<String>, reasons: &mut BTreeMap<String, DropReason>| {
        let mut closure = BTreeSet::new();
        let mut limit = None;
        for root in unit {
            match source.closure(root) {
                Ok(parts) => closure.extend(parts),
                Err(l) => limit = Some(l),
            }
        }
        if closure.len() > MAX_CLOSURE_SIZE {
            limit = Some("closure size");
        }
        let regenerated = closure.iter().any(|p| {
            matches!(source.policy(p), Some(Policy::Modelled | Policy::Drop))
                || (source.reached.get(p).is_some_and(|(_, k)| k == "chart")
                    && is_pivot_chart(&mut package, p))
        });
        let reason = match (limit, regenerated) {
            (Some(l), _) => DropReason::LimitExceeded {
                limit: l.to_string(),
            },
            (None, true) => DropReason::ClosureReachesRegenerated,
            // Keepable: no writer yet (T-5000 onward).
            (None, false) => DropReason::NotPreserved,
        };
        for part in closure.iter().chain(unit) {
            let replayed = matches!(
                source.policy(part),
                Some(Policy::Replay | Policy::ReplayIfClean) | None
            );
            if replayed && !kept.contains(part) {
                reasons
                    .entry(part.clone())
                    .or_insert_with(|| reason.clone());
            }
        }
    };
    let groups = coupling_groups(&source, bytes, limits);
    let grouped: BTreeSet<String> = groups.iter().flatten().cloned().collect();
    for group in &groups {
        decide_unit(group, &mut reasons);
    }
    let mut keys: Vec<String> = source.entries.iter().map(|e| part_key(e)).collect();
    keys.sort();
    for key in &keys {
        if grouped.contains(key) || kept.contains(key) || reasons.contains_key(key) {
            continue;
        }
        match source.policy(key) {
            Some(Policy::Modelled) => {
                if !source_ids
                    .get(key)
                    .is_some_and(|id| written_ids.contains(id))
                {
                    reasons.insert(key.clone(), DropReason::NotPreserved);
                }
            }
            Some(Policy::Drop) => {
                reasons.insert(key.clone(), DropReason::Policy);
            }
            // The export writes no macro-enabled package yet (T-5010): the macros are lost
            // because nothing writes them, not because the caller asked for a plain file.
            Some(Policy::Active) => {
                reasons.insert(key.clone(), DropReason::NotPreserved);
            }
            Some(Policy::Replay | Policy::ReplayIfClean) | None => {
                decide_unit(&BTreeSet::from([key.clone()]), &mut reasons);
            }
        }
    }
    Ok(source
        .entries
        .iter()
        .filter_map(|e| {
            reasons
                .get(&part_key(e))
                .map(|r| Dropped::new(e.clone(), r.clone()))
        })
        .collect())
}

/// The calcChain parts of `bytes` (by the relationship that reaches them), within `limits`.
pub(super) fn calc_chains(bytes: &[u8], limits: &ImportLimits) -> Result<Vec<String>, String> {
    let graph = Graph::read(bytes, limits)?;
    Ok(graph
        .entries
        .iter()
        .filter(|e| graph.policy(&part_key(e)) == Some(Policy::Drop))
        .filter(|e| {
            graph
                .reached
                .get(&part_key(e))
                .is_some_and(|(_, k)| k == "calcChain")
        })
        .cloned()
        .collect())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::io::{Cursor, Write as _};

    use super::{policy_of, regenerated, Graph, Policy, Reach, MAX_CLOSURE_DEPTH};
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
        let parts: Vec<(&str, String)> =
            parts.iter().map(|(n, t)| (n.as_str(), t.clone())).collect();
        let deep = package(&[("rId9", "next", "../x/p0.xml", false)], &[], &parts);
        let graph = Graph::read(&deep, &limits()).expect("reads");
        assert_eq!(graph.closure("x/p0.xml"), Err("closure depth"));
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
}
