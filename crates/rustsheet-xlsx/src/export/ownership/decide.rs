//! The decisions of a regenerated save (SPEC-0034 D5.2, D5.6): which units the source holds, what
//! each unit's closure reaches, and the reason each source part the save did not write is
//! reported with.
//!
//! - **Roots** are the parts reached from the package, the workbook or a sheet, and the parts no
//!   relationship reaches. A part reached only from another part is never decided alone: it
//!   takes the verdict of the roots whose closures hold it.
//! - **Coupling groups** are per sheet: the legacy drawing group, and the slicer and timeline
//!   group, which joins the caches its slicers and timelines name. Groups that share a part are
//!   merged, so the union is one unit.
//! - **The shared-part rule.** A part held by a keepable unit is keepable; otherwise it takes its
//!   units' drop reason, a limit before a closure reaching regenerated content.
//! - **Bounds.** Each part is probed at most once and for at most [`MAX_PROBE_BYTES`]; each root's
//!   closure is computed once; all units together visit at most [`MAX_WORK`] closure parts, past
//!   which every remaining unit is `LimitExceeded`. A part the probe cannot read makes its unit
//!   `LimitExceeded`, never silently "not coupled".

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{Cursor, Read as _};

use super::{Graph, Policy, Reach, MAX_CLOSURE_SIZE};
use crate::export::patch::tag::attr_span;
use crate::export::report::{DropReason, Dropped};
use crate::import::{part_key, ImportLimits, Package};

/// D5.6: the most closure parts all units of one save may visit together.
const MAX_WORK: usize = 1_000_000;
/// D5.6: the most bytes of one part a probe reads.
const MAX_PROBE_BYTES: u64 = 1 << 20;

/// The relationship kinds of a sheet's legacy drawing group (D5.2).
const LEGACY: &[&str] = &[
    "vmlDrawing",
    "comments",
    "threadedComment",
    "ctrlProp",
    "control",
    "oleObject",
    "package",
];
/// The relationship kinds of a sheet's slicer and timeline group (D5.2).
const SLICERS: &[&str] = &["slicer", "timeline"];
/// The workbook relationship kinds of the caches slicers and timelines name.
const CACHES: &[&str] = &["slicerCache", "timelineCache"];

/// Bounded, cached reads of a package's parts: each part is read once, at most
/// [`MAX_PROBE_BYTES`] of it. `None` is a part that cannot be read (missing, over the budget, not
/// UTF-8).
struct Probe<'a> {
    package: Option<Package<Cursor<&'a [u8]>>>,
    texts: HashMap<String, Option<String>>,
}

impl<'a> Probe<'a> {
    fn new(bytes: &'a [u8], limits: &ImportLimits) -> Self {
        Probe {
            package: Package::new(Cursor::new(bytes), *limits).ok(),
            texts: HashMap::new(),
        }
    }

    fn text(&mut self, key: &str) -> Option<&str> {
        if !self.texts.contains_key(key) {
            let read = self.package.as_mut().and_then(|p| {
                let mut text = String::new();
                p.open(key)
                    .ok()?
                    .take(MAX_PROBE_BYTES)
                    .read_to_string(&mut text)
                    .ok()?;
                Some(text)
            });
            self.texts.insert(key.to_string(), read);
        }
        self.texts.get(key).and_then(Option::as_deref)
    }

    /// Whether `key` holds any of `needles`; `None` when it cannot be read.
    fn contains(&mut self, key: &str, needles: &[&str]) -> Option<bool> {
        self.text(key)
            .map(|t| needles.iter().any(|n| t.contains(n)))
    }

    /// Every value of the attribute `name` across the start tags of `key`.
    fn attr_values(&mut self, key: &str, name: &str) -> Vec<String> {
        let Some(text) = self.text(key) else {
            return Vec::new();
        };
        let mut values = Vec::new();
        let mut rest = text;
        while let Some(start) = rest.find('<') {
            let Some(len) = rest[start..].find('>') else {
                break;
            };
            let tag = &rest[start..start + len];
            if let Some((a, b)) = attr_span(tag, name) {
                values.push(tag[a..b].to_string());
            }
            rest = &rest[start + len..];
        }
        values
    }

    /// The attribute `name` of the root element of `key`.
    fn root_attr(&mut self, key: &str, name: &str) -> Option<String> {
        let text = self.text(key)?;
        let mut rest = text;
        loop {
            let start = rest.find('<')?;
            let len = rest[start..].find('>')?;
            let tag = &rest[start..start + len];
            if !tag.starts_with("<?") && !tag.starts_with("<!") {
                return attr_span(tag, name).map(|(a, b)| tag[a..b].to_string());
            }
            rest = &rest[start + len..];
        }
    }
}

/// Each reached part's identity, the same in a source and in a save of it (D5.1): the owner's
/// identity, the relationship kind and a label the export keeps. A worksheet is labelled by its
/// `sheetId`, a table and a pivot table by their `name`, and a pivot cache by the names of the
/// pivot tables that use it (the export renumbers `cacheId`). Anything else is labelled by its
/// ordinal among the owner's relationships of that kind: when a save writes fewer of them, the
/// last are the ones reported, which can name the wrong sibling. The round-trip suite scores such a
/// misnamed part `changed` (SPEC-0033 D7.3), so it cannot pass silently.
fn identities(graph: &Graph, probe: &mut Probe<'_>) -> HashMap<String, String> {
    let mut tables_of_cache: HashMap<String, Vec<String>> = HashMap::new();
    for (key, (_, kind)) in &graph.reached {
        if kind == "pivotTable" {
            if let (Some(cache), Some(name)) = (
                probe.root_attr(key, "cacheId"),
                probe.root_attr(key, "name"),
            ) {
                tables_of_cache.entry(cache).or_default().push(name);
            }
        }
    }
    for names in tables_of_cache.values_mut() {
        names.sort();
    }
    let workbook = graph.workbook.clone().unwrap_or_default();
    let mut ids = HashMap::new();
    let mut queue = std::collections::VecDeque::from([(String::new(), String::new())]);
    let mut seen = BTreeSet::new();
    while let Some((part, id)) = queue.pop_front() {
        let mut ordinal: HashMap<&str, usize> = HashMap::new();
        for edge in graph.edges.get(&part).into_iter().flatten() {
            let n = ordinal.entry(edge.kind.as_str()).or_default();
            let named = match edge.kind.as_str() {
                "worksheet" if part == workbook => graph
                    .sheet_ids
                    .get(&edge.id)
                    .map(|s| format!("sheetId {s}")),
                "table" | "pivotTable" => probe
                    .root_attr(&edge.target, "name")
                    .map(|s| format!("name {s}")),
                "pivotCacheDefinition" if part == workbook => graph
                    .cache_ids
                    .get(&edge.id)
                    .and_then(|c| tables_of_cache.get(c))
                    .map(|names| format!("tables {}", names.join(","))),
                _ => None,
            };
            let child = format!(
                "{id}/{}[{}]",
                edge.kind,
                named.unwrap_or_else(|| n.to_string())
            );
            *n += 1;
            if seen.insert(edge.target.clone()) {
                ids.insert(edge.target.clone(), child.clone());
                queue.push_back((edge.target.clone(), child));
            }
        }
    }
    ids
}

/// One unit's verdict.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Verdict {
    /// Keepable: no writer yet (T-5000 onward), so still reported `NotPreserved`.
    Keepable,
    /// A limit, or a part that could not be read.
    Limit(String),
    /// Its closure reaches what the export regenerates.
    Regenerated,
}

/// The units of `source`: its coupling groups, merged where they share a part, and every root
/// no group holds. Each is a set of part keys, with whether a part its detection needed could not
/// be read.
fn units(source: &Graph, probe: &mut Probe<'_>) -> Vec<(BTreeSet<String>, bool)> {
    let mut groups: Vec<(BTreeSet<String>, bool)> = Vec::new();
    // Each cache's part key by the name its definition carries.
    let mut caches: HashMap<String, String> = HashMap::new();
    if let Some(workbook) = &source.workbook {
        for edge in source.edges.get(workbook).into_iter().flatten() {
            if CACHES.contains(&edge.kind.as_str()) {
                if let Some(name) = probe.root_attr(&edge.target, "name") {
                    caches.insert(name, edge.target.clone());
                }
            }
        }
    }
    for (owner, edges) in &source.edges {
        if source.reach_of(owner) != Reach::Sheet {
            continue;
        }
        let (mut legacy, mut slicers) = ((BTreeSet::new(), false), (BTreeSet::new(), false));
        for edge in edges {
            let kind = edge.kind.as_str();
            if LEGACY.contains(&kind) {
                legacy.0.insert(edge.target.clone());
            }
            if SLICERS.contains(&kind) {
                slicers.0.insert(edge.target.clone());
                for cache in probe.attr_values(&edge.target, "cache") {
                    if let Some(part) = caches.get(&cache) {
                        slicers.0.insert(part.clone());
                    }
                }
            }
            if kind == "drawing" {
                match probe.contains(&edge.target, &[":compatExt"]) {
                    Some(true) => {
                        legacy.0.insert(edge.target.clone());
                    }
                    Some(false) => {}
                    // Unreadable: kept with the group it might belong to, which then drops.
                    None => {
                        legacy.0.insert(edge.target.clone());
                        legacy.1 = true;
                    }
                }
                let frames = ["/drawing/2010/slicer", "/drawing/2012/timeslicer"];
                if probe.contains(&edge.target, &frames) == Some(true) {
                    slicers.0.insert(edge.target.clone());
                }
            }
        }
        for group in [legacy, slicers] {
            if !group.0.is_empty() {
                groups.push(group);
            }
        }
    }
    // Merge groups that share a part, until none do.
    let mut merged: Vec<(BTreeSet<String>, bool)> = Vec::new();
    for (mut parts, mut unreadable) in groups {
        let mut rest = Vec::new();
        for other in merged {
            if other.0.is_disjoint(&parts) {
                rest.push(other);
            } else {
                parts.extend(other.0);
                unreadable |= other.1;
            }
        }
        rest.push((parts, unreadable));
        merged = rest;
    }
    let grouped: BTreeSet<&String> = merged.iter().flat_map(|g| &g.0).collect();
    let roots: Vec<String> = (source.entries.iter())
        .map(|e| part_key(e))
        .filter(|k| !grouped.contains(k) && is_root(source, k))
        .filter(|k| replayed(source.policy(k)))
        .collect();
    let mut units: Vec<(BTreeSet<String>, bool)> = merged;
    units.extend(roots.into_iter().map(|r| (BTreeSet::from([r]), false)));
    units.sort_by(|a, b| a.0.first().cmp(&b.0.first()));
    units
}

/// Whether a part is a root: reached from the package, the workbook or a sheet, or reached by no
/// relationship.
fn is_root(source: &Graph, key: &str) -> bool {
    source
        .reached
        .get(key)
        .is_none_or(|(owner, _)| source.reach_of(owner) != Reach::Other)
}

/// Whether a policy is one a regenerated save would copy (D5.1).
fn replayed(policy: Policy) -> bool {
    matches!(policy, Policy::Replay | Policy::ReplayIfClean)
}

/// Every source part a regenerated save left out, each with its reason, in the source's entry
/// order. `output` is the saved package, `kept` the source part keys the save wrote anyway
/// (T-4950's pivot drawings). `Err` names the package that could not be read within `limits`.
pub(in crate::export) fn regenerated(
    bytes: &[u8],
    output: &[u8],
    kept: &BTreeSet<String>,
    limits: &ImportLimits,
) -> Result<Vec<Dropped>, &'static str> {
    regenerated_within(bytes, output, kept, limits, MAX_WORK)
}

/// [`regenerated`] with `max_work` closure-part visits, so a test can reach the work limit.
pub(super) fn regenerated_within(
    bytes: &[u8],
    output: &[u8],
    kept: &BTreeSet<String>,
    limits: &ImportLimits,
    max_work: usize,
) -> Result<Vec<Dropped>, &'static str> {
    let source = Graph::read(bytes, limits).map_err(|_| "the source package")?;
    let written = Graph::read(output, limits).map_err(|_| "the saved package")?;
    let mut probe = Probe::new(bytes, limits);
    let mut out_probe = Probe::new(output, limits);
    let written_ids: BTreeSet<String> =
        identities(&written, &mut out_probe).into_values().collect();
    let source_ids = identities(&source, &mut probe);
    // Each replayed part's verdicts across the units whose closures hold it.
    let mut verdicts: BTreeMap<String, Verdict> = BTreeMap::new();
    let mut closures: HashMap<String, Result<BTreeSet<String>, &'static str>> = HashMap::new();
    let mut work = 0usize;
    for (unit, unreadable) in units(&source, &mut probe) {
        let mut closure = BTreeSet::new();
        let mut verdict = unreadable.then(|| Verdict::Limit("part read".into()));
        if work > max_work {
            verdict = Some(Verdict::Limit("closure work".into()));
        }
        for root in &unit {
            let found = closures
                .entry(root.clone())
                .or_insert_with(|| source.closure(root));
            match found {
                Ok(parts) => {
                    work += parts.len();
                    closure.extend(parts.iter().cloned());
                }
                Err(limit) => verdict = verdict.or_else(|| Some(Verdict::Limit((*limit).into()))),
            }
        }
        if closure.len() > MAX_CLOSURE_SIZE {
            verdict = verdict.or_else(|| Some(Verdict::Limit("closure size".into())));
        }
        let verdict = verdict.unwrap_or_else(|| {
            let mut regenerated = false;
            let mut unread = false;
            for part in &closure {
                match source.policy(part) {
                    Policy::Modelled | Policy::Drop => regenerated = true,
                    _ => {}
                }
                if source.reached.get(part).is_some_and(|(_, k)| k == "chart") {
                    match probe.contains(part, &["pivotSource"]) {
                        Some(true) => regenerated = true,
                        Some(false) => {}
                        None => unread = true,
                    }
                }
            }
            match (unread, regenerated) {
                (true, _) => Verdict::Limit("part read".into()),
                (false, true) => Verdict::Regenerated,
                (false, false) => Verdict::Keepable,
            }
        });
        for part in closure.iter().chain(&unit) {
            if replayed(source.policy(part)) && !kept.contains(part) {
                // The shared-part rule (D5.2): a part a keepable unit holds is keepable, so the
                // least verdict wins. TODO(T-5000): a keepable part is then written, not reported.
                let entry = verdicts
                    .entry(part.clone())
                    .or_insert_with(|| verdict.clone());
                if verdict < *entry {
                    *entry = verdict.clone();
                }
            }
        }
    }
    let mut dropped = Vec::new();
    for entry in &source.entries {
        let key = part_key(entry);
        if kept.contains(&key) {
            continue;
        }
        let reason = match source.policy(&key) {
            Policy::Modelled
                if source_ids
                    .get(&key)
                    .is_some_and(|id| written_ids.contains(id)) =>
            {
                continue;
            }
            Policy::Modelled => DropReason::NotPreserved,
            Policy::Drop => DropReason::Policy,
            // The export writes no macro-enabled package yet (T-5010): the macros are lost because
            // nothing writes them, not because the caller asked for a plain file.
            Policy::Active => DropReason::NotPreserved,
            Policy::Replay | Policy::ReplayIfClean => match verdicts.get(&key) {
                Some(Verdict::Regenerated) => DropReason::ClosureReachesRegenerated,
                Some(Verdict::Limit(limit)) => DropReason::LimitExceeded {
                    limit: limit.clone(),
                },
                // Keepable, or held by no unit (reached only from a modelled part).
                Some(Verdict::Keepable) | None => DropReason::NotPreserved,
            },
        };
        dropped.push(Dropped::new(entry.clone(), reason));
    }
    Ok(dropped)
}

/// The calcChain parts of `bytes` (by the relationship that reaches them), within `limits`.
pub(in crate::export) fn calc_chains(
    bytes: &[u8],
    limits: &ImportLimits,
) -> Result<Vec<String>, &'static str> {
    let graph = Graph::read(bytes, limits).map_err(|_| "the source package")?;
    Ok(graph
        .entries
        .iter()
        .filter(|e| (graph.reached.get(&part_key(e))).is_some_and(|(_, kind)| kind == "calcChain"))
        .cloned()
        .collect())
}
