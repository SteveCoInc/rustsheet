//! TOMS 708 `bup` and bratio's L140 for x0 <= 0.7 (SPEC-0025's `bup.rs`): w = bup(b0', a0, y0, x0, n) +
//! bpser(a0, b0', x0).

use super::super::beta_density::beta_density_small_shape;
use super::bgrat::{bgrat_increment, Dd, MAX_Z};
use super::bpser::bpser;
use super::EPS;
use crate::arithmetic::{extended_quotient, extended_sum};
use crate::x87_exp::{dd_add, dd_exp, dd_mul, excel_exp, two_sum};
use crate::x87_log10::{excel_ln, ln_double_double_pair};
use crate::x87_register::Register;

/// The most bup terms L140 bup+bgrat takes T-3568's composition for: its residual grows with
/// (n - 1)·|ln y0| from n = 10 (docs/research/t2464/r5-bup RESULTS.md section 5), and on its probes
/// every cell the port moves from within 15 digits to visibly off has n >= 19 (T-3561). Past it
/// today's kernel answers.
const MAX_BUP_TERMS: f64 = 9.0;

/// The largest a0 L140 bup+bgrat takes T-3568's composition for at n >= 2 or c = 1. T-3568's probes
/// cover a0 <= 3000; past that, up to 1E7, the fallback kernel is what is off: on F's calls at a
/// small df1 (4 to 20) and df2 of 1E5 to 2E7 it is up to 3.7E-10 relative off the 50-digit math,
/// and this composition within 8E-15 (T-3754, docs/research/t3754: `F.DIST.RT(1.09080885395379,8,
/// 5567623)` 0.3658936312385961 against Excel's correctly rounded 0.3658936312909749). Above 1E7
/// it stops: Excel's W tracks its density's error, which grows with a0, the engine's density is not
/// Excel's to the bit there (T-3569), and at a0 = 5E9 the port takes probes-numeric
/// t2528-f-dist-huge-df2 A4 and A16 from within 4 ulp to visibly off.
const MAX_BUP_A0: f64 = 1e7;

/// bratio's split of L140's b0 into n bup terms and the lowest shape b0' = b0 - n (b0' = 1 and
/// n - 1 at an integer b0).
fn bup_split(b0: f64) -> (f64, f64) {
    // b0 < 40 on every L140 call, so n is a small whole number of bup terms.
    let n = b0.trunc();
    let b0_frac = b0 - n;
    if b0_frac == 0.0 {
        (n - 1.0, 1.0)
    } else {
        (n, b0_frac)
    }
}

/// Whether L140 bup+bgrat at this b0 takes T-3568's n-term composition (n >= 2 bup terms, or one
/// term at c = 1, b0 = 2) rather than T-3382's one term at 1 < b0 < 2.
pub(super) fn bup_takes_terms(b0: f64) -> bool {
    let (n, c) = bup_split(b0);
    n > 1.0 || c == 1.0
}

/// How L140 bup+bpser forms bup's sum W.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum BupSum {
    /// T-3568's power sum ([`bup_power_sum`]), BETA.DIST's and F's (T-3198).
    Powers,
    /// TOMS's recurrence ([`bup`]), BETA.INV's search.
    Recurrence,
}

/// bratio's L140 for x0 <= 0.7 (a, b > 1, b0 < 40, b0·x0 > 0.7): with n = int(b0) and
/// b0' = b0 - n (b0' = 1 and n - 1 at an integer b0), w = bup(b0', a0, y0, x0, n) + bpser(a0, b0', x0)
/// (statistics.md "Incomplete beta L140 and bfrac: the prefactor is Excel's own density times x·y";
/// model `w4-fractions/tomsx.py` `l140_best`, `bratio_excel.py` `bup` with `bup_arith = "w4"` and
/// `brcmp1 = "pdf"`).
///
/// bup's sum W is T-3568's power sum ([`bup_power_sum`]), not TOMS's recurrence: on the L140
/// bup+bpser BETA.DIST cells of probes-numeric t3198-f-l140-args it is bit-exact on 674 of 816
/// against 601 (hold-out half: 341 of 408 against 299), and on the bratio fixture's rows on 99 of
/// 122 against 91 (T-3198; statistics.md "Incomplete beta L140 bup+bpser at the F points").
///
/// BETA.INV's search takes `BupSum::Recurrence` (`bratio::beta_search_tails`).
pub(super) fn l140_bup_bpser(a0: f64, b0: f64, x0: f64, y0: f64, sum: BupSum) -> Option<f64> {
    let (n, b0_frac) = bup_split(b0);
    let up = match sum {
        BupSum::Powers => bup_with_power_sum(b0_frac, a0, y0, x0, n)?,
        BupSum::Recurrence => bup(b0_frac, a0, y0, x0, n)?,
    };
    let series = bpser(a0, b0_frac, x0)?;
    Some(up + series)
}

/// bup's prefactor in L140, ret = x87(P·fl(x0·y0)/c) held in a register, P the Beta(c, a0) density at
/// y0 ([`l140_bup_bgrat`] multiplies it by S; [`l140_bup_prefactor_underflows`] reads its size).
fn l140_bup_ret(density: f64, c: f64, x0: f64, y0: f64) -> Register {
    Register::load(density)
        .mul(Register::load(x0 * y0))
        .div(Register::load(c))
}

/// Whether Excel's L140 bup+bgrat tail w is exactly 0 because bup's prefactor ([`l140_bup_ret`]) is
/// below the smallest normal double: Excel's ret is then 0, and bgrat's increment, at z > 708 there,
/// is subnormal and flushed. A density below the normal range gives a smaller ret, so it is 0 too.
/// It holds at any a0 (it reads only the prefactor, not [`MAX_BUP_A0`]'s port). On probes-numeric
/// t4045-f-dist-rt-zero-cut, ln(x0^a0·y0^c/B(c, a0)) brackets Excel's last number and first 0 at
/// (-707.42, -708.42), (-708.38, -709.38) and (-708.28, -709.28) in its three families (df1 30, 20
/// and 31 at df2 1E8, 1E7 and 1E8), which meet only around `ln(DBL_MIN)` = -708.396:
/// `F.DIST.RT(47.6,30,1E8)` is a number and `F.DIST.RT(47.6666666667,30,1E8)` is 0 (statistics.md
/// "Incomplete beta at a huge first shape", T-4045). BETA.DIST's cdf takes the same edge
/// (probes/t4456-beta-t-zero-cut C and D, a0 = 5E6 at b0 = 25.5 and 30: the first 0 is between
/// ln ret -708 and -709, T-4456). F's tails and BETA.DIST's worksheet cdf (T-3764) ask it; the
/// cdf under F.INV's search keeps today's kernel there (`F.INV(1.43795578761177E-262,107702,58)`,
/// accuracy/F.INV/s1 A146). The t never reaches L140 (its second shape is 1/2).
pub(super) fn l140_bup_prefactor_underflows(a0: f64, b0: f64, x0: f64, y0: f64) -> bool {
    let (_, c) = bup_split(b0);
    beta_density_small_shape(y0, c, a0).is_some_and(|density| {
        !density.is_normal() || !l140_bup_ret(density, c, x0, y0).store().is_normal()
    })
}

/// Whether Excel's L140 bup+bgrat tail w is exactly 0 at n >= 3 bup terms because bup's sum
/// W = ret·S ([`l140_bup_ret`] times [`bup_power_sum`]) is below 2^-970, the smallest normal double
/// divided by the double epsilon, though W itself is a normal double there. probes/
/// t4437-beta-t-zero-bracket walks BETA.DIST by 0.5 in ln ret at (a, b) = (5E6, 3.5), (5E6, 4),
/// (1E6, 3.5) and (5E6, 5.5). Each family's last number has W between 1.10 and 1.60 times 2^-970,
/// and its first 0 has W between 0.67 and 0.97 times it (`BETA.DIST(0.999862932,5E6,3.5,TRUE)`
/// row 9 is 1.1078134827430778E-292, row 10 is 0). The domain n >= 3 is part of the rule as frozen
/// (docs/research/t4437/freeze2.md): the second fresh sweep after it, eight untouched families with
/// n from 3 to 7 (probes/t4437-beta-zero-fresh2, rows 0.05 nats apart), puts 176 of 176 cells on the
/// predicted side, every cut within 0.025 nats of 2^-970 (statistics.md "Where the exact 0 starts
/// (T-4045)"). At n = 2 Excel's cut lies below the edge and moves with a0 (T-4564).
pub(super) fn l140_bup_sum_flushed(a0: f64, b0: f64, x0: f64, y0: f64) -> bool {
    /// 2^-970: Excel's L140 bup+bgrat tail is 0 below it at n >= 3.
    const FLUSH: f64 = f64::MIN_POSITIVE / f64::EPSILON;
    let (n, c) = bup_split(b0);
    n >= 3.0
        && beta_density_small_shape(y0, c, a0)
            .filter(|density| density.is_normal())
            .is_some_and(|density| {
                let w = l140_bup_ret(density, c, x0, y0)
                    .mul(Register::load(bup_power_sum(c, a0, y0, x0, n)))
                    .store();
                w < FLUSH
            })
}

/// bratio's L140 for x0 > 0.7 and a0 > 15 (b0 < 40): w = bgrat(a0, c, x0, y0, bup(c, a0, y0, x0, n))
/// with n and c = b0 - n from [`bup_split`], as Excel computes it, RN(W + D) (statistics.md "Incomplete
/// beta: the integrated model", L140 bup+bgrat at 1 < b < 2 (T-3382) and with n >= 2 bup terms
/// (T-3568, docs/research/t2464/r5-bup `model.py`)):
/// - W = fl(ret·S), one rounding of the register product, with ret = x87(P·fl(x0·y0)/c), P the beta
///   density in bup's own orientation, Beta(c, a0) at y0 (the engine's [`beta_density_small_shape`]),
///   and S = [`bup_power_sum`] (1 at n = 1, where W is ret rounded once);
/// - D = `I_x0(a0, c)`, accurate: the bgrat increment ([`bgrat_increment`]) for c < 1, and x0^a0
///   for c = 1 ([`power_increment`]).
///
/// ret's single register rounding is not [`bup`]'s fl(fl(P·fl(x·y))/a): each form is the one its own
/// branch's evidence picked. Here the register form wins on the hold-out half of T-2512's cells,
/// 1,306 of 1,613 bit-exact against 1,046 for the stored one (r3-bgrat RESULTS.md finding 1,
/// `bupdens.out`), and T-3568's n >= 2 model takes it too. On [`bup`]'s own callers the prefactor's
/// rounding is not yet scored (`bup_arith = "w4"` covers only its W loop): T-3770.
///
/// `None` past [`MAX_BUP_TERMS`] or [`MAX_BUP_A0`] at n >= 2 or c = 1, where either piece is not
/// finite or bgrat's z is out of its range, so the caller keeps today's kernel.
pub(super) fn l140_bup_bgrat(a0: f64, b0: f64, x0: f64, y0: f64) -> Option<f64> {
    let (n, c) = bup_split(b0);
    if bup_takes_terms(b0) && !(n <= MAX_BUP_TERMS && a0 <= MAX_BUP_A0) {
        return None;
    }
    // A density at or past the subnormal edge has lost the digits W needs (it underflows to 0
    // while W, times S's large powers, is still a normal double): today's kernel keeps those.
    let density = beta_density_small_shape(y0, c, a0).filter(|p| p.is_normal())?;
    let up = l140_bup_ret(density, c, x0, y0)
        .mul(Register::load(bup_power_sum(c, a0, y0, x0, n)))
        .store();
    let increment = if c == 1.0 {
        power_increment(a0, y0)?
    } else {
        bgrat_increment(a0, c, x0, y0)?
    };
    let w = dd_add((up, 0.0), increment).0;
    w.is_finite().then_some(w)
}

/// Whether bratio's L140 bup sum overflows at this point (`b0 < 40` after the swap, either side of
/// x0 = 0.7): S's running product of ratios (apb + l)/(ap1 + l) ([`bup_power_sum`]) passes the
/// largest double, so S is not finite and Excel's incomplete beta is `#NUM!`. On probes/
/// t3576-negbinom-cdf-huge-s every L140 cell whose ln of that product is 713.72 or more is `#NUM!`
/// and every one at 707.81 or less is a value (ln of the largest double is 709.78), bup + bpser
/// and bup + bgrat alike, and BETA.DIST with the same shapes is `#NUM!` at the same points
/// (statistics.md "NEGBINOM.DIST's cdf is #NUM! at a huge `number_s`", T-3576).
pub(super) fn l140_bup_sum_overflows(a0: f64, b0: f64, x0: f64, y0: f64) -> bool {
    let (n, c) = bup_split(b0);
    n > 1.0 && !bup_power_sum(c, a0, y0, x0, n).is_finite()
}

/// bup's sum S = 1 + `Σ_{i<n} t_i` of `bup(a, b, x, y, n)` as Excel forms it in L140 bup+bgrat
/// (T-3568, r5-bup `model.py` `bup_sum`) and, for BETA.DIST and F, L140 bup+bpser (T-3198):
/// `t_i = fl(R_i·EXP(fl(i·LN(x))))` with Excel's EXP and LN, `R_i` the running double product of
/// the stored ratios fl(fl(apb + l)/fl(ap1 + l)), each partial sum stored, and TOMS's k and eps
/// break ([`bup_break_after`]). TOMS's recurrence d·(r·x) in any
/// arithmetic leaves an sd of 17.8 units on its cells against 1.6 for this form at n <= 9
/// (statistics.md "L140 bup+bgrat with n >= 2 bup terms").
fn bup_power_sum(a: f64, b: f64, x: f64, y: f64, n: f64) -> f64 {
    let k = bup_break_after(a, b, x, y, n - 1.0);
    let (apb, ap1) = (a + b, a + 1.0);
    let ln_x = excel_ln(x);
    let (mut ratio, mut sum) = (1.0, 1.0);
    let mut i = 1.0;
    while i < n {
        let l = i - 1.0;
        ratio *= (apb + l) / (ap1 + l);
        let term = ratio * excel_exp(i * ln_x);
        sum += term;
        if l >= k && term <= EPS * sum {
            break;
        }
        i += 1.0;
    }
    sum
}

/// `I_x0(a0, 1)` = x0^a0 in double-double, from the exact y0 = 1 - x0 (x0 > 0.7 is the rounded side
/// when bratio swapped): bgrat's increment at c = 1, where Q(1, z) = e^-z (T-3568's model takes the
/// accurate x0^a0 there). `None` past bgrat's z window ([`MAX_Z`], where [`dd_exp`]'s tail word
/// turns subnormal), as for c < 1.
fn power_increment(a0: f64, y0: f64) -> Option<Dd> {
    let ln_x0 = ln_double_double_pair(two_sum(1.0, -y0));
    let exponent = dd_mul((a0, 0.0), ln_x0);
    if exponent.0.is_nan() || exponent.0 < -MAX_Z {
        return None;
    }
    Some(dd_exp(exponent))
}

/// TOMS bup's k for `bup(a, b, x, y, n)` with nm1 = n - 1: the terms summed before the eps break may
/// stop the sum.
fn bup_break_after(a: f64, b: f64, x: f64, y: f64, nm1: f64) -> f64 {
    if b <= 1.0 {
        return 0.0;
    }
    if y <= 1e-4 {
        return nm1;
    }
    let r = (b - 1.0) * x / y - a;
    if r < 1.0 {
        0.0
    } else if r < nm1 {
        r.trunc()
    } else {
        nm1
    }
}

/// [`bup`] with its sum W formed as [`bup_power_sum`] (L140 bup+bpser, T-3198): the same stored
/// prefactor fl(fl(P·fl(x·y))/a), times S.
fn bup_with_power_sum(a: f64, b: f64, x: f64, y: f64, n: f64) -> Option<f64> {
    let ret = bup_prefactor(a, b, x, y)?;
    if n == 1.0 || ret == 0.0 {
        return Some(ret);
    }
    Some(ret * bup_power_sum(a, b, x, y, n))
}

/// bup's prefactor fl(fl(P·fl(x·y))/a), P Excel's own beta density of Beta(a, b) at x ([`bup`]).
fn bup_prefactor(a: f64, b: f64, x: f64, y: f64) -> Option<f64> {
    let density = beta_density_small_shape(x, a, b)?;
    Some(density * (x * y) / a)
}

/// TOMS 708 `bup(a, b, x, y, n)` = `I_x(a, b) - I_x(a + n, b)` for a whole n >= 1 and a <= 1, as
/// Excel computes it (`bratio_excel.py` `bup` and `_bup_term`, `bup_arith = "w4"`):
/// - the prefactor is Excel's own beta density P of Beta(a, b) at x (T-2503, a <= 1) times x·y, so
///   bup = fl(fl(P·fl(x·y))/a)·W, with no exp(mu) scaling;
/// - W = 1 + `d_1` + ... with `d_l = d_{l-1}·(r_l·x)`, `r_l = fl(fl(apb + l)/fl(ap1 + l))` stored and `r_l·x`
///   held in a register; each partial sum stored;
/// - the sum stops after term k (TOMS's (b - 1)x/y - a) once d <= eps·W.
pub(super) fn bup(a: f64, b: f64, x: f64, y: f64, n: f64) -> Option<f64> {
    let ret = bup_prefactor(a, b, x, y)?;
    if n == 1.0 || ret == 0.0 {
        return Some(ret);
    }
    let nm1 = n - 1.0;
    let k = bup_break_after(a, b, x, y, nm1);
    let (apb, ap1) = (a + b, a + 1.0);
    let x_reg = Register::load(x);
    let mut d = 1.0;
    let mut w = 1.0;
    let mut l = 0.0;
    while l < nm1 {
        let ratio = extended_quotient(apb + l, ap1 + l);
        d = Register::load(d)
            .mul(Register::load(ratio).mul(x_reg))
            .store();
        w = extended_sum(w, d);
        if l >= k && d <= EPS * w {
            break;
        }
        l += 1.0;
    }
    Some(ret * w)
}
