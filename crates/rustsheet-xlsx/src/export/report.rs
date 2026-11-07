//! What a save left out of the package, and why (SPEC-0033 D7, SPEC-0034 D4, T-4562).
//!
//! The report is coarse until stage 5 (SPEC-0034 D5): a regenerated save names each source part
//! the regenerating export does not write and did not keep, one `NotPreserved` entry each, and
//! calcChain, which no tier ever writes once it would be stale, as `Policy`. The Patched tier names
//! the calcChain it dropped. The Verbatim tier drops nothing.

use std::collections::{BTreeSet, HashMap};
use std::io::Cursor;

use super::SaveTier;
use crate::import::{
    part_key, part_relationships, resolve_target, ImportLimits, Package, PartNames,
};

/// What a save wrote and what it left out (SPEC-0034 D4).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SaveReport {
    /// How the package was written.
    pub tier: SaveTier,
    /// One entry per source part not written, in the source's part order.
    pub dropped: Vec<Dropped>,
}

impl SaveReport {
    /// A report of `tier` with these drops.
    #[must_use]
    pub fn new(tier: SaveTier, dropped: Vec<Dropped>) -> Self {
        SaveReport { tier, dropped }
    }

    /// The report of a workbook saved with no source package: regenerated, nothing preserved to
    /// drop.
    #[must_use]
    pub fn without_source() -> Self {
        SaveReport {
            tier: SaveTier::Regenerated {
                reason: "the workbook has no source package".into(),
            },
            dropped: Vec::new(),
        }
    }
}

/// One item a save did not write (SPEC-0033 D7).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Dropped {
    /// The source part's name, as its zip entry spells it (`xl/calcChain.xml`). The round-trip
    /// suite turns it into the part's fidelity-item key (SPEC-0033 D2). Element-level entries
    /// come with stage 5's fragments.
    pub item: String,
    /// Why it was not written.
    pub reason: DropReason,
}

impl Dropped {
    /// The item `item` (a source part's name), dropped for `reason`.
    #[must_use]
    pub fn new(item: impl Into<String>, reason: DropReason) -> Self {
        Dropped {
            item: item.into(),
            reason,
        }
    }
}

/// Why a save dropped an item (SPEC-0033 D7).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DropReason {
    /// The workbook carries no preserved copy of it (today: everything the model does not hold).
    NotPreserved,
    /// The preservation policy never writes it (calcChain, digital signatures).
    Policy,
    /// A structural edit on this sheet made range-bound content stale (decision 0275 D4).
    StaleAfterStructuralEdit {
        /// The edited sheet.
        sheet: String,
    },
    /// A content edit made kept content stale, on this sheet or workbook-wide (SPEC-0034 D5.5).
    StaleAfterEdit {
        /// The sheet, `None` for workbook-wide content.
        sheet: Option<String>,
    },
    /// Its owner (a sheet, a table) was deleted.
    OwnerDeleted,
    /// Its relationship closure reaches a part the export regenerates (SPEC-0034 D5.2).
    ClosureReachesRegenerated,
    /// Keeping it would exceed a preservation limit.
    LimitExceeded {
        /// The limit (SPEC-0034 D5.6).
        limit: String,
    },
    /// One of its relationship ids could not be remapped.
    UnmappableRelationship,
    /// Active content (VBA) cannot go into a file that is not macro-enabled (decision 0275 D5).
    ActiveContentInPlainFormat,
}

impl DropReason {
    /// The reason's name as SPEC-0033 D7 spells it (`NotPreserved`, `Policy`, ...).
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            DropReason::NotPreserved => "NotPreserved",
            DropReason::Policy => "Policy",
            DropReason::StaleAfterStructuralEdit { .. } => "StaleAfterStructuralEdit",
            DropReason::StaleAfterEdit { .. } => "StaleAfterEdit",
            DropReason::OwnerDeleted => "OwnerDeleted",
            DropReason::ClosureReachesRegenerated => "ClosureReachesRegenerated",
            DropReason::LimitExceeded { .. } => "LimitExceeded",
            DropReason::UnmappableRelationship => "UnmappableRelationship",
            DropReason::ActiveContentInPlainFormat => "ActiveContentInPlainFormat",
        }
    }
}

/// The last segment of the relationship types whose parts the regenerating export writes from
/// the model (SPEC-0034 D5.1's `Modelled` row). The full ownership matrix replaces this list in
/// T-4999.
const MODELLED: &[&str] = &[
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
];

/// The relationship type whose parts no save writes (`Policy`).
const CALC_CHAIN: &str = "calcChain";

/// The source parts a regenerated save left out: every content part reached by a relationship
/// type the export does not model, that `kept` (part keys) does not hold, or that no
/// relationship reaches. A source the importer's reader cannot open reports nothing, since the
/// save itself still succeeded.
pub(super) fn regenerated(source: &[u8], kept: &BTreeSet<String>) -> Vec<Dropped> {
    let Some((entries, kinds)) = inventory(source) else {
        return Vec::new();
    };
    entries
        .into_iter()
        .filter_map(|entry| {
            let key = part_key(&entry);
            let reason = match kinds.get(&key).map(String::as_str) {
                Some(CALC_CHAIN) => DropReason::Policy,
                Some(kind) if MODELLED.contains(&kind) => return None,
                _ if kept.contains(&key) => return None,
                _ => DropReason::NotPreserved,
            };
            Some(Dropped {
                item: entry,
                reason,
            })
        })
        .collect()
}

/// The calcChain a patched save dropped: the source's, when `output` no longer holds it.
pub(super) fn patched(source: &[u8], output: &[u8]) -> Vec<Dropped> {
    let (Some((entries, kinds)), Some(out)) = (inventory(source), entry_names(output)) else {
        return Vec::new();
    };
    let out = PartNames::new(out.iter().map(String::as_str));
    entries
        .into_iter()
        .filter(|entry| kinds.get(&part_key(entry)).is_some_and(|k| k == CALC_CHAIN))
        .filter(|entry| out.entry(entry).is_none())
        .map(|entry| Dropped {
            item: entry,
            reason: DropReason::Policy,
        })
        .collect()
}

/// The source's content parts in entry order (no `.rels`, no `[Content_Types].xml`, no folders),
/// and each part's key mapped to the last segment of the relationship type that reaches it.
fn inventory(source: &[u8]) -> Option<(Vec<String>, HashMap<String, String>)> {
    let names = entry_names(source)?;
    let mut package = Package::new(Cursor::new(source), ImportLimits::default()).ok()?;
    let mut kinds = HashMap::new();
    for rels in names.iter().filter(|n| is_rels(n)) {
        let Some(base) = rels_owner_folder(rels) else {
            continue;
        };
        let Ok(relationships) = part_relationships(&mut package, rels) else {
            continue;
        };
        for (_, rel) in relationships {
            if rel.external {
                continue;
            }
            let kind = rel
                .rel_type
                .rsplit('/')
                .next()
                .unwrap_or_default()
                .to_string();
            kinds
                .entry(part_key(&resolve_target(&base, &rel.target)))
                .or_insert(kind);
        }
    }
    let content = names
        .into_iter()
        .filter(|n| !n.ends_with('/') && !is_rels(n))
        .filter(|n| part_key(n) != part_key("[Content_Types].xml"))
        .collect();
    Some((content, kinds))
}

/// The zip entry names of `bytes`, in directory order.
fn entry_names(bytes: &[u8]) -> Option<Vec<String>> {
    let archive = zip::ZipArchive::new(Cursor::new(bytes)).ok()?;
    archive
        .file_names()
        .map(|n| n.map(|n| n.to_string()))
        .collect::<Result<_, _>>()
        .ok()
}

/// The folder a relationships part's targets resolve from: `_rels/.rels` from the root,
/// `xl/_rels/workbook.xml.rels` from `xl`. `None` for a `.rels` outside a `_rels` folder.
fn rels_owner_folder(rels: &str) -> Option<String> {
    let rels = rels.replace('\\', "/");
    let (folder, _) = rels.rsplit_once("_rels/")?;
    Some(folder.trim_end_matches('/').to_string())
}

/// Whether a zip entry is a relationships part (`.rels`, any case: part names match
/// case-insensitively).
fn is_rels(name: &str) -> bool {
    let key = part_key(name);
    let file = key.rsplit('/').next().unwrap_or_default();
    // `_rels/.rels` has no extension to `Path`: its whole name is `.rels`.
    file == ".rels"
        || std::path::Path::new(file)
            .extension()
            .is_some_and(|e| e == "rels")
}
