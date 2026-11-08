//! The small-x prefactor of the incomplete gamma's series and density (T-3219), its own module so
//! `mod.rs` stays under the file-size gate.

use super::{excel_exp, excel_gammaln, excel_ln};

/// Excel's prefactor at a shape above 1 and x / beta at most 1 (T-3219; docs/excel-semantics/
/// statistics.md "Incomplete gamma at a small x: three rounded factors"): `power` = EXP(a·LN(y))
/// and `gamma` = EXP(GAMMALN(a+1)), each a double, with EXP(-y) a third. Excel is the less
/// accurate one here, 5 to 36 ulp from the true value: `GAMMA.DIST(2,8,10,TRUE)` is Excel's
/// 5.316161131057619E-11 (ironcalc-calc `BETA_GAMMA.xlsx` `BETA_DIST!L3`), `GAMMA.DIST(1E-5,9,2,TRUE)`
/// 5.38226469068969E-54 (GAMMA.xlsx E16), `CHISQ.DIST(1E-10,8,FALSE)` 1.0416666666145884E-32
/// (gen-m9/984cba67bbe6972b A49).
#[derive(Clone, Copy)]
pub(super) struct LogSeriesFactors {
    power: f64,
    gamma: f64,
}

impl LogSeriesFactors {
    /// y^a / Gamma(a + 1), the density's leading factor: the power normalized first.
    pub(super) fn leading(self) -> f64 {
        self.power / self.gamma
    }

    /// P(a, y) from the series: the decay before the normalization and the series last. The order
    /// shows at 1 ulp: `CHISQ.DIST(1.5,6,TRUE)` (gen-m9/35e8168aae2b00f8 A119) needs the decay
    /// before the normalization, and probes/stat-argument-modes H152 `CHISQ.DIST(1.5,5,TRUE)` and
    /// probes-numeric/m9-gamma-dist-a2 A21 rule out the series before it.
    pub(super) fn lower(self, decay: f64, series: f64) -> f64 {
        self.power * decay / self.gamma * series
    }
}

/// [`LogSeriesFactors`] at a shape above 1 and a y in (0, 1], or `None` where a factor is not a
/// normal double (y^a underflows from a shape of a few hundred, Gamma(a + 1) overflows past 170):
/// the stable prefactor keeps its digits there.
pub(super) fn small_x_factors(a: f64, y: f64) -> Option<LogSeriesFactors> {
    if !(a > 1.0 && y > 0.0 && y <= 1.0) {
        return None;
    }
    let power = excel_exp(a * excel_ln(y));
    let gamma = excel_exp(excel_gammaln(a + 1.0));
    (power.is_normal() && gamma.is_normal()).then_some(LogSeriesFactors { power, gamma })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::super::tests::{ulp_distance, within_15_digits};
    use super::super::{
        gamma_cdf, gamma_density, gamma_quantile, pair_with, Exponent, Tail, Terms, MAX_TERMS,
    };
    use statrs::distribution::Gamma;

    /// T-3219: from a shape above 1 at x / beta up to 1, Excel's cdf and density are built from three
    /// separately rounded factors, EXP(a·LN(y)), EXP(GAMMALN(a+1)) and EXP(-y). Excel's own values,
    /// each 6 to 118 ulp from the prefactor's before: ironcalc-calc `BETA_GAMMA.xlsx` `BETA_DIST!L3`
    /// `GAMMA.DIST(2,8,10,TRUE)` (17 ulp before), GAMMA.xlsx E16 `GAMMA.DIST(1E-5,9,2,TRUE)` (6),
    /// gen-m9/68d8ffb69ff739d8 A341 P(30, 0.3) (118) and gen-m9/984cba67bbe6972b A49
    /// `CHISQ.DIST(1E-10,8,FALSE)` (37). The engine's EXP, LN and GAMMALN leave up to 5 ulp of residue
    /// (the exponent a·LN(y) is near -95 at A49 and -36 at A341). The order of the operations shows
    /// at 1 ulp, so cells that already passed are kept bit for bit: gen-m9/35e8168aae2b00f8 A119
    /// `CHISQ.DIST(1.5,6,TRUE)`, probes/stat-argument-modes H152 `CHISQ.DIST(1.5,5,TRUE)` and
    /// probes-numeric/m9-gamma-dist-a2 A21 `GAMMA.DIST(200000,2,1000000,TRUE)`.
    #[test]
    fn the_small_x_prefactor_is_excels_three_rounded_factors() {
        for (a, y, excel, ulps) in [
            (8.0, 0.2, 5.316_161_131_057_619e-11_f64, 1),
            (9.0, 5e-6, 5.382_264_690_689_69e-54, 0),
            (30.0, 0.3, 5.806_457_425_277_688e-49, 5),
            (3.0, 0.75, 0.040_505_439_744_813_87, 0),
            (2.5, 0.75, 0.086_930_185_455_604_56, 0),
            (2.0, 0.2, 0.017_523_096_306_421_772, 0),
        ] {
            let ours = gamma_cdf(&Gamma::new(a, 1.0).unwrap(), y);
            assert!(ulp_distance(ours, excel) <= ulps, "P({a},{y}): {ours:e}");
            assert!(within_15_digits(ours, excel), "P({a},{y}): {ours:e}");
        }
        let ours = gamma_density(4.0, 1e-10, 2.0).unwrap();
        let excel = 1.041_666_666_614_588_4e-32;
        assert!(ulp_distance(ours, excel) <= 2, "{ours:e}");
        assert!(within_15_digits(ours, excel), "{ours:e}");
        // The quantiles keep the accurate exponent, which the factors leave alone.
        assert_ne!(
            pair_with(8.0, 0.2, &mut Terms(MAX_TERMS), Exponent::Accurate).0,
            gamma_cdf(&Gamma::new(8.0, 1.0).unwrap(), 0.2)
        );
    }

    /// T-3146: below a unit quantile of 1/2 at shapes 1 to 20 Excel's chi-squared quantile is Halley
    /// steps from the leading-factor root on its own cdf, the upper tail as (0.5 - P) + 0.5:
    /// gen-m9/285d87cea1aaa73c A261 and 4ab2ba98d26c8a5e A85 `CHIINV(0.975,5)`, 10 ulp above the true
    /// quantile; gen-m9/4250f880ab640c49 A371 `CHISQ.INV.RT(((-0.0001)+1),3)`, 3,084 ulp from it; and
    /// gen-m9/c2974c061384fdd9 `CHISQ.INV(1E-20,10)`.
    #[test]
    fn small_chi_squared_quantiles_are_excels_halley_landing() {
        for (shape, p, tail, excel) in [
            (2.5, 0.975, Tail::Upper, 0.831_211_613_486_663_8_f64),
            (1.5, -0.0001 + 1.0, Tail::Upper, 0.005_214_832_329_030_269),
            (5.0, 1e-20, Tail::Lower, 0.000_521_056_841_397_433_4),
        ] {
            let ours = gamma_quantile(shape, 2.0, p, tail);
            assert_eq!(ours.to_bits(), excel.to_bits(), "({shape}, {p}): {ours}");
        }
    }
}
