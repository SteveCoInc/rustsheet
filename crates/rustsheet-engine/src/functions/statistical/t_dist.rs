use statrs::distribution::{Continuous, ContinuousCDF, StudentsT};

use crate::arithmetic::{extended_quotient, flush_subnormal};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{div_error, num_error, number_or_error, LiftCost};
use crate::functions::math_and_trigonometry::excel_tan;
use crate::functions::statistical::inverse_beta::{t_inverse_cdf, t_quantile_beta_root};
use crate::functions::statistical::loader::excel_t_density_df_one_overflows;
use crate::functions::statistical::paired::{MarkedSample, PairedSamplesError, PairedValues};
use crate::functions::statistical::special::{
    bisect_increasing, ln_gamma, nearest_crossing_from, refine_increasing,
};
use crate::functions::statistical::student_t::{
    student_t_centre, student_t_density, student_t_tail_at, student_t_tails, t_density,
};
use crate::functions::Function;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// Student-t quantile at one degree of freedom for the two-tailed probability `q` in (0, 1]:
/// `1/TAN(PI()*q/2)` through Excel's own TAN, not the true Cauchy quantile (statistics.md
/// "Student-t quantile at one degree of freedom is 1/TAN(π·p/2) through Excel's TAN", T-2858).
pub(crate) fn cauchy_two_tail_quantile(q: f64) -> f64 {
    // The worksheet `/` rounds twice (numbers.md "Division rounds twice"), which matters at the
    // round points: TINV(0.5,1) is 1, not 1.0000000000000002.
    extended_quotient(1.0, excel_tan(std::f64::consts::PI * q / 2.0))
}

fn mean(xs: &[f64]) -> f64 {
    let n = xs.len();
    if n == 0 {
        return 0.0;
    }
    let mut s = 0.0;
    for &x in xs {
        s += x;
    }
    s / (n as f64)
}

pub(crate) fn sample_var(xs: &[f64]) -> f64 {
    let n = xs.len();
    if n < 2 {
        return 0.0;
    }
    let m = mean(xs);
    let mut s = 0.0;
    for &x in xs {
        let d = x - m;
        s += d * d;
    }
    s / ((n - 1) as f64)
}

enum TTestType {
    Paired,
    TwoSampleEqualVar,
    TwoSampleUnequalVar,
}

enum TTestTails {
    OneTailed,
    TwoTailed,
}

/// One T.TEST element over the samples read once: `tails`, then `type`, then the samples' error,
/// then the test.
fn t_test_element(
    model: &Model,
    samples: &Result<PairedValues, PairedSamplesError>,
    values: &[CalcResult],
    cell: CellReferenceIndex,
) -> CalcResult {
    let [_, _, tails, test_type] = values else {
        return CalcResult::new_args_number_error(cell);
    };
    // A sample that is itself an error value, or a blank one-cell reference, comes before tails
    // and type (`T.TEST(<#VALUE!>,42,1,-1)` is `#VALUE!`, `TTEST(<#VALUE!>,<#REF!>,2,<#REF!>)`
    // too, probes/t2475-stat-subexpressions P!A58, P!A222; `T.TEST(E11,A6:C12,2,123456789.123)`
    // with E11 blank is `#VALUE!`, probes/tbill-error-order-inner-calls; the second sample's
    // `#VALUE!` before tails' `#N/A`, gen-m9 9e2225ead8a6a07b A59, and its `#DIV/0!` before
    // tails' `#VALUE!`, 3b7990bd4e3307ba A100), while an error inside a range comes after.
    if let Err(PairedSamplesError {
        error,
        argument_is_error_value: true,
        ..
    }) = samples
    {
        return error.clone();
    }
    // Both are read before either is checked: type's `#VALUE!` beats tails 7
    // (`T.TEST(..,{..},7,<#VALUE!>)`, gen-m9 607e85cfee83a689 A194).
    let tails = match model.kernel_number(tails, cell) {
        Ok(tails) => tails.trunc(),
        Err(error) => return error,
    };
    let test_type = match model.kernel_number(test_type, cell) {
        Ok(test_type) => test_type.trunc(),
        Err(error) => return error,
    };
    let tails = match tails {
        1.0 => TTestTails::OneTailed,
        2.0 => TTestTails::TwoTailed,
        _ => return num_error(cell),
    };
    let test_type = match test_type {
        1.0 => TTestType::Paired,
        2.0 => TTestType::TwoSampleEqualVar,
        3.0 => TTestType::TwoSampleUnequalVar,
        _ => return num_error(cell),
    };
    match samples {
        Ok(samples) => number_or_error(t_test_p(samples, &tails, &test_type, cell)),
        // A paired test of samples of different sizes is `#N/A` before an error cell is
        // (probes/ftest-decomposition G12); the unpaired tests return the error (G13).
        Err(PairedSamplesError {
            sizes_differ: true, ..
        }) if matches!(test_type, TTestType::Paired) => {
            CalcResult::new_error(Error::NA, cell, String::new())
        }
        Err(PairedSamplesError { error, .. }) => error.clone(),
    }
}

/// T.TEST's p-value for valid `tails` and `test_type`.
fn t_test_p(
    samples: &PairedValues,
    tails: &TTestTails,
    test_type: &TTestType,
    cell: CellReferenceIndex,
) -> Result<f64, CalcResult> {
    let error = |error| Err(CalcResult::new_error(error, cell, String::new()));
    let paired = matches!(test_type, TTestType::Paired);
    // For a paired test both samples must have the same size.
    if paired && !samples.same_size {
        return error(Error::NA);
    }
    let (values1, values2): (Vec<f64>, Vec<f64>) = if paired {
        // The pairs zipping the samples makes; a pair with a non-number side is skipped.
        samples
            .first
            .iter()
            .zip(&samples.second)
            .take(samples.paired)
            .filter_map(|pair| match pair {
                (Some(v1), Some(v2)) => Some((*v1, *v2)),
                _ => None,
            })
            .unzip()
    } else {
        // keep only numeric entries, ignore non-numeric (Option::None)
        (
            samples.first.iter().flatten().copied().collect(),
            samples.second.iter().flatten().copied().collect(),
        )
    };

    let n1 = values1.len();
    let n2 = values2.len();
    if n1 == 0 || n2 == 0 {
        return error(Error::DIV);
    }

    let (t_square, df) = match test_type {
        TTestType::Paired => {
            if n1 != n2 {
                return error(Error::NA);
            }
            // At least two pairs.
            if n1 < 2 {
                return error(Error::DIV);
            }
            let diffs: Vec<f64> = values1
                .iter()
                .zip(&values2)
                .map(|(v1, v2)| v1 - v2)
                .collect();
            let nd = diffs.len();
            let md = mean(&diffs);
            let vd = sample_var(&diffs);
            // A sample variance that overflows (data whose squares do) is #NUM!, not a p from an
            // infinite deviation, in every test type (`T.TEST(B6:D12,{1E+200,1E+200,-1E+200},1,2)`,
            // gen-m9 a612e89321a6df9c A375; statistics.md "Overflowing squares (T-2432)").
            if vd.is_infinite() {
                return error(Error::NUM);
            }
            // Zero variance of the differences.
            if vd <= 0.0 {
                return error(Error::DIV);
            }
            // Excel's squared statistic is md^2 / (vd / n): bit-exact on 7 of the 11 paired
            // cells (probes-numeric t1048/t1049-ttest, `T_TEST.xlsx` F2 and F5) and within
            // 4 ulp on the rest; md^2 n / vd misses F2 by 2 ulp.
            let t_square = md * md / (vd / (nd as f64));
            let df = (nd - 1) as f64;
            (t_square, df)
        }

        // 2: two-sample, equal variance (homoscedastic)
        TTestType::TwoSampleEqualVar => {
            // At least two values in each sample.
            if n1 < 2 || n2 < 2 {
                return error(Error::DIV);
            }

            let m1 = mean(&values1);
            let m2 = mean(&values2);
            let v1 = sample_var(&values1);
            let v2 = sample_var(&values2);
            if v1.is_infinite() || v2.is_infinite() {
                return error(Error::NUM);
            }

            let df_i = (n1 + n2 - 2) as i32;
            if df_i <= 0 {
                return error(Error::DIV);
            }
            let df = df_i as f64;

            let sp2 = (((n1 - 1) as f64) * v1 + ((n2 - 1) as f64) * v2) / df; // pooled variance

            if sp2 <= 0.0 {
                return error(Error::DIV);
            }

            let denom = (sp2 * (1.0 / (n1 as f64) + 1.0 / (n2 as f64))).sqrt();
            if denom == 0.0 {
                return error(Error::DIV);
            }

            let t_stat = (m1 - m2) / denom;
            (t_stat * t_stat, df)
        }

        // two-sample, unequal variance (Welch)
        TTestType::TwoSampleUnequalVar => {
            // At least two values in each sample.
            if n1 < 2 || n2 < 2 {
                return error(Error::DIV);
            }

            let m1 = mean(&values1);
            let m2 = mean(&values2);
            let v1 = sample_var(&values1);
            let v2 = sample_var(&values2);
            if v1.is_infinite() || v2.is_infinite() {
                return error(Error::NUM);
            }

            let s1n = v1 / (n1 as f64);
            let s2n = v2 / (n2 as f64);
            let denom = (s1n + s2n).sqrt();
            if denom == 0.0 {
                return error(Error::DIV);
            }

            let t_stat = (m1 - m2) / denom;

            let num_df = (s1n + s2n).powi(2);
            let den_df = (s1n * s1n) / ((n1 - 1) as f64) + (s2n * s2n) / ((n2 - 1) as f64);
            if den_df == 0.0 {
                return error(Error::DIV);
            }
            let df = num_df / den_df;
            (t_stat * t_stat, df)
        }
    };

    if df <= 0.0 {
        return error(Error::DIV);
    }

    // Excel rounds y = df/(df + t^2) to a double before the tail (`student_t_tail_at`).
    let tail = student_t_tail_at(df / (df + t_square), df);
    let mut p = match tails {
        TTestTails::OneTailed => tail,
        TTestTails::TwoTailed => 2.0 * tail,
    };

    // clamp tiny fp noise
    if p < 0.0 && p > -1e-15 {
        p = 0.0;
    }
    if p > 1.0 && p < 1.0 + 1e-15 {
        p = 1.0;
    }

    Ok(p)
}

/// Largest (truncated) `deg_freedom` for which Excel evaluates the cumulative and inverse Student-t
/// functions; above it they are `#NUM!` while the pdf stays finite. Without the guard `statrs`'
/// quantile search does not finish for a huge df (fuzz hang on `T.INV(0.9,5E10)`).
/// Excel: see docs/excel-semantics/statistics.md#student-t-sweep-df-truncation-df-limits-tinv2t-above-1-t-1043
pub(crate) const T_DF_MAX: f64 = 1e10;

/// Whether the half `p / 2` of a positive two-tailed probability falls below the smallest normal
/// double, which Excel flushes to 0 (p below `2.0 * f64::MIN_POSITIVE`): T.INV.2T and TINV then answer as at p = 0
/// (`#DIV/0!` at df 1, `#NUM!` from df 2) and CONFIDENCE.T is `#NUM!`, although T.INV at p itself is
/// a number. The cut is df independent: `T.INV.2T(4.4501477170144E-308,df)` is an error and
/// `T.INV.2T(4.45014771701441E-308,df)` a number at df 1, 2, 9 and 30 (probes/t4541-t-inv-2t-tiny-p;
/// statistics.md "T.DIST's density is #NUM! at a huge x, and T.INV.2T and TINV at p = 2.2E-308").
pub(crate) fn two_tailed_half_underflows(p: f64) -> bool {
    p > 0.0 && flush_subnormal(p / 2.0) == 0.0
}

/// T.INV (`two_tailed` false, p in (0, 1)) and T.INV.2T / TINV (p in (0, 2)) at one degree of
/// freedom: Excel's closed form (T-2858) through [`cauchy_two_tail_quantile`]. T.INV(0.5) is 0,
/// unprobed (T-3540).
fn df_one_quantile(p: f64, two_tailed: bool) -> f64 {
    if two_tailed {
        if p == 1.0 {
            // 1/TAN(PI/2) would be 6.1E-17; Excel returns 0 (gen-m9 0620505280d2ea6e A90
            // `TINV(1,1)`, 7f7ca726c3f68554 A166 `TINV(1-1E-17,1)`).
            0.0
        } else if p > 1.0 {
            -cauchy_two_tail_quantile(2.0 - p)
        } else {
            cauchy_two_tail_quantile(p)
        }
    } else if p < 0.5 {
        -cauchy_two_tail_quantile(2.0 * p)
    } else if p > 0.5 {
        cauchy_two_tail_quantile(2.0 * (1.0 - p))
    } else {
        0.0
    }
}

/// Whether the Student-t functions refuse `x` as `#NUM!`: fl(x x) overflows (|x| >= 2^512). Exact for
/// T.DIST's density (T-3554); UNVERIFIED (T-2474) for T.DIST (cumulative), T.DIST.RT, T.DIST.2T and TDIST,
/// `#NUM!` at 1E+300 but a number at `T.DIST.2T(1E+15,1)` (statistics.md "... extreme arguments, part 2").
fn t_x_overflows(x: f64) -> bool {
    (x * x).is_infinite()
}

/// Probabilities below this take the tail path of [`t_tail_quantile`] instead of `statrs`.
/// T-3395 moved the cut in full eval runs (a temporary override, reverted) and scored every
/// changed cell in ulps against Excel; after T-2511 the 25 cells it moves are df 1 and 2 (and
/// df 1.5, truncated), q 5E-11..5E-3. A lower cut makes them worse (1E-8: 3 cells, 1E-11: 15
/// worse, 4 better), 1E-4 changes none, 1E-3 brings one closer (`T.INV.2T(0.001,1)`) and 1E-2
/// makes 4 worse and 2 better. So the cells place the cut in 1E-5..1E-3, leaning to the top;
/// T-3477 owns any move.
/// FITTED(T-1531, 25 cells, q 5E-11..5E-3 at df 1..2; cells place the cut in 1E-5..1E-3)
const T_TAIL_P: f64 = 1e-5;

/// Below this `x = df / (df + t^2)` the incomplete beta `I_x(df/2, 1/2)` equals its leading term and
/// the first correction to better than 1E-16, so the tail quantile has a closed form.
const T_TAIL_X: f64 = 1e-8;

/// Below this probability the df-1 quantile comes from bisection on the cdf, not `statrs`'
/// `inverse_cdf`, which hangs at 1E-100 for df = 1000 and panics from about 1E-200. Only df 1
/// reaches it (every other df takes [`t_rounded_y_quantile`]). No cell moves with it (T-3395: the
/// same cut-moving runs as [`T_TAIL_P`], at 1E-20, 1E-11, 1E-9 and 1E-7, change no cell's value),
/// so after T-2511 it is in effect the guard against statrs' hang and panic, not a fit to cells.
/// FITTED(T-1531, 105 cells, p 1E-300..1E-10, df 1..1E5; no cell moves with the cut from 1E-20 to 1E-7)
const T_STATRS_MIN_P: f64 = 1e-8;

/// The tail quantile's closed form: `t` with `P(T < -t) = q` from the leading term and first
/// correction of `I_x(df/2, 1/2)`, where `x = df / (df + t^2)` is below [`T_TAIL_X`] (`None`
/// otherwise). In the deep tail Excel's quantile is within an ulp or so of this form where the
/// rounded-y back-transform is visibly off (`TINV(1E-200,2)` = 1.0000000000000101E+100, this form
/// 1.0000000000000103E+100, the back-transform 9.99999999999996E+99; gen-m9/7980ffb6095a340d A312, T-2511).
fn t_tail_closed_form(df: f64, q: f64) -> Option<f64> {
    let a = 0.5 * df;
    // P(T < -t) = I_x(a, 1/2) / 2 with I_x ~ x^a / (a B(a, 1/2)) * (1 + a / (2 (a + 1)) x).
    let ln_b = ln_gamma(a) + ln_gamma(0.5) - ln_gamma(a + 0.5);
    let mut ln_x = ((2.0 * q).ln() + a.ln() + ln_b) / a;
    // A NaN ln_x (a df its Gamma ratio refuses) has no closed form either.
    if ln_x.is_nan() || ln_x >= T_TAIL_X.ln() {
        return None;
    }
    let x = ln_x.exp();
    ln_x -= (a / (2.0 * (a + 1.0)) * x).ln_1p() / a;
    let x = ln_x.exp();
    Some(df.sqrt() * (-0.5 * ln_x).exp() * (1.0 - x).sqrt())
}

/// Magnitude `t > 0` with `P(T < -t) = q` for a small `q` at df 1 (every other df takes
/// [`t_rounded_y_quantile`]); bounded time.
/// Excel gives finite values down to the smallest p it accepts (probes-numeric `t1531-t-tiny-p`:
/// `T.INV(1E-300,1)` = -3.183098861837907E299). Where `x` is not tiny `statrs`' search is kept
/// (its df-1 closed form is T-2858's).
fn t_tail_quantile(dist: &StudentsT, df: f64, q: f64) -> f64 {
    if let Some(t) = t_tail_closed_form(df, q) {
        return t;
    }
    if q >= T_STATRS_MIN_P {
        return -t_inverse_cdf(df, q);
    }
    // x is not small and q is tiny: the quantile is moderate, so bisect on the cdf; -cdf(-t) is
    // increasing in t and -cdf(-t) >= -q exactly when P(T < -t) <= q.
    bisect_increasing(|t| -dist.cdf(-t), -q, 64)
}

/// Central probabilities `c = |2p - 1|` below this take [`t_centre_quantile`]. statrs' inversion
/// solves for `y = df / (df + t^2)` near 1 and back-transforms with `1 - y`, which keeps only the
/// digits of `y` below 1 (`TINV(1-1E-10,5)` 2.36E-8, Excel 1.32E-10; T-2724). Excel switches from the
/// rounded-y form to the exact quantile between two-tailed p = 0.9 (c = 0.1) and 0.95 (c = 0.05)
/// (statistics.md "Student-t quantile is SQRT(df*(1/y-1)) of the rounded beta quantile, away from
/// the centre"), so the cut sits inside that bracket. T-3395 moved it in full eval runs (a
/// temporary override, after T-2511): a cut of 0.04 breaks gen-m9's six cells at c = 0.05
/// (`TINV(0.95,3)`, `TINV(0.95,5)`, `T.INV.2T(0.95,12)`, `CONFIDENCE.T(0.95,..,20)`) and takes 5
/// more at c 0.05..0.057 further off; a cut of 0.0999 brings two failing t3349 sweep cells closer,
/// `T.INV.2T(0.921314149721905,4)` (c 0.0787) and `T.INV.2T(0.90147488203954,8)` (c 0.0985). So
/// on net the cells favour a cut in 0.0985..0.1; T-3477 owns any move.
/// FITTED(T-2724, 13 cells, c 0.05..0.1, df 3..1E10; cut bracketed 0.05..0.1, cells favour 0.0985..0.1)
const T_CENTRE_C: f64 = 0.075;

/// The centre cut of T.INV.2T's negative branch, p in (1, 2), with c = p - 1: Excel is the exact
/// quantile there at c = 0.0986 and 0.0991, past [`T_CENTRE_C`] (`T.INV.2T(LN(3),7)` =
/// -0.12847323195087978, gen-m9 c2684066d8df263e A64; d910f000b60e2eac A357 at df 8).
/// FITTED(T-2724, 2 cells, c 0.0986..0.0991, df 7..8)
const T_NEGATIVE_CENTRE_C: f64 = 0.1;

/// Magnitude `t >= 0` with `P(|T| < t) = c` for a central probability `c` below [`T_CENTRE_C`]
/// (so `t` < 0.12 at every df): the crossing of the centre probability itself
/// (`student_t_centre`), which never forms `1 - y`, so a tiny quantile keeps its digits. Excel's
/// centre quantiles are accurate (statistics.md "Student t sweep", T-2724). The estimate is
/// `c / (2 f(0))`, `f` the density; at most 129 evaluations (`refine_increasing`).
/// `df` is a truncated integer from 1 to [`T_DF_MAX`] here, every one of which `student_t_centre`
/// takes (it is NaN only for a df its Gamma ratio refuses).
fn t_centre_quantile(df: f64, c: f64) -> f64 {
    let estimate = student_t_density(0.0, df).map_or(f64::NAN, |density| 0.5 * c / density);
    refine_increasing(|t| student_t_centre(t, df), c, estimate, 0.0, 1.0)
}

/// Below this df statrs' search is bounded and accurate enough to seed [`t_rounded_y_quantile`]'s
/// bracket (seconds per call at 1E7, no finish at 1E8).
const T_STATRS_MAX_DF: f64 = 1e5;

/// From this df the quantile away from the centre is [`t_rounded_y_quantile`]; df 1 keeps statrs'
/// search and the tail forms below (its closed form is T-2858's).
const T_ROUNDED_Y_MIN_DF: f64 = 2.0;

/// Magnitude `t > 0` with `P(T > t) = q` for a one-sided `q` in (0, 1/2) outside the centre:
/// Excel's back-transform t = fl(sqrt(fl(df * fl(fl(1/y) - 1)))) of y, the double nearest the
/// crossing of the tail at a rounded y, L(y) = q (`student_t_tail_at`, T.TEST's tail; statistics.md
/// "Student-t quantile is SQRT(df*(1/y-1)) of the rounded beta quantile, away from the centre",
/// T-3061; the crossing is T-2511's). The ported kernel's tail at y (`bratio::t_kernel_tails`,
/// as T.DIST takes it) fixes 4 more cells but makes 3 newly visibly off (`T.INV(0.2,5)`,
/// gen-m9/0bbab6206b5c9a07 A67, 6 ulp). L rises with y, so the search is `nearest_crossing` over
/// [0, 1], seeded by AS 109's root where statrs is reliable: about 25 to 30 tails with a good seed,
/// at most 66 unseeded and about 81 with a poor seed, for every df up to [`T_DF_MAX`] (the cost is in lift/kinds/statistical.rs
/// `T_QUANTILE_WEIGHT`). A deep tail keeps
/// [`t_tail_closed_form`].
fn t_rounded_y_quantile(df: f64, q: f64) -> f64 {
    if q < T_TAIL_P {
        if let Some(t) = t_tail_closed_form(df, q) {
            return t;
        }
    }
    // AS 109's root (statrs' search, a few microseconds where it is reliable) seeds the bracket:
    // about 25 tails instead of 64, which matters where one tail sums the integer-df series
    // (about 30 us at df 1000; T-2511's performance review).
    let estimate = if df < T_STATRS_MAX_DF && q >= T_STATRS_MIN_P {
        t_quantile_beta_root(df, q)
    } else {
        f64::NAN
    };
    let y = nearest_crossing_from(|y| student_t_tail_at(y, df), q, estimate, 0.0, 1.0);
    // A y of 0 gives an infinite t, which the callers turn into #NUM! (`is_finite` checks in
    // `t_inv_of` and CONFIDENCE.T).
    (df * (1.0 / y - 1.0)).sqrt()
}

/// The signed quantile with `P(T > t) = q` from [`t_rounded_y_quantile`], for q outside the centre.
fn t_rounded_y_upper(df: f64, q: f64) -> f64 {
    // 1 - q is exact for q in [1/2, 1] (Sterbenz).
    if q < 0.5 {
        t_rounded_y_quantile(df, q)
    } else {
        -t_rounded_y_quantile(df, 1.0 - q)
    }
}

/// Quantile `t` with `P(T > t) = q`. For a tiny `q`, `1 - q` rounds to 1, so the tail is asked for
/// directly (T-1531); near 1/2 the centre is (T-2724); otherwise this is `t_quantile(1 - q)`.
pub(crate) fn t_upper_quantile(dist: &StudentsT, df: f64, q: f64) -> f64 {
    // 1 - 2q is exact for q in [1/4, 1] (Sterbenz) where 1 - q is not for q below 1/2.
    let c = 1.0 - 2.0 * q;
    if c.abs() < T_CENTRE_C {
        return t_centre_quantile(df, c.abs()).copysign(c);
    }
    if df >= T_ROUNDED_Y_MIN_DF {
        return t_rounded_y_upper(df, q);
    }
    if q < T_TAIL_P {
        -t_quantile(dist, df, q)
    } else {
        t_quantile(dist, df, 1.0 - q)
    }
}

/// Quantile of Student's t with `df` degrees of freedom for `p` in (0,1); the time is bounded for
/// every df up to [`T_DF_MAX`]. The centre is [`t_centre_quantile`], every df from 2 elsewhere
/// [`t_rounded_y_quantile`]; df 1 keeps statrs' search (the capped port of `inverse_beta.rs`,
/// T-1848) and the tail forms of [`t_tail_quantile`].
pub(crate) fn t_quantile(dist: &StudentsT, df: f64, p: f64) -> f64 {
    // 2p - 1 is exact for p in [1/4, 1] (Sterbenz).
    let c = 2.0 * p - 1.0;
    if c.abs() < T_CENTRE_C {
        return t_centre_quantile(df, c.abs()).copysign(c);
    }
    if df >= T_ROUNDED_Y_MIN_DF {
        // P(T < t) = p is P(T > t) = 1 - p, i.e. -t has the upper tail p.
        return -t_rounded_y_upper(df, p);
    }
    // df 1: below the tail threshold `statrs` is fine; above it its inverse saturates
    // (T.INV(1E-10,1) gives -8.9E7, Excel -3.18E9) and panics from p about 1E-200 (T-1531).
    let (q, sign) = if p < 0.5 { (p, -1.0) } else { (1.0 - p, 1.0) };
    if q < T_TAIL_P {
        return sign * t_tail_quantile(dist, df, q);
    }
    t_inverse_cdf(df, p)
}

impl Model {
    /// `T.DIST(x, deg_freedom, cumulative)`: every argument lifts over an array and a range
    /// (SPEC-0011; probes/stat-argument-modes H505-H509, probes/stat-later-argument-lift H61).
    /// The cdf and the integer-df density come from `student_t` (T-0969); other densities from statrs.
    pub(crate) fn fn_t_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::TDist, args, cell, &mut |model, values| {
            number_or_error(model.t_dist_of(values, cell))
        })
    }

    fn t_dist_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [x, df, cumulative] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let x = self.cast_number_ref(x, cell)?;
        let df = self.cast_number_ref(df, cell)?.trunc();
        let cumulative = Self::cast_bool_ref(cumulative, cell)?;
        // The density divides by the truncated df: df in (-1, 1) is `#DIV/0!` (`T.DIST(2,0.5,FALSE)`;
        // probes-numeric m9-t-edges A27-A41, "Student t sweep"), as is df 1's where its denominator
        // overflows (T-3554, statistics.md "T.DIST's density is #NUM! at a huge x").
        if !cumulative && (df == 0.0 || (df == 1.0 && excel_t_density_df_one_overflows(x))) {
            return Err(div_error(cell));
        }
        if df < 1.0 || t_x_overflows(x) || (cumulative && df > T_DF_MAX) {
            return Err(num_error(cell));
        }
        let result = if cumulative {
            student_t_tails(x, df).0
        } else if let Some(density) = t_density(x, df) {
            density
        } else {
            StudentsT::new(0.0, 1.0, df)
                .map_err(|_| num_error(cell))?
                .pdf(x)
        };
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `T.DIST.2T(x, deg_freedom)`: both arguments lift over an array and a range
    /// (probes/stat-argument-modes H510-H513, probes/stat-later-argument-lift H63).
    pub(crate) fn fn_t_dist_2t(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::TDist2T, args, cell, &mut |model, values| {
            let [x, df] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(
                model
                    .cast_number_ref(x, cell)
                    .and_then(|x| model.t_dist_2t_of(x, df, cell)),
            )
        })
    }

    /// T.DIST.2T once `x` is read. The degrees of freedom are read before `x` is checked
    /// (`T.DIST.2T(-1,NA())` is #N/A; probes/stat-overflow-arguments F1, F5).
    fn t_dist_2t_of(
        &self,
        x: f64,
        df: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let df = self.cast_number_ref(df, cell)?.trunc();
        if x < 0.0 || !(1.0..=T_DF_MAX).contains(&df) || t_x_overflows(x) {
            return Err(num_error(cell));
        }
        let result = 2.0 * student_t_tails(x, df).1;
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `T.DIST.RT(x, deg_freedom)`: both arguments lift over an array and a range
    /// (probes/stat-argument-modes H514-H517, probes/stat-later-argument-lift H65).
    pub(crate) fn fn_t_dist_rt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::TDistRT, args, cell, &mut |model, values| {
            let [x, df] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(
                model
                    .cast_number_ref(x, cell)
                    .and_then(|x| model.t_dist_rt_of(x, df, cell)),
            )
        })
    }

    /// T.DIST.RT once `x` is read.
    fn t_dist_rt_of(
        &self,
        x: f64,
        df: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let df = self.cast_number_ref(df, cell)?.trunc();
        if !(1.0..=T_DF_MAX).contains(&df) || t_x_overflows(x) {
            return Err(num_error(cell));
        }
        let result = student_t_tails(x, df).1;
        if !result.is_finite() || result < 0.0 {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `T.INV(probability, deg_freedom)`: both arguments lift over an array and a range
    /// (SPEC-0011; probes/stat-argument-modes, probes/stat-quantile-lift). The quantile is a
    /// crossing of the tail at y from df 2 (`t_rounded_y_quantile`) or a closed form, and at df 1 the capped port of
    /// statrs' inversion or a bisection of at most 264 cdfs (`t_quantile`), which the lifting
    /// table charges (lift/kinds/statistical.rs `T_QUANTILE_WEIGHT`).
    pub(crate) fn fn_t_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::TInv, args, cell, &mut |model, values| {
            number_or_error(model.t_inv_of(values, false, cell))
        })
    }

    /// `T.INV.2T(probability, deg_freedom)`, and TINV: as T.INV, two-tailed.
    pub(crate) fn fn_t_inv_2t(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::TInv2T, args, cell, &mut |model, values| {
            number_or_error(model.t_inv_of(values, true, cell))
        })
    }

    /// The T.INV kernel; `two_tailed` for T.INV.2T and TINV, which take p in (0, 2) and return
    /// the magnitude with `P(|T| > t) = p` for p up to 1, and the negative quantile of 2 - p
    /// above it.
    fn t_inv_of(
        &self,
        values: &[CalcResult],
        two_tailed: bool,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [p, df] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let mut p = self.cast_number_ref(p, cell)?;
        let df = self.cast_number_ref(df, cell)?.trunc();
        if two_tailed && two_tailed_half_underflows(p) {
            p = 0.0;
        }
        // p = 0 with a df that truncates to 1 is `#DIV/0!`, from df 2 up `#NUM!` (`TINV(0,1)`,
        // `T.INV(0,1.999)`, `T.INV.2T(FALSE,1.005)` against `T.INV(0,2)`; statistics.md "Student t
        // quantile at p = 0 with df below 2 is #DIV/0!", probes/t2475-stat-subexpressions P!A124).
        if p == 0.0 && df == 1.0 {
            return Err(div_error(cell));
        }
        let p_out = if two_tailed {
            p <= 0.0 || p >= 2.0
        } else {
            p <= 0.0 || p >= 1.0
        };
        if p_out || !(1.0..=T_DF_MAX).contains(&df) {
            return Err(num_error(cell));
        }
        if df == 1.0 {
            let x = df_one_quantile(p, two_tailed);
            return if x.is_finite() {
                Ok(x)
            } else {
                Err(num_error(cell))
            };
        }
        let dist = StudentsT::new(0.0, 1.0, df).map_err(|_| num_error(cell))?;
        let x = if two_tailed && p > 1.0 {
            // T.INV.2T(p) = T.INV(1 - p/2), the negative quantile, for p in (1, 2) (m9-t-edges A2
            // `T.INV.2T(1.5,5)`, A43 `T.INV.2T(1.5,10)`). 2 - p is exact on [1, 2] (Sterbenz), and
            // the upper tail of (2 - p) / 2 reaches the tiny-p path for p near 2. p - 1 is exact too.
            if p - 1.0 < T_NEGATIVE_CENTRE_C {
                -t_centre_quantile(df, p - 1.0)
            } else {
                -t_upper_quantile(&dist, df, (2.0 - p) / 2.0).abs()
            }
        } else if two_tailed {
            // Two-sided: F(x) = 1 - p/2
            t_upper_quantile(&dist, df, p / 2.0).abs()
        } else {
            t_quantile(&dist, df, p)
        };
        if !x.is_finite() {
            return Err(num_error(cell));
        }
        Ok(x)
    }

    /// `T.TEST(array1, array2, tails, type)` and TTEST: `tails` and `type` lift over an array
    /// and a multi-cell range, the samples are read once (SPEC-0011, T-1982). An element
    /// truncates and checks `tails`, then `type`, then takes the samples' error:
    /// `T.TEST(D1:D3,A1:A3,3,2)` with D2 = `#N/A` is `#NUM!`, while a lifted element with good
    /// arguments is the samples' `#N/A`; `tails` 3 or `type` 4 is `#NUM!` per element, `1.7` and
    /// `2.9` are 1 and 2, TRUE is 1 (probes/stat-test-lift P60-P67, P88). Each element walks the
    /// samples, so the call is charged their cells per element.
    pub(crate) fn fn_t_test(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // TTEST shares T.TEST's table entry (`TWO_DATA_THEN_2`).
        let kind = Function::TTest;
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        // A text sample is an array of one cell with no number, not `#VALUE!`
        // (`TTEST(IFERROR(#VALUE!,""),Data!B11:E11,1,3)` is `#DIV/0!`, gen-m9
        // 46ac3ef5e61583ca A344; statistics.md "T.TEST: an error sample first ...").
        let samples = self
            .paired_samples(args, cell, MarkedSample::NumError)
            .map_err(|error| PairedSamplesError {
                error: error.error.without_message(),
                ..error
            });
        let cells = samples.as_ref().map_or(0, |samples| {
            let cells = samples.first.len().saturating_add(samples.second.len());
            u64::try_from(cells).unwrap_or(u64::MAX)
        });
        self.lift_call_costed(
            kind,
            args,
            // At least one unit an element, so a samples error copied per element is charged too.
            LiftCost::Cells(cells.max(1)),
            cell,
            &mut |model, values| t_test_element(model, &samples, values, cell),
        )
    }

    /// `TDIST(x, deg_freedom, tails)`: x >= 0; tails 1 is T.DIST.RT, tails 2 T.DIST.2T. Every
    /// argument lifts over an array and a range (probes/stat-argument-modes H530-H534,
    /// probes/stat-later-argument-lift H67, L8-L10). It reads all three before it checks any.
    pub(crate) fn fn_tdist_compat(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::TDistCompat, args, cell, &mut |model, values| {
            number_or_error(model.tdist_of(values, cell))
        })
    }

    fn tdist_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> Result<f64, CalcResult> {
        let [x, df, tails] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        // Every argument is read before any is checked: an error in deg_freedom wins over x < 0
        // and over a bad tails (`TDIST(-1,NA(),1)` and `TDIST(1,NA(),3)` are #N/A;
        // probes/stat-overflow-arguments F2, F3, F6).
        let x = self.cast_number_ref(x, cell)?;
        self.cast_number_ref(df, cell)?;
        let tails = self.cast_number_ref(tails, cell)?.trunc() as i64;
        if x < 0.0 {
            return Err(num_error(cell));
        }
        match tails {
            1 => self.t_dist_rt_of(x, df, cell),
            2 => self.t_dist_2t_of(x, df, cell),
            _ => Err(num_error(cell)),
        }
    }
}

#[cfg(test)]
mod tests;
