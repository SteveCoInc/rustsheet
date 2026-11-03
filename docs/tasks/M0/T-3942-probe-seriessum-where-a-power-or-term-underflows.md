---
id: T-3942
title: "Probe SERIESSUM where a power or term underflows (the flush T-3676's definition infers)"
milestone: M0
lane: research
status: blocked
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what SERIESSUM gives when a power x^(n+i·m) or a term c_i·x^(n+i·m) falls below the smallest normal
double. T-3676's Gate A definition (tools/src/rustsheet_tools/accuracy/defs/seriessum.py) flushes such a power or term
to 0, inferring it from numbers.md "Subnormal results become 0" (`^` and POWER flush) and T-2163 (SERIESSUM's power is
Excel's `^`). No oracle cell checks it.

## Context
- Filed by T-3676's Excel-fidelity review (MINOR). No s1 point depends on the rule today: `_normal_powers` drops draws
  whose power would underflow.
- Probe ideas: `SERIESSUM(1E-200,2,1,{1})` with an `=(SERIESSUM(1E-200,2,1,{1}))=0` companion cell (decision 0042), a
  subnormal term from a normal power (`SERIESSUM(1E-154,2,0,{1E-10})`), a flushed term next to a normal one
  (`SERIESSUM(1E-200,2,1,{1,1E+300})`), and the same rows with a negative x. Use the `excel-probe` skill, excel-graph.

## Acceptance criteria
- [ ] A probes case with the cells above (and their `=0` companions), filled by excel-graph.
- [ ] docs/excel-semantics/numbers.md records the result next to the SERIESSUM entries, VERIFIED or with the deviation.
- [ ] If Excel does not flush, a follow-up task to change `defs/seriessum.py` is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's SERIESSUM.

## Outcome
- Added probes/seriessum-subnormal-power-and-term (34 cells, A1:B17) and probes/seriessum-subnormal-term-sum-and-power-points (12 cells, A1:B6), each call with its `=0` companion, filled by excel-graph.
- Excel flushes a subnormal SERIESSUM power and a subnormal term to 0, per term and not on the sum (term-sum case A1, A2), so T-3676's
  per-term definition is right. It also found that `^` with a positive exponent keeps its subnormal while POWER and a negative
  exponent (2^-1023) flush, and that the engine keeps a subnormal SERIESSUM term and flushes `^`. numbers.md records all of it
  (VERIFIED, limited to the probed bases and exponents). Follow-ups: T-3949 (SERIESSUM term) and T-3950 (`^`), each owning its cells.
- The acceptance item "if Excel does not flush, file a task to change defs/seriessum.py" does not apply: Excel flushes.
- Eval delta: none gated; 10 new known-failing cells, all owned (T-3949: power-and-term A2, B2, A9, B9, term-sum A1; T-3950: power-and-term A16, B15, B16, term-sum B3, B4).

## Blocked because
Blocked because: review-round limit

Three review rounds ran on this branch (heads b828edb14, 73ce0f947, 20d32fb86). The design reviewer approved the last head.
The Excel-fidelity reviewer asked for changes each round, each time about how far the numbers.md claims reach:
1. The `^` claim was broader than the probe (C16 shows a negative exponent flushes).
2. The per-term SERIESSUM flush was not separated from a result flush (fixed with the term-sum case).
3. `x^2` keeping its subnormal conflicts with `power-large-integer-exponents` B103, B107, B117, B119 (positive-exponent chains flush).
The third round's fixes are committed after this note (numbers.md and T-3950 narrowed to the probed x^2 points, cross-notes added),
but that head has had no reviewer run. Next step: an opus run resumes this branch per work-loop step 5c, runs `make check-fast` and
both reviewers from round 1 on the current head, then enqueues. Do not widen the `^` claim beyond x^2 at 1E-160, 1E-155, 1.1E-155.
