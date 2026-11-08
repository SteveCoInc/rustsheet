// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::match_same_arms,
    clippy::needless_pass_by_value
)]
// LINT-DEBT-END
use super::regression::{known_points, layout, solve_units, Grid};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{LiftCost, LiftOptions};
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

// ─── Least-squares solve ─────────────────────────────────────────────────────

/// A column whose remaining norm, once the columns kept before it are reflected out, is at most
/// this fraction of its own norm is collinear and dropped. The value is carried over from the
/// Gram-Schmidt filter the QR solve replaced; Excel's own cut-off is unprobed: TODO(T-2569).
const COLLINEAR_CUT: f64 = 1e-10;

/// Result of the core linear regression.
pub(super) struct RegResult {
    /// Coefficients in Excel output order: [`β_p`, β_{p-1}, ..., `β_1`, `β_0`]
    /// (last x column first, intercept last, or omitted if const=FALSE)
    pub(super) coeffs: Vec<f64>,
    /// Standard errors in the same order (only if stats=TRUE)
    std_errors: Option<Vec<f64>>,
    r_squared: Option<f64>,
    se_y: Option<f64>,
    f_stat: Option<f64>,
    df_res: Option<f64>,
    ss_reg: Option<f64>,
    ss_res: Option<f64>,
}

/// A centered, column-pivoted Householder QR of the design: the kept columns in pivot order,
/// the upper-triangular factor over them and `Qᵀy`.
struct Factored {
    /// Indices of the kept x columns, in pivot order.
    kept: Vec<usize>,
    /// `r[i][j]`: row i of the triangular factor at pivot column j (j >= i).
    r: Vec<Vec<f64>>,
    /// `Qᵀy`: its first `kept.len()` entries are the fitted part, the rest the residual.
    qty: Vec<f64>,
}

/// The sum of squares of `values` in order.
fn sum_of_squares(values: &[f64]) -> f64 {
    values.iter().map(|v| v * v).sum()
}

/// The position in `remaining` of the column whose rows from `step` on have the largest sum of
/// squares, the later one on a tie.
fn largest_remaining(columns: &[Vec<f64>], remaining: &[usize], step: usize) -> Option<usize> {
    remaining
        .iter()
        .enumerate()
        .map(|(position, &j)| (position, sum_of_squares(&columns[j][step..])))
        .fold(
            None,
            |best: Option<(usize, f64)>, (position, norm)| match best {
                Some((_, best_norm)) if best_norm > norm => best,
                _ => Some((position, norm)),
            },
        )
        .map(|(position, _)| position)
}

/// Householder QR with column pivoting of the n x p `columns` against `y`. `first`, when given,
/// is reflected first (the constant's column of ones); after it each step takes the remaining
/// column of largest norm (the later one on a tie) and drops every remaining column whose rest is
/// collinear ([`COLLINEAR_CUT`]). The reflector `v = x - alpha e1`, `alpha = -sign(head)·|x|`, has
/// `|v|² = new_head² + tail` with the unchanged tail summed on its own (not `|x|²` less the head's
/// square, which loses an exact fit's exact cancellation: logest-modes K170:K172, residual 0 and
/// F `#NUM!`), and reflects each target as `t -= (v·t)·(2/|v|²)·v`, the dot product summed in
/// row order from the head. These knobs are the structural fit of T-4975 (statistics.md "LINEST
/// and LOGEST solve by a centered, pivoted Householder QR (T-2690)").
fn householder_qr(mut columns: Vec<Vec<f64>>, mut y: Vec<f64>, first: Option<usize>) -> Factored {
    let n = y.len();
    let norms: Vec<f64> = columns.iter().map(|c| sum_of_squares(c).sqrt()).collect();
    let mut remaining: Vec<usize> = (0..columns.len()).collect();
    let mut kept = Vec::new();
    let mut r: Vec<Vec<f64>> = Vec::new();
    for step in 0..n {
        remaining.retain(|&j| {
            let rest = sum_of_squares(&columns[j][step..]).sqrt();
            norms[j] > 0.0 && rest > COLLINEAR_CUT * norms[j]
        });
        let forced = first
            .filter(|_| step == 0)
            .and_then(|f| remaining.iter().position(|&j| j == f));
        let Some(position) = forced.or_else(|| largest_remaining(&columns, &remaining, step))
        else {
            break;
        };
        let pivot = remaining.remove(position);
        let head = columns[pivot][step];
        // The reflector's unchanged tail, summed on its own.
        let tail = sum_of_squares(&columns[pivot][step + 1..]);
        let alpha = -(head * head + tail).sqrt().copysign(head);
        let new_head = head - alpha;
        let v_norm = new_head * new_head + tail;
        if v_norm == 0.0 {
            break;
        }
        let v_tail = columns[pivot][step + 1..].to_vec();
        let reflect = |target: &mut [f64]| {
            let dot = v_tail
                .iter()
                .zip(&target[1..])
                .fold(new_head * target[0], |sum, (a, b)| sum + a * b);
            let scale = dot * (2.0 / v_norm);
            target[0] -= scale * new_head;
            for (value, vi) in target[1..].iter_mut().zip(&v_tail) {
                *value -= scale * vi;
            }
        };
        // The pivot column is reflected too: its new head, not alpha, is R's diagonal, so a y
        // proportional to it gives the exact ratio (`LINEST(A12:A13,B12:B13)` is exactly
        // 0.5, -1.5, linest-modes K134:L134).
        reflect(&mut columns[pivot][step..]);
        let mut row = vec![0.0; columns.len()];
        row[pivot] = columns[pivot][step];
        for &j in &remaining {
            reflect(&mut columns[j][step..]);
            row[j] = columns[j][step];
        }
        reflect(&mut y[step..]);
        kept.push(pivot);
        r.push(row);
    }
    let r = r
        .iter()
        .map(|row| kept.iter().map(|&j| row[j]).collect())
        .collect();
    Factored { kept, r, qty: y }
}

/// Solves `R z = b` for the upper-triangular `r`, back to front.
fn back_substitute(r: &[Vec<f64>], b: &[f64]) -> Vec<f64> {
    let k = r.len();
    let mut z = vec![0.0; k];
    for i in (0..k).rev() {
        let partial: f64 = (i + 1..k).map(|j| r[i][j] * z[j]).sum();
        z[i] = (b[i] - partial) / r[i][i];
    }
    z
}

/// Solves `Rᵀ w = b` for the upper-triangular `r`, front to back.
fn forward_substitute(r: &[Vec<f64>], b: &[f64]) -> Vec<f64> {
    let k = r.len();
    let mut w = vec![0.0; k];
    for i in 0..k {
        let partial: f64 = (0..i).map(|j| r[j][i] * w[j]).sum();
        w[i] = (b[i] - partial) / r[i][i];
    }
    w
}

/// The mean of `values`, `Σx/n` in doubles.
fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

/// Linear regression for any number of predictors, as LINEST: a centered, column-pivoted
/// Householder QR (Microsoft documents LINEST as a QR since Excel 2003; findings
/// docs/research/excel-parity-findings-2026-11-07.md item 12). With a constant term the columns
/// and y are centered on their means and the intercept is `ȳ - Σ βj x̄j`. The pivot keeps the
/// largest-norm column first, so a collinear column is dropped with coefficient 0 and standard
/// error 0 and the degrees of freedom count only the kept ones: with E = 2A,
/// `LINEST(B2:B9,A2:E9,TRUE,TRUE)` keeps E and drops A (A's se 0, df 3, linest-modes K106), one
/// point drops its x (`LINEST(B15,A15,TRUE,TRUE)` is `{0,9; 0,0; 1,0; #NUM!,0; 0,0}`, K141), and
/// one point of four variables without a constant keeps its largest (285d87cea1aaa73c A293).
/// The statistics come from the transformed residual and the triangular factor: the residual
/// sum of squares is that of `Qᵀy` past the rank, and the regression sum is the total less it
/// (`LINEST(B2:B9,A2:A9,TRUE,TRUE)` gives 167.60023809523813, linest-modes K12, where the fitted
/// part of `Qᵀy` gives ...238). `None` is `#NUM!` (no point).
pub(super) fn linear_regression(
    y: &[f64],
    x: &[Vec<f64>],
    p: usize,
    const_term: bool,
    compute_stats: bool,
) -> Option<RegResult> {
    let n = y.len();
    if n == 0 {
        return None;
    }
    let mut columns: Vec<Vec<f64>> = (0..p)
        .map(|j| x.iter().map(|row| row[j]).collect())
        .collect();
    let x_means: Vec<f64> = if const_term {
        columns.iter().map(|c| mean(c)).collect()
    } else {
        vec![0.0; p]
    };
    let y_mean = if const_term { mean(y) } else { 0.0 };
    for (column, &m) in columns.iter_mut().zip(&x_means) {
        for v in column.iter_mut() {
            *v -= m;
        }
    }
    let centered_y: Vec<f64> = y.iter().map(|v| v - y_mean).collect();
    // With a constant the first reflector is the column of ones, against the centered columns
    // and y (logest-modes and growth-modes one-variable fits; statistics.md "LINEST and LOGEST
    // solve by a centered, pivoted Householder QR (T-2690)").
    let (kept, r, qty) = if const_term {
        let mut design = vec![vec![1.0; n]];
        design.extend(columns);
        let Factored { kept, r, qty } = householder_qr(design, centered_y.clone(), Some(0));
        let kept: Vec<usize> = kept.iter().skip(1).map(|j| j - 1).collect();
        let r: Vec<Vec<f64>> = r.iter().skip(1).map(|row| row[1..].to_vec()).collect();
        (kept, r, qty[1..].to_vec())
    } else {
        let Factored { kept, r, qty } = householder_qr(columns, centered_y.clone(), None);
        (kept, r, qty)
    };
    let rank = kept.len();
    let z = back_substitute(&r, &qty[..rank]);
    let mut beta = vec![0.0; p];
    for (&j, &b) in kept.iter().zip(&z) {
        beta[j] = b;
    }
    let intercept = if const_term {
        y_mean - beta.iter().zip(&x_means).map(|(b, m)| b * m).sum::<f64>()
    } else {
        0.0
    };
    // Excel output order: the last x column first, the intercept last (0 when const=FALSE).
    let mut coeffs: Vec<f64> = beta.iter().rev().copied().collect();
    coeffs.push(intercept);

    if !compute_stats {
        return Some(RegResult {
            coeffs,
            std_errors: None,
            r_squared: None,
            se_y: None,
            f_stat: None,
            df_res: None,
            ss_reg: None,
            ss_res: None,
        });
    }

    let ss_res = sum_of_squares(&qty[rank..]);
    let ss_tot = sum_of_squares(&centered_y);
    let ss_reg = ss_tot - ss_res;
    let df = (n - rank - usize::from(const_term)) as f64;
    let r_sq = if ss_tot > 0.0 {
        (1.0 - ss_res / ss_tot).clamp(0.0, 1.0)
    } else {
        1.0
    };
    // An exact fit with no degree of freedom left has se_y and standard errors 0
    // (`LINEST(A12:A13,B12:B13,TRUE,TRUE)`, two points, linest-modes K134:L138).
    let var_y = if df > 0.0 { ss_res / df } else { 0.0 };
    let se_y = var_y.sqrt();
    // F is `#NUM!` (not finite) for an exact fit or no degree of freedom (linest-modes K137,
    // K144, K249).
    let f_stat = if df > 0.0 {
        (ss_reg / rank as f64) / (ss_res / df)
    } else {
        f64::NAN
    };
    // se(βj)² = var_y·((RᵀR)⁻¹)jj, the squared norm of row j of R⁻¹; the intercept's adds
    // var_y·(1/n + |R⁻ᵀ x̄|²).
    let mut se = vec![0.0; p];
    for (i, &j) in kept.iter().enumerate() {
        let mut unit = vec![0.0; rank];
        unit[i] = 1.0;
        let row = forward_substitute(&r, &unit);
        se[j] = (var_y * sum_of_squares(&row)).sqrt();
    }
    let mut std_errors: Vec<f64> = se.iter().rev().copied().collect();
    std_errors.push(if const_term {
        let kept_means: Vec<f64> = kept.iter().map(|&j| x_means[j]).collect();
        let w = forward_substitute(&r, &kept_means);
        (var_y * (1.0 / n as f64 + sum_of_squares(&w))).sqrt()
    } else {
        f64::NAN
    });

    Some(RegResult {
        coeffs,
        std_errors: Some(std_errors),
        r_squared: Some(r_sq),
        se_y: Some(se_y),
        f_stat: Some(f_stat),
        df_res: Some(df),
        ss_reg: Some(ss_reg),
        ss_res: Some(ss_res),
    })
}

/// Build LINEST/LOGEST output array from a `RegResult`.
/// ncols = `coeffs.len()` (one column per coefficient)
fn build_output_array(reg: RegResult, stats: bool) -> CalcResult {
    let ncols = reg.coeffs.len();

    let row0: Vec<ArrayNode> = reg.coeffs.iter().map(|&v| ArrayNode::Number(v)).collect();

    if !stats {
        return CalcResult::Array(vec![row0]);
    }

    // Row 1: standard errors
    let row1: Vec<ArrayNode> = reg
        .std_errors
        .as_deref()
        .unwrap_or(&vec![f64::NAN; ncols])
        .iter()
        .map(|&v| {
            if v.is_nan() {
                ArrayNode::Error(Error::NA)
            } else {
                ArrayNode::Number(v)
            }
        })
        .collect();

    // Row 2: [R², se_y, #N/A, ...]
    let mut row2 = vec![ArrayNode::Error(Error::NA); ncols];
    row2[0] = ArrayNode::Number(reg.r_squared.unwrap_or(f64::NAN));
    if ncols > 1 {
        let se = reg.se_y.unwrap_or(f64::NAN);
        row2[1] = if se.is_nan() {
            ArrayNode::Error(Error::NA)
        } else {
            ArrayNode::Number(se)
        };
    }

    // Row 3: [F, df, #N/A, ...]
    let mut row3 = vec![ArrayNode::Error(Error::NA); ncols];
    row3[0] = match reg.f_stat {
        Some(f) if f.is_finite() => ArrayNode::Number(f),
        _ => ArrayNode::Error(Error::NUM),
    };
    if ncols > 1 {
        row3[1] = match reg.df_res {
            Some(df) if !df.is_nan() => ArrayNode::Number(df),
            _ => ArrayNode::Error(Error::NA),
        };
    }

    // Row 4: [SS_reg, SS_res, #N/A, ...]
    let mut row4 = vec![ArrayNode::Error(Error::NA); ncols];
    row4[0] = match reg.ss_reg {
        Some(v) if !v.is_nan() => ArrayNode::Number(v),
        _ => ArrayNode::Error(Error::NA),
    };
    if ncols > 1 {
        row4[1] = match reg.ss_res {
            Some(v) if !v.is_nan() => ArrayNode::Number(v),
            _ => ArrayNode::Error(Error::NA),
        };
    }

    CalcResult::Array(vec![row0, row1, row2, row3, row4])
}

// ─── Public function implementations ─────────────────────────────────────────

impl Model {
    /// LINEST (`log` false) or LOGEST (true) of `known_y, [known_x], [const], [stats]`: LOGEST is
    /// LINEST of ln(y) with the coefficients exponentiated; its stats rows are those of the
    /// log-linear fit. The checks, in Excel's order:
    /// 1. an argument that is an error value, in argument order: `LINEST({1;2},1/0,"x")` and
    ///    `LINEST(1/0,{1;3},"x")` are `#DIV/0!` (probes/t2455-transpose-single-value A4:A27), while
    ///    `LINEST(Data!C3:C7,Data!B5:B10,NA(),"1")` is `#N/A`, not the size `#REF!` (gen-m9
    ///    05d44f6dd6cfc567 A195), and so is `LINEST(Data!D10:E10,Data!B1:B7,<text cell>,<#N/A>)`
    ///    (bcb24385baafb2f0 A40; statistics.md "LINEST and LOGEST report error values first");
    /// 2. the shapes (`#REF!`, [`layout`]): `LINEST(Data!B11:E12,(BYROW(..)+HOUR(0.5)),2,-1)` is
    ///    `#REF!` though its x holds errors (gen-m9 05d44f6dd6cfc567 A317);
    /// 3. an error element of `known_y` or `known_x` (`#VALUE!`, [`Grid::has_error`]);
    /// 4. a const or stats that is text and no boolean (`#VALUE!`; after the shapes:
    ///    `LOGEST(Data!A10:B10,Data!B8:B8,-1,<"a" cell>)` and `LOGEST(Data!A10:A11,-0,
    ///    LAMBDA(acc,Data!B3)(..))` with B3 = " x " are `#REF!`, f34f66fb7f81e673 A179,
    ///    bbdfc06cae0e0d71 A324);
    /// 5. the data ([`known_points`]).
    ///
    /// An array const or stats lifts, as TREND's const: each element is the top-left value of the
    /// call with those flags (`MAKEARRAY(2,3,..)` as stats is `#SPILL!`, gen-m9 c2974c061384fdd9
    /// A89), so `LOGEST(Data!C$10,0.1,EXPAND(Data!A10:C11,2,4))` spills 2x4 (`#SPILL!` there,
    /// gen-m9 165db58d1540f014 A327).
    fn linest_or_logest(
        &mut self,
        args: &[Node],
        log: bool,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(1..=4).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let data = match self.read_regression_grids(args, cell) {
            Ok(data) => data,
            Err(error) => return error,
        };
        if let Err(error) = self.spend_work(solve_units(&data.0, &data.1), cell) {
            return error;
        }
        // An omitted flag is an empty argument: its default.
        let mut flag_arg = |index: usize| match args.get(index) {
            Some(node) => flag_input(self, node, cell),
            None => (CalcResult::EmptyArg, true),
        };
        let (const_value, const_typed) = flag_arg(2);
        let (stats_value, stats_typed) = flag_arg(3);
        let is_array = |value: &CalcResult| matches!(value, CalcResult::Array(_));
        if !is_array(&const_value) && !is_array(&stats_value) {
            let const_flag = flag_value(self, const_value, true, const_typed, cell);
            let stats_flag = flag_value(self, stats_value, false, stats_typed, cell);
            return regression_output(&data, const_flag, stats_flag, log, cell);
        }
        let (y, x) = &data;
        let cost =
            u64::try_from(y.elements.len().saturating_add(x.elements.len())).unwrap_or(u64::MAX);
        self.lift_values(
            vec![const_value, stats_value],
            LiftCost::Cells(cost),
            &LiftOptions::REUSE_ANY_EQUAL_TOP_LEFT,
            cell,
            &mut |model, values| {
                let const_flag = flag_value(model, values[0].clone(), true, true, cell);
                let stats_flag = flag_value(model, values[1].clone(), false, true, cell);
                regression_output(&data, const_flag, stats_flag, log, cell)
            },
        )
    }

    /// Reads `known_y` and `known_x` (counting 1, 2, 3, ... in y's shape when omitted or empty),
    /// stopping at the first that is an error value.
    fn read_regression_grids(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(Grid, Grid), CalcResult> {
        let (y, x) = self.read_known_y_and_x(args, cell)?;
        let y = Grid::from_data(y);
        let x = match x {
            Some(x) => Grid::from_data(x),
            None => Grid::counting(y.rows, y.cols),
        };
        Ok((y, x))
    }

    // LINEST(known_y's, [known_x's], [const], [stats])
    pub(crate) fn fn_linest(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.linest_or_logest(args, false, cell)
    }

    // LOGEST(known_y's, [known_x's], [const], [stats])
    // Fits y = b * m1^x1 * m2^x2 * ... via linear regression on ln(y).
    pub(crate) fn fn_logest(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.linest_or_logest(args, true, cell)
    }
}

/// A const or stats flag: `Err` is an error value, or typed or computed text that is no boolean
/// (step 1 of [`Model::linest_or_logest`]), `Ok(Err)` a text cell that is no boolean (step 4).
type Flag = Result<Result<bool, CalcResult>, CalcResult>;

/// LINEST's or LOGEST's array for one const (steps 1 to 5 of [`Model::linest_or_logest`]).
fn regression_output(
    data: &(Grid, Grid),
    const_term: Flag,
    stats: Flag,
    log: bool,
    cell: CellReferenceIndex,
) -> CalcResult {
    let run = || -> Result<CalcResult, CalcResult> {
        let (y, x) = data;
        let const_term = const_term?;
        let stats = stats?;
        let layout = layout(y, x, cell)?;
        if y.has_error() || x.has_error() {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Regression: error value in known_y or known_x".to_string(),
            ));
        }
        let (const_term, stats) = (const_term?, stats?);
        let known = known_points(y, x, layout, log, cell)?;
        let Some(mut reg) = linear_regression(&known.ys, &known.points, known.p, const_term, stats)
        else {
            return Err(CalcResult::new_error(
                Error::NUM,
                cell,
                "Regression: singular matrix".to_string(),
            ));
        };
        if log {
            for c in &mut reg.coeffs {
                *c = c.exp();
            }
        }
        Ok(build_output_array(reg, stats))
    };
    run().unwrap_or_else(|error| error)
}

/// The value of a const or stats argument, and whether it is typed (true) or the value of the
/// cell it refers to, a LAMBDA's result included (`LAMBDA(acc,Data!B3)(..)` with B3 = " x " waits
/// for the shapes, gen-m9 bbdfc06cae0e0d71 A324). A range is its top-left cell
/// ([`flag_range_value`]).
fn flag_input(model: &mut Model, node: &Node, cell: CellReferenceIndex) -> (CalcResult, bool) {
    match model.evaluate_node_with_reference(node, cell) {
        range @ CalcResult::Range { .. } => (flag_range_value(model, range), false),
        value => (value, true),
    }
}

/// What a range given as LINEST's, LOGEST's, TREND's or GROWTH's const or stats reads as: the
/// value of its top-left cell, whatever the range's size, the other cells, or the formula's row.
/// It does not lift and is no implicit intersection: `LINEST(y,x,TRUE,Data!Z100:Z102)` in row
/// 101 with Z100:Z102 = 1, 0, 1 is the 5x2 stats array, `TREND(y,x,6,Data!Z124:Z125)` with
/// 1, `#N/A` is the TRUE call, and a text top-left cell is `#VALUE!`
/// (probes/t4222-regression-flag-ranges A101, A124, A130; statistics.md "A range as const or
/// stats reads its top-left cell"). Any other value is itself.
pub(super) fn flag_range_value(model: &mut Model, value: CalcResult) -> CalcResult {
    match value {
        CalcResult::Range { left, .. } => model.evaluate_cell(left),
        value => value,
    }
}

/// The [`Flag`] of a const or stats value. Text that is no boolean is `#VALUE!`
/// (`LINEST(B2:B9,A2:A9,"x")`, `LINEST(B2:B9,A2:A9,TRUE,"x")`, linest-modes K155, K162;
/// `LINEST({1;2},{1;3},"x")`, probes/t2455-transpose-single-value A7, A26): at once when `typed`
/// (not a cell reference: `LINEST(UNIQUE(Data!D9:E9),Data!B7,TRUE,"1")` and
/// `LOGEST(Data!C6:E9,Data!D$6,1,"")` are `#VALUE!` against unequal shapes, gen-m9
/// a013a45a8fbcab4f A276, fd9fa22d212080d6 A201), else after the shapes.
fn flag_value(
    model: &mut Model,
    value: CalcResult,
    default: bool,
    typed: bool,
    cell: CellReferenceIndex,
) -> Flag {
    match value {
        CalcResult::EmptyArg => Ok(Ok(default)),
        // A blank cell is FALSE, not the default: `LINEST(y,x,Data!Z151,FALSE)` with Z151 blank
        // fits through the origin (probes/t4222-regression-flag-ranges A154).
        CalcResult::EmptyCell => Ok(Ok(false)),
        CalcResult::Boolean(b) => Ok(Ok(b)),
        CalcResult::Number(n) => Ok(Ok(n != 0.0)),
        err @ CalcResult::Error { .. } => Err(err),
        text @ CalcResult::String(_) => match model.cast_to_bool(text, cell) {
            Err(error) if typed => Err(error),
            flag => Ok(flag),
        },
        _ => Ok(Ok(default)),
    }
}

/// Reconstruct internal beta vector [`β_0`, `β_1`, ..., `β_p`] from Excel-order coefficients.
pub(super) fn reconstruct_beta(coeffs: &[f64], const_term: bool, p: usize) -> Vec<f64> {
    // coeffs is: [β_p, β_{p-1}, ..., β_1, β_0] (if const) or [β_p, ..., β_1]
    let mut beta = vec![0.0_f64; p + usize::from(const_term)];
    if const_term {
        // intercept is the last element of coeffs
        beta[0] = *coeffs.last().unwrap_or(&0.0);
        // x coefficients are reversed
        for j in 0..p {
            beta[j + 1] = coeffs[p - 1 - j];
        }
    } else {
        for j in 0..p {
            beta[j] = coeffs[p - 1 - j];
        }
    }
    beta
}

/// Predict y for a single x row given beta in internal order.
pub(super) fn predict(beta: &[f64], x_row: &[f64], const_term: bool) -> f64 {
    let mut pred = if const_term { beta[0] } else { 0.0 };
    let x_start = usize::from(const_term);
    for (j, &xj) in x_row.iter().enumerate() {
        if x_start + j < beta.len() {
            pred += beta[x_start + j] * xj;
        }
    }
    pred
}

#[cfg(test)]
mod tests {
    use super::linear_regression;

    // Excel's values (probes/linest-modes, excel-graph): the QR solve's exact zeros and noise.
    #[test]
    fn linest_solves_by_centered_pivoted_qr() {
        // K134:L134, `LINEST(A12:A13,B12:B13,TRUE,TRUE)`: the two-point fit is exactly 0.5, -1.5.
        let reg = linear_regression(&[1.0, 2.0], &[vec![5.0], vec![7.0]], 1, true, true);
        assert_eq!(reg.map(|r| r.coeffs), Some(vec![0.5, -1.5]));
        // K183:L183, `LINEST(D2:D9,A2:A9,TRUE,TRUE)`.
        let x: Vec<Vec<f64>> = (1..=8).map(|v| vec![f64::from(v)]).collect();
        let y = [3.0, 5.5, 9.0, 16.0, 27.0, 50.0, 88.0, 160.0];
        let reg = linear_regression(&y, &x, 1, true, true);
        assert_eq!(
            reg.map(|r| r.coeffs),
            Some(vec![19.589_285_714_285_71, -43.339_285_714_285_71])
        );
        // ironcalc-calc LINEST_LOGEST.xlsx D2:E2 (Excel-cached), `LINEST(A2:A6,B2:B6)`.
        let x: Vec<Vec<f64>> = [1.1, 1.9, 3.1, 4.05, 5.0]
            .iter()
            .map(|&v| vec![v])
            .collect();
        let reg = linear_regression(&[0.2, 0.41, 0.62, 0.8, 1.1], &x, 1, true, false);
        assert_eq!(
            reg.map(|r| r.coeffs),
            Some(vec![0.219_288_879_935_535_92, -0.038_445_306_204_673_68])
        );
    }
}
