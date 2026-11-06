---
id: T-4687
title: "numbers.md: settle the stale huge-angle trig and huge num_digits ROUND notes from the entries that answered them"
milestone: M5
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/numbers.md has two older entries whose "Ours" or "Status" lines still say UNVERIFIED or describe
engine behaviour from before the fix, although later entries in the same file answer them with excel-graph evidence.
Each older entry is rewritten so it points at the entry that settles it and no longer contradicts it. No engine
change.

## Context
- Gap triage 2026-11-06 (evening steward run, `make eval` on agent/steward/2026-11-06). The functions-math lane had no
  ready task. Every failing math cell of the run has an open owner (the deferred last-digit fits T-1940, T-2651,
  T-3018/T-3020, T-3727, T-2744, T-2141, T-2384, plus T-4373 and T-2906), and the math argument-kind gaps have research
  owners (T-2862, T-2871, T-4035). A reader of numbers.md who sees an UNVERIFIED rule that is in fact verified may
  re-probe it, or "fix" the engine back to the stale description.
- Note 1: "## Huge arguments of trig functions" (around line 2467).
  - "Ours: COS, SIN, SEC and CSC compute a number; ACOS and ASIN yield a NaN number that `powf(0)` turns into 1 (and
    that comparisons read as FALSE)." This is stale. A scratch evaluation on 2026-11-06 gives `ISERROR(SIN(1E+15))`
    TRUE, `ACOS(1E+15)^0` `#NUM!` and `ISERROR(ACOS(1E+15))` TRUE.
  - "Status: VERIFIED for the observed cells; the exact threshold and TAN are UNVERIFIED." The cut is settled by
    "## Trig of a huge angle" (around line 2820): |x| >= 2^27 for SIN, COS, TAN, COT, SEC and CSC, from
    probes/trig-huge-argument (T-1201, passes 86 of 86, includes TAN(2^27-1), TAN(2^27), TAN(2^28)) and the fractional
    cut of T-2172. The out-of-domain ACOS/ASIN part is settled by "## Out-of-domain ACOS/ASIN/ACOSH/ATANH are `#NUM!` at
    once, even nested (T-1202)".
- Note 2: "## ROUND family with huge num_digits" (around line 2017) has "Status: UNVERIFIED (the rule separating the two
  outcomes needs a probe before implementing; T-0218)". T-0218 is done. "## ROUND family: huge num_digits wrap" (line
  288, Status VERIFIED, probes/round-huge-digits and probes/round-huge-digits-wrap, T-0218, T-0268) gives the rule
  that separates the two outcomes: ROUND saturates num_digits to 32 bits and wraps the kept-digit count. Check the three
  gen-m2 cells the old entry cites (gen-m2/f7e6d351c8538ef5 A35, 893302d6c5b99e2e A136, 7469c3e920e1ade4 A336) and the
  `ROUND(1000000000000000.0,Data!$D$7)` = 0 example against that rule with `make eval-show`, and say in the old entry
  which branch each one takes.
- Keep both older entries and their evidence lines: they hold gen-cell evidence the newer entries do not repeat. Edit
  only their "Ours" and "Status" lines and add a "See" pointer.

## Acceptance criteria
- [ ] In "## Huge arguments of trig functions", the "Ours" line describes the current engine (an error for |x| >= 2^27,
      `#NUM!` for out-of-domain ACOS/ASIN, nested too) and names the code that does it, found with `rg`. The "Status"
      line no longer says UNVERIFIED and points at "## Trig of a huge angle".
- [ ] In "## ROUND family with huge num_digits", the Status is VERIFIED and points at "## ROUND family: huge num_digits
      wrap". The cited gen-m2 cells are each placed on the wrap rule's branch they take.
- [ ] `make eval-show` shows that every cell named in the edited entries passes. Any that does not becomes a
      functions-math task filed with `make new-task`, and the entry says so instead of VERIFIED.
- [ ] No file outside docs/excel-semantics/numbers.md changes, apart from this task file's Outcome.
- [ ] `make check-fast` green; reviewers approve.

## Out of scope
- Any other numbers.md entry; the trig last-digit fits (T-3018, T-3020, T-2744, T-3727).
