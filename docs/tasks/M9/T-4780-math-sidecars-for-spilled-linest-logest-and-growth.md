---
id: T-4780
title: "Math sidecars for spilled LINEST, LOGEST and GROWTH results: 98 counted cells of the linest, logest and growth modes probes with no math value"
milestone: M9
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A cell holding a spilled LINEST, LOGEST, TREND or GROWTH result (the anchor `=LINEST(B2:B9,A2:E9,TRUE,TRUE)` and the
cells it spills into) gets a math value from the existing exact array-regression definitions (T-4096, decision 0271).
This includes calls over ranges of constant cells, not only array literals. Then the 98 visibly-off cells of
probes/linest-modes, logest-modes and growth-modes are sorted by decision 0188 R1(c): Phase 1b evidence, a `campaign:`
cell for T-2690, or a stated skip. Today they count only because they have no math value (decision 0325), and
T-2690's visible-stage fit cannot start on them before the math is attached.

## Context
- Gap triage 2026-11-07 (`make eval` at b9ae5216f, excel-graph). None of the three cases has a `.math.json`.
  `eval_math.candidates` gives their anchors `an array result outside INDEX` (36 for linest-modes). The spilled cells
  have no formula text, so they are not candidates at all. The definitions (`defs/array_regression.py`) are `indexed`:
  they read one element `=INDEX(LINEST({..},{..},TRUE,TRUE),row,col)`, and known_y and known_x must be array literals.
- What has to change, and why it is a design task (opus) rather than a mechanical one:
  - the sidecar must key each spilled cell to its anchor's call and its (row, column) offset. The Rust side attaches a
    math value "only while `f` still matches the cell's formula" (eval_math module doc), and a spilled cell has no
    formula of its own, so the match rule for spill cells needs a decision: the anchor's formula plus the offset, kept
    in step with the spill extent the oracle recorded;
  - range arguments of constant cells must be read as the literal arrays the definitions take (blank or text cells:
    Excel's LINEST is `#VALUE!` there, so no math value);
  - the definitions decline rank-deficient and near-exact fits (Excel drops a collinear column, and its residual is
    rounding: T-2569; statistics.md "Near-exact fits are not snapped"). linest-modes is the T-1621 probe of exactly
    those modes ("const/stats flags, multi-column x, collinear, errors"), so part of the 98 will stay skipped. The
    Outcome must list them with that reason, so they are not mistaken for cells still waiting for math.
- Code: `tools/src/rustsheet_tools/accuracy/eval_math.py` (`candidates`, `indexed_call`), `defs/array_regression.py`,
  `fingerprint.py`; crates/rustsheet-eval's math attachment (`accuracy` module, the `f` match).
- Owners of the cells' fix work, which keep them: T-2690 (LINEST, LOGEST and GROWTH to visible parity) and T-2006 (the
  least-squares kernel), and T-2569 for the collinear linest-modes cells.

Cells this task owns for their math value:
- probes/growth-modes.case.json (10): Sheet1!K145, Sheet1!K146, Sheet1!K147, Sheet1!K148, Sheet1!K149, Sheet1!K150, Sheet1!K151, Sheet1!K152, Sheet1!K157, Sheet1!K169
- probes/linest-modes.case.json (39): Sheet1!K51, Sheet1!K53, Sheet1!K106, Sheet1!K107, Sheet1!K134, Sheet1!K191, Sheet1!K198, Sheet1!L1, Sheet1!L8, Sheet1!L15, Sheet1!L22, Sheet1!L29, Sheet1!L36, Sheet1!L52, Sheet1!L54, Sheet1!L64, Sheet1!L71, Sheet1!L78, Sheet1!L99, Sheet1!L107, Sheet1!L108, Sheet1!L110, Sheet1!L134, Sheet1!L138, Sheet1!L148, Sheet1!L169, Sheet1!L190, Sheet1!L191, Sheet1!L192, Sheet1!L194, Sheet1!L197, Sheet1!L198, Sheet1!L199, Sheet1!L201, Sheet1!M106, Sheet1!M107, Sheet1!N107, Sheet1!P106, Sheet1!P107
- probes/logest-modes.case.json (49): Sheet1!K9, Sheet1!K30, Sheet1!K72, Sheet1!K79, Sheet1!K85, Sheet1!K86, Sheet1!K88, Sheet1!K93, Sheet1!K95, Sheet1!K135, Sheet1!K151, Sheet1!K170, Sheet1!K198, Sheet1!L10, Sheet1!L12, Sheet1!L31, Sheet1!L33, Sheet1!L54, Sheet1!L73, Sheet1!L75, Sheet1!L80, Sheet1!L82, Sheet1!L85, Sheet1!L86, Sheet1!L87, Sheet1!L89, Sheet1!L92, Sheet1!L93, Sheet1!L94, Sheet1!L96, Sheet1!L115, Sheet1!L117, Sheet1!L136, Sheet1!L138, Sheet1!L152, Sheet1!L155, Sheet1!L159, Sheet1!L170, Sheet1!L171, Sheet1!L173, Sheet1!L198, Sheet1!L199, Sheet1!L201, Sheet1!M85, Sheet1!M86, Sheet1!M92, Sheet1!M93, Sheet1!N85, Sheet1!N86

ironcalc-calc/STATISTICAL/LINEST_LOGEST.xlsx (5 counted cells) is an xlsx case, and tools never parse xlsx values, so it
is out of reach here.

## Acceptance criteria
- [ ] A short decision record or SPEC-0029 amendment states how a spilled cell's math is keyed (anchor formula, offset)
      and when the runner attaches it.
- [ ] `cd tools && uv run rustsheet-accuracy math --suite probes` writes math sidecars for the three cases. Every cell
      above has a value or a named skip (rank-deficient, near-exact residual, `#N/A` stats cell, non-number range
      element). `rustsheet-accuracy math --check` passes, and `rustsheet-accuracy verify LINEST`, `LOGEST`, `TREND` and
      `GROWTH` show no change.
- [ ] After `make eval`, the Outcome gives the counts per case: valued, Phase 1b evidence, still counting (with
      Excel-minus-math), and skipped by reason. The M9 probe row's R1(c) count drops by the Phase 1b ones.
      `eval/baseline/` gains no new line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change to LINEST, LOGEST, TREND or GROWTH (T-2690, T-2006, T-2569).
- Spilled results of other array functions; file a follow-up if the keying rule generalizes cheaply.
