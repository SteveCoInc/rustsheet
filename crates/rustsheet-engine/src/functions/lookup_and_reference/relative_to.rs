//! PIVOTBY's `relative_to`: the relative set each grid cell gives a two-argument function such as
//! PERCENTOF (SPEC-0006 step 6, T-1327). Excel: docs/excel-semantics/arrays.md "GROUPBY and
//! PIVOTBY", evidence probes/groupby-pivotby P!Y25-A73 and probes/groupby-pivotby-functions
//! P!A1-A841 (excel-graph, 2026-10-16), where `LAMBDA(x,y,SUM(y))` shows each cell's set.

use std::collections::{BTreeSet, HashMap};

use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
    expressions::{parser::ArrayNode, token::Error, types::CellReferenceIndex},
    functions::util::array_node_to_calc_result,
    model::Model,
};

use super::grouping::{empty_text, value_error, Axis, Grouping, Line, Relative, SetId};

/// Which records of an axis a cell's relative set keeps: all, those of the cell's own row or
/// column item (a group, a subtotal or the Total), or those of that item's parent, its key prefix
/// one field shorter (for a single field, and for a Total, all).
#[derive(Clone, Copy, PartialEq)]
enum Select {
    All,
    Item,
    Parent,
}

/// The `relative_to` argument: omitted or 0 the column total, 1 the row total, 2 the grand total,
/// 3 the parent column total, 4 the parent row total. As (rows, columns) selections: 0 is all
/// rows of the cell's column, 1 the cell's row in all columns, 3 the cell's row under the parent
/// of its column, 4 the parent of its row in its column (P!Y25, P!A49, P!M49, P!Y49, P!A73; two
/// fields per axis: probes/groupby-pivotby-functions P!A101-A321; with subtotals P!A341-A441).
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct RelativeTo {
    rows: Select,
    columns: Select,
}

impl RelativeTo {
    /// Omitted: the column total.
    pub(crate) fn column_total() -> Self {
        RelativeTo {
            rows: Select::All,
            columns: Select::Item,
        }
    }

    /// The relative set of a group's value in a value sort: the set of its Total cell (the
    /// Total column for rows, the Total row for columns), given its `rows`, the group one level
    /// up, `up` (`None` at the top level), and all `records`: its own rows, all records, or the
    /// group one level up, all records at the top. With `relative_to` 4 over two row fields the
    /// Region parents tie at the grand total and keep their key order (probes/groupby-pivotby-
    /// functions3 P!A91; 1: P!A121); with 3 over two column fields the years tie at their
    /// product's total (Apple 2023 before Apple 2024 descending, both 64), while 0 and 4 sort them
    /// by their own records (49 before 15), 1 and 2 tie at the grand total, with and without
    /// column subtotals and over one column field (probes/groupby-pivotby-functions4 P!A1-A201).
    pub(crate) fn of_sort<'a>(
        self,
        axis: Axis,
        rows: &'a [usize],
        up: Option<Relative<'a>>,
        records: &'a [usize],
    ) -> Relative<'a> {
        let all = Relative {
            id: Some(SetId::All),
            rows: records,
        };
        let select = match axis {
            Axis::Rows => self.rows,
            Axis::Columns => self.columns,
        };
        match select {
            Select::All => all,
            Select::Parent => up.unwrap_or(all),
            Select::Item => Relative { id: None, rows },
        }
    }
}

impl Model {
    /// One `relative_to` element: 0 to 4, text and booleans read as numbers (`"1"` and TRUE are 1:
    /// probes/groupby-pivotby-functions P!A521, P!A541); 5, -1 and 0.5 are `#VALUE!`, also when
    /// the function takes one argument and does not use it (P!A461-A501, P!A581).
    pub(crate) fn relative_to(
        &mut self,
        arg: Option<CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<RelativeTo, CalcResult> {
        use Select::{All, Item, Parent};
        let Some(result) = arg else {
            return Ok(RelativeTo::column_total());
        };
        let (rows, columns) = match self.cast_to_number(result, cell)? {
            0.0 => (All, Item),
            1.0 => (Item, All),
            2.0 => (All, All),
            3.0 => (Item, Parent),
            4.0 => (Parent, Item),
            _ => return Err(value_error(cell, "relative_to must be 0, 1, 2, 3 or 4")),
        };
        Ok(RelativeTo { rows, columns })
    }

    /// The evaluated `relative_to` argument: 0 to 4 ([`Model::relative_to`]), checked whatever
    /// the function (P!A581), or an array, whose elements are returned to be lifted
    /// ([`Model::lifted_relative_to`]; probes/pivotby-followup P!A681).
    pub(crate) fn relative_to_arg(
        &mut self,
        arg: Option<CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<(RelativeTo, Option<Elements>), CalcResult> {
        match arg {
            Some(array @ (CalcResult::Array(_) | CalcResult::Range { .. })) => Ok((
                RelativeTo::column_total(),
                Some(self.result_to_array(array, cell)?),
            )),
            other => Ok((self.relative_to(other, cell)?, None)),
        }
    }

    /// An array `relative_to`, lifted per element as Excel lifts it (`{0;1}` is a 2x1 array,
    /// probes/pivotby-followup P!A681, probes/groupby-pivotby-functions P!A721): each element is
    /// the top-left cell of the PIVOTBY for that element, which is the `""` corner of the header
    /// rows, or the element's error (`{5;1}` is `#VALUE!` over `""`: P!A741). The other arguments
    /// are checked first, an error among them fills the array's shape (`Model::pivotby`), and no
    /// function is called.
    pub(crate) fn lifted_relative_to(
        &mut self,
        elements: Vec<Vec<ArrayNode>>,
        cell: CellReferenceIndex,
    ) -> Vec<Vec<ArrayNode>> {
        elements
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|element| {
                        let element = array_node_to_calc_result(&element, cell);
                        match self.relative_to(Some(element), cell) {
                            Ok(_) => empty_text(),
                            Err(error) => calc_result_to_array_node(error),
                        }
                    })
                    .collect()
            })
            .collect()
    }
}

/// The elements of an array `relative_to`, to be lifted ([`Model::lifted_relative_to`]).
pub(crate) type Elements = Vec<Vec<ArrayNode>>;

/// A row or column item of the grid: a group or subtotal line, or the Total (`None`).
pub(crate) type Item = Option<Line>;

/// The records an item selects on one axis: a key-prefix level and the class (its id in
/// [`Grouping::new`]'s tree) the records reached at that level, or `None` for every record (level
/// 0, or a class no record reached, which holds every record as such a leaf does).
type Class = (usize, Option<usize>);

/// One axis of the grid for the relative sets: the classes each record reached.
struct AxisSets<'g> {
    select: Select,
    fields: usize,
    grouping: &'g Grouping,
    reached: Vec<&'g [usize]>,
}

impl<'g> AxisSets<'g> {
    /// The levels an axis' items select, counted before anything is built; level 0 (every
    /// record) is always among them, as the Total's and an unreached class's.
    fn levels(select: Select, fields: usize, items: &[Line]) -> BTreeSet<usize> {
        let mut levels = BTreeSet::new();
        levels.insert(0);
        for &line in items {
            levels.insert(level(select, fields, Some(line)));
        }
        levels
    }

    fn new(select: Select, grouping: &'g Grouping, height: usize) -> Self {
        AxisSets {
            select,
            fields: grouping.fields(),
            grouping,
            reached: grouping.reached_classes(height),
        }
    }

    /// The class of the records an item selects: those whose descent reached the item's class at
    /// its level, a record that reached no leaf under it included, or every record when none did,
    /// as a subtotal cell's records are (`Grid::cell_rows`, [`Grouping::class_rows`];
    /// probes/t4382-pivotby-intransitive-paths Calc!D2, E17; T-4402). While `=` is transitive
    /// these are the records of the item's groups.
    fn class(&self, item: Item) -> Class {
        let level = level(self.select, self.fields, item);
        let group = match item {
            Some(Line::Group(group)) => group,
            Some(Line::Subtotal { start, .. }) => start,
            None => 0,
        };
        match self.grouping.reached_class(group, level) {
            Some(class) => (level, Some(class)),
            None => (0, None),
        }
    }

    /// The class `record` reached at `level` (`None` at level 0), or nothing when its descent
    /// stopped above that level.
    fn class_of_record(&self, record: usize, level: usize) -> Option<Class> {
        if level == 0 {
            return Some((0, None));
        }
        let path = self.reached.get(record)?;
        path.get(level - 1).map(|&class| (level, Some(class)))
    }
}

/// The level of the key prefix a selection keeps of an item over `fields` key fields: 0 for all
/// records, the item's own for `Item`, one field shorter for `Parent`.
fn level(select: Select, fields: usize, item: Item) -> usize {
    let level = match item {
        None => 0,
        Some(Line::Group(_)) => fields,
        Some(Line::Subtotal { level, .. }) => level,
    };
    match select {
        Select::All => 0,
        Select::Item => level,
        Select::Parent => level.saturating_sub(1),
    }
}

/// Most record entries the relative sets of one PIVOTBY call may hold (each record once per pair
/// of key-prefix levels the cells use): 2^24, 128 MiB of indices, so that many key fields with
/// subtotals at every level cannot fill memory (directive 5). Excel has no such limit:
/// DEVIATION, docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY: two-parameter LAMBDAs and the
/// cell-visit cap".
const MAX_SET_ENTRIES: u64 = 1 << 24;

/// The relative sets of one PIVOTBY call, each built once and shared by its cells: for each pair
/// of levels the cells use (a row level, a column level), the records bucketed by the row and
/// column classes they reached at those levels, in source order. A cell's set is the records of
/// its row item's class that are in its column item's class ([`AxisSets::class`]).
pub(crate) struct RelativeSets<'g> {
    rows: AxisSets<'g>,
    cols: AxisSets<'g>,
    index: HashMap<(Class, Class), usize>,
    sets: Vec<Vec<usize>>,
}

impl<'g> RelativeSets<'g> {
    /// The sets of `records` (in source order) for the grid of `rows` and `cols` with the row and
    /// column lines `row_lines` and `col_lines`. Each level pair holds each record at most once; past
    /// [`MAX_SET_ENTRIES`] the whole result is `#ERROR!`, checked from the level counts before any
    /// set, prefix table or call is made (only many key fields with subtotals at many levels get
    /// there).
    pub(crate) fn new(
        relative_to: RelativeTo,
        (rows, cols): (&'g Grouping, &'g Grouping),
        (row_lines, col_lines): (&[Line], &[Line]),
        records: &[usize],
        cell: CellReferenceIndex,
    ) -> Result<Self, CalcResult> {
        let row_levels = AxisSets::levels(relative_to.rows, rows.fields(), row_lines);
        let col_levels = AxisSets::levels(relative_to.columns, cols.fields(), col_lines);
        let pairs = (row_levels.len() as u64).saturating_mul(col_levels.len() as u64);
        if pairs.saturating_mul(records.len() as u64) > MAX_SET_ENTRIES {
            return Err(CalcResult::new_error(
                Error::ERROR,
                cell,
                "Too many relative set entries".to_string(),
            ));
        }
        let height = records.iter().max().map_or(0, |&row| row + 1);
        let mut sets = RelativeSets {
            rows: AxisSets::new(relative_to.rows, rows, height),
            cols: AxisSets::new(relative_to.columns, cols, height),
            index: HashMap::new(),
            sets: Vec::new(),
        };
        for &row_level in &row_levels {
            for &col_level in &col_levels {
                for &record in records {
                    let (Some(row_class), Some(col_class)) = (
                        sets.rows.class_of_record(record, row_level),
                        sets.cols.class_of_record(record, col_level),
                    ) else {
                        continue;
                    };
                    let key = (row_class, col_class);
                    let next = sets.sets.len();
                    let at = *sets.index.entry(key).or_insert(next);
                    if at == next {
                        sets.sets.push(Vec::new());
                    }
                    if let Some(set) = sets.sets.get_mut(at) {
                        set.push(record);
                    }
                }
            }
        }
        Ok(sets)
    }

    /// The relative set of the cell in row item `row` and column item `col`.
    pub(crate) fn of_cell(&self, row: Item, col: Item) -> Relative<'_> {
        let key = (self.rows.class(row), self.cols.class(col));
        let at = self.index.get(&key).copied();
        Relative {
            id: at.map(SetId::Pivot),
            rows: at
                .and_then(|at| self.sets.get(at))
                .map_or(&[][..], Vec::as_slice),
        }
    }
}
