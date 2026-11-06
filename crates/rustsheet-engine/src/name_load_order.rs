//! What an unqualified use of an undefined name was bound to when Excel loaded the workbook
//! (T-4618; docs/excel-semantics/syntax.md "A sheet-scoped name added after load").
//!
//! Excel binds a formula's names as it loads it: the file's defined names first, then the
//! active sheet, then the other sheets in tab order, each sheet's cells by row and then column,
//! and each formula's names in the order it writes them. An undefined name gets places to bind
//! to: a sheet-qualified `S!N` makes S's own place, unless one stands; an unqualified `N` binds
//! to the provisional place if there is one, else to its sheet's own place, else makes the
//! provisional place, which belongs to its sheet. A sheet-qualified `S!N` met while S holds the
//! provisional place fixes that place to the workbook. A sheet-scoped `N` added through the API
//! on S later fills S's own place, and the provisional place when S holds it unfixed, so S's
//! unqualified uses read it only then (probes/t4554-api-name-local-name-body Last!D3,
//! probes/t4394-api-name-active-last S2!A1 and S2!A3, and the t4618-* probes, excel-graph
//! 2026-11-06).

use crate::expressions::lexer::LexerMode;
use crate::expressions::parser::stringify::{name_load_events, NameLoadEvent};
use crate::expressions::types::CellReferenceRC;
use crate::model::Model;

impl Model {
    /// Whether the unqualified uses of the undefined name `name` on the sheet at index `sheet`
    /// are bound to a place that a sheet-scoped `name` added on that sheet fills, read from the
    /// formulas as they stand, in Excel's load order (module docs). A use typed after the load is
    /// read where its cell stands (UNVERIFIED: the probes type only on the scoped sheet, where
    /// the binding agrees). `false` when the sheet has no such use.
    pub(crate) fn loaded_uses_take_sheet_place(&mut self, name: &str, sheet: u32) -> bool {
        let mut places = LoadPlaces::default();
        // The file's defined names are read before any cell (probes/t4618-name-body-qualified).
        let context = self.defined_name_context();
        self.parser.set_lexer_mode(LexerMode::A1);
        for index in 0..self.workbook.defined_names.len() {
            let Some(defined_name) = self.workbook.defined_names.get(index) else {
                break;
            };
            let on = defined_name.sheet_id;
            let stored = defined_name.formula.trim_start();
            let body = stored.strip_prefix('=').unwrap_or(stored);
            let mut node = self.parser.parse(body, &context);
            places.read(on, sheet, &name_load_events(&mut node, name));
        }
        let active = self.active_sheet();
        let count = u32::try_from(self.workbook.worksheets.len()).unwrap_or(u32::MAX);
        let order = std::iter::once(active).chain((0..count).filter(|&index| index != active));
        // All internal cell formulas are R1C1, and a name's binding does not depend on the cell.
        self.parser.set_lexer_mode(LexerMode::R1C1);
        for on in order {
            let Ok(worksheet) = self.workbook.worksheet(on) else {
                continue;
            };
            let cell_reference = CellReferenceRC {
                sheet: worksheet.get_name(),
                row: 1,
                column: 1,
            };
            // Each shared formula is read once; its events repeat for every cell that holds it.
            let mut events: Vec<Option<Vec<NameLoadEvent>>> =
                vec![None; worksheet.shared_formulas.len()];
            for (_, _, cell) in worksheet.sheet_data.cells() {
                let Some(index) = cell.get_formula().and_then(|f| usize::try_from(f).ok()) else {
                    continue;
                };
                let (Some(slot), Some(formula)) =
                    (events.get_mut(index), worksheet.shared_formulas.get(index))
                else {
                    continue;
                };
                let formula_events = slot.get_or_insert_with(|| {
                    name_load_events(&mut self.parser.parse(formula, &cell_reference), name)
                });
                places.read(Some(on), sheet, formula_events);
            }
        }
        self.parser.set_lexer_mode(LexerMode::A1);
        places.filled_by_add_on(sheet)
    }
}

/// The places of one undefined name while the workbook loads (module docs).
#[derive(Default)]
struct LoadPlaces {
    /// The sheets whose own place a sheet-qualified use made.
    own: Vec<u32>,
    /// The provisional place: the sheet whose unqualified use made it (`None` for a workbook
    /// name's body), and whether a sheet-qualified use fixed it to the workbook.
    provisional: Option<(Option<u32>, bool)>,
    /// What the unqualified uses on the scoped sheet bound to: its own place (`Some(true)`) or
    /// the provisional one (`Some(false)`). They all bind alike, a sheet's cells being read
    /// together.
    scoped_uses: Option<bool>,
}

impl LoadPlaces {
    /// Reads one formula's events, met on sheet `on` (`None` for a workbook name's body), for
    /// the uses on sheet `scoped`.
    fn read(&mut self, on: Option<u32>, scoped: u32, events: &[NameLoadEvent]) {
        for &event in events {
            match event {
                NameLoadEvent::Qualified(target) => {
                    if self.own.contains(&target) {
                        continue;
                    }
                    match &mut self.provisional {
                        Some((holder, fixed)) if *holder == Some(target) => *fixed = true,
                        _ => self.own.push(target),
                    }
                }
                // A use pinned to the workbook name binds as a fixed provisional place does.
                NameLoadEvent::Workbook => match &mut self.provisional {
                    Some((_, fixed)) => *fixed = true,
                    None => self.provisional = Some((None, true)),
                },
                NameLoadEvent::Bare => {
                    let own =
                        self.provisional.is_none() && on.is_some_and(|on| self.own.contains(&on));
                    if !own && self.provisional.is_none() {
                        self.provisional = Some((on, false));
                    }
                    if on == Some(scoped) && self.scoped_uses.is_none() {
                        self.scoped_uses = Some(own);
                    }
                }
            }
        }
    }

    /// Whether a sheet-scoped add on `scoped` fills the place its unqualified uses bound to.
    fn filled_by_add_on(&self, scoped: u32) -> bool {
        match self.scoped_uses {
            Some(own) => own || self.provisional == Some((Some(scoped), false)),
            None => false,
        }
    }
}
