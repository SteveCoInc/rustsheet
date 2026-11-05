---
id: T-4370
title: "TEXT fraction digits of a second at a tie: the cells T-4332's field-by-field cut still misses"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
TEXT's fraction digits of a second, and the clock beside them, match Excel at the rounding ties that T-4332's
field-by-field cut (`field_split`, formatter/elapsed.rs) still misses. Most are below serial 1, where Excel rounds the
tie up and the cut gives one unit less: `TEXT(0.0005/86400,"[h]:mm:ss.000")` is "0:00:00.001", ours "0:00:00.000";
`TEXT(3.55/86400,"[s].0")` is "3.6", ours "3.5".

## Context
T-4332 cuts `frac(x + 0.5/(86400*s))` field by field (24, 60, 60, s). That gets 134 of the 156 tie cells below serial
1 and 469 of 470 from 1 up in the five t4323 cases, and 103 of 120 in the fresh sweep probes/t4332-fraction-fresh.
T-4323's register `floor(frac(y)*86400*s)` gets 101, 467 and 72. Five cells the register matched now miss (decision
0294): t4323-fraction-ties A121, A122, A124; t4323-fraction-sweep-2 A124; t4323-fraction-holdout A100. In most misses
the cut's last remainder lies 1 to 11 ulp below a whole unit at a small seconds value. Yet cells Excel truncates lie
8 to 32 ulp below (holdout A5 `20.65/86400` "[s].0" is "20.6"; sweep-2 A34 `9.35/86400`). So no ulp snap on that
remainder separates them. docs/research/t4332/chain.py scores the model, and README.md lists what was tried and refuted
(see also docs/research/t4323/README.md).

Visibly-off cells this task owns (the `[s]`/`[h]`/`[m]` count cells stay with T-4333: t4323-fraction-ties A90;
t4323-fraction-sweep-2 A1, A24, A41, A84; t4323-fraction-sweep-3 A7):
- probes/t4323-fraction-ties A96, A97, A121, A122, A124, A128
- probes/t4323-fraction-sweep-2 A13, A15, A33, A36, A55, A61, A70, A82, A105, A109, A120, A124
- probes/t4323-fraction-sweep-3 A25
- probes/t4323-fraction-holdout A100
- probes/t4332-fraction-fresh A2, A4, A5, A8, A13, A28, A30, A31, A32, A34, A36, A42, A43, A55, A65, A74, A80

A structural model: owes the R4 hold-out and a fresh sweep.

## Acceptance criteria
- [ ] The listed cells pass, and no other cell of the six fraction cases (five t4323, t4332-fraction-fresh) moves;
      at the least, beat `field_split`'s count on them (`python3 docs/research/t4332/chain.py`).
- [ ] A fresh sweep generated after the model is frozen confirms it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The elapsed counts before the fraction (T-4333).
