//! The pivot result tree: a pivot's filtered records and the accumulator of every (row node,
//! column node, data field), its subtotals and grand totals included (SPEC-0032 "`PivotResult`",
//! stages 1 to 3; T-4142).
//!
//! [`compute`] is a pure function of the definition and the cache:
//! 1. **Record filter.** A page field's selected item keeps only its records, and a hidden item
//!    (`item@h`, [`PivotField::is_hidden`](super::PivotField::is_hidden)) of a field on an axis drops its records. Only a page
//!    field's selection and hidden items are evidenced (probes/t4137-pivot-captions K4); hidden
//!    items of a row or column field, and the totals over them, are UNVERIFIED (T-4164). A page
//!    `item` that names no data item reads as `(All)`, as T-4139's import does, and a hidden item
//!    on a field off every axis is ignored (both UNVERIFIED, T-4633; docs/excel-semantics/
//!    pivots.md "Record filters"). A slicer's selection replaces its field's hidden items, on an
//!    axis or not, and a timeline keeps only the records whose day its selection spans
//!    (`pivot::slicer`, T-4188; probes/t4767-pivot-slicers, probes/t4767-pivot-timelines).
//!
//!    Then the pivot's `<filters>` and `AutoShow` (`pivot::filter`, T-4165): caption and date
//!    filters, then value filters and Top-N per parent, each keeping the records of the items it
//!    keeps, so the totals cover only what is shown. Then the value sorts (`pivot::sort`,
//!    T-4827) place each sorted field's items within their parent by the value its
//!    `autoSortScope` names, over the records the filters kept.
//! 2. **Groups** are already the records' items: a grouped field's records index its group items
//!    (`number_groups`, `date_groups`).
//! 3. **Accumulate.** Each axis is a tree of item tuples in display order: a field's listed
//!    items in `<items>` order, then the cache items it does not list, in cache order (pivots.md
//!    "Items show in the order the pivot field lists them"), a value-sorted field's items first by
//!    their sort position within their parent. Records are fed to every
//!    accumulator that covers them in one order: by row leaf, then column leaf, in display
//!    order, and in record order within a leaf. So an item cell accumulates in record order, and
//!    a total accumulates item by item in the display order of the field it totals, in record
//!    order within an item: Excel's order for Average, Var, `StdDev`, Varp, `StdDevp` and
//!    Product, and for Sum beside any other aggregate (pivots.md "Summation order of Sum,
//!    Average and Product totals": probes/t4634-pivot-total-order-4 X1 to X12, A1 to A3, V1 to
//!    V7; t4616-pivot-total-order-3 S2, S3, V1 to V4, P1, P2; t4614 S2, P2; t4589 S1). A cell
//!    that is a total on both axes (the grand-grand total, a subtotal by a subtotal or by a
//!    grand total) instead adds the sums and products of the leaf-row cells below it in its
//!    column, in row display order (`merge_crossings`; t4634 T2, T3, X1 to X12, V1, V6; Varp,
//!    `StdDevp` and Product by analogy, UNVERIFIED, T-4678).
//!    An axis of one field with `showAll` on (`pivotField@showAll`, on when absent; Excel and the
//!    case builder write `showAll="0"`) also shows the items it does not hide that no kept
//!    record holds, in display order, with no accumulator, so their cells are blank
//!    (probes/t4311-pivot-showall-column-and-groups H: a grouped row field, read back from
//!    Excel's own layout in Calc I1:J8). On a column field only GETPIVOTDATA is evidenced
//!    (t4311 Calc B1, B3, B5 answer 0 on P!A13 with `showAll="1"`, against `#REF!` with it off;
//!    P!A13's `rendered` cells are the builder's, not Excel's), so the rendered column is
//!    UNVERIFIED (T-4657). On an axis of several fields `showAll` is not applied, and whether an
//!    item the page filter leaves without records shows is unprobed (both UNVERIFIED, T-4657).
//!    The rest is read from the accumulators ([`PivotResult::value_as`]):
//!    - in a pivot whose data fields are all Sum, a total is the sum of leaf cells in display
//!      order, never of the subtotals between: a cell of a non-leaf row sums the leaf-row cells
//!      below it in its column, and a total in a leaf row sums that row's leaf-column cells
//!      below its column (t4634 T1, O1 to O4, S1; t4616 S1, S4 to S6; t4135 `A_sum!S6`, S8).
//!      Any other aggregate beside Sum switches Sum to the records as above (t4634 X1 to X12).
//!
//!    A calculated data field is fed nothing: once the cells are full its value is its formula
//!    over each cell's Sums of the fields it names, extra accumulators (`pivot::calculated`).
//! 4. **Show values as** (stage 6, `pivot::show_as`): a post-pass over the values above that
//!    replaces each value of a data field with a show-values-as mode by what the mode shows, for
//!    every cell, leaves, subtotals and grand totals alike (T-4163).
//!
//! Stage 7, the layout lines and location, is `pivot::layout::lay_out`, a separate call on the
//! result, so that a pivot too large to lay out still has its values (T-4147).
//!
//! Every stage charges its work before doing it, and the nodes and accumulators are counted
//! against [`MAX_PIVOT_NODES`] as they are made, so an untrusted pivot is bounded (CLAUDE.md
//! directive 5; pivots.md "A pivot result is bounded in work and nodes", DEVIATION).

mod calculate;

use std::borrow::Cow;
use std::collections::HashMap;

use thiserror::Error as ThisError;

use crate::expressions::token::Error;

use super::accumulate::{sum_in_order, Accumulator, CutCount};
use super::calculated::Plan;
use super::filter::{apply as apply_filters, FilterContext};
use super::model::bit;
use super::show_as::{show_values_as, Shown};
use super::sort::value_sorts;
use super::{Aggregate, Axis, CacheItem, ItemType, PageField, PivotCache, PivotTable};
use calculate::calculate;

mod axis;

pub(super) use axis::{axis_fields, Order};
use axis::{place, AxisColumns};

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
    /// The layout would pass the sheet's last row or column. The grid writer collapses such a
    /// pivot to `#SPILL!` (pivots.md "The grid writer's own readings", T-4151).
    #[error("a pivot's layout passes the sheet's last row or column")]
    PastSheet,
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
    pub(crate) fn ancestors(&self, node: u32) -> impl Iterator<Item = u32> + '_ {
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
    /// Per data field, the values its show-values-as mode shows (`show_as`); `None` for a field
    /// shown as itself.
    shown: Vec<Option<Shown>>,
    /// Accumulators per cell: one per data field, then one per database field a calculated
    /// field's formula reads (`calculated::Plan::operands`).
    slots: usize,
    /// Per data field, whether it is over a calculated field.
    calculated_fields: Vec<bool>,
    /// For a pivot with a calculated data field, per accumulator index, the value of the
    /// calculated data field of that slot (`calculated`, T-4174); empty otherwise.
    calculated: Vec<Option<Result<f64, Error>>>,
    /// Per data field over a calculated field, its formula's value where no record reaches: over
    /// sums of 0.
    calculated_empty: Vec<Option<Result<f64, Error>>>,
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
    /// (`PivotField::custom_subtotals`). A data field with a show-values-as mode shows the mode's
    /// value under its own function (`pivot::show_as`), and its plain value under another
    /// (UNVERIFIED, T-4750).
    #[must_use]
    pub fn value_as(
        &self,
        row: u32,
        column: u32,
        data_field: usize,
        function: Aggregate,
    ) -> Option<Result<f64, Error>> {
        if let Some(Some(shown)) = self.shown.get(data_field) {
            if self.functions.get(data_field) == Some(&function) {
                return shown.get(&(row, column)).cloned();
            }
        }
        self.plain_as(row, column, data_field, function)
    }

    /// The data fields' functions.
    pub(super) fn functions(&self) -> &[Aggregate] {
        &self.functions
    }

    /// [`PivotResult::value_as`] before show values as: the aggregate of the cell's records.
    pub(super) fn plain_as(
        &self,
        row: u32,
        column: u32,
        data_field: usize,
        function: Aggregate,
    ) -> Option<Result<f64, Error>> {
        if data_field >= self.functions.len() {
            return None;
        }
        let base = self.cells.get(&(row, column)).map(|&base| base as usize);
        // A calculated field shows its formula's value whatever the function, the data field's
        // or a custom subtotal's (pivots.md "Calculated fields as Excel for the web renders
        // them", CNT; a custom subtotal is unprobed: UNVERIFIED, T-4897), and where no record
        // reaches, its formula over sums of 0 (LC B7).
        if self.calculated_fields.get(data_field) == Some(&true) {
            return match base {
                Some(base) => self.calculated.get(base + data_field).cloned().flatten(),
                None => self.calculated_empty.get(data_field).cloned().flatten(),
            };
        }
        self.slot_as(row, column, base?, data_field, function)
    }

    /// The value under `function` of accumulator `slot` of the cell (`row`, `column`), whose
    /// accumulators start at `base`: a data field's, or a calculated field's operand's. An
    /// operand slot (`slot >= functions.len()`) has no function of its own: it is read as Sum.
    fn slot_as(
        &self,
        row: u32,
        column: u32,
        base: usize,
        slot: usize,
        function: Aggregate,
    ) -> Option<Result<f64, Error>> {
        let accumulator = self.accumulators.get(base + slot)?;
        let total = !self.rows.is_leaf(row) || !self.columns.is_leaf(column);
        // A custom Count or CountNums subtotal under a Sum data field counts each inner-item
        // cell up to its first error (pivots.md "Which function the countA and count custom
        // subtotals compute", t4622). Under the other aggregates it is unprobed and keeps the
        // Count total rule (UNVERIFIED, T-4636).
        let own = self.functions.get(slot).copied();
        if total
            && own == Some(Aggregate::Sum)
            && matches!(function, Aggregate::Count | Aggregate::CountNums)
        {
            return Some(accumulator.cut_count(function, self.counts_mixed));
        }
        // A Sum over records that are all blank shows blank, as an intersection no record
        // reaches does (docs/excel-semantics/pivots.md "Aggregates over text, blanks, booleans
        // and errors": probes/t4164-pivot-value-sort SEd B7, SEa B4). A Sum total covering only
        // such records is blank too, and the other aggregates over them keep the worksheet
        // functions' answers (both UNVERIFIED, T-4633).
        if function == Aggregate::Sum && accumulator.is_blank() {
            return None;
        }
        let sum = (function == Aggregate::Sum && total && self.sum_only)
            .then(|| accumulator.children_sum());
        Some(accumulator.finish(function, total, self.counts_mixed, sum))
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
pub(super) fn count_nodes(nodes: &mut usize, count: usize) -> Result<(), PivotComputeError> {
    *nodes = nodes.saturating_add(count);
    if *nodes > MAX_PIVOT_NODES {
        return Err(PivotComputeError::Limit);
    }
    Ok(())
}

/// A node or accumulator index; within [`MAX_PIVOT_NODES`], so it fits a u32.
fn node_index(len: usize) -> Result<u32, PivotComputeError> {
    u32::try_from(len).map_err(|_| PivotComputeError::Limit)
}

/// The work of sorting `n` keys: `n` times its bit length.
pub(super) fn sort_units(n: usize) -> u64 {
    let n = units(n);
    n.saturating_mul(u64::from(u64::BITS - n.leading_zeros()))
}

/// The records stage 1 keeps, in record order (see the module documentation).
fn filter(
    table: &PivotTable,
    cache: &PivotCache,
    work: &mut u64,
) -> Result<Vec<u32>, PivotComputeError> {
    /// What one field asks of a record's item.
    enum Test<'a> {
        /// The page field's one selected item.
        Page(u32),
        /// No item of this bitset over the field's cache items: the slicer's selection or the
        /// hidden items (borrowed), or the items a timeline's span leaves out (built once).
        Out(Cow<'a, [u64]>),
    }
    let mut tests: Vec<(&[u32], Test<'_>)> = Vec::new();
    let column = |field: u32| {
        cache
            .fields
            .get(field as usize)
            .map(|f| f.records.as_slice())
            .ok_or(PivotComputeError::Malformed)
    };
    for page in &table.page_fields {
        let records = column(page.field)?;
        if let Some(selected) = page_selection(table, page) {
            tests.push((records, Test::Page(selected)));
        }
    }
    for (field, pivot_field) in (0u32..).zip(&table.fields) {
        let on_axis = matches!(
            pivot_field.axis,
            Some(Axis::Row | Axis::Column | Axis::Page)
        );
        // A slicer filters its field whether or not the field is on an axis (UNVERIFIED,
        // T-4962: only a row field is probed), hidden items only a field on one.
        let slicer = pivot_field.slicer.is_some();
        if (slicer || on_axis) && pivot_field.excludes_any() {
            let words = Cow::Borrowed(pivot_field.excluded_words());
            tests.push((column(field)?, Test::Out(words)));
        }
        if let Some(span) = pivot_field.timeline {
            let items = cache
                .fields
                .get(field as usize)
                .map(|f| f.items.as_slice())
                .ok_or(PivotComputeError::Malformed)?;
            // Each cache item tested once, not each record.
            charge(work, units(items.len()))?;
            let mut out = vec![0_u64; items.len().div_ceil(64)];
            for (index, item) in items.iter().enumerate() {
                if let (false, Some(word)) = (span.keeps(item), out.get_mut(index / 64)) {
                    *word |= 1 << (index % 64);
                }
            }
            tests.push((column(field)?, Test::Out(Cow::Owned(out))));
        }
    }
    let count = cache.record_count as usize;
    charge(work, units(count.saturating_mul(tests.len().max(1))))?;
    let mut kept = Vec::with_capacity(count);
    'records: for record in 0..cache.record_count {
        for (records, test) in &tests {
            let item = *records
                .get(record as usize)
                .ok_or(PivotComputeError::Malformed)?;
            let keeps = match test {
                Test::Page(selected) => *selected == item,
                Test::Out(words) => !bit(words, item),
            };
            if !keeps {
                continue 'records;
            }
        }
        kept.push(record);
    }
    Ok(kept)
}

/// The cache item page field `page` selects, if any. An item index past the field's items, or
/// naming a line that is no data item, selects none, as `(All)` does (UNVERIFIED, T-4633).
pub(super) fn page_selection(table: &PivotTable, page: &PageField) -> Option<u32> {
    table
        .fields
        .get(page.field as usize)
        .and_then(|f| f.items.get(page.item? as usize))
        .filter(|item| item.kind == ItemType::Data)
        .and_then(|item| item.index)
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
    compute_on(table, cache, None, work)
}

/// [`compute`] on the day `today` (a serial day, as TODAY gives it from the workbook's clock),
/// which the relative date filters read (`pivot::filter`); with `None` they keep every item.
///
/// # Errors
/// As [`compute`].
pub fn compute_on(
    table: &PivotTable,
    cache: &PivotCache,
    today: Option<f64>,
    work: &mut u64,
) -> Result<PivotResult, PivotComputeError> {
    // Counts in an accumulator are u32s: a cache within MAX_RECORDS keeps them exact.
    if cache.record_count as usize > super::MAX_RECORDS {
        return Err(PivotComputeError::Limit);
    }
    let records = filter(table, cache, work)?;
    let mut kept = apply_filters(table, cache, records, FilterContext { today }, work)?;
    let mut rows = AxisColumns::new(table, cache, &table.row_fields, work)?;
    let mut columns = AxisColumns::new(table, cache, &table.column_fields, work)?;
    let [row_sorts, column_sorts] = value_sorts(
        table,
        cache,
        &kept,
        [&rows.orders(), &columns.orders()],
        work,
    )?;
    rows.sort(row_sorts, work)?;
    columns.sort(column_sorts, work)?;
    let records = std::mem::take(&mut kept.records);
    charge(
        work,
        units(
            records
                .len()
                .saturating_mul(rows.fields.len() + columns.fields.len() + 1),
        ),
    )?;
    let (rows, columns, mut placed, mut nodes) = place(rows, columns, &records, &kept, work)?;
    charge(work, sort_units(placed.len()))?;
    placed.sort_unstable();

    let data_fields: Vec<u32> = table.data_fields.iter().map(|d| d.field).collect();
    let mut plan = Plan::new(&data_fields, &cache.fields, work)?;
    let operands = plan.as_ref().map_or(&[][..], |p| p.operands.as_slice());
    // Per accumulator slot, the field's records and items: `None` for a calculated data field,
    // whose accumulator is fed nothing.
    let data: Vec<Option<(&[u32], &[CacheItem])>> = (data_fields.iter().chain(operands))
        .map(|&field| {
            let f = cache
                .fields
                .get(field as usize)
                .ok_or(PivotComputeError::Malformed)?;
            Ok(f.formula
                .is_none()
                .then_some((f.records.as_slice(), f.items.as_slice())))
        })
        .collect::<Result<_, _>>()?;
    let calculated_fields: Vec<bool> = data
        .iter()
        .take(data_fields.len())
        .map(Option::is_none)
        .collect();
    let functions: Vec<Aggregate> = table.data_fields.iter().map(|d| d.function).collect();
    // A calculated data field shows the Sum whatever its function, so the flags read it as Sum
    // (UNVERIFIED, T-4897: no probe has a calculated field beside an error or a Sum-only check).
    let flagged = || {
        (functions.iter().zip(&calculated_fields))
            .map(|(&f, &calculated)| if calculated { Aggregate::Sum } else { f })
    };
    let mut result = PivotResult {
        records,
        rows,
        columns,
        sum_only: flagged().all(|f| f == Aggregate::Sum),
        counts_mixed: flagged().any(|f| !matches!(f, Aggregate::Count | Aggregate::CountNums)),
        functions,
        cells: HashMap::new(),
        accumulators: Vec::new(),
        shown: Vec::new(),
        slots: data.len(),
        calculated_fields,
        calculated: Vec::new(),
        calculated_empty: Vec::new(),
    };
    accumulate(&mut result, &placed, &data, work, &mut nodes)?;
    merge_crossings(&mut result, work)?;
    if result.sum_only {
        sum_children(&mut result, work)?;
    }
    if let Some(plan) = plan.as_mut() {
        calculate(&mut result, plan, work)?;
    }
    result.shown = show_values_as(&result, table, work, &mut nodes)?;
    Ok(result)
}

/// Feeds every placed record (sorted by row leaf, column leaf, record) to each accumulator that
/// covers it, making the accumulators as their cells are first reached.
fn accumulate(
    result: &mut PivotResult,
    placed: &[(u32, u32, u32)],
    data: &[Option<(&[u32], &[CacheItem])>],
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
    let mut cuts: Vec<CutCount> = Vec::new();
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
        // The run is one inner-item cell, in record order: its counts that end at its first
        // error go to every cell that covers it (`Accumulator::cut_count`).
        cuts.clear();
        cuts.resize(data.len(), CutCount::default());
        for &(_, _, record) in run {
            for (d, (field, cut)) in data.iter().zip(&mut cuts).enumerate() {
                let Some((records, items)) = field else {
                    continue;
                };
                let item = records
                    .get(record as usize)
                    .and_then(|&i| items.get(i as usize))
                    .ok_or(PivotComputeError::Malformed)?;
                cut.push(item);
                for &base in &bases {
                    if let Some(acc) = accumulators.get_mut(base + d) {
                        acc.push(item);
                    }
                }
            }
        }
        for (d, cut) in cuts.iter().enumerate() {
            for &base in &bases {
                if let Some(acc) = accumulators.get_mut(base + d) {
                    acc.add_cut(*cut);
                }
            }
        }
    }
    Ok(())
}

/// Rebuilds the order-dependent folds of every cell that is a total on both axes from the cells
/// of the leaf rows below it in the same column, in row display order: a grand-grand total, a
/// subtotal by a subtotal or by a grand total. Each leaf-row cell has its records accumulated
/// item by item, so such a total adds those partial sums rather than accumulating its records
/// (t4634 T2 and T3 every cell, X1 to X12 H9, V1 and V6 E11, X6 and X7 I9; t4616 S2 H9, V1
/// E11). Varp, `StdDevp` and Product follow it by analogy: their t4634 cells fit it, but also
/// fit other readings (UNVERIFIED, T-4678). Every leaf-row cell is folded into each of its row
/// ancestors, so the work is the cells times the row depth.
fn merge_crossings(result: &mut PivotResult, work: &mut u64) -> Result<(), PivotComputeError> {
    let PivotResult {
        rows,
        columns,
        slots,
        cells,
        accumulators,
        ..
    } = result;
    let slots = *slots;
    let mut parts: Vec<(u32, u32)> = cells
        .keys()
        .filter(|&&(row, column)| rows.is_leaf(row) && !columns.is_leaf(column))
        .copied()
        .collect();
    if parts.is_empty() {
        return Ok(());
    }
    charge(work, sort_units(parts.len()))?;
    parts.sort_unstable();
    let depth = rows.fields.len().saturating_add(1);
    charge(
        work,
        units(
            cells
                .len()
                .saturating_add(parts.len().saturating_mul(depth))
                .saturating_mul(slots),
        ),
    )?;
    for (&(row, column), &base) in cells.iter() {
        if !rows.is_leaf(row) && !columns.is_leaf(column) {
            for d in 0..slots {
                if let Some(acc) = accumulators.get_mut(base as usize + d) {
                    acc.clear_folds();
                }
            }
        }
    }
    for (row, column) in parts {
        let Some(&part) = cells.get(&(row, column)) else {
            continue;
        };
        for up in rows.ancestors(row).skip(1) {
            let Some(&total) = cells.get(&(up, column)) else {
                continue;
            };
            for d in 0..slots {
                let folds = accumulators.get(part as usize + d).map(Accumulator::folds);
                if let (Some(folds), Some(acc)) = (folds, accumulators.get_mut(total as usize + d))
                {
                    acc.add_folds(folds);
                }
            }
        }
    }
    Ok(())
}

/// The child cells of every Sum total: `(parent, display key, child)` accumulator bases, sorted
/// by parent and then display order. A cell of a non-leaf row sums the cells of the leaf rows
/// below it in the same column, and a cell of a leaf row the leaf-column cells below its column:
/// leaf cells, never the subtotals between (t4634 T1 every cell; so the grand-grand total sums
/// the row totals, t4616 S1 E8). Each cell is the child of at most one cell per row ancestor
/// and, in a leaf row, per column ancestor, so the index has at most the cells times the axis
/// depths entries.
fn child_cells(
    rows: &AxisTree,
    columns: &AxisTree,
    cells: &HashMap<(u32, u32), u32>,
    work: &mut u64,
) -> Result<Vec<(u32, u32, u32)>, PivotComputeError> {
    let depths = rows.fields.len().saturating_add(columns.fields.len());
    charge(work, units(cells.len().saturating_mul(depths.max(1))))?;
    let mut edges = Vec::new();
    for (&(row, column), &child) in cells {
        if !rows.is_leaf(row) {
            continue;
        }
        for up in rows.ancestors(row).skip(1) {
            if let Some(&parent) = cells.get(&(up, column)) {
                edges.push((parent, row, child));
            }
        }
        if columns.is_leaf(column) {
            for up in columns.ancestors(column).skip(1) {
                if let Some(&parent) = cells.get(&(row, up)) {
                    edges.push((parent, column, child));
                }
            }
        }
    }
    charge(work, sort_units(edges.len()))?;
    edges.sort_unstable();
    Ok(edges)
}

/// For a pivot whose data fields are all Sum: sets each cell's children sum
/// ([`Accumulator::set_children_sum`]), children before parents ([`child_cells`]); an item
/// cell's is its record sum. The work is linear in the cells times the axis depths (the index
/// is), whatever the axes' shapes: a diagonal pivot of n row and n column items costs O(n), not
/// O(n²).
fn sum_children(result: &mut PivotResult, work: &mut u64) -> Result<(), PivotComputeError> {
    let PivotResult {
        rows,
        columns,
        slots,
        cells,
        accumulators,
        ..
    } = result;
    let slots = *slots;
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
    // Each cell is set once and read once per edge, per data field.
    charge(
        work,
        units(
            order
                .len()
                .saturating_add(edges.len())
                .saturating_mul(slots),
        ),
    )?;
    for (_, _, base) in order {
        let from = edges.partition_point(|e| e.0 < base);
        let to = edges.partition_point(|e| e.0 <= base);
        let children = edges.get(from..to).unwrap_or_default();
        for d in 0..slots {
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
