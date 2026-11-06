//! Workbook tables (`ListObject`s) added through the API; imported tables come from the xlsx
//! reader. Tables matter to evaluation through structured references and through filters
//! (SUBTOTAL skips filter-hidden rows, functions/subtotal.rs).

use crate::{
    expressions::{
        lexer::LexerMode,
        parser::{
            parse_range,
            stringify::{
                to_rc_lowering_at, to_string_lowering, to_string_with_ref_errors, OffRows,
            },
            Node,
        },
        types::{Area, CellReferenceRC},
        utils::{is_valid_identifier, number_to_column},
    },
    formula_rewrite::RewrittenFormula,
    model::Model,
    table_edits::{bounds, Bounds},
    types::{Cell, Table, TableColumn, TableExtent, TableStyleInfo, Worksheet},
};

/// Gives every formula cell of `worksheet` whose shared formula is one of `by_row` (index, parsed
/// formula holding a `#This Row` reference to one of `tables`) the text that formula lowers to on
/// the cell's own row, a shared formula of its own: a duplicated sheet, which gets no table, then
/// reads what each cell read before, `#VALUE!` off the data rows (T-1699). Rows whose text is the
/// same share one formula. Cells on the rows `keep_rows` (inclusive) keep their shared formula.
/// A reference off the data rows prints as `off_rows` says: detached for a duplicate, `#REF!`
/// for a convert to range (T-4020).
pub(crate) fn lower_this_row_cells(
    worksheet: &mut Worksheet,
    by_row: &[(usize, Node)],
    tables: &[String],
    keep_rows: Option<(i32, i32)>,
    off_rows: OffRows,
) {
    if by_row.is_empty() {
        return;
    }
    let cells: Vec<(i32, i32, usize)> = worksheet
        .sheet_data
        .cells()
        .filter(|(row, _, _)| keep_rows.is_none_or(|(top, bottom)| *row < top || *row > bottom))
        .filter_map(|(row, column, cell)| {
            let index = usize::try_from(cell.get_formula()?).ok()?;
            Some((row, column, index))
        })
        .collect();
    let mut known: std::collections::HashMap<String, i32> = worksheet
        .shared_formulas
        .iter()
        .enumerate()
        .filter_map(|(index, text)| Some((text.clone(), i32::try_from(index).ok()?)))
        .collect();
    for (row, column, index) in cells {
        let Some((_, node)) = by_row.iter().find(|(i, _)| *i == index) else {
            continue;
        };
        let text = to_rc_lowering_at(node, row, tables, off_rows);
        let Ok(next) = i32::try_from(worksheet.shared_formulas.len()) else {
            return;
        };
        let new_index = *known.entry(text.clone()).or_insert_with(|| {
            worksheet.shared_formulas.push(text);
            next
        });
        if let Some(Cell::CellFormula { f, .. } | Cell::ArrayFormula { f, .. }) =
            worksheet.cell_mut(row, column)
        {
            *f = new_index;
        }
    }
}

impl Table {
    /// A table named `name` over `reference` (`A1:C4`, header and totals rows included) on
    /// `sheet_name`, with the columns `column_names` left to right, no differential formats and
    /// no filter buttons, styled as the xlsx reader defaults a table without `tableStyleInfo`.
    #[must_use]
    pub fn new(
        name: &str,
        sheet_name: &str,
        reference: &str,
        (header_row_count, totals_row_count): (u32, u32),
        column_names: impl IntoIterator<Item = String>,
    ) -> Table {
        Table {
            name: name.to_string(),
            display_name: name.to_string(),
            sheet_name: sheet_name.to_string(),
            reference: reference.to_string(),
            totals_row_count,
            header_row_count,
            header_row_dxf_id: None,
            data_dxf_id: None,
            totals_row_dxf_id: None,
            columns: (1..)
                .zip(column_names)
                .map(|(id, name)| TableColumn {
                    id,
                    name,
                    ..TableColumn::default()
                })
                .collect(),
            style_info: TableStyleInfo {
                name: None,
                show_first_column: false,
                show_last_column: false,
                show_row_stripes: true,
                show_column_stripes: false,
            },
            has_filters: false,
            filter_columns: Vec::new(),
        }
    }
}

impl Model {
    /// Adds a table on sheet `sheet` over rows `top..=bottom` and columns `left..=right`
    /// (1-based). The first row is the header row: its cells name the columns, so they must be
    /// non-empty and distinct (ignoring case). `has_filters` marks the table's autoFilter as
    /// filtering, which is how an imported table with a filter looks; it hides no row by itself.
    /// Reparses and re-evaluates the workbook so structured references can name the table.
    ///
    /// # Errors
    /// An invalid or taken name (tables and defined names share one namespace), an invalid
    /// sheet, a range without a data row or outside the sheet, a range that overlaps another
    /// table, or an empty or duplicate header cell.
    pub fn add_table(
        &mut self,
        sheet: u32,
        name: &str,
        range: (i32, i32, i32, i32),
        has_filters: bool,
    ) -> Result<(), String> {
        self.add_table_with_totals(sheet, name, range, has_filters, false)
    }

    /// [`Model::add_table`] for a range whose last row is the table's totals row when
    /// `totals_row` is set (`Table[#Totals]`, `Table[#All]`; the cells of that row keep what they
    /// hold). Such a table needs a header row, a data row and the totals row.
    ///
    /// # Errors
    /// As [`Model::add_table`], with the range one row taller for a totals row.
    pub fn add_table_with_totals(
        &mut self,
        sheet: u32,
        name: &str,
        range: (i32, i32, i32, i32),
        has_filters: bool,
        totals_row: bool,
    ) -> Result<(), String> {
        self.add_table_with_rows(sheet, name, range, has_filters, (true, totals_row))
    }

    /// [`Model::add_table_with_totals`] for a table with or without a header row
    /// (`(header_row, totals_row)`). A table without one starts its data on the first row of the
    /// range and names its columns `Column1`, `Column2`, ... as the spec builder writes them
    /// (probes/table-edits table-no-header-baseline, T-3260); it needs one data row and takes no
    /// filter.
    ///
    /// # Errors
    /// As [`Model::add_table`], with the range one row shorter without a header row.
    pub fn add_table_with_rows(
        &mut self,
        sheet: u32,
        name: &str,
        (top, left, bottom, right): (i32, i32, i32, i32),
        has_filters: bool,
        (header_row, totals_row): (bool, bool),
    ) -> Result<(), String> {
        if !is_valid_identifier(name) {
            return Err(format!("Invalid table name '{name}'"));
        }
        let upper = name.to_uppercase();
        if self
            .workbook
            .tables
            .keys()
            .any(|t| t.to_uppercase() == upper)
            || self
                .workbook
                .defined_names
                .iter()
                .any(|d| d.name.to_uppercase() == upper)
        {
            return Err(format!("The name '{name}' is already taken"));
        }
        let sheet_name = self.workbook.worksheet(sheet)?.get_name();
        let (Some(first), Some(last)) = (number_to_column(left), number_to_column(right)) else {
            return Err("Table columns are outside the sheet".to_string());
        };
        let totals_rows = i32::from(totals_row);
        if top < 1
            || top
                .saturating_add(totals_rows)
                .saturating_add(i32::from(header_row))
                > bottom
            || left > right
            || bottom > crate::constants::LAST_ROW
        {
            return Err("A table needs a header row and at least one data row".to_string());
        }
        let new_bounds = Bounds {
            left,
            top,
            right,
            bottom,
            data: (top, bottom),
        };
        let overlaps = self.workbook.tables.values().any(|t| {
            t.sheet_name == sheet_name && bounds(t).is_some_and(|b| b.overlaps(&new_bounds))
        });
        if overlaps {
            return Err("A table cannot overlap another table".to_string());
        }
        if !header_row && has_filters {
            return Err("A table without a header row has no filter".to_string());
        }
        let mut columns: Vec<String> = Vec::new();
        for column in left..=right {
            if !header_row {
                columns.push(format!("Column{}", column - left + 1));
                continue;
            }
            let header = self.get_formatted_cell_value(sheet, top, column)?;
            if header.is_empty()
                || columns
                    .iter()
                    .any(|c| c.to_uppercase() == header.to_uppercase())
            {
                return Err(format!("Table header '{header}' is empty or repeated"));
            }
            columns.push(header);
        }
        let table = Table {
            has_filters,
            ..Table::new(
                name,
                &sheet_name,
                &format!("{first}{top}:{last}{bottom}"),
                (u32::from(header_row), u32::from(totals_row)),
                columns,
            )
        };
        self.workbook.tables.insert(name.to_string(), table);
        self.reset_tables();
        Ok(())
    }

    /// Removes the table `name` (the undo of [`Model::add_table`]); its cells stay.
    ///
    /// # Errors
    /// There is no table with that exact name.
    pub fn remove_table(&mut self, name: &str) -> Result<(), String> {
        self.workbook
            .tables
            .remove(name)
            .ok_or_else(|| format!("No table named '{name}'"))?;
        self.reset_tables();
        Ok(())
    }

    /// Deletes the table `name` as Graph's `DELETE tables('T')` does: the table and every cell of
    /// its range go (T-2324: `SUM(S!C4:C7)` is 0 afterwards; the rows stay in place, a plain
    /// reference below the table does not shift). Does not recalculate.
    ///
    /// # Errors
    /// There is no table with that exact name, or its range cannot be cleared (it holds part of
    /// an array formula).
    pub fn delete_table(&mut self, name: &str) -> Result<(), String> {
        let table = self
            .workbook
            .tables
            .get(name)
            .ok_or_else(|| format!("No table named '{name}'"))?;
        let sheet = self
            .get_sheet_index_by_name(&table.sheet_name)
            .ok_or_else(|| format!("No sheet named '{}'", table.sheet_name))?;
        let (left, top, right, bottom) = parse_range(&table.reference)
            .map_err(|_| format!("Table '{name}' has an invalid range"))?;
        // Clamped to the used area: the range comes from the file and can span the whole sheet,
        // and the cells past the used area are empty, so clearing them changes nothing.
        let (last_row, last_column) = self.workbook.worksheet(sheet)?.used_area_end();
        let (bottom, right) = (bottom.min(last_row), right.min(last_column));
        if top <= bottom && left <= right {
            self.range_clear_all(&Area {
                sheet,
                row: top,
                column: left,
                width: right - left + 1,
                height: bottom - top + 1,
            })?;
        }
        // Excel rewrites the references to a deleted table to `#REF!` (FORMULATEXT
        // `=SUM(#REF!)`, `ROWS(Sales)` is `#REF!`; T-2340, probes/table-edits/table-delete-table);
        // `remove_table` alone would leave them as parse errors.
        self.rewrite_table_references(&[name.to_lowercase()], None, TableRewrite::RefError);
        self.remove_table(name)
    }

    /// Converts the table `name` to a plain range as Graph's `convertToRange` does: the cells
    /// stay and every structured reference to it, in any sheet's formulas and in defined names,
    /// becomes the absolute range it resolved to (T-2342: `=SUM(Sales[Qty])` is `=SUM(S!$C$4:$C$7)`
    /// in FORMULATEXT, `ROWS(Sales[#All])` stays 5). Does not recalculate.
    ///
    /// # Errors
    /// There is no table with that exact name.
    pub fn convert_table_to_range(&mut self, name: &str) -> Result<(), String> {
        if !self.workbook.tables.contains_key(name) {
            return Err(format!("No table named '{name}'"));
        }
        let tables = [name.to_lowercase()];
        let data_rows = self
            .workbook
            .tables
            .get(name)
            .and_then(|table| Some(bounds(table)?.data));
        self.lower_this_row_references(&tables, data_rows);
        self.rewrite_table_references(&tables, None, TableRewrite::Lower);
        self.remove_table(name)
    }

    /// Hands the table list to the parser and reparses every formula against it.
    pub(crate) fn reset_tables(&mut self) {
        self.parser.set_tables(self.workbook.tables.clone());
        self.reset_parsed_structures();
    }

    /// Points the tables of the sheet named `old_name` at `new_name` and hands the list to the
    /// parser, so the reparse after a sheet rename still finds them (SPEC-0016 "Sheet rename,
    /// delete and restore").
    pub(crate) fn rename_tables_sheet(&mut self, old_name: &str, new_name: &str) {
        for table in self.workbook.tables.values_mut() {
            if table.is_on_sheet(old_name) {
                new_name.clone_into(&mut table.sheet_name);
            }
        }
        self.parser.set_tables(self.workbook.tables.clone());
    }

    /// The tables on the sheet `sheet_name`, by name.
    pub(crate) fn tables_on_sheet(&self, sheet_name: &str) -> Vec<(String, Table)> {
        self.workbook
            .tables
            .iter()
            .filter(|(_, table)| table.is_on_sheet(sheet_name))
            .map(|(name, table)| (name.clone(), table.clone()))
            .collect()
    }

    /// The extents of the tables on `sheet`, to compare before and after a row delete (T-3634).
    pub(crate) fn table_extents(&self, sheet: u32) -> Result<Vec<TableExtent>, String> {
        let sheet_name = self.workbook.worksheet(sheet)?.get_name();
        Ok((self.tables_on_sheet(&sheet_name).into_iter())
            .map(|(name, table)| TableExtent {
                name,
                reference: table.reference,
                totals_row_count: table.totals_row_count,
            })
            .collect())
    }

    /// Those of `before` (the [`Model::table_extents`] of `sheet` before an edit) that the edit
    /// changed: what the edit's undo restores.
    pub(crate) fn changed_table_extents(
        &self,
        sheet: u32,
        before: Vec<TableExtent>,
    ) -> Result<Vec<TableExtent>, String> {
        let after = self.table_extents(sheet)?;
        Ok(before.into_iter().filter(|e| !after.contains(e)).collect())
    }

    /// Gives the tables of `sheet` named in `extents` those extents back (the undo of a row
    /// delete that resized them, T-3634), and reparses. The extents can come from a peer's diff
    /// (directive 5): each must name a table on `sheet` and keep its columns, with room for its
    /// header and totals rows, or nothing changes.
    pub(crate) fn restore_table_extents(
        &mut self,
        sheet: u32,
        extents: &[TableExtent],
    ) -> Result<(), String> {
        self.check_table_extents(sheet, extents)?;
        if extents.is_empty() {
            return Ok(());
        }
        for extent in extents {
            if let Some(table) = self.workbook.tables.get_mut(&extent.name) {
                table.reference.clone_from(&extent.reference);
                table.totals_row_count = extent.totals_row_count;
            }
        }
        self.reset_tables();
        Ok(())
    }

    /// Checks that [`Model::restore_table_extents`] would accept `extents`, changing nothing, so
    /// an undo can refuse a bad peer diff before it moves any row (T-4671).
    pub(crate) fn check_table_extents(
        &self,
        sheet: u32,
        extents: &[TableExtent],
    ) -> Result<(), String> {
        if extents.is_empty() {
            return Ok(());
        }
        let sheet_name = self.workbook.worksheet(sheet)?.get_name();
        for extent in extents {
            let table = (self.workbook.tables.get(&extent.name))
                .filter(|table| table.is_on_sheet(&sheet_name))
                .ok_or_else(|| format!("No table '{}' on the sheet", extent.name))?;
            let restored = Table {
                reference: extent.reference.clone(),
                totals_row_count: extent.totals_row_count,
                ..table.clone()
            };
            let fits = bounds(table)
                .zip(bounds(&restored))
                .is_some_and(|(now, then)| {
                    then.left == now.left
                        && then.right == now.right
                        && then.top <= then.bottom
                        && then.data.0 <= then.data.1.saturating_add(1)
                        && extent.totals_row_count <= 1
                });
            if !fits {
                return Err(format!("Invalid extent for table '{}'", extent.name));
            }
        }
        Ok(())
    }

    /// Deletes `count` columns of `sheet` from `column` ([`Model::delete_columns`]) and returns
    /// the tables it changed but kept, as they were before: what its undo restores (T-4652). A
    /// column delete can drop table columns, with their names, formulas and filters, so the
    /// whole table is kept, not only its [`TableExtent`].
    pub(crate) fn delete_columns_keeping_tables(
        &mut self,
        sheet: u32,
        column: i32,
        count: i32,
    ) -> Result<Vec<(String, Table)>, String> {
        let sheet_name = self.workbook.worksheet(sheet)?.get_name();
        let before = self.tables_on_sheet(&sheet_name);
        self.delete_columns(sheet, column, count)?;
        let tables = &self.workbook.tables;
        Ok((before.into_iter())
            .filter(|(name, table)| tables.get(name).is_some_and(|now| now != table))
            .collect())
    }

    /// Checks everything about `tables` that [`Model::restore_tables_of_sheet`] needs except the
    /// overlap test, changing nothing, so a column-delete undo can refuse a bad peer diff before
    /// it re-inserts any column (T-4671). The overlap test stays after the insert: the other
    /// tables' extents only take their final place once the columns are back.
    pub(crate) fn check_tables_of_sheet(
        &self,
        sheet: u32,
        tables: &[(String, Table)],
    ) -> Result<(), String> {
        let sheet_name = self.workbook.worksheet(sheet)?.get_name();
        for (name, then) in tables {
            let now = (self.workbook.tables.get(name))
                .filter(|now| now.is_on_sheet(&sheet_name))
                .ok_or_else(|| format!("No table '{name}' on the sheet"))?;
            if !table_fits_restore(now, then) {
                return Err(format!("Invalid table '{name}' to restore"));
            }
        }
        Ok(())
    }

    /// Gives the tables of `sheet` named in `tables` those tables back (the undo of a column
    /// delete that changed them, T-4652), and reparses. The tables can come from a peer's diff
    /// (directive 5): each must replace a table on `sheet` with the same names and rows, have one
    /// column per column of its reference with distinct non-empty names, filter only those, and
    /// overlap no other table, or nothing changes.
    pub(crate) fn restore_tables_of_sheet(
        &mut self,
        sheet: u32,
        tables: &[(String, Table)],
    ) -> Result<(), String> {
        if tables.is_empty() {
            return Ok(());
        }
        self.check_tables_of_sheet(sheet, tables)?;
        let sheet_name = self.workbook.worksheet(sheet)?.get_name();
        let placed: Vec<_> = (tables.iter())
            .filter_map(|(name, then)| Some((name, bounds(then)?)))
            .collect();
        let others: Vec<_> = (self.workbook.tables.iter())
            .filter(|(name, table)| {
                table.is_on_sheet(&sheet_name) && !tables.iter().any(|(n, _)| n == *name)
            })
            .filter_map(|(name, table)| Some((name, bounds(table)?)))
            .collect();
        for (i, (name, b)) in placed.iter().enumerate() {
            let mut rest = placed.iter().skip(i + 1).chain(&others);
            if rest.any(|(_, other)| b.overlaps(other)) {
                return Err(format!("Table '{name}' would overlap another table"));
            }
        }
        for (name, table) in tables {
            self.workbook.tables.insert(name.clone(), table.clone());
        }
        self.reset_tables();
        Ok(())
    }

    /// Puts back tables taken out with their sheet (the undo of a sheet delete), and reparses.
    /// A table added since under the same name keeps its place.
    pub(crate) fn restore_tables(&mut self, tables: Vec<(String, Table)>) {
        for (name, table) in tables {
            self.workbook.tables.entry(name).or_insert(table);
        }
        self.reset_tables();
    }

    /// The lowercase names of the tables on the sheet `sheet_name`, for the delete rewrite.
    pub(crate) fn table_names_on_sheet(&self, sheet_name: &str) -> Vec<String> {
        self.workbook
            .tables
            .iter()
            .filter(|(_, table)| table.is_on_sheet(sheet_name))
            .map(|(name, _)| name.to_lowercase())
            .collect()
    }

    /// Takes out the tables of the sheet at `sheet_index`, before the sheet is deleted, with the
    /// formulas that referred to them. Their structured references in the other sheets' formulas
    /// and in defined names become `#REF!`, as in Excel (`=SUM(#REF!)`, T-2340,
    /// probes/table-edits/table-delete-sheet); a sheet added later under the old name brings
    /// none of them back, and [`Model::restore_table_references`] puts the text back.
    pub(crate) fn take_tables_of_sheet(
        &mut self,
        sheet_index: u32,
    ) -> (Vec<(String, Table)>, Vec<RewrittenFormula>) {
        let Ok(sheet_name) = self.workbook.worksheet(sheet_index).map(|w| w.get_name()) else {
            return (Vec::new(), Vec::new());
        };
        let lowercase = self.table_names_on_sheet(&sheet_name);
        if lowercase.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let rewritten = self.rewrite_table_references(
            &lowercase,
            Some(sheet_index as usize),
            TableRewrite::RefError,
        );
        let (taken, kept) = std::mem::take(&mut self.workbook.tables)
            .into_iter()
            .partition(|(_, table)| table.is_on_sheet(&sheet_name));
        self.workbook.tables = kept;
        (taken.into_iter().collect(), rewritten)
    }

    /// Gives every cell off the table's `data_rows` whose formula holds a `#This Row` reference
    /// to one of `tables` the text that reference lowers to there (`lower_this_row_cells`), so it
    /// stays `#VALUE!` after the convert as in Excel (S!F1, S!F9 and T!A1 of
    /// probes/table-edits/table-convert-names-this-row, T-2643): a shared formula has no row.
    /// Cells on the data rows keep the shared formula, which lowers to the relative form Excel
    /// prints (`=$C4*2`, table-convert-same-sheet).
    fn lower_this_row_references(&mut self, tables: &[String], data_rows: Option<(i32, i32)>) {
        self.parser.set_lexer_mode(LexerMode::R1C1);
        for worksheet in &mut self.workbook.worksheets {
            let context = CellReferenceRC {
                sheet: worksheet.get_name(),
                row: 1,
                column: 1,
            };
            let by_row: Vec<(usize, Node)> = worksheet
                .shared_formulas
                .iter()
                .enumerate()
                .filter_map(|(index, formula)| {
                    let node = self.parser.parse(formula, &context);
                    node.has_this_row_to(tables).then_some((index, node))
                })
                .collect();
            lower_this_row_cells(worksheet, &by_row, tables, data_rows, OffRows::RefError);
        }
        self.parser.set_lexer_mode(LexerMode::A1);
    }

    /// Prints `#REF!` for every structured reference to one of `tables` (lowercase names) in the
    /// formulas of every sheet but `skip_sheet` and in the defined names, and returns what it
    /// changed. Does not reparse.
    fn rewrite_table_references(
        &mut self,
        tables: &[String],
        skip_sheet: Option<usize>,
        rewrite: TableRewrite,
    ) -> Vec<RewrittenFormula> {
        self.rewrite_formulas(skip_sheet, |node, context| match rewrite {
            TableRewrite::RefError => to_string_with_ref_errors(&node, context, tables),
            TableRewrite::Lower => to_string_lowering(&node, context, tables),
        })
    }
}

/// How [`Model::rewrite_table_references`] prints a structured reference to a removed table.
#[derive(Clone, Copy)]
enum TableRewrite {
    /// `#REF!`: the table is deleted.
    RefError,
    /// The absolute area it resolved to: the table is converted to a range.
    Lower,
}

/// Whether `then` can replace `now` in a restore: same names and rows, one column per column of
/// its reference with distinct non-empty names, and filters only on those columns.
fn table_fits_restore(now: &Table, then: &Table) -> bool {
    bounds(now).zip(bounds(then)).is_some_and(|(now_b, b)| {
        let width = usize::try_from(i64::from(b.right) - i64::from(b.left) + 1);
        let mut names: Vec<String> = then.columns.iter().map(|c| c.name.to_lowercase()).collect();
        names.sort_unstable();
        names.dedup();
        then.name == now.name
            && then.display_name == now.display_name
            && then.sheet_name == now.sheet_name
            && then.header_row_count == now.header_row_count
            && then.totals_row_count <= 1
            && (b.top, b.bottom) == (now_b.top, now_b.bottom)
            && b.data.0 <= b.data.1.saturating_add(1)
            && width.is_ok_and(|w| w == then.columns.len() && w == names.len())
            && names.iter().all(|n| !n.is_empty())
            && (then.filter_columns.iter())
                .all(|f| usize::try_from(f.col_id).is_ok_and(|id| id < names.len()))
    })
}
