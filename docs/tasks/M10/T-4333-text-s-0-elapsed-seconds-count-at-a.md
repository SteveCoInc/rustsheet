---
id: T-4333
title: "TEXT elapsed count before a fraction at a tie where Excel rounds up (45000+0.95/86400 under [s].0 is 3888000001.9)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The elapsed `[h]`/`[m]`/`[s]` count before a fraction matches Excel at the ties where it rounds up while the clock and fraction digits truncate: `TEXT(45000+0.95/86400,"[s].0")` is "3888000001.9", `TEXT(3+5.95/86400,"[s].0")` "259206.9", `TEXT(45000+59.95/86400,"[m]:ss.0")` "64800001:59.9" (ours one less each).

## Context
Found by T-4323, which counts with fraction digits from the half-step register, `floor(y*86400/unit)` (`FractionalSeconds::new`, formatter/elapsed.rs): 350 of 358 count cells of the six earlier t4323 cases and 117 of 120 of the fresh carry sweep. Visibly-off cells: probes/t4323-fraction-ties A90; probes/t4323-fraction-sweep-2 A1, A24, A41, A84; probes/t4323-fraction-sweep-3 A7; probes/t4323-carry-ties A27, A31; probes/t4323-carry-sweep A37, A49, A83. Every one has y*86400 just below a whole second (e.g. 3888000000.9999995), but so do cells Excel truncates (`TEXT(3497.95/86400,"[s].0")` "3497.9", y*86400 3497.9999999999995; sweep-2 A9, A19, A27). Refuted counts (docs/research/t4323, scratch scoring in T-4323's Outcome): the value rounded to the shown digit (346/358, misses the carry ties), whole days plus the register's clock count, 12- to 17-digit snaps of y*86400/unit, and the register floored in units before dividing. See docs/excel-semantics/number-formats.md "Fraction digits of a second at a tie". A structural model: owes the R4 hold-out and a fresh sweep.

- Since T-4332 (PR #3389), the clock and fraction digits are cut field by field (`field_split`, formatter/elapsed.rs),
  while these counts still read the register `floor(y*86400/unit)`. Near a whole-second carry the count and the
  fraction can therefore come from different cuts, for example a count one too low beside ".0". No gated cell shows
  this yet; check it when choosing the count model. When this task lands, also restate the "Rule:" paragraph of
  number-formats.md "Fraction digits of a second at a tie": it still names the register for the clock and fraction
  digits, which T-4332's cut now drives at every serial (T-4332 fidelity review, round 2).

## Acceptance criteria
- [ ] Those cells pass without moving another cell of the seven t4323 probe cases.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
