#![allow(clippy::unwrap_used, reason = "tests")]

//! Iteration settings from untrusted input are brought into Excel's range (T-4048): a count
//! outside 1 to 32,767 or a delta that is not finite and non-negative falls back to Excel's
//! default (recalc.md, "Iteration settings outside Excel's range fall back to its defaults").

use crate::test::util::new_empty_model;
use crate::types::Iteration;
use crate::Model;

/// The `bitcode_workbook` nightly fuzz timeout of 2026-11-02 (run 41384443922), cut to the bytes
/// that decode outside the fuzzer: under `cfg(fuzzing)` bitcode does not refuse trailing bytes,
/// so the saved input decodes only there. A cycle on Sheet1 with a decoded count of 33,558,315
/// and a delta of 0: one recalculation took 34 s in a release build before T-4048.
#[rustfmt::skip]
const NIGHT_ONE: [u8; 377] = [
    0, 0, 1, 2, 65, 49, 0, 0, 6, 83, 104, 101, 101, 116, 49, 1, 1, 255, 4, 0,
    4, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 15, 4, 10, 15, 82, 85, 0, 0, 0, 0,
    0, 0, 0, 240, 63, 4, 0, 4, 10, 0, 4, 10, 0, 86, 0, 0, 0, 0, 0, 0,
    0, 0, 64, 0, 0, 0, 0, 0, 0, 16, 64, 0, 0, 0, 0, 0, 0, 32, 64, 6,
    4, 6, 4, 1, 11, 82, 91, 45, 48, 93, 67, 91, 48, 93, 42, 50, 4, 1, 4, 0,
    0, 2, 0, 0, 4, 0, 4, 0, 1, 4, 0, 4, 1, 4, 1, 4, 10, 15, 4, 1,
    4, 1, 4, 1, 4, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 4, 12, 2, 5,
    73, 110, 116, 101, 114, 4, 2, 0, 2, 8, 1, 0, 0, 0, 0, 0, 0, 0, 1, 4,
    0, 4, 0, 4, 0, 4, 0, 1, 1, 245, 254, 254, 254, 1, 4, 0, 4, 0, 4, 0,
    4, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 6, 110, 111, 114, 109, 97, 108,
    4, 0, 4, 0, 0, 4, 115, 101, 101, 100, 3, 85, 84, 67, 2, 101, 110, 0, 35, 0,
    43, 15, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// The timeout of 2026-11-03 (run 41573652529, artifact `timeout-03955d2d…`), cut the same way:
/// A1 holds `=A1+1` with a decoded count of 50,529,030 and a delta of 3.7e-294. One recalculation
/// took 6.7 s in a release build, and over 30 s under the fuzzer, before T-4048. Its three linked
/// books (6, 3 and 3 empty sheet names, nothing else) were re-encoded when `ExternalBook` gained
/// `target` and `data_sheets` (T-4143): the decoded workbook is the same, with both fields empty.
#[rustfmt::skip]
const NIGHT_TWO: [u8; 380] = [
    0, 0, 1, 2, 65, 49, 0, 0, 6, 83, 104, 101, 105, 116, 49, 1, 1, 255, 4, 0,
    4, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 4, 1, 1, 82, 0, 10, 0, 0, 0,
    0, 0, 240, 63, 4, 0, 4, 0, 4, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 64,
    6, 2, 6, 2, 1, 11, 82, 91, 45, 48, 93, 67, 91, 48, 93, 43, 49, 4, 1, 4,
    0, 0, 2, 0, 0, 4, 0, 4, 0, 1, 4, 0, 4, 1, 4, 1, 4, 10, 13, 4,
    1, 4, 1, 4, 1, 4, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 4, 12, 2,
    5, 73, 110, 116, 101, 114, 4, 2, 0, 2, 8, 1, 0, 0, 0, 0, 0, 0, 0, 1,
    4, 0, 4, 0, 4, 0, 4, 0, 1, 1, 1, 1, 1, 1, 1, 4, 0, 4, 0, 4,
    0, 4, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 6, 110, 111, 114, 109, 97,
    108, 4, 0, 4, 0, 0, 4, 115, 101, 101, 100, 3, 85, 84, 67, 2, 101, 110, 0, 1,
    0, 6, 3, 3, 3, 0, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3,
    3, 3, 10, 0, 2, 54, 3, 10, 0, 0, 10, 0, 10, 0, 10, 0, 10, 0, 0, 0,
];

/// Each decoded count is past 32,767, so it falls back to Excel's 100; each delta is finite and
/// non-negative, so it is kept. The cycles then sweep 100 times: night two's `=A2+1` goes from 2
/// to 102, and night one's doubling cells end 2^100 times their start.
#[test]
fn fuzz_reproducers_sweep_excel_default_count() {
    let mut model = Model::from_bytes(&NIGHT_ONE, "en").unwrap();
    assert_eq!(
        model.workbook.settings.iteration,
        Some(Iteration {
            max_iterations: 100,
            max_change: 0.0,
        })
    );
    model.evaluate();
    assert_eq!(model._get_text("A2"), "2.5353E+30");
    assert_eq!(model._get_text("A4"), "1.01412E+31");

    let mut model = Model::from_bytes(&NIGHT_TWO, "en").unwrap();
    assert_eq!(
        model.workbook.settings.iteration,
        Some(Iteration {
            max_iterations: 100,
            max_change: 3.720_974_344_869_6e-294,
        })
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("A2"), "102");
}

/// The count and the delta each fall back on their own; values in Excel's range are kept,
/// including both ends of the count's.
#[test]
fn each_setting_out_of_range_falls_back_to_its_default() {
    let default = Iteration::default();
    for count in [0, Iteration::MAX_ITERATIONS + 1, 50_529_030, u32::MAX] {
        let got = Iteration {
            max_iterations: count,
            max_change: 0.5,
        }
        .within_excel_range();
        assert_eq!(got.max_iterations, default.max_iterations, "count {count}");
        assert_eq!(got.max_change, 0.5);
    }
    for delta in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        -0.001,
        -f64::MIN_POSITIVE,
    ] {
        let got = Iteration {
            max_iterations: 7,
            max_change: delta,
        }
        .within_excel_range();
        assert_eq!(got.max_iterations, 7);
        assert_eq!(got.max_change, default.max_change, "delta {delta}");
    }
    for kept in [
        Iteration {
            max_iterations: 1,
            max_change: 0.0,
        },
        Iteration {
            max_iterations: Iteration::MAX_ITERATIONS,
            max_change: 1e300,
        },
    ] {
        assert_eq!(kept.within_excel_range(), kept);
    }
}

/// A decoded workbook goes through the same rule: out-of-range settings are replaced at load,
/// in-range ones survive a save and reload unchanged, and iteration off stays off.
#[test]
fn a_decoded_workbook_keeps_in_range_settings_and_replaces_the_rest() {
    let load = |iteration: Option<Iteration>| {
        let mut model = new_empty_model();
        model.workbook.settings.iteration = iteration;
        let bytes = model.to_bytes();
        Model::from_bytes(&bytes, "en")
            .unwrap()
            .workbook
            .settings
            .iteration
    };
    let in_range = Iteration {
        max_iterations: Iteration::MAX_ITERATIONS,
        max_change: 0.0,
    };
    assert_eq!(load(Some(in_range)), Some(in_range));
    assert_eq!(load(None), None);
    assert_eq!(
        load(Some(Iteration {
            max_iterations: u32::MAX,
            max_change: f64::NAN,
        })),
        Some(Iteration::default())
    );
}
