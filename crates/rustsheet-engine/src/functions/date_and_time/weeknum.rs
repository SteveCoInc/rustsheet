use chrono::Datelike;

use super::num_error;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    formatter::dates::from_excel_date,
    functions::{util::near_integer::near_integer_whole, Function},
    model::Model,
    tailed_array::range_corners,
};

/// WEEKNUM's stale `return_type` state: the start weekday the last WEEKNUM or ISOWEEKNUM of the
/// recalc kept, read by a `return_type` of 22+ or negative (`None` when empty; SPEC-0017, dates.md
/// "WEEKNUM's stale `return_type` state").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum WeeknumStart {
    Sunday,
    Monday,
}

/// Whether `node` is a written range spanning more than one cell (`C1:C2`; `C1` is not), and
/// whether it spans more than one row and more than one column (`C1:D2`). Other forms (a name,
/// INDEX) are unprobed and count as neither.
fn written_extent(node: &Node, cell: CellReferenceIndex) -> (bool, bool) {
    let Some((first, last)) = range_corners(node.ungrouped(), cell) else {
        return (false, false);
    };
    let (rows, columns) = (first.row != last.row, first.column != last.column);
    (rows || columns, rows && columns)
}

/// Whether `node` is a written reference to one cell (`C1`, or a range such as `C1:C1`).
fn is_one_cell(node: &Node, cell: CellReferenceIndex) -> bool {
    match node.ungrouped() {
        Node::ReferenceKind { .. } => true,
        Node::RangeKind { .. } => !written_extent(node, cell).0,
        _ => false,
    }
}

/// Whether `node` is, as written, a reference CONCAT and TEXTJOIN join: a cell, a range, or a call
/// that returns one (`CONCAT(INDEX(C1:D2,1,1))` keeps WEEKNUM's state, probes/weeknum-stale-call-
/// shapes-2 S26; `CONCAT(INDEX({1,2},1))` empties it, probes/weeknum-stale-call-shapes-3 S17).
/// Read from the formula, never by evaluating the argument again. IF and CHOOSE count when a
/// result argument does, LET when any argument does, and a defined name always (shapes-3
/// S20-S23, a name for a range; a name for a constant is unprobed).
fn joins_a_reference(node: &Node) -> bool {
    match node.ungrouped() {
        Node::ReferenceKind { .. }
        | Node::RangeKind { .. }
        | Node::OpRangeKind { .. }
        | Node::SheetSpanKind { .. } => true,
        Node::FunctionKind { kind, args } => match kind {
            Function::Index => args.first().is_some_and(joins_a_reference),
            Function::If | Function::Choose => args.iter().skip(1).any(joins_a_reference),
            Function::Let => args.iter().any(joins_a_reference),
            _ => kind.returns_reference(),
        },
        Node::DefinedNameKind(_) => true,
        _ => false,
    }
}

impl Model {
    /// Empties WEEKNUM's state after a call of `kind` with `args` when Excel's does (dates.md
    /// "Which calls empty WEEKNUM's state", T-4289, T-4315; probes/weeknum-stale-between-calls,
    /// -between-ranges, -call-shapes, -call-shapes-2 and -call-shapes-3). The rule is per function and argument
    /// shape, nested calls included, and Excel gives no reason for it:
    /// - CHOOSEROWS, CHOOSECOLS, TOCOL, TOROW, VSTACK, HSTACK, TAKE, DROP, TRIMRANGE, WRAPROWS, WRAPCOLS,
    ///   TEXTSPLIT and EXPAND always do (a constant array, one cell or a range);
    /// - SORT and UNIQUE with a multi-cell range (`SORT(C1:C2)`; `SORT({3,1})` and `SORT(C1)` keep
    ///   it); SORTBY unless its array is one cell (`SORTBY({1,2},{2,1})` empties, `SORTBY(C1,1)`
    ///   keeps it); ARRAYTOTEXT with a range of more than one row and column (`C1:D2`; `C1:C2`,
    ///   `C1` and `{1,2}` keep it);
    /// - CONCAT and TEXTJOIN unless a joined argument is a reference (`CONCAT("a",1)` and
    ///   `CONCAT(C1&"")` empty it; `CONCAT(C1:D2,"x")`, `CONCAT(INDEX(C1:D2,1,1))` and
    ///   `TEXTJOIN(",",1,C1,"x")` keep it; only TEXTJOIN's joined arguments are read).
    ///
    /// FILTER, TRANSPOSE, OFFSET, INDEX, SEQUENCE, XLOOKUP, LET and LAMBDA keep it. Whether the
    /// emptying comes before or after the call's own arguments run is unprobed (here: after).
    fn weeknum_effect_of_call(&mut self, kind: &Function, args: &[Node], cell: CellReferenceIndex) {
        let first = || args.first().map(|node| written_extent(node, cell));
        let empties = match kind {
            Function::Chooserows
            | Function::Choosecols
            | Function::Tocol
            | Function::Torow
            | Function::Vstack
            | Function::Hstack
            | Function::Take
            | Function::Drop
            | Function::Trimrange
            | Function::Wraprows
            | Function::Wrapcols
            | Function::Textsplit
            | Function::Expand => true,
            Function::Sort | Function::Unique => first().is_some_and(|(multi, _)| multi),
            Function::Sortby => !args.first().is_some_and(|node| is_one_cell(node, cell)),
            Function::Arraytotext => first().is_some_and(|(_, both)| both),
            Function::Concat => !args.iter().any(joins_a_reference),
            Function::Textjoin => !args.iter().skip(2).any(joins_a_reference),
            _ => false,
        };
        if empties {
            self.empty_weeknum_state();
        }
    }

    /// Calls `implementation` (through `call_by_reference_elements`), then applies the call's
    /// effect on WEEKNUM's state (`weeknum_effect_of_call`): every function call goes through it
    /// (`evaluate_function`).
    pub(crate) fn call_with_order_effects(
        &mut self,
        kind: &Function,
        args: &[Node],
        cell: CellReferenceIndex,
        implementation: fn(&mut Model, &[Node], CellReferenceIndex) -> CalcResult,
    ) -> CalcResult {
        let result = self.call_by_reference_elements(kind, args, cell, implementation);
        self.weeknum_effect_of_call(kind, args, cell);
        result
    }

    /// Empties WEEKNUM's state and counts it, so an Excel-order anchor's run can tell that it did
    /// (`AnchorEffects::emptied_weeknum`).
    pub(crate) fn empty_weeknum_state(&mut self) {
        self.evaluation.order_state.weeknum_start = None;
        self.evaluation.anchor_effects.weeknum_empties += 1;
    }
}

impl Model {
    /// `WEEKNUM(serial, [return_type])`; 1900 weeks follow Excel's Sunday serial 1 (docs/excel-semantics/dates.md).
    /// Both arguments lift over arrays and refuse a multi-cell range (SPEC-0011 `LiftArrays`,
    /// probes/date-argument-modes).
    pub(crate) fn fn_weeknum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let result = self.lift_call(Function::Weeknum, args, cell, &mut |model, values| {
            model.weeknum_of(values, cell)
        });
        if matches!(result, CalcResult::Array(_)) {
            // A WEEKNUM that lifts over an array empties the state, whatever its types
            // (`SUM(WEEKNUM(100,{2,2}))`; dates.md "WEEKNUM's stale return_type state").
            self.evaluation.order_state.weeknum_start = None;
        }
        result
    }

    /// WEEKNUM of one element. Once both arguments convert and the serial is valid, it updates
    /// the stale `return_type` state, or reads it for a `return_type` of 22+ or negative (SPEC-0017
    /// rules 2, 4 and 5).
    fn weeknum_of(&mut self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let (serial, return_type) = match values {
            [serial] => (serial, None),
            [serial, return_type] => (serial, Some(return_type)),
            _ => return CalcResult::new_args_number_error(cell),
        };
        // A boolean serial is #VALUE!, before the return_type is read (`WEEKNUM(<FALSE cell>,1)`;
        // dates.md "WEEKNUM of a boolean serial is #VALUE!", gen-m6, T-2276); WEEKDAY and
        // ISOWEEKNUM read it as 1. A typed or array-constant boolean is `#VALUE!` too
        // (probes/weeknum-boolean-serial A1:A8, A11).
        let serial = match self.cast_number_no_bools_ref(serial, cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let out_of_range = || num_error(cell);
        // The weeks are numbered on the 1900 calendar: a 1904 serial is shifted to it first, so
        // `WEEKNUM(0)` is 1 there (1904-01-01; probes/date-system-1904-readers A5).
        let Some(serial) = self.workbook.settings.date_system.to_1900_serial(serial) else {
            return out_of_range();
        };
        // Serial 0 is "Saturday 0 January 1900": no calendar date, but WEEKNUM numbers it.
        let date = match from_excel_date(serial) {
            Ok(d) => Some(d),
            Err(_) if serial == 0 => None,
            Err(_) => return out_of_range(),
        };
        // A boolean return_type is #VALUE! (`WEEKNUM(100,TRUE)`); text that reads as a number
        // is accepted (`WEEKNUM(100,"2")` = 16; probes/weeknum-serial-0 C35-C38, T-1023).
        let return_type = match return_type {
            // A positive return_type just below an integer counts as that integer, as LEFT's count
            // does (`WEEKNUM(45019,2-2^-23)` is type 2's 15; probes/t4454-near-midnight-date-args
            // AA1:AB4, T-4454); 2^-22 + 9*2^-36 below it truncates. A negative one floors with
            // the same snap, not truncates: -0.5 reads as -1 (the stale state) and -2^-30 as 0
            // (`#NUM!`; probes/t4470-weeknum-return-type-edges, T-4470). The negative-side window is
            // half the positive one, as DATE/TIME's: -2^-23 and -(2^-23+2^-34) snap to 0, -(2^-22+2^-33)
            // and -(2^-22+9*2^-36) floor to -1 (probes/t4619-weeknum-negative-snap-window, T-4619).
            Some(value) => match self.cast_number_no_bools_ref(value, cell) {
                Ok(f) => near_integer_whole(f) as i32,
                Err(s) => return s,
            },
            None => 1,
        };
        // Types 1 and 17 keep a Sunday start, 2, 11 and 21 a Monday one, and every other type
        // below 22 empties the state, the always-#NUM! ones included (probes/weeknum-stale-setters).
        let state = &mut self.evaluation.order_state.weeknum_start;
        match return_type {
            1 | 17 => *state = Some(WeeknumStart::Sunday),
            2 | 11 | 21 => *state = Some(WeeknumStart::Monday),
            0..=20 => *state = None,
            _ => {}
        }
        if return_type == 21 {
            return match iso_week(serial) {
                Ok(week) => CalcResult::Number(week),
                Err(error) => CalcResult::new_error(error, cell, String::new()),
            };
        }
        let start_offset = match return_type {
            1 | 17 => chrono::Weekday::Sun,
            2 | 11 => chrono::Weekday::Mon,
            12 => chrono::Weekday::Tue,
            13 => chrono::Weekday::Wed,
            14 => chrono::Weekday::Thu,
            15 => chrono::Weekday::Fri,
            16 => chrono::Weekday::Sat,
            // 0, 3-10 and 18-20 are #NUM! (probes/weeknum-serial-0 C3, C4, C18, C19, C23-C26).
            0..=20 => return num_error(cell),
            // 22+ and negatives read the state left by the WEEKNUM and ISOWEEKNUM calls Excel ran
            // before (in its fresh-load order, `excel_order.rs`): weeks numbered as type 1 does,
            // from that weekday, and #NUM! when it is empty (docs/excel-semantics/dates.md
            // "WEEKNUM's stale return_type state", probes/weeknum-stale-*, SPEC-0017).
            _ => {
                self.evaluation.order_state_read = true;
                match self.evaluation.order_state.weeknum_start {
                    Some(WeeknumStart::Sunday) => chrono::Weekday::Sun,
                    Some(WeeknumStart::Monday) => chrono::Weekday::Mon,
                    None => return num_error(cell),
                }
            }
        };
        // Excel numbers the weeks of 1900 as if serial 1 were a Sunday (the real 1 Jan 1900 is a
        // Monday): the week-1 anchor is the last serial <= 1 on the start weekday, and serials
        // 0 and 1 may share a week. Verified for serials 0..=366, later years use the real
        // calendar (docs/excel-semantics/dates.md, probes/dates-early-1900-edges,
        // probes/weeknum-1900-later; T-0995).
        if (0..=366).contains(&serial) {
            let start = i64::from(start_offset.num_days_from_sunday());
            let first = 1 - (-start).rem_euclid(7);
            return CalcResult::Number(((serial - first).div_euclid(7) + 1) as f64);
        }
        let Some(date) = date else {
            return out_of_range();
        };
        // 1 January of `date`'s year, then back to the start weekday.
        let Some(mut first) = date.with_ordinal(1) else {
            return out_of_range();
        };
        while first.weekday() != start_offset {
            first -= chrono::Duration::days(1);
        }
        let week = (date - first).num_days() / 7 + 1;
        CalcResult::Number(week as f64)
    }
}

/// The ISO week of `serial` (WEEKNUM type 21, ISOWEEKNUM). Serial 0 ("Saturday 0 January 1900")
/// shares serial 1's ISO week: `WEEKNUM(0,21)` = `ISOWEEKNUM(0)` = 52 (probes/weeknum-serial-0
/// C1, C2, C17, C21; T-1023); a negative serial is #NUM!.
pub(super) fn iso_week(serial: i64) -> Result<f64, Error> {
    let serial = if serial == 0 { 1 } else { serial };
    let date = from_excel_date(serial).map_err(|_| Error::NUM)?;
    Ok(f64::from(date.iso_week().week()))
}
