//! The axis trees of a pivot result (T-4142; value sorts T-4827, label sorts T-4768): each axis's
//! fields, their display orders (`Order`, a label sort or a value sort of an outermost field
//! folded in) and the keys whose
//! sorted distinct values make the tree ([`KeyPart`]: narrow unless an inner field is
//! value-sorted).

use std::collections::HashMap;

use super::super::filter::Kept;
use super::super::sort::{display_order, Placing};
use super::super::{AxisField, ItemType, PivotCache, PivotField, PivotTable};
use super::PivotComputeError;
use super::{charge, count_nodes, node_index, sort_units, units, AxisNode, AxisTree};

/// A field's display order: per cache item its rank, and per rank its cache item. The field's
/// listed data items come first, in `<items>` order (a repeat or an index past the cache items
/// is skipped), then the cache items it does not list, in cache order.
pub(in crate::pivot) struct Order {
    pub(in crate::pivot) rank: Vec<u32>,
    item: Vec<u32>,
}

impl Order {
    pub(in crate::pivot) fn of(field: Option<&PivotField>, item_count: usize) -> Order {
        let mut rank = vec![u32::MAX; item_count];
        let mut item = Vec::with_capacity(item_count);
        let listed = field
            .into_iter()
            .flat_map(|f| &f.items)
            .filter(|i| i.kind == ItemType::Data)
            .filter_map(|i| i.index);
        // The cache holds at most MAX_ITEMS items, so every index and rank fits a u32.
        for index in listed.chain(0..u32::try_from(item_count).unwrap_or(u32::MAX)) {
            if let Some(slot) = rank.get_mut(index as usize) {
                if *slot == u32::MAX {
                    *slot = u32::try_from(item.len()).unwrap_or(u32::MAX);
                    item.push(index);
                }
            }
        }
        Order { rank, item }
    }

    /// The order a value sort of an outermost field or a label sort gives (`pivot::sort`): per
    /// cache item its position, which must be a permutation of the items.
    pub(in crate::pivot) fn permuted(positions: Vec<u32>) -> Result<Order, PivotComputeError> {
        let mut item = vec![u32::MAX; positions.len()];
        for (index, &position) in (0u32..).zip(&positions) {
            match item.get_mut(position as usize) {
                Some(slot) if *slot == u32::MAX => *slot = index,
                _ => return Err(PivotComputeError::Malformed),
            }
        }
        Ok(Order {
            rank: positions,
            item,
        })
    }
}

/// The cache fields of an axis, outer first, the Values field left out.
pub(in crate::pivot) fn axis_fields(axis: &[AxisField]) -> Vec<u32> {
    axis.iter()
        .filter_map(|f| match f {
            AxisField::Field(index) => Some(*index),
            AxisField::Values => None,
        })
        .collect()
}

/// One field of an axis: its records, its display order (a value sort of an outermost field
/// folded in) and, for a value sort of an inner field, per kept record the position of its item
/// within its parent.
struct AxisColumn<'a> {
    records: &'a [u32],
    order: Order,
    placed: Option<Vec<u32>>,
}

/// One axis's fields with their records and display orders.
pub(super) struct AxisColumns<'a> {
    pub(super) fields: Vec<u32>,
    columns: Vec<AxisColumn<'a>>,
    /// The axis's one field when it has `showAll` on: its items with no record are shown too.
    show_all: Option<&'a PivotField>,
}

/// An item's part of an axis key. Without an inner value sort it is the item's display rank
/// (`u32`); with one it is its position within its parent, then its rank (`u64`), so a parent's
/// children sort by position and then rank. Only an axis with an inner value sort pays for the
/// wider key.
trait KeyPart: Copy + Ord + std::hash::Hash {
    /// The part of an item at `position` (0 on a field with no inner value sort, the only one a
    /// `u32` key is built for) and `rank`.
    fn part(position: u32, rank: u32) -> Self;
    /// The display rank of the part.
    fn rank(self) -> u32;
}

impl KeyPart for u32 {
    fn part(_: u32, rank: u32) -> Self {
        rank
    }

    fn rank(self) -> u32 {
        self
    }
}

impl KeyPart for u64 {
    fn part(position: u32, rank: u32) -> Self {
        (u64::from(position) << 32) | u64::from(rank)
    }

    fn rank(self) -> u32 {
        // The low half is the rank.
        (self & u64::from(u32::MAX)) as u32
    }
}

impl<'a> AxisColumns<'a> {
    pub(super) fn new(
        table: &'a PivotTable,
        cache: &'a PivotCache,
        axis: &[AxisField],
        work: &mut u64,
    ) -> Result<Self, PivotComputeError> {
        let fields = axis_fields(axis);
        let mut columns = Vec::with_capacity(fields.len());
        for &field in &fields {
            let cache_field = cache
                .fields
                .get(field as usize)
                .ok_or(PivotComputeError::Malformed)?;
            columns.push(AxisColumn {
                records: cache_field.records.as_slice(),
                order: display_order(table, cache_field, field, work)?,
                placed: None,
            });
        }
        let show_all = match fields.as_slice() {
            [field] => table.fields.get(*field as usize).filter(|f| f.show_all),
            _ => None,
        };
        Ok(AxisColumns {
            fields,
            columns,
            show_all,
        })
    }

    /// The display orders of the axis's fields, outer first.
    pub(super) fn orders(&self) -> Vec<&Order> {
        self.columns.iter().map(|c| &c.order).collect()
    }

    /// Applies the axis's value sorts, one per field ([`value_sorts`]): an outermost field's
    /// becomes its display order, an inner field's its per-record positions.
    pub(super) fn sort(
        &mut self,
        sorts: Vec<Option<Placing>>,
        work: &mut u64,
    ) -> Result<(), PivotComputeError> {
        // One sort per field (`value_sorts` reads the same axis fields); anything else is a
        // mismatch, refused rather than applied to the wrong fields.
        if sorts.len() != self.columns.len() {
            return Err(PivotComputeError::Malformed);
        }
        for (column, sort) in self.columns.iter_mut().zip(sorts) {
            match sort {
                Some(Placing::ByItem(positions)) => {
                    charge(work, units(positions.len()))?;
                    column.order = Order::permuted(positions)?;
                }
                Some(Placing::ByRecord(positions)) => column.placed = Some(positions),
                None => {}
            }
        }
        Ok(())
    }

    /// Whether the axis has a value sort of an inner field, so needs the wide keys.
    fn wide(&self) -> bool {
        self.columns.iter().any(|c| c.placed.is_some())
    }

    /// Under the axis field's `showAll`, interns every item it does not hide and no filter
    /// removed, in display order, so that an item no kept record holds is shown with blank
    /// values (see the module documentation). Such an axis has one field, so no inner sort.
    fn show_all_keys<K: KeyPart>(
        &self,
        keys: &mut HashMap<Box<[K]>, u32>,
        kept: &Kept,
        work: &mut u64,
    ) -> Result<(), PivotComputeError> {
        let (Some(field), Some(column), Some(&index)) =
            (self.show_all, self.columns.first(), self.fields.first())
        else {
            return Ok(());
        };
        charge(work, units(column.order.item.len()))?;
        for (rank, &item) in (0u32..).zip(&column.order.item) {
            // Nor an item a filter removed (UNVERIFIED, T-4799: no probe has `showAll` beside a
            // filter).
            if !field.is_hidden(item) && !kept.is_removed(index, item) {
                intern(keys, &[K::part(0, rank)])?;
            }
        }
        Ok(())
    }

    /// Writes the key parts of `record`'s items into `key`; `index` is its index among the kept
    /// records, which an inner value sort places by.
    fn key<K: KeyPart>(
        &self,
        index: usize,
        record: u32,
        key: &mut Vec<K>,
    ) -> Result<(), PivotComputeError> {
        key.clear();
        for column in &self.columns {
            let item = column
                .records
                .get(record as usize)
                .ok_or(PivotComputeError::Malformed)?;
            let rank = column
                .order
                .rank
                .get(*item as usize)
                .ok_or(PivotComputeError::Malformed)?;
            let position = match &column.placed {
                Some(positions) => *positions.get(index).ok_or(PivotComputeError::Malformed)?,
                None => 0,
            };
            key.push(K::part(position, *rank));
        }
        Ok(())
    }

    /// The tree of the distinct `keys` ([`KeyPart`]s), and per key id its leaf node.
    fn tree<K: KeyPart>(
        self,
        keys: HashMap<Box<[K]>, u32>,
        work: &mut u64,
        nodes: &mut usize,
    ) -> Result<(AxisTree, Vec<u32>), PivotComputeError> {
        let mut sorted: Vec<(Box<[K]>, u32)> = keys.into_iter().collect();
        charge(work, sort_units(sorted.len()))?;
        sorted.sort_unstable();
        let mut tree = vec![AxisNode {
            parent: None,
            item: None,
            depth: 0,
            end: 0,
        }];
        let mut leaves = vec![0; sorted.len()];
        // The open nodes, root first.
        let mut path: Vec<u32> = vec![0];
        let mut previous: Option<&[K]> = None;
        for (key, id) in &sorted {
            let common = previous.map_or(0, |p| {
                p.iter().zip(key.iter()).take_while(|(a, b)| a == b).count()
            });
            while path.len() > common + 1 {
                close(&mut tree, path.pop())?;
            }
            let depth_new = key.len().saturating_sub(common);
            count_nodes(nodes, depth_new)?;
            charge(work, units(depth_new))?;
            for (depth, &part) in key.iter().enumerate().skip(common) {
                let item = self
                    .columns
                    .get(depth)
                    .and_then(|c| c.order.item.get(part.rank() as usize))
                    .copied()
                    .ok_or(PivotComputeError::Malformed)?;
                let index = node_index(tree.len())?;
                tree.push(AxisNode {
                    parent: path.last().copied(),
                    item: Some(item),
                    depth: u32::try_from(depth + 1).map_err(|_| PivotComputeError::Limit)?,
                    end: 0,
                });
                path.push(index);
            }
            if let Some(leaf) = leaves.get_mut(*id as usize) {
                *leaf = path.last().copied().unwrap_or(0);
            }
            previous = Some(key);
        }
        while let Some(open) = path.pop() {
            close(&mut tree, Some(open))?;
        }
        Ok((
            AxisTree {
                fields: self.fields,
                nodes: tree,
            },
            leaves,
        ))
    }
}

/// The row and column trees of `records` (the kept records), each record placed at its (row
/// leaf, column leaf), in one pass over the records, each axis's keys as narrow as its sorts
/// allow ([`KeyPart`]).
pub(super) fn place(
    rows: AxisColumns<'_>,
    columns: AxisColumns<'_>,
    records: &[u32],
    kept: &Kept,
    work: &mut u64,
) -> Result<Placed, PivotComputeError> {
    match (rows.wide(), columns.wide()) {
        (false, false) => place_with::<u32, u32>(rows, columns, records, kept, work),
        (true, false) => place_with::<u64, u32>(rows, columns, records, kept, work),
        (false, true) => place_with::<u32, u64>(rows, columns, records, kept, work),
        (true, true) => place_with::<u64, u64>(rows, columns, records, kept, work),
    }
}

/// An axis's distinct keys and their ids.
type Keys<K> = HashMap<Box<[K]>, u32>;

/// The row tree, the column tree, every record at its (row leaf, column leaf), and the nodes
/// made.
pub(super) type Placed = (AxisTree, AxisTree, Vec<(u32, u32, u32)>, usize);

fn place_with<R: KeyPart, C: KeyPart>(
    rows: AxisColumns<'_>,
    columns: AxisColumns<'_>,
    records: &[u32],
    kept: &Kept,
    work: &mut u64,
) -> Result<Placed, PivotComputeError> {
    let mut row_keys: Keys<R> = HashMap::new();
    let mut column_keys: Keys<C> = HashMap::new();
    let (mut row_key, mut column_key) = (Vec::new(), Vec::new());
    let mut placed = Vec::with_capacity(records.len());
    for (index, &record) in records.iter().enumerate() {
        rows.key(index, record, &mut row_key)?;
        let row = intern(&mut row_keys, &row_key)?;
        columns.key(index, record, &mut column_key)?;
        let column = intern(&mut column_keys, &column_key)?;
        placed.push((row, column, record));
    }
    rows.show_all_keys(&mut row_keys, kept, work)?;
    columns.show_all_keys(&mut column_keys, kept, work)?;
    let mut nodes = 1;
    let (rows, row_leaves) = rows.tree(row_keys, work, &mut nodes)?;
    let (columns, column_leaves) = columns.tree(column_keys, work, &mut nodes)?;
    for (row, column, _) in &mut placed {
        *row = row_leaves.get(*row as usize).copied().unwrap_or(0);
        *column = column_leaves.get(*column as usize).copied().unwrap_or(0);
    }
    Ok((rows, columns, placed, nodes))
}

/// Ends the subtree of the open node `node` at the tree's current length.
fn close(tree: &mut [AxisNode], node: Option<u32>) -> Result<(), PivotComputeError> {
    let end = node_index(tree.len())?;
    if let Some(n) = node.and_then(|n| tree.get_mut(n as usize)) {
        n.end = end;
    }
    Ok(())
}

/// The id of `key` in `keys`, adding it when new.
fn intern<K: KeyPart>(
    keys: &mut HashMap<Box<[K]>, u32>,
    key: &[K],
) -> Result<u32, PivotComputeError> {
    if let Some(&id) = keys.get(key) {
        return Ok(id);
    }
    let id = node_index(keys.len())?;
    keys.insert(key.into(), id);
    Ok(id)
}
