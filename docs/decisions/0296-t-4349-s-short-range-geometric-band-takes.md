# 0296 — T-4349's short-range geometric band takes decision 0083's ulp allowance for seven CUMPRINC cells

- **Status:** accepted (2026-11-05, T-4349). It applies decision 0188 R4 and decision 0083's standing ulp allowance,
  answers decision 0295's open question for one band, and adds no rule.
- **Context:**
  - Decision 0295 kept CUMPRINC and CUMIPMT's short-range cut `GEOMETRIC_MIN_LEAD` (0.3) and asked T-4349 for a
    decider that beats it on test and on a fresh sweep drawn after its freeze.
  - T-4349 built the engine with the cut at 0 (the geometric sum at every lead) beside today's, so every selector
    between the two forms could be scored without another build (docs/research/t4349/dump.py; the live build gave
    the same counts). Over the 923 old and T-3524 cells where the forms differ each is bit-exact on only about 30% (265 and 286), so neither is
    Excel's algorithm there and no selector replaces the cut outright.
  - Selectors of one to three conditions on the type, the rate, the lead and the period count were ranked on the
    train side of the old cells (fit `cumulative`, the fit lab's fixed split), keeping only those that put no old cell
    newly visibly off. The first is the band: at most 4 periods, rate at least 1E-3, lead below 0.095. Against the
    cut, bit-exact: train 403 vs 386 of 880, held out 156 vs 149 of 358, T-3524's sweeps 574 vs 566 and 475 vs 475
    of 1,000; on the sweep drawn after the freeze (probes-numeric/t4349-cumprinc-short-selector-fresh-sweep) 299 vs
    258 of 800. Visibly off: 45 vs 47 (train), 14 vs 16 (test), 57 vs 61, 140 vs 140 and 190 vs 216 (51 new, 77
    fixed, all CUMIPMT).
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
  - `GEOMETRIC_MIN_LEAD` stays as decision 0295 records it: T-4349 found no decider that replaces it.
- **Consequences:** eval/baseline/probes.txt gains exactly the seven lines above and loses the fixed cells' lines;
  reviewers confirm the baseline diff adds no other line. T-4390 owns the seven cells and the visibly-off cells of
  the three short-range sweeps. Decision 0295's owner question (decision 0083's order against R4.3 for the cut
  itself) stays open until T-4390 finds Excel's own form or shows none.
