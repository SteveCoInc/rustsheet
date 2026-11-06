//! The pivot result tree: a pivot's filtered records and the accumulator of every (row node,
//! column node, data field), its subtotals and grand totals included (SPEC-0032 "`PivotResult`",
//! stages 1 to 3; T-4142).
//!
//! [`compute`] is a pure function of the definition and the cache:
//! 1. **Record filter.** A page field's selected item keeps only its records, and a hidden item
//!    (`item@h`, [`PivotField::is_hidden`]) of a field on an axis drops its records. Only a page
//!    field's selection and hidden items are evidenced (probes/t4137-pivot-captions K4); hidden
//!    items of a row or column field, and the totals over them, are UNVERIFIED (T-4164). A page
//!    `item` that names no data item reads as `(All)`, as T-4139's import does, and a hidden item
//!    on a field off every axis is ignored (both UNVERIFIED, T-4633; docs/excel-semantics/
//!    pivots.md "Record filters").
//! 2. **Groups** are already the records' items: a grouped field's records index its group items
//!    (`number_groups`, `date_groups`).
//! 3. **Accumulate.** Each axis is a tree of item tuples in display order: a field's listed
//!    items in `<items>` order, then the cache items it does not list, in cache order (pivots.md
//!    "Items show in the order the pivot field lists them"). Records are fed to every
//!    accumulator that covers them in one order: by row leaf, then column leaf, in display
//!    order, and in record order within a leaf. So an item cell accumulates in record order, and
//!    a total accumulates item by item in the display order of the field it totals, in record
//!    order within an item: Excel's order for Average, Var, `StdDev` and Product, and for Sum
//!    beside another aggregate (pivots.md "Summation order of Sum, Average and Product totals":
//!    probes/t4616-pivot-total-order-3 S2, S3, V1 to V4, P1, P2; t4614 S2, P2; t4589 S1).
//!    The rest is read from the accumulators ([`PivotResult::value_as`]):
//!    - in a pivot whose data fields are all Sum, a total is the sum of its child cells in
//!      display order, and the 2-D grand-grand total the sum of the row totals (t4616 S1, S4 to
//!      S6; t4135 `A_sum!S6`, S8). Which mixes switch Sum to the records, whether a Sum-only
//!      total over one axis field sums cells or records, and a cell that is a total on both
//!      axes other than the grand-grand total (a subtotal by a subtotal or a grand total, with
//!      several fields on an axis), where row children are summed first, are UNVERIFIED
//!      (T-4634);
//!    - the 2-D grand-grand total of a Sum or Average beside other aggregates is the sum of the
//!      row totals' record sums (t4616 S2 H9 and I9 fit it, and so does S3 E8; UNVERIFIED,
//!      T-4634); the Var and `StdDev` one is row-major, which fits t4616 V1 E11 and V2 E11 among
//!      other readings (UNVERIFIED, T-4634).
//!
//! Stage 7, the layout lines and location, is `pivot::layout::lay_out`, a separate call on the
//! result, so that a pivot too large to lay out still has its values (T-4147).
//!
//! Every stage charges its work before doing it, and the nodes and accumulators are counted
//! against [`MAX_PIVOT_NODES`] as they are made, so an untrusted pivot is bounded (CLAUDE.md
//! directive 5; pivots.md "A pivot result is bounded in work and nodes", DEVIATION).

use std::collections::HashMap;

use thiserror::Error as ThisError;

use crate::expressions::token::Error;

use super::accumulate::{sum_in_order, Accumulator};
use super::{Aggregate, Axis, AxisField, CacheItem, ItemType, PivotCache, PivotField, PivotTable};

/// Most nodes one pivot result holds: axis nodes, plus one accumulator per (row node, column
/// node, data field) that some record reaches (SPEC-0032 "Untrusted-input limits").
pub const MAX_PIVOT_NODES: usize = 10_000_000;

/// The work units a standalone [`compute`] may spend: one per record and stage, per node or
/// accumulator made, per value fed to an accumulator, per item ordered and per sort step.
/// It is the size of SPEC-0024's per-formula work budget.
pub const MAX_PIVOT_WORK: u64 = 1 << 28;

/// Why a pivot has no result.
#[derive(ThisError, Debug, PartialEq, Eq, Clone, Copy)]
pub enum PivotComputeError {
    /// The pivot needs more work than it was given or more than [`MAX_PIVOT_NODES`] nodes.
    #[error("a pivot needs more work or result nodes than the limits allow")]
    Limit,
    /// A field index of the definition, or a record's item index, is not in the cache.
    #[error("a pivot does not match its cache")]
    Malformed,
}

/// One node of an axis tree: the grand total (the root), a subtotal, or a leaf tuple.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxisNode {
    /// The parent node; `None` for the root.
    pub parent: Option<u32>,
    /// The cache item of the node's field (the axis field at `depth - 1`); `None` for the root.
    pub item: Option<u32>,
    /// 0 for the root, the number of axis fields for a leaf.
    pub depth: u32,
    /// One past the node's last descendant: its subtree is the nodes `[index, end)`.
    pub end: u32,
}

/// The item tuples one axis shows, as a tree in display order (preorder: a node, then its
/// children in display order). Node 0 is the root, the axis's grand total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AxisTree {
    fields: Vec<u32>,
    nodes: Vec<AxisNode>,
}

impl AxisTree {
    /// The cache fields of the axis, outer first (the Values field left out).
    #[must_use]
    pub fn fields(&self) -> &[u32] {
        &self.fields
    }

    /// The nodes, in display order.
    #[must_use]
    pub fn nodes(&self) -> &[AxisNode] {
        &self.nodes
    }

    /// Whether `node` is a leaf: a tuple of one item per axis field (the root of an axis with no
    /// field is one).
    #[must_use]
    pub fn is_leaf(&self, node: u32) -> bool {
        self.nodes
            .get(node as usize)
            .is_some_and(|n| n.depth as usize == self.fields.len())
    }

    /// The child of `node` for cache item `item`. Linear in the node's children: a caller that
    /// looks up every cell this way is quadratic (T-4149 and T-4154 walk the tree instead).
    #[must_use]
    pub fn child(&self, node: u32, item: u32) -> Option<u32> {
        self.children(node).find(|&child| {
            self.nodes
                .get(child as usize)
                .is_some_and(|c| c.item == Some(item))
        })
    }

    /// The children of `node`, in display order.
    pub fn children(&self, node: u32) -> impl Iterator<Item = u32> + '_ {
        let end = self.nodes.get(node as usize).map_or(0, |n| n.end);
        std::iter::successors(Some(node + 1), move |&child| {
            self.nodes.get(child as usize).map(|c| c.end)
        })
        .take_while(move |&child| child < end)
    }

    /// The node of the tuple `items` (cache items, outer first; empty for the root); linear in
    /// the children it passes, like [`AxisTree::child`].
    #[must_use]
    pub fn find(&self, items: &[u32]) -> Option<u32> {
        items
            .iter()
            .try_fold(0, |node, &item| self.child(node, item))
    }

    /// `node` and its ancestors, the root last.
    fn ancestors(&self, node: u32) -> impl Iterator<Item = u32> + '_ {
        std::iter::successors(Some(node), |&n| {
            self.nodes.get(n as usize).and_then(|n| n.parent)
        })
    }
}

/// A pivot's computed values (SPEC-0032 stages 1 to 3): derived state, never encoded.
#[derive(Debug, Clone, PartialEq)]
pub struct PivotResult {
    records: Vec<u32>,
    rows: AxisTree,
    columns: AxisTree,
    functions: Vec<Aggregate>,
    /// Every data field is a Sum: Sum totals are then the sums of their child cells.
    sum_only: bool,
    /// A data field is neither Count nor `CountNums`: a Count total covering an error is then the
    /// error.
    counts_mixed: bool,
    /// Per (row node, column node) that some record reaches, the index of its first data
    /// field's accumulator.
    cells: HashMap<(u32, u32), u32>,
    accumulators: Vec<Accumulator>,
}

impl PivotResult {
    /// The records the filters keep, in record order.
    #[must_use]
    pub fn records(&self) -> &[u32] {
        &self.records
    }

    /// The row axis.
    #[must_use]
    pub fn rows(&self) -> &AxisTree {
        &self.rows
    }

    /// The column axis.
    #[must_use]
    pub fn columns(&self) -> &AxisTree {
        &self.columns
    }

    /// The value of data field `data_field` at (`row`, `column`) under its own function: `None`
    /// for an intersection no record reaches, which renders blank for every aggregate (pivots.md
    /// "Aggregates over text, blanks, booleans and errors"), and the aggregate's error otherwise
    /// as `Err`. `errorCaption` and `missingCaption` are the renderer's (T-4149).
    #[must_use]
    pub fn value(&self, row: u32, column: u32, data_field: usize) -> Option<Result<f64, Error>> {
        let function = *self.functions.get(data_field)?;
        self.value_as(row, column, data_field, function)
    }

    /// [`PivotResult::value`] under `function` instead, for a custom subtotal line
    /// (`PivotField::custom_subtotals`).
    #[must_use]
    pub fn value_as(
        &self,
        row: u32,
        column: u32,
        data_field: usize,
        function: Aggregate,
    ) -> Option<Result<f64, Error>> {
        if data_field >= self.functions.len() {
            return None;
        }
        let base = *self.cells.get(&(row, column))? as usize;
        let accumulator = self.accumulators.get(base + data_field)?;
        let total = !self.rows.is_leaf(row) || !self.columns.is_leaf(column);
        let grand_grand = row == 0
            && column == 0
            && !self.rows.fields.is_empty()
            && !self.columns.fields.is_empty();
        let sum = match function {
            Aggregate::Sum if total && self.sum_only => Some(accumulator.children_sum()),
            Aggregate::Sum | Aggregate::Average if grand_grand => {
                Some(self.row_totals_sum(data_field))
            }
            _ => None,
        };
        Some(accumulator.finish(function, total, self.counts_mixed, sum))
    }

    /// The record sums of the row grand totals (the Grand Total column), down the rows in
    /// display order: the first error among them, or their sum.
    /// Linear in the row items, so a caller must not call it per cell (T-4149: one
    /// grand-grand cell per data field).
    fn row_totals_sum(&self, data_field: usize) -> Result<f64, Error> {
        sum_in_order(self.rows.children(0).filter_map(|row| {
            let base = *self.cells.get(&(row, 0))? as usize;
            self.accumulators
                .get(base + data_field)
                .map(Accumulator::record_sum)
        }))
    }
}

/// Takes `units` from `work`, or empties it and fails.
pub(super) fn charge(work: &mut u64, units: u64) -> Result<(), PivotComputeError> {
    match work.checked_sub(units) {
        Some(left) => {
            *work = left;
            Ok(())
        }
        None => {
            *work = 0;
            Err(PivotComputeError::Limit)
        }
    }
}

/// `count` as a work charge, saturating.
pub(super) fn units(count: usize) -> u64 {
    u64::try_from(count).unwrap_or(u64::MAX)
}

/// Counts `count` more nodes against [`MAX_PIVOT_NODES`].
fn count_nodes(nodes: &mut usize, count: usize) -> Result<(), PivotComputeError> {
    *nodes = nodes.saturating_add(count);
    if *nodes > MAX_PIVOT_NODES {
        return Err(PivotComputeError::Limit);
    }
    Ok(())
}

/// A field's display order: per cache item its rank, and per rank its cache item. The field's
/// listed data items come first, in `<items>` order (a repeat or an index past the cache items
/// is skipped), then the cache items it does not list, in cache order.
struct Order {
    rank: Vec<u32>,
    item: Vec<u32>,
}

impl Order {
    fn of(field: Option<&PivotField>, item_count: usize) -> Order {
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
}

/// One axis's fields with their records and display orders.
struct AxisColumns<'a> {
    fields: Vec<u32>,
    records: Vec<&'a [u32]>,
    orders: Vec<Order>,
}

impl<'a> AxisColumns<'a> {
    fn new(
        table: &PivotTable,
        cache: &'a PivotCache,
        axis: &[AxisField],
        work: &mut u64,
    ) -> Result<Self, PivotComputeError> {
        let fields: Vec<u32> = axis
            .iter()
            .filter_map(|f| match f {
                AxisField::Field(index) => Some(*index),
                AxisField::Values => None,
            })
            .collect();
        let mut records = Vec::with_capacity(fields.len());
        let mut orders = Vec::with_capacity(fields.len());
        for &field in &fields {
            let cache_field = cache
                .fields
                .get(field as usize)
                .ok_or(PivotComputeError::Malformed)?;
            let pivot_field = table.fields.get(field as usize);
            let listed = pivot_field.map_or(0, |f| f.items.len());
            charge(work, units(cache_field.items.len().saturating_add(listed)))?;
            records.push(cache_field.records.as_slice());
            orders.push(Order::of(pivot_field, cache_field.items.len()));
        }
        Ok(AxisColumns {
            fields,
            records,
            orders,
        })
    }

    /// Writes the display ranks of `record`'s items into `key`.
    fn key(&self, record: u32, key: &mut Vec<u32>) -> Result<(), PivotComputeError> {
        key.clear();
        for (column, order) in self.records.iter().zip(&self.orders) {
            let item = column
                .get(record as usize)
                .ok_or(PivotComputeError::Malformed)?;
            let rank = order
                .rank
                .get(*item as usize)
                .ok_or(PivotComputeError::Malformed)?;
            key.push(*rank);
        }
        Ok(())
    }

    /// The tree of the distinct `keys` (display ranks), and per key id its leaf node.
    fn tree(
        self,
        keys: HashMap<Box<[u32]>, u32>,
        work: &mut u64,
        nodes: &mut usize,
    ) -> Result<(AxisTree, Vec<u32>), PivotComputeError> {
        let mut sorted: Vec<(Box<[u32]>, u32)> = keys.into_iter().collect();
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
        let mut previous: Option<&[u32]> = None;
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
            for (depth, &rank) in key.iter().enumerate().skip(common) {
                let item = self
                    .orders
                    .get(depth)
                    .and_then(|o| o.item.get(rank as usize))
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

/// Ends the subtree of the open node `node` at the tree's current length.
fn close(tree: &mut [AxisNode], node: Option<u32>) -> Result<(), PivotComputeError> {
    let end = node_index(tree.len())?;
    if let Some(n) = node.and_then(|n| tree.get_mut(n as usize)) {
        n.end = end;
    }
    Ok(())
}

/// A node or accumulator index; within [`MAX_PIVOT_NODES`], so it fits a u32.
fn node_index(len: usize) -> Result<u32, PivotComputeError> {
    u32::try_from(len).map_err(|_| PivotComputeError::Limit)
}

/// The work of sorting `n` keys: `n` times its bit length.
fn sort_units(n: usize) -> u64 {
    let n = units(n);
    n.saturating_mul(u64::from(u64::BITS - n.leading_zeros()))
}

/// The records stage 1 keeps, in record order (see the module documentation).
fn filter(
    table: &PivotTable,
    cache: &PivotCache,
    work: &mut u64,
) -> Result<Vec<u32>, PivotComputeError> {
    // (the field's records, the one item it keeps or None, its hidden items)
    let mut tests: Vec<(&[u32], Option<u32>, Option<&PivotField>)> = Vec::new();
    let column = |field: u32| {
        cache
            .fields
            .get(field as usize)
            .map(|f| f.records.as_slice())
            .ok_or(PivotComputeError::Malformed)
    };
    for page in &table.page_fields {
        let records = column(page.field)?;
        // An item index past the field's items, or naming a line that is no data item, is
        // `(All)` (UNVERIFIED, T-4633).
        let selected = table
            .fields
            .get(page.field as usize)
            .and_then(|f| f.items.get(page.item? as usize))
            .filter(|item| item.kind == ItemType::Data)
            .and_then(|item| item.index);
        if selected.is_some() {
            tests.push((records, selected, None));
        }
    }
    for (field, pivot_field) in (0u32..).zip(&table.fields) {
        let on_axis = matches!(
            pivot_field.axis,
            Some(Axis::Row | Axis::Column | Axis::Page)
        );
        if on_axis && pivot_field.hidden.iter().any(|word| *word != 0) {
            tests.push((column(field)?, None, Some(pivot_field)));
        }
    }
    let count = cache.record_count as usize;
    charge(work, units(count.saturating_mul(tests.len().max(1))))?;
    let mut kept = Vec::with_capacity(count);
    'records: for record in 0..cache.record_count {
        for (records, selected, hidden) in &tests {
            let item = *records
                .get(record as usize)
                .ok_or(PivotComputeError::Malformed)?;
            if selected.is_some_and(|s| s != item) || hidden.is_some_and(|f| f.is_hidden(item)) {
                continue 'records;
            }
        }
        kept.push(record);
    }
    Ok(kept)
}

/// The id of `key` in `keys`, adding it when new.
fn intern(keys: &mut HashMap<Box<[u32]>, u32>, key: &[u32]) -> Result<u32, PivotComputeError> {
    if let Some(&id) = keys.get(key) {
        return Ok(id);
    }
    let id = node_index(keys.len())?;
    keys.insert(key.into(), id);
    Ok(id)
}

/// Computes the result of `table` over `cache`, spending at most `work` units ([`MAX_PIVOT_WORK`]
/// for a standalone call; SPEC-0024's run budget once a caller draws on it).
///
/// # Errors
/// [`PivotComputeError::Limit`] past `work` or [`MAX_PIVOT_NODES`] (`work` is then 0), and
/// [`PivotComputeError::Malformed`] when a field index of the definition or a record's item is
/// not in the cache.
pub fn compute(
    table: &PivotTable,
    cache: &PivotCache,
    work: &mut u64,
) -> Result<PivotResult, PivotComputeError> {
    // Counts in an accumulator are u32s: a cache within MAX_RECORDS keeps them exact.
    if cache.record_count as usize > super::MAX_RECORDS {
        return Err(PivotComputeError::Limit);
    }
    let records = filter(table, cache, work)?;
    let rows = AxisColumns::new(table, cache, &table.row_fields, work)?;
    let columns = AxisColumns::new(table, cache, &table.column_fields, work)?;
    charge(
        work,
        units(
            records
                .len()
                .saturating_mul(rows.fields.len() + columns.fields.len() + 1),
        ),
    )?;
    let (mut row_keys, mut column_keys) = (HashMap::new(), HashMap::new());
    let mut key = Vec::new();
    let mut placed = Vec::with_capacity(records.len());
    for &record in &records {
        rows.key(record, &mut key)?;
        let row = intern(&mut row_keys, &key)?;
        columns.key(record, &mut key)?;
        let column = intern(&mut column_keys, &key)?;
        placed.push((row, column, record));
    }
    let mut nodes = 1;
    let (rows, row_leaves) = rows.tree(row_keys, work, &mut nodes)?;
    let (columns, column_leaves) = columns.tree(column_keys, work, &mut nodes)?;
    for (row, column, _) in &mut placed {
        *row = row_leaves.get(*row as usize).copied().unwrap_or(0);
        *column = column_leaves.get(*column as usize).copied().unwrap_or(0);
    }
    charge(work, sort_units(placed.len()))?;
    placed.sort_unstable();

    let data: Vec<(&[u32], &[CacheItem])> = table
        .data_fields
        .iter()
        .map(|d| {
            cache
                .fields
                .get(d.field as usize)
                .map(|f| (f.records.as_slice(), f.items.as_slice()))
                .ok_or(PivotComputeError::Malformed)
        })
        .collect::<Result<_, _>>()?;
    let functions: Vec<Aggregate> = table.data_fields.iter().map(|d| d.function).collect();
    let mut result = PivotResult {
        records,
        rows,
        columns,
        sum_only: functions.iter().all(|f| *f == Aggregate::Sum),
        counts_mixed: functions
            .iter()
            .any(|f| !matches!(f, Aggregate::Count | Aggregate::CountNums)),
        functions,
        cells: HashMap::new(),
        accumulators: Vec::new(),
    };
    accumulate(&mut result, &placed, &data, work, &mut nodes)?;
    if result.sum_only {
        sum_children(&mut result, work)?;
    }
    Ok(result)
}

/// Feeds every placed record (sorted by row leaf, column leaf, record) to each accumulator that
/// covers it, making the accumulators as their cells are first reached.
fn accumulate(
    result: &mut PivotResult,
    placed: &[(u32, u32, u32)],
    data: &[(&[u32], &[CacheItem])],
    work: &mut u64,
    nodes: &mut usize,
) -> Result<(), PivotComputeError> {
    let PivotResult {
        rows,
        columns,
        cells,
        accumulators,
        ..
    } = result;
    let mut bases: Vec<usize> = Vec::new();
    for run in placed.chunk_by(|a, b| (a.0, a.1) == (b.0, b.1)) {
        let Some(&(row_leaf, column_leaf, _)) = run.first() else {
            continue;
        };
        // Every (row ancestor, column ancestor) pair is looked up, data fields or none.
        let pairs = (rows.fields.len() + 1).saturating_mul(columns.fields.len() + 1);
        charge(work, units(pairs))?;
        bases.clear();
        for row in rows.ancestors(row_leaf) {
            for column in columns.ancestors(column_leaf) {
                let base = match cells.get(&(row, column)) {
                    Some(&base) => base,
                    None => {
                        // A cell counts as one node even with no data field.
                        count_nodes(nodes, data.len().max(1))?;
                        charge(work, units(data.len()).saturating_add(1))?;
                        let base = node_index(accumulators.len())?;
                        accumulators.extend((0..data.len()).map(|_| Accumulator::new()));
                        cells.insert((row, column), base);
                        base
                    }
                };
                bases.push(base as usize);
            }
        }
        charge(
            work,
            units(
                run.len()
                    .saturating_mul(bases.len())
                    .saturating_mul(data.len()),
            ),
        )?;
        for &(_, _, record) in run {
            for (d, (records, items)) in data.iter().enumerate() {
                let item = records
                    .get(record as usize)
                    .and_then(|&i| items.get(i as usize))
                    .ok_or(PivotComputeError::Malformed)?;
                for &base in &bases {
                    if let Some(acc) = accumulators.get_mut(base + d) {
                        acc.push(item);
                    }
                }
            }
        }
    }
    Ok(())
}

/// The child cells of every Sum total: `(parent, display key, child)` accumulator bases, sorted
/// by parent and then display order. A cell of a non-leaf row sums its row children in the same
/// column, and a cell of a leaf row its column children (so the grand-grand total sums the row
/// totals, t4616 S1 E8). Each cell is the child of at most two cells, its row parent's and, in a
/// leaf row, its column parent's, so the index has at most twice as many entries as cells.
fn child_cells(
    rows: &AxisTree,
    columns: &AxisTree,
    cells: &HashMap<(u32, u32), u32>,
    work: &mut u64,
) -> Result<Vec<(u32, u32, u32)>, PivotComputeError> {
    let parent = |tree: &AxisTree, node: u32| tree.nodes.get(node as usize).and_then(|n| n.parent);
    charge(work, units(cells.len().saturating_mul(2)))?;
    let mut edges = Vec::new();
    for (&(row, column), &child) in cells {
        if let Some(up) = parent(rows, row).and_then(|r| cells.get(&(r, column))) {
            edges.push((*up, row, child));
        }
        if rows.is_leaf(row) {
            if let Some(up) = parent(columns, column).and_then(|c| cells.get(&(row, c))) {
                edges.push((*up, column, child));
            }
        }
    }
    charge(work, sort_units(edges.len()))?;
    edges.sort_unstable();
    Ok(edges)
}

/// For a pivot whose data fields are all Sum: sets each cell's children sum
/// ([`Accumulator::set_children_sum`]), children before parents ([`child_cells`]); an item
/// cell's is its record sum. The work is linear in the cells (the index is), whatever the axes'
/// shapes: a diagonal pivot of n row and n column items costs O(n), not O(n²).
fn sum_children(result: &mut PivotResult, work: &mut u64) -> Result<(), PivotComputeError> {
    let PivotResult {
        rows,
        columns,
        functions,
        cells,
        accumulators,
        ..
    } = result;
    let edges = child_cells(rows, columns, cells, work)?;
    let depth = |tree: &AxisTree, node: u32| tree.nodes.get(node as usize).map_or(0, |n| n.depth);
    let mut order: Vec<(std::cmp::Reverse<u32>, std::cmp::Reverse<u32>, u32)> = cells
        .iter()
        .map(|(&(row, column), &base)| {
            (
                std::cmp::Reverse(depth(rows, row)),
                std::cmp::Reverse(depth(columns, column)),
                base,
            )
        })
        .collect();
    charge(work, sort_units(order.len()))?;
    order.sort_unstable();
    // Each cell is set once and read at most twice as a child, per data field.
    charge(
        work,
        units(
            order
                .len()
                .saturating_add(edges.len())
                .saturating_mul(functions.len()),
        ),
    )?;
    for (_, _, base) in order {
        let from = edges.partition_point(|e| e.0 < base);
        let to = edges.partition_point(|e| e.0 <= base);
        let children = edges.get(from..to).unwrap_or_default();
        for d in 0..functions.len() {
            let sum = if children.is_empty() {
                accumulators
                    .get(base as usize + d)
                    .map(Accumulator::record_sum)
            } else {
                Some(sum_in_order(children.iter().filter_map(
                    |&(_, _, child)| {
                        accumulators
                            .get(child as usize + d)
                            .map(Accumulator::children_sum)
                    },
                )))
            };
            if let (Some(sum), Some(acc)) = (sum, accumulators.get_mut(base as usize + d)) {
                acc.set_children_sum([sum]);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
