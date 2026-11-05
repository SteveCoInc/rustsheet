---
id: T-4366
title: "Math sidecars: BETA.INV's A = B and B < A as cited #NUM! domain errors; BETA.DIST given-x end overlap gives no math value"
milestone: M0
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3923]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
The BETA.DIST and BETA.INV math definitions (tools/src/rustsheet_tools/accuracy/defs/beta.py, as T-3923 extends
them to the [A, B] bounds) say only what the evidence says about degenerate and reversed bounds.

## Context
These are T-3923's excel-fidelity review MINOR findings (PR #3355). They do not block that PR: no committed
sidecar has a cell in these forms today.
- BETA.INV with A = B: beta.py calls it "unprobed" and gives no math value. It has been probed:
  docs/excel-semantics/statistics.md has `=BETA.INV(0.4,2,3,TRUE,1)` (A equal to B) as `#NUM!` (search for it).
  Cite that entry and make A = B a domain error (`#NUM!`) instead of "no math value".
- BETA.INV's domain function says "B < A is `#NUM!`" with no cited evidence. The only source is a comment in
  crates/rustsheet-engine (beta.rs). Either cite an oracle probe (use the `excel-probe` skill and record the
  result in statistics.md) or give no math value until one exists.
- BETA.DIST, non-cumulative, with a given x of 0 or 1 that is also an end of [A, B]. `_beta_dist` gives no math
  value only when the shape at that end of t is 1 or less. Otherwise (for example x = 0 = B with alpha <= 1 and
  beta > 1) it returns a density of 0, although Excel's given-x `#NUM!` rule (statistics.md, "BETA.DIST's density
  is #NUM! at an x of 0 or 1 (T-2435)") may apply. Give no math value for every such call, or probe it and follow
  Excel. The comment in `_beta_dist_domain` must then say what the code does.

## Acceptance criteria
- [ ] BETA.INV with A = B is a cited domain error. Its unit test is in tools/tests/test_accuracy_call_forms.py.
- [ ] BETA.INV with B < A has either a cited probe or no math value, with a unit test.
- [ ] A non-cumulative BETA.DIST call whose given x of 0 or 1 is also an end of [A, B] gets no math value (or the
      probed Excel value), with a unit test. The code comment matches the code.
- [ ] `make eval-math-check` is current, with any sidecar regenerated case by case (never the whole-suite `math`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other BETA.DIST/BETA.INV accuracy work (T-3933 and its successors).
