---
id: T-3976
title: "Probe the edges of the letter-period colon tail T-3957 left unprobed: N 11-23 and 12, an empty G, DATEVALUE of a dated non-colon form past 9999"
milestone: M6
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3957]
---

## Goal
Oracle evidence (excel-graph) for the letter-period colon-tail edges that T-3957 (PR #3023) placed without a probe, so
dates.md "Typed clock-first date-times" names each one VERIFIED, or an engine task owns the cells that differ.

## Context
- T-3957 decoded `N A. F:G` from probes/t3957-letter-period-colon-tail (excel-graph, 2026-11-03). Its Excel-fidelity
  review (MINOR) listed the edges with no row:
  - the clock bound: only N = 0, 10 (numbers) and 24 (text) are probed; `colon_tail_parts` (formatter/date_clock.rs)
    accepts N < 24. Probe `11`, `12`, `13`, `23` with `A.` and `P.` (is `12 A.` 12 h or 0 h?).
  - an empty G: `10 A. 5:` stays text in the engine, but ordinary clock text ignores one trailing `:`.
  - DATEVALUE without the range check (T-3957 dropped it for every letter-period reading, evidenced only past
    9999-12-31 for the colon form): probe a dated non-colon form near the top, `10 A. 5 12/31/9999`, and a colon form
    with a date, `10 A. 5:30 12/31/9999`.
  - three-digit F or G other than `100:30` (`10 A. 5:100`, `10 A. 123:30`), and `/` or `-` right after G
    (`10 A. 5:30/1/2025`, `10 A. 5:30-2025`).
- Layout: copy probes/t3957-letter-period-colon-tail (A typed, B ISNUMBER, C value or LEN, D TEXT, E VALUE, F `0+`,
  G DATEVALUE, H TIMEVALUE). Use the `excel-probe` skill.
- Design note from T-3957's review for any engine follow-up: the date-tail block of `colon_tail_parts` repeats the one in
  `letter_period_parts` (about 6 lines); share a helper when the code changes next.

## Acceptance criteria
- [ ] A probes case with the rows above, filled by excel-graph.
- [ ] dates.md records the results, VERIFIED where the engine agrees; every visibly-off cell has an owner task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a follow-up task does that).
