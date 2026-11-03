---
id: T-3942
title: "Probe SERIESSUM where a power or term underflows (the flush T-3676's definition infers)"
milestone: M0
lane: research
status: done
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
