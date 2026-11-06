// T-3408 research replay, not compiled with the crate. An uncapped copy of `irr_secant` that prints,
// per IRR cell, how the run from the guess and the retry from 0.1 end and at which step (the source of
// long_runs.tsv). To rerun: copy it next to financial_util.rs as scratch_t3408.rs, add
// `#[cfg(test)] #[path = "scratch_t3408.rs"] mod scratch_t3408;` to that file, write the cell list
// (`case!Sheet!A1 TAB guess TAB excel TAB flows`) and run
// `T3408_IN=cells.tsv T3408_OUT=scan.tsv cargo test --release -p rustsheet_engine --lib scratch_scan -- --ignored`.
#![allow(clippy::all, clippy::pedantic, unused)]
use super::*;

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
