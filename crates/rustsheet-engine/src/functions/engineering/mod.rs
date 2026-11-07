mod bessel;
mod bit_operations;
mod complex;
mod convert;
mod misc;
mod number_basis;
pub(crate) mod transcendental;

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::{lift::Kernel, util::empty_args::written_empty_na, Function},
    model::Model,
};

impl Model {
    /// A fixed-arity Analysis `ToolPak` engineering call whose first `strict` arguments may not be
    /// written empty (`DEC2BIN(,2)`, `ERF(6,)`, or a LET or LAMBDA name bound to nothing): such an
    /// argument reads as `#N/A` at its own turn, inside the lift, so an earlier argument's error
    /// wins (`COMPLEX(1/0,)` and `ERF(1/0,)` are `#DIV/0!`, `DEC2BIN(,1/0)` too, as DEC2BIN reads
    /// places first) and a lifted call gives it per element (`ROWS(COMPLEX({1;2},))` is 2). This is
    /// T-4870's date-function rule ([`written_empty_or`]). A later argument written empty is
    /// dropped, read as omitted (`DEC2BIN(5,)` is "101", `COMPLEX(1,1,)` "1+i",
    /// `LET(e,,GESTEP(1,e))` 1). A call with more than `max` arguments is refused first, empty or
    /// not: Excel refuses it, so it is `#ERROR!` (syntax.md "Minimum argument counts";
    /// `DEC2BIN(5,,)`, `COMPLEX(1,1,,)`). Evidence: probes/t4872-atp-empty-argument and
    /// t4872-r2-order-and-lift (excel-graph); engineering.md "An empty argument of the `ToolPak`
    /// engineering functions" (T-4872).
    pub(super) fn atp_lift_call(
        &mut self,
        kind: Function,
        args: &[Node],
        (strict, max): (usize, usize),
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
    ) -> CalcResult {
        if args.len() > max {
            return CalcResult::new_args_number_error(cell);
        }
        let mut end = args.len();
        while end > strict && args.get(end - 1).is_some_and(|a| self.is_empty_argument(a)) {
            end -= 1;
        }
        let args = args.get(..end).unwrap_or(args);
        let empty: Vec<bool> = (args.iter().take(strict))
            .map(|a| self.is_empty_argument(a))
            .collect();
        if !empty.contains(&true) {
            return self.lift_call(kind, args, cell, kernel);
        }
        let na = written_empty_na(cell);
        self.lift_call(kind, args, cell, &mut |model, values| {
            let values: Vec<CalcResult> = (values.iter().enumerate())
                .map(|(index, value)| {
                    if empty.get(index).copied().unwrap_or(false) {
                        na.clone()
                    } else {
                        value.clone()
                    }
                })
                .collect();
            kernel(model, &values)
        })
    }
}

/// The argument past which Excel's engineering functions treat `exp` as overflowing, a constant in
/// Excel's code below the true overflow of `exp` at `ln(DBL_MAX)` = 709.7827 (EXP(709.75) itself is
/// finite). BESSELI is `#NUM!` at every order once |x| exceeds it (`BESSELI(709.7,0)` answers,
/// `BESSELI(709.70001,0)` is `#NUM!`; probes/besseli-overflow-cut-finest), and IMSIN and IMCOS once
/// the imaginary part does, whatever the real part (`IMSIN("0.5+709.7i")` answers,
/// `IMSIN("0.5+709.700000000001i")` is `#NUM!`; probes-numeric/t3616-imsin-imcos-positive-cut-probe).
/// engineering.md "BESSELI overflow point" and "IMSIN and IMCOS positive overflow cut".
const ENGINEERING_EXP_CUT: f64 = 709.7;
