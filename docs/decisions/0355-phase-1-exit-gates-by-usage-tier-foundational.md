# 0355 — Phase 1 exit gates by usage tier: foundational and expert functions

- **Status:** proposed (2026-11-08, owner request; recorded from the eval report of main b84a7256). It becomes
  accepted when the owner confirms items 1 and 3 on its PR. Once accepted, it amends:
  - decision 0188 R1 and R2: Gate A and Gate B are replaced by the tier gates of item 3;
  - decision 0188 R1(c) and decision 0146: the milestone probe and ironcalc-calc rows apply the tier bars (item 4);
  - decision 0267: its 13-digit bar no longer gates anything, since the envelope's bar is 12 digits;
  - CLAUDE.md prime directive 2, "never regress": in the eval gate it means that no cell leaves the envelope and
    no cell's type or error goes wrong (item 6).
- **Context:**
  - The owner's aim (2026-11-08): common, foundational functions should be nearly perfect, and always right on
    type and error. Rare expert functions must not be embarrassingly far from Excel or from the math, but some
    type and error misses are acceptable there.
  - The last-digit work is costly, and nobody sees it. Of the numeric cells that differ from Excel, almost all
    agree to 12 digits or more. On many of them Excel's own value is further from the 50-digit math than ours
    (decision 0188, Context).
  - **Usage data** (docs/research/function-usage-2026-11-08):
    - the catalog's `prior` (Enron, EUSES and FUSE for the most-used functions; hand-set relative to them for the
      rest);
    - a count over SpreadsheetBench, the real-world corpus that corpus-bench samples: 1,136 input workbooks that
      contain formulas. `usage.tsv` lists every in-scope function.
    - The functions whose parity is hardest appear in none of those 1,136 workbooks: IRR, RATE, BETA.INV, F.INV,
      F.INV.RT, T.TEST, CHIINV, GAMMA.DIST and the FORECAST.ETS family.
  - **Measured at b84a7256.** These numbers come from the committed report, not from a new run. They leave out the
    decision 0134 Phase 1 exceptions, the decision 0036 deviations and the pivot cells (decision 0354).

    | Tier | Functions | Attributed cells | Type or error wrong | Outside the envelope |
    |---|---|---|---|---|
    | Foundational | 122 | 2,119,054 | 31 (99.9985%) | 1 (99.99995%) |
    | Expert | 384 | 902,063 | 1,355 (99.850%) | 405 (99.955%) |

    - The foundational misses:
      - TEXT `[s].0`, 28 cells (T-4333, with T-4460 and T-4466);
      - `COUNTIF(XLOOKUP({2;9},…),…)`, probes/t4742 A40 and C40 (T-4826);
      - `EDATE(IF(TRUE,),1)` (T-4912);
      - one NPV cell of ironcalc-calc IRR.xlsx (M17), evaluated at IRR's root (T-2131).
      - The other foundational cells that look off (CELL, SIN, COS, AND, SUMPRODUCT and CHAR on calipers and poi)
        are decision 0134 exceptions: Excel for the web agrees with the engine, and only a desktop cache disagrees.
    - Expert functions below 90% on either measure: IRR only, at 75.3% type match (it returns `#NUM!` where Excel
      converges, decision 0188 Context item 1).
    - Expert functions below 99% on either measure:
      - IRR, RATE, BETA.INV, T.TEST, F.INV, F.INV.RT, FINV, CHIINV, F.DIST.RT and NORMINV;
      - ACCRINT and MROUND on type and error.
- **Decision:**
  1. **Usage tiers.** Every in-scope catalog function is foundational or expert:
     - It is **foundational** if its catalog `prior` is at least 0.5, or if at least 6 of the 1,136 SpreadsheetBench
       workbooks with formulas call it (about 0.5%). That makes 122 functions at b84a7256. IRR (prior 0.4, no
       workbook) is expert.
     - **Every other in-scope function is expert**, and so is a function added to the catalog later, until a
       decision moves it.
     - The tier is recorded in eval/catalog/functions.yaml as `usage: foundational | expert`. It changes only by a
       decision record, and never so that a gate passes.
  2. **The envelope.** A value is inside it if it is either:
     - within 12 significant digits of Excel's value (`closeness::within_significant_digits` with 12); or
     - within 1e-9 relative of the 50-digit math value (absolute 1e-300 near 0).

     A wrong type, a wrong error, wrong text, or an error against a number is never inside it. Cells where decision
     0188 R3 follows the math are checked against the math only.
  3. **The tier gates.** These replace Gate A and Gate B:

     | | Type, error and text right | Numbers inside the envelope |
     |---|---|---|
     | **Foundational** | 100% of the tier's cells | ≥ 99.99% of the tier's cells |
     | **Expert** | ≥ 99% of the tier's cells, and ≥ 90% of each function's cells | the same: ≥ 99% pooled, ≥ 90% per function |

     - **Cells counted:** the probes, probes-numeric, the eval/accuracy tables, and the Gate B suites (the gen-*
       suites, ironcalc-calc, corpus-bench, calipers, poi, smoke and fixtures).
     - **Attribution:** as decision 0118 says.
     - **Left out:** the decision 0134 exceptions, the decision 0036 deviations, and the pivot suites and
       GETPIVOTDATA (Phase 2, decision 0354).
     - The 90% per-function floor stops one broken expert function from hiding inside the pooled 99%.
  4. **Milestone rows.** A milestone's probe and ironcalc-calc rows (R1(c)) are met when the cells of the
     milestone's functions meet the item 3 bars of their tiers. Coverage, the evidence floor, the owners criterion
     (every failure has an open task or a `DEVIATION` entry) and the semantic probe-row criteria are unchanged.
  5. **What stays as information.** The report keeps showing the old Gate A (15 digits or 1e-9) and Gate B
     (99.9% within 15 digits) numbers on every run, beside the tier gates. Exact Excel bit parity stays the target
     of Phase 1b (`defer: phase-1b`).
  6. **The CI gate ratchets each cell against the envelope, not each bit.** On a pull request and on the merge
     group, the eval gate fails if:
     - a cell inside the envelope leaves it;
     - a cell with the right type, error or text gets it wrong;
     - or a tier gate goes from met to unmet.

     A change that moves a cell's bits inside the envelope passes, and the gate reports it as "bit drift". In every
     case, eval/baseline keeps each cell's exact value. That way the Phase 1b work starts from facts, and a drift
     can be traced to its commit. `--allow-regressions` stays as decision 0329 limits it.
- **Consequences:**
  - The Phase 1 numeric work narrows to:
    - the 32 foundational cells, whose owner tasks move to P1 in this decision's PR (T-4333, T-4419, T-4460,
      T-4466, T-4826, T-4912 and T-2131);
    - IRR convergence;
    - the per-function floor, which only IRR misses.
  - Last-digit and 13- to 15-digit tasks are left alone. Each keeps its `defer:` or gets `defer: phase-1b` once
    the gates land.
  - Implementing this is a harness change, specified first (CLAUDE.md directive 4):
    - the `usage:` catalog field;
    - the tier gate rows in the report;
    - `scripts/phase1_exit.py`;
    - the eval gate's envelope ratchet;
    - the CLAUDE.md directive 2 wording.

    Until that lands, Gate A, Gate B and the bit-level baseline stay in force.
