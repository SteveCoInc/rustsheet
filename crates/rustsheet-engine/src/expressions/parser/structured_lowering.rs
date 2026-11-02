//! Lowering of a parsed structured reference to the area it resolves to against the parser's
//! table list. Moved out of operands.rs unchanged (T-3466).

use super::super::token::{self, TableReference};
use super::super::utils::number_to_column;
use super::structured_reference::{COLUMN_NOT_FOUND, TABLE_NOT_FOUND, TABLE_SHEET_NOT_FOUND};
use super::*;

impl Parser {
    /// The area a structured reference resolves to against the parser's table list: a
    /// `ReferenceKind`, a `RangeKind`, a `ThisRowKind`, an `ErrorKind` or a `ParseErrorKind`.
    pub(super) fn lower_structured_reference(
        &mut self,
        table_name: String,
        specifier: Option<token::TableSpecifier>,
        table_reference: Option<TableReference>,
    ) -> Node {
        // We will try to convert to a normal reference
        // table_name[column_name] => cell1:cell2
        // table_name[[#This Row], [column_name]:[column_name]] => the columns on the cell's row
        let context = &self.context;
        let context_sheet_index = match self.get_sheet_index_by_name(&context.sheet) {
            Some(i) => i,
            None => {
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    expecting: vec![ExpectedTokens::Other],
                    position: 0,
                    message: format!("sheet not found: {}", context.sheet),
                };
            }
        };
        // table-name => table
        let table = match self.table_named(&table_name) {
            Some(t) => t,
            None => {
                let message = format!(
                    "{TABLE_NOT_FOUND}'{table_name}' at '{}!{}{}'",
                    context.sheet,
                    number_to_column(context.column).unwrap_or(format!("{}", context.column)),
                    context.row
                );
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    expecting: vec![ExpectedTokens::Other],
                    position: 0,
                    message,
                };
            }
        };
        let table_sheet_index = match self.get_sheet_index_by_name(&table.sheet_name) {
            Some(i) => i,
            None => {
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    expecting: vec![ExpectedTokens::Other],
                    position: 0,
                    message: format!("{TABLE_SHEET_NOT_FOUND}{}", table.sheet_name),
                };
            }
        };

        let sheet_name = if table_sheet_index == context_sheet_index {
            None
        } else {
            Some(table.sheet_name.clone())
        };

        // The table metadata comes from the workbook (untrusted): a stored reference that does not
        // parse is #REF! (directive 5: no panics; docs/excel-semantics/references.md, DEVIATION).
        let Ok((column_start, mut row_start, column_end, mut row_end)) =
            parse_range(&table.reference)
        else {
            return Node::ErrorKind(token::Error::REF);
        };

        // The counts are workbook values too: arithmetic that does not fit is #REF!, not a wrap.
        let (Ok(totals_row_count), Ok(header_row_count)) = (
            i32::try_from(table.totals_row_count),
            i32::try_from(table.header_row_count),
        ) else {
            return Node::ErrorKind(token::Error::REF);
        };
        let (Some(data_end), Some(data_start)) = (
            row_end.checked_sub(totals_row_count),
            row_start.checked_add(header_row_count),
        ) else {
            return Node::ErrorKind(token::Error::REF);
        };
        let Some(totals_row) = data_end.checked_add(1) else {
            return Node::ErrorKind(token::Error::REF);
        };
        row_end = data_end;

        let this_row = specifier == Some(token::TableSpecifier::ThisRow);
        match specifier {
            // Decided at the evaluating cell's row (`Node::ThisRowKind`), below.
            Some(token::TableSpecifier::ThisRow) => {}
            Some(token::TableSpecifier::Totals) if totals_row_count != 0 => {
                row_start = totals_row;
                row_end = row_start;
            }
            // [#Totals] is #REF! without a totals row, [#Headers] without a header row (T-3260).
            Some(token::TableSpecifier::Totals) => return Node::ErrorKind(token::Error::REF),
            Some(token::TableSpecifier::Headers) if header_row_count == 0 => {
                return Node::ErrorKind(token::Error::REF);
            }
            Some(token::TableSpecifier::Headers) => row_end = row_start,
            Some(token::TableSpecifier::Data) => {
                row_start = data_start;
            }
            Some(token::TableSpecifier::All) => {
                if totals_row_count != 0 {
                    row_end = totals_row;
                }
            }
            // [#Headers],[#Data]: header row through the last data row (row_end already excludes totals).
            Some(token::TableSpecifier::HeadersData) => {}
            // [#Data],[#Totals]: data rows plus the totals row when there is one.
            Some(token::TableSpecifier::DataTotals) => {
                row_start = data_start;
                if totals_row_count != 0 {
                    row_end = totals_row;
                }
            }
            None => {
                // skip the headers
                row_start = data_start;
            }
        }
        let one_column = matches!(table_reference, Some(TableReference::ColumnReference(_)));
        let lowered = match table_reference {
            None => Node::RangeKind {
                sheet_name,
                sheet_index: table_sheet_index,
                absolute_row1: true,
                absolute_column1: true,
                row1: row_start,
                column1: column_start,
                absolute_row2: true,
                absolute_column2: true,
                row2: row_end,
                column2: column_end,
            },
            Some(TableReference::ColumnReference(s)) => {
                let column_index = match get_table_column_by_name(&s, table) {
                    Some(s) => s + column_start,
                    None => {
                        return Node::ParseErrorKind {
                            formula: self.lexer.get_formula(),
                            expecting: vec![ExpectedTokens::Other],
                            position: self.lexer.get_position() as usize,
                            message: format!("{COLUMN_NOT_FOUND}{s} in table {table_name}"),
                        };
                    }
                };
                if row_start == row_end && !this_row {
                    return Node::ReferenceKind {
                        sheet_name,
                        sheet_index: table_sheet_index,
                        absolute_row: true,
                        absolute_column: true,
                        row: row_start,
                        column: column_index,
                    };
                }
                Node::RangeKind {
                    sheet_name,
                    sheet_index: table_sheet_index,
                    absolute_row1: true,
                    absolute_column1: true,
                    row1: row_start,
                    column1: column_index,
                    absolute_row2: true,
                    absolute_column2: true,
                    row2: row_end,
                    column2: column_index,
                }
            }
            Some(TableReference::RangeReference((left, right))) => {
                let left_column_index = match get_table_column_by_name(&left, table) {
                    Some(f) => f + column_start,
                    None => {
                        return Node::ParseErrorKind {
                            formula: self.lexer.get_formula(),
                            expecting: vec![ExpectedTokens::Other],
                            position: self.lexer.get_position() as usize,
                            message: format!("{COLUMN_NOT_FOUND}{left} in table {table_name}"),
                        };
                    }
                };

                let right_column_index = match get_table_column_by_name(&right, table) {
                    Some(f) => f + column_start,
                    None => {
                        return Node::ParseErrorKind {
                            formula: self.lexer.get_formula(),
                            expecting: vec![ExpectedTokens::Other],
                            position: self.lexer.get_position() as usize,
                            message: format!("{COLUMN_NOT_FOUND}{right} in table {table_name}"),
                        };
                    }
                };
                // The same area whichever column is written first (references.md, column-range
                // 44cecbb1ecd5b2da K143).
                Node::RangeKind {
                    sheet_name,
                    sheet_index: table_sheet_index,
                    absolute_row1: true,
                    absolute_column1: true,
                    row1: row_start,
                    column1: left_column_index.min(right_column_index),
                    absolute_row2: true,
                    absolute_column2: true,
                    row2: row_end,
                    column2: left_column_index.max(right_column_index),
                }
            }
        };
        if !this_row {
            return lowered;
        }
        // Off the data rows (header, totals, above, below) it is #VALUE!, at the evaluating
        // cell's row (docs/excel-semantics/references.md, gen-m8b this-row e1bf5aa0ffb4ed87).
        match lowered {
            Node::RangeKind {
                sheet_name,
                sheet_index,
                column1,
                column2,
                ..
            } => Node::ThisRowKind {
                sheet_name,
                sheet_index,
                column1,
                column2,
                data_start,
                data_end,
                cell: one_column,
            },
            other => other,
        }
    }
}
