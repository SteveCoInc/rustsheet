//! Excel's incomplete beta kernel K(a, b, x, y): Didonato and Morris's TOMS 708 `bratio`, the 1992
//! Fortran, ported one branch at a time (SPEC-0025; docs/excel-semantics/statistics.md "Incomplete beta
//! is TOMS 708 `bratio`"). This file is the dispatch: which branch bratio takes for a point, after
//! its swap, and the branches ported so far ([`ported_tails`]): bfrac ([`bfrac`]) for b0 >= 40, bpser ([`bpser`]) for
//! w = bpser(a0, b0, x0), w1 = bpser(b0, a0, y0) and min(a, b) > 1, L140's bup + bpser
//! ([`bup`]), and basym ([`basym`]) for two shapes above 100 near the mean. The line-by-line
//! model is `docs/research/t2464/r2-bratio/bratio_excel.py` `kernel`; every branch not ported yet
//! keeps today's kernel (`incomplete_beta.rs`, SPEC-0025's migration).

mod basym;
mod bfrac;
mod bgrat;
pub(super) mod bpser;
mod bup;
pub(super) mod ln_beta;

use super::incomplete_beta::{beta_reg_tails, closed_form_tails, half_half_cdf, search_tails};
use super::large_shape_beta::large_shape_lower;
use crate::arithmetic::extended_sum;
use crate::x87_exp::dd_add;
use basym::basym;
use bfrac::bfrac;
pub(crate) use bfrac::past_evidenced_shapes;
use bgrat::{bgrat_half, bgrat_increment};
use bpser::bpser;
use bup::{
    bup, bup_takes_terms, l140_bup_bgrat, l140_bup_bpser, l140_bup_prefactor_underflows,
    l140_bup_sum_flushed, l140_bup_sum_overflows, BupSum,
};

/// TOMS 708's `eps` in `bratio`: max(2^-52, 1e-15).
pub(super) const EPS: f64 = 1e-15;

/// bratio's switch to w1 = bpser(b0, a0, y0) for min(a, b) <= 1: the 1992 Fortran's 0.3, not R's 0.29
/// (probes-numeric t2464-r2-bratio-x29: Excel takes the 0.3 dispatch on 284 of the 434 cells in
/// [0.29, 0.3) where the two differ, the 0.29 one on 87).
const W1_BPSER_MIN_X: f64 = 0.3;

/// bratio's tolerance for bfrac, 15·eps.
const BFRAC_EPS_FACTOR: f64 = 15.0;

/// bratio's tolerance for basym, 100·eps.
const BASYM_EPS_FACTOR: f64 = 100.0;

/// The bup terms bratio takes before bgrat for b0 <= 15 (its `n = 20`).
const BUP_BGRAT_TERMS: f64 = 20.0;

/// The branch bratio computes a point with (the model's names; `bratio_excel.py` `kernel`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Branch {
    /// x = 0: (0, 1).
    Zero,
    /// y = 0: (1, 0).
    One,
    /// max(a, b) < eps/1000: (b/(a+b), a/(a+b)).
    Tiny,
    Fpser,
    Apser,
    /// w = bpser(a0, b0, x0), for min(a, b) <= 1.
    WBpser,
    /// w1 = bpser(b0, a0, y0), for min(a, b) <= 1 and x0 >= 0.3.
    W1Bpser,
    /// w1 = bup(b0, a0, y0, x0, 20) then bgrat from b0 + 20.
    BupBgrat,
    /// w1 = bgrat(b0, a0, y0, x0) for b0 > 15.
    Bgrat,
    /// w = bpser(a0, b0, x0) for min(a, b) > 1 and b0·x0 <= 0.7.
    BpserLarge,
    L140BupBpser,
    L140BupBgrat,
    L140BupBupBgrat,
    Bfrac,
    Basym,
}

/// A point after bratio's swap: the oriented shapes and arguments the branch computes with, and
/// whether they are (b, a, y, x), in which case the branch's w is the caller's w1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Dispatch {
    pub branch: Branch,
    pub swapped: bool,
    pub a0: f64,
    pub b0: f64,
    pub x0: f64,
    pub y0: f64,
    /// bratio's |λ| for min(a, b) > 1 (bfrac's and basym's argument), 0 elsewhere.
    pub lambda: f64,
}

/// bratio's dispatch for K(a, b, x, y), x and y formed by the caller; `None` off bratio's domain
/// (a non-positive or non-finite shape, x or y outside [0, 1], both shapes 0). The closed forms
/// (`incomplete_beta::closed_form_tails`, T-2502) are tried by the caller before this.
pub(crate) fn dispatch(a: f64, b: f64, x: f64, y: f64) -> Option<Dispatch> {
    let in_unit = |v: f64| (0.0..=1.0).contains(&v);
    if !(a.is_finite() && b.is_finite() && a >= 0.0 && b >= 0.0 && a + b > 0.0)
        || !(in_unit(x) && in_unit(y))
    {
        return None;
    }
    let plain = |branch| Dispatch {
        branch,
        swapped: false,
        a0: a,
        b0: b,
        x0: x,
        y0: y,
        lambda: 0.0,
    };
    if x == 0.0 {
        return Some(plain(Branch::Zero));
    }
    if y == 0.0 {
        return Some(plain(Branch::One));
    }
    if a.max(b) < EPS * 0.001 {
        return Some(plain(Branch::Tiny));
    }
    if a.min(b) <= 1.0 {
        let swapped = x > 0.5;
        let (a0, b0, x0, y0) = if swapped { (b, a, y, x) } else { (a, b, x, y) };
        return Some(Dispatch {
            branch: small_shape_branch(a0, b0, x0),
            swapped,
            a0,
            b0,
            x0,
            y0,
            lambda: 0.0,
        });
    }
    // λ as TOMS forms it, so its sign (the swap) matches bit for bit.
    let lambda = if a > b {
        (a + b) * y - b
    } else {
        a - (a + b) * x
    };
    let swapped = lambda < 0.0;
    let (a0, b0, x0, y0) = if swapped { (b, a, y, x) } else { (a, b, x, y) };
    let lambda = lambda.abs();
    let branch = if b0 < 40.0 {
        if b0 * x0 <= 0.7 {
            Branch::BpserLarge
        } else if x0 <= 0.7 {
            Branch::L140BupBpser
        } else if a0 <= 15.0 {
            Branch::L140BupBupBgrat
        } else {
            Branch::L140BupBgrat
        }
    } else if (a0 > b0 && (b0 <= 100.0 || lambda > b0 * 0.03))
        || (a0 <= b0 && (a0 <= 100.0 || lambda > a0 * 0.03))
    {
        Branch::Bfrac
    } else {
        Branch::Basym
    };
    Some(Dispatch {
        branch,
        swapped,
        a0,
        b0,
        x0,
        y0,
        lambda,
    })
}

/// Whether Excel's incomplete beta K(a, b, x, y) is `#NUM!` because bratio's L140 bup sum overflows
/// ([`bup::l140_bup_sum_overflows`]): a huge shape beside one below 40, as in
/// `NEGBINOM.DIST(28,1E+13,0.323212154487823,TRUE)` and `BETA.DIST(0.323212154487823,1E+13,29,TRUE)`
/// (T-3576). Only BETA.DIST's and NEGBINOM.DIST's cdfs are probed there and ask it.
pub(crate) fn l140_overflows(a: f64, b: f64, x: f64, y: f64) -> bool {
    dispatch(a, b, x, y).is_some_and(|d| {
        matches!(d.branch, Branch::L140BupBpser | Branch::L140BupBgrat)
            && l140_bup_sum_overflows(d.a0, d.b0, d.x0, d.y0)
    })
}

/// bratio's branch for min(a, b) <= 1 on the oriented point (x0 <= 0.5).
fn small_shape_branch(a0: f64, b0: f64, x0: f64) -> Branch {
    if b0 < EPS.min(EPS * a0) {
        return Branch::Fpser;
    }
    if a0 < EPS.min(EPS * b0) && b0 * x0 <= 1.0 {
        return Branch::Apser;
    }
    // The two powf here only pick a branch (against 0.7 and 0.9); bratio's own pow may differ
    // from libm's by an ulp right at those cuts.
    if a0.max(b0) > 1.0 {
        if b0 <= 1.0 {
            return Branch::WBpser;
        }
        if x0 >= W1_BPSER_MIN_X {
            return Branch::W1Bpser;
        }
        if x0 < 0.1 && (x0 * b0).powf(a0) <= 0.7 {
            return Branch::WBpser;
        }
        if b0 > 15.0 {
            return Branch::Bgrat;
        }
    } else {
        if a0 >= 0.2_f64.min(b0) || x0.powf(a0) <= 0.9 {
            return Branch::WBpser;
        }
        if x0 >= W1_BPSER_MIN_X {
            return Branch::W1Bpser;
        }
    }
    Branch::BupBgrat
}

/// BETA.DIST's cdf, w of K(a, b, x, y) with y = fl(1 - x) formed by the caller (SPEC-0025's caller
/// table): [`kernel_tails`]'s w, plus bratio's bgrat branches at a0 = 1/2 (T-2509's `bgrat_half`)
/// and bup + bgrat at any other a0 < 1 (T-3665),
/// and Excel's arcsine form at a = b = 1/2 (`incomplete_beta::half_half_cdf`, T-3192).
/// Those are a net visible gain for BETA.DIST (T-2510: 30 fewer visibly-off cells, none more), not
/// for F (see [`t_kernel_tails`]), so F's callers (`fisher.rs`) keep [`kernel_tails`].
/// BETA.INV's search (`beta.rs`) crosses it where it is closed or ported ([`beta_search_tails`]).
pub(crate) fn beta_dist_lower_tail(x: f64, y: f64, a: f64, b: f64) -> f64 {
    if a == 0.5 && b == 0.5 && x > 0.0 && y > 0.0 {
        return half_half_cdf(x, y);
    }
    kernel_tails_with(x, y, a, b, true).0
}

/// BETA.DIST's worksheet cdf: [`beta_dist_lower_tail`], and Excel's exact 0 where L140 bup+bgrat's
/// prefactor underflows ([`l140_zero_tails`], T-4045's rule). Column A of probes-numeric
/// t3568-r5-bup-tune, -hold and -fresh has 47 Excel zeros where the true tail is up to 2E-257
/// (`BETA.DIST(0.7115070068952,2160.0,30.5,TRUE)`, t3568-r5-bup-fresh A291): the rule gives all 47
/// (28 before), zeroes none of the column's Excel numbers, and gives the 44 zeros of
/// probes/t4254-beta-t-l140-zero-cut below `ln(DBL_MIN)` (T-3764). The quantile searches keep
/// [`beta_dist_lower_tail`]: with the 0 under F.INV's search
/// `F.INV(1.43795578761177E-262,107702,58)` (accuracy/F.INV/s1 A146) leaves Gate A's envelope.
/// At n >= 3 bup terms it is also 0 where bup's sum W is below 2^-970 ([`l140_bup_sum_flushed`],
/// T-4437).
pub(crate) fn beta_dist_cdf(x: f64, y: f64, a: f64, b: f64) -> f64 {
    l140_zero_tails(a, b, x, y, beta_dist_l140_zero)
        .map_or_else(|| beta_dist_lower_tail(x, y, a, b), |t| t.0)
}

/// BETA.DIST's worksheet cdf is Excel's exact 0 on L140 bup+bgrat where bup's prefactor underflows
/// (T-4045) or, at n >= 3 bup terms, where bup's sum is below 2^-970 (T-4437).
fn beta_dist_l140_zero(a0: f64, b0: f64, x0: f64, y0: f64) -> bool {
    l140_bup_prefactor_underflows(a0, b0, x0, y0) || l140_bup_sum_flushed(a0, b0, x0, y0)
}

/// (w, w1) of BETA.DIST's kernel ([`beta_dist_lower_tail`]) where it is closed or ported, the tails
/// BETA.INV's search crosses (T-3398). On the branches not ported yet BETA.DIST sums the accurate
/// half-integer series (thousands of double-double terms near x = 1: 0.32 ms a cdf at
/// a = 170, b = 1.5, x = 0.99), so the search takes the bounded Cephes kernel there instead
/// (`incomplete_beta::search_tails`, at most two 300-step fractions), as it did before T-3398:
/// 129 cdfs of the series would cost 8 ms an element, past BETA.INV's lift weight.
///
/// L140 bup+bgrat with n >= 2 bup terms or c = 1 (T-3561) is BETA.DIST's only: there the search
/// keeps the Cephes kernel, because crossing the new cdf moves gated ironcalc-calc
/// `BETADIST_BETAINV.xlsx` H28 (`BETA.INV` at shapes 6 and 1000) from Excel's bit to 1 ulp off
/// (T-3867 owns the search there). On L140 bup+bpser the search crosses bup's recurrence, not the
/// power sum BETA.DIST takes (T-3198): the power sum moves gated H32 (`BETA.INV` at shapes 5 and 5,
/// 0.555) to 0.5550000000000002.
///
/// Branches that are not ported fall back to the bounded Cephes `search_tails`; bup + bgrat at
/// a0 < 1 is ported (T-3665), so the search runs it too.
pub(crate) fn beta_search_tails(x: f64, y: f64, a: f64, b: f64) -> (f64, f64) {
    let ported = || {
        let many_terms = dispatch(a, b, x, y)
            .is_some_and(|d| d.branch == Branch::L140BupBgrat && bup_takes_terms(d.b0));
        if many_terms {
            None
        } else {
            ported_tails_with(a, b, x, y, true, BupSum::Recurrence)
        }
    };
    closed_form_tails(x, y, a, b)
        .or_else(ported)
        .unwrap_or_else(|| search_tails(x, y, a, b))
}

/// (w, w1) of K(a, b, x, y), x and y formed by the caller per SPEC-0025's caller table:
/// [`closed_or_ported_tails`], and today's kernel (`incomplete_beta::beta_reg_tails`) for every
/// branch not ported yet.
pub(crate) fn kernel_tails(x: f64, y: f64, a: f64, b: f64) -> (f64, f64) {
    kernel_tails_with(x, y, a, b, false)
}

/// [`kernel_tails`], with bratio's bgrat branches at a0 = 1/2 when `half_bgrat`.
fn kernel_tails_with(x: f64, y: f64, a: f64, b: f64, half_bgrat: bool) -> (f64, f64) {
    closed_or_ported_with(x, y, a, b, half_bgrat).unwrap_or_else(|| beta_reg_tails(x, y, a, b))
}

/// Excel's closed forms, then the port's branches (with bgrat at a0 = 1/2 when `half_bgrat`);
/// `None` where neither applies and the caller picks its fallback kernel.
fn closed_or_ported_with(x: f64, y: f64, a: f64, b: f64, half_bgrat: bool) -> Option<(f64, f64)> {
    closed_form_tails(x, y, a, b)
        .or_else(|| ported_tails_with(a, b, x, y, half_bgrat, BupSum::Powers))
}

/// (w, w1) of K(a, b, x, y), x and y formed by the caller per SPEC-0025's caller table: Excel's
/// closed forms first (`incomplete_beta::closed_form_tails`, T-2502), then the branches of the
/// port ([`ported_tails`]). `None` where neither applies, and the caller keeps today's kernel.
pub(crate) fn closed_or_ported_tails(x: f64, y: f64, a: f64, b: f64) -> Option<(f64, f64)> {
    closed_or_ported_with(x, y, a, b, false)
}

/// [`closed_or_ported_tails`] for the F tails (`fisher.rs`), with bgrat at b = 1/2 as the Student
/// t's and BETA.DIST's callers take it: on F's own arguments (`fisher.rs` `f_beta_arguments`) a df
/// of 1 reaches Excel's bits there (T-3580: probes-numeric t3567-r5-fdf1-hold's d1 = 1 bgrat
/// region 991 of 1,000 bit-exact, against 298 on today's kernel; statistics.md "F's
/// incomplete-beta arguments keep r >= 1"). T-2510 kept F off bgrat because of the arguments it
/// was handed, not the kernel.
pub(crate) fn f_closed_or_ported_tails(x: f64, y: f64, a: f64, b: f64) -> Option<(f64, f64)> {
    l140_zero_tails(a, b, x, y, l140_bup_prefactor_underflows)
        .or_else(|| closed_or_ported_with(x, y, a, b, true))
}

/// The tails (lower, upper) of K(a, b, x, y) where L140 bup+bgrat's w is Excel's exact 0 by `zero`,
/// read at bratio's own (a0, b0, x0, y0): w is 0 and its complement 1, oriented back through
/// bratio's swap. F's tails take bup's prefactor underflow
/// ([`bup::l140_bup_prefactor_underflows`], T-4045); BETA.DIST's worksheet cdf
/// ([`beta_dist_cdf`]) takes [`beta_dist_l140_zero`]; the quantile searches take neither.
fn l140_zero_tails(
    a: f64,
    b: f64,
    x: f64,
    y: f64,
    zero: fn(f64, f64, f64, f64) -> bool,
) -> Option<(f64, f64)> {
    let d = dispatch(a, b, x, y)?;
    (d.branch == Branch::L140BupBgrat && zero(d.a0, d.b0, d.x0, d.y0)).then_some(if d.swapped {
        (1.0, 0.0)
    } else {
        (0.0, 1.0)
    })
}

/// (w, w1) of K(a, b, x, y) on the branches ported so far, `None` elsewhere. bratio computes one
/// tail and forms the other as (0.5 - w) + 0.5, each step an x87 op stored to a double
/// (statistics.md "Incomplete beta is TOMS 708 `bratio`").
///
/// A shape of 1 outside the closed forms (a0 = 1 with y0 > 0.9) stays on
/// `incomplete_beta::shape_one_tails`: bpser(1, b0, x0) is further off Excel's w there (T-3007).
pub(super) fn ported_tails(a: f64, b: f64, x: f64, y: f64) -> Option<(f64, f64)> {
    ported_tails_with(a, b, x, y, false, BupSum::Powers)
}

/// (w, w1) of the Student t kernel call K(n/2, 1/2, y, z) ([`ported_tails`] plus bgrat at b = 1/2,
/// T-2509). The Student t tails, BETA.DIST ([`beta_dist_lower_tail`]) and F's tails
/// ([`f_closed_or_ported_tails`], T-3580) take bgrat at b = 1/2; the binomial family and F.TEST
/// ([`kernel_tails`]) do not.
pub(super) fn t_kernel_tails(a: f64, y: f64, z: f64) -> Option<(f64, f64)> {
    ported_tails_with(a, 0.5, y, z, true, BupSum::Powers)
}

/// [`ported_tails`], with bratio's bgrat branches at a0 = 1/2, bup + bgrat at other a0 < 1
/// (BETA.DIST's cdf, F's tails and BETA.INV's search, T-3665) and L140 bup+bgrat (BETA.DIST's,
/// the Student t's and F's callers) when `half_bgrat`, and L140 bup+bpser's sum formed as
/// `bup_sum` says.
fn ported_tails_with(
    a: f64,
    b: f64,
    x: f64,
    y: f64,
    half_bgrat: bool,
    bup_sum: BupSum,
) -> Option<(f64, f64)> {
    if a == 1.0 || b == 1.0 {
        return None;
    }
    let d = dispatch(a, b, x, y)?;
    let other = half_complement;
    let (w, w1) = match d.branch {
        // Both shapes below eps/1000: bratio's w = b/(a + b), w1 = a/(a + b) (TOMS 708 L230).
        // Excel's cdf to the bit on all eleven Gate A points there (accuracy/BETA.DIST/s1 A184,
        // A187, A193; BETADIST/s1 A183, A187, A193, A198; BETA.DIST/r4-T-4437 A185, A189, A192,
        // A194), where the kernel this replaces gave 0 (T-3577).
        Branch::Tiny => (b / (a + b), a / (a + b)),
        Branch::WBpser | Branch::BpserLarge => {
            let w = bpser(d.a0, d.b0, d.x0)?;
            (w, other(w))
        }
        Branch::W1Bpser => {
            let w1 = bpser(d.b0, d.a0, d.y0)?;
            (other(w1), w1)
        }
        // bgrat's b is a0: only b = 1/2 is ported (T-2509; T-3582 owns the rest).
        Branch::Bgrat if half_bgrat && d.a0 == 0.5 => {
            let w1 = bgrat_half(d.b0, d.y0, d.x0, 0.0)?;
            (other(w1), w1)
        }
        Branch::BupBgrat if half_bgrat && d.a0 == 0.5 => {
            let up = bup(d.b0, d.a0, d.y0, d.x0, BUP_BGRAT_TERMS)?;
            let w1 = bgrat_half(d.b0 + BUP_BGRAT_TERMS, d.y0, d.x0, up)?;
            (other(w1), w1)
        }
        // Any other a0 < 1: the 20 bup terms plus the accurate bgrat increment (T-3665). Excel is
        // within 1.7 ulp of the math there, the kernel this replaces 4 to 6 ulp off.
        Branch::BupBgrat if half_bgrat => {
            let up = bup(d.b0, d.a0, d.y0, d.x0, BUP_BGRAT_TERMS)?;
            let increment = bgrat_increment(d.b0 + BUP_BGRAT_TERMS, d.a0, d.y0, d.x0)?;
            let w1 = dd_add((up, 0.0), increment).0;
            (other(w1), w1)
        }
        // n bup terms, 1 < b0 < 40 (T-3382, T-3561); BETA.DIST's callers only, as at b = 1/2.
        Branch::L140BupBgrat if half_bgrat => {
            let w = l140_bup_bgrat(d.a0, d.b0, d.x0, d.y0)?;
            (w, other(w))
        }
        Branch::L140BupBpser => {
            let w = l140_bup_bpser(d.a0, d.b0, d.x0, d.y0, bup_sum)?;
            (w, other(w))
        }
        // A huge a0 beside b0 of 40 to 200 takes the large-shape expansion, not bfrac (T-4435): bfrac's
        // prefactor is Excel's density at x0, which loses digits as a0 grows, and on the swapped call
        // (the F upper tail) x0 is the caller's rounded complement near 1, which a0 multiplies.
        // `FDIST(1.5,80,1E10)` is 1 - I(40, 5E9) there, 4.7E-8 off the math by bfrac
        // (probes-numeric/t4045-f-dist-rt-zero-cut E3); the expansion reads ln x0 from y0. Excel is
        // itself up to 8.5E-9 off the math on those calls (T-4434's sweep, T-4214), so this is the
        // `accuracy` stage, not Excel's arithmetic. BETA.DIST's, F's and BETA.INV's callers only, as
        // for L140 bup+bgrat, and only past a0 = 5000: up to there T-3571's probes show Excel's own
        // bfrac arithmetic, which the port keeps.
        Branch::Bfrac if half_bgrat && past_evidenced_shapes(d.a0) => {
            let w = large_shape_lower(d.a0, d.b0, d.x0, d.y0)
                .map_or_else(|| bfrac(&d, BFRAC_EPS_FACTOR * EPS), Some)?;
            (w, other(w))
        }
        Branch::Bfrac => {
            let w = bfrac(&d, BFRAC_EPS_FACTOR * EPS)?;
            (w, other(w))
        }
        Branch::Basym => basym_oriented(&d)?,
        _ => return None,
    };
    Some(if d.swapped { (w1, w) } else { (w, w1) })
}

/// (w, w1) of basym on a dispatched point, before the swap back, the other tail bratio's
/// (0.5 - w) + 0.5. Every caller of the kernel takes it (T-3443; the binomial family since T-3352):
/// with it `BETA.DIST(0.555,1000,1000,TRUE,0.1,1)` is Excel's 0.6903551301985376 bit for bit
/// (ironcalc-calc `BETADIST_BETAINV.xlsx` G29), and BETA.INV crosses it by its own rule at these
/// points (`beta.rs` `beta_crossing`).
fn basym_oriented(d: &Dispatch) -> Option<(f64, f64)> {
    let w = basym(d.a0, d.b0, d.lambda, BASYM_EPS_FACTOR * EPS)?;
    Some((w, half_complement(w)))
}

/// (0.5 - v) + 0.5, each step an x87 op stored to a double: bratio's complement and bpser's
/// series factor 1 - b/n.
pub(super) fn half_complement(v: f64) -> f64 {
    extended_sum(extended_sum(0.5, -v), 0.5)
}

#[cfg(test)]
mod tests;
