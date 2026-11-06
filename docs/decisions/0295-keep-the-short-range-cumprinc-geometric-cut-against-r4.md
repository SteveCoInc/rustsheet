# 0295 — Keep the short-range CUMPRINC geometric cut against R4's replace rule while visible parity says so

- **Status:** accepted (2026-11-05, T-3524). An owner ruling may overturn it; T-4349 re-decides it on new evidence.
- **Context:**
  - T-3524 re-validated CUMPRINC and CUMIPMT's two geometric-principal cuts (functions/financial/cumulative.rs)
    under decision 0188 R4. The long-range cut (3.6) went (decision 0291). The short-range cut `GEOMETRIC_MIN_LEAD`
    (0.3, ranges of at most 32 periods) is within the hold-out margin on the old cells but does not beat the
    geometric sum at every leading exponent on the old held-out cells (149 vs 150 bit-exact; visibly 11 lost
    against 10 won) or on either fresh sweep: over a short
    range 254 vs 260 bit-exact at rates 1E-4..2 (probes-numeric/t3524-cumprinc-lead-fresh-sweep) and 237 vs 265 at
    1E-12..1E-4 (t3524-cumprinc-tiny-rate-sweep). Visibly off, removing it loses 9 and wins 15 on the first sweep
    and loses 18 and wins 33 on the second. R4.3 and the task's own acceptance criterion then replace it.
  - Removing it (measured on the T-3524 branch, the engine with the short cut gone and the long cut already gone,
    `rustsheet-eval check` against the branch's baselines):
    - probes 171 cells regress and 158 are fixed, gen-m9 3 regress (b72d952c5f52cb4d A257, d912671baa1364aa A300,
      e168be7f6d3f84c2 A379) and 4 are fixed, and ironcalc-calc's Excel-authored `CUMPRINC_CUMIPMT.xlsx` regresses
      on G6, H6, G22 and H22;
    - 21 more cells are visibly off in 8 gated cases (fin-cumipmt-tiny-rate-structure 38 to 49,
      fin-cumipmt-overflow-and-forms 2 to 4, fin-cumprinc-near-1e15 4 to 6, fin-cumprinc-underflow-ends 4 to 6, and
      one each in gen-m9 b72d952c5f52cb4d and e168be7f6d3f84c2, fin-remaining-later-argument-ranges and
      t3179-cumipmt-ipmt-sum): the in-scope visibly-off count goes from 1,904 to 1,920;
    - six unit tests that pin Excel's values fail, one of them visibly: `CUMIPMT(1E-10,296,197540,13,19,1)` is
      -0.00013127066904417006 against Excel's -0.00013127066813467536 (gen-m9 b72d952c5f52cb4d A257).
  - So on visible parity the evidence cancels: 21 more visibly-off gated cells against 21 fewer on the two fresh
    sweeps (6 and 15 net). On the bit, the gated suites lose 16 net and the fresh sweeps gain 34. Neither form is
    Excel's over a short range: the leading exponent does not decide it, and the old tiny-rate cells and the fresh
    sweeps disagree below a lead of 0.095. Decision 0083 orders the work visible parity first, then ulps, then
    bits; decision 0188 R4 forbids keeping a fit that does not generalize. Here they conflict.
- **Options considered:**
  - *Remove the cut (R4.3 as written).* Trades 21 visibly-off gated cells, 4 Excel-authored workbook cells and six
    unit tests pinned to Excel for 21 visibly-off fresh cells. No visible gain, and a known visible loss on gated
    Excel evidence.
  - *Refit the cut.* A new fit on the cells that just failed it, which R4.2 would need another fresh sweep to
    accept; T-3524's analysis found no lead cut that wins in every bucket.
  - *Keep it, recorded (this decision).*
- **Decision:** `GEOMETRIC_MIN_LEAD` stays, over a short range only, with its `FITTED` tag restated to the cells it
  was measured on. This is a recorded deviation from decision 0188 R4.3 for this one selector, not a keep-matched
  precedent: it holds only while removing the cut puts at least as many gated cells visibly off as it fixes on fresh
  sweeps. T-4349 (P1) owns the question: it must find a decider (rate, type, range length or underflow) that beats
  the cut on test and on a fresh sweep drawn after its freeze, or show that none exists, and then either replace
  the cut or ask the owner to rule.
- **Consequences:** T-3524's acceptance criterion on replacing a selector is met for the long cut only; its Outcome
  cites this record for the short one. Reviewers must not cite this record to keep another fit that loses its
  fresh sweep: each such case needs its own record with the same visible-parity accounting. No baseline line
  changes because of this decision.
