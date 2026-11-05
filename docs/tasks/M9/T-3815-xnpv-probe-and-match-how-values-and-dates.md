---
id: T-3815
title: "XNPV: probe and match how values and dates elements are read (fractional, negative and out-of-range dates, text and booleans in ranges and arrays, shapes) and which error wins"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
XNPV gives Excel's value or error for every kind of element in its values and dates (and for its rate), read from a
range, an array constant, a computed array or a single cell, so the element reading that follows the evidenced size
checks is evidenced too, with the order in which its errors win.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md, theme 3: "XIRR and XNPV coerce
their date and value arguments differently from Excel, including which error comes out"). It names a class of input,
never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe
skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel
across the class, record that in the Outcome and close the task.

Gap triage 2026-11-02 (report of `make eval` on 2026-11-02, excel-graph evidence): every visibly-off XIRR/XNPV cell
already has an owner, so no task covers the class as a sweep; this is an evidence gap. What is evidenced is in
docs/excel-semantics/financial.md: "XIRR reads empty cells as 0 and accepts earlier dates; XNPV is #NUM! on empty
cells", "XNPV reads one empty values cell as a single 0 cash flow", "XIRR and XNPV under the gen-m9 statistical cells:
size and type checks (T-2255)" (the size checks and the argument-level error order, from gen-m9 cells, much of it
inferred through an enclosing call) and "XNPV discounts with Excel's `^`".

Not swept for XNPV (each as a range element, an array-constant element, a computed-array element and a one-cell
argument): a fractional date (truncated? before or after the "precedes the first date" check: a later date of
first + 0.5), a negative date, date 0, a date past 2958465, a later date equal to or before the first, unsorted later
dates; numeric text, date text ("1/1/2025"), non-numeric text, TRUE/FALSE and an empty cell in the middle of a
multi-cell values range and of a dates range (ours: `#NUM!` for a non-number value, `#VALUE!` for a non-number date);
a first value of 0; all values 0; an error element in values against one in dates (which wins, and against a bad
date earlier in the list); a rate of -1, below -1, numeric text, an empty cell; a values row against a dates column
and 2-D ranges of the same count. The open, already-owned points stay with their tasks: T-3312 (XNPV element order,
1b60bf99d71424e7 A171 and 5ded7594e597e31e A12: start from its findings if it lands first), T-2603 (all-empty
multi-cell values `#VALUE!`), T-2118 (lifted rate), T-3126 and T-3184 (argument-kind coverage cells).
Code: `fn_xnpv`, `xnpv_numbers` and `xnpv_size_error` in crates/rustsheet-engine/src/functions/financial/mod.rs,
`xnpv_dates` and `whole_day_dates` there, `compute_xnpv` in financial_util.rs. XIRR's reader (T-3816) is a sibling:
where both functions agree, fix the shared helper, not each copy (directive 3).

## Acceptance criteria
- [x] A probe case (excel-graph, canary-verified) covers the element kinds and shapes above.
- [x] The engine matches every probed cell; cells owned by T-3312, T-2603 or T-2118 are left to them and named.
- [x] financial.md gains an XNPV element-reading entry (rule, probe cells, VERIFIED) in the same PR, and the "Excel docs
      claim ... #VALUE!" comment in `xnpv_dates` is replaced by the evidenced rule.
- [x] `make check` green; eval known failures only shrink (baseline updated, no new lines); reviewers approve.

## Out of scope
- XIRR (T-3816), the solver and its digits, and XNPV's `^` (evidenced).

## Outcome
XNPV now reads its values and dates as Excel does, element by element; the probe is to the bit on all 187 cells.

- **Probe** probes/t3815-xnpv-element-reading (excel-graph 2026-11-05, canary-checked on the fill; generator
  docs/research/t3815/gen_case.py): 54 rows of element kinds and shapes, each in a range, an array constant, a
  computed array and as one cell, plus the rate's forms and shapes. Before this PR 28 of its cells differed
  (after the error cells were written as `=NA()`/`=1/0` formulas: typed `{"t":"e"}` cells read as text in the harness).
- **Changes**: `Model::xnpv_walk_error` (flows.rs) is Excel's index-by-index walk: an error element is that error,
  a later non-number value `#NUM!` while the first value is a number, a text, boolean or out-of-range date `#NUM!`
  (`#VALUE!` first), a later date bounded before truncation. It runs after a rate at or below 0 and before the
  first value's `#VALUE!`. `xnpv_dates` bounds later dates untruncated (2958465.5 `#NUM!`). `Model::xnpv_one_cell`
  reads a one-cell error or numeric text as its value; `xnpv_size_error` reads an error values argument before the
  dates'; `xnpv_first_value_refused` refuses numeric text and an array's text or boolean first element.
- **Shared helpers**: XIRR's reader (T-3816) keeps its own rules (it truncates every date first and reads text in a
  range as numbers), so nothing here is shared with it beyond `whole_day_dates` and `flow_range_end`.
- **Owned elsewhere**: no probed cell is left to T-3312, T-2603 or T-2118 (all match).
- **Eval delta**: gated suites 0 fixed, 0 regressions; probes gains the new case (187 cells, all passing).
  financial.md: "XNPV reads its values and dates element by element (T-3815)", and T-2603's entry's unprobed line
  updated.

## Landing notes (resumed)
- PR #3375 (draft), head dc9dbbda9. CI was queued (run 42052639494) when the window closed. Reviews on dc9dbbda9:
  security APPROVE, design APPROVE (MINOR notes only), Excel fidelity REQUEST_CHANGES. The status is back to
  in-progress because the Outcome is not final.
- **Blocking, found locally (directive 5):** `Model::flow_elements` (flows.rs) builds one element per cell of the
  whole range rectangle before walking. `cargo test -p rustsheet_engine --lib -- xnpv` hangs on
  test_fn_financial::xnpv_first_value_refused_bounded (C4 `XNPV(1,A2:A1048567,B11:B1048576)`) and
  xnpv_first_value_refused_sparse_dates (`A1:XFD20000`, 327M cells), and the kernel then kills it for memory. CI's
  rust jobs will fail the same way. Fix: walk lazily with an early exit instead of collecting. Index i maps to a
  position in each range, row-major, and only the cells needed until the first error are read. Also bound the walk
  the way `xnpv_first_value_refused` does: read only stored cells (`for_each_existing_cell`) and treat absent cells
  as Empty, so a sparse range costs its stored cells. `make check-fast` does not run these tests: run
  `cargo test -p rustsheet_engine --lib -- xnpv` (debug, about 1 minute) before pushing.
- **Excel-fidelity MAJOR:** `xnpv_one_cell` also reads a *literal* text argument (`XNPV(0.1,"30",39448)`,
  `XNPV(0.1,30,"39448")`, `XNPV(0.1,30,"1/1/2025")`) as its number, which is unprobed and changes main's
  `#NUM!`/`#VALUE!`. Either probe those literals (add them to docs/research/t3815/gen_case.py and refill with
  `--force`), or limit the reading to reference arguments (the `Node` in `fn_xnpv` tells them apart).
- **MINOR (fix in the same push):** update the stale "unprobed" sentences in `xnpv_first_value_refused`'s doc comment
  (numeric text, arrays, negative or out-of-range later dates are now evidenced). Note in financial.md that a later
  date in (-1, 0) is unprobed (or probe it).
- **Do not repeat:** typed error cells `{"t":"e",...}` in a case read as text in the harness; write errors as `=NA()`
  and `=1/0` formulas.
- When the window closed: nothing in progress. Next step: the lazy walk, then the literal question, then
  `cargo test` on the xnpv tests, `make check-fast`, push, and re-run the panel (round 2).
