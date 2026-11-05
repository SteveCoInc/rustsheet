//! INDIRECT's fallbacks for text that is not an A1 or R1C1 reference: a defined name that holds a
//! reference, and a structured reference (docs/excel-semantics/lookup.md).

use crate::calc_result::CalcResult;
use crate::defined_names::ParsedDefinedName;
use crate::expressions::parser::Node;
use crate::expressions::types::{CellReferenceIndex, CellReferenceRC};
use crate::expressions::utils::{
    is_indirect_unquoted_sheet, is_out_of_range_r1c1_name, unquote_sheet_prefix,
};
use crate::model::Model;

impl Model {
    /// INDIRECT of the text of a structured reference (`INDIRECT("Sales[Qty]")`): the area the
    /// parser lowers the reference to, as for the same text in a formula. A table or column the
    /// workbook lacks stays `#REF!` (docs/excel-semantics/lookup.md "INDIRECT of a structured
    /// reference text resolves the table area"; probes/table-edits/table-insert-column-inside-header F!A4:A5, T-2911).
    ///
    /// Excel: every qualified shape resolves as in a formula (specifiers, column ranges, a bare
    /// table name, `@` on the INDIRECT cell's row, also from another sheet; any case), and a
    /// `Sheet!` prefix is ignored, even one naming a sheet without the table. An unqualified
    /// `[Col]` is `#REF!` even inside the table, and so is text starting with whitespace or ending
    /// in a tab; trailing spaces and line feeds are ignored, as for A1 text
    /// (docs/excel-semantics/lookup.md "INDIRECT of structured-reference text: every qualified
    /// shape"; probes/t3394-indirect-structref-shapes).
    pub(super) fn indirect_structured(
        &mut self,
        text: &str,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        if text.starts_with(char::is_whitespace) {
            return None;
        }
        let text = text.trim_end_matches([' ', '\n']);
        if text.ends_with(char::is_whitespace) {
            return None;
        }
        // A sheet prefix only has to name a sheet: `'Co A'!TabA[Qty]` and `S!TabA[Qty]` are the
        // table on another sheet. One naming no sheet is `#REF!` (`Nope!Sales[Qty]`,
        // `'No Sheet'!Sales[Qty]`), and so is an unquoted one Excel's A1 text refuses, as for a
        // defined name: `Co A!TabA[Qty]`, `P&L!TabB[Qty]` and `O'B!TabC[Qty]`, while their quoted
        // forms resolve (probes/t4354-indirect-structref-sheet-prefix).
        let text = match text.split_once('!') {
            Some((prefix, rest)) if !prefix.contains('[') => {
                if !prefix.starts_with('\'') && !is_indirect_unquoted_sheet(prefix) {
                    return None;
                }
                self.get_sheet_index_by_name(&unquote_sheet_prefix(prefix)?)?;
                rest
            }
            _ => text,
        };
        let sheet = self
            .workbook
            .worksheets
            .get(cell.sheet as usize)?
            .get_name();
        let context = CellReferenceRC {
            sheet,
            row: cell.row,
            column: cell.column,
        };
        let Node::StructuredReferenceKind { reference, lowered } =
            self.parser.parse(text, &context)
        else {
            return None;
        };
        if reference.unqualified {
            return None;
        }
        // `@` / `#This Row` is the INDIRECT cell's row; off the data rows it is `#REF!`.
        let lowered = lowered.this_row_at(cell.row).unwrap_or(*lowered);
        let (sheet, row1, column1, row2, column2) = match lowered {
            Node::ReferenceKind {
                sheet_index,
                row,
                column,
                ..
            } => (sheet_index, row, column, row, column),
            Node::RangeKind {
                sheet_index,
                row1,
                column1,
                row2,
                column2,
                ..
            } => (sheet_index, row1, column1, row2, column2),
            _ => return None,
        };
        Some(CalcResult::Range {
            left: CellReferenceIndex {
                sheet,
                row: row1,
                column: column1,
            },
            right: CellReferenceIndex {
                sheet,
                row: row2,
                column: column2,
            },
        })
    }

    /// INDIRECT of the text of a defined name that stands for a reference (`INDIRECT("RCR")`,
    /// `ISREF` TRUE; probes/r1c1-name-use-rcr, excel-graph). A name spelled like an out-of-range
    /// R1C1 reference is never resolved (T-1657).
    ///
    /// Excel: see docs/excel-semantics/lookup.md "INDIRECT resolves a defined name that holds a
    /// reference": `a1` FALSE still resolves a name, trailing (not leading) spaces are ignored, and
    /// a `Sheet!name` prefix picks the name scoped to that sheet, else the workbook name
    /// (probes/indirect-defined-name-text A5, A8, A22; T-1682).
    ///
    /// A relative name (`=Jan!A1`) is read from `cell`, the cell of the INDIRECT, as any other use
    /// of it is; that INDIRECT case is not probed yet (references.md "Relative references in
    /// defined names").
    pub(super) fn indirect_defined_name(
        &self,
        text: &str,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        let sheet = cell.sheet;
        let text = text.trim_end_matches([' ', '\n']);
        if is_out_of_range_r1c1_name(text) {
            return None;
        }
        let scope = match text.rsplit_once('!') {
            Some((prefix, _)) => {
                // Unquoted, the sheet name may hold only letters, digits, '_' and '.':
                // `INDIRECT("Co A!tn")`, `"P&L!tn"` and `"O'B!tn"` are #REF!, while `"2024!tn"`,
                // `"Q1!tn"`, `"x.y!tn"` and the quoted `"'O''B'!tn"` resolve
                // (probes/indirect-sheet-scoped-name, docs/excel-semantics/lookup.md).
                if !prefix.starts_with('\'') && !is_indirect_unquoted_sheet(prefix) {
                    return None;
                }
                self.get_sheet_index_by_name(&unquote_sheet_prefix(prefix)?)?
            }
            None => sheet,
        };
        let found = self
            .parsed_defined_name(text, Some(scope))
            .or_else(|| self.parsed_defined_name(text, None));
        match found? {
            ParsedDefinedName::CellReference(reference)
            | ParsedDefinedName::RangeReference(reference) => Some(reference.range_at(cell)),
            _ => None,
        }
    }
}
