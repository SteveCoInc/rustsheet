---
id: T-4333
title: "TEXT elapsed count before a fraction at a tie where Excel rounds up (45000+0.95/86400 under [s].0 is 3888000001.9)"
milestone: M10
lane: functions-text
status: blocked
priority: P1
size: M
tier: opus
depends_on: [T-4460]
---

## Goal
The elapsed `[h]`/`[m]`/`[s]` count before a fraction matches Excel at the ties where it rounds up while the clock and fraction digits truncate: `TEXT(45000+0.95/86400,"[s].0")` is "3888000001.9", `TEXT(3+5.95/86400,"[s].0")` "259206.9", `TEXT(45000+59.95/86400,"[m]:ss.0")` "64800001:59.9" (ours one less each).

## Context
- Decision 0355: this task owns foundational-tier cells, which the tier gate needs 100% right on type, error and text, so it is P1.
Found by T-4323, which counts with fraction digits from the half-step register, `floor(y*86400/unit)` (`FractionalSeconds::new`, formatter/elapsed.rs): 350 of 358 count cells of the six earlier t4323 cases and 117 of 120 of the fresh carry sweep. Visibly-off cells: probes/t4323-fraction-ties A90; probes/t4323-fraction-sweep-2 A1, A24, A41, A84; probes/t4323-fraction-sweep-3 A7; probes/t4323-carry-ties A27, A31; probes/t4323-carry-sweep A37, A49, A83. Every one has y*86400 just below a whole second (e.g. 3888000000.9999995), but so do cells Excel truncates (`TEXT(3497.95/86400,"[s].0")` "3497.9", y*86400 3497.9999999999995; sweep-2 A9, A19, A27). Refuted counts (docs/research/t4323, scratch scoring in T-4323's Outcome): the value rounded to the shown digit (346/358, misses the carry ties), whole days plus the register's clock count, 12- to 17-digit snaps of y*86400/unit, and the register floored in units before dividing. See docs/excel-semantics/number-formats.md "Fraction digits of a second at a tie". A structural model: owes the R4 hold-out and a fresh sweep.

- Since T-4332 (PR #3389), the clock and fraction digits are cut field by field (`field_split`, formatter/elapsed.rs),
  while these counts still read the register `floor(y*86400/unit)`. Near a whole-second carry the count and the
  fraction can therefore come from different cuts, for example a count one too low beside ".0". No gated cell shows
  this yet; check it when choosing the count model. When this task lands, also restate the "Rule:" paragraph of
  number-formats.md "Fraction digits of a second at a tie": it still names the register for the clock and fraction
  digits, which T-4332's cut now drives at every serial (T-4332 fidelity review, round 2).

## Acceptance criteria
- [ ] Those cells, and probes/t4333-count-ulp-scan D1:D4, D7 and E6, pass without moving another cell of the seven t4323 probe cases.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The clock and fraction digits (T-4332's field-by-field cut, T-4370 below serial 1).

## Blocked because: time-boxed (decision 0056)
Second attempt without a gain (the first was T-4323's count search; see Context). This one (2026-11-05, opus run)
scored the count candidates in docs/research/t4333 (`count_models.py`, table in its README) on every elapsed-count cell
with fraction digits: the seven t4323 cases, probes/t4332-fraction-fresh and a new ulp scan, 590 cells. The engine's
register `floor(y*86400/unit)` stays best at 573. Next: 571, `floor(days*86400 + (frac(x)+h)*86400)`; 568 to 553,
the rounded value, `x*86400` plus the half step, whole days plus the field-cut or register clock, and exact `(x+h)*86400`;
545 to 550, `y*86400` snapped to 13 to 17 digits. In scratch, nothing beat the register: 64-bit extended
intermediates, a constant other than 0.5 in the half step, a float32 half step, and chains through 24/60/60 or
1440/60 with the half step at the seconds stage.

New evidence: probes/t4333-count-ulp-scan (excel-graph) steps eight tie serials by -3 to +3 ulp from Excel's
computed serial under `[s].0`. Every cell where the count disagrees with the fraction digit is the double nearest the
decimal tie: D1:D4, rounded up beside ".9", and E6. E6 is `3733.95/86400` + 1 ulp, the nearest double in row 6 because
the computed serial is 0.61 ulp below the tie, and Excel shows "3733.0": the fraction carried, the count not. One ulp
or more from the nearest double, the count and the fraction agree. D7 (60000+82799.95 s) truncates where the register
carries. These cells are this task's: probes/t4333-count-ulp-scan D1, D2, D3, D4, D7, E6, plus the eleven listed in
Context.

Reopen on a new hypothesis recorded here. Two would be worth testing:
- the count may come from a decimal conversion of the serial that is exact only at the nearest double, such as a
  digit string rounded half-up. The x snaps tried were binary-to-decimal at 13, 15 and 17 digits with half-even
  ties, then exact arithmetic;
- Excel may compute the count from the `[s]` token's own whole-second rounding with a carry test against the
  fraction it formats. A probe of `[s]` beside `.0`, `.00` and `.000` on the same serials would separate the two.

Gap triage 2026-11-06: T-4460 (research) is the oracle sweep that tests both hypotheses. This task waits on it.
