//! The value filters and Top-N of a field (T-4165): its items ranked within each tuple of the
//! outer fields of its axis by their value of one data field, over the records still kept
//! (docs/excel-semantics/pivots.md "Top-N filters at their edges", "Several filters on one
//! field", "Filters on an inner field rank per parent").

use std::collections::HashMap;

use crate::expressions::token::Error;

use super::super::accumulate::Accumulator;
use super::super::result::{charge, sort_units, units, Order, PivotComputeError};
use super::super::{Aggregate, PivotCache, PivotTable};
use super::{column, mark_removed, number, FilterCriteria, FilterOperator, Kept, PivotFilter};
use super::{AutoShow, PivotFilterKind};

/// A value filter as a ranking of a field's items within a parent.
pub(super) enum Ranking {
    /// A `value*` filter: each item by its own value, against each condition's number (`None`
    /// for a value that is no number, which no item meets).
    Value {
        and: bool,
        conditions: Vec<(FilterOperator, Option<f64>)>,
    },
    /// A count Top-N (or `AutoShow`): the N largest or smallest and every item tied with the Nth
    /// (pivots.md "Top-N filters at their edges": `TC2`, `TB1`, `TC9`, `NT3`).
    Count { top: bool, n: f64 },
    /// A percent Top-N.
    Percent { top: bool, percent: f64 },
    /// A sum Top-N.
    Sum { top: bool, sum: f64 },
}

impl Ranking {
    /// The ranking of a value filter or Top-N, its numbers parsed once; `None` for any other
    /// filter, and for a pairing of type and criterion that is unprobed, which applies nothing
    /// (UNVERIFIED, T-4799).
    pub(super) fn of(filter: &PivotFilter) -> Option<Self> {
        let finite = |text: &str| number(text).filter(|n| n.is_finite());
        Some(match (filter.kind, &filter.criteria) {
            (PivotFilterKind::Value, FilterCriteria::Custom { and, conditions }) => Self::Value {
                and: *and,
                conditions: (conditions.iter())
                    .map(|c| (c.operator, number(&c.value)))
                    .collect(),
            },
            (PivotFilterKind::Count, FilterCriteria::Top10 { top, value }) => Self::Count {
                top: *top,
                n: finite(value)?,
            },
            (PivotFilterKind::Percent, FilterCriteria::Top10 { top, value }) => Self::Percent {
                top: *top,
                percent: finite(value)?,
            },
            (PivotFilterKind::Sum, FilterCriteria::Top10 { top, value }) => Self::Sum {
                top: *top,
                sum: finite(value)?,
            },
            _ => return None,
        })
    }

    /// A field's `AutoShow` as the count Top-N it is.
    pub(super) fn auto_show(auto_show: AutoShow) -> Self {
        Self::Count {
            top: auto_show.top,
            n: f64::from(auto_show.count),
        }
    }

    /// Which of `items` (their values, sorted by the field's `<items>` order) it keeps. An item
    /// whose value is an error is never kept (UNVERIFIED, T-4799).
    fn keeps(
        &self,
        items: &[Result<f64, Error>],
        work: &mut u64,
    ) -> Result<Vec<bool>, PivotComputeError> {
        // The keep flags and the numbers, then a ranked copy of them.
        charge(work, units(items.len()).saturating_mul(3))?;
        let mut keep = vec![false; items.len()];
        let numbers: Vec<(usize, f64)> = (items.iter().enumerate())
            .filter_map(|(i, value)| value.as_ref().ok().map(|v| (i, *v)))
            .collect();
        let ranked = |top: bool, work: &mut u64| -> Result<Vec<(usize, f64)>, PivotComputeError> {
            charge(work, sort_units(numbers.len()))?;
            let mut ranked = numbers.clone();
            // Stable, so ties keep the `<items>` order in either direction (`TP40i`, `TSb10i`).
            ranked.sort_by(|a, b| {
                let ordering = a.1.total_cmp(&b.1);
                if top {
                    ordering.reverse()
                } else {
                    ordering
                }
            });
            Ok(ranked)
        };
        match self {
            Self::Value { and, conditions } => {
                charge(work, units(numbers.len().saturating_mul(conditions.len())))?;
                for &(i, value) in &numbers {
                    let mut results = conditions.iter().map(|&(operator, criterion)| {
                        criterion
                            .and_then(|criterion| value.partial_cmp(&criterion))
                            .is_some_and(|ordering| operator.holds(ordering))
                    });
                    let holds = if *and {
                        results.all(|h| h)
                    } else {
                        results.any(|h| h)
                    };
                    if let Some(slot) = keep.get_mut(i) {
                        *slot = holds;
                    }
                }
            }
            &Self::Count { top, n } => {
                let ranked = ranked(top, work)?;
                // N is a whole count; one below 1 keeps nothing (UNVERIFIED, T-4799).
                let n = n.floor();
                #[expect(
                    clippy::cast_precision_loss,
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "a list length is exact in f64, and `n` is a whole number in 1..len"
                )]
                let cut = if n < 1.0 {
                    None
                } else if n >= ranked.len() as f64 {
                    ranked.last()
                } else {
                    // `n` is below the length here, so it indexes the list.
                    ranked.get(n as usize - 1)
                };
                if let Some(&(_, nth)) = cut {
                    for &(i, value) in &ranked {
                        let within = if top { value >= nth } else { value <= nth };
                        if let (true, Some(slot)) = (within, keep.get_mut(i)) {
                            *slot = true;
                        }
                    }
                }
            }
            &Self::Percent { top, percent } => {
                // The share of the parent's total, compared in absolute value: the reading that
                // fits a negative total (`NP50`, `NBP50`; UNVERIFIED, T-4799).
                let total: f64 = numbers.iter().map(|(_, v)| v).sum();
                let threshold = (percent * total / 100.0).abs();
                take_until(&ranked(top, work)?, &mut keep, |running| {
                    running.abs() >= threshold
                });
            }
            &Self::Sum { top, sum } => {
                // Signed: a bottom sum of 10 over negative items never reaches it (`NBS`).
                take_until(&ranked(top, work)?, &mut keep, |running| running >= sum);
            }
        }
        Ok(keep)
    }
}

/// Keeps `ranked` items in order until the running sum `reached` its threshold, the item that
/// reaches it included, and no tie after it (pivots.md "Top-N filters at their edges": `TS50`,
/// `TS60`, `TS110`, `NS3`, `TP40`, `TPb5`).
fn take_until(ranked: &[(usize, f64)], keep: &mut [bool], reached: impl Fn(f64) -> bool) {
    let mut running = 0.0;
    for &(i, value) in ranked {
        if let Some(slot) = keep.get_mut(i) {
            *slot = true;
        }
        running += value;
        if reached(running) {
            break;
        }
    }
}

/// A parent tuple's cell as (rank, item, cell, value).
type Ranked = (u32, u32, usize, Result<f64, Error>);

/// One value filter of one field: what it reads.
pub(super) struct ValueStep<'a> {
    pub(super) table: &'a PivotTable,
    pub(super) cache: &'a PivotCache,
    pub(super) field: u32,
    /// The fields outside it on its axis, outer first.
    pub(super) parents: &'a [u32],
    /// The field's display order: its `<items>` order breaks a Top-N tie.
    pub(super) order: &'a Order,
    pub(super) measure: u32,
    /// Whether an item's value is a total (`Accumulator::finish`).
    pub(super) total: bool,
}

impl ValueStep<'_> {
    /// Ranks the field's items within each parent tuple by their value of the data field
    /// `measure` over the kept records, and keeps the records of the items kept.
    pub(super) fn run(
        &self,
        ranking: &Ranking,
        kept: &mut Kept,
        work: &mut u64,
    ) -> Result<(), PivotComputeError> {
        // A data field that is not there applies nothing (UNVERIFIED, T-4799).
        let Some(data) = self.table.data_fields.get(self.measure as usize) else {
            return Ok(());
        };
        if kept.records.is_empty() {
            return Ok(());
        }
        let values = column(self.cache, data.field)?;
        let value_items = &self
            .cache
            .fields
            .get(data.field as usize)
            .ok_or(PivotComputeError::Malformed)?
            .items;
        let items = column(self.cache, self.field)?;
        let item_count = self
            .cache
            .fields
            .get(self.field as usize)
            .map_or(0, |f| f.items.len());
        let parents: Vec<&[u32]> = (self.parents.iter())
            .map(|&p| column(self.cache, p))
            .collect::<Result<_, _>>()?;
        // Each record is read once per parent field, the field and the data field, its key
        // hashed once, and its cell kept for the drop pass.
        let per_record = self.parents.len().saturating_add(4);
        charge(work, units(kept.records.len().saturating_mul(per_record)))?;
        // Per (parent tuple, item) its accumulator, and per kept record its cell.
        let mut cells: HashMap<Box<[u32]>, usize> = HashMap::new();
        let mut accumulators: Vec<(Box<[u32]>, u32, Accumulator)> = Vec::new();
        let mut cell_of: Vec<usize> = Vec::with_capacity(kept.records.len());
        let mut key = Vec::with_capacity(self.parents.len() + 1);
        for &record in &kept.records {
            key.clear();
            for column in &parents {
                let item = column.get(record as usize);
                key.push(*item.ok_or(PivotComputeError::Malformed)?);
            }
            let item = *items
                .get(record as usize)
                .ok_or(PivotComputeError::Malformed)?;
            key.push(item);
            let value = values
                .get(record as usize)
                .and_then(|&v| value_items.get(v as usize))
                .ok_or(PivotComputeError::Malformed)?;
            let index = if let Some(&index) = cells.get(key.as_slice()) {
                index
            } else {
                let index = accumulators.len();
                let parent: Box<[u32]> = key.get(..self.parents.len()).unwrap_or_default().into();
                accumulators.push((parent, item, Accumulator::new()));
                cells.insert(key.as_slice().into(), index);
                index
            };
            if let Some((_, _, accumulator)) = accumulators.get_mut(index) {
                accumulator.push(value);
            }
            cell_of.push(index);
        }
        drop(cells);
        let counts_mixed = (self.table.data_fields.iter())
            .any(|d| !matches!(d.function, Aggregate::Count | Aggregate::CountNums));
        let mut by_parent: HashMap<&[u32], Vec<Ranked>> = HashMap::new();
        charge(work, units(accumulators.len()).saturating_mul(2))?;
        for (cell, (parent, item, accumulator)) in accumulators.iter().enumerate() {
            let rank = self
                .order
                .rank
                .get(*item as usize)
                .copied()
                .unwrap_or(u32::MAX);
            let value = accumulator.finish(data.function, self.total, counts_mixed, None);
            by_parent
                .entry(parent)
                .or_default()
                .push((rank, *item, cell, value));
        }
        let mut kept_cell = vec![true; accumulators.len()];
        let mut gone = Vec::new();
        for entries in by_parent.values_mut() {
            charge(work, sort_units(entries.len()))?;
            entries.sort_by_key(|&(rank, item, _, _)| (rank, item));
            let values: Vec<Result<f64, Error>> = entries
                .iter()
                .map(|(_, _, _, value)| value.clone())
                .collect();
            let keep = ranking.keeps(&values, work)?;
            for (&(_, item, cell, _), keep) in entries.iter().zip(keep) {
                if !keep {
                    if let Some(slot) = kept_cell.get_mut(cell) {
                        *slot = false;
                    }
                    gone.push(item);
                }
            }
        }
        // An item removed from the one parent of an outermost field is removed from the field.
        if self.parents.is_empty() {
            for item in gone {
                mark_removed(kept, self.field, item, item_count);
            }
        }
        let mut cells = cell_of.into_iter();
        kept.records.retain(|_| {
            cells
                .next()
                .and_then(|cell| kept_cell.get(cell).copied())
                .unwrap_or(false)
        });
        Ok(())
    }
}
