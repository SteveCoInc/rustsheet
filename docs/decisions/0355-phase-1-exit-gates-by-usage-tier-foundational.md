# 0355 — Phase 1 exit gates by usage tier: foundational and expert functions

- **Status:** accepted (2026-11-08, owner rulings; recorded by T-5091, from the eval report of main b84a7256).
  SPEC-0037 says how it is built, and tasks T-5092 to T-5095 build it.
- **Amends** (each from the moment the piece that implements it lands; the old rule holds until then):
  - decision 0188 R1 and R2: Gate A and Gate B are replaced by the tier gates of item 3. Their old numbers stay in
    the report as information;
  - decision 0188 R1(c) and decision 0146: the milestone probe and ironcalc-calc rows count by the envelope and the
    tier bars (item 4);
  - decision 0267: its 13-digit bar no longer gates anything, since the envelope's bar is 12 digits;
  - CLAUDE.md prime directive 2: in the eval gate, "never regress" means no cell leaves the envelope, no type or
    error goes wrong, and no tier gate goes from met to unmet (item 6);
  - decision 0023's task order: among tasks of the same milestone and priority, foundational work comes first
    (item 7).
- **Context:**
  - **The owner's aim** (2026-11-08): common, foundational functions should be nearly perfect, and always right on
    type and error. Rare expert functions must not be embarrassingly far from Excel or from the math; some type and
    error misses are acceptable there. The last-digit work is costly and nobody sees it. Of the numeric cells that
    differ from Excel, almost all agree to 12 digits or more, and on many of them Excel's own value is further from
    the 50-digit math than ours (decision 0188, Context).
  - **Usage data** (docs/research/function-usage-2026-11-08):
    - the catalog's `prior`: Enron, EUSES and FUSE for the most-used functions, and hand-set relative to them for
      the rest;
    - a count over SpreadsheetBench, the real-world corpus that corpus-bench samples: 1,136 input workbooks that
      contain formulas.

    eval/catalog/usage.tsv lists every in-scope function. The functions whose parity is hardest appear in none of those
    1,136 workbooks: IRR, RATE, BETA.INV, F.INV, F.INV.RT, T.TEST, CHIINV, GAMMA.DIST and the FORECAST.ETS
    family.
  - **Measured at b84a7256.** These numbers come from the committed report, not from a new run. They leave out the
    decision 0134 Phase 1 exceptions, the decision 0036 deviations and the pivot cells (decision 0354).

    | Tier | Functions | Attributed cells | Type or error wrong | Outside the envelope |
    |---|---|---|---|---|
    | Foundational | 122 | 2,119,054 | 31 (99.9985%) | 1 (99.99995%) |
    | Expert | 384 | 902,063 | 1,355 (99.850%) | 405 (99.955%) |

    - **The foundational misses:**
      - TEXT `[s].0`, 28 cells (T-4333, with T-4419, T-4460 and T-4466);
      - `COUNTIF(XLOOKUP({2;9},…),…)`, probes/t4742 A40 and C40 (T-4826);
      - `EDATE(IF(TRUE,),1)` (T-4912);
      - one NPV cell of ironcalc-calc IRR.xlsx (M17), evaluated at IRR's root (T-2131).
      - The other foundational cells that look off (CELL, SIN, COS, AND, SUMPRODUCT and CHAR on calipers and poi)
        are decision 0134 exceptions: Excel for the web agrees with the engine, and only a desktop cache disagrees.
    - **Expert, pooled:** 99.955% inside the envelope meets 99.9%. 99.850% on type does not: it needs at most 902
      wrong. IRR alone has 930.
    - **Expert, per function at 99%:** 12 functions miss.
      - IRR (75.3% type);
      - RATE (96.5% type, 98.4% envelope);
      - BETA.INV (96.4% type);
      - T.TEST (91.3% type, 95.7% envelope);
      - F.INV (96.8% envelope), F.INV.RT (98.3% type, 97.1% envelope), FINV (97.5% envelope);
      - CHIINV (98.7% envelope), F.DIST.RT (98.8% envelope), NORMINV (99.0% envelope, one cell);
      - ACCRINT (98.9% type), MROUND (97.7% type).
- **Decision:**
  1. **Usage tiers.** Every in-scope catalog function is foundational or expert.
     - It is **foundational** if its catalog `prior` is at least 0.5, or if at least 6 of the 1,136 SpreadsheetBench
       workbooks with formulas call it (about 0.5%). That makes 122 functions; IRR (prior 0.4, no workbook) is
       expert.
     - **Every other in-scope function is expert**, and so is a function added to the catalog later, until a
       decision moves it.
     - The tier is the `usage:` field of eval/catalog/functions.yaml: `usage: foundational`, and absent means
       expert. It changes only by a decision record, and never so that a gate passes.
  2. **The envelope.** A value is inside it if it is either:
     - within 12 significant digits of Excel's value (`closeness::within_significant_digits` with 12); or
     - within 1e-9 relative of the 50-digit math value (absolute floor 1e-300).

     A wrong type, a wrong error, wrong text, or an error against a number is never inside it. Cells where decision
     0188 R3 follows the math are checked against the math only.
  3. **The tier gates.** These replace Gate A and Gate B:

     | | Type, error and text right | Numbers inside the envelope |
     |---|---|---|
     | **Foundational** | 100% of the tier's cells | ≥ 99.99% of the tier's cells |
     | **Expert** | ≥ 99.9% of the tier's cells, and ≥ 99% of each function's cells | ≥ 99.9% of the tier's cells, and ≥ 99% of each function's cells |

     - **Cells counted:**
       - the probes, probes-numeric and the eval/accuracy tables;
       - the Gate B suites: the gen-* suites, ironcalc-calc, corpus-bench, calipers, poi, smoke and fixtures.
     - **Attribution:** as decision 0118 says.
     - **Left out:** the decision 0134 exceptions, the decision 0036 deviations, and the pivot suites and
       GETPIVOTDATA (Phase 2, decision 0354).
  4. **Milestone rows.** A milestone's probe and ironcalc-calc rows (R1(c)) count only the cells outside the
     envelope or with a wrong type, error or text. A row is met when its counted cells, by tier, meet item 3's
     bars. Unchanged: coverage, the evidence floor, the owners criterion (every failure has an open task or a
     `DEVIATION` entry), and the semantic probe-row criteria.
  5. **What stays as information.**
     - The report shows the old Gate A (15 digits or 1e-9) and Gate B (99.9% within 15 digits) numbers on every
       run, beside the tier gates.
     - The baselines keep every bit-level failure.
     - Exact Excel bit parity stays the target of Phase 1b (`defer: phase-1b`).
  6. **The CI gate ratchets each cell against the envelope, not each bit.** On a pull request and on the merge
     group, the eval gate fails if:
     - a cell inside the envelope leaves it;
     - a cell with the right type, error or text gets it wrong;
     - or a tier gate goes from met to unmet.

     Bit drift inside the envelope is allowed. It is recorded in the baseline by `make eval-baseline` without
     `--allow-regressions`, so each drift shows in the PR's diff. `--allow-regressions` stays as decision 0329
     limits it.

     T-5093 builds it: the cell baselines carry `E` lines (the known failures `cell_envelope` puts outside, and
     every whole-case item) and `W` lines (those of them `cell_envelope` calls `WrongType`) beside the bit-level
     `F` lines. `baseline::diff` reports a new `E` or `W` item as a regression, so an item outside as a number
     that becomes a wrong type is caught; a new `F` item without an `E` item as drift; an item leaving `E` as
     into the envelope; and one leaving `W` but staying in `E` as right type. The
     visible-parity lock counts `cell_envelope`'s `Outside` and `WrongType` cells, and eval/baseline/usage-gates.txt
     records each tier's met state on the vendored gated suites.
     `make catch-up` and `make renumber` resolve a conflict on that lock to `main`'s side, re-record it, and
     refuse a merge that leaves a tier unmet that `main` has met (T-5096).
  7. **Task order.** Among ready tasks of the same milestone and priority, a task about a foundational function
     comes before any other.
     - P0, handoffs, re-reviews and campaign stages still come first, as decisions 0023, 0094 and 0095 say.
     - A task's tier is its `usage:` front-matter key. Without one, it is the tier of the first catalog function
       its title names, since titles name their subject first (`scripts/task_usage.py`, T-5095). A title that
       quotes another function first sets the key.
     - `tasks.py new` and the planning agents set the key on function tasks.
  8. **Reporting.** The eval report (report.json `usage_gates` and report.md), `make phase1-exit`, the daily
     progress dashboard and the steward's daily status all lead with the two tier gates: rates against the bars,
     met or not, and the failing functions. On the dashboard this is the "Usage tiers" block right after the bottom line
     (`tools/src/rustsheet_tools/progress_report/usage.py`, T-5094), and in docs/status.md the `**Tier gates:**`
     line that leads the daily snapshot.
- **Consequences:**
  - **What Phase 1 needs, measured at b84a7256:**
    - the 32 foundational cells. Their owner tasks are P1 (T-2131, T-4333, T-4419, T-4460, T-4466, T-4826 and
      T-4912);
    - expert type misses down to at most 0.1% pooled: IRR's convergence (930 cells) is the lever;
    - the 12 expert functions under 99%;
    - the milestone rows under item 4.
  - The 13- to 15-digit tasks keep their `defer: phase-1b` or get it as the gates land. No last-digit task blocks
    Phase 1.
  - **How it is built:**
    - T-5091 lands this record, SPEC-0037 and the catalog field;
    - T-5092: the envelope, the report and `make phase1-exit`;
    - T-5093: the CI ratchet and the CLAUDE.md wording;
    - T-5094: the dashboard and the daily status;
    - T-5095: task order and creation.

    Until each lands, the rule it replaces stays in force.
