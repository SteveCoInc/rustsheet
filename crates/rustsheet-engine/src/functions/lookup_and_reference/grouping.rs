//! The grouping core GROUPBY and PIVOTBY share (SPEC-0006 Design steps 2 and 4, T-1322, T-1323):
//! records grouped by a key tuple, sorted by key or by an aggregate, and laid out with the
//! subtotals of a total depth beyond 1 (T-1324). Excel: docs/excel-semantics/arrays.md "GROUPBY
//! and PIVOTBY".

use std::cmp::Ordering;
use std::collections::HashMap;

use std::borrow::Cow;

use super::group_order::{compare_aggregates, key_cmp};
use crate::area_budget::text_units;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::ArrayNode, token::Error, types::CellReferenceIndex},
    functions::{
        spill_functions::{
            cell_key, has_intransitive_text, unique_cell_key, unique_first_lines, CellKey,
        },
        util::{collation::cmp_text_pairwise, sort_indices_pairwise},
    },
};

/// How the groups are ordered: GROUPBY's `sort_order` (PIVOTBY's `row_sort_order` and
/// `col_sort_order` too), read by `Model::sort_order`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SortOrder {
    /// By the key fields, `ascending[i]` for field `i`; fields not named ascend
    /// ([`Grouping::sort_by_keys`]). `named` lists the fields `sort_order` names, in written
    /// order: a table relationship sorts by them first ([`Grouping::sort_table_by_keys`]).
    Keys {
        ascending: Vec<bool>,
        named: Vec<usize>,
    },
    /// By the aggregate of value column `column` (0-based among the value columns), with
    /// [`Grouping::sort_by_aggregate`].
    Value { column: usize, ascending: bool },
}

/// The directions [`Grouping::new`] builds its tree in for `sort`: a hierarchy's key directions
/// (probes/t4241-groupby-intransitive-runs Calc!C, E), ascending for a table relationship
/// (`order` [`ParentOrder::Flat`]) and for a sort by a value, which match Excel on intransitive
/// runs (probes/t4344-groupby-descending-search B55:E66, probes/t4359-groupby-value-sort-parents).
pub(crate) fn tree_directions(sort: &SortOrder, order: ParentOrder) -> &[bool] {
    match sort {
        SortOrder::Keys { ascending, .. } if order != ParentOrder::Flat => ascending,
        _ => &[],
    }
}

/// `#VALUE!` for an invalid GROUPBY or PIVOTBY argument.
pub(crate) fn value_error(cell: CellReferenceIndex, message: &str) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, message.to_string())
}

/// A cell of a 2-D array, blank when outside it (arrays from `eval_to_array` are rectangular, so
/// this only guards against a panic).
pub(crate) fn at(data: &[Vec<ArrayNode>], row: usize, column: usize) -> ArrayNode {
    data.get(row)
        .and_then(|cells| cells.get(column))
        .cloned()
        .unwrap_or(ArrayNode::Empty)
}

/// A relative set: the source rows a two-argument function gets as its second argument (all rows
/// for GROUPBY, the `relative_to` set for PIVOTBY). `id` names a set that is built once and shared,
/// so PERCENTOF sums it once; `None` for a set made for one call.
#[derive(Clone, Copy)]
pub(crate) struct Relative<'a> {
    pub(crate) id: Option<SetId>,
    pub(crate) rows: &'a [usize],
}

/// The axis of a grid: PIVOTBY's rows (and GROUPBY's groups) or its columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Axis {
    Rows,
    Columns,
}

/// Which shared set a [`Relative`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum SetId {
    /// All (kept) records.
    All,
    /// Set `n` of PIVOTBY's relative sets (`RelativeSets`).
    Pivot(usize),
    /// Parent `group` at sort level `level` of `axis` ([`Grouping::sort_by_aggregate`]).
    Parent {
        axis: Axis,
        level: usize,
        group: usize,
    },
}

/// Empty text, the filler of total rows, corners and empty PIVOTBY cells (not a blank: G!AC33).
pub(crate) fn empty_text() -> ArrayNode {
    ArrayNode::String("".into())
}

/// One group: its key tuple as first met in the source (the first spelling, `East` for `East`
/// and `east`, G!A1) and the indices of its source rows, in source order.
pub(crate) struct Group {
    pub(crate) keys: Vec<ArrayNode>,
    pub(crate) rows: Vec<usize>,
    /// The keys as they group: per field, the id of the group's class in that field's level of
    /// [`Grouping::new`]'s tree, so that comparing key prefixes later compares no text.
    cells: Vec<usize>,
    /// Per field, the position of the group's class among its parent's classes, in the order
    /// [`Grouping::new`] built them (its `ascending`): texts sort by it, not by comparing them
    /// again.
    ranks: Vec<usize>,
    /// Whether any row reached the group's leaf; a leaf none reaches holds every row
    /// ([`Grouping::new`] step 4).
    reached: bool,
}

impl Group {
    /// Orders two groups' keys in `field`: texts under one parent (equal keys in the fields
    /// before it) by their rank, the order [`Grouping::new`] sorted them in, `built` (ascending
    /// or not), so `"AB"` before `"ab"&U+00AD` (probes/t4195-groupby-intransitive Calc!B31:B38);
    /// anything else by [`key_cmp`].
    fn key_order(
        &self,
        other: &Group,
        field: usize,
        (ascending, built): (bool, bool),
        cell: CellReferenceIndex,
    ) -> Ordering {
        let (Some(x), Some(y)) = (self.keys.get(field), other.keys.get(field)) else {
            return Ordering::Equal;
        };
        match (x, y, self.ranks.get(field), other.ranks.get(field)) {
            (ArrayNode::String(_), ArrayNode::String(_), Some(a), Some(b))
                if self.cells.get(..field) == other.cells.get(..field) =>
            {
                if ascending == built {
                    a.cmp(b)
                } else {
                    b.cmp(a)
                }
            }
            _ => key_cmp(x, y, ascending, cell),
        }
    }
}

/// Rows grouped by a key tuple: the unit GROUPBY lays out as rows and PIVOTBY as a grid.
pub(crate) struct Grouping {
    pub(crate) groups: Vec<Group>,
    /// The directions [`Grouping::new`] built the groups in, per field (ascending past the end):
    /// the ranks' order.
    ascending: Vec<bool>,
    /// Each grouped source row with the ids of the classes it reached, one per field from the
    /// first (fewer when it reached no leaf; [`Grouping::new`] step 3), as [`Group::cells`] holds
    /// them.
    paths: Vec<(usize, Vec<usize>)>,
    /// Per field, the source rows (in source order) that reached each class of it, by class id:
    /// a subtotal's and a value sort parent's rows ([`Grouping::class_rows`]).
    reached_by: Vec<HashMap<usize, Vec<usize>>>,
    /// Every grouped source row, in source order.
    every: Vec<usize>,
}

impl Grouping {
    /// Groups `rows` (indices into `keys`, one key tuple per row) as Excel's GROUPBY and PIVOTBY
    /// do, with each field sorted ascending or not by `ascending` (ascending past its end). Two
    /// keys of a field are one when Excel's `=` finds them equal (case folded, `ß` with `ss`, the
    /// prefix rule: probes/t4086-groupby-weightless, probes/t4088-groupby-multifield B14:B21;
    /// T-4088); numbers compare exactly ([`group_key`]). Excel's model, from
    /// probes/t4195-groupby-intransitive, probes/t4241-groupby-intransitive-runs and the hold-out
    /// probes/t4342-groupby-tree-holdout (docs/excel-semantics/coercion.md "GROUPBY on an
    /// intransitive run"), is a tree over UNIQUE's sorted keys:
    /// 1. The keys are the rows UNIQUE keeps ([`unique_first_lines`]), sorted on whole rows by
    ///    SORT's merge ([`sort_indices_pairwise`]), each field in its direction ([`Keys::sorted`]).
    /// 2. Classes are built field by field over that list: under one parent a key joins the last
    ///    class when its field `=` the field of the class's first key, else it starts a class; a
    ///    class shows its first key's spelling (so `abc` for (`ABC`,y), (`abc`,x): t4342 Calc!B51).
    /// 3. Each row descends the tree by a lower-bound search over its parent's classes, in the
    ///    direction its first two classes run in (T-4344: on an intransitive run a descending
    ///    field's can run ascending), over [0, n-1] ([`Tree::descend`]); it is in no group when the
    ///    class found is not `=` to it.
    /// 4. A leaf no row reaches aggregates every row (t4241 Calc!C8, F2): such a row is in two
    ///    groups.
    ///
    /// While `=` is transitive (no key text outside ASCII) this is grouping by equality, the
    /// groups in key order, and each row descends as the first row equal to it does. The groups
    /// come out in tree order, which [`Grouping::sort_by_keys`] keeps for the same `ascending`.
    /// Every comparison is charged to `spend`. Fails only with `spend`'s error or `limit`'s.
    /// `limit(n)` is the error `n` groups would give (the output size check, SPEC-0006 step 6);
    /// the tree stops at the first group that does not fit, before any row descends it.
    pub(crate) fn new(
        keys: &[Vec<ArrayNode>],
        rows: impl IntoIterator<Item = usize>,
        ascending: &[bool],
        limit: impl Fn(usize) -> Option<CalcResult>,
        mut spend: impl FnMut(u64) -> Result<(), CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<Self, CalcResult> {
        let rows: Vec<usize> = rows.into_iter().filter(|&row| row < keys.len()).collect();
        let fields = rows
            .first()
            .and_then(|&row| keys.get(row))
            .map_or(0, Vec::len);
        let lines = Keys {
            keys,
            rows: &rows,
            ascending,
            cell,
        };
        let first = unique_first_lines(
            rows.len(),
            fields,
            |line, field| lines.at(line, field),
            &mut spend,
            cell,
        )?;
        let sorted = lines.sorted(&first, fields, &mut spend)?;
        let tree = Tree::build(&lines, &sorted, fields, limit, &mut spend)?;
        let intransitive =
            has_intransitive_text(rows.len(), fields, |line, field| lines.at(line, field));
        // members[leaf]: the lines that reach it; reached[line]: the classes the line reached.
        let mut members: Vec<Vec<usize>> = vec![Vec::new(); tree.leaves()];
        let mut reached: Vec<Vec<usize>> = vec![Vec::new(); rows.len()];
        for line in 0..rows.len() {
            // While `=` is transitive a row descends as the first row equal to it (UNIQUE's).
            let classes = match first.get(line) {
                Some(&kept) if !intransitive && kept < line => {
                    reached.get(kept).cloned().unwrap_or_default()
                }
                _ => tree.descend(&lines, line, fields, &mut spend)?,
            };
            let leaf = classes.last().copied().filter(|_| classes.len() == fields);
            if let Some(entry) = leaf.and_then(|leaf| members.get_mut(leaf)) {
                entry.push(line);
            }
            if let Some(slot) = reached.get_mut(line) {
                *slot = classes;
            }
        }
        let mut groups = Vec::with_capacity(members.len());
        for (leaf, members) in members.into_iter().enumerate() {
            let path = tree.path(leaf);
            let reached = !members.is_empty();
            // A leaf no row reaches holds every row: charged per row, so crafted intransitive
            // keys with many such leaves end as the work budget's error, not as leaves x rows
            // of memory (directive 5).
            let rows_of_leaf: Vec<usize> = if reached {
                members
                    .iter()
                    .filter_map(|&line| rows.get(line).copied())
                    .collect()
            } else {
                spend(u64::try_from(rows.len()).unwrap_or(u64::MAX))?;
                rows.clone()
            };
            groups.push(Group {
                keys: path
                    .iter()
                    .enumerate()
                    .map(|(field, node)| {
                        lines
                            .at(node.key, field)
                            .map_or_else(empty_text, positive_zero)
                    })
                    .collect(),
                rows: rows_of_leaf,
                cells: path.iter().map(|node| node.id).collect(),
                ranks: path.iter().map(|node| node.rank).collect(),
                reached,
            });
        }
        let mut reached_by: Vec<HashMap<usize, Vec<usize>>> = vec![HashMap::new(); fields];
        for (&row, path) in rows.iter().zip(&reached) {
            for (classes, &class) in reached_by.iter_mut().zip(path) {
                classes.entry(class).or_default().push(row);
            }
        }
        for members in reached_by.iter_mut().flat_map(HashMap::values_mut) {
            members.sort_unstable();
        }
        let paths = rows.iter().copied().zip(reached).collect();
        let mut every = rows;
        every.sort_unstable();
        Ok(Grouping {
            groups,
            ascending: ascending.to_vec(),
            paths,
            reached_by,
            every,
        })
    }

    /// The number of key fields.
    pub(crate) fn fields(&self) -> usize {
        self.groups.first().map_or(0, |group| group.keys.len())
    }

    /// For each group, the id of its first `level` keys (groups sharing them share the id; all
    /// 0 for level 0): the key prefixes of PIVOTBY's relative sets.
    pub(crate) fn prefix_ids(&self, level: usize) -> Vec<usize> {
        let mut index: HashMap<&[usize], usize> = HashMap::new();
        self.groups
            .iter()
            .map(|group| {
                let prefix = group.cells.get(..level).unwrap_or(group.cells.as_slice());
                let next = index.len();
                *index.entry(prefix).or_insert(next)
            })
            .collect()
    }

    /// For each of `height` source rows, the index of its group (`usize::MAX` for rows in no
    /// group: headers and filtered rows). A row in a group no row reached too ([`Grouping::new`]
    /// step 4) maps to the group it reached; such a group gets only the rows in no other
    /// (PIVOTBY's columns give such a group every record instead, [`Grouping::unreached`];
    /// relative sets unprobed).
    pub(crate) fn group_of(&self, height: usize) -> Vec<usize> {
        let mut group_of = vec![usize::MAX; height];
        for reached in [true, false] {
            for (index, group) in self.groups.iter().enumerate() {
                if group.reached != reached {
                    continue;
                }
                for &row in &group.rows {
                    if let Some(entry) = group_of.get_mut(row).filter(|entry| **entry == usize::MAX)
                    {
                        *entry = index;
                    }
                }
            }
        }
        group_of
    }

    /// For each of `height` source rows, the ids of the classes it reached ([`Grouping::paths`]):
    /// empty for rows in no class (headers, filtered rows, or a first field no class equals).
    pub(crate) fn reached_classes(&self, height: usize) -> Vec<&[usize]> {
        let mut classes: Vec<&[usize]> = vec![&[]; height];
        for (row, path) in &self.paths {
            if let Some(slot) = classes.get_mut(*row) {
                *slot = path;
            }
        }
        classes
    }

    /// The source rows (in source order) of class `class` of field `field` as a parent: those
    /// whose descent reached it, a row that reached no leaf under it included, and every row
    /// when none did, as a leaf no row reaches holds every row ([`Grouping::new`] step 4). A
    /// subtotal's rows, whatever the sort, and a value sort's parent's
    /// (probes/t4359-groupby-value-sort-parents, probes/t4359-groupby-value-sort-holdout; T-4359).
    /// While `=` is transitive that is the rows of the class's groups.
    fn class_rows(&self, field: usize, class: usize) -> &[usize] {
        self.reached_by
            .get(field)
            .and_then(|classes| classes.get(&class))
            .map_or(&self.every[..], Vec::as_slice)
    }

    /// The class ids of group `group`'s first `level` keys, the prefix a subtotal at that level
    /// covers (empty when there is no such group).
    pub(crate) fn prefix(&self, group: usize, level: usize) -> &[usize] {
        self.groups
            .get(group)
            .and_then(|group| group.cells.get(..level))
            .unwrap_or_default()
    }

    /// The indices of the groups no row reached ([`Grouping::new`] step 4), which hold every row.
    pub(crate) fn unreached(&self) -> Vec<usize> {
        self.groups
            .iter()
            .enumerate()
            .filter(|(_, group)| !group.reached)
            .map(|(index, _)| index)
            .collect()
    }

    /// Sorts the groups stably by `cmp` with [`sort_indices_pairwise`], which cannot panic: texts
    /// compare by rank under one parent and by `cmp_text` across parents ([`Group::key_order`]),
    /// and a `sort_order` naming a later field first can mix the two into a cycle, which
    /// `slice::sort_by` may panic on.
    fn sort_groups(&mut self, mut cmp: impl FnMut(&Group, &Group) -> Ordering) {
        let mut order: Vec<usize> = (0..self.groups.len()).collect();
        let groups = &self.groups;
        sort_indices_pairwise(&mut order, |a, b| match (groups.get(a), groups.get(b)) {
            (Some(x), Some(y)) => cmp(x, y),
            _ => Ordering::Equal,
        });
        let mut slots: Vec<Option<Group>> = std::mem::take(&mut self.groups)
            .into_iter()
            .map(Some)
            .collect();
        self.groups = order
            .into_iter()
            .filter_map(|index| slots.get_mut(index).and_then(Option::take))
            .collect();
    }

    /// Sorts the groups by their keys, field by field, `ascending[i]` for field `i` (ascending
    /// when missing). Mixed types sort numbers < text < FALSE < TRUE (G!A193) and the blank key
    /// sorts last in both directions (G!A1, G!A145): that is `sort_key_cmp`. Numbers compare
    /// exactly, not to 15 digits: 0.3 sorts before 0.1+0.2 (probes/groupby-pivotby-followup
    /// G!A761), matching the exact `cell_key` grouping. (Errors against text, booleans and the
    /// blank key: unprobed, SPEC-0006.)
    pub(crate) fn sort_by_keys(&mut self, ascending: &[bool], cell: CellReferenceIndex) {
        let built = self.ascending.clone();
        let direction =
            |directions: &[bool], field: usize| directions.get(field).copied().unwrap_or(true);
        self.sort_groups(|a, b| {
            (0..a.keys.len().min(b.keys.len()))
                .map(|field| {
                    let directions = (direction(ascending, field), direction(&built, field));
                    a.key_order(b, field, directions, cell)
                })
                .find(|ordering| ordering.is_ne())
                .unwrap_or(Ordering::Equal)
        });
    }

    /// Sorts the groups of a GROUPBY table relationship (`field_relationship` 1) by their keys:
    /// the fields `sort_order` names first, in written order, then the others ascending in column
    /// order, so `sort_order` 2 sorts by the second key and then the first, and `{-1,2}` and
    /// `{2,-1}` differ, where a hierarchy always sorts in column order (probes/t3838-groupby-
    /// option-order G!B179-B216 against G!B217-B229). A descending field reverses its ascending
    /// order whole, so the blank key sorts first there (G!B205, B207), where a hierarchy keeps it
    /// last (G!B217). Mixed types descending in a table: unprobed, read the same way.
    pub(crate) fn sort_table_by_keys(
        &mut self,
        ascending: &[bool],
        named: &[usize],
        cell: CellReferenceIndex,
    ) {
        let fields = self.groups.first().map_or(0, |group| group.keys.len());
        let priority: Vec<usize> = named
            .iter()
            .copied()
            .chain((0..fields).filter(|field| !named.contains(field)))
            .collect();
        let built = self.ascending.clone();
        self.sort_groups(|a, b| {
            priority
                .iter()
                .map(|&field| {
                    let built = built.get(field).copied().unwrap_or(true);
                    let ordering = a.key_order(b, field, (true, built), cell);
                    if ascending.get(field).copied().unwrap_or(true) {
                        ordering
                    } else {
                        ordering.reverse()
                    }
                })
                .find(|ordering| ordering.is_ne())
                .unwrap_or(Ordering::Equal)
        });
    }

    /// Sorts the groups by an aggregate (GROUPBY's `sort_order` naming a value column; PIVOTBY's
    /// sort by a total), and returns the aggregates it sorted by, each group's and (when the
    /// parents sort by value) each parent's, so the caller shows them, subtotals included, instead
    /// of calling the function again: no duplicate calls, and a volatile function shows the values
    /// the order follows. `aggregate` gets source rows in source order.
    ///
    /// With several key fields the sort is hierarchical: the parents (key prefixes) are ordered
    /// by the aggregate of all their rows, and the children by their own value inside each parent
    /// (probes/groupby-pivotby-followup2 G!A261; groupby-sort-filter-args B21-B22 descending).
    /// [`ParentOrder::ByKey`] keeps the parents in key order and sorts only the last level by
    /// value, as PIVOTBY's columns do without subtotals (probes/pivotby-followup P!A1141, P!A1161);
    /// [`ParentOrder::Flat`] sorts every group by its own value, ignoring the parents, as GROUPBY's
    /// table relationship does (probes/groupby-subtotals G!A391-A481).
    /// Equal values keep the ascending key order in both directions (groupby-sort-filter-args
    /// B32-B33, B27-B28), and the blank key is not pinned last (G!J145, G!S145). Values compare like
    /// key cells: numbers < text < FALSE < TRUE < errors, errors last ascending and first
    /// descending (B34-B37; groupby-sort-filter-args2 B27-B28, B37). A parent's value is the
    /// function of all its rows, not of its children's values (AVERAGE and `100-SUM(x)` over two
    /// keys, args2 B22-B25), at every level (three keys, args2 B19-B21). On an intransitive run
    /// a parent's rows are [`Grouping::class_rows`] (t4344 Calc!D56; T-4359), so a shown subtotal
    /// is the value its parent sorted by.
    ///
    /// `aggregate` also gets the group one level up as a relative set (PIVOTBY's `relative_to` 4 sorts
    /// rows by it, 3 columns: `RelativeTo::of_sort`); `None` at the top level. The parents' rows
    /// are gathered for that whatever the order (O(rows log rows) per level).
    ///
    /// Work: one `aggregate` call per group and per parent, at most groups x key fields, which the
    /// output size bounds; each level's parent rows are gathered once (O(rows log rows) per level).
    pub(crate) fn sort_by_aggregate(
        &mut self,
        ascending: bool,
        (order, axis): (ParentOrder, Axis),
        cell: CellReferenceIndex,
        mut aggregate: impl FnMut(&[usize], Option<Relative<'_>>) -> ArrayNode,
    ) -> Aggregates {
        self.sort_by_keys(&[], cell);
        let fields = match order {
            ParentOrder::Flat => 1,
            _ => self.groups.first().map_or(0, |group| group.keys.len()),
        };
        let parents_by_value = order == ParentOrder::ByValue;
        // `levels[l][g]`: the parent of group g at depth l + 1 (an index in key order of first
        // appearance, so equal parent values keep key order), with the parents' aggregates.
        let mut levels: Vec<(Vec<usize>, Vec<ArrayNode>)> = Vec::new();
        let mut previous = vec![0; self.groups.len()];
        // The rows of the previous level's parents, handed to `aggregate` as the group one level up.
        let mut previous_rows: Vec<Vec<usize>> = Vec::new();
        for level in 0..fields.saturating_sub(1) {
            let mut index: HashMap<(usize, usize), usize> = HashMap::new();
            let mut parent_rows: Vec<Vec<usize>> = Vec::new();
            // The previous level's parent of each parent.
            let mut above: Vec<usize> = Vec::new();
            let mut parents = Vec::with_capacity(self.groups.len());
            for (group, parent) in self.groups.iter().zip(&previous) {
                let key = group.cells.get(level).copied().unwrap_or(usize::MAX);
                let id = *index.entry((*parent, key)).or_insert_with(|| {
                    parent_rows.push(self.class_rows(level, key).to_vec());
                    above.push(*parent);
                    parent_rows.len() - 1
                });
                parents.push(id);
            }
            let values = parent_rows
                .iter()
                .zip(&above)
                .map(|(rows, &parent)| {
                    if !parents_by_value {
                        return ArrayNode::Empty;
                    }
                    let up = level
                        .checked_sub(1)
                        .and_then(|level| parent_set(axis, level, &previous_rows, parent));
                    aggregate(rows, up)
                })
                .collect();
            previous.clone_from(&parents);
            previous_rows = parent_rows;
            levels.push((parents, values));
        }
        let leaves: Vec<ArrayNode> = self
            .groups
            .iter()
            .zip(&previous)
            .map(|(group, &parent)| {
                let up = fields
                    .checked_sub(2)
                    .and_then(|level| parent_set(axis, level, &previous_rows, parent));
                aggregate(&group.rows, up)
            })
            .collect();
        let mut order: Vec<usize> = (0..self.groups.len()).collect();
        // A stable sort: groups that compare equal stay in key order.
        order.sort_by(|&a, &b| {
            for (parents, values) in &levels {
                let (x, y) = (parents.get(a), parents.get(b));
                if x != y {
                    let (x, y) = (x.copied().unwrap_or(0), y.copied().unwrap_or(0));
                    if !parents_by_value {
                        return x.cmp(&y);
                    }
                    return compare_aggregates(values.get(x), values.get(y), ascending, cell)
                        .then(x.cmp(&y));
                }
            }
            compare_aggregates(leaves.get(a), leaves.get(b), ascending, cell)
        });
        let mut groups: Vec<Option<Group>> = std::mem::take(&mut self.groups)
            .into_iter()
            .map(Some)
            .collect();
        self.groups = order
            .iter()
            .filter_map(|&at| groups.get_mut(at).and_then(Option::take))
            .collect();
        let in_order = |values: &[usize]| -> Vec<usize> {
            order
                .iter()
                .map(|&at| values.get(at).copied().unwrap_or(0))
                .collect()
        };
        Aggregates {
            leaves: order
                .iter()
                .map(|&at| leaves.get(at).cloned().unwrap_or(ArrayNode::Empty))
                .collect(),
            parents: if parents_by_value {
                levels
                    .iter()
                    .map(|(parents, values)| (in_order(parents), values.clone()))
                    .collect()
            } else {
                Vec::new()
            },
        }
    }
}

/// Parent `group` of sort level `level` of `axis` as a relative set
/// ([`Grouping::sort_by_aggregate`]).
fn parent_set(axis: Axis, level: usize, rows: &[Vec<usize>], group: usize) -> Option<Relative<'_>> {
    rows.get(group).map(|rows| Relative {
        id: Some(SetId::Parent { axis, level, group }),
        rows,
    })
}

/// The key tuples of a grouping's rows as lines (a line is an index into `rows`), and how two
/// lines' keys compare in one field ([`Grouping::new`]).
struct Keys<'a> {
    keys: &'a [Vec<ArrayNode>],
    rows: &'a [usize],
    /// Per field, whether it sorts ascending (ascending past the end).
    ascending: &'a [bool],
    cell: CellReferenceIndex,
}

impl<'a> Keys<'a> {
    /// Line `line`'s key in `field`.
    fn at(&self, line: usize, field: usize) -> Option<&'a ArrayNode> {
        self.rows
            .get(line)
            .and_then(|&row| self.keys.get(row))
            .and_then(|tuple| tuple.get(field))
    }

    /// The work units of comparing two lines' keys in `field` ([`text_units`] of two texts).
    fn units(&self, a: usize, b: usize, field: usize) -> u64 {
        match (self.at(a, field), self.at(b, field)) {
            (Some(ArrayNode::String(x)), Some(ArrayNode::String(y))) => {
                text_units(x.len() + y.len()).max(1)
            }
            _ => 1,
        }
    }

    /// Two lines' keys in `field` in that field's direction: texts by Excel's `<` and `=`
    /// ([`cmp_text_pairwise`], reversed descending), keys with one [`group_key`] equal, anything
    /// else by [`key_cmp`] (the blank key last both ways).
    fn cmp(&self, a: usize, b: usize, field: usize) -> Ordering {
        let ascending = self.ascending.get(field).copied().unwrap_or(true);
        match (self.at(a, field), self.at(b, field)) {
            (Some(ArrayNode::String(x)), Some(ArrayNode::String(y))) => {
                let ordering = cmp_text_pairwise(x, y);
                if ascending {
                    ordering
                } else {
                    ordering.reverse()
                }
            }
            (Some(x), Some(y)) if group_key(x) == group_key(y) => Ordering::Equal,
            (Some(x), Some(y)) => key_cmp(x, y, ascending, self.cell),
            _ => Ordering::Equal,
        }
    }

    /// The lines UNIQUE keeps (`first[line] == line`), sorted on whole rows by SORT's merge
    /// ([`sort_indices_pairwise`]), each field in its direction: probes/t4241-groupby-intransitive-
    /// runs Calc!C (`sort_order` -1 reverses the first field) and E (-2, the second).
    fn sorted(
        &self,
        first: &[usize],
        fields: usize,
        mut spend: impl FnMut(u64) -> Result<(), CalcResult>,
    ) -> Result<Vec<usize>, CalcResult> {
        let mut sorted: Vec<usize> = (0..first.len())
            .filter(|&line| first.get(line) == Some(&line))
            .collect();
        let mut cost = 0u64;
        sort_indices_pairwise(&mut sorted, |a, b| {
            (0..fields)
                .map(|field| {
                    cost = cost.saturating_add(self.units(a, b, field));
                    self.cmp(a, b, field)
                })
                .find(|ordering| ordering.is_ne())
                .unwrap_or(Ordering::Equal)
        });
        spend(cost)?;
        Ok(sorted)
    }
}

/// A key cell's bucket as it groups: UNIQUE's (`unique_cell_key`: a text by its collation key),
/// except that -0 is 0. Excel stores a cell `=-0` as 0, so 0, `=-0` and `=0` are one group
/// (probes/groupby-pivotby-followup G!A781); the engine keeps the sign of a computed -0.
fn group_key(node: &ArrayNode) -> CellKey {
    match node {
        ArrayNode::Number(n) if *n == 0.0 => cell_key(&ArrayNode::Number(0.0)),
        other => unique_cell_key(other),
    }
}

/// A key cell as a group shows it, -0 as 0 (see [`group_key`]).
fn positive_zero(node: &ArrayNode) -> ArrayNode {
    match node {
        ArrayNode::Number(n) if *n == 0.0 => ArrayNode::Number(0.0),
        other => other.clone(),
    }
}

/// How [`Grouping::sort_by_aggregate`] orders the parents (key prefixes) of a value sort.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ParentOrder {
    /// Each parent by the aggregate of all its rows, at every level: GROUPBY, PIVOTBY's rows, and
    /// its columns with subtotals (probes/pivotby-followup P!A941; probes/groupby-subtotals G!A961,
    /// G!A991).
    ByValue,
    /// The parents in key order: PIVOTBY's columns without subtotals (P!A1141, P!A1161).
    ByKey,
    /// No parents: every group by its own value, as GROUPBY's table relationship sorts
    /// (probes/groupby-subtotals G!A391-A481).
    Flat,
}

/// One line of a grouping laid out with subtotals: group `index`, or the subtotal of the groups
/// `start..end`, which share their first `level` keys and no others.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Line {
    Group(usize),
    Subtotal {
        level: usize,
        start: usize,
        end: usize,
    },
}

/// The aggregates a value sort ordered the groups by ([`Grouping::sort_by_aggregate`]).
pub(crate) struct Aggregates {
    /// Each group's, in the sorted order.
    leaves: Vec<ArrayNode>,
    /// Per level l (the first l + 1 keys), each group's parent (in the sorted order) and each
    /// parent's aggregate; empty unless the parents sort by value.
    parents: Vec<(Vec<usize>, Vec<ArrayNode>)>,
}

impl Aggregates {
    /// The aggregate of a group or subtotal line, when the sort computed it: a subtotal of the
    /// first `level` keys is the aggregate of the parent at that level.
    pub(crate) fn of_line(&self, line: Line) -> Option<&ArrayNode> {
        match line {
            Line::Group(index) => self.leaves.get(index),
            Line::Subtotal { level, start, .. } => {
                let (parents, values) = self.parents.get(level.checked_sub(1)?)?;
                values.get(*parents.get(start)?)
            }
        }
    }
}

impl Grouping {
    /// The key cells and the source rows (in source order) of a group or subtotal line: a
    /// group's keys as shown, or a subtotal's ([`Grouping::subtotal_label_cells`]) over the rows
    /// of the class it subtotals ([`Grouping::subtotal_rows`]). `None` for a group that does not
    /// exist.
    pub(crate) fn line_parts(
        &self,
        line: Line,
        key_columns: usize,
    ) -> Option<(Vec<ArrayNode>, Cow<'_, [usize]>)> {
        match line {
            Line::Group(index) => {
                let group = self.groups.get(index)?;
                // A blank key is a true blank in the array (INDEX TYPE 1, COUNT skips it); it displays 0
                // when spilled (G!A1; T-2460, T-2854).
                let label = group.keys.clone();
                Some((label, Cow::Borrowed(&group.rows[..])))
            }
            Line::Subtotal { level, start, .. } => Some((
                self.subtotal_label_cells(level, start, key_columns),
                Cow::Borrowed(self.subtotal_rows(level, start)),
            )),
        }
    }

    /// The groups in their order with the subtotals of the first `levels` key fields (a total
    /// depth of ±(`levels` + 1)): after the groups they cover, the deepest first, or before them
    /// (`before`, a negative depth), the shallowest first. Depth 2 over three keys subtotals only
    /// the first, 3 both (probes/groupby-subtotals G!A1, G!A61, G!A91; G!J49, G!S49).
    ///
    /// Groups that share a key prefix are next to each other after any sort used with subtotals
    /// (the key sort is lexicographic, the value sort hierarchical), so one linear pass over the
    /// groups finds the subtotals (SPEC-0006 Design step 2); there are at most `levels` per group.
    pub(crate) fn lines(&self, levels: usize, before: bool) -> Vec<Line> {
        let count = self.groups.len();
        // `breaks[i]`: the first key field where group i differs from group i - 1, and 0 before
        // the first group and after the last, where every subtotal starts and ends.
        let breaks: Vec<usize> = (0..=count).map(|i| self.first_difference(i)).collect();
        let mut starts = vec![0; levels + 1];
        let mut lines = Vec::with_capacity(count);
        for (index, pair) in breaks.windows(2).enumerate() {
            let &[here, next] = pair else {
                continue;
            };
            for start in starts.iter_mut().skip(here + 1) {
                *start = index;
            }
            lines.push(Line::Group(index));
            for level in (next + 1..=levels).rev() {
                let start = starts.get(level).copied().unwrap_or(index);
                lines.push(Line::Subtotal {
                    level,
                    start,
                    end: index + 1,
                });
            }
        }
        if !before {
            return lines;
        }
        // Above its groups: each subtotal goes right before its first group, the shallowest
        // first.
        let (mut subtotals, groups): (Vec<Line>, Vec<Line>) = lines
            .into_iter()
            .partition(|line| matches!(line, Line::Subtotal { .. }));
        subtotals.sort_by_key(|line| match *line {
            Line::Subtotal { level, start, .. } => (start, level),
            Line::Group(index) => (index, 0),
        });
        let mut subtotals = subtotals.into_iter().peekable();
        let mut ordered = Vec::with_capacity(groups.len() + subtotals.len());
        for group in groups {
            let Line::Group(index) = group else {
                continue;
            };
            while let Some(line) = subtotals
                .next_if(|line| matches!(line, Line::Subtotal { start, .. } if *start == index))
            {
                ordered.push(line);
            }
            ordered.push(group);
        }
        ordered
    }

    /// Sorts by the keys `order` names, or ascending when it sorts by a value (which
    /// [`Grouping::sort_by_aggregate`] does later, after the output size check): sorting by keys
    /// calls no function.
    pub(crate) fn sort_by_key_order(&mut self, order: &SortOrder, cell: CellReferenceIndex) {
        match order {
            SortOrder::Keys { ascending, .. } => self.sort_by_keys(ascending, cell),
            SortOrder::Value { .. } => self.sort_by_keys(&[], cell),
        }
    }

    /// The first key field where group `index` differs from the group before it, 0 for the
    /// first group and past the last one.
    fn first_difference(&self, index: usize) -> usize {
        let previous = index.checked_sub(1).and_then(|i| self.groups.get(i));
        let (Some(previous), Some(group)) = (previous, self.groups.get(index)) else {
            return 0;
        };
        previous
            .cells
            .iter()
            .zip(&group.cells)
            .position(|(a, b)| a != b)
            .unwrap_or(group.cells.len())
    }

    /// The source rows, in source order, of the subtotal of the groups from `start` that share
    /// their first `level` keys: the [`Grouping::class_rows`] of that prefix's class, which while
    /// `=` is transitive are the rows of those groups. A subtotal's function gets its values in
    /// source order, as a group's does (`LAMBDA(x,ARRAYTOTEXT(x))`: East is "10, 30, 5",
    /// probes/groupby-subtotals G!A331, G!A1081). On an intransitive run, whatever the sort, it
    /// counts a row that reached the class but no leaf under it, and not the extra rows of a leaf
    /// no row reaches (probes/t4359-groupby-value-sort-holdout Calc!E, F; T-4359).
    fn subtotal_rows(&self, level: usize, start: usize) -> &[usize] {
        let field = level.saturating_sub(1);
        match self
            .groups
            .get(start)
            .and_then(|group| group.cells.get(field))
        {
            Some(&class) => self.class_rows(field, class),
            None => &[],
        }
    }

    /// The key cells of the subtotal of the groups from `start` that share their first `level`
    /// keys: those keys, then `""` (G!J49; PIVOTBY's subtotal columns too, P!M97).
    pub(crate) fn subtotal_label_cells(
        &self,
        level: usize,
        start: usize,
        key_columns: usize,
    ) -> Vec<ArrayNode> {
        let keys = self
            .groups
            .get(start)
            .map_or(&[][..], |group| &group.keys[..]);
        (0..key_columns)
            .map(|field| match keys.get(field) {
                Some(key) if field < level => key.clone(),
                _ => empty_text(),
            })
            .collect()
    }
}

/// The total depth one axis is laid out with: as given, except that a value sort over several
/// fields with the depth omitted shows subtotals at every level, as the full depth does
/// (two keys: probes/pivotby-followup P!A921 for GROUPBY, P!A201, P!A901 for PIVOTBY's rows,
/// P!A941 for its columns; three keys: probes/groupby-subtotals G!A121). An explicit depth is
/// kept (followup2 G!A261; P!A861, P!A1141), and so is an omitted one with `col_total_depth` 0,
/// although the rows then keep their key order (G!A1021).
pub(crate) fn effective_depth(depth: isize, given: bool, sort: &SortOrder, fields: usize) -> isize {
    if !given && fields > 1 && matches!(sort, SortOrder::Value { .. }) {
        isize::try_from(fields).unwrap_or(isize::MAX)
    } else {
        depth
    }
}

/// The number of subtotal levels of a total depth: none for 0 and ±1, the first |depth| - 1 key
/// fields otherwise.
pub(crate) fn subtotal_levels(depth: isize) -> usize {
    depth.unsigned_abs().saturating_sub(1)
}

mod tree;

use tree::Tree;

#[cfg(test)]
mod tests;
