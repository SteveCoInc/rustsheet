// T-3408 research replay, not compiled with the crate. `scratch_scan` runs an uncapped copy of
// `irr_secant` (the run from the guess and the retry from 0.1: how each ends and at which step), the
// source of long_runs.tsv and stalls.tsv. `scratch_caps` runs `compute_irr` with each candidate cap
// applied inside `IRR_LONG_RUN_RANGE` and keeps the cells whose result changes, the source of
// cap_runs.tsv. To rerun: copy this next to financial_util.rs as scratch_t3408.rs, add
// `#[cfg(test)] #[path = "scratch_t3408.rs"] mod scratch_t3408;` to that file, write the cell list
// (`case!Sheet!A1 TAB guess TAB excel TAB flows`, one IRR literal cell with an oracle value per line)
// and run `T3408_IN=cells.tsv T3408_OUT=out.tsv cargo test --release -p rustsheet_engine --lib
// scratch_scan -- --ignored` (or `scratch_caps`). The TSVs add each cell's fit lab side.
#![allow(clippy::all, clippy::pedantic, unused)]
use super::*;


pub(super) const CAPS: [usize; 14] = [
    50, 100, 150, 190, 200, 207, 216, 217, 241, 260, 281, 291, 300, 400,
];

/// `irr_secant` with the in-range cap `cap` (50 outside the range), the shipped stall rule and flat rule.
fn secant_cap(values: &[f64], guess: f64, cap: usize) -> (Result<f64, SecantStop>, usize, bool) {
    let npv = |d: f64| irr_npv(values, 1.0 - d).ok_or(SecantStop::DivZero);
    let peak = largest_flow(values);
    let steps = if irr_secant_steps(values, guess) == IRR_LONG_RUN_STEPS {
        cap
    } else {
        IRR_SHORT_RUN_STEPS
    };
    let mut d0 = extended_quotient(guess, 1.0 + guess);
    let mut f0 = match npv(d0) {
        Ok(f) => f,
        Err(e) => return (Err(e), 0, false),
    };
    let mut d1 = d0 - if f0 > 0.0 { 1e-3 } else { -1e-3 };
    let mut f1 = match npv(d1) {
        Ok(f) => f,
        Err(e) => return (Err(e), 0, false),
    };
    let mut largest_npv = f0.abs().max(f1.abs());
    for step in 0..steps {
        if f1 == f0 {
            return (
                Err(
                    if d1 == d0 && irr_flat_is_div_zero(values, guess, largest_npv) {
                        SecantStop::DivZero
                    } else {
                        SecantStop::Failed
                    },
                ),
                step + 1,
                false,
            );
        }
        let d2 = d1 - extended_quotient(f1 * (d1 - d0), f1 - f0);
        if d2 == 1.0 {
            return (Err(SecantStop::DivZero), step + 1, false);
        }
        if !d2.is_finite() || d2 > 1.0 {
            return (Err(SecantStop::Failed), step + 1, false);
        }
        let f2 = match npv(d2) {
            Ok(f) => f,
            Err(e) => return (Err(e), step + 1, false),
        };
        largest_npv = largest_npv.max(f2.abs());
        let zero = d2 == d1 && f2.abs() < 1e-7 * peak;
        let stalled = zero && step < IRR_SHORT_RUN_STEPS;
        if (d2 - d1).abs() < 1e-7 && (f2.abs() < 1e-7 || stalled) {
            return (
                irr_from_discount(1.0 - d2).ok_or(SecantStop::Failed),
                step + 1,
                stalled && f2.abs() >= 1e-7,
            );
        }
        (d0, f0, d1, f1) = (d1, f1, d2, f2);
    }
    (Err(SecantStop::Failed), steps, false)
}

fn show(r: &Result<f64, SecantStop>) -> String {
    match r {
        Ok(v) => format!("{v:?}"),
        Err(SecantStop::DivZero) => "#DIV/0!".into(),
        Err(SecantStop::Failed) => "fail".into(),
    }
}

/// `compute_irr` with in-range cap `cap`.
fn compute_cap(values: &[f64], guess: f64, cap: usize) -> String {
    let values = values
        .iter()
        .position(|&c| c != 0.0)
        .map_or(values, |first| &values[first..]);
    let (mut result, _, _) = secant_cap(values, guess, cap);
    if result == Err(SecantStop::Failed) && guess != 0.1 {
        result = secant_cap(values, 0.1, cap).0;
    }
    match result {
        Ok(root) => Some(root),
        Err(SecantStop::DivZero) => return "#DIV/0!".into(),
        Err(SecantStop::Failed) => None,
    }
    .or_else(|| irr_fallback(values))
    .filter(|r| r.is_finite())
    .map_or("#NUM!".into(), |r| format!("{r:?}"))
}

#[test]
#[ignore]
fn scratch_caps() {
    let input = std::env::var("T3408_IN").unwrap();
    let output = std::env::var("T3408_OUT").unwrap();
    let text = std::fs::read_to_string(input).unwrap();
    let mut out = String::from("cell\tin_range\tguess_run_steps\tguess_run\tretry_steps\tretry");
    for c in CAPS {
        out.push_str(&format!("\tcap_{c}"));
    }
    out.push('\n');
    for line in text.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        let guess: f64 = parts[1].parse().unwrap();
        let values: Vec<f64> = parts[3].split(',').map(|t| t.parse().unwrap()).collect();
        if !(guess > -1.0 && guess < IRR_GUESS_BOUND)
            || values.iter().all(|&x| x >= 0.0)
            || values.iter().all(|&x| x <= 0.0)
        {
            continue;
        }
        let res: Vec<String> = CAPS
            .iter()
            .map(|&c| compute_cap(&values, guess, c))
            .collect();
        if res.iter().all(|r| r == &res[0]) {
            continue;
        }
        let trimmed = values
            .iter()
            .position(|&c| c != 0.0)
            .map_or(&values[..], |f| &values[f..]);
        let (g, gs, _) = secant_cap(trimmed, guess, 100_000);
        let (r, rs, _) = if guess != 0.1 {
            secant_cap(trimmed, 0.1, 100_000)
        } else {
            (Err(SecantStop::Failed), 0, false)
        };
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            parts[0],
            irr_secant_steps(trimmed, guess) == IRR_LONG_RUN_STEPS,
            gs,
            show(&g),
            if guess != 0.1 {
                rs.to_string()
            } else {
                "-".into()
            },
            if guess != 0.1 { show(&r) } else { "-".into() }
        ));
        for r in res {
            out.push_str(&format!("\t{r}"));
        }
        out.push('\n');
    }
    std::fs::write(output, out).unwrap();
}



/// Instrumented copy of irr_secant without the cap: returns a feature line.
pub(super) fn run(values: &[f64], guess: f64, cap: usize) -> String {
    let npv = |d: f64| irr_npv(values, 1.0 - d);
    let peak = largest_flow(values);
    let mut d0 = extended_quotient(guess, 1.0 + guess);
    let Some(mut f0) = npv(d0) else {
        return "div0@init".into();
    };
    let mut d1 = d0 - if f0 > 0.0 { 1e-3 } else { -1e-3 };
    let Some(mut f1) = npv(d1) else {
        return "div0@init".into();
    };
    let mut largest = f0.abs().max(f1.abs());
    let (mut dmin, mut dmax) = (d0.min(d1), d0.max(d1));
    let mut signchg = 0;
    let mut zero_steps = 0;
    for k in 0..cap {
        if f1 == f0 {
            return format!(
                "flat\t{k}\t{}\t{dmin}\t{dmax}\t{largest:e}\t{signchg}\t{zero_steps}\t",
                d1 == d0
            );
        }
        let d2 = d1 - extended_quotient(f1 * (d1 - d0), f1 - f0);
        if d2 == 1.0 {
            return format!("v0\t{k}");
        }
        if !d2.is_finite() || d2 > 1.0 {
            return format!("out\t{k}\t{d2}");
        }
        let Some(f2) = npv(d2) else {
            return format!("div0\t{k}\t{d2}");
        };
        if d2 == d1 {
            zero_steps += 1;
        }
        if (f2 > 0.0) != (f1 > 0.0) {
            signchg += 1;
        }
        largest = largest.max(f2.abs());
        dmin = dmin.min(d2);
        dmax = dmax.max(d2);
        let stalled = d2 == d1 && f2.abs() < 1e-7 * peak;
        if (d2 - d1).abs() < 1e-7 && (f2.abs() < 1e-7 || stalled) {
            let r = irr_from_discount(1.0 - d2).map_or(f64::NAN, |r| r);
            return format!(
                "root\t{k}\t{r:?}\t{dmin}\t{dmax}\t{largest:e}\t{signchg}\t{zero_steps}\t{:e}",
                f2.abs()
            );
        }
        (d0, f0, d1, f1) = (d1, f1, d2, f2);
    }
    format!("cap\t{cap}\t\t{dmin}\t{dmax}\t{largest:e}\t{signchg}\t{zero_steps}")
}

#[test]
#[ignore]
fn scratch_scan() {
    let input = std::env::var("T3408_IN").unwrap();
    let output = std::env::var("T3408_OUT").unwrap();
    let text = std::fs::read_to_string(input).unwrap();
    let mut out = String::new();
    for line in text.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        let guess: f64 = parts[1].parse().unwrap();
        let mut values: Vec<f64> = parts[3].split(',').map(|t| t.parse().unwrap()).collect();
        while values.first() == Some(&0.0) {
            values.remove(0);
        }
        if values.is_empty()
            || values.iter().all(|&x| x >= 0.0)
            || values.iter().all(|&x| x <= 0.0)
            || !(guess > -1.0)
        {
            continue;
        }
        let main =
            compute_irr(&values, guess).map_or_else(|(e, _)| format!("{e}"), |r| format!("{r:?}"));
        let g = run(&values, guess, 5000);
        let r = if guess != 0.1 {
            run(&values, 0.1, 5000)
        } else {
            String::new()
        };
        let fb = irr_fallback(&values).map_or("none".to_string(), |r| format!("{r:?}"));
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\tG\t{g}\tR\t{r}\tFB\t{fb}\n",
            parts[0],
            parts[2],
            values.len(),
            guess,
            main
        ));
    }
    std::fs::write(output, out).unwrap();
}
