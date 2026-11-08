// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::float_cmp)]
// LINT-DEBT-END
use super::annuity::{annuity_factor_rounded, annuity_power};
use crate::{
    arithmetic::{extended_product, extended_quotient, extended_sum, flush_subnormal},
    expressions::token::Error,
};

// The solvers behind RATE, IRR and XIRR. IRR (a secant in d = 1 - v, v = 1/(1+r)) and XIRR (a
// bracket walk and bisection) follow models fitted to oracle sweeps in the fit lab (T-1525,
// T-1533, T-3120; see their doc comments). RATE is a fitted secant too (T-2005, financial.md "RATE is a secant on the FV
// residual").

/// RATE's residual f(r) = ((pv·P) + (pmt·(k·((P-1)/r)))) + fv, P = (1+r)^n by FV's power
/// (`annuity_power`), k = 1+r for type 1, with each `+`, `*` and `/` Excel's twice-rounded operator
/// ([`annuity_factor_rounded`]; a once-rounded factor misses 4 of the 782 fit-lab
/// cells); f(0) is the limit pv + pmt·n + fv. `None` where Excel's `^` errs (a power past the
/// double range). Fit lab `rate` (T-2527): bit-exact on all 782 RATE cells of the fit set, a
/// nested PMT at Excel's bits (financial.md "RATE is a secant on the FV residual";
/// probes/rate-excel-pmt-bits pins it).
fn rate_residual(
    rate: f64,
    (nper, pmt, pv, fv): (f64, f64, f64, f64),
    period_start: bool,
) -> Option<f64> {
    if rate == 0.0 {
        return Some(pv + pmt * nper + fv);
    }
    let p = annuity_power(rate, nper).ok()?;
    let factor = annuity_factor_rounded(rate, p, period_start);
    let f = extended_sum(
        extended_sum(extended_product(pv, p), extended_product(pmt, factor)),
        fv,
    );
    f.is_finite().then_some(f)
}

/// The most secant steps RATE takes, Excel's own cap: the probe guesses whose run needs 200
/// steps converge and those that need 201 to 216 are `#NUM!` (probes/rate-step-cap).
/// A lifted RATE is charged the time of a run of this many steps (`RATE_WEIGHT` in the lifting
/// table, T-2814): refit it if the cap or a step's work changes.
pub(crate) const RATE_MAX_STEPS: u32 = 200;

/// RATE as Excel computes it: a secant on the residual [`rate_residual`] from r0 = guess and
/// r1 = r0 - 0.001 if f(r0) > 0, else r0 + 0.001 (a zero residual steps up, T-3876), each step `r2 = r1 - f1·(r1-r0)/(f1-f0)` with Excel's
/// twice-rounded `-`, `*` and `/`. It stops at the first iterate with |r2 - r1| < 1e-7 and
/// |f(r2)| < 1e-7. The iterate of step [`RATE_MAX_STEPS`] is returned unconverged when it passes
/// a looser test: |r2 - r1|, |f(r2)| and 1 + r2 against 1e-4, all absolute
/// (probes/rate-cap-return-rule, rate-pmt-zero-cap, rate-step-cap A30). The modelled outcome of a
/// flat secant (f1 = f0) is a hold at r1, judged by the same two tests, and an iterate at or below
/// -1 runs on (T-3348, decision 0198; the rule, its evidence and its known misses are in
/// financial.md "RATE holds on a flat secant and runs on below -1"). An early flat secant on two
/// distinct iterates with a small |f1| restarts the run once from the older iterate instead
/// (T-3698, financial.md "RATE restarts once on an early flat secant"). A residual Excel's `^`
/// cannot form and a run that has not stopped by then are `#NUM!`. Microsoft documents Newton's
/// method and 20 steps; neither fits. The fit, its scores and the rules not yet probed are in
/// financial.md "RATE is a secant on the FV residual" (T-2005).
pub(crate) fn compute_rate(
    pv: f64,
    fv: f64,
    nper: f64,
    pmt: f64,
    annuity_type: i32,
    guess: f64,
) -> Result<f64, Error> {
    if guess <= -1.0 {
        return Err(Error::VALUE);
    }
    let args = (nper, pmt, pv, fv);
    rate_secant(args, annuity_type != 0, guess, true)
}

/// One secant run of [`compute_rate`] from `guess`; `restart` allows the one restart of an early
/// flat secant (T-3698), which the restarted run does not get again.
fn rate_secant(
    args: (f64, f64, f64, f64),
    period_start: bool,
    guess: f64,
    restart: bool,
) -> Result<f64, Error> {
    // The start offset was chosen by scoring (T-2005; fit lab `rate_candidates` scores +-1E-3 and
    // 1E-4): on the 782 cells of the fit lab's `rate` set the model is bit-exact on all, with
    // nper 1..1000, |pmt| 1E-6..1E+6, pv -100..1E+7, guess -0.5..10 and rates -0.75..2. T-3478
    // measured it past that box on a fresh 300-point sweep (probes-numeric/t3478-rate-r4-wide):
    // bit-exact on all 100 points where Excel returns a number, where 1E-4 or the other side's
    // -1E-3 gets 8 and 14 (financial.md "RATE and IRR secant constants past their fit sets").
    // The tag's second range is where those 100 roots lie; past it (rates to 100, nper to 1E+5)
    // the sweep only agrees with Excel's `#NUM!`. Its 14 cells off Excel were not this constant's:
    // 13 converged to the spurious root r = -1 (`#NUM!` since T-3585's floor on 1 + r) and one is
    // T-3424's.
    /// FITTED(T-2005, 782 cells, RATE nper 1..1000, |pmt| 1E-6..1E+6, guess -0.5..10, rate -0.75..2; T-3478 sweep, 100 returned roots: nper 1..22908, |pmt| 1.6E-47..1.9E+10, |pv| and |fv| 1.1E-8..9.7E+11, guess -0.97..980, rate -0.985..21.3)
    const OFFSET: f64 = 0.001;
    // Three uses. The step stop is not a fit: Microsoft's RATE documentation gives 0.0000001 as the
    // convergence bound. The residual stop |f| < 1e-7 is not documented and was chosen by scoring
    // (financial.md "RATE is a secant on the FV residual": |f| < 1e-6 scores 188, < 1e-8 181), as
    // IRR's is (`irr_secant`). On T-3478's sweep 1e-7 keeps 100 of 100, 1e-6 92 and 1e-8 93. The
    // third is the floor on 1 + r of a converged iterate (T-3585, below).
    /// FITTED(T-2005, 782 cells, RATE nper 1..1000, |pmt| 1E-6..1E+6, guess -0.5..10, rate -0.75..2; T-3478 sweep, 100 returned roots: nper 1..22908, |pmt| 1.6E-47..1.9E+10, |pv| and |fv| 1.1E-8..9.7E+11, guess -0.97..980, rate -0.985..21.3; the residual stop)
    const TOLERANCE: f64 = 1e-7;
    // Excel's looser test at the cap (T-2689): the 200th iterate is returned when its step and
    // |f| are below 1e-4 and it is above -1 + 1e-4. probes/rate-cap-return-rule: steps of 7.3e-5
    // (A68) and 7.8e-5 (A72) return and 1.3e-4 (A70) is `#NUM!`, also where the step is 4e-4 of
    // the rate (A72);
    // |f| 7.3e-5 returns (rate-step-cap A30) and 1.06e-4 is `#NUM!` (A53); 1 + r 1.02e-4 returns (A1)
    // and 9.4e-5 is `#NUM!` (A16). One value sits inside all three brackets, so it is a cut placed
    // on the cells of the three cap probes (T-3395 counted them).
    /// FITTED(T-2689, 178 cells, RATE at the 200-step cap; one cut for all three tests bracketed 9.4E-5..1.02E-4)
    const RATE_CAP_TOLERANCE: f64 = 1e-4;
    // A flat secant on two distinct iterates at steps RATE_RESTART_FIRST..=RATE_RESTART_STEPS, with
    // |f1| from RATE_RESTART_F_MIN up to RATE_RESTART_F_MAX, restarts the run from the older iterate
    // r0 (T-3698). The step bound and the |f1| cut were chosen on the train side of the fit lab's
    // hash split only (holdout.py, margin 5 points stated before scoring; docs/research/t3698):
    // over K in 3..200 and the cut in 1E-5..1E-3, K = 16 or 17 is best on train (1,395 of 1,477
    // bit-exact) and the smaller is taken. The evidence brackets the cut: probes/t3698-rate-flat-
    // restart restarts at |f1| up to 4.3E-5 (A3:A8) and holds at 1.7E-4 (A1). Out of sample
    // (probes/t3906-rate-restart-oos, 240 fresh calls where the rule and the hold differ) the rule
    // is bit-exact on 48 and the hold on 12, but the rule moves A152 and A194 out of the Gate A
    // envelope (A194 becomes #NUM!), so it is not cleared: T-3850 must narrow or revert it
    // (T-3906). Outside the evidenced range (the first two
    // steps, |f1| below the smallest probed flat secant) the run holds as before. `r0 > -1` is a
    // guard, not evidence: a restart from r0 <= -1 would be a guess that RATE refuses.
    /// FITTED(T-3698, 1477 cells, RATE flat secant at steps 3..16 with |f1| 1.3E-7..1E-4)
    const RATE_RESTART_STEPS: u32 = 16;
    /// The first step the restart is evidenced at (no probed flat secant comes earlier; T-3698).
    const RATE_RESTART_FIRST: u32 = 3;
    /// The evidence floor of the restart's |f1| (T-3698). It sits just above 1.27E-7, the smallest |f1| of a
    /// probed flat secant on distinct iterates (step 58). Inside the rule's own steps 3..16 the smallest probed
    /// |f1| is 1.67E-7 (the nper-1 family at step 3), so 1.3E-7..1.67E-7 there is unprobed (T-3906 review).
    const RATE_RESTART_F_MIN: f64 = 1.3e-7;
    /// The |f1| cut of the restart: part of T-3698's fit (the FITTED tag on `RATE_RESTART_STEPS` covers it,
    /// scored over 1E-5..1E-3). It equals the cap's bound `RATE_CAP_TOLERANCE` but is not tied to it, so
    /// a change to that fit does not move this one (T-3906).
    const RATE_RESTART_F_MAX: f64 = 1e-4;
    let residual = |rate: f64| rate_residual(rate, args, period_start).ok_or(Error::NUM);
    let (mut r0, mut f0) = (guess, residual(guess)?);
    // Only a positive residual steps down: a residual of exactly 0 at the guess steps up, to
    // guess + 0.001. With every residual rounding noise (nper 1, pv + pmt = 0, fv 0) Excel returns
    // the guess exactly where f(guess + 0.001) is not 0 (the secant steps back onto the guess) and
    // is `#NUM!` where it is 0 too (a flat secant on 0), on all 14 such cells of
    // probes/t3876-rate-noise-residual; stepping down gets 11 (financial.md "RATE steps up from a
    // zero residual", T-3876).
    let mut r1 = if f0 > 0.0 { r0 - OFFSET } else { r0 + OFFSET };
    let mut f1 = residual(r1)?;
    for step in 1..=RATE_MAX_STEPS {
        // A flat secant (f1 = f0) has no finite step: 0/0 after a zero-step stall (r1 = r0),
        // a nonzero number over 0 otherwise. Holding at r1 is the modelled outcome that fits the
        // evidence; the engine stopped with `#NUM!` here before T-3348. The held point is judged
        // by the convergence test and by the cap's test, as the 200th iterate would be. Near the
        // root of a long annuity |f| is rounding noise above 1e-7: `RATE(239,-14841.27102594,
        // 344293.4825124,-155496.6323522,0)` is held at |f| 3.2e-7 and is 0.04310546919282547
        // in Excel (probes/t3348-rate-fv-nonzero A1). A residual of exactly 0 on both sides is
        // `#NUM!` (`RATE(1,0,0,0)`, probes/rate-zero-residual-guess A1, A2). Excel errs on 13
        // holds the rule returns (decision 0198, T-3424).
        let flat = f1 == f0;
        let (r2, f2) = if flat {
            if f1 == 0.0 {
                return Err(Error::NUM);
            }
            // The restart (T-3698): a fresh run from r0, as if r0 were the guess
            // (probes/t3579-rate-residual-nper1: A1's run is A6's, whose guess is A1's r0). A
            // restarted run that finds no root falls back to this run's hold when the hold's step
            // has converged, |r1 - r0| < 1e-7, and only its residual is noise (T-4975 audit 6):
            // probes/t3906-rate-restart-oos A194 `RATE(1,-57090142708.1999,57155400514.2199,
            // -66085711.1548055,1,-0.0413434246245947)` is 0.012686683557386938 in Excel, where the
            // restarted run meets a second flat secant at step 92 (|f| 1.65E-4) and was `#NUM!`;
            // the hold gives 0.012686683564913244, Excel's type and inside Gate A's envelope (its
            // iterates are 6.7E-16 apart). A26 `RATE(1,65619121.9595683,-65619121.9595674,0,1,
            // 0.0414411258870795)`, whose flat secant comes at step 3 on iterates 0.078 apart, stays
            // `#NUM!` as in Excel.
            if restart
                && r1 != r0
                && (RATE_RESTART_FIRST..=RATE_RESTART_STEPS).contains(&step)
                && (RATE_RESTART_F_MIN..RATE_RESTART_F_MAX).contains(&f1.abs())
                && r0 > -1.0
            {
                let restarted = rate_secant(args, period_start, r0, false);
                if restarted.is_ok() || (r1 - r0).abs() >= TOLERANCE {
                    return restarted;
                }
            }
            (r1, f1)
        } else {
            let r2 = extended_sum(
                r1,
                -extended_quotient(
                    extended_product(f1, extended_sum(r1, -r0)),
                    extended_sum(f1, -f0),
                ),
            );
            if !r2.is_finite() {
                return Err(Error::NUM);
            }
            // An iterate at or below -1 does not end the run (Excel's `^` takes the negative
            // base at an integer nper): only the result must be above -1
            // (probes/t3348-rate-below-minus-one, 50/50; T-3348). At a non-integer nper the
            // residual's `^` errs there and the run is `#NUM!`; Excel is unprobed (financial.md).
            (r2, residual(r2)?)
        };
        let step_size = (r2 - r1).abs();
        if step_size < TOLERANCE && f2.abs() < TOLERANCE {
            // A converged iterate is returned only when 1 + r is above 1e-7, the step bound: roots
            // with 1 + r from 1E-4 down to 1.0000034E-7 return and 1E-7 and below are `#NUM!`,
            // genuine roots and the spurious root -1 of a type-1, fv-0 residual alike
            // (probes/t3585-rate-num-near-minus-one; financial.md "RATE is #NUM! when 1 + r is
            // 1e-7 or less", T-3585).
            return if 1.0 + r2 > TOLERANCE {
                Ok(r2)
            } else {
                Err(Error::NUM)
            };
        }
        if (flat || step == RATE_MAX_STEPS)
            && step_size < RATE_CAP_TOLERANCE
            && f2.abs() < RATE_CAP_TOLERANCE
            && 1.0 + r2 > RATE_CAP_TOLERANCE
        {
            return Ok(r2);
        }
        if flat {
            return Err(Error::NUM);
        }
        (r0, f0, r1, f1) = (r1, f1, r2, f2);
    }
    Err(Error::NUM)
}

/// The worksheet NPV with Excel's arithmetic and range rules: (v1/d0 + v2/d1 + ... + vn/d(n-1))
/// / (1+rate), where d0 = 1 and d(i) = d(i-1)*(1+rate) is a running product, every step Excel's
/// twice-rounded operator and the quotients summed left to right (financial.md "NPV's residue at
/// IRR's root is not the double sum"; T-2671, probes/npv-term-order 382/382 and m9-fin-nper-npv
/// 88/88 in the fit lab, with plain or twice-rounded operators alike; IRR's NPV needs the latter,
/// T-3205).
/// The call is `#DIV/0!` when 1+rate is 0 (a rate of -1, also with no flow but 0s) or when a
/// discount a period divides by overflows or is below the smallest normal double (financial.md
/// "NPV's discount past the double range", probes/npv-power-edges and npv-power-edges-2):
/// `NPV(1E+300,1,1)` is 1E-300 but `NPV(1E+300,0,0,0)` is `#DIV/0!`, and `NPV(1,SEQUENCE(1024))`
/// is a number but 1025 flows are `#DIV/0!`. A subnormal quotient is 0, as for `/`
/// (`NPV(9E+307,1)` is 0).
pub(crate) fn excel_npv(rate: f64, values: &[f64]) -> Result<f64, Error> {
    let base = 1.0 + rate;
    if base == 0.0 {
        return Err(Error::DIV);
    }
    let sum = discounted_sum(base, values).ok_or(Error::DIV)?;
    Ok(flush_subnormal(extended_quotient(sum, base)))
}

/// `c0/d0 + c1/d1 + ... + cn/dn` with d0 = 1 and d(i) = d(i-1)*base a running product, every `*`,
/// `/` and `+` Excel's twice-rounded operator and the quotients summed left to right: the sum
/// worksheet NPV divides by `base` and the NPV IRR's secant evaluates at `base` = 1 + r. The
/// twice-rounded `*` and `+` decide IRR's last secant step: with plain doubles
/// `IRR({-1000,230,200,125,200,10},5)` and gen-m9's `IRR({0.5,0.1,-0.5},0.5)` stop one iterate
/// early, 16 ulp off (financial.md "IRR is a secant in d = 1 - v", T-3205). `None` when a discount
/// overflows or is below the smallest normal double; a subnormal quotient is 0, as for `/`.
fn discounted_sum(base: f64, values: &[f64]) -> Option<f64> {
    let mut sum = 0.0;
    // d(i), the running product this period divides by.
    let mut discount = 1.0_f64;
    for (i, item) in values.iter().enumerate() {
        if i > 0 {
            discount = flush_subnormal(extended_product(discount, base));
        }
        if discount == 0.0 || !discount.is_finite() {
            return None;
        }
        sum = extended_sum(sum, flush_subnormal(extended_quotient(*item, discount)));
    }
    Some(sum)
}

/// NPV at the discount factor `v` = 1/(1 + r), as IRR's search evaluates it: the
/// [`discounted_sum`] at x = 1/v, the running product worksheet NPV uses (financial.md "IRR is a
/// secant in d = 1 - v"; fit lab `irr`, T-3120). `None` when a discount overflows or underflows
/// or the sum is not finite: that ends the secant's call with `#DIV/0!` (T-3407).
fn irr_npv(values: &[f64], v: f64) -> Option<f64> {
    discounted_sum(extended_quotient(1.0, v), values).filter(|total| total.is_finite())
}

/// The smallest 1 + r IRR does not return: Excel's IRR is `#NUM!` at roots this far out, from
/// the secant or the fallback alike (financial.md "IRR's far-root bound"): `IRR({-1,3E+6})` and
/// `IRR({-1,0,1E+14})` (1 + r = 9999999.994) return, `IRR({-1,1E+7})` and every larger
/// two-flow root are `#NUM!` (probes/irr-fallback-roots A25-A39, irr-guess-edges-2 A15-A34).
/// The exact edge between 9999999.994 and 1E+7 is not pinned: the round value is placed in that
/// bracket (T-2727).
/// FITTED(T-2727, 35 cells, IRR roots with 1 + r 1E+4..2E+15; cut bracketed 9999999.994..1E+7)
const IRR_FAR_ROOT_BOUND: f64 = 1e7;

/// The |NPV| above which a search that ends on a flat secant after a zero step is `#DIV/0!` rather
/// than `#NUM!`: decision 0316's error-only classifier (financial.md "IRR is #DIV/0! on a flat secant
/// after a far step"; T-3347). On long conventional series a run jumps far toward r = -1, where NPV
/// is 1E+200 or more, comes back and stalls. Placed on the train side of the fit lab's split
/// (decision 0188 R4.1): the largest train `#NUM!` run reaches 1.556E+226
/// (probes/t3406-irr-num-boundary A65), the smallest train `#DIV/0!` run above it 6.57E+230 (A81).
/// It is a proxy, not Excel's mechanism: which error Excel gives where no attempt converges is
/// path-dependent (t3406 n = 60 gives A81, A82 `#DIV/0!`, A83 `#NUM!`, A84 `#DIV/0!`; addendum 3 of
/// docs/research/excel-parity-findings-2026-11-07.md, "Still open"). Since T-4975 it is read only
/// after every attempt has failed ([`compute_irr`]), so it cannot pre-empt a root a retry finds
/// (probes-numeric/t3347-irr-r4-fresh A52), and it applies to every input.
/// FITTED(T-4975, 36 cells, every IRR input after all attempts fail; fitted on flat runs of 27..144 flows by T-3347)
const IRR_FLAT_DIV_ZERO_NPV: f64 = 3e226;

/// The updates of one IRR secant attempt ([`irr_secant`]), for every input: no dependence on the
/// flow count, their size or the guess (addendum 3 of docs/research/excel-parity-findings-2026-11-07.md,
/// section H; T-4975 audit 3). The initial NPV evaluations and a flat slope's resampling are not
/// updates. Excel finishes runs of 51 to 200 updates on the run's own digits (83 cells; T-3408,
/// decision 0320): gen-m9/31474c20fc1dafd6 Calc!A173 and ba4ddb817a8e700e Calc!A373 reach Excel
/// after updates 51 and 68 with a 1E+15 flow, and probes-numeric/t3347-irr-conventional-holdout
/// P!A25 returns at update 200 itself (update 199 is another grid point). Until T-4975 the 200
/// applied only inside a training-set box of flows, peak and guess (50 updates outside), so
/// appending a zero flow turned a root `#NUM!` and t3478-irr-r4-wide A206 was `#NUM!`.
/// FITTED(T-4975, 900 cells, every IRR input: 200 updates per attempt, addendum 3 H; first fitted by T-3408 on 6..54 flows)
const IRR_SECANT_STEPS: usize = 200;

/// The stop's bound on |step| and |NPV| at every update but the last: Microsoft's documented
/// 0.00001 percent for the step, and the residual's chosen by scoring (T-1533). The last update
/// takes [`IRR_LAST_UPDATE_TOLERANCE`].
const IRR_TOLERANCE: f64 = 1e-7;

/// The bound on |step| and |NPV| at the [`IRR_SECANT_STEPS`]th update (addendum 3, section H;
/// t3347-irr-conventional-holdout P!A25 returns there), as RATE's cap test (`RATE_CAP_TOLERANCE`).
const IRR_LAST_UPDATE_TOLERANCE: f64 = 1e-4;

/// The secant's start offset in d and a flat slope's resampling step (T-1533).
const IRR_OFFSET: f64 = 1e-3;

/// The coordinate d = 1 - v of IRR's last attempt, after the guess's and the default guess's: the
/// literal d = -0.1, not a rate guess of -0.1 (addendum 3, section H). Its signed start offset
/// explains the last attempt's negation-sensitive digits: `IRR({42,1,1,-1})` is -0.7467044766537371
/// at its update 47 and the negated flows give ...7407 (probes-numeric/t2927-irr-fallback-sweep
/// A80); ironcalc-calc IRR.xlsx General!L17 `IRR({-400,1,1,1,1,1})` is -0.6737366751808405 at
/// update 125. It replaced T-2727's Newton fallback from v = 0, whose digits were 15 to 35 ulp off.
const IRR_LAST_START: f64 = -0.1;

/// The rate 1/v - 1 at a converged discount factor `v`, or `None` past [`IRR_FAR_ROOT_BOUND`].
fn irr_from_discount(v: f64) -> Option<f64> {
    let x = extended_quotient(1.0, v);
    (x < IRR_FAR_ROOT_BOUND).then_some(x - 1.0)
}

/// How an attempt of [`irr_secant`] ended without a root.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SecantStop {
    /// No root; `far_flat` when it went flat after a zero step having seen an |NPV| past
    /// [`IRR_FLAT_DIV_ZERO_NPV`] (the error-only classifier, read after every attempt failed).
    Failed { far_flat: bool },
    /// An iterate reached v = 0, or NPV could not be evaluated: the call is `#DIV/0!`.
    DivZero,
}

/// One attempt of IRR's secant in d = 1 - v = r/(1 + r) from d0 = `start` (addendum 3 of
/// docs/research/excel-parity-findings-2026-11-07.md, section H; financial.md "IRR is a secant in
/// d = 1 - v"): d1 = d0 -+ [`IRR_OFFSET`] toward larger |NPV|, then up to [`IRR_SECANT_STEPS`]
/// updates `d2 = d1 - f1·(d1 - d0)/(f1 - f0)`, every operation and NPV's running product Excel's
/// twice-rounded operator, `1 - d` included. Where the two residuals are equal the older endpoint
/// is replaced by `d0 -+ IRR_OFFSET` (away from d1) and re-evaluated; if they are still equal the
/// attempt fails. It stops when |d2 - d1| and |f2| are below [`IRR_TOLERANCE`]
/// ([`IRR_LAST_UPDATE_TOLERANCE`] at the last update) and returns the new iterate's rate.
/// Intermediate discounts may be negative; the final one must be positive and within
/// [`IRR_FAR_ROOT_BOUND`] (T-2928, T-4975 item 11). `DivZero` at v = 0 or where NPV cannot be
/// evaluated (T-2802, T-3407).
/// FITTED(T-1533, 263 cells, IRR of 2..41 flows, |c| 1E-6..1E+9, guess -0.99..100, roots -0.99..99; T-3478 sweep, 231 Excel roots: 2..148 flows, |c| 2.3E-15..4.0E+12, guess -0.9997..9.8E+7, roots -0.9997..9.0E+5)
fn irr_secant(values: &[f64], start: f64) -> Result<f64, SecantStop> {
    let npv = |d: f64| {
        let v = extended_sum(1.0, -d);
        if v == 0.0 {
            return Err(SecantStop::DivZero);
        }
        irr_npv(values, v).ok_or(SecantStop::DivZero)
    };
    let mut d0 = start;
    let mut f0 = npv(d0)?;
    let mut d1 = d0 - if f0 > 0.0 { IRR_OFFSET } else { -IRR_OFFSET };
    let mut f1 = npv(d1)?;
    let mut largest_npv = f0.abs().max(f1.abs());
    // Whether the attempt met a flat slope after a zero step having gone far: the error-only
    // classifier's case (T-3347), read only if the attempt then fails.
    let mut far_flat = false;
    for update in 1..=IRR_SECANT_STEPS {
        if f1 == f0 {
            far_flat |= d1 == d0 && largest_npv > IRR_FLAT_DIV_ZERO_NPV;
            d0 += if d1 <= d0 { -IRR_OFFSET } else { IRR_OFFSET };
            f0 = npv(d0)?;
            largest_npv = largest_npv.max(f0.abs());
            if f1 == f0 {
                return Err(SecantStop::Failed { far_flat });
            }
        }
        let d2 = extended_sum(
            d1,
            -extended_quotient(
                extended_product(f1, extended_sum(d1, -d0)),
                extended_sum(f1, -f0),
            ),
        );
        if !d2.is_finite() {
            return Err(SecantStop::Failed { far_flat });
        }
        let f2 = npv(d2)?;
        largest_npv = largest_npv.max(f2.abs());
        let tolerance = if update == IRR_SECANT_STEPS {
            IRR_LAST_UPDATE_TOLERANCE
        } else {
            IRR_TOLERANCE
        };
        if (d2 - d1).abs() < tolerance && f2.abs() < tolerance {
            let v = extended_sum(1.0, -d2);
            return if v > 0.0 {
                irr_from_discount(v).ok_or(SecantStop::Failed { far_flat })
            } else {
                Err(SecantStop::Failed { far_flat })
            };
        }
        (d0, f0, d1, f1) = (d1, f1, d2, f2);
    }
    Err(SecantStop::Failed { far_flat })
}

/// The smallest guess IRR refuses as too large: 2^53.
const IRR_GUESS_BOUND: f64 = 9_007_199_254_740_992.0;

/// IRR as Excel computes it (addendum 3 of docs/research/excel-parity-findings-2026-11-07.md,
/// section H): [`irr_secant`] from the guess's d = guess/(1 + guess), then from the default guess's
/// d = 0.1/1.1, then from d = [`IRR_LAST_START`], returning the first root found. Where every
/// attempt fails the call is `#NUM!`, or `#DIV/0!` where one went flat far out (decision 0316's
/// error-only classifier, [`IRR_FLAT_DIV_ZERO_NPV`]); an attempt that reaches v = 0 or cannot
/// evaluate NPV ends the call with `#DIV/0!` (T-3407). Leading zero flows are dropped first.
pub(crate) fn compute_irr(values: &[f64], guess: f64) -> Result<f64, (Error, String)> {
    // Excel: a guess at or above 2^53 is #VALUE! too, before the sign-change check:
    // `IRR({-100,60,60},2^53-1)` and `(..,9E+15)` converge, `(..,2^53)` and `(..,1E+16)` are #VALUE!,
    // and so is `IRR(COUNTBLANK(..),1E+300)` (probes/irr-guess-edges A1-A19,
    // irr-guess-edges-2 A1-A14; financial.md "IRR's guess range").
    if guess <= -1.0 || guess >= IRR_GUESS_BOUND {
        return Err((
            Error::VALUE,
            "Rate initial guess must be > -1 and < 2^53".to_string(),
        ));
    }
    // The values cannot be all positive or all negative
    if values.iter().all(|&x| x >= 0.0) || values.iter().all(|&x| x <= 0.0) {
        return Err((Error::NUM, "Failed to converge".to_string()));
    }
    // Leading zero flows are dropped: `IRR({0,-100,110},0.05)` and `IRR({0,0,-100,110},10)` are
    // `IRR({-100,110},..)`'s 0.09999999999999987, not the secant's value on the full flows
    // (probes/irr-guess-edges A33, m9-fin-irr-rate A173 and A175; T-3120).
    let values = values
        .iter()
        .position(|&c| c != 0.0)
        .map_or(values, |first| &values[first..]);
    let div_zero = || (Error::DIV, "Division by zero".to_string());
    let starts = [
        extended_quotient(guess, 1.0 + guess),
        extended_quotient(0.1, 1.1),
        IRR_LAST_START,
    ];
    let mut far_flat = false;
    for start in starts {
        match irr_secant(values, start) {
            Ok(root) if root.is_finite() => return Ok(root),
            Ok(_) => {}
            Err(SecantStop::DivZero) => return Err(div_zero()),
            Err(SecantStop::Failed { far_flat: far }) => far_flat |= far,
        }
    }
    if far_flat {
        Err(div_zero())
    } else {
        Err((Error::NUM, "Failed to converge".to_string()))
    }
}

/// XNPV: the sum of `value_i / (1 + rate)^((date_i - date_0)/365)`, days counted from the first date;
/// `#NUM!` when the sum is not finite or a power is `#NUM!` for a base <= 0.
pub(crate) fn compute_xnpv(
    rate: f64,
    values: &[f64],
    dates: &[f64],
) -> Result<f64, (Error, String)> {
    let mut xnpv = values[0];
    let d0 = dates[0];
    let n = values.len();
    for i in 1..n {
        let vi = values[i];
        let di = dates[i];
        // Excel's own `^` (docs/excel-semantics/financial.md "XNPV discounts with Excel's `^`").
        // A positive base whose power overflows (`#NUM!`) still adds value/inf = 0, as `powf` did (unprobed, unchanged).
        match crate::arithmetic::power(1.0 + rate, (di - d0) / 365.0) {
            Ok(power) => xnpv += vi / power,
            Err(Error::NUM) if 1.0 + rate > 0.0 => {}
            Err(e) => return Err((e, "NaN".to_string())),
        }
    }
    if xnpv.is_infinite() || xnpv.is_nan() {
        return Err((Error::NUM, "NaN".to_string()));
    }
    Ok(xnpv)
}

/// XNPV at a trial rate of XIRR's search, or `None` where it has no value (`1+rate <= 0`, or the
/// sum overflows, or a power overflows or underflows): such a point ends a bracket walk.
fn xirr_xnpv(rate: f64, values: &[f64], dates: &[f64]) -> Option<f64> {
    let base = 1.0 + rate;
    if base <= 0.0 {
        return None;
    }
    let d0 = dates[0];
    let mut total = 0.0;
    for (&value, &date) in values.iter().zip(dates) {
        // As in the lab model, a power that overflows or underflows to 0 has no value (rather
        // than adding value/inf = 0): `XIRR({-1,1E+20,1},{45000,45365,63615},0.1)` is `#NUM!`.
        let power = crate::arithmetic::power(base, (date - d0) / 365.0).ok()?;
        if power == 0.0 {
            return None;
        }
        total += value / power;
    }
    total.is_finite().then_some(total)
}

/// The bracket XIRR bisects: the first point of a trial chain whose XNPV sign differs from
/// XNPV(0)'s (any nonzero XNPV when XNPV(0) is exactly 0), with the point before it. With `s`
/// the guess (1e-5 for a guess of 0) the chain is 0, s, 2s, 4s, ... for a guess >= 0, then, when
/// that finds nothing and `s` < 1, 0, -s and p → (p - 1)/2 toward -1; a negative guess walks
/// only 0, g, (g - 1)/2, ...
/// (docs/excel-semantics/financial.md "XIRR is a bracketing bisection, not Newton"; fit lab `xirr`, T-1525).
/// Excel's cap on the walk is unprobed: 100 points per walk is enough on every oracle cell.
/// A zero first value skips the walk: the bracket is [0, s] whatever the flows, even where XNPV is
/// flat or has no root there, and a negative guess has no bracket (probes/xirr-zero-first-bracket, T-2329).
fn xirr_bracket(values: &[f64], dates: &[f64], guess: f64) -> Option<(f64, f64)> {
    const WALK_CAP: usize = 100;
    let step = if guess == 0.0 { 1e-5 } else { guess };
    if values.first() == Some(&0.0) {
        return (guess >= 0.0).then_some((0.0, step));
    }
    let f0 = xirr_xnpv(0.0, values, dates)?;
    let walk = |first: f64, next: &dyn Fn(f64) -> f64| -> Option<(f64, f64)> {
        let (mut previous, mut point) = (0.0, first);
        for _ in 0..WALK_CAP {
            let fp = xirr_xnpv(point, values, dates)?;
            // An exact XNPV(0) = 0 (a root at 0) differs from any nonzero point, so the first
            // point brackets: `XIRR({-100,10,20,30,40},{0,1,59,60,61},G)` is G/2^k, not a
            // negative near 0 (probes/irr-guess-edges A41-A47, T-2728).
            if (fp > 0.0) != (f0 > 0.0) || (f0 == 0.0 && fp != 0.0) {
                return Some((previous, point));
            }
            (previous, point) = (point, next(point));
        }
        None
    };
    let toward_minus_one = |p: f64| (p - 1.0) / 2.0;
    if guess >= 0.0 {
        walk(step, &|p| p * 2.0).or_else(|| {
            if step < 1.0 {
                walk(-step, &toward_minus_one)
            } else {
                None
            }
        })
    } else {
        walk(guess, &toward_minus_one)
    }
}

/// XIRR as Excel computes it: not Newton, but a bisection of [`xirr_bracket`]. `a` starts at the
/// upper end; each step takes m = a + (b - a)/2 and replaces the upper end when
/// XNPV(m)·values[0] >= 0, else the lower one (an exact 0 does not stop the search). It returns m
/// once the half-width is below 5e-9 + 1e-8·|m| and, on a negative midpoint only, |XNPV(m)| is
/// below 2e-8·max|value| (no XNPV bound when the first value is 0). Every positive root in the
/// oracle cells stops at the first step-test pass and every negative one fits that bound (T-3268),
/// so the fitted rule matches all 227 cells of the fit set (probes/solver-iteration-traces sheet X,
/// probes-numeric/m9-fin-irr-rate) and 80 of 81 out of sample, the miss being the input check of
/// xirr-early-dates A7 (fit lab `xirr`, `xirr-oos`; T-1525, T-1583, T-3268).
/// Excel's bisection cap is unprobed: 200 steps is enough on every oracle cell.
/// Range (T-3385): the 267 are the numeric cells of the fit lab's `xirr` and `xirr-oos` sweeps (308
/// cells, 278 numeric) whose first value is nonzero: 2 to 6 flows over 0.5 to 124 years, |value| 5E-6 to 1.4E+21,
/// roots -0.93 to 22.05 (184 positive, 83 negative).
/// FITTED(T-3268, 267 cells, 2..6 flows over 0.5..124 years, |value| 5E-6..1.4E+21, root -0.93..22.05)
pub(crate) fn compute_xirr(
    values: &[f64],
    dates: &[f64],
    guess: f64,
) -> Result<f64, (Error, String)> {
    const BISECTION_CAP: usize = 200;
    let no_result = || (Error::NUM, "Failed to converge".to_string());
    // A guess at or below -1 is `#NUM!`: `XIRR({-100,110},{39448,39508},-1)`
    // (probes/t3312-xirr-guess-dates-weibull A9, T-3312).
    if guess <= -1.0 {
        return Err((Error::NUM, "Rate initial guess must be > -1".to_string()));
    }
    // The values cannot be all positive or all negative
    if values.iter().all(|&x| x >= 0.0) || values.iter().all(|&x| x <= 0.0) {
        return Err(no_result());
    }
    let (first, second) = xirr_bracket(values, dates, guess).ok_or_else(no_result)?;
    let (mut upper, mut lower) = if first > second {
        (first, second)
    } else {
        (second, first)
    };
    let first_value = values[0];
    let largest = values.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
    // `a` (here `upper`) is the end the midpoint is measured from
    for _ in 0..BISECTION_CAP {
        let m = upper + (lower - upper) / 2.0;
        let fm = xirr_xnpv(m, values, dates).ok_or_else(no_result)?;
        let half_width = (lower - upper).abs() / 2.0;
        if half_width < 5e-9 + 1e-8 * m.abs()
            && (m >= 0.0 || first_value == 0.0 || fm.abs() < 2e-8 * largest)
        {
            return Ok(m);
        }
        if fm * first_value >= 0.0 {
            upper = m;
        } else {
            lower = m;
        }
    }
    Err(no_result())
}

#[cfg(test)]
mod tests {
    use super::compute_irr;

    #[test]
    fn the_secant_cap_does_not_depend_on_the_flows_or_the_guess() {
        // T-4975 audit 3: a trailing zero flow leaves NPV and the root unchanged; it used to move
        // the call out of the long cap's training box (6..54 flows) and turn it `#NUM!`.
        let mut flows = vec![-50.0];
        flows.extend(std::iter::repeat_n(1.0, 53));
        let root = compute_irr(&flows, -0.9);
        assert!(root.is_ok(), "{root:?}");
        flows.push(0.0);
        assert_eq!(compute_irr(&flows, -0.9), root);
    }
}
