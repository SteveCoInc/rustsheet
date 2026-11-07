//! Value sorts (T-4827): a field whose `sortType` is ascending or descending with an
//! `autoSortScope` naming a data field shows its items ordered by that data field's value, within
//! each item of the outer fields of its axis (docs/excel-semantics/pivots.md "Pivot sorts and
//! filters as Excel for the web renders them", value-sort bullet, and "Value sorts: ties, blanks
//! and errors").
//!
//! The importer reads [`PivotField::auto_sort`] from the field's `autoSortScope`, which it keeps
//! verbatim in [`super::PivotTable::opaque`] for export, as it reads `fillDownLabels`.
//!
//! [`value_sorts`] runs after the filters (`pivot::filter::apply`), over the records they keep, and
//! gives each sorted field's items their position within their parent. An outermost field's
//! positions are one order for the whole field, which replaces its display order
//! (`result::axis::Order`); an inner field's are per record, and only an axis with one widens
//! its keys to put the position before the display rank (`result::axis::KeyPart`), so that the
//! axis tree (`result::axis::AxisColumns::tree`) orders each parent's children by it. An
//! outermost field sorts only the items the kept records hold; the others follow in cache order
//! and are never shown, unless `showAll` shows them, when they sort as 0. The values are the
//! ones a value filter ranks by (`filter::rank::ValueStep::cells`), so a sort and a filter read
//! one definition of an item's value. The order (`Placing::sort`):
//! - by the value, largest first for `descending`; **ties keep the cache order the file wrote**
//!   (probes/t4164-pivot-value-sort `STd`, `STo`, `STc`), not the `<items>` order or the source's;
//! - `ascending` is the exact reverse, so its ties run against the cache order (`STa`, `STca`);
//! - an item with no value in the scope (no record reaches its cell, or its records hold no
//!   number) **sorts as 0** (`SEd`, `SEa`, `SCd`, `SCa`, SC2);
//! - an item whose value is an error **sorts last in both directions** (`SEd`, `SEa`);
//! - the scope's reference to a column item sorts by that column's cell (probes/t4130-pivot-sort
//!   SVC, t4164 SC2), and the data reference's `x` picks the data field (t4130 SV2).
//!
//! UNVERIFIED (probe pending: T-4799): a blank against a real 0 (tied here, by the cache order),
//! the order of several errors (the cache order, both ways), and ties of an inner field (the cache
//! order, as for an outer one). UNVERIFIED (unprobed, T-4799): a `reference@x` read as an index
//! into the field's `<items>` (every probe lists them in cache order, so an index into the cache
//! would agree), and only its first `x` read; a reference to a field on no axis, on the sorted
//! field's own axis or past the cache, which is ignored; a scope whose data field or item is past
//! the definition's, or that names no data field, which sorts nothing, as does a scope under
//! `sortType="manual"`; an item that `showAll` shows with no record, which sorts as 0; a value
//! sort on a column field, read as on a row field with the row items as its scope; and a Count or
//! Average item with no number in the scope, which sorts as its cell shows it (0, or `#DIV/0!`
//! last). Label sorts (`sortType` with no `autoSortScope`) are [`labels`]'s (T-4768): they give
//! the display orders a value sort's parents group by, and a field with a scope is never
//! label-sorted, whether or not its scope sorts anything.
//!
//! Every pass charges its work before doing it (CLAUDE.md directive 5).

use bitcode::{Decode, Encode};

use crate::expressions::token::Error;

use super::filter::{Kept, ValueStep};
use super::result::{axis_fields, charge, sort_units, units, Order, PivotComputeError};
use super::{ItemType, PivotCache, PivotField, PivotTable, SortType};

mod labels;

pub(super) use labels::display_order;

/// `reference@field` for the data fields (`-2` as an unsigned int): its `x` names a data field.
pub const DATA_FIELD_REFERENCE: u32 = 4_294_967_294;

/// A field's `autoSortScope` (ECMA-376 `CT_AutoSortScope`): the `pivotArea` whose value a value
/// sort orders the field's items by. Its other attributes (`axis`, `field`, `type`, the
/// reference's subtotal flags) change nothing visible (probes/t4130-pivot-sort SXA, SXR) and are
/// not read.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Default)]
pub struct AutoSortScope {
    /// The data field: the first `x` of the reference whose `field` is [`DATA_FIELD_REFERENCE`];
    /// `None` when the area has no such reference.
    pub data_field: Option<u32>,
    /// The area's other references, in file order.
    pub references: Vec<SortReference>,
}

/// A `reference` of an `autoSortScope` to a field's items.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct SortReference {
    /// `field`: the cache field.
    pub field: u32,
    /// Its `x`s, in file order: indices into the field's `<items>` (UNVERIFIED, T-4799: see the
    /// module documentation). The sort reads the first.
    pub items: Vec<u32>,
}

/// Where a sorted field puts its items: a position within their parent, smaller first.
pub(super) enum Placing {
    /// An outermost field (one parent): per cache item its position, a permutation of the items,
    /// which becomes the field's display order (`result::axis::Order`).
    ByItem(Vec<u32>),
    /// An inner field: per kept record (in the order of the records the sort was given) the
    /// position of its item within its parent.
    ByRecord(Vec<u32>),
}

impl Placing {
    /// Sorts `entries` (`(cache item, value, slot)`, a parent's cells) by the value (see the
    /// module documentation), and writes each one's position into `positions` at its slot.
    /// Charged by the caller.
    fn sort(entries: &mut [Entry], descending: bool, positions: &mut [u32]) {
        // The class (a number before an error), then the value, then the cache item. `+ 0.0`
        // makes -0 a 0, so `total_cmp` ties them; the values are finite or errors.
        let key = |&(item, ref value, _): &Entry| match value {
            Ok(v) => (0u8, *v + 0.0, item),
            Err(_) => (1u8, 0.0, item),
        };
        entries.sort_by(|a, b| {
            let ((ca, va, ia), (cb, vb, ib)) = (key(a), key(b));
            ca.cmp(&cb).then_with(|| {
                if ca == 1 {
                    // Errors last, in cache order both ways (UNVERIFIED, T-4799).
                    ia.cmp(&ib)
                } else if descending {
                    vb.total_cmp(&va).then(ia.cmp(&ib))
                } else {
                    va.total_cmp(&vb).then(ib.cmp(&ia))
                }
            })
        });
        for (position, &(_, _, slot)) in (0u32..).zip(entries.iter()) {
            if let Some(at) = positions.get_mut(slot) {
                *at = position;
            }
        }
    }
}

/// A cell a sort orders: its cache item, its value and the slot its position goes to.
type Entry = (u32, Result<f64, Error>, usize);

/// Per axis (rows, then columns), per axis field (the Values field left out), its value sort.
pub(super) type Sorts = [Vec<Option<Placing>>; 2];

/// The value sort a field shows, if any: its direction and scope.
fn sorted(field: Option<&PivotField>) -> Option<(bool, &AutoSortScope)> {
    let field = field?;
    let scope = field.auto_sort.as_ref()?;
    match field.sort_type {
        SortType::Ascending => Some((false, scope)),
        SortType::Descending => Some((true, scope)),
        // A scope under `manual` sorts nothing (UNVERIFIED, T-4799).
        SortType::Manual => None,
    }
}

/// The cache item `reference` names: its first `x` as an index into the field's `<items>`.
fn referenced_item(table: &PivotTable, reference: &SortReference) -> Option<u32> {
    let x = *reference.items.first()?;
    let item = table
        .fields
        .get(reference.field as usize)?
        .items
        .get(x as usize)?;
    (item.kind == ItemType::Data)
        .then_some(item.index)
        .flatten()
}

/// The value sorts of `table`'s row and column fields over `kept` (see the module
/// documentation); `orders` are the display orders of each axis's fields, outer first.
pub(super) fn value_sorts(
    table: &PivotTable,
    cache: &PivotCache,
    kept: &Kept,
    orders: [&[&Order]; 2],
    work: &mut u64,
) -> Result<Sorts, PivotComputeError> {
    let axes = [
        axis_fields(&table.row_fields),
        axis_fields(&table.column_fields),
    ];
    charge(work, units(axes[0].len().saturating_add(axes[1].len())))?;
    let mut sorts: Sorts = [Vec::new(), Vec::new()];
    for (index, fields) in axes.iter().enumerate() {
        let other = axes.get(1 - index).map_or(&[][..], Vec::as_slice);
        let axis_orders = orders.get(index).copied().unwrap_or_default();
        for (depth, &field) in fields.iter().enumerate() {
            let sorted = sorted(table.fields.get(field as usize));
            let placing = match (sorted, axis_orders.get(depth)) {
                (Some((descending, scope)), Some(order)) => {
                    let parents = fields.get(..depth).unwrap_or_default();
                    let sort = FieldSort {
                        table,
                        cache,
                        field,
                        order,
                        parents,
                        other,
                        inner: depth + 1 < fields.len(),
                        show_all: fields.len() == 1,
                    };
                    sort.placing(scope, descending, kept, work)?
                }
                _ => None,
            };
            if let Some(slot) = sorts.get_mut(index) {
                slot.push(placing);
            }
        }
    }
    Ok(sorts)
}

/// One sorted field and where it sits.
struct FieldSort<'a> {
    table: &'a PivotTable,
    cache: &'a PivotCache,
    field: u32,
    /// Its display order (`result::axis::AxisColumns`), which groups a parent's cells.
    order: &'a Order,
    /// The fields outside it on its axis, outer first.
    parents: &'a [u32],
    /// The fields of the other axis: the scope's column items are among them.
    other: &'a [u32],
    /// Whether the field has fields inside it on its axis: its items' values are subtotals.
    inner: bool,
    /// Whether it is its axis's one field, which `showAll` can show items of with no record.
    show_all: bool,
}

impl FieldSort<'_> {
    /// The field's placing by `scope`; `None` for a scope that sorts nothing.
    fn placing(
        &self,
        scope: &AutoSortScope,
        descending: bool,
        kept: &Kept,
        work: &mut u64,
    ) -> Result<Option<Placing>, PivotComputeError> {
        let Some(measure) = scope.data_field else {
            return Ok(None);
        };
        if measure as usize >= self.table.data_fields.len() {
            return Ok(None);
        }
        // Each reference is looked for among the other axis's fields.
        let scan = units(scope.references.len()).saturating_mul(units(self.other.len().max(1)));
        charge(work, scan.saturating_mul(2))?;
        // The references to items of the other axis's fields: a record's value counts only when
        // it holds each of them.
        let mut within = Vec::new();
        let mut pinned = Vec::new();
        for reference in &scope.references {
            if !self.other.contains(&reference.field) {
                continue;
            }
            pinned.push(reference.field);
            let Some(item) = referenced_item(self.table, reference) else {
                return Ok(None);
            };
            // The other axis's fields are in the cache (`AxisColumns::new` refuses the pivot
            // otherwise), so this guard does not fire; a reference past the cache was skipped
            // above, as a field on no axis is.
            let Some(records) = self.cache.fields.get(reference.field as usize) else {
                return Ok(None);
            };
            within.push((records.records.as_slice(), item));
        }
        let item_count = self
            .cache
            .fields
            .get(self.field as usize)
            .ok_or(PivotComputeError::Malformed)?
            .items
            .len();
        let pivot_field = self.table.fields.get(self.field as usize);
        // A field of the other axis the scope leaves open sums into the item's value.
        let open = self.other.iter().any(|field| !pinned.contains(field));
        let step = ValueStep {
            table: self.table,
            cache: self.cache,
            field: self.field,
            parents: self.parents,
            order: self.order,
            measure,
            total: self.inner || open,
            within: &within,
        };
        let cells = step.cells(&kept.records, work)?;
        if self.parents.is_empty() {
            // One parent: the items its cells hold are sorted; the rest, which no kept record
            // holds, follow in cache order and are never shown, unless `showAll` shows them:
            // then they sort as 0 among the others.
            let show_all = self.show_all && pivot_field.is_some_and(|f| f.show_all);
            charge(work, units(item_count).saturating_mul(2))?;
            let mut held = vec![false; item_count];
            let mut entries: Vec<Entry> = Vec::new();
            for parent in &cells.parents {
                for (_, item, _, value) in parent {
                    if let Some(slot) = held.get_mut(*item as usize) {
                        *slot = true;
                        entries.push((*item, value.clone(), *item as usize));
                    }
                }
            }
            let rest = (0u32..)
                .zip(&held)
                .filter(|(_, &h)| !h)
                .map(|(item, _)| item);
            let mut tail = Vec::new();
            for item in rest {
                if show_all {
                    entries.push((item, Ok(0.0), item as usize));
                } else {
                    tail.push(item);
                }
            }
            charge(work, sort_units(entries.len()))?;
            let mut positions = vec![0; item_count];
            Placing::sort(&mut entries, descending, &mut positions);
            for (position, item) in (u32::try_from(entries.len()).unwrap_or(u32::MAX)..).zip(tail) {
                if let Some(slot) = positions.get_mut(item as usize) {
                    *slot = position;
                }
            }
            return Ok(Some(Placing::ByItem(positions)));
        }
        // Per cell its position within its parent, then per record its cell's.
        let mut of_cell = vec![0; cells.count];
        for parent in &cells.parents {
            let mut entries: Vec<Entry> = (parent.iter())
                .map(|(_, item, cell, value)| (*item, value.clone(), *cell))
                .collect();
            charge(
                work,
                sort_units(entries.len()).saturating_add(units(entries.len())),
            )?;
            Placing::sort(&mut entries, descending, &mut of_cell);
        }
        charge(work, units(cells.cell_of.len()))?;
        let positions = (cells.cell_of.iter())
            .map(|&cell| of_cell.get(cell).copied().unwrap_or(0))
            .collect();
        Ok(Some(Placing::ByRecord(positions)))
    }
}

#[cfg(test)]
mod tests;
