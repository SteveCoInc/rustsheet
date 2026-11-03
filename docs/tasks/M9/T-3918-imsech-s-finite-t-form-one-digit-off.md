---
id: T-3918
title: "IMSECH's finite-t form one digit off at x = -354.03 (IMCSCH flip-start-negative A3; T-3883 follow-up)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
`IMCSCH(COMPLEX(-354.03,1))` prints Excel's text. It goes through IMSECH's finite-t form, where t = sinh²x + cos²y does not
overflow.

## Context
- Left over from T-3883. That task changed only the overflowed form (`overflowed_parts`, parts over cosh |v|), which
  fixed the 24 overflow-band IMSECH and IMCSCH cells it owned. This cell is not on that path.
- probes/t3881-imcsch-flip-start-negative.case.json Calc!A3 (excel-graph): Excel
  1.90717870311855E-154+2.97025484453415E-154i, ours ...2.97025484453414E-154i. Inside IMCSCH's negated band, IMCSCH
  is IMSECH's parts (`sech_parts` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs, T-3881). So the
  digit is IMSECH's own `(cosh x·cos y, -sinh x·sin y)/t`, with sinh²x fused into t.
- IMSECH(COMPLEX(-354.03,1)) is unprobed. Probe it first, plus a few x in [-355.6, -354] and [354, 355.6] (both
  functions), where t is finite but huge. One candidate is the parts over cosh x (T-3883's form) once sinh²x is
  near overflow. Ranking candidates owes the R4 guards (reviewer-excel-fidelity check 7).
- One digit at the 15th place, not visibly off (decision 0188 R5), so `defer: phase-1b`.

## Acceptance criteria
- [ ] flip-start-negative A3 passes, and no other IM* cell moves; or the Outcome lists the forms tried.
- [ ] R4 guards recorded in the Outcome if candidates are ranked.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The overflowed form (T-3883, done).
