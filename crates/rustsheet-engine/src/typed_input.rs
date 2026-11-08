//! Text a user types, which Excel refuses in places where it reads the same text stored.
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::match_same_arms)]
// LINT-DEBT-END

use crate::calculated_columns::TableGrowth;
use crate::expressions::{
    lexer::LexerMode,
    parser::{is_limit_error, Node},
    types::CellReferenceRC,
};
use crate::model::Model;

mod r1c1;

/// Start of the error message [`Model::set_typed_user_input`] gives for a formula that does not
/// parse, which Excel refuses typed (Graph PATCH 400 `InvalidArgument`) where it keeps stored text
/// (T-3213). Callers that replay what a user typed leave such a cell as it was (decision 0179).
pub const TYPED_FORMULA_DOES_NOT_PARSE: &str = "Formula does not parse";

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
        let cell_reference = &CellReferenceRC { sheet, column, row };
        if self.parser.is_r1c1_entry() {
            // Typed in R1C1 mode ([`Model::set_typed_r1c1_user_input`]): the short forms are
            // written out in the R1C1 grammar the parser reads (T-3212), and the parser reads
            // that grammar for this one formula only, so that a stored formula re-parsed while
            // the entry runs is read as ever.
            let expanded = r1c1::expand_short_forms(formula, row, column).ok_or_else(|| {
                format!("{TYPED_FORMULA_DOES_NOT_PARSE}: a short R1C1 form Excel refuses")
            })?;
            self.parser.set_lexer_mode(LexerMode::R1C1);
            let parsed = self.parse_typed_text(&expanded, cell_reference);
            self.parser.set_lexer_mode(LexerMode::A1);
            return parsed;
        }
        self.parse_typed_text(formula, cell_reference)
    }

    /// [`Model::parse_cell_formula`] of `formula` in the parser's current grammar.
    fn parse_typed_text(
        &mut self,
        formula: &str,
        cell_reference: &CellReferenceRC,
    ) -> Result<Node, String> {
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
        self.as_external(|model| model.reenter_printed_input(sheet, row, column, value))
    }

    /// Runs `entry` with the parser reading formulas as untyped text from outside the engine
    /// ([`Parser::set_external`](crate::expressions::parser::Parser)), then restores the engine's
    /// reading. Each public entry that takes formula text from an embedder wraps its crate-internal
    /// re-entry with it (T-3258, T-4023).
    pub(crate) fn as_external<T>(&mut self, entry: impl FnOnce(&mut Self) -> T) -> T {
        self.parser.set_external(true);
        let result = entry(self);
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

    /// [`Model::set_typed_user_input`] of `value` typed with the workbook in R1C1 reference style,
    /// as Excel's `Range.FormulaR1C1` reads it: `=R1C2+1` in A3 is `=$B$1+1`, `=R[-1]C[1]` is
    /// `=B2`, and text without a leading `=` is a constant, the same in both styles. Excel's short
    /// forms (`RC[-1]`, `R[-1]C`, a whole row `R2` or column `C[-1]`, lower case) read as Excel
    /// reads them, and a relative offset wraps around the grid (T-3212); a LET name or LAMBDA
    /// parameter `r`, `c`, `rc` or `r1c1` is then a reference, which Excel refuses. A name spelled
    /// like an A1 cell (`f1`) is no reference here: a LET or LAMBDA binds it (T-3897), and unbound
    /// it is Excel's undefined name, which FORMULATEXT shows quoted, `='f1'` (T-4040). The parsed
    /// formula is stored as it is, with no A1 text in between, so no text typed in A1 mode reads
    /// as such a name (T-4288). A formula that does not parse is an error starting with
    /// [`TYPED_FORMULA_DOES_NOT_PARSE`], and the cell keeps its content.
    pub fn set_typed_r1c1_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: String,
    ) -> Result<(), String> {
        self.as_r1c1_entry(|model| model.set_typed_user_input(sheet, row, column, value))
    }

    /// Runs `entry` with typed formulas read as typed in R1C1 reference style
    /// ([`Model::set_typed_r1c1_user_input`]), then restores A1 style. An embedder that runs its
    /// own typed-entry steps around the write (the facade's calculated-column log reads
    /// [`Model::table_expansion`] first) wraps them with it too.
    pub fn as_r1c1_entry<T>(&mut self, entry: impl FnOnce(&mut Self) -> T) -> T {
        self.parser.set_r1c1_entry(true);
        let result = entry(self);
        self.parser.set_r1c1_entry(false);
        result
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
