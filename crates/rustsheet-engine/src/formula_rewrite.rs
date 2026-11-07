//! Rewriting the stored text of every formula and defined name for a workbook edit (a table or
//! sheet delete, a sheet move), keeping what changed so the undo can put it back; and the formula
//! cells a row or column delete rewrote past what re-inserting them undoes.

use std::collections::HashMap;

use crate::{
    expressions::{
        lexer::LexerMode,
        parser::{
            stringify::{
                to_english_string, to_rc_format, to_string_displaced, to_string_displaced_keeping,
                DisplaceData, Displacement,
            },
            Node,
        },
        types::CellReferenceRC,
    },
    language::get_default_language,
    locale::get_default_locale,
    model::Model,
    types::Cell,
};

impl Model {
    /// Parses every stored formula of sheet `only` (of every sheet when `None`), applies
    /// `rewrite` to it in place and stores the result. Unlike [`Model::rewrite_formulas`] it
    /// leaves defined names alone and keeps no undo record: a defined-name rename or add, whose
    /// own undo re-reads the formulas. The caller re-parses the workbook afterwards
    /// ([`Model::reset_parsed_structures`]).
    pub(crate) fn rewrite_shared_formulas(
        &mut self,
        only: Option<u32>,
        mut rewrite: impl FnMut(&mut Node),
    ) {
        // All internal formulas are R1C1.
        self.parser.set_lexer_mode(LexerMode::R1C1);
        for (index, worksheet) in self.workbook.worksheets.iter_mut().enumerate() {
            if only.is_some_and(|only| usize::try_from(only).ok() != Some(index)) {
                continue;
            }
            let cell_reference = CellReferenceRC {
                sheet: worksheet.get_name(),
                row: 1,
                column: 1,
            };
            let mut formulas = Vec::with_capacity(worksheet.shared_formulas.len());
            for formula in &worksheet.shared_formulas {
                let mut t = self.parser.parse(formula, &cell_reference);
                rewrite(&mut t);
                formulas.push(to_rc_format(&t));
            }
            worksheet.shared_formulas = formulas;
        }
        self.parser.set_lexer_mode(LexerMode::A1);
    }

    /// [`Model::rewrite_formulas`] for a sheet edit: `edit` changes the parsed formula in place,
    /// then it is printed in English (A1) where a cell is known and in R1C1 for a shared formula.
    pub(crate) fn rewrite_formulas_edited(
        &mut self,
        skip_sheet: Option<usize>,
        edit: impl Fn(&mut Node),
    ) -> Vec<RewrittenFormula> {
        self.rewrite_formulas(skip_sheet, |mut node, context| {
            edit(&mut node);
            match context {
                Some(context) => to_english_string(&node, context),
                None => to_rc_format(&node),
            }
        })
    }

    /// Prints every formula of every sheet but `skip_sheet`, and every defined name, with
    /// `print` (given the parsed formula, and the cell it is read on for a defined name, or no
    /// cell for an R1C1 shared formula), stores the text where it changed and returns what it
    /// changed. Does not reparse.
    pub(crate) fn rewrite_formulas(
        &mut self,
        skip_sheet: Option<usize>,
        print: impl Fn(Node, Option<&CellReferenceRC>) -> String,
    ) -> Vec<RewrittenFormula> {
        let mut rewritten = Vec::new();
        self.parser.set_lexer_mode(LexerMode::R1C1);
        for (index, worksheet) in self.workbook.worksheets.iter_mut().enumerate() {
            if Some(index) == skip_sheet {
                continue;
            }
            let context = CellReferenceRC {
                sheet: worksheet.get_name(),
                row: 1,
                column: 1,
            };
            for (formula_index, formula) in worksheet.shared_formulas.iter_mut().enumerate() {
                let node = self.parser.parse(formula, &context);
                let text = print(node, None);
                if text != *formula {
                    rewritten.push(RewrittenFormula {
                        place: FormulaPlace::Shared {
                            sheet_id: worksheet.sheet_id,
                            index: formula_index,
                        },
                        original: std::mem::replace(formula, text.clone()),
                        rewritten: text,
                    });
                }
            }
        }
        self.parser.set_lexer_mode(LexerMode::A1);
        // Defined names have no cell context; any cell of a remaining sheet does.
        let remaining = self
            .workbook
            .worksheets
            .iter()
            .enumerate()
            .find(|(index, _)| Some(*index) != skip_sheet)
            .map(|(_, worksheet)| worksheet.get_name());
        let Some(sheet) = remaining else {
            return rewritten;
        };
        let context = CellReferenceRC {
            sheet,
            row: 1,
            column: 1,
        };
        let names =
            self.rewrite_defined_names(&context, |node, context| print(node, Some(context)));
        rewritten.extend(
            names
                .into_iter()
                .map(|(index, original, text)| RewrittenFormula {
                    place: FormulaPlace::DefinedName(index),
                    original,
                    rewritten: text,
                }),
        );
        rewritten
    }

    /// Puts back the formulas [`Model::rewrite_formulas`] rewrote, where they still hold the
    /// rewritten text (the undo of the edit). Does not reparse. It goes from the last entry to
    /// the first, so an edit that rewrote a formula twice (a sheet delete's table references,
    /// then its 3-D spans) gets its original text back.
    pub(crate) fn restore_rewritten_formulas(&mut self, rewritten: Vec<RewrittenFormula>) {
        for RewrittenFormula {
            place,
            original,
            rewritten,
        } in rewritten.into_iter().rev()
        {
            let slot = match place {
                FormulaPlace::Shared { sheet_id, index } => self
                    .workbook
                    .worksheets
                    .iter_mut()
                    .find(|worksheet| worksheet.sheet_id == sheet_id)
                    .and_then(|worksheet| worksheet.shared_formulas.get_mut(index)),
                FormulaPlace::DefinedName(index) => self
                    .workbook
                    .defined_names
                    .get_mut(index)
                    .map(|defined_name| &mut defined_name.formula),
            };
            if let Some(slot) = slot.filter(|slot| **slot == rewritten) {
                *slot = original;
            }
        }
    }
}

impl Model {
    /// Whether `displaced`, the text the row or column delete `data` gave a formula that read
    /// `original` (both in the active locale, on the cell `context`), reads otherwise once the
    /// rows or columns are inserted again: a reference to a deleted cell made `#REF!` stays so,
    /// and a range that lost an edge does not get it back (T-4668). Any other edit is never lost.
    pub(crate) fn lost_on_reinsert(
        &mut self,
        data: &DisplaceData,
        context: &CellReferenceRC,
        original: &str,
        displaced: &str,
    ) -> bool {
        let reinsert = match *data {
            DisplaceData::Column {
                sheet,
                column,
                delta,
            } if delta < 0 => DisplaceData::Column {
                sheet,
                column,
                delta: delta.saturating_neg(),
            },
            DisplaceData::Row { sheet, row, delta } if delta < 0 => DisplaceData::Row {
                sheet,
                row,
                delta: delta.saturating_neg(),
            },
            _ => return false,
        };
        let node = self.parser.parse(displaced, context);
        to_string_displaced(&node, context, &reinsert, self.locale, self.language) != original
    }
}

/// A formula cell that a row or column delete rewrote so that re-inserting the rows or columns
/// does not give its formula back ([`Model::lost_on_reinsert`]), as it was before the delete and at
/// its place then, so the undo can put it back (T-4668).
#[derive(Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub(crate) struct RewrittenCell {
    /// The index of the cell's sheet.
    pub(crate) sheet: u32,
    pub(crate) row: i32,
    pub(crate) column: i32,
    pub(crate) cell: Cell,
}

/// The formula cells a delete moved on one sheet, by the place it moved each to: the place it had
/// before and the cell as it was there (its formula is relative to that place).
pub(crate) type MovedFormulas = HashMap<(i32, i32), ((i32, i32), Cell)>;

/// `rewritten`, read on the cells after a delete moved them, at the places and with the cells they
/// had before it: a cell the delete moved on `sheet` is in `moved`.
pub(crate) fn before_moves(
    mut rewritten: Vec<RewrittenCell>,
    sheet: u32,
    mut moved: MovedFormulas,
) -> Vec<RewrittenCell> {
    for entry in rewritten.iter_mut().filter(|entry| entry.sheet == sheet) {
        if let Some(((row, column), cell)) = moved.remove(&(entry.row, entry.column)) {
            (entry.row, entry.column, entry.cell) = (row, column, cell);
        }
    }
    rewritten
}

/// A formula that a workbook edit rewrote, kept so the undo can put it back.
#[derive(Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub(crate) struct RewrittenFormula {
    place: FormulaPlace,
    original: String,
    rewritten: String,
}

/// Where a [`RewrittenFormula`] lives.
#[derive(Debug, Clone, Copy, bitcode::Encode, bitcode::Decode)]
enum FormulaPlace {
    /// A shared formula of the sheet with this id.
    Shared { sheet_id: u32, index: usize },
    /// A defined name, by its index in the workbook's list.
    DefinedName(usize),
}

impl Model {
    /// Rewrites every defined name's formula for a row or column insert or delete on `sheet`, as a
    /// cell formula's references are rewritten: Excel shifts a name over the lines after the edit,
    /// shrinks a range that loses an edge and makes a name on a deleted cell `#REF!`
    /// (probes/sheet-ops/t4702-names-*, t4702-name-deleted-cell-*, excel-graph, 2026-11-07).
    /// Moves are left alone (unprobed). Names are stored in English and have no cell of their
    /// own: A1 of the edited sheet anchors them, which only a relative reference reads (unprobed).
    /// A name the edit does not change keeps its text as stored.
    pub(crate) fn displace_defined_names(&mut self, sheet: u32, displacement: Displacement) {
        // Excel leaves a relative reference in a name where it is, whichever sheet uses the name
        // (probes/sheet-ops/t4747-name-relative-*, t4760-name-relative-same-sheet-*, excel-graph);
        // absolute ones shift (T-4702).
        let displacement = Displacement {
            keep_relative: true,
            ..displacement
        };
        if !matches!(
            displacement.data,
            DisplaceData::Row { .. } | DisplaceData::Column { .. }
        ) {
            return;
        }
        let Some(worksheet) = self.workbook.worksheets.get(sheet as usize) else {
            return;
        };
        let context = CellReferenceRC {
            sheet: worksheet.get_name(),
            row: 1,
            column: 1,
        };
        let changed = self.rewrite_defined_names(&context, |node, context| {
            let (locale, language) = (get_default_locale(), get_default_language());
            to_string_displaced_keeping(&node, context, displacement, locale, language)
        });
        if !changed.is_empty() {
            // Evaluation reads the parsed names, not the stored text.
            self.parse_defined_names();
        }
    }

    /// Prints each defined name's formula, parsed in English (as names are stored) on
    /// `context`, with `print`, and stores the text, keeping a leading `=`, where it differs from
    /// the name as parsed. Returns each changed name's index, formula before and formula after.
    fn rewrite_defined_names(
        &mut self,
        context: &CellReferenceRC,
        print: impl Fn(Node, &CellReferenceRC) -> String,
    ) -> Vec<(usize, String, String)> {
        let mut changed = Vec::new();
        for index in 0..self.workbook.defined_names.len() {
            let Some(original) = self
                .workbook
                .defined_names
                .get(index)
                .map(|n| n.formula.clone())
            else {
                continue;
            };
            let stored = original.trim_start();
            let (eq, body) = stored.strip_prefix('=').map_or(("", stored), |b| ("=", b));
            let node = self.parse_internal_formula(body, context);
            let unchanged = to_english_string(&node, context);
            let text = print(node, context);
            if text != unchanged {
                let text = format!("{eq}{text}");
                if let Some(name) = self.workbook.defined_names.get_mut(index) {
                    name.formula.clone_from(&text);
                }
                changed.push((index, original, text));
            }
        }
        changed
    }
}
