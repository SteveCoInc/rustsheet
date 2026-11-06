---
id: T-4089
title: "Gate A tables, dispersion I: STDEV, STDEV.S, STDEVA, VAR, VAR.S, VARA and DEVSQ"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
STDEV, STDEV.S, STDEVA, VAR, VAR.S, VARA and DEVSQ each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
Every failing table point has an owner.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029): decision 0271 (T-3799) classified these functions as needing a table.
- The generator's one-row array literals (T-3586) and 2-D literals (T-3686) drive the data arguments. Variadic data
  follows T-3549's SUM and AVERAGE definitions (the draw of a variable-length array and the near-zero snap rule):
  reuse it, do not copy it (directive 3).
- An Excel error region is part of the domain or a `domain` rule, never a math value. Where Excel's number is more than
  G = 1e-4 off the math, list the points in the Outcome as R3 candidates for an owner ruling and do not mark them
  (decision 0188 R3).
- Tools: as in T-3925 (`rustsheet-accuracy` table generation, oracle fill, `rustsheet-eval accuracy`).
- Sources (statistics.md): "VAR/STDEV switch between a one-pass and a two-pass formula" (the math is the exact
  sample variance; Excel's formula choice matters only where it moves a digit), "VARA family: single-cell reference
  arguments".
- STDEVA and VARA read array elements the way AVERAGEA does (statistics.md "AVERAGEA, MAXA and MINA over an array"):
  numeric draws only, since the generator has no text kind (T-4084).

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing samplers and
      kernels, each citing its sources, with unit tests against Microsoft's published examples or the semantics
      entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other sample-statistics groups of decision 0271.

## Outcome
- **Definitions:** `accuracy/defs/dispersion.py` defines all seven functions over the literals' doubles x[1..n].
  - DEVSQ is the exact sum of squared deviations from the exact mean.
  - VAR, VAR.S and VARA are that sum over n - 1. STDEV, STDEV.S and STDEVA are its square root.
  - STDEVA and VARA over numbers equal STDEV.S and VAR.S, because the sampler draws numbers only.
  - Domain: VAR and STDEV of one number is `#DIV/0!`, and a one-pass sum of squares beyond the double range is
    `#NUM!` (statistics.md).
  - The definitions reuse aggregate.py's `running_total` and `MAX_ARGS` and _common's `scalars` and
    `signed_log_uniform`.
- **Draws:**
  - Strata: typical, offset data near the one-pass/two-pass switch (relative spread 1E-9 to 0.1), edge shapes, and
    long mixed-magnitude calls.
  - Edge shapes: one number (VAR and STDEV `#DIV/0!`, DEVSQ 0), two numbers, two to six equal numbers, small
    integers, and the argument limit.
  - After review round 1, the equal numbers and the spread below 1E-6 are drawn again rather than cut.
- **DEVSQ domain:** DEVSQ has its own two-pass overflow rule (the sum, a squared deviation, or their sum), inferred
  and not probed. `DEVSQ(1E+200,1E+200)` is 0, not `#NUM!` (review round 1).
- **New quirk (syntax.md "The repeating maxima"; inputs in docs/research/t4089/xlfn-arguments.md):** an xlsx with a
  255-argument `_xlfn.VAR.S` call does not open in Excel for the web, while 254 arguments, or legacy `VAR` with 255,
  do. VAR.S and STDEV.S are therefore drawn to
  254 arguments.
- **Tables:** `eval/accuracy/tables/{VAR,VAR.S,VARA,STDEV,STDEV.S,STDEVA,DEVSQ}/s1`, 120 points each, filled from
  excel-graph.
  - `rustsheet-eval accuracy` puts all 840 points in the envelope, with 0 failing and 0 known failures.
  - Missing tables went from 135 to 128.
  - R3 candidates (Excel more than 1E-4 off the math): none. The worst relative difference between Excel and the
    math is 1.0E-14 (DEVSQ A117).
  - 4 points are within 15 digits of Excel only (accuracy's "Excel 15 digits only" went from 1333 to 1337). The
    engine matches Excel to 15 digits there but is outside the math envelope.
- **Math sidecars:** registering the definitions made two probe cases' one-call cells math candidates. Their new
  sidecars hold 207 math cells: probes-numeric/t1749-devsq-fresh (200 DEVSQ cells) and
  probes/stat-order-floor-batch (7). Both cases had no sidecar before, so no existing entry changed.
  `make eval-math-check` reports current.
- **Eval delta:** none, because no engine or eval-suite case changed.

## Landing notes (resumed)
Resumed 2026-11-06 by an opus run: the sidecars are regenerated (Outcome, "Math sidecars").

Session 2026-11-06 (opus worker): the landing window closed with this PR (#3465) not yet enqueued.
- **State:** the head before these notes is 12bf9ab8a. All three reviewers approved that head in round 2
  (security, design, excel-fidelity); their MINOR findings are filed as T-4439 (#3467).
- **CI is red on 12bf9ab8a:** job "eval (Excel-compatibility gate)", run
  https://github.com/SteveCoInc/rustsheet/actions/runs/42154393195. The failure is `make eval-math-check`: "math
  sidecars: probes, probes-numeric: stale".
  - Registering the seven new definitions turns every existing one-call STDEV, STDEV.S, STDEVA, VAR, VAR.S, VARA and
    DEVSQ probe cell into a math candidate, and those cells have no sidecar entry yet. One example is
    probes-numeric/t1749-devsq-fresh, which has about 200 DEVSQ cells.
  - T-3925 met the same step: its Outcome lists the 42 stale cases it regenerated.
  - Everything else passed: `rustsheet-eval accuracy` reports 0 failing and missing tables 128, and the eval run
    itself is green.
- **Next:**
  1. Regenerate only the stale cases, additively. Take each case's sidecar as it stands on origin/main and add the
     new candidates, so that no unrelated one-call entry drifts.
     - The T-3993 session did exactly this with a short script over `eval_math._spec_entries(suite)`, calling
       `case_math` per case. The whole-suite `rustsheet-accuracy math` takes over 30 minutes.
     - Check the drift with a key-by-key comparison against origin/main. Expect no existing entry to be gone or
       changed.
  2. Run `make eval-math-check`, then `make check-fast`. Push, and add the regenerated case count to the Outcome.
  3. Re-run only the reviewers the sidecar change concerns (excel-fidelity), then enqueue.
- **Do not repeat:**
  - The first draws cut out equal numbers (more than two) and relative spreads below 1E-6. The fidelity review
    rejected that as tuning the domain to agreement, so both regions are drawn again.
  - Giving DEVSQ VAR's one-pass overflow rule was also rejected.
