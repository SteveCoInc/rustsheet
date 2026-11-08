use super::pivot_index::values_on_axis;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::lift::{LiftCost, LiftOptions},
    model::Model,
    pivot::page_selection,
};

/// The `#REF!` of every GETPIVOTDATA that finds no answer.
fn no_answer(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::REF, cell, "No pivot table answer".to_string())
}

/// Where a pair's field sits on the pivot.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Place {
    /// The row axis field at this depth.
    Row(usize),
    /// The column axis field at this depth.
    Column(usize),
    /// The page field of this cache field, with its selected cache item (`None` for `(All)`).
    Page(u32, Option<u32>),
    /// The Values field.
    Values,
}

/// What one call needs of its pivot, read once before the elements of a lift.
struct Shape {
    /// The cache fields of the row axis, then of the column axis, outer first.
    rows: Vec<u32>,
    columns: Vec<u32>,
    /// Lowercased cache field names, by cache field.
    names: Vec<String>,
    /// The page fields: cache field and selected cache item.
    pages: Vec<(u32, Option<u32>)>,
    /// Per cache field, whether a line totals its items (`defaultSubtotal` on, no custom
    /// subtotal).
    subtotals: Vec<bool>,
    /// The Values field's lowercased caption, when the pivot shows several data fields.
    values: Option<String>,
    /// The data fields' lowercased captions, then their lowercased source field names.
    captions: Vec<String>,
    sources: Vec<String>,
}

impl Model {
    /// `=GETPIVOTDATA(data_field, pivot_table, [field1, item1, ...])` (SPEC-0015 "GETPIVOTDATA",
    /// docs/excel-semantics/web.md "GETPIVOTDATA against a pivot table" and "GETPIVOTDATA on
    /// other pivot shapes (T-2085)", T-2083, T-4154). Error arguments (T-4006 gen-m6 cells,
    /// T-4823), in order: a typed error literal as `data_field` (`#NULL!,...,2,"C"` is
    /// `#NULL!`), an error in `pivot_table` that is not a cell of a pivot, an error in any
    /// `field` as is (`..,#NAME?,2` is `#NAME?`), then a *referenced* error as `data_field` is
    /// `#REF!`, as is an error in `item`. Then the pivot whose area meets the reference answers
    /// the value it shows for the data field and the items of the pairs: anything it does not
    /// show is `#REF!`, as is no pivot or several and a pivot shape SPEC-0015 still lists as
    /// unknown. An array `data_field`, field name or item lifts (IT Q!E9:F10; L1, L5).
    pub(crate) fn fn_getpivotdata(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // Excel for the web refuses the workbook for 5, 7, ... arguments (a second or later field
        // without its item), so, like every refused count, it is `#ERROR!`, before any argument
        // is read (docs/excel-semantics/syntax.md "GETPIVOTDATA with an odd count").
        if args.len() < 2 || (args.len() >= 5 && args.len() % 2 == 1) {
            return CalcResult::new_args_number_error(cell);
        }
        let data_field = self.evaluate_node_in_context(&args[0], cell);
        // A typed error (a literal; a computed one is unprobed) is the result; a referenced one waits behind the field errors
        // and is then `#REF!` (T-4823). Typed-vs-field order is unprobed: typed goes first.
        if data_field.is_error() && !self.is_reference(&args[0], cell) {
            return data_field;
        }
        let pivot = match self.evaluate_node_with_reference(&args[1], cell) {
            CalcResult::Range { left, right } => {
                let pivot = self.pivot_at(left, right);
                // A cell of a pivot picks it whatever it holds, an error included (IT Q!E20:E23);
                // an error cell outside every pivot is the result (T-1703).
                if left == right && pivot.is_none() {
                    let value = self.evaluate_cell(left);
                    if value.is_error() {
                        return value;
                    }
                }
                // A pivot collapsed to `#SPILL!` answers only through its anchor
                // (docs/excel-semantics/pivots.md "GETPIVOTDATA against a pivot collapsed to
                // #SPILL!": a range holding it is `#REF!`, probes/t4130-pivot-spill-getpivotdata
                // Q!B6).
                match pivot {
                    Some(p) if self.pivot_collapsed(p) => {
                        let anchor = self.workbook.pivot_tables.get(p).map(|t| t.location);
                        let is_anchor = left == right
                            && anchor.is_some_and(|a| {
                                i64::from(a.top) + 1 == i64::from(left.row)
                                    && i64::from(a.left) + 1 == i64::from(left.column)
                            });
                        if !is_anchor {
                            return no_answer(cell);
                        }
                        pivot
                    }
                    _ => pivot,
                }
            }
            value if value.is_error() => return value,
            _ => None,
        };
        // An error in `field` is the result as is, before any item error or referenced
        // data_field error (T-4823).
        for pair in args[2..].chunks_exact(2) {
            let field = self.evaluate_node_in_context(&pair[0], cell);
            if field.is_error() {
                return field;
            }
        }
        let Some(pivot) = pivot else {
            return no_answer(cell);
        };
        match self.getpivotdata_on(pivot, (&args[0], data_field), &args[2..], cell) {
            Some(result) => result,
            None => no_answer(cell),
        }
    }

    /// The one pivot on the reference's sheet whose area meets the rectangle `left`:`right`
    /// (H11, H16; none is H12, H20); several is SPEC-0015 unknown 9. A pivot's area is its
    /// location and the page fields above it: `PF!A1` and `PF!B1`, the page field's name and
    /// item over a location at A3, pick it (LY Q!A5, A6). The page area is the rows above the
    /// location less one blank row, two columns per page column (`rowPageCount` and
    /// `colPageCount`; several page columns are UNVERIFIED).
    fn pivot_at(&self, left: CellReferenceIndex, right: CellReferenceIndex) -> Option<usize> {
        if left.sheet != right.sheet {
            return None;
        }
        let sheet_id = self.workbook.worksheet(left.sheet).ok()?.sheet_id;
        let (top, bottom) = (left.row.min(right.row), left.row.max(right.row));
        let (first, last) = (left.column.min(right.column), left.column.max(right.column));
        let mut found = self
            .workbook
            .pivot_tables
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                // The model's rows and columns are 1-based, a pivot's location 0-based.
                let area = p.location;
                let pages = p.page_fields.len();
                let (page_rows, page_columns) = if pages == 0 {
                    (0, 0)
                } else {
                    let rows = match p.geometry.row_page_count {
                        0 => u32::try_from(pages).unwrap_or(u32::MAX),
                        n => n,
                    };
                    (rows.saturating_add(1), p.geometry.column_page_count.max(1))
                };
                let area_top = area.top.saturating_sub(page_rows);
                let area_right = area.right.max(
                    area.left
                        .saturating_add(page_columns.saturating_mul(2))
                        .saturating_sub(1),
                );
                p.sheet_id == sheet_id
                    && i64::from(area_top) < i64::from(bottom)
                    && i64::from(top) <= i64::from(area.bottom) + 1
                    && i64::from(area.left) < i64::from(last)
                    && i64::from(first) <= i64::from(area_right) + 1
            })
            .map(|(index, _)| index);
        let pivot = found.next()?;
        found.next().is_none().then_some(pivot)
    }

    /// Whether pivot `pivot` collapsed to `#SPILL!`: the grid writer then saves its location as
    /// its one anchor cell, which holds `#SPILL!` (`pivot::grid`, pivots.md "A pivot that would
    /// meet a foreign cell collapses to #SPILL!").
    fn pivot_collapsed(&mut self, pivot: usize) -> bool {
        let Some(table) = self.workbook.pivot_tables.get(pivot) else {
            return false;
        };
        let area = table.location;
        if area.top != area.bottom || area.left != area.right {
            return false;
        }
        let sheet_id = table.sheet_id;
        let Some(sheet) = self
            .workbook
            .worksheets
            .iter()
            .position(|w| w.sheet_id == sheet_id)
            .and_then(|s| u32::try_from(s).ok())
        else {
            return false;
        };
        let (Ok(row), Ok(column)) = (
            i32::try_from(i64::from(area.top) + 1),
            i32::try_from(i64::from(area.left) + 1),
        ) else {
            return false;
        };
        matches!(
            self.evaluate_cell(CellReferenceIndex { sheet, column, row }),
            CalcResult::Error {
                error: Error::SPILL,
                ..
            }
        )
    }

    /// What GETPIVOTDATA reads of pivot `pivot` (see [`Shape`]); `None` when it answers nothing
    /// from it.
    fn pivot_shape(&mut self, pivot: usize) -> Option<Shape> {
        let (rows, columns) = {
            let index = self.pivot_index(pivot)?;
            (
                index.result.rows().fields().to_vec(),
                index.result.columns().fields().to_vec(),
            )
        };
        let table = self.workbook.pivot_tables.get(pivot)?;
        let cache = self.workbook.pivot_caches.get(table.cache as usize)?;
        let names = cache.fields.iter().map(|f| f.name.to_lowercase()).collect();
        let pages = table
            .page_fields
            .iter()
            .map(|page| (page.field, page_selection(table, page)))
            .collect();
        let subtotals = (0..cache.fields.len())
            .map(|f| {
                table
                    .fields
                    .get(f)
                    .is_none_or(|p| p.default_subtotal && p.custom_subtotals.is_empty())
            })
            .collect();
        let values = values_on_axis(table).then(|| {
            table
                .options
                .data_caption
                .as_deref()
                .unwrap_or("Values")
                .to_lowercase()
        });
        let captions = table
            .data_fields
            .iter()
            .map(|d| d.caption.to_lowercase())
            .collect();
        let sources = table
            .data_fields
            .iter()
            .map(|d| {
                cache
                    .fields
                    .get(d.field as usize)
                    .map(|f| f.name.to_lowercase())
                    .unwrap_or_default()
            })
            .collect();
        Some(Shape {
            rows,
            columns,
            names,
            pages,
            subtotals,
            values,
            captions,
            sources,
        })
    }

    /// Whether a blank `data_field` cell names pivot `pivot`'s only data field: a blank cell
    /// inside the pivot's area does (J7; DF S1!C3, A18); a blank cell outside it is `#REF!`, on
    /// the pivot's sheet or another (DF A15, A17), as is a blank cell with several data fields
    /// (DF A6, A20). An omitted argument is `#REF!` too, UNVERIFIED (SPEC-0015 unknown 8, T-4911),
    /// and so is a blank cell inside another pivot's area or in this pivot's page-field area,
    /// UNVERIFIED (T-4911).
    fn blank_names_data_field(
        &mut self,
        pivot: usize,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> bool {
        let Some(table) = self.workbook.pivot_tables.get(pivot) else {
            return false;
        };
        if table.data_fields.len() != 1 {
            return false;
        }
        let location = table.location;
        let CalcResult::Range { left, right } = self.evaluate_node_with_reference(node, cell)
        else {
            return false;
        };
        let inside = i64::from(location.top) < i64::from(left.row)
            && i64::from(left.row) <= i64::from(location.bottom) + 1
            && i64::from(location.left) < i64::from(left.column)
            && i64::from(left.column) <= i64::from(location.right) + 1;
        left == right && inside && self.pivot_at(left, right) == Some(pivot)
    }

    /// GETPIVOTDATA against pivot `pivot`, `None` for `#REF!`.
    fn getpivotdata_on(
        &mut self,
        pivot: usize,
        (data_node, data_field): (&Node, CalcResult),
        pairs: &[Node],
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        let shape = self.pivot_shape(pivot)?;
        // A field with no item (H15).
        if pairs.len() % 2 == 1 {
            return None;
        }
        let collapsed = self.pivot_collapsed(pivot);
        let has_pairs = !pairs.is_empty();
        let blank_data = matches!(data_field, CalcResult::EmptyCell)
            && self.blank_names_data_field(pivot, data_node, cell);
        let omitted: Vec<bool> = pairs
            .chunks_exact(2)
            .map(|pair| matches!(pair[1], Node::EmptyArgKind))
            .collect();
        // An array data_field or field name lifts; a range there does not, and names nothing
        // (`#REF!`, gen-m6 16b67941f34e0346 A40, A43). A range of items lifts (L1, L5).
        let unlifted = |value: CalcResult| match value {
            CalcResult::Range { .. } => no_answer(cell),
            value => value,
        };
        let mut args = Vec::with_capacity(pairs.len() + 1);
        args.push(unlifted(data_field));
        for pair in pairs.chunks_exact(2) {
            let field = self.evaluate_node_in_context(&pair[0], cell);
            args.push(unlifted(field));
            args.push(self.evaluate_node_in_context(&pair[1], cell));
        }
        Some(self.lift_values(
            args,
            LiftCost::Unit,
            &LiftOptions::EVERY_ELEMENT,
            cell,
            &mut |model, args| match model.pivot_answer(pivot, &shape, args, &omitted, blank_data) {
                // A collapsed pivot (pivots.md "GETPIVOTDATA against a pivot collapsed to
                // #SPILL!", t4130 Q!B1:B4): a pair naming an item is `#SPILL!`, the grand total
                // and an unknown item `#REF!`.
                Some(_) if collapsed && has_pairs => {
                    CalcResult::new_error(Error::SPILL, cell, "Pivot table collapsed".into())
                }
                _ if collapsed => no_answer(cell),
                Some(Ok(value)) => CalcResult::Number(value),
                Some(Err(error)) => {
                    CalcResult::new_error(error, cell, "No pivot table answer".into())
                }
                None => no_answer(cell),
            },
        ))
    }

    /// The data field the scalar `value` names: a caption, then a source field name, ignoring
    /// case, untrimmed (H1, H2, H10, H19; "SUM OF SALES", IT D7). A source name two data fields
    /// share picks the first (DF A1); `"Values"`, `""`, a number or a boolean names none (DF
    /// A10, A14, A16; IT E6:E8). Which wins when one field's caption is another's source name is
    /// unprobed: the caption does here.
    fn pivot_data_field(shape: &Shape, value: &CalcResult, blank_data: bool) -> Option<usize> {
        match value {
            CalcResult::EmptyCell => blank_data.then_some(0),
            CalcResult::String(name) => {
                let name = name.to_lowercase();
                shape
                    .captions
                    .iter()
                    .position(|c| *c == name)
                    .or_else(|| shape.sources.iter().position(|s| *s == name))
            }
            _ => None,
        }
    }

    /// One element's answer: `args` is the data field then each pair's field and item, all
    /// scalars. `None` is `#REF!`; `Err` an error the shown cell holds.
    fn pivot_answer(
        &mut self,
        pivot: usize,
        shape: &Shape,
        args: &[CalcResult],
        omitted: &[bool],
        blank_data: bool,
    ) -> Option<Result<f64, Error>> {
        let (data, pairs) = args.split_first()?;
        let data = Self::pivot_data_field(shape, data, blank_data)?;
        let mut rows: Vec<Option<u32>> = vec![None; shape.rows.len()];
        let mut columns: Vec<Option<u32>> = vec![None; shape.columns.len()];
        for (pair, &omitted) in pairs.chunks_exact(2).zip(omitted) {
            // A field name is text naming a field of the pivot (H8, J3).
            let CalcResult::String(name) = &pair[0] else {
                return None;
            };
            let name = name.to_lowercase();
            let place = if shape.values.as_deref() == Some(name.as_str()) {
                Place::Values
            } else {
                let field = u32::try_from(shape.names.iter().position(|n| *n == name)?).ok()?;
                if let Some(depth) = shape.rows.iter().position(|&f| f == field) {
                    Place::Row(depth)
                } else if let Some(depth) = shape.columns.iter().position(|&f| f == field) {
                    Place::Column(depth)
                } else {
                    let &(_, selected) = shape.pages.iter().find(|(f, _)| *f == field)?;
                    Place::Page(field, selected)
                }
            };
            let item = &pair[1];
            // The Values field's one item is the chosen data field's caption (DF A8; another data
            // field's is `#REF!`, A9, A11).
            if place == Place::Values {
                match item {
                    CalcResult::String(caption)
                        if shape.captions.get(data) == Some(&caption.to_lowercase()) => {}
                    _ => return None,
                }
                continue;
            }
            let field = match place {
                Place::Row(depth) => *shape.rows.get(depth)?,
                Place::Column(depth) => *shape.columns.get(depth)?,
                Place::Page(field, _) => field,
                Place::Values => return None,
            };
            // An item is text, a label cell's value included (J6), matched by its label; an
            // omitted argument or `"(blank)"` for the blank item (pv23_041 G5:G7,
            // probes/t4236-pivot-date-groups); a number or a boolean by value (IT A1:C5). An error
            // is `#REF!` (IT E4, E5); `""` or a blank cell unprobed.
            let found = match item {
                _ if omitted => self.pivot_blank_item(pivot, field),
                CalcResult::String(text) => self.pivot_item(pivot, field, text),
                CalcResult::Number(value) => self.pivot_number_item(pivot, field, *value),
                CalcResult::Boolean(value) => self.pivot_boolean_item(pivot, field, *value),
                _ => None,
            }?;
            let slot = match place {
                Place::Row(depth) => rows.get_mut(depth)?,
                Place::Column(depth) => columns.get_mut(depth)?,
                Place::Page(_, selected) => {
                    // A page field's pair names its selected item, which the totals already
                    // keep; another item, or any item under `(All)`, is not shown (LY A3, A4, A8).
                    if selected != Some(found) {
                        return None;
                    }
                    continue;
                }
                Place::Values => return None,
            };
            // The same field twice with the same item is that item (IT E1, E2); with another
            // item, unprobed, `#REF!`.
            if slot.is_some_and(|item| item != found) {
                return None;
            }
            *slot = Some(found);
        }
        let index = self.pivot_index(pivot)?;
        let result = &index.result;
        let row = Self::shown_node(result.rows(), &rows, &shape.rows, &shape.subtotals)?;
        let column =
            Self::shown_node(result.columns(), &columns, &shape.columns, &shape.subtotals)?;
        // An intersection no record reaches shows blank, and GETPIVOTDATA answers 0 there (DF
        // B6:J6; probes/getpivotdata-empty-intersection).
        Some(result.value(row, column, data).unwrap_or(Ok(0.0)))
    }

    /// The node of `tree` the items `items` (per axis depth) name, when the pivot shows a line
    /// for it: the items must be the outer fields' (the inner field alone is `#REF!`, LY B4:D4),
    /// the tuple must be one the pivot shows (an inner item with no records under its outer
    /// item is `#REF!`, LY B6:D6; so is a hidden item, A10), and a tuple of some but not all axis
    /// fields is a subtotal, shown only while its innermost field totals its items (LY E1:E6).
    fn shown_node(
        tree: &crate::pivot::AxisTree,
        items: &[Option<u32>],
        fields: &[u32],
        subtotals: &[bool],
    ) -> Option<u32> {
        let given = items.iter().take_while(|item| item.is_some()).count();
        if items[given..].iter().any(Option::is_some) {
            return None;
        }
        let path: Vec<u32> = items[..given].iter().filter_map(|&item| item).collect();
        let node = tree.find(&path)?;
        if given > 0 && given < fields.len() {
            let field = *fields.get(given - 1)?;
            if !subtotals.get(field as usize).copied().unwrap_or(true) {
                return None;
            }
        }
        Some(node)
    }
}
