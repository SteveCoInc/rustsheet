---
id: T-4332
title: "TEXT fraction digits of a second at a tie for a serial below 1 (d=0) follow no half-step model yet"
milestone: M10
lane: functions-text
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
For a serial below 1, TEXT's fraction digits of a second at a rounding tie match Excel (`TEXT(0.015/86400,"s.00")` is "0.02", `TEXT(21.2685/86400,"[h]:mm:ss.000")` "0:00:21.268").

## Context
Found by T-4323. The half-step register (`half_step_split`, formatter/elapsed.rs) fits every tie at serials of 1 or more but only 98 of 150 below 1 (the engine's earlier rounding got 110). Visibly-off cells (fraction digit one low unless noted): probes/t4323-fraction-ties A92, A96:A98, A123, A125, A126, A128, A130; probes/t4323-fraction-sweep-2 A5, A7, A13, A15:A17, A20, A23, A28, A33, A36, A42, A43, A45, A49:A51, A54, A55, A60, A61, A64, A70, A77, A80, A82, A87, A88, A93, A94, A100, A101, A103, A105, A107, A109, A110, A116:A118, A120; probes/t4323-fraction-sweep-3 A25, and A2 (a serial of 30 whose register is exactly 394741 yet Excel shows ".0"); probes/t4323-fraction-holdout A5, A56, and A26 (a serial of 3, `[s].0` shows ".1", ours ".2"). The engine's rounding before T-4323 matched 36 of these, all below serial 1 (the cost T-4323 accepted for one model over the whole domain): t4323-fraction-ties A97, A98, A123, A125, A126, A130; t4323-fraction-sweep-2 A5, A7, A13, A15, A17, A20, A23, A33, A43, A45, A49, A50, A54, A55, A60, A61, A77, A82, A87, A93, A94, A100, A101, A105, A107, A109, A110, A116, A117; t4323-fraction-holdout A56. Candidates already refuted are listed in docs/excel-semantics/number-formats.md "Fraction digits of a second at a tie" and docs/research/t4323 (operation orders, 64-bit intermediates, 15-digit snap, 1+2^-52 snap). A structural model: owes the R4 hold-out and a fresh sweep.

## Acceptance criteria
- [ ] Those cells pass without moving another cell of the five t4323 probe cases; at the least, beat the earlier rounding's 114 of 156 cells below serial 1 across the five cases (`docs/research/t4323/fu.py`).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
