---
id: T-3810
title: "gen-m7 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the spilling text functions TEXTSPLIT and REGEXEXTRACT"
milestone: M7
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
gen-m7 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` that raises the pairwise argument
coverage of TEXTSPLIT and REGEXEXTRACT, spill members included. Every Excel cell the engine fails is owned by a
task.

## Context
- Gap triage 2026-11-02 (`make eval` on main 2290b8769, report.json `argument_coverage`; decision 0031 evidence
  gaps). Every failing Excel cell of these functions has an open owner (`make unowned-off ALL=1` lists none), and both
  are above the evidence floor. Pairwise share (unseen pairs): TEXTSPLIT 0.217 (1421, six parameters; pad_with has
  only 7 of 11 classes, and row_delimiter, ignore_empty and match_mode 9) and REGEXEXTRACT 0.230 (559;
  case_sensitivity never empty text). Cells and passes (report.json `functions`): TEXTSPLIT 1007/1008, REGEXEXTRACT
  886/890.
- These two are the M7 text functions the existing batches left out because their results spill: T-3513 (TEXTBEFORE,
  TEXTAFTER: "TEXTSPLIT is left out here because its result spills") and T-3514 (REGEXTEST, REGEXREPLACE:
  "REGEXEXTRACT (its return_mode 1 and 2 results spill)"). T-3497 (done, opus) ran the same policy over the
  spilling array-shaping functions and gave `dynamic_array` functions the 61-row `SPILL_ROW_STRIDE` and no `check`
  list, so every spill member Excel records is scored (tools/src/rustsheet_tools/gen/arg_pairs.py). Both functions
  are `dynamic_array: true` in eval/catalog/functions.yaml. TEXTSPLIT spills across columns as well as rows, so check
  that no result reaches another formula's cells. Widen the layout in the generator, with a unit test, if one can.
- T-2876 and T-2877 (research, todo) probe single argument kinds of these functions. This batch covers the pairs.
  The forced classes are plain values (`2`, `"abc"`, `TRUE`, ...), so REGEXEXTRACT's patterns are literal strings:
  a failing cell is an argument-coercion, error-order or spill-shape rule, not a regex-engine one (as T-3514 says).
- Run `make eval` first so the policy reads current unseen pairs, then
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M7 --functions TEXTSPLIT,REGEXEXTRACT`.
  Fill with `uv run rustsheet-oracle fill -b excel-graph --suite gen-m7` and score.
- M7's exit counts the gen-m7 row and its spill extents (docs/milestones.md, decision 0140), so a failing cell here
  matters. Own each one in the same PR: an existing open task that covers the rule, or a new task via
  `make new-task` in this lane with `campaign: semantics` and a `## Visible-parity stage` section for a wrong type,
  error or text. Put the case file stem and A1 names in one bullet (decision 0159), and record any new Excel quirk
  in docs/excel-semantics/text.md or arrays.md, citing the cells. Code: functions/text/textsplit.rs
  (`fn_textsplit`) and functions/text/regex_functions.rs (`fn_regexextract`).
- Tier: opus. Every result can spill, in two directions for TEXTSPLIT, and the policy's layout has only been
  checked for column-shaped spills (T-3497), so the run may need generator judgment before its cells can be trusted.

## Acceptance criteria
- [x] New gen-m7 cases from `--policy arg-pairs --milestone M7 --functions TEXTSPLIT,REGEXEXTRACT` with their
      excel-graph oracle sidecars are committed, and no scored cell is a `#SPILL!` caused by the layout.
- [x] After `make eval`, both functions have a `pairwise` share of at least 0.9 in report.json `argument_coverage`,
      or the PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to load).
- [x] Failing cells are known failures in eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions), each owned
      by an open task, and `make exit-owners MILESTONE=M7` and `make unowned-off ALL=1` report no unowned cell from
      the batch.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine for the cells this batch finds (the owning tasks), beyond a one-line coercion fix in the
  function's own module that the PR also tests.
- Regex-syntax probes (T-1972, T-2356, T-2478) and the single-kind probes T-2876 and T-2877.

## Outcome
- Added 21 gen-m7 cases from `rustsheet-gen --policy arg-pairs --milestone M7 --functions TEXTSPLIT,REGEXEXTRACT`
  (seed 0; TEXTSPLIT 15 parts over 1421 unseen pairs, REGEXEXTRACT 6 parts over 559), with excel-graph sidecars
  (2026-11-02). Layout: every formula sits in Calc column A at the 61-row spill stride, with no `check` list, so every
  spill member is scored. Only column A holds formulas, so a TEXTSPLIT spill across columns reaches no other formula.
  No scored cell is a layout `#SPILL!`.
- Coverage (report.json `argument_coverage`): TEXTSPLIT pairwise 0.217 → 1.0, REGEXEXTRACT 0.230 → 1.0, no unseen pair left.
- Triage: 5729 scored cells; 6 failed before the fix, all one REGEXEXTRACT error-order rule (gen-m7/3d156a08a7bacf35
  Calc!A8236, A9273, A9334, A11408; gen-m7/781b74b7c112265f Calc!A6833:A6834). Excel coerces return_mode in its turn,
  but checks its range only after case_sensitivity is read, so an out-of-range return_mode loses to an error
  case_sensitivity, while non-number text is still `#VALUE!` first (B6833). Fixed in `fn_regexextract`
  (functions/text/regex_functions.rs; `regex_option_in_range` split out of `regex_option`) with a unit test
  (`test_regexextract_case_error_beats_out_of_range_return_mode`). text.md "REGEX* optional arguments" records it.
- Eval delta: gen-m7 +21 cases (5729 cells), all pass; 0 regressions, 0 new known failures, gen-m7 known failures 2538 → 2538.
