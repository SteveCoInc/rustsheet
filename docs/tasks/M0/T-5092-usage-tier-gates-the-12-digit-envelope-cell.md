---
id: T-5092
title: "Usage-tier gates: the 12-digit envelope, cell_envelope, R1(c), usage_gates and the Phase 1 exit lines (SPEC-0037 step 2)"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-5091]
spec: docs/specs/SPEC-0037-usage-tier-gates.md
---

## Goal
The eval measures decision 0355's tier gates: the 12-digit envelope everywhere it is used, the `usage_gates` report
section, R1(c) and the milestone rows, and the `foundational` and `expert` lines of `scripts/phase1_exit.py`.

## Context
- SPEC-0037, design sections 2, 3, 4 and 5. The schema and signatures there are fixed, because T-5093, T-5094 and
  T-5095 are built against them in parallel.
- Code: crates/rustsheet-eval/src/accuracy/envelope.rs, accuracy.rs, r1c.rs, probe_milestones.rs,
  milestone_exit.rs, report.rs, attribution.rs, phase1_exceptions.rs; scripts/phase1_exit.py.
- Worked example: report.json of main b84a7256, with the tier counts in decision 0355's Context.

## Acceptance criteria
- [x] `envelope()` takes its Excel side at `ENVELOPE_DIGITS` (12), the same test `cell_envelope` uses; both are in
  accuracy/envelope.rs since T-5091. `Envelope::Excel15` is renamed, with every reader of accuracy.json updated.
- [x] R1(c) counts `Outside` and `WrongType` cells only (SPEC-0037 design 2). The milestone rows apply the tier
  bars (design 4).
- [x] The accuracy check runs at 12 digits. Its stale known-failure lines are removed from the check's own output.
- [x] report.json has `usage_gates` (design 3) and report.md has its "Usage tiers" section.
- [x] `scripts/phase1_exit.py` prints `foundational` and `expert` in place of `gate A` and `function rate`, with
  the old two as `info`. scripts/test_phase1_exit.py covers this.
- [ ] Unit tests per SPEC-0037's test plan (done). A full `make eval` (not run: the owner asked not to re-run
  everything; the worked example below stands in for it). A full `make eval` shows the tier numbers in the Outcome.
- [ ] `make check` green; reviewers approve (the `make check-fast` gates are green locally, apart from
  `py-bindings`, which this change does not touch; review and CI are the PR's).

## Out of scope
- The CI ratchet (T-5093), the dashboard (T-5094) and task ordering (T-5095).

## Outcome

What changed (SPEC-0037 designs 2 to 5):
- **Envelope** (`accuracy/envelope.rs`). `envelope()` reads Excel's side through `within_excel`, the one
  Excel-side test it shares with `cell_envelope`: a pass, a deviation, 4 ulp (decision 0088, so an Excel 0
  against a subnormal stays inside) or `within_significant_digits(.., ENVELOPE_DIGITS)` (12). This replaces
  `Visible::is_off_at_fifteen_digits`, which had no other caller and is gone. `Envelope::Excel15` is now
  `Envelope::Excel` (serialized `excel`). No reader in tools/ or scripts/ parsed `excel15`; the check's summary
  line now reads "Excel 12 digits only".
- **Accuracy check at 12 digits.** `rustsheet-eval accuracy` reported exactly 4 known-failure lines as stale:
  YIELD s1 A230 and A261 and YIELD r4-T-2788 A252 (T-4758), and ODDFYIELD s1 A289 (T-4551). Those 4 lines are
  removed, so the file goes from 183 to 179 lines. The check now passes: 45,083 points, 44,904 inside, 179
  known failures, 0 failing.
- **R1(c)** (`r1c.rs`). `classify` counts a cell only when `cell_envelope` says `Outside` or `WrongType`. A
  cell `cell_envelope` puts inside is `Phase1b`. The "Excel more than ε off the math" clause is gone.
  `R1cCells::miss` says how a counted cell misses (a cell the run did not classify reads as a wrong type).
- **Tier gates** (`usage_gates.rs`, built in `score_milestones` right after `v15_checkpoint`). These are the
  report.json `usage_gates` and the report.md "Usage tiers" section. They are built on T-5093's
  `gate_tiers::function_cells` and `gate_tiers::tiers`, merged in from origin/agent/harness/T-5093-envelope-ratchet
  (7352742554b), so there is one tier computation.
  - Attribution is the sole non-glue call only, with no decision 0118 evidence-floor fallback, so no cell is
    counted twice in a pooled count.
  - GETPIVOTDATA is left out (decision 0354), so expert counts 383 functions.
  - Owner rulings (2026-11-08) applied to the merged gate_tiers:
    - a wrong type is a type miss only, so the envelope bars count `outside` alone (`Cells::meets`);
    - `failing_functions` lists the functions below a per-function bar;
    - a new key, `contributors` per tier, lists the functions with any miss, most first, top 20 (noted in
      SPEC-0037 design 3).
  - Other small changes to gate_tiers: `Cells` is serde and has `miss`, `PIVOT_FUNCTION` is pub, and
    `meets` uses a saturating subtraction.
- **Milestone rows** (design 4).
  - Probe rows (`MilestoneRow::tiers`) and ironcalc-calc rows (`MilestoneScope::tiers`, plus
    `FunctionScope::failing_usage`) count their non-deviation cells by tier. A cell's tier comes from
    `usage_gates::cell_usage`: the tier of its sole non-glue call, else foundational when it calls a foundational
    function or no catalog function. The misses are R1(c)'s.
  - `milestone_exit` applies the pooled tier bars to both rows (new `probe_tiers` and `calc_tiers`). M9's old
    0.9995 allowance is still shown, as information only.
  - The M10 text rules and the M7 spill-extent rules are unchanged.
- **`scripts/phase1_exit.py`**.
  - New `foundational` and `expert` lines: `usage_gates` plus the accuracy.json points, each point under its
    table's function, against the tier bars. Each line names the failing functions, and the pooled-miss
    contributors when the tier is not met.
  - `gate A` (now the 12-digit check's `holds`) and `function rate` (Gate B) are printed after them with
    verdict `info`; they never fail the exit.
  - The JSON shape is unchanged: these are ordinary `lines[]` entries.
- Decision 0355 items 3 and 4 now say where all of this is computed (governance ok).

Worked example. These tier misses come from the b84a7256 report's failing cells; under sole-call attribution
they are exact, but the cell totals need a fresh run:
- foundational: 29 wrong type and 1 outside, from TEXT, EDATE and NPV;
- expert: 1,356 wrong type and 393 outside, against decision 0355's 31/1 and 1,355/405 under its first count;
- an earlier count of mine, with the report's evidence-floor fallback, found exactly decision 0355's 12 expert
  functions under 99%: IRR, RATE, BETA.INV, T.TEST, F.INV, F.INV.RT, FINV, CHIINV, F.DIST.RT, NORMINV, ACCRINT
  and MROUND. Under sole-call attribution the per-function denominators need a fresh run;
- the gated vendored suites (`make eval-gates`, T-5093's lock line):
  - foundational: 142,228 cells, 29 wrong type, 1 outside, unmet (TEXT, EDATE);
  - expert: 276,969 cells, 799 wrong type, 126 outside, unmet (IRR, RATE, BETA.INV, T.TEST, F.INV.RT, ACCRINT,
    MROUND, XIRR, F.INV).

The 4-ulp rule (owner ruling): `within_excel`, the Excel side of `cell_envelope` and `envelope`, counts a
`NumericUlp` cell as inside. SPEC-0037 design 2 and the envelope.rs doc comment say so, and a unit test checks
0 against a subnormal within 4 ulp (inside) and 0 against 1e-300 and 1e-200 (outside). It moves one baseline
cell into the envelope: gen-m9/899f9555e67da2d6.case.json Calc!A386,
`=LOGINV(1E-20,WEEKNUM(61,1),(Data!$D3+BITAND(765,337)))`, Excel 0 against the engine's 1e-323. Its `E` line
was removed by `make eval-baseline`, not by hand; it stays an `F` line.

Which numbers move:
- R1(c) now leaves out every visibly-off number that is within 12 digits of Excel or on the math, so the probe
  and calc rows' `r1c_off` falls and their `phase1b` rises.
- The every-suite Phase 1 rate (`v15_checkpoint` `rate_phase1b`, decision 0227) leaves out more cells.
- Gate B is unchanged.

Tests:
- `cargo test -p rustsheet_eval`: 499 unit tests plus 4 integration tests. They cover:
  - envelope: the 12-digit edge on both sides, 4 ulp from 0, R3, errors;
  - R1(c) classes and `miss`;
  - `usage_gates`: exact 99% and 99.9%, contributors, wrong type as a type miss only, `cell_usage`;
  - row tiers in probe_milestones and function_scope;
  - milestone_exit tier bars, including one foundational wrong type failing an M9 calc row.
- `python3 -m unittest scripts/test_phase1_exit.py`: 24 tests.
- `make fmt-check hygiene typos clippy py-check` green. `make eval-gates` was green except for "baseline out
  of date" on the gen-m9 cell above, which `make eval-baseline` recorded; `py-bindings` not run (bindings
  untouched).
- `make eval` not run, as the owner asked.

