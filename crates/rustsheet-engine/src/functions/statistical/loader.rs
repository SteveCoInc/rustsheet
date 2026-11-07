//! Catherine Loader's saddle point ("Fast and Accurate Computation of Binomial Probabilities",
//! 2000) as Excel's beta density computes it for min(a, b) > 1.
//!
//! Excel's beta density for min(a, b) > 1 is the same saddle point in Excel's own arithmetic
//! (statistics.md "Beta density for min(a,b) > 1 is Loader's saddle point", T-2464, T-2513): x87
//! stored operations on Excel's LN, a 0.2 cut in its bd0 ([`excel_bd0`]), the exact Stirling error
//! ([`stirlerr_exact`]) and an exponent held in an x87 register into Excel's EXP
//! ([`excel_dbinom_exponent`], [`excel_beta_density`]). The model's shape constant carries an
//! unexplained residual of 2-4 units of 2^-53 and the series branch 3-41 units of noise (T-2586),
//! so it is not bit-exact everywhere.
//!
//! `poisson.rs` is a second consumer: Excel's POISSON.DIST pmf is this saddle point too, on
//! [`excel_bd0`] and [`stirlerr_exact`] (T-2055), so retuning either moves POISSON cells as well.
//! The Student t density is a third ([`excel_t_density`], T-2585).
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::float_cmp, clippy::many_single_char_names)]
#![cfg_attr(test, expect(clippy::unreadable_literal))]
// LINT-DEBT-END

use crate::arithmetic::{
    extended_product, extended_quotient, extended_sqrt, extended_sum, flush_subnormal,
};
use crate::x87_exp::{
    dd_add, dd_div, dd_div_small, dd_mul, excel_exp, excel_exp_extended, two_sum,
};
use crate::x87_log10::{excel_ln, ln_double_double_pair};
use crate::x87_register::Register;

use super::gammaln::STIRLING;
use super::incomplete_beta::FRAC_1_SQRT_2PI;

/// A double-double `hi + lo`.
type Dd = (f64, f64);

/// ln sqrt(2 pi) = 0.918938533204672741780..., the correctly rounded double
/// (`0x3FED_67F1_C864_BEB5`). Deliberately not `gammaln.rs`'s `LN_SQRT_2PI`, which is one ulp
/// lower because it reproduces the constant Excel's GAMMALN Stirling sum adds; these pieces are
/// accurate forms, not Excel's, so they take the exact value.
const LN_SQRT_2PI_CORRECTLY_ROUNDED: f64 = 0.918_938_533_204_672_8;

/// The bound on [`excel_bd0`]'s series terms; it converges in far fewer (|v| < 0.2, so v^2 < 0.04).
const BD0_MAX_TERMS: u32 = 1000;

/// The Stirling series terms for k = 11 and 12, `B_2k / (2k (2k - 1))`, after `gammaln::STIRLING`'s
/// k = 1..=10. At n = 15 the next term is under 1e-26, far below the 2^-64 of the exponent
/// [`stirlerr_exact`] goes into.
const STIRLING_EXTRA: [f64; 2] = [77_683.0 / 5_796.0, -236_364_091.0 / 1_506_960.0];

/// From this n on [`stirlerr_exact`] sums the Stirling series; below it the recurrence first
/// steps n up past it.
const STIRLING_SERIES_MIN: f64 = 15.0;

/// The exact Stirling error `ln Gamma(n + 1) - (n + 1/2) ln n + n - ln sqrt(2 pi)` of a double
/// n > 0 as a double-double (about 1e-25 absolute), the stirlerr of Excel's beta density
/// (statistics.md "Beta density for min(a,b) > 1 ...": stirlerr is accurate, a GAMMALN or a
/// double-rounded lgamma form misses by rms 15-26 units). Not the F density's model of
/// Excel's rounded stirlerr (`f_density`'s error table), which that density keeps. Below 15 the recurrence
/// stirlerr(m) = stirlerr(m + 1) + (m + 1/2) (ln(m + 1) - ln m) - 1 steps n up (at most 15 steps,
/// every m = n + i exact as a double-double), then the series in 1/m.
pub(crate) fn stirlerr_exact(n: f64) -> Dd {
    let mut sum = (0.0, 0.0);
    let mut step = 0.0;
    let mut ln_here = ln_double_double_pair(two_sum(n, 0.0));
    while n + step < STIRLING_SERIES_MIN {
        let ln_next = ln_double_double_pair(two_sum(n, step + 1.0));
        let factor = two_sum(n, step + 0.5);
        let term = dd_mul(factor, dd_add(ln_next, (-ln_here.0, -ln_here.1)));
        sum = dd_add(sum, dd_add(term, (-1.0, 0.0)));
        ln_here = ln_next;
        step += 1.0;
    }
    dd_add(sum, stirling_series(two_sum(n, step)))
}

/// Stirling's series `sum_k B_2k / (2k (2k - 1) m^(2k - 1))` for m >= 15, the two leading terms in
/// double-double and the rest (`STIRLING` from k = 3, then [`STIRLING_EXTRA`]) in double (it is under
/// 1e-9 there).
fn stirling_series(m: Dd) -> Dd {
    let z = dd_div((1.0, 0.0), m);
    let z2 = dd_mul(z, z);
    let tail = STIRLING[2..]
        .iter()
        .chain(&STIRLING_EXTRA)
        .rev()
        .fold(0.0, |acc, &coefficient| coefficient + z2.0 * acc);
    let second = dd_add(dd_div_small((-1.0, 0.0), 360.0), dd_mul(z2, (tail, 0.0)));
    let first = dd_add(dd_div_small((1.0, 0.0), 12.0), dd_mul(z2, second));
    dd_mul(z, first)
}

/// The ratio cut of Excel's bd0: its series runs while |k - M| < 0.2 (k + M) (statistics.md "Beta
/// density for min(a,b) > 1 ...": 0.25 drops (5.5, 12.5) from 113 to 13 of 120 exact cells; R's
/// is 0.1). The t density reuses it (T-2585): of 0.1, 0.15, 0.25 and 0.3 on its cells, only 0.25
/// and 0.3 gain exact cells (18 and 4 of 12,037), each losing within-4-ulp and 15-digit cells, and
/// none beats it on the fresh batch r4-T-2585 (decision 0299).
/// FITTED(T-2464, 3072 cells, BETA.DIST a 2.5, b 3.5, v 0.130..0.250; T.DIST density on 12,158 cells, df 3..2.7E297, per decision 0299)
const EXCEL_BD0_CUT: f64 = 0.2;

/// Excel's bd0(k, M) = k ln(k / M) + M - k, as the double its beta density sums (model M4 of
/// docs/research/t2464/r3-loader-const, with r3-loader-bd0's knobs), in x87 stored operations:
/// d = fl(k - M) and s = fl(k + M); then
/// - the log form when |d| >= RN64(0.2 s): fl(t + fl(M - k)) with t = fl(k LN(fl(k / M))) and LN
///   Excel's (M - k stored: beyond a ratio of 2 it rounds, and the stored form fits Excel);
/// - below the cut Loader's series in v = fl(d / s) (stored), in extended precision, which carries
///   +1 times the rounding error of fl(2 k v): Excel's arithmetic there is not identified, and this
///   phenomenological term (r3-loader-bd0's `ph` knob) is what fits (T-2586 owns the rest).
///
/// For k > 0 and M > 0; the series has at most [`BD0_MAX_TERMS`] terms.
pub(crate) fn excel_bd0(k: f64, m: f64) -> f64 {
    match bd0_branch(k, m) {
        Bd0Branch::Series(sum) => sum.store(),
        Bd0Branch::Log(t) => extended_sum(t, extended_sum(m, -k)),
    }
}

/// [`excel_bd0`] as Excel's t density uses it, left in an x87 register: the series sum unstored,
/// and the log form `t + (M - k)` summed in the register rather than stored (r3-loader-bd0's
/// `ld = 'e'`; docs/research/t2464/r3-tdensity model LT1). Storing it first moves
/// `T.DIST(2.2014...,11,FALSE)` (t2513-r3-tdensity-hold2) by 3 ulp off Excel's (T-2585).
fn excel_bd0_unstored(k: f64, m: f64) -> Register {
    match bd0_branch(k, m) {
        Bd0Branch::Series(sum) => sum,
        Bd0Branch::Log(t) => Register::load(t).add(Register::load(m).add(Register::load(-k))),
    }
}

/// Which form [`excel_bd0`] takes, with what both callers share of it.
enum Bd0Branch {
    /// The series sum, unstored.
    Series(Register),
    /// The log form's t = fl(k LN(fl(k / M))).
    Log(f64),
}

/// [`excel_bd0`]'s cut between its series and its log form.
fn bd0_branch(k: f64, m: f64) -> Bd0Branch {
    let d = extended_sum(k, -m);
    let s = extended_sum(k, m);
    let cut = Register::load(EXCEL_BD0_CUT).mul(Register::load(s));
    if Register::load(d.abs()).add(cut.neg()).is_negative() {
        return Bd0Branch::Series(excel_bd0_series(k, d, s));
    }
    Bd0Branch::Log(extended_product(k, excel_ln(extended_quotient(k, m))))
}

/// [`excel_bd0`]'s series branch: R's loop `s = d v + 2k sum_j v^(2j+1) / (2j + 1)` in x87 registers
/// with v stored, less the rounding error of fl(2 k v), left unstored.
fn excel_bd0_series(k: f64, d: f64, s: f64) -> Register {
    let v = Register::load(extended_quotient(d, s));
    let mut ej = Register::load(2.0 * k).mul(v);
    let rounding = Register::load(ej.store()).add(ej.neg());
    let mut sum = Register::load(d).mul(v).add(rounding.neg());
    let v2 = v.mul(v);
    for j in 1..BD0_MAX_TERMS {
        ej = ej.mul(v2);
        let next = sum.add(ej.div(Register::load(f64::from(2 * j + 1))));
        if next.same_value(sum) {
            return next;
        }
        sum = next;
    }
    sum
}

/// ln(2 pi), the double Excel's lf starts from (R's `M_LN_2PI`, correctly rounded:
/// `0x1.d67f1c864beb5p+0`, exactly twice [`LN_SQRT_2PI_CORRECTLY_ROUNDED`]). Bit-exact counts are
/// very sensitive to it (statistics.md "Beta density for min(a,b) > 1 ...").
const LN_2PI: f64 = 2.0 * LN_SQRT_2PI_CORRECTLY_ROUNDED;

/// The exponent of Excel's Loader binomial density `dbinom_raw(k, n, p, q)` = EXP(E) for
/// 0 < k < n, as the exact double-double sum
/// E = stirlerr(n) - stirlerr(k) - stirlerr(nk) - lf/2 + X, which Excel holds in an x87 register
/// (statistics.md "Beta density for min(a,b) > 1 ...", model M4):
/// - k, n and nk = n - k are the caller's stored doubles (Excel's own, not recomputed: single-ulp
///   neighbours of a shape jump by up to 20 units with the rounding of nk);
/// - the stirlerr terms exact ([`stirlerr_exact`]);
/// - lf = RN64(RN64(RN64(ln 2 pi + LN k) + LN nk) - LN n), Excel's LN, extended sums;
/// - X = RN64(-bd0(k, fl(n p)) - bd0(nk, fl(n q))) with [`excel_bd0`].
///
/// `None` where a piece is not finite (an n p or n q of 0, a ratio past the double range). Exposed
/// for the binomial pmf, which shares it (T-2146).
pub(crate) fn excel_dbinom_exponent(k: f64, n: f64, nk: f64, p: f64, q: f64) -> Option<Dd> {
    let np = extended_product(n, p);
    let nq = extended_product(n, q);
    if !(k > 0.0 && nk > 0.0 && np > 0.0 && nq > 0.0 && n.is_finite()) {
        return None;
    }
    let bd0_p = excel_bd0(k, np);
    let bd0_q = excel_bd0(nk, nq);
    // A register loads a NaN or an infinity as 0, so a ratio past the double range (k / M for a
    // subnormal M) must stop here.
    if !(bd0_p.is_finite() && bd0_q.is_finite()) {
        return None;
    }
    let x_part = Register::load(-bd0_p).add(Register::load(-bd0_q));
    let lf = Register::load(LN_2PI)
        .add(Register::load(excel_ln(k)))
        .add(Register::load(excel_ln(nk)))
        .add(Register::load(-excel_ln(n)));
    let (lf_hi, lf_lo) = lf.to_double_double();
    let parts = dd_add(stirlerr_exact(k), stirlerr_exact(nk));
    let stirling = dd_add(stirlerr_exact(n), (-parts.0, -parts.1));
    let exponent = dd_add(
        dd_add(stirling, (-0.5 * lf_hi, -0.5 * lf_lo)),
        x_part.to_double_double(),
    );
    (exponent.0.is_finite() && exponent.1.is_finite()).then_some(exponent)
}

/// The binomial counts of [`excel_beta_density`]'s `dbinom_raw(k, n, ..)`: k = fl(a - 1),
/// n = fl(fl(a + b) - 2) and nk = fl(n - k), x87 stored operations, as Excel rounds them.
fn beta_density_counts(a: f64, b: f64) -> (f64, f64, f64) {
    let k = extended_sum(a, -1.0);
    let n = extended_sum(extended_sum(a, b), -2.0);
    (k, n, extended_sum(n, -k))
}

/// Whether Excel's Loader beta density (both shapes above 1) is `#NUM!` at every x because its
/// count nk = n - k of [`beta_density_counts`] is not positive, so that its LN(nk) has no value:
/// above 2^53 fl(a - 1) and fl(fl(a + b) - 2) round onto the same double (nk = 0) or past each
/// other (nk = -4 at `BETA.INV(0.4,2.23872113856835E+16,1.5)`). `BETA.DIST(1-2^-51,a,2,FALSE)` is
/// `#NUM!` at exactly the 30 of 41 alphas from 7.9E+15 to 2E+16 where nk = 0, and a number (one
/// with nk = 2 instead of 1) at the other 11 (probes/t4713-beta-inv-cut-pieces H1:H41;
/// docs/excel-semantics/statistics.md "BETA.INV and BETA.DIST's density are #NUM! where Loader's
/// n - k is not positive (T-4713)").
pub(crate) fn beta_density_counts_vanish(a: f64, b: f64) -> bool {
    a > 1.0 && b > 1.0 && beta_density_counts(a, b).2 <= 0.0
}

/// Excel's BETA.DIST(x, a, b, FALSE) for min(a, b) > 1 and 0 < x < 1: R dbeta's
/// `(a + b - 1) dbinom_raw(a - 1, a + b - 2, x, fl(1 - x))` (Loader 2000) in Excel's arithmetic,
/// EXP(LN(n + 1) + E) with E [`excel_dbinom_exponent`]'s, the sum held in an x87 register into
/// Excel's EXP (statistics.md "Beta density for min(a,b) > 1 is Loader's saddle point", T-2464,
/// T-2513). k = fl(a - 1), n = fl(fl(a + b) - 2) and nk = fl(n - k) are x87 stored operations. No
/// switch at large shapes (no a0 = 8 or a + b = 171.6 branch). `None` outside that domain or where
/// a piece is not finite, where the caller keeps its other forms.
pub(crate) fn excel_beta_density(x: f64, a: f64, b: f64) -> Option<f64> {
    if !(a > 1.0 && b > 1.0 && x > 0.0 && x < 1.0 && a.is_finite() && b.is_finite()) {
        return None;
    }
    let (k, n, nk) = beta_density_counts(a, b);
    let q = extended_sum(1.0, -x);
    let exponent = excel_dbinom_exponent(k, n, nk, x, q)?;
    let ln_scale = excel_ln(extended_sum(n, 1.0));
    let total = dd_add((ln_scale, 0.0), exponent);
    if !total.0.is_finite() {
        return None;
    }
    let density = excel_exp_extended(Register::from_double_double(total));
    density.is_finite().then_some(density)
}

/// Where Excel's t density leaves Loader's bd0 form for the log form: `xx > T_LOG_BRANCH_CUT * n`
/// (R 2.x's cut is 0.2). Chosen by scoring the cut on the in-sample probes t2513-r3-tdensity-a1/-a2;
/// on the verifier's hold-out 1 gives 505 exact cells against 391 for 0.2 and 417 for no log
/// branch (hold2: 573, 480, 520; statistics.md "Student t density is Loader's dt"). It is applied over
/// the whole range below, which the hold-outs, the 19,793 T.DIST(.,FALSE) rows of
/// docs/research/t2464/cells.tsv and the fresh Gate A batch r4-T-2585 measure: outside the fitted
/// df 3..100, |t| 0.31..4.3 the cut still scores at least as well as 0.2, no log branch and the
/// accurate density on every count (decision 0299).
/// Applied for every df >= 3 where t·t is finite; the range below gives the measured extremes.
/// FITTED(T-2513, 2406 cells, fitted on df 3..100, |t| 0.31..4.3; measured on df 3..2.7E297, |t| up to 1.1E153)
const T_LOG_BRANCH_CUT: f64 = 1.0;

/// The bound on [`loader_t_constant`]'s series terms; at n = 3 (ratio 1/3) it reaches 1E-33 in
/// about 70.
const T_CONSTANT_MAX_TERMS: u32 = 400;

/// The exact shape constant of Loader's t density,
/// `-bd0(n/2, (n+1)/2) + stirlerr((n+1)/2) - stirlerr(n/2)`, as a double-double. The bd0 part is
/// `(n/2) ln(1 + 1/n) - 1/2 = sum_{k>=2} (-1)^(k+1) / (2k n^(k-1))`, summed as that series so
/// that it keeps its digits at large n (it is about -1/(4n)). Excel's own constant is 1-2 units
/// of 2^-53 off it per df, unidentified (statistics.md "Student t density is Loader's dt", T-2586).
fn loader_t_constant(n: f64) -> Dd {
    let z = dd_div((1.0, 0.0), (n, 0.0));
    let mut power = z;
    let mut sum = (0.0, 0.0);
    for k in 2..T_CONSTANT_MAX_TERMS {
        let term = dd_div_small(power, f64::from(2 * k));
        sum = if k % 2 == 0 {
            dd_add(sum, (-term.0, -term.1))
        } else {
            dd_add(sum, term)
        };
        if term.0.abs() < 1e-36 * sum.0.abs() {
            break;
        }
        power = dd_mul(power, z);
    }
    let stirling = dd_add(stirlerr_exact(0.5 * (n + 1.0)), {
        let half = stirlerr_exact(0.5 * n);
        (-half.0, -half.1)
    });
    dd_add(sum, stirling)
}

/// The denominator fl(pi fl(1 + xx)) of Excel's df-1 density, xx = fl(t t).
fn df_one_denominator(xx: f64) -> f64 {
    extended_product(std::f64::consts::PI, extended_sum(1.0, xx))
}

/// Whether Excel's T.DIST(t, 1, FALSE) is `#DIV/0!`: fl(t t) is finite but the denominator
/// fl(pi fl(1 + fl(t t))) overflows, |t| from about 7.56E+153 to 2^512 (`T.DIST(7.6E+153,1,FALSE)`
/// is `#DIV/0!`, `T.DIST(7.55E+153,1,FALSE)` is 0; probes/t3554-tdist-density-df1-div0 A8:A10,
/// probes/t3554-tdist-density-huge-x A1, A18; statistics.md "T.DIST's density is #NUM! at a huge x").
pub(crate) fn excel_t_density_df_one_overflows(t: f64) -> bool {
    let xx = extended_product(t, t);
    xx.is_finite() && !df_one_denominator(xx).is_finite()
}

/// Excel's T.DIST(t, n, FALSE) for a truncated df n >= 1 (statistics.md "Student t density is
/// Loader's dt", T-2513, T-2585), every operation an x87 one stored to a double:
/// - df 1: `fl(1/fl(pi fl(1 + fl(t t))))`; df 2: `EXP(fl(-1.5 LN(fl(2 + fl(t t)))))`;
/// - df >= 3, Loader's saddle point: xx = fl(t t), q = fl(xx/n), w = fl(1 + q); when xx > n,
///   u = fl(fl(LN(w) n)/2) ([`T_LOG_BRANCH_CUT`]), otherwise u = fl(-bd0(fl(n/2), fl(fl(n + xx)/2)) + fl(xx/2)) with
///   Excel's bd0 held in a register ([`excel_bd0_unstored`]); then E = fl(C - u) with the exact constant C
///   ([`loader_t_constant`]) held in an x87 register, and
///   `fl(fl(EXP(E)/fl(sqrt(w))) / sqrt(2 pi))`, `1/sqrt(2 pi)` the correctly rounded double.
///
/// On the 19,793 T.DIST(.,FALSE) rows of docs/research/t2464/cells.tsv (the research's 19,776
/// plus the 17 rows of m9-t-nonint-df, m9-t-edges and t1518-t-df-cut) the engine is within 15 digits on 18,646 of the
/// 18,714 df >= 3 rows and bit-exact on all 1,079 df 1 and df 2 rows. `None` for a non-finite piece (t t
/// overflowing), where the caller keeps its other form.
pub(crate) fn excel_t_density(t: f64, n: f64) -> Option<f64> {
    if !(n >= 1.0 && t.is_finite() && n.is_finite()) {
        return None;
    }
    let xx = extended_product(t, t);
    if !xx.is_finite() {
        return None;
    }
    if n == 1.0 {
        // A subnormal quotient is 0 (probes/t3554-tdist-density-df1-div0 A3:A8, |t| 3.8E+153 to
        // 7.55E+153), as Excel's `/` flushes one (numbers.md "Subnormal results become 0").
        let density = flush_subnormal(extended_quotient(1.0, df_one_denominator(xx)));
        return density.is_finite().then_some(density);
    }
    if n == 2.0 {
        let exponent = extended_product(-1.5, excel_ln(extended_sum(2.0, xx)));
        let density = excel_exp(exponent);
        return density.is_finite().then_some(density);
    }
    let q = extended_quotient(xx, n);
    let w = extended_sum(1.0, q);
    let u = if xx > T_LOG_BRANCH_CUT * n {
        extended_quotient(extended_product(excel_ln(w), n), 2.0)
    } else {
        let mean = extended_quotient(extended_sum(n, xx), 2.0);
        excel_bd0_unstored(extended_quotient(n, 2.0), mean)
            .neg()
            .add(Register::load(extended_quotient(xx, 2.0)))
            .store()
    };
    if !u.is_finite() {
        return None;
    }
    let exponent = Register::from_double_double(loader_t_constant(n))
        .add(Register::load(-u))
        .store();
    let scaled = extended_quotient(excel_exp(exponent), extended_sqrt(w));
    // Excel's last multiply flushes a subnormal product to 0, as `*` does (numbers.md "Subnormal
    // results become 0"; Gate A T.DIST/r4-T-2585 A138: T.DIST(-1.1002374143529E+20,15,FALSE) is 0).
    let density = flush_subnormal(extended_product(scaled, FRAC_1_SQRT_2PI));
    density.is_finite().then_some(density)
}

/// The ratio cut of Loader's own bd0, R nmath `bd0.c` (`fabs(x-np) < 0.1*(x+np)`), copied verbatim:
/// the accurate form, not Excel's ([`EXCEL_BD0_CUT`]).
const LOADER_BD0_CUT: f64 = 0.1;

/// Loader's deviance bd0(x, M) = x ln(x / M) + M - x, accurately (R nmath `bd0.c`, Loader 2000):
/// below the cut the odd series in v = (x - M) / (x + M), which avoids the cancellation of the log
/// form near x = M, else the log form. M is a double-double, so the rounding of the caller's M
/// (a product and a quotient of counts) does not reach the result: its low part enters through
/// the first-order term d bd0 / dM = 1 - x / M, exact to O(lo^2 / M). Not Excel's bd0
/// ([`excel_bd0`], an x87 model fitted to BETA.DIST and T.DIST cells): this is the math, for the
/// kernels whose target is the Gate A envelope (HYPGEOM.DIST past its walk, T-4502).
///
/// For x > 0 and M > 0; the series has at most [`BD0_MAX_TERMS`] terms.
pub(crate) fn loader_bd0(x: f64, m: Dd) -> f64 {
    let correction = (1.0 - x / m.0) * m.1;
    let d = x - m.0;
    if d.abs() < LOADER_BD0_CUT * (x + m.0) {
        let mut v = d / (x + m.0);
        let mut sum = d * v;
        if sum.abs() < f64::MIN_POSITIVE {
            return sum + correction;
        }
        let mut ej = 2.0 * x * v;
        v *= v;
        for j in 1..BD0_MAX_TERMS {
            ej *= v;
            let next = sum + ej / f64::from(2 * j + 1);
            if next == sum {
                break;
            }
            sum = next;
        }
        return sum + correction;
    }
    x * (x / m.0).ln() + (m.0 - x) + correction
}

/// ln of the binomial probability of x successes in n trials, `dbinom_raw(x, n, p, q, log = TRUE)`
/// of R nmath `dbinom.c` (Loader's saddle point), with the means n p and n q given as
/// double-doubles: stirlerr(n) - stirlerr(x) - stirlerr(n - x) - bd0(x, n p) - bd0(n - x, n q) less
/// half of ln(2 pi x (n - x) / n) ([`stirlerr_exact`], [`loader_bd0`]), and R's own forms at x = 0
/// and x = n. For whole x and n with 0 <= x <= n, and n p, n q >= 0 summing to n; -inf where the
/// probability is 0. Each piece is within a few ulp of its exact value and none is much larger than
/// the result (the deviances are non-negative), so the result is within a few units of 2^-52 of its
/// own magnitude, plus about 1E-15.
pub(crate) fn loader_ln_binomial(x: f64, n: f64, np: Dd, nq: Dd) -> f64 {
    if np.0 == 0.0 {
        return if x == 0.0 { 0.0 } else { f64::NEG_INFINITY };
    }
    if nq.0 == 0.0 {
        return if x == n { 0.0 } else { f64::NEG_INFINITY };
    }
    if x == 0.0 {
        if n == 0.0 {
            return 0.0;
        }
        // R: p < 0.1 ? -bd0(n, n q) - n p : n ln q, with p = n p / n.
        return if np.0 < 0.1 * n {
            -loader_bd0(n, nq) - (np.0 + np.1)
        } else {
            n * ((nq.0 + nq.1) / n).ln()
        };
    }
    if x == n {
        return if nq.0 < 0.1 * n {
            -loader_bd0(n, np) - (nq.0 + nq.1)
        } else {
            n * ((np.0 + np.1) / n).ln()
        };
    }
    if !(x > 0.0 && x < n) {
        return f64::NEG_INFINITY;
    }
    let nx = n - x;
    let stirling = stirlerr_exact(n).0 - stirlerr_exact(x).0 - stirlerr_exact(nx).0;
    let deviance = loader_bd0(x, np) + loader_bd0(nx, nq);
    let lf = LN_2PI + x.ln() + (-x / n).ln_1p();
    stirling - deviance - 0.5 * lf
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "test: a malformed fixture fails the test"
    )]
    use super::*;
    use crate::test::util::{ulps, within_fifteen_digits};

    // Loader's count n - k at alpha 7.9E+15 to 2E+16, beta 2: BETA.DIST(1-2^-51,a,2,FALSE) is
    // #NUM! exactly where it is 0 (probes/t4713-beta-inv-cut-pieces H7, H11 #NUM!; H9, H37 numbers).
    #[test]
    fn beta_density_counts_vanish_where_excel_density_is_num() {
        assert!(beta_density_counts_vanish(9.120_108_393_559_12e15, 2.0));
        assert!(beta_density_counts_vanish(1e16, 2.0));
        assert!(!beta_density_counts_vanish(9.549_925_860_214_37e15, 2.0));
        assert!(!beta_density_counts_vanish(1.819_700_858_609_99e16, 2.0));
        assert!(!beta_density_counts_vanish(1e15, 2.0));
        assert!(!beta_density_counts_vanish(2.0, 1e16));
    }

    // Loader's binomial log density against exact small binomial probabilities, in each of R's
    // branches: the saddle point inside (with bd0's series near the mean and its log form away
    // from it), and x = 0 and x = n with p and q on both sides of 0.1.
    #[test]
    fn loader_ln_binomial_is_the_exact_log() {
        let cases: [(f64, f64, f64, f64); 7] = [
            (3.0, 10.0, 0.5, 120.0 / 1024.0),
            (5.0, 10.0, 0.5, 252.0 / 1024.0),
            (2.0, 5.0, 0.3, 10.0 * 0.09 * 0.343),
            (0.0, 5.0, 0.3, 0.168_07),
            (0.0, 20.0, 0.05, 0.95f64.powi(20)),
            (20.0, 20.0, 0.95, 0.95f64.powi(20)),
            (5.0, 5.0, 0.3, 0.002_43),
        ];
        for (x, n, p, exact) in cases {
            let ours = loader_ln_binomial(x, n, (n * p, 0.0), (n - n * p, 0.0));
            assert!(
                (ours - exact.ln()).abs() < 1E-14,
                "b({x}; {n}, {p}): {ours} {}",
                exact.ln()
            );
        }
        // A mean's low part moves bd0 to first order: bd0(x, M + lo) = bd0(x, M) + (1 - x/M) lo.
        let shifted = loader_bd0(10.0, (9.0, 1E-12));
        let expected = loader_bd0(10.0, (9.0, 0.0)) + (1.0 - 10.0 / 9.0) * 1E-12;
        assert_eq!(shifted, expected);
        assert_eq!(loader_bd0(7.0, (7.0, 0.0)), 0.0);
    }

    #[test]
    fn stirlerr_exact_matches_mpmath() {
        // mpmath (50 digits) ln Gamma(n + 1) - (n + 1/2) ln n + n - ln sqrt(2 pi) at the same
        // doubles, as hi + lo: the recurrence below 15 and the series from 15 on.
        for (n, hi, lo) in [
            (1e-10, 10.593_986_934_110_418, 7.124_858_491_460_29e-16),
            (0.03, 0.952_950_298_497_347_8, -9.226_922_120_730_322e-19),
            (0.5, 0.153_426_409_720_027_36, -1.159_523_406_923_149_8e-17),
            (1.0, 0.081_061_466_795_327_26, -2.850_421_842_770_954_6e-18),
            (4.3, 0.019_345_426_904_332_788, 2.960_181_844_418_298e-19),
            (7.5, 0.011_104_559_758_206_917, -2.038_131_079_125_446e-20),
            (14.99, 0.005_558_438_081_636_329, 2.937_702_605_005_666e-19),
            (15.0, 0.005_554_733_551_962_801, 3.185_347_414_196_856e-19),
            (
                16.0,
                0.005_207_655_919_609_640_4,
                -2.883_608_994_661_908_6e-21,
            ),
            (
                100.0,
                0.000_833_330_555_634_914_7,
                1.246_488_391_106_547_6e-20,
            ),
        ] {
            let (got_hi, got_lo) = stirlerr_exact(n);
            let error = (got_hi - hi) + (got_lo - lo);
            assert!(
                error.abs() < 1e-24 * hi.max(1.0),
                "stirlerr({n}): {error:e}"
            );
        }
    }

    #[test]
    fn excel_beta_density_matches_excel() {
        // Excel's BETA.DIST(x, a, b, FALSE) bit for bit (the arguments are the doubles of each
        // cell's formula; the model is bit-exact on about 20-30% of cells, T-2586): both bd0 sides
        // in the log form (probes-numeric t2513-r3-loader-const-hold1 A342, A369, A3251), a series
        // side (t2513-r3-loader-bd0-a A4814, A5192, A5787; t2513-verify-r3-loader-bd0-hold A649,
        // A1214), integer and half-integer shapes (t2513-r3-loader-bd0-b A2760, A3666), a shape near 1
        // and min >= 8 (t2464-r2-density-large-p1 A9967, A7281) and diverse shapes
        // (t2513-r3-loader-const-diverse1 A252, A1307).
        let cases: [(f64, f64, f64, f64); 14] = [
            (
                0.376_922_732_080_715_4,
                8.3,
                5.899_999_999_999_998,
                0.886_766_852_474_694_8,
            ),
            (
                0.365_288_592_308_167_5,
                8.3,
                5.899_999_999_999_999_5,
                0.772_288_531_152_508_3,
            ),
            (
                0.444_288_270_094_662_3,
                33.976,
                14.85,
                0.009_542_680_147_732_819,
            ),
            (0.523_647_654_910_737_2, 5.5, 12.5, 0.534_367_175_242_143_3),
            (0.344_717_003_125_003_9, 5.5, 12.5, 3.187_884_049_445_971_7),
            (0.192_030_067_200_634_6, 5.5, 12.5, 2.547_970_230_648_507),
            (0.128_688_030_000_000_23, 9.7, 21.3, 0.258_876_334_448_779_7),
            (
                0.707_297_924_512_993_8,
                4.75,
                11.25,
                0.011_149_186_257_390_1,
            ),
            (0.247_103_647_826_088_96, 3.0, 5.0, 2.060_100_389_491_967),
            (0.679_114_979_411_773_7, 4.5, 4.5, 1.439_649_122_004_031),
            (0.099_840_000_000_003_4, 1.1, 1.9, 1.579_203_777_920_108_2),
            (0.165_435_806_451_616, 8.5, 40.0, 7.447_826_760_045_769),
            (
                0.781_164_318_528_283_5,
                7.3,
                7.501,
                0.232_804_337_859_794_24,
            ),
            (0.616_917_685_110_905_7, 8.67, 1.45, 0.428_699_138_085_424_5),
        ];
        for (x, a, b, excel) in cases {
            let got = excel_beta_density(x, a, b);
            assert_eq!(
                got.map(f64::to_bits),
                Some(excel.to_bits()),
                "BETA.DIST({x},{a},{b},FALSE) = {got:?}, Excel {excel}"
            );
        }
    }

    #[test]
    fn excel_beta_density_domain() {
        // Outside min(a, b) > 1 and 0 < x < 1, and where n x underflows to 0, the caller keeps
        // its other forms.
        assert_eq!(excel_beta_density(0.5, 1.0, 3.0), None);
        assert_eq!(excel_beta_density(0.5, 3.0, 0.5), None);
        assert_eq!(excel_beta_density(0.0, 3.0, 3.0), None);
        assert_eq!(excel_beta_density(1.0, 3.0, 3.0), None);
        assert_eq!(excel_beta_density(5e-324, 1.5, 3.0), None);
        assert_eq!(excel_beta_density(f64::NAN, 3.0, 3.0), None);
    }

    /// The two hold-outs of `test/fixtures/t_density_hold.fixture`: (t, n, Excel's T.DIST(t,n,FALSE)).
    fn t_density_holdouts() -> [Vec<(f64, f64, f64)>; 2] {
        let bits = |field: &str| f64::from_bits(u64::from_str_radix(field, 16).unwrap());
        let mut parts = [Vec::new(), Vec::new()];
        let mut part = 0;
        for line in include_str!("../../test/fixtures/t_density_hold.fixture").lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            match fields.as_slice() {
                [] => part = 1,
                ["#", ..] => {}
                [t, n, excel] => parts[part].push((bits(t), n.parse().unwrap(), bits(excel))),
                _ => panic!("malformed fixture line {line:?}"),
            }
        }
        parts
    }

    // T-2585 acceptance: the research model LT1 with the exact constant scores 573 exact / 960 within
    // 4 ulp / 960 within 15 digits on t2513-r3-tdensity-hold2 and 505 / 918 / 977 on the verifier's
    // t2513-verify-r3-tdensity-hold (df 13-50); the correctly rounded density gets 266 / 776 / 839 and
    // 93 / 555 / 787 (docs/research/t2464/r3-tdensity RESULTS.md, verify-r3-tdensity).
    #[test]
    fn t_density_holdouts_score_as_the_loader_model() {
        let [hold2, verify] = t_density_holdouts();
        assert_eq!((hold2.len(), verify.len()), (960, 1000));
        for (rows, min_exact, min_ulps, min_fifteen) in
            [(hold2, 573, 960, 960), (verify, 505, 918, 977)]
        {
            let (mut exact, mut close, mut fifteen) = (0, 0, 0);
            for (t, n, excel) in rows {
                let density = excel_t_density(t, n).unwrap_or(f64::NAN);
                exact += usize::from(density == excel);
                close += usize::from(ulps(density, excel) <= 4);
                fifteen += usize::from(within_fifteen_digits(excel, density));
            }
            assert!(
                exact >= min_exact && close >= min_ulps && fifteen >= min_fifteen,
                "{exact} / {close} / {fifteen}"
            );
        }
    }

    #[test]
    fn t_density_closed_forms_at_df_one_and_two() {
        // fl(1/fl(pi fl(1 + t t))) and EXP(fl(-1.5 LN(fl(2 + t t)))); T.DIST(0,1,FALSE) is 1/pi.
        assert_eq!(excel_t_density(0.0, 1.0), Some(1.0 / std::f64::consts::PI));
        assert_eq!(
            excel_t_density(0.0, 2.0),
            Some(excel_exp(-1.5 * excel_ln(2.0)))
        );
        assert_eq!(excel_t_density(1e200, 5.0), None);
        // Gate A T.DIST/r4-T-2585 A138: the subnormal product is flushed, as Excel's `*` does.
        assert_eq!(excel_t_density(-1.1002374143529E+20, 15.0), Some(0.0));
        assert_eq!(excel_t_density(f64::NAN, 5.0), None);
        // probes/t3554-tdist-density-df1-div0 A1, A2 (normal) and A3 (subnormal, flushed).
        assert_eq!(
            excel_t_density(3.7e153, 1.0),
            Some(2.325_126_999_151_137_3e-308)
        );
        assert_eq!(
            excel_t_density(-3.75e153, 1.0),
            Some(2.263_536_968_418_066_7e-308)
        );
        assert_eq!(excel_t_density(3.8e153, 1.0), Some(0.0));
    }

    #[test]
    fn df_one_denominator_overflow_brackets_excel() {
        // probes/t3554-tdist-density-df1-div0 A8:A10 and probes/t3554-tdist-density-huge-x A1, A2.
        // 2^512 - 2^460, two doubles below 2^512 (the probed H2; 2^512's predecessor is unprobed).
        let two_below = 2f64.powi(512) * (1.0 - f64::EPSILON);
        assert!(!excel_t_density_df_one_overflows(7.55e153));
        assert!(excel_t_density_df_one_overflows(7.6e153));
        assert!(excel_t_density_df_one_overflows(-two_below));
        assert!(!excel_t_density_df_one_overflows(2f64.powi(512)));
    }
}
