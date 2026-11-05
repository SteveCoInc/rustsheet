---
id: T-4295
title: "TEXT \"[s] m\" shows the seconds field and 26640 (not 6138 42): find the rule"
milestone: M10
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`TEXT(1.7/24+0.3/1440,"[s] m")` matches Excel's "18 26640" in both date systems, with the rule documented.

## Context
Found by T-2135. probes/t2135-fractional-second-edges-1900 S!A39 and probes/t2135-fractional-second-edges-1904 S!A39: the engine gives "6138 42". `m [m]` already gives "1 26640", so 26640 is likely the same quantity; the rule for `[s]` printing 18 and for 26640 is unknown and needs more probes (other values, `[s] mm`, `[s]:m`). See number-formats.md "Elapsed-time tokens".

## Acceptance criteria
- [x] Probe the rule (excel-probe skill), record it in number-formats.md, and make both A39 cells pass.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Rule found and implemented. `[s]` before an hour or minute token hands its elapsed count to that token: `[s] m`
prints like `s [m]`, counted from the date section's register, which the engine already had for elapsed tokens after
a date token (seconds within the minute plus the half-second step, as days: 18.5 days is 26640 minutes).
- `move_elapsed_second` (formatter/elapsed.rs) rewrites the parsed section. `[s] m s` was #VALUE! and is now
  "18 26640 18".
- New probes probes/t4295-elapsed-with-month (126 cells) and -b (72 cells, excel-graph). Generator in
  docs/research/t4295/. The rule is in number-formats.md "Elapsed-time tokens".
- Eval delta: t2135-fractional-second-edges-1900 and -1904 A39 fixed. The new probes pass 198 of 198 (45 of them
  fixed by this change). Regressions 0.
- `[ss]` before `h`/`m`, and a second hour or minute token after the first, are UNVERIFIED (not probed).
