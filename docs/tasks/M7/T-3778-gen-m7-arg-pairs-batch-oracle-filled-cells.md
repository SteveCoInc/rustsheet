---
id: T-3778
title: "gen-m7 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of GROUPBY and PIVOTBY"
milestone: M7
lane: functions-lookup
status: blocked
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
gen-m7 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise
argument coverage of GROUPBY and PIVOTBY, and every Excel cell the engine fails is owned by a task.

## Context
- Gap triage 2026-11-01 (`make eval` on main 65c266573; report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). Neither function has an unowned visibly-off cell
  (`make unowned-off ALL=1`), and both are above the evidence floor. Yet nearly all of their argument-class pairs
  (`crates/rustsheet-eval/src/argument_coverage.rs`) have no Excel cell, so their behaviour on those pairs is
  unverified however many cells pass.
- Pairwise share and unseen pairs at 65c266573:

  | function | pairwise share | unseen pairs | first unseen pairs |
  |---|---|---|---|
  | GROUPBY | 0.11 | 2256 | row_fields/values = (number, zero), (number, negative) |
  | PIVOTBY | 0.09 | 4964 | row_fields/col_fields = (number, zero), (number, negative) |

  That is 7220 unseen pairs, the largest M7 gap after RANDARRAY (volatile, out of scope here).
- No task covers these pairs. T-3497 (done) ran the batch for the M7 array-shaping functions and T-3683 (todo) for
  LET, MAKEARRAY, REDUCE and SCAN; both list GROUPBY and PIVOTBY as out of scope ("pairs mostly involve LAMBDA
  arguments, lookup lane"). The open GROUPBY/PIVOTBY tasks are single-kind or single-shape probes and do not overlap:
  T-2875 (option argument kinds), T-3109 (options as OFFSET/CHOOSE/IF references), T-2644 (eta functions whose
  minimum is 0), T-3112 (near-zero snap of aggregation LAMBDAs), T-2686 (operator over a whole GROUPBY result),
  T-2993 (TTEST over PIVOTBY slices). Give new cells of their shapes to them.
- Dry run on 65c266573 into a scratch suite that was then deleted:
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M7 --functions GROUPBY,PIVOTBY --spill` wrote
  73 cases (about 14,400 calls, 200 per case, no pair skipped). Examples:
  - `=_xlfn.PIVOTBY(Data!C1:E12,Data!A1:A12,0,LAMBDA(w,w),0,2,#NULL!)`
  - `=_xlfn.PIVOTBY(TRUE,Data!C1:C12,"abc",LAMBDA(x,x))`
  - `=_xlfn.PIVOTBY(Data!B1:B12,Data!B1:B12,Data!H2,_xleta.MIN,1,-2,Data!G3)`
- Why opus: the `function` argument mixes eta-reduced names and LAMBDAs that ignore their parameter or return a
  reference; both functions spill tables whose size depends on the data, so a draw can outgrow the case layout's
  spill stride. Telling a generator artefact (an unloadable formula, a layout `#SPILL!`, a directive-5 size limit)
  from a real Excel rule takes judgment before cells are owned. The batch is also large: consider generating it in
  two PRs (GROUPBY, then PIVOTBY) if oracle fill or eval time needs it, each meeting the criteria below.
- Steps: run `make eval` so the policy reads current unseen pairs; generate with `--spill`; fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m7`; score.
- M7's exit needs gen-m7 at 99.9% or more, with 0 spill-extent failures, at visible parity (docs/milestones.md,
  decision 0140), and the Phase 1 rate counts gen-* cells (decision 0188). Own each failing cell in the same PR:
  an existing open task that covers the rule, or a new task via `make new-task` (lane functions-lookup for
  `functions/lookup_and_reference/{groupby,pivotby}.rs`; engine for LAMBDA binding or evaluation). A wrong type,
  error or text gets `campaign: semantics` and a `## Visible-parity stage` section; name the case file stem and the
  A1 cells in one bullet (decision 0159). Record any new Excel quirk in docs/excel-semantics/ (arrays.md), citing
  the cells.

## Acceptance criteria
- [x] New gen-m7 cases from `--policy arg-pairs --functions GROUPBY,PIVOTBY --spill` are committed with their
      excel-graph oracle sidecars and 0 fill failures.
- [x] After `make eval`, GROUPBY and PIVOTBY each have a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`, or the PR lists the pairs left unseen and why (a pair Excel refuses to load, a layout
      limit).
- [x] No scored cell of the batch is a `#SPILL!` caused by the case layout; spill-extent failures in the M7 exit row
      stay at 0.
- [x] Failing cells are known failures in eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions), each owned
      by an open task, and `make exit-owners MILESTONE=M7` and `make unowned-off ALL=1` report no new unowned cell
      from the batch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the cells this batch finds (the owning tasks make them), beyond a one-line coercion fix in the
  function's own module that the PR also tests.
- RANDARRAY (volatile), TEXTSPLIT and REGEXEXTRACT pairs, the single-kind probes named above, and other functions or
  milestones.

## Outcome
- **Batch**: 73 new gen-m7 cases from `rustsheet-gen --policy arg-pairs --milestone M7 --functions GROUPBY,PIVOTBY --spill`
  (default seed; 23 GROUPBY parts, 50 PIVOTBY parts, 14,440 calls, no pair skipped), filled by
  `rustsheet-oracle fill -b excel-graph --suite gen-m7`: 73 sidecars, 0 fill failures. One PR, not two: the fill took 11 minutes.
- **Coverage**: after `make eval`, GROUPBY and PIVOTBY each have a pairwise share of 1.0 with 0 unseen pairs (before:
  0.11 with 2256 unseen, 0.09 with 4964).
- **Layout**: no Excel value in the batch is `#SPILL!` (every oracle sidecar checked), so no failure comes from the case
  layout; the 61-row spill stride holds every GROUPBY and PIVOTBY result.
- **Eval delta**: gen-m7 grows from 106 to 179 cases and from 100% (54,449 cells, 1 known failure) to 98.81% (87,701
  cells, 1,047 scored known failures, 0 regressions; `make eval-baseline`). The M7 row's visibly-off and spill-extent
  counts rise by the batch's 13,381 visibly-off cells (`extent_failures` 12,339), all engine rules below, not layout:
  12,173 of them are the spilled tables we return for calls with a one-value filter_array that Excel answers with a single `#VALUE!` (T-3844).
  The visible-parity lock grows by the same 13,381 cells in 72 new cases. gen-m7 is below its M7 exit bar until the owners land.
- **Owners** (`make exit-owners MILESTONE=M7`: 0 unowned; `make unowned-off ALL=1`: no gen-m7 cell): five tasks, each rule
  recorded in docs/excel-semantics/arrays.md, "GROUPBY and PIVOTBY: what the T-3778 arg-pairs batch found":
  - **T-3842** (sonnet, 359 cells in 314 formulas): a one-cell error reference as row_fields or col_fields is `#VALUE!` once values and options pass; its option-position cells follow T-3846's argument order.
  - **T-3843** (sonnet, 292 cells in 82 formulas): a one-cell text or blank `values` is `#VALUE!`.
  - **T-3844** (opus, 12,173 cells in 43 formulas): when a one-value filter_array is `#VALUE!` rather than broadcast is unknown (two batch cells broadcast); probe first.
  - **T-3845** (sonnet, 493 cells in 166 formulas): a multi-cell PIVOTBY relative_to lifts into one result per cell.
  - **T-3846** (opus, 68 cells in 21 formulas): argument-check order and leftovers.

## Blocked because
Blocked because: review-round limit
- PR #2852 (draft). Head 81d7ba5 + this commit. CI green on 33326f9; `make check-fast` green on every round's head.
- Rounds 1 to 3: design and security APPROVE each round. Excel fidelity REQUEST_CHANGES each round, always on the
  docs (arrays.md section and T-3842..T-3846 wording); the data, baseline, coverage and ownership pass every check
  (eval-check 0 regressions, pairwise 1.0, no `#SPILL!`/`#CALC!`, every new F cell owned once).
- Open findings at round 3 (verdict record on the PR, the excel-fidelity records):
  - [MAJOR] The argument-precedence rule in arrays.md bullet 1 and the T-3842 Goal ("an earlier invalid option's
    `#VALUE!` beats a later error") is too general. Depth and field_headers options: text "x"/"" is checked first
    (cf6ab0ac7d65a5bb A2014, 65a4611a6242a59f A10920, f310ddb6bc8ec33a A8968, 42c026cbec22a61f A6650); numeric text and
    blank are coerced, so the later error wins (eb01a9867300331b A8602, 65a4611a6242a59f A2380, 07c0c130d894cfe0 A2197).
    Sort orders and filter_array: text, empty text and numeric text lose to a later error (07c0c130d894cfe0 A11164,
    A11103; 7ab51243a366f57b A306, A11774; 0dac515958ce4371 A4820, A6162, A4759; 547d6192794dbf57 A550;
    9c128bd4a5d1d3b9 A11042; 709913a7469ca178 A672, A2014), but blank wins (53473f95ff58029d A4088, 0dac515958ce4371
    A9944, 7ab307dba9cb2062 A9883, A9944, 9077d156f68bf703 A11225, A11286, cb6c98a18513801b A12079, A12140,
    5883a41f98113db2 A6406), and so does an out-of-range numeric sort_order (88f592c463d2ffb0 A7138, 547d6192794dbf57
    A4576). Restate the precedence by position and kind, or drop it to "unknown, see T-3846", in arrays.md, T-3842 and T-3846.
  - [MINOR] arrays.md bullet 4 and T-3845: Data!A1:B2 holds valid relative_to cells (blank and 0 are valid:
    410128dc65d03200 A7809, febfd6901374d79c A4637, ceb2c91035864a71 A5491); drop "no valid relative_to", say a
    valid cell's lifted result is unprobed, add a probe criterion to T-3845.
  - [MINOR] arrays.md bullet 2 and T-3843: the one-cell number/boolean values table holds only with one-row row_fields
    (c636506a3a65eb5d A1038, A1099 are `#VALUE!`).
  - [MINOR] The span lists of T-3842, T-3845 and T-3846 also cover 180 passing cells (T-3845's 2x2 spans include the
    passing anchor); counts and spans disagree. State the counts as span sizes or list only failing cells.
- Do not repeat: stating a rule from bucket counts. Search all 14,440 anchors for counterexamples before writing any
  precedence claim; the reviewer's scratch scripts were lib.py and own.py (not committed).
