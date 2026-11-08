//! Which expectations that read a random draw keep their value whatever the draws are (T-5100,
//! decision 0357). Excel's draws cannot be reproduced, so such a cell is compared with Excel only
//! when its value does not depend on them: the case is evaluated again under alternate random
//! sources, and a cell whose value moves is skipped as volatile, as every random cell was before.

use std::borrow::Cow;

use rustsheet::{RandomSource, LARGEST_DRAW};

use super::{Expectation, Prepared, PreparedCase};
use crate::engine::{Engine, Evaluated};

/// The sources a case is evaluated under besides the harness's own: two other seeds, and the
/// two ends of the draw range, which move any value that compares a draw with a threshold.
/// Fixed, so the cells a run scores are the same on every run.
fn alternate_sources() -> Vec<RandomSource> {
    let ends = [0.0, LARGEST_DRAW].map(RandomSource::fixed);
    [RandomSource::seeded(1), RandomSource::seeded(0xC0FF_EE00)]
        .into_iter()
        .chain(ends.into_iter().flatten())
        .collect()
}

/// The expectations to score: every one that reads no draw, and each one that does whose value in
/// `evaluated` (the harness source's run) is the value of every alternate run. Returns them with
/// the number dropped as draw-dependent. When the engine cannot evaluate the case under another
/// source, every expectation that reads a draw is dropped.
pub(super) fn draw_independent<'a>(
    engine: &(dyn Engine + Send),
    prepared: &'a PreparedCase,
    evaluated: &dyn Evaluated,
) -> (Cow<'a, [Expectation]>, usize) {
    let expectations = &prepared.expectations;
    let mut keep: Vec<bool> = expectations.iter().map(|e| !e.reads_draws).collect();
    if keep.iter().all(|&k| k) {
        return (Cow::Borrowed(expectations), 0);
    }
    let mut steady: Vec<bool> = keep.iter().map(|&k| !k).collect();
    let name = prepared.workbook_name.as_deref();
    for random in alternate_sources() {
        let other = match &prepared.input {
            Prepared::Xlsx { bytes } => engine.evaluate_xlsx_drawing(bytes, name, random),
            Prepared::Spec { spec } => engine.evaluate_spec_drawing(spec, name, random),
        };
        let Ok(other) = other else {
            steady.fill(false);
            break;
        };
        for (e, steady) in expectations.iter().zip(steady.iter_mut()) {
            if *steady {
                *steady = other.value(&e.sheet, e.addr) == evaluated.value(&e.sheet, e.addr);
            }
        }
    }
    for (keep, steady) in keep.iter_mut().zip(&steady) {
        *keep |= *steady;
    }
    let dropped = keep.iter().filter(|&&k| !k).count();
    let kept = expectations
        .iter()
        .zip(&keep)
        .filter(|(_, &k)| k)
        .map(|(e, _)| e.clone())
        .collect();
    (Cow::Owned(kept), dropped)
}
