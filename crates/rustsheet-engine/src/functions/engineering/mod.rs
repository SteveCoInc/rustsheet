mod bessel;
mod bit_operations;
mod complex;
mod convert;
mod misc;
mod number_basis;
pub(crate) mod transcendental;

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

impl Model {
    /// The arguments of a fixed-arity Analysis `ToolPak` function whose first `strict` arguments
    /// may not be written empty: one of them that is (`DEC2BIN(,2)`, `ERF(6,)`, or a LET or
    /// LAMBDA name bound to nothing) is `#N/A`, and a later one written empty is dropped, read as
    /// omitted (`DEC2BIN(5,)` is "101", `COMPLEX(1,1,)` "1+i", `LET(e,,GESTEP(1,e))` 1). A call
    /// with more than `max` arguments is refused first, empty or not: Excel refuses it, so it is
    /// `#ERROR!` (syntax.md "Minimum argument counts"; `DEC2BIN(5,,)`, `COMPLEX(1,1,,)`)
    /// (probes/t4872-atp-empty-argument, excel-graph; engineering.md, "An empty argument of the
    /// `ToolPak` engineering functions"; T-4872).
    fn atp_arguments<'a>(
        &self,
        args: &'a [Node],
        (strict, max): (usize, usize),
        cell: CellReferenceIndex,
    ) -> Result<&'a [Node], CalcResult> {
        if args.len() > max {
            return Err(CalcResult::new_args_number_error(cell));
        }
        if args.iter().take(strict).any(|a| self.is_empty_argument(a)) {
            return Err(CalcResult::new_error(Error::NA, cell, String::new()));
        }
        let mut end = args.len();
        while end > strict && args.get(end - 1).is_some_and(|a| self.is_empty_argument(a)) {
            end -= 1;
        }
        Ok(args.get(..end).unwrap_or(args))
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
