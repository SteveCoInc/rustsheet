---
id: T-4084
title: "Gate A generator: a text parameter kind (complex numbers and CONVERT units)"
milestone: M0
lane: harness
status: done
priority: P3
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
The Gate A generator has a `text` parameter kind, so a definition can draw complex-number text and fixed-list text
(CONVERT's unit names) and the table and math sidecar carry it.

## Context
- Decision 0270 (T-3800) made IMABS, IMARGUMENT and CONVERT `composite`, although they have floating-point
  kernels, only because their arguments are text and `accuracy/model.py` `Kind` has `real`, `int`, `bool` and `array`.
- Decision 0228 did the same for the matrix functions until T-3686 added 2-D literals. This task is the counterpart.
- Shapes needed: a complex number `"a+bi"` / `"a-bj"` built from two drawn reals (the IM* parse rules are in
  engineering.md, "Leading plus in IM*..."), and a choice from a fixed list (CONVERT's units, engineering.md "CONVERT
  unit names").
- The text must reach the formula as a quoted string literal, and the math side must receive the parsed values.

## Acceptance criteria
- [x] `Kind` gains a text kind with a complex-number sampler and a fixed-list sampler, with tests. The formula writer
      quotes it and escapes `"`.
- [x] Existing tables regenerate byte for byte (no table changes).
- [x] `make check` green; [ ] reviewers approve.

## Out of scope
- The tables themselves (T-4085).

## Outcome
- `model.Kind` gains `text`; `literal.Arg` gains `str`, written by `literal.text` as a string literal (`gen.common.quoted`,
  a `"` doubled). `Param.choices` makes a fixed-list text (CONVERT's units); without choices `Param.is_complex` holds and
  the text is a complex number `a+bi` / `a-bj` (`literal.complex_text`, both parts 15-digit literals, the imaginary part
  always signed). The sampler draws the two parts with the parameter's own typical, extreme and whole-domain draws over
  the parameter as a real, and one suffix per point for all its complex texts (mixed suffixes are an error,
  engineering.md "IMSUM and IMPRODUCT take their arguments whole"); only a definition with a complex text draws it, so
  no other definition's draws move. Text parameters get no edge points.
- The math receives an `mpc` of the parsed parts (`fitlab.sweep.parse_complex_text`, reused) or the list text itself;
  `check_domain` checks each complex part against the parameter's interval. `tables.parse_formula` reads a string
  literal back (`sweep.string_literal`, split out of `_literal`).
- Eval cells: `eval_math.fit_arg` takes a string literal or text cell for a text parameter: a listed choice, or a complex
  text `parse_complex_text` reads with numbers of at most 15 significant digits (longer IM* parts are unprobed); one
  call's complex texts must share a suffix. A reason is now an `model.Unfit` (a str subclass) so it is told apart from a
  fitted text; composite cells use the same path.
- `MathArg` gains `mpc | str`, so six defs files narrow with `(bool, str, tuple)` where they narrowed `(bool, tuple)`.
- Tests: `tools/tests/test_accuracy_text_param.py` (both samplers, quoting and escaping, round trip, math, eval cells).
- Byte-identical: all 228 committed batches (224, then 4 more after merging origin/main) were regenerated in memory (committed math reused for points it holds,
  everything else computed). 217 are byte-identical; the other 11 (BETA.DIST s1 and r4-T-3879, BETA.INV, CHISQ.INV,
  CHISQ.INV.RT, GAMMA.INV, T.DIST s1 and r4-T-2585, T.INV, T.INV.2T, TINV) give exactly the same bytes under origin/main's
  generator, so that gap predates this change (as T-3686 recorded). `rustsheet-accuracy verify` passes for every function.
  `make eval-math-check` is current.
