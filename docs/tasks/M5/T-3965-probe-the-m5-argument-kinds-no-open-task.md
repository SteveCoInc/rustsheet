---
id: T-3965
title: "Probe the M5 argument kinds no open task lists: an error in ISREF's value and LENB's text"
milestone: M5
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
ISREF's `value` and LENB's `text` each have an Excel-backed, passing cell for the one argument kind the report's
argument coverage still misses (an error), so both reach `covered == expected`.

## Context
- Gap triage 2026-11-03 (opus, after T-3880; `make eval` on 786fc8b2, excel-graph evidence; report.json
  `argument_coverage`, decision 0031 evidence gaps; decision 0094 wants every argument mode covered). Every failing
  cell of that run has an open owner (`scripts/exit_owners.py --all-off`: only cells owned by `defer:` tasks are left,
  which decision 0245 accepts), and the evidence floor is met. That leaves the argument kinds as the open evidence
  gap. These two M5 parameters are missing a kind and no open task names them. T-2862, T-2863, T-2871 and T-2872
  cover the other M5 gaps.
- Coverage on that run (the kinds of `crates/rustsheet-eval/src/argument_coverage.rs`, `classify_value`):
  - ISREF.value (mode `array`): 10 of 11; seen number, zero, negative, fraction, numeric text, text, empty text,
    boolean, blank, range; missing **an error**.
  - LENB.text (mode `value`): 10 of 11; same seen set; missing **an error**.
- A kind counts only when an Excel-evidence cell passes it **as a literal or a single-cell reference**. A nested call
  such as `NA()` is not classified (docs/progress.md "Argument coverage"), so write literal errors (`#N/A`, `#DIV/0!`,
  `#VALUE!`, `#REF!`) and cells holding an error (a formula cell `=1/0`, `=NA()` referenced as `A1`).
- ISREF of an error value is likely FALSE, and LENB of an error is likely that error. Neither has been probed:
  the oracle decides. A literal `#REF!` in ISREF is worth its own row, because a reference-shaped error may read
  differently from a value error.

Write one probe case, probes/m5-argument-kind-gaps-isref-lenb (excel-probe skill, excel-graph, tag M5):
- `=ISREF(#N/A)`, `=ISREF(#DIV/0!)`, `=ISREF(#REF!)`, `=ISREF(A1)` with A1 `=1/0`, and `=ISREF(A2)` with A2 `=NA()`.
- `=LENB(#N/A)`, `=LENB(#VALUE!)`, `=LENB(A1)`, `=LENB(A2)`, each also wrapped as `=ERROR.TYPE(LENB(..))` so the
  error kind is scored by number as well as by value.

## Acceptance criteria
- [ ] probes/m5-argument-kind-gaps-isref-lenb is committed with excel-graph oracle values and covers both
      parameter/kind pairs above. After `make eval`, report.json `argument_coverage` shows `covered == expected` for
      ISREF.value and LENB.text.
- [ ] Each cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's
      own module) or listed in a new functions-* task (campaign: semantics, with a `## Visible-parity stage`
      section) filed with `make new-task` that names the case and cells. The new known failures are recorded with
      `make eval-baseline` and each one has that owner (decision 0245's new-case gate).
- [ ] Any new Excel rule goes into docs/excel-semantics/information.md (ISREF) or text.md (LENB), citing the probe
      cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The M5 parameters T-2862, T-2863, T-2871 and T-2872 list; the M6 and M7 gaps (T-3966, T-3967).
