//! LOGNORM.DIST, LOGNORM.INV and LOGNORMDIST (LOGINV is LOGNORM.INV): every argument lifts over
//! an array and a range (SPEC-0011; probes/stat-argument-modes H353-H373,
//! probes/stat-later-argument-lift H49-H55). The kernels are closed form.

use crate::arithmetic::{extended_product, extended_quotient, extended_sum};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::statistical::normal::normal_quantile;
use crate::functions::statistical::special::{normal_cdf, LN_SQRT_2PI_CORRECTLY_ROUNDED};
use crate::functions::Function;
use crate::x87_exp::{excel_exp, two_sum};
use crate::x87_log10::excel_ln;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    /// `LOGNORM.DIST(x, mean, standard_dev, cumulative)`.
    pub(crate) fn fn_log_norm_dist(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::LogNormDist, args, cell, &mut |model, values| {
            let [x, mean, std_dev, cumulative] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.log_norm_dist_of(x, mean, std_dev, Some(cumulative), cell))
        })
    }

    /// The LOGNORM.DIST kernel; `cumulative` is `None` for LOGNORMDIST, always cumulative.
    fn log_norm_dist_of(
        &self,
        x: &CalcResult,
        mean: &CalcResult,
        std_dev: &CalcResult,
        cumulative: Option<&CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let x = self.cast_number_ref(x, cell)?;
        let mean = self.cast_number_ref(mean, cell)?;
        let std_dev = self.cast_number_ref(std_dev, cell)?;
        let cumulative = match cumulative {
            Some(value) => Self::cast_bool_ref(value, cell)?,
            None => true,
        };
        // Excel domain checks
        if x <= 0.0 || std_dev <= 0.0 {
            return Err(num_error(cell));
        }
        let result = if cumulative {
            normal_cdf(x.ln(), mean, std_dev)
        } else {
            // Not `normal_pdf`'s kernel (T-4321).
            lognormal_density(x, mean, std_dev)
        };
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `LOGNORM.INV(probability, mean, standard_dev)`, and LOGINV.
    pub(crate) fn fn_log_norm_inv(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::LogNormInv, args, cell, &mut |model, values| {
            number_or_error(model.log_norm_inv_of(values, cell))
        })
    }

    fn log_norm_inv_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [p, mean, std_dev] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let p = self.cast_number_ref(p, cell)?;
        let mean = self.cast_number_ref(mean, cell)?;
        let std_dev = self.cast_number_ref(std_dev, cell)?;
        // Excel domain checks
        if p <= 0.0 || p >= 1.0 || std_dev <= 0.0 {
            return Err(num_error(cell));
        }
        // EXP of NORM.INV's quantile, so the PPND16 tails reach LOGNORM.INV too (T-0925).
        let result = normal_quantile(p, mean, std_dev, cell)?.exp();
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `LOGNORMDIST(x, mean, standard_dev)`: LOGNORM.DIST with `cumulative` TRUE.
    pub(crate) fn fn_lognormdist_compat(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(
            Function::LogNormDistCompat,
            args,
            cell,
            &mut |model, values| {
                let [x, mean, std_dev] = values else {
                    return CalcResult::new_args_number_error(cell);
                };
                number_or_error(model.log_norm_dist_of(x, mean, std_dev, None, cell))
            },
        )
    }
}

/// Excel's LOGNORM.DIST(x, mean, sigma, FALSE) for x > 0 and sigma > 0 (statistics.md
/// "LOGNORM.DIST's density is a `TwoSum` chain of its negative log (T-4321)"). The negative log
/// density `(d/sigma)²/2 + LN(sigma) + LN(√2π) + LN(x)`, d = LN(x) - mean, is accumulated in
/// that order as a compensated (`TwoSum`) chain whose residuals Excel adds back with the wrong
/// sign: the result is `EXP(-h)·(1 + (e1 + e2 + e3))`, not `·(1 - e)`. The square of the
/// displacement and of sigma come before the quotient: `(d·d)/(sigma·sigma)·0.5`, each an x87
/// operation stored to a double, falling back to `(d/sigma)²·0.5` where either square is not a
/// normal double. With Excel's LN and EXP, `LOGNORM.DIST(1,0,1,FALSE)` is 0.39894228040143265,
/// EXP(-LN(√2π)) and one ulp under 1/√2π (probes-numeric/m9-normpdf-lognorm-dist A4), and
/// `LOGNORM.DIST(40,0,0.5,FALSE)` 3.0218137820578166E-14 (gen-m9/b4173fe25109a238 A48). A
/// `half` past the double range is a zero density.
fn lognormal_density(x: f64, mean: f64, sigma: f64) -> f64 {
    let ln_x = excel_ln(x);
    let d = extended_sum(ln_x, -mean);
    let (d2, s2) = (extended_product(d, d), extended_product(sigma, sigma));
    let half = if d2.is_normal() && s2.is_normal() {
        extended_product(extended_quotient(d2, s2), 0.5)
    } else {
        let z = extended_quotient(d, sigma);
        extended_product(extended_product(z, z), 0.5)
    };
    if half.is_infinite() {
        return 0.0;
    }
    let (h, e1) = two_sum(half, excel_ln(sigma));
    let (h, e2) = two_sum(h, LN_SQRT_2PI_CORRECTLY_ROUNDED);
    let (h, e3) = two_sum(h, ln_x);
    excel_exp(-h) * (1.0 + (e1 + e2 + e3))
}

#[cfg(test)]
mod tests {
    use super::lognormal_density;

    /// The data's 3.14159, as a quotient (the same double) for clippy's PI lint.
    const PI_ISH: f64 = 314_159.0 / 100_000.0;

    // Excel's values (excel-graph): the density cells of T-4321.
    #[test]
    fn lognormal_density_matches_excel() {
        let cases = [
            // probes-numeric/m9-normpdf-lognorm-dist A4.
            ((1.0, 0.0, 1.0), 0.398_942_280_401_432_65),
            // gen-m9/b4173fe25109a238 A48.
            ((40.0, 0.0, 0.5), 3.021_813_782_057_816_6e-14),
            // gen-m9/3a25a03eb94fcfbd A220: LOGNORM.DIST(TRUE+<blank>,FLOOR.PRECISE(0.2^0),0.1,FALSE).
            ((1.0, 1.0, 0.1), 7.694_598_626_706_419e-22),
            // gen-m9/85f48d5cdfb24555 A310: LOGNORM.DIST(37.5,"1.5",3.14159,0).
            ((37.5, 1.5, PI_ISH), 0.002_694_261_202_816_920_7),
            // gen-m9/caa1eed48b1883fe A250.
            ((10.0, 0.2, PI_ISH), 0.010_150_659_136_505_528),
        ];
        for ((x, mean, sigma), excel) in cases {
            assert_eq!(
                lognormal_density(x, mean, sigma),
                excel,
                "LOGNORM.DIST({x},{mean},{sigma},FALSE)"
            );
        }
    }
}
