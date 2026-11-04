//! Text a user types, which Excel refuses in places where it reads the same text stored.

use std::collections::HashMap;

use crate::calculated_columns::TableGrowth;
use crate::expressions::{
    lexer::LexerMode,
    parser::{
        is_limit_error, new_parser_english,
        stringify::bound_variables::to_english_string_typed_from_r1c1, Node,
    },
    types::CellReferenceRC,
};
use crate::model::Model;

mod r1c1;

/// Start of the error message [`Model::set_typed_user_input`] gives for a formula that does not
/// parse, which Excel refuses typed (Graph PATCH 400 `InvalidArgument`) where it keeps stored text
/// (T-3213). Callers that replay what a user typed leave such a cell as it was (decision 0179).
pub const TYPED_FORMULA_DOES_NOT_PARSE: &str = "Formula does not parse";

/// The A1 text of `text` typed in R1C1 mode into the 1-based `row`, `column` of `sheet`, as
/// Excel's `Range.FormulaR1C1` reads it (`=R1C2+1` in A3 is `=$B$1+1`, `=R[-1]C[1]` is `=B2`);
/// text without a leading `=` is a constant, the same in both modes. `None` when the formula does
/// not parse, which Excel refuses. `worksheets` are the workbook's sheet names; defined names are
/// kept as written and resolve when the A1 text is entered, but a structured reference does not
/// parse here (no tables), so such a formula is `None` (T-3113). Excel's short forms (`RC[-1]`,
/// `R[-1]C`, a whole row `R2` or column `C[-1]`, lower case) read as Excel reads them, and a
/// relative offset wraps around the grid (T-3212); a LET name or LAMBDA parameter `r`, `c`, `rc`
/// or `r1c1` is then a reference, which Excel refuses there as the A1 entry refuses `f1`. A name
/// spelled like an A1 cell (`f1`) is no reference in R1C1 mode and Excel binds it, so the A1 text
/// writes it `_xlpm.f1`, which the typed A1 entry binds (T-3897). Unbound, such a name is
/// Excel's undefined name (`#NAME?`) and `None` here, as the A1 text would read the cell.
#[must_use]
pub fn r1c1_formula_to_a1(
    text: &str,
    worksheets: &[String],
    sheet: &str,
    row: i32,
    column: i32,
) -> Option<String> {
    let Some(formula) = text.strip_prefix('=') else {
        return Some(text.to_string());
    };
    let formula = r1c1::expand_short_forms(formula, row, column)?;
    let mut parser = new_parser_english(worksheets.to_vec(), Vec::new(), HashMap::new());
    parser.set_lexer_mode(LexerMode::R1C1);
    // Typed, so that what Excel refuses typed is refused here too (a LET name that reads as a
    // reference in R1C1 mode, T-3212).
    parser.set_typed(true);
    let context = CellReferenceRC {
        sheet: sheet.to_string(),
        row,
        column,
    };
    let node = parser.parse(&formula, &context);
    // The A1 text printed below drops the whitespace, so a TAB or NBSP Excel refuses typed
    // (`=S!` TAB `R1C4`, probes/t3102-r1c1-ws-after-bang, T-3102) is refused here.
    if matches!(node, Node::ParseErrorKind { .. })
        || parser.skipped_refused_whitespace()
        || parser.typed_refusal().is_some()
    {
        return None;
    }
    to_english_string_typed_from_r1c1(&node, &context).map(|text| format!("={text}"))
}

/// Whether the one `(` that `formula` leaves open opens a call's arguments (`SUM(1`, `LOG10(2`,
/// `SUM(1,,`) rather than a group (`(1`, `1+(2`): typed, Excel closes the first and refuses the
/// second (probes/t3213-typed-parse-errors A2, A3, A6; T-3213). Text inside string literals, quoted
/// sheet names and brackets is skipped.
pub(crate) fn open_parenthesis_is_a_call(formula: &str) -> bool {
    let mut open = Vec::new();
    let mut quote = None;
    let mut brackets = 0_usize;
    let mut previous = None;
    for (index, c) in formula.char_indices() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') if brackets == 0 => quote = Some(c),
            (None, '[') => brackets += 1,
            (None, ']') => brackets = brackets.saturating_sub(1),
            (None, '(') if brackets == 0 => open.push(previous),
            (None, ')') if brackets == 0 => {
                open.pop();
            }
            _ => {}
        }
        previous = Some((index, c));
    }
    matches!(
        open.as_slice(),
        [Some((_, c))] if c.is_alphanumeric() || matches!(c, '_' | '.')
    )
}

impl Model {
    /// Parses a formula typed into a cell. A formula that fails to parse is
    /// retried with a closing parenthesis (`SUM(A1:A3` => `SUM(A1:A3)`) and
    /// otherwise kept as a parse error, except one past the parser's limits (nested too deep
    /// or too many operators; no `)` helps), which is rejected: the cell could never hold it.
    /// Typed ([`Model::set_typed_user_input`]), only a call's missing `)` is added, and any
    /// other parse error is rejected.
    pub(crate) fn parse_cell_formula(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        formula: &str,
    ) -> Result<Node, String> {
        let sheet = self.workbook.worksheet(sheet)?.get_name();
        let cell_reference = &CellReferenceRC { sheet, row, column };
        let mut node = self.parser.parse(formula, cell_reference);
        // Typed, Excel closes a call's parenthesis only, not a group's (`=(1` is refused).
        if matches!(&node, Node::ParseErrorKind { message, .. } if !is_limit_error(message))
            && (!self.parser.is_typed() || open_parenthesis_is_a_call(formula))
        {
            let closed = format!("{formula})");
            let retry = self.parser.parse(&closed, cell_reference);
            if !matches!(retry, Node::ParseErrorKind { .. }) {
                node = retry;
            }
        }
        match node {
            Node::ParseErrorKind { message, .. } if is_limit_error(&message) => {
                Err(format!("Formula: {message}"))
            }
            // Excel refuses a typed formula that does not parse (Graph PATCH 400
            // `InvalidArgument`, probes/t3113-entered-refused, t3213-typed-parse-errors), where it
            // keeps stored text as written (syntax.md "A typed formula that does not parse is
            // refused", T-3213).
            Node::ParseErrorKind { message, .. } if self.parser.is_typed() => {
                Err(format!("{TYPED_FORMULA_DOES_NOT_PARSE}: {message}"))
            }
            node => self.parser.typed_refusal().map_or(Ok(node), Err),
        }
    }

    /// Sets a cell parametrized by (`sheet`, `row`, `column`) with `value`.
    ///
    /// This mimics a user entering a value on a cell.
    ///
    /// If you enter a currency `$100` it will set as a number and update the style
    ///  Note that for currencies/percentage there is only one possible style
    ///  The value is always a string, so we need to try to cast it into numbers/booleans/errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rustsheet_engine::Model;
    /// # use rustsheet_engine::cell::CellValue;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// model.set_user_input(0, 1, 1, "$100".to_string());
    /// model.set_user_input(0, 2, 1, "$125".to_string());
    /// model.set_user_input(0, 3, 1, "-$10".to_string());
    /// model.set_user_input(0, 1, 2, "=SUM(A:A)".to_string());
    /// model.evaluate();
    /// assert_eq!(model.get_cell_value_by_index(0, 1, 2), Ok(CellValue::Number(215.0)));
    /// assert_eq!(model.get_formatted_cell_value(0, 1, 2), Ok("$215 ".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See also:
    /// * [`Model::update_cell_with_formula()`]
    /// * [`Model::update_cell_with_number()`]
    /// * [`Model::update_cell_with_bool()`]
    /// * [`Model::update_cell_with_text()`]
    pub fn set_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: String,
    ) -> Result<(), String> {
        // Text from outside the engine: a padded bare column `T[ Col ]` is no column here, as
        // Excel reads it typed or loaded (T-3258); the engine's own re-entries of its printed
        // text go through `reenter_printed_input`.
        self.parser.set_external(true);
        let result = self.reenter_printed_input(sheet, row, column, value);
        self.parser.set_external(false);
        result
    }

    /// [`Model::set_user_input`] for text a user types, which Excel refuses where it would read
    /// stored text: a LET name or LAMBDA parameter spelled like a cell (`=LET(f1,1,f1+1)`) is an
    /// error here (decision 0103, T-2497) and binds through `set_user_input`. A formula typed
    /// into a table's data rows fills its column as a calculated column, when
    /// [`Model::calculated_column`] finds the column empty or filled (T-2958, T-3024); a stored
    /// formula never fills. A formula or a constant typed just right of a table grows the table
    /// by a column first, and one typed just below it by a row ([`Model::table_expansion`],
    /// T-2968, T-3218, T-3273). A formula that does not parse is an error
    /// starting with [`TYPED_FORMULA_DOES_NOT_PARSE`], and the cell keeps its content (T-3213).
    pub fn set_typed_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: String,
    ) -> Result<(), String> {
        // A formula or constant typed just right of a table grows it first, so the new column
        // fills (T-2968, T-3218); one typed into the header cell only names the column. One typed
        // just below it grows it by a row (T-3273).
        let expansion = self.table_expansion(sheet, row, column, &value)?;
        if let Some(expansion) = &expansion {
            self.expand_table(sheet, expansion)?;
            if expansion.header {
                return Ok(());
            }
        }
        // A new row fills no calculated column, not even an empty or calculated one: unprobed
        // (t3003-below-last-row types into a column of constants), and the facade logs no fill.
        let fill = match expansion.map(|e| e.growth) {
            Some(TableGrowth::Down) => None,
            _ => self.calculated_column_fill(sheet, row, column)?,
        };
        self.parser.set_typed(true);
        let result = self.set_user_input(sheet, row, column, value);
        self.parser.set_typed(false);
        result?;
        match fill {
            Some(fill) => self.fill_calculated_column(sheet, row, column, &fill),
            None => Ok(()),
        }
    }

    /// [`Model::set_user_input`] for a formula as a file stores it: one naming a table or column
    /// the workbook lacks is `#REF!` instead of a parse error (T-2322).
    pub fn set_loaded_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: String,
    ) -> Result<(), String> {
        self.parser.set_loaded(true);
        let result = self.set_user_input(sheet, row, column, value);
        self.parser.set_loaded(false);
        result
    }
}
