# 0296 — T-4349's short-range geometric band: seven CUMPRINC cells one ulp off, on decision 0083's conditions

- **Status:** accepted (2026-11-05, T-4349). It applies decision 0188 R4. T-4349 is not a `campaign: visible` task,
  so decision 0083's standing allowance does not cover it: this record is its own allowance for the seven cells
  below, held to 0083's four conditions. It answers decision 0295's question for one band and files the rest as an
  owner ruling (T-4400).
- **Context:**
  - Decision 0295 kept CUMPRINC and CUMIPMT's short-range cut `GEOMETRIC_MIN_LEAD` (0.3) and asked T-4349 for a
    decider that beats it on test and on a fresh sweep drawn after its freeze.
  - T-4349 built the engine with the cut at 0 (the geometric sum at every lead) beside today's, so every selector
    between the two forms could be scored without another build (docs/research/t4349/dump.py; the live build gave
    the same counts). Over the 923 old and T-3524 cells where the forms differ each is bit-exact on only about 30% (265 and 286), so neither is
    Excel's algorithm there and no selector replaces the cut outright.
  - Selectors of one to three conditions on the type, the rate, the lead and the period count, at cut-points fixed
    before scoring, were ranked on the train side of the old cells only (fit `cumulative`, the fit lab's fixed split),
    keeping only those that put no train cell newly visibly off (round 1 filtered on held-out cells too; on the train
    side alone the same selector is first). The first is the band: at most 4 periods, rate at least 1E-3, lead below 0.095. Against the
    cut, bit-exact: train 403 vs 386 of 880, held out 156 vs 149 of 358, T-3524's sweeps 574 vs 566 and 475 vs 475
    of 1,000; on the sweep drawn after the freeze (probes-numeric/t4349-cumprinc-short-selector-fresh-sweep) 299 vs
    258 of 800. Visibly off: 45 vs 47 (train), 14 vs 16 (test), 57 vs 61, 140 vs 140 and 190 vs 216 (51 new, 77
    fixed, all CUMIPMT). Within 4 ulps (the oracle's reproducibility floor, R4.4): train 825 vs 827, test 340 vs
    342, T-3524 fresh 922 vs 926, tiny 835 vs 835, the new sweep 542 vs 558; where the forms differ by more than 4
    ulps the band is closer on 133 cells of the new sweep and the cut on 87.
  - R4.1 (`rustsheet-fitlab cumulative` and `cumulative-short` at the PR head), bit-exact / within 4 ulps / within 15
    digits, the engine against the correctly rounded candidate: old train 403/827/835 vs 252/709/723 of 880, old test
    156/342/344 vs 106/298/303 of 358 (train 45.8%, test 43.6%, within the 5-point margin: accepted); new sweep
    216/401/435 vs 120/331/390 of 566 and 83/157/176 vs 57/148/167 of 234 (accepted). Gate A on the new sweep: 800 of
    800 within 1e-9 of the 50-digit value (`kernels.cumulative.exact`), none outside, max 1,024 ulps.
  - The band costs seven gated cells, bit-exact before and 1 ulp off now, each within 15 digits:
    | cell | call | Excel | ours |
    |---|---|---|---|
    | probes/fin-cumprinc-near-1e15 Sheet1!B2 | `CUMPRINC(0.001,1E15,100000,1E15-1,1E15,0)` | -199.70039950059933 | -199.7003995005993 |
    | probes/fin-cumprinc-near-1e15 Sheet1!B3 | `CUMPRINC(0.001,1E15,100000,1E15-2,1E15,0)` | -299.40099850209725 | -299.4009985020972 |
    | probes/fin-cumprinc-single-period-rate-scan Sheet1!K52 | `CUMPRINC(0.00271227,100,100000,98,98,0)` | -1133.8327647960855 | -1133.8327647960853 |
    | probes/fin-cumprinc-single-period-rate-scan Sheet1!K57 | `CUMPRINC(0.00584341,100,100000,98,98,0)` | -1300.3725575224778 | -1300.372557522478 |
    | probes/fin-cumprinc-single-period-rate-scan Sheet1!K59 | `CUMPRINC(0.00794328,100,100000,98,98,0)` | -1418.8829994872897 | -1418.88299948729 |
    | probes/fin-cumprinc-underflow-ends Sheet1!D2 | `CUMPRINC(0.001,1E10,100000,1E10-1,1E10,0)` | -199.70039950059933 | -199.7003995005993 |
    | probes/fin-cumprinc-underflow-ends Sheet1!D3 | `CUMPRINC(0.001,1E10,100000,1E10-2,1E10,0)` | -299.40099850209725 | -299.4009985020972 |

    It fixes 31 gated probes cells, 4 of them visibly off (fin-cumprinc-underflow-ends I3, I4; fin-cumprinc-near-1e15
    E3, E4: CUMIPMT 32 to 64 ulps off before), and no gen-m9 or ironcalc-calc cell changes.
  - The seven meet decision 0083's four conditions: numeric and within 4 ulp; nothing newly visibly off in a gated
    suite; 4 visibly-off cells fixed against none regressed; T-4349's Outcome lists them and T-4390 owns them.
- **Decision:**
  - `short_takes_geometric` (functions/financial/cumulative.rs) sends a short range inside the band to the geometric
    sum below `GEOMETRIC_MIN_LEAD`, with one `FITTED(T-4349, ...)` tag over its three constants; quality/fitted.json
    rises by one for this fit (`scripts/check_fitted.py --update`).
  - `rustsheet-eval update-baseline --allow-regressions` runs once in the T-4349 PR, for the seven cells above alone.
  - `GEOMETRIC_MIN_LEAD` stays as decision 0295 records it: T-4349 found no decider that replaces it, and filed the
    ruling decision 0295 asks for as T-4400. Its `FITTED` range is tightened to where it decides (a lead below 0.3,
    so rates below 0.35): 2,241 cells, old probes at rates 1E-300 to 0.1 and the fresh sweeps 1E-12 to 0.35.
- **Consequences:** eval/baseline/probes.txt gains exactly the seven lines above and loses the fixed cells' lines;
  reviewers confirm the baseline diff adds no other line. T-4390 owns the seven cells and the visibly-off cells of
  the three short-range sweeps. Decision 0295's owner question (decision 0083's order against R4.3 for the cut
  itself) is T-4400; T-4390 may make it moot by finding Excel's own form.
