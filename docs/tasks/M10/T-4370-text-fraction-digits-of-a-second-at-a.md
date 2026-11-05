---
id: T-4370
title: "TEXT fraction digits of a second at a tie: the cells T-4332's field-by-field cut still misses"
milestone: M10
lane: functions-text
status: done
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
remainder separates them. All the evidence is excel-graph (Excel for the web), and web and desktop may differ by up to
4 ulp (decision 0188 R4.4, docs/research/oracle-calibration.md). Before chasing outcomes decided within a few ulp,
check them on excel-desktop, or treat them as at the reproducibility floor. docs/research/t4332/chain.py scores the model, and README.md lists what was tried and refuted
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
- [x] The listed cells pass, and no other cell of the six fraction cases (five t4323, t4332-fraction-fresh) moves;
      at the least, beat `field_split`'s count on them (`python3 docs/research/t4332/chain.py`).
- [x] A fresh sweep generated after the model is frozen confirms it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The elapsed counts before the fraction (T-4333).

## Outcome
Excel reads the fraction digits of a second apart from the clock: the remainder `r` of a second that T-4332's
half-step cut leaves after the second 60 is rounded to 15 significant digits, and the digits are `floor(r15*scale)`
modulo `scale` (formatter/elapsed.rs `fraction_digits`; number-formats.md "Fraction digits of a second at a tie").
The seconds field is cut on its own and does not carry when `r15` reaches 1: `TEXT(41.95/86400-2*2^-64,"[s].0")` is
"41.0" (probes/t4370-ulp-map-2 A417), the clue that split the two paths.

- Evidence: two ulp maps generated for the search, probes/t4370-ulp-map (340 cells) and t4370-ulp-map-2 (462), which
  step 70 tie serials up to 64 ulp and show the flip depends on the serial only, not the format; and the fresh sweep
  probes/t4370-fraction-fresh (120 cells, seed 43700, the first 60 separating the model from the cut), generated after
  docs/research/t4370/model.py was frozen. All excel-graph 2026-11-05.
- Scores (model / T-4332's cut): the six fraction cases 743/746 against 706/746, so all 37 listed cells pass and no
  other cell moves: the engine still misses only T-4333's six `[s]` count cells (ties A90, sweep-2 A1, A24, A41, A84,
  sweep-3 A7), as before (the Python scorer's simpler count rule misses three others instead); fresh sweep 120/120 against 54/120; ulp
  maps 739/802 against 554/802 (the engine 740). R4: the model is structural (no fitted constant); it was chosen on the
  ulp maps and the six cases, and the fresh sweep drawn after the freeze is its out-of-sample check.
- Left: `.000` on a remainder below about 0.06 s rounds up further than the 15th digit allows (as if 1.6E-17 to 1.9E-17
  s were added): the engine's 62 ulp-map misses, all owned by the follow-up T-4389. Searches refuted on the way are
  in docs/research/t4370/README.md.
- Eval delta: probes 37 cells fixed (the listed ones), 0 regressions; the three new cases add 62 known failures, all owned
  by T-4389 (visible-parity lock +62 for them, 1838 in-scope cells visibly off).
