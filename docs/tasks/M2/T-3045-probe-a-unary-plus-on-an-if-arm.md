---
id: T-3045
title: "Probe a unary plus on an IF arm and on OFFSET's reference under legacy N (T-2885 follow-up)"
milestone: M2
lane: research
status: blocked
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Know what legacy `N(IF(TRUE,+B1:B3))` and `N(OFFSET(+B1:B3,0,0))` do in Excel (value or `@`, FORMULATEXT), and make
`is_plain_reference` match.

## Context
T-2885 made a plus on the whole N/T argument intersect (probes/t2772-unary-plus-in-range-rows J2, J5). A plus on an IF
arm or on OFFSET's first argument is unprobed; `is_plain_reference` (static_analysis/legacy.rs) still reads them as plain
references. Use the excel-probe skill; see docs/excel-semantics/syntax.md "A typed unary plus stays in the formula text".

## Acceptance criteria
- [ ] A probe case commits oracle values for both shapes, on and off the range's rows.
- [ ] `is_plain_reference` matches them; syntax.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
None. Also probe `N((+B1:B3))` (a plus under written parentheses) and `T(+B1:B3)` (T shares the N rule unprobed, T-2885).

## Blocked because
Oracle: `rustsheet-oracle fill -b excel-graph --suite probes` on 2026-11-04 failed for
`probes/t3045-unary-plus-if-arm-offset` with Graph 501 `FileCorruptTryRepair` (unsupportedWorkbook) at createSession; two
unrelated existing probe cases (t0481-curly-quote-names-c, typed-if-one-argument) failed the same way in the same run, so
it may be transient. The case is committed with `"expected": {}` (pending). Next: re-run the fill; if it still fails for
this case alone, bisect the 8 formula shapes (a stored plus on OFFSET's reference or `(+range)` is the likely refusal) and
use `entered` mode as in probes/t2723. Then add the `is_plain_reference` change and the syntax.md entry.
