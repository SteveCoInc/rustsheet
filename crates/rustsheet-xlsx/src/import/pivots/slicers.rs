//! Slicer and timeline caches as pivot filters (T-4188): the x14 `slicerCacheDefinition` and x15
//! `timelineCacheDefinition` parts the workbook's extension list names (`x14:slicerCaches`,
//! `x15:timelineCacheRefs`), each read for the pivots it drives (its `pivotTables`, by sheet
//! `tabId` and pivot name) and the cache field it filters (`sourceName`).
//!
//! A tabular slicer's items (`data/tabular/items/i`, `x` an index into the field's cache items,
//! `s` selected) become the field's [`PivotField::slicer`], and a timeline's
//! `state/selection` its [`PivotField::timeline`]; the engine applies both
//! (`rustsheet_engine::pivot::DateSpan`; docs/excel-semantics/pivots.md "Slicers and timelines
//! filter the pivot they drive"). The parts themselves are not modelled: a save keeps them only
//! where the source-preserving export keeps the package (SPEC-0034), as for every part the
//! model does not hold. How Excel saves a slicer's state is UNVERIFIED (T-4956), so no part is
//! written from the model.
//!
//! Not modelled, each cache skipped and logged: an OLAP slicer (`data/olap`; the importer skips
//! OLAP pivot caches too), a table slicer (`x15:tableSlicerCache`: it filters a table, not a
//! pivot), and a `timelinePivotFilter` (Excel for the web refuses the file, pivots.md
//! "Capability matrix 3").

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, Read, Seek};

use rustsheet_engine::pivot::{DateSpan, PivotCache, PivotTable, MAX_ITEMS};
use rustsheet_engine::types::DateSystem;

use super::super::iso_date::pivot_cache_date_serial;
use super::super::package::{part_key, resolve_relative_target, ImportLimits, Package};
use super::super::Relationship;
use super::table::flag;
use super::{at, attribute, local_name, log_skip, skip, walk, Skip, Tag, MAX_PIVOTS};

/// Most slicer and timeline caches read from one workbook; the ones past it are dropped.
pub(crate) const MAX_SLICER_CACHES: usize = 1_000;

const SLICER: &str = "slicerCacheDefinition";
const TIMELINE: &str = "timelineCacheDefinition";

/// One slicer or timeline cache, as far as it filters pivots.
#[derive(Debug, PartialEq)]
pub(super) struct SlicerCache {
    /// `sourceName`: the name of the cache field it filters.
    source: String,
    /// `pivotTables/pivotTable`: the `tabId` (the sheet's `sheetId`) and name of each pivot it
    /// drives.
    pivots: Vec<(u32, String)>,
    selection: Selection,
}

/// What a cache selects.
#[derive(Debug, PartialEq)]
enum Selection {
    /// A tabular slicer's items (`i@x`, selected when `s`), read once per cache whatever the
    /// pivots it drives: a bitset of the unselected ones, and whether any is selected. An `x`
    /// past [`MAX_ITEMS`] names no cache item, so the bitset never outgrows a field.
    Items {
        unselected: Vec<u64>,
        any_selected: bool,
    },
    /// A timeline's `selection` (`startDate`, `endDate`), as serials; `None` when it has none,
    /// which filters nothing (probes/t4767-pivot-timelines TN).
    Dates(Option<(f64, f64)>),
}

/// Reads a slicer or timeline cache part.
pub(super) fn read_slicer_cache<B: BufRead>(
    part: B,
    limits: &ImportLimits,
    date_system: DateSystem,
) -> Result<SlicerCache, Skip> {
    let mut root = String::new();
    let mut source = None;
    let mut pivots = Vec::new();
    // A part may list one pivot many times: each is kept once.
    let mut seen: HashSet<(u32, String)> = HashSet::new();
    let mut tabular = false;
    let mut items = 0_usize;
    let mut unselected: Vec<u64> = Vec::new();
    let mut any_selected = false;
    let mut dates = None;
    walk(part, |path, tag| {
        let Tag::Open(element) = tag else {
            return Ok(());
        };
        let name = local_name(element);
        if path.is_empty() {
            if name != SLICER && name != TIMELINE {
                return skip("not a slicer or timeline cache");
            }
            let Some(field) = attribute(element, "sourceName")? else {
                return skip("no sourceName");
            };
            limits.check_text(&field)?;
            source = Some(field);
            root = name;
            return Ok(());
        }
        match name.as_str() {
            "pivotTable" if at(path, &[root.as_str(), "pivotTables"]) => {
                if pivots.len() >= MAX_PIVOTS {
                    return skip(format!("drives more than {MAX_PIVOTS} pivots"));
                }
                let tab = attribute(element, "tabId")?.and_then(|t| t.parse::<u32>().ok());
                let (Some(tab), Some(pivot)) = (tab, attribute(element, "name")?) else {
                    return skip("a pivotTable without a tabId and a name");
                };
                limits.check_text(&pivot)?;
                if seen.insert((tab, pivot.clone())) {
                    pivots.push((tab, pivot));
                }
            }
            "olap" if at(path, &[SLICER, "data"]) => {
                return skip("an OLAP slicer is not modelled");
            }
            "tabular" if at(path, &[SLICER, "data"]) => tabular = true,
            "i" if at(path, &[SLICER, "data", "tabular", "items"]) => {
                if items >= MAX_ITEMS {
                    return skip(format!("more than {MAX_ITEMS} slicer items"));
                }
                items += 1;
                let x = attribute(element, "x")?.and_then(|x| x.parse::<u32>().ok());
                let Some(x) = x else {
                    return skip("a slicer item without a valid x");
                };
                if flag(element, "s", false)? {
                    any_selected = true;
                } else if (x as usize) < MAX_ITEMS {
                    let word = x as usize / 64;
                    if unselected.len() <= word {
                        unselected.resize(word + 1, 0);
                    }
                    if let Some(bits) = unselected.get_mut(word) {
                        *bits |= 1 << (x % 64);
                    }
                }
            }
            "tableSlicerCache" => return skip("a table slicer is not a pivot filter"),
            "timelinePivotFilter" if at(path, &[TIMELINE]) => {
                return skip("a timelinePivotFilter is not modelled");
            }
            "selection" if at(path, &[TIMELINE, "state"]) => {
                let date = |key: &str| -> Result<Option<f64>, Skip> {
                    Ok(attribute(element, key)?
                        .and_then(|text| pivot_cache_date_serial(&text, date_system)))
                };
                let (Some(start), Some(end)) = (date("startDate")?, date("endDate")?) else {
                    return skip("a timeline selection without valid dates");
                };
                dates = Some((start, end));
            }
            _ => {}
        }
        Ok(())
    })?;
    let Some(source) = source else {
        return skip("an empty part");
    };
    let selection = if root == TIMELINE {
        Selection::Dates(dates)
    } else if tabular {
        Selection::Items {
            unselected,
            any_selected,
        }
    } else {
        return skip("a slicer cache without tabular data");
    };
    Ok(SlicerCache {
        source,
        pivots,
        selection,
    })
}

/// Reads each slicer and timeline cache `slicer_caches` names (their `r:id`s in the workbook
/// relationships `rels`) and applies it to the pivots it drives, one part at a time, so no more
/// than one cache's items is held. A cache that does not read is logged and skipped.
pub(super) fn load_slicer_caches<R: Read + Seek>(
    archive: &mut Package<R>,
    rels: &HashMap<String, Relationship>,
    slicer_caches: &[String],
    tables: &mut [PivotTable],
    pivot_caches: &[PivotCache],
    date_system: DateSystem,
) {
    if tables.is_empty() || slicer_caches.is_empty() {
        return;
    }
    let by_name = pivots_by_name(tables);
    let limits = *archive.limits();
    // A relationship id or a part listed twice is read once: applying a cache again changes
    // nothing, and would only repeat the work.
    let mut seen = std::collections::HashSet::new();
    let mut seen_parts = std::collections::HashSet::new();
    for rel_id in slicer_caches.iter().take(MAX_SLICER_CACHES) {
        if !seen.insert(rel_id.as_str()) {
            continue;
        }
        let part = (rels.get(rel_id))
            .filter(|rel| {
                !rel.external
                    && (rel.rel_type.ends_with("/slicerCache")
                        || rel.rel_type.ends_with("/timelineCache"))
            })
            .and_then(|rel| resolve_relative_target("xl", &rel.target));
        let Some(part) = part else {
            log_skip(
                "slicer cache",
                rel_id,
                &Skip("no internal slicer or timeline cache relationship".into()),
            );
            continue;
        };
        if !seen_parts.insert(part_key(&part)) {
            continue;
        }
        let read = (archive.open(&part).map_err(Skip::from))
            .and_then(|reader| read_slicer_cache(reader, &limits, date_system));
        match read {
            Ok(cache) => apply_slicer_cache(&cache, tables, &by_name, pivot_caches),
            Err(reason) => log_skip("slicer cache", &part, &reason),
        }
    }
}

/// Sets the filter state `cache` gives each pivot of `tables` it drives, over `pivot_caches`.
/// A pivot it names that is not loaded, or whose cache has no field `sourceName`, is left as
/// it is. Two slicers on one field of a pivot both apply (an item either leaves unselected is
/// left out), and of two timelines the later one in the workbook's list applies (both
/// UNVERIFIED, T-4962).
pub(super) fn apply_slicer_cache(
    cache: &SlicerCache,
    tables: &mut [PivotTable],
    by_name: &PivotsByName,
    pivot_caches: &[PivotCache],
) {
    for (tab, name) in &cache.pivots {
        let Some(table) = (by_name.get(tab))
            .and_then(|names| names.get(name.as_str()))
            .and_then(|&index| tables.get_mut(index))
        else {
            continue;
        };
        let Some(fields) = pivot_caches.get(table.cache as usize).map(|c| &c.fields) else {
            continue;
        };
        let Some((index, field)) =
            (fields.iter().enumerate()).find(|(_, f)| f.name == cache.source)
        else {
            continue;
        };
        let Some(pivot_field) = table.fields.get_mut(index) else {
            continue;
        };
        match &cache.selection {
            Selection::Items {
                unselected,
                any_selected,
            } => {
                // An item the cache does not list shows, and a slicer that selects nothing
                // filters nothing (UNVERIFIED, T-4962: the probes list every item and select
                // one or more).
                if !any_selected {
                    continue;
                }
                let count = field.items.len();
                let bits = pivot_field
                    .slicer
                    .get_or_insert_with(|| vec![0; count.div_ceil(64)]);
                // The cache's bitset, cut to the field's items: one pass over the words.
                for (word, from) in bits.iter_mut().zip(unselected) {
                    *word |= from;
                }
                if let Some(last) = bits.last_mut().filter(|_| count % 64 != 0) {
                    *last &= (1_u64 << (count % 64)) - 1;
                }
            }
            Selection::Dates(Some((start, end))) => {
                if let Some(span) = DateSpan::from_serials(*start, *end) {
                    pivot_field.timeline = Some(span);
                }
            }
            Selection::Dates(None) => {}
        }
    }
}

/// The pivots of a workbook by their sheet's `sheetId`, then by name: what a cache's
/// `pivotTables` name them by.
pub(super) type PivotsByName = HashMap<u32, HashMap<String, usize>>;

/// [`PivotsByName`] of `tables`, each an index into it.
pub(super) fn pivots_by_name(tables: &[PivotTable]) -> PivotsByName {
    let mut out: PivotsByName = HashMap::new();
    for (index, table) in tables.iter().enumerate() {
        (out.entry(table.sheet_id).or_default()).insert(table.name.clone(), index);
    }
    out
}

#[cfg(test)]
mod tests;
