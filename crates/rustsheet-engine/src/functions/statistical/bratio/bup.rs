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

/// The largest a0 L140 bup+bgrat takes T-3568's composition for at n >= 2 or c = 1: the range its
/// probes cover (a0 <= 3000). Excel's W tracks its density's error, which grows with a0, and the
/// engine's density is not Excel's to the bit there (T-3569): at a0 = 5E9 the port takes
/// probes-numeric t2528-f-dist-huge-df2 A4 and A16 from within 4 ulp to visibly off.
const MAX_BUP_A0: f64 = 3000.0;

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

/// bratio's L140 for x0 <= 0.7 (a, b > 1, b0 < 40, b0·x0 > 0.7): with n = int(b0) and
/// b0' = b0 - n (b0' = 1 and n - 1 at an integer b0), w = bup(b0', a0, y0, x0, n) + bpser(a0, b0', x0)
/// (statistics.md "Incomplete beta L140 and bfrac: the prefactor is Excel's own density times x·y";
/// model `w4-fractions/tomsx.py` `l140_best`, `bratio_excel.py` `bup` with `bup_arith = "w4"` and
/// `brcmp1 = "pdf"`).
pub(super) fn l140_bup_bpser(a0: f64, b0: f64, x0: f64, y0: f64) -> Option<f64> {
    let (n, b0_frac) = bup_split(b0);
    let up = bup(b0_frac, a0, y0, x0, n)?;
    let series = bpser(a0, b0_frac, x0)?;
    Some(up + series)
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
    let ret = Register::load(density)
        .mul(Register::load(x0 * y0))
        .div(Register::load(c));
    let up = ret
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

/// bup's sum S = 1 + `Σ_{i<n} t_i` of `bup(a, b, x, y, n)` as Excel forms it in L140 bup+bgrat
/// (T-3568, r5-bup `model.py` `bup_sum`): `t_i = fl(R_i·EXP(fl(i·LN(x))))` with Excel's EXP and LN,
/// `R_i` the running double product of the stored ratios fl(fl(apb + l)/fl(ap1 + l)), each partial sum
/// stored, and TOMS's k and eps break ([`bup_break_after`]). TOMS's recurrence d·(r·x) in any
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

/// TOMS 708 `bup(a, b, x, y, n)` = `I_x(a, b) - I_x(a + n, b)` for a whole n >= 1 and a <= 1, as
/// Excel computes it (`bratio_excel.py` `bup` and `_bup_term`, `bup_arith = "w4"`):
/// - the prefactor is Excel's own beta density P of Beta(a, b) at x (T-2503, a <= 1) times x·y, so
///   bup = fl(fl(P·fl(x·y))/a)·W, with no exp(mu) scaling;
/// - W = 1 + `d_1` + ... with `d_l = d_{l-1}·(r_l·x)`, `r_l = fl(fl(apb + l)/fl(ap1 + l))` stored and `r_l·x`
///   held in a register; each partial sum stored;
/// - the sum stops after term k (TOMS's (b - 1)x/y - a) once d <= eps·W.
pub(super) fn bup(a: f64, b: f64, x: f64, y: f64, n: f64) -> Option<f64> {
    let density = beta_density_small_shape(x, a, b)?;
    let ret = density * (x * y) / a;
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
