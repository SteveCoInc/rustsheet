---
id: T-3979
title: "Read typed '10 A. 5:30' (a colon in the letter-period tail) as Excel does once T-3957 decodes it: the M2 owner of probes/t3948-letter-period-gate-edges B10:H10"
milestone: M2
lane: functions-date
status: done
priority: P2
size: S
tier: opus
depends_on: [T-3957]
campaign: semantics
---

## Goal
Typed `10 A. 5:30` and its coerced forms (VALUE, `0+`, DATEVALUE, TIMEVALUE) give Excel's number, so the seven
M2 exit cells of probes/t3948-letter-period-gate-edges row 10 pass, using the colon-tail rule T-3957 decodes.

## Context
- Gap triage 2026-11-03 (`make eval` on origin/main b9dac1c15 with calipers, poi and corpus-bench fetched;
  `make phase1-exit`): the `owners` line is not met because `make exit-owners MILESTONE=M2` lists 7 cells with no
  M2 owner. The case is tagged M2 (decision 0051: case tag, not in eval/milestones/probe-milestones.tsv), so its
  failing cells count toward the M0-M2 probe row; their only open owner, T-3957, is an M6 research task, which the
  M2 exit does not count. This task is the M2 owner; T-3957 keeps the decoding (the sweep and the rule in dates.md).
- Cells (`make eval-show CASE=probes/t3948-letter-period-gate-edges.case.json`, excel-graph 2026-11-03), Excel
  against ours:
  - B10 `ISNUMBER(A10)` TRUE, ours FALSE (the typed entry stays text);
  - C10 `IF(ISNUMBER(A10),A10,LEN(A10))` 2982616.579230324, ours 10;
  - D10 `IF(ISNUMBER(A10),TEXT(A10,"yyyy-mm-dd hh:mm:ss.000"),"")` #VALUE!, ours "" (a number past 9999-12-31);
  - E10 `VALUE("10 A. 5:30")` and F10 `0+"10 A. 5:30"` 2982616.579230324, ours #VALUE!;
  - G10 `DATEVALUE("10 A. 5:30")` 2982616, ours #VALUE!;
  - H10 `TIMEVALUE("10 A. 5:30")` 0.5792303238995373, ours #VALUE!.
- probes/t3948-letter-period-gate-edges.case.json (7): S!B10, C10, D10, E10, F10, G10, H10 (shared with T-3957,
  which decodes the rule; this task implements it and owns the M2 exit of these cells).
- The quirk is recorded: docs/excel-semantics/dates.md "Typed clock-first date-times" (the paragraph citing
  t3948-letter-period-gate-edges, "The odd one: typed `10 A. 5:30` ..."). Excel is right however odd the value
  (60 x (2^32 - 22) s + 10 h + 5.5 s); do not tidy it.
- Code: crates/rustsheet-engine/src/formatter/date_clock.rs `is_letter_period_shape` (the gate that leaves the text a
  string today) and `letter_period_parts` (the reading T-3907, T-3920 and T-3948 built, functions-date lane). One
  reading serves typed input and the four readers, so the fix goes there, not in each reader.
- Why opus: the rule is not decoded yet (one Excel point); T-3957's sweep supplies the evidence. If T-3957's PR
  already implements it and these cells pass, close this task as done by that PR in its Outcome.

## Visible-parity stage
`semantics`: the seven cells are a wrong type, error or number from a reading rule (the text is not read as a
date-time at all). Target: S!B10:H10 pass bit-exact.

## Acceptance criteria
- [x] probes/t3948-letter-period-gate-edges S!B10, C10, D10, E10, F10, G10 and H10 pass, so these 7 known-failure
      lines disappear from eval/baseline/probes.txt with no new ones (`make eval-baseline`, 0 regressions).
- [x] The rest of T-3957's sweep (its colon-tail rows) passes, or each remaining cell is owned by an open task.
- [x] `make exit-owners MILESTONE=M2` lists none of these cells; dates.md's entry says the engine matches them.
- [x] `make check` green; reviewers approve.

## Out of scope
- Decoding the rule and the probe sweep (T-3957).
- Other letter-period shapes already matched (T-3907, T-3920, T-3948) and DATEVALUE("10 A.") (T-3944, done).

## Outcome
Done by T-3957's PR #3023 (b7700b7d, "decode and read letter-period text with a colon tail"), which landed the
reading as well as the decoding. On origin/main 989f4dc1, eval/baseline/probes.txt lists
probes/t3948-letter-period-gate-edges.case.json with no known-failure lines (only its `C ... 112` count line), so
S!B10:H10 pass, and probes/t3957-letter-period-colon-tail.case.json (168 cells) has none either. No code change here;
eval delta 0.
