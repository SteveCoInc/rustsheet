//! What a save left out of the package, and why (SPEC-0033 D7, SPEC-0034 D4, T-4562).
//!
//! - a regenerated save names each source part it did not write, with the ownership matrix's
//!   reason ([`super::ownership`], SPEC-0034 stage 5): a modelled part the output lacks, a part
//!   whose closure reaches what the export regenerates, a limit, a policy drop, and, until the
//!   writers of T-5000 onward land, every part the matrix would keep (`NotPreserved`);
//! - the Patched tier names the calcChain it dropped, and the Verbatim tier drops nothing;
//! - a source the report cannot inventory within the load's limits gives one `LimitExceeded`
//!   entry with an empty item, never an empty list that would claim nothing was dropped.

use std::collections::BTreeSet;

use super::SaveTier;
use crate::import::{zip_entry_names, ImportLimits, PartNames};

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

/// The source parts a regenerated save left out, each with its reason: the ownership matrix's
/// decisions (SPEC-0034 D5.1, D5.2, D5.6; [`super::ownership`]). `output` is the saved package,
/// `kept` the source part keys it wrote anyway, `limits` the ones the workbook was loaded with.
pub(super) fn regenerated(
    source: &[u8],
    output: &[u8],
    kept: &BTreeSet<String>,
    limits: &ImportLimits,
) -> Vec<Dropped> {
    super::ownership::regenerated(source, output, kept, limits)
        .unwrap_or_else(|_| vec![unreadable("the source package")])
}

/// The calcChain a patched save dropped: the source's, when `output` no longer holds it.
pub(super) fn patched(source: &[u8], output: &[u8], limits: &ImportLimits) -> Vec<Dropped> {
    let chains = match super::ownership::calc_chains(source, limits) {
        Ok(chains) => chains,
        Err(_) => return vec![unreadable("the source package")],
    };
    let Some(out) = zip_entry_names(output) else {
        return vec![unreadable("the saved package")];
    };
    let out = PartNames::new(out.iter().map(String::as_str));
    chains
        .into_iter()
        .filter(|entry| out.entry(entry).is_none())
        .map(|entry| Dropped::new(entry, DropReason::Policy))
        .collect()
}

/// The entry for a package the report could not inventory within the load's limits: an empty
/// item, so it names no part, and the reason.
fn unreadable(what: &str) -> Dropped {
    Dropped::new(
        "",
        DropReason::LimitExceeded {
            limit: format!("{what} could not be inventoried within the load's limits"),
        },
    )
}
