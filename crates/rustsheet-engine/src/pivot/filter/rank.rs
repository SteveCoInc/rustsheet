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

/// One value filter (or value sort, `pivot::sort`) of one field: what it reads.
pub(in crate::pivot) struct ValueStep<'a> {
    pub(in crate::pivot) table: &'a PivotTable,
    pub(in crate::pivot) cache: &'a PivotCache,
    pub(in crate::pivot) field: u32,
    /// The fields outside it on its axis, outer first.
    pub(in crate::pivot) parents: &'a [u32],
    /// The field's display order: its `<items>` order breaks a Top-N tie.
    pub(in crate::pivot) order: &'a Order,
    pub(in crate::pivot) measure: u32,
    /// Whether an item's value is a total (`Accumulator::finish`).
    pub(in crate::pivot) total: bool,
    /// Per field of the other axis a value sort's scope names, its records and the item a
    /// record must hold for its value to count (`pivot::sort`); empty for a filter.
    pub(in crate::pivot) within: &'a [(&'a [u32], u32)],
}

/// A cell of [`ItemCells`]: its rank in the field's display order, its cache item, its index and
/// its value.
pub(in crate::pivot) type Ranked = (u32, u32, usize, Result<f64, Error>);

/// The cells of a field's items within each parent tuple, over some records.
pub(in crate::pivot) struct ItemCells {
    /// Per parent tuple, its cells, in the field's display order.
    pub(in crate::pivot) parents: Vec<Vec<Ranked>>,
    /// Per record, in the order given, the index of its cell.
    pub(in crate::pivot) cell_of: Vec<usize>,
    /// How many cells there are.
    pub(in crate::pivot) count: usize,
}

impl ValueStep<'_> {
    /// The value of the data field `measure` of each (parent tuple, item) cell over `records`:
    /// what a value filter ranks and a value sort orders by. A cell none of whose records holds
    /// the [`ValueStep::within`] items has the value 0, since a blank sorts as 0 (`SCd`, `SCa`,
    /// SC2 in docs/excel-semantics/pivots.md "Value sorts: ties, blanks and errors").
    pub(in crate::pivot) fn cells(
        &self,
        records: &[u32],
        work: &mut u64,
    ) -> Result<ItemCells, PivotComputeError> {
        let empty = ItemCells {
            parents: Vec::new(),
            cell_of: Vec::new(),
            count: 0,
        };
        // A data field that is not there applies nothing (UNVERIFIED, T-4799).
        let Some(data) = self.table.data_fields.get(self.measure as usize) else {
            return Ok(empty);
        };
        if records.is_empty() {
            return Ok(empty);
        }
        // A value filter, Top-N or value sort (`pivot::sort`) by a calculated data field leaves
        // the pivot without a result: what Excel shows is unprobed (UNVERIFIED, T-4897), and
        // ranking by the formula's values per item is not implemented (`pivot::calculated`).
        let calculated =
            (self.cache.fields.get(data.field as usize)).is_some_and(|f| f.formula.is_some());
        if calculated {
            return Err(PivotComputeError::Malformed);
        }
        let values = column(self.cache, data.field)?;
        let value_items = &self
            .cache
            .fields
            .get(data.field as usize)
            .ok_or(PivotComputeError::Malformed)?
            .items;
        let items = column(self.cache, self.field)?;
        let parents: Vec<&[u32]> = (self.parents.iter())
            .map(|&p| column(self.cache, p))
            .collect::<Result<_, _>>()?;
        // Each record is read once per parent field, scope item, the field and the data field,
        // its key hashed once, and its cell kept for the drop pass.
        let per_record = (self.parents.len())
            .saturating_add(self.within.len())
            .saturating_add(4);
        charge(work, units(records.len().saturating_mul(per_record)))?;
        // Per (parent tuple, item) its accumulator and whether a record in scope reached it,
        // and per record its cell.
        let mut cells: HashMap<Box<[u32]>, usize> = HashMap::new();
        let mut accumulators: Vec<(Box<[u32]>, u32, Accumulator, bool)> = Vec::new();
        let mut cell_of: Vec<usize> = Vec::with_capacity(records.len());
        let mut key = Vec::with_capacity(self.parents.len() + 1);
        for &record in records {
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
            let mut in_scope = true;
            for &(column, wanted) in self.within {
                let held = column.get(record as usize);
                in_scope &= *held.ok_or(PivotComputeError::Malformed)? == wanted;
            }
            let index = match cells.get(key.as_slice()) {
                Some(&index) => index,
                None => {
                    let index = accumulators.len();
                    let parent: Box<[u32]> =
                        key.get(..self.parents.len()).unwrap_or_default().into();
                    accumulators.push((parent, item, Accumulator::new(), false));
                    cells.insert(key.as_slice().into(), index);
                    index
                }
            };
            if let (true, Some((_, _, accumulator, reached))) =
                (in_scope, accumulators.get_mut(index))
            {
                accumulator.push(value);
                *reached = true;
            }
            cell_of.push(index);
        }
        drop(cells);
        let counts_mixed = (self.table.data_fields.iter())
            .any(|d| !matches!(d.function, Aggregate::Count | Aggregate::CountNums));
        let mut by_parent: HashMap<&[u32], Vec<Ranked>> = HashMap::new();
        charge(work, units(accumulators.len()).saturating_mul(2))?;
        for (cell, (parent, item, accumulator, reached)) in accumulators.iter().enumerate() {
            let rank = self
                .order
                .rank
                .get(*item as usize)
                .copied()
                .unwrap_or(u32::MAX);
            let value = if *reached {
                accumulator.finish(data.function, self.total, counts_mixed, None)
            } else {
                Ok(0.0)
            };
            by_parent
                .entry(parent)
                .or_default()
                .push((rank, *item, cell, value));
        }
        let mut parents: Vec<Vec<Ranked>> = by_parent.into_values().collect();
        for entries in &mut parents {
            charge(work, sort_units(entries.len()))?;
            entries.sort_by_key(|&(rank, item, _, _)| (rank, item));
        }
        Ok(ItemCells {
            parents,
            cell_of,
            count: accumulators.len(),
        })
    }

    /// Ranks the field's items within each parent tuple by their value of the data field
    /// `measure` over the kept records, and keeps the records of the items kept.
    pub(super) fn run(
        &self,
        ranking: &Ranking,
        kept: &mut Kept,
        work: &mut u64,
    ) -> Result<(), PivotComputeError> {
        let item_count = self
            .cache
            .fields
            .get(self.field as usize)
            .map_or(0, |f| f.items.len());
        let cells = self.cells(&kept.records, work)?;
        if cells.cell_of.is_empty() {
            return Ok(());
        }
        let mut kept_cell = vec![true; cells.count];
        let mut gone = Vec::new();
        for entries in &cells.parents {
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
        let mut cells = cells.cell_of.into_iter();
        kept.records.retain(|_| {
            cells
                .next()
                .and_then(|cell| kept_cell.get(cell).copied())
                .unwrap_or(false)
        });
        Ok(())
    }
}
