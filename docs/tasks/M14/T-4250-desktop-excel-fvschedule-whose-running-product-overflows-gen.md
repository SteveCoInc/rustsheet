---
id: T-4250
title: "Desktop Excel: FVSCHEDULE whose running product overflows (gen-m9 89f0aee953a55a45 A349 is 0 on excel-graph, which cannot open a small workbook holding the call)"
milestone: M14
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Desktop Excel's value for FVSCHEDULE where the running product leaves the double range is known, so the engine
can match it.

## Context
Found by T-3312 (2026-11-04). gen-m9/89f0aee953a55a45 Calc!A349 `=FVSCHEDULE(5916,{-1E+300,1E+300})` is 0 on
excel-graph (filled 2026-10-23, and the same on a refill of the whole case on 2026-11-04); the engine is `#DIV/0!`
(its overflow rule in `fn_fvschedule`, crates/rustsheet-engine/src/functions/financial/mod.rs). The product is about
-5.9E+603, so 0 looks like a non-finite result the Graph value read cannot carry, not a computed value. It cannot be
probed further on excel-graph: a small workbook holding the same formula, alone, with 58 other formulas, with the
case's own Data sheet, or with the schedule in cells, fails to open (`createSession` 501 FileCorruptTryRepair), as
does any FVSCHEDULE whose product overflows (`FVSCHEDULE(1,{1E+300,1E+300})`). So only desktop Excel can answer
(decision 0039).
Probe on desktop Excel: `FVSCHEDULE(5916,{-1E+300,1E+300})`, `FVSCHEDULE(1,{1E+300,1E+300})`,
`FVSCHEDULE(1,{-1E+300,-1E+300})`, `FVSCHEDULE(1E+300,{1E+10})`, `FVSCHEDULE(1,{1E+300,1E+300,0})` (inf times 0),
`FVSCHEDULE(1,{1E+300,1E+300,1E-300})` (back in range after an overflow), and `FVSCHEDULE(1E+308,{1})`.
Cells:
- gen-m9/89f0aee953a55a45.case.json (1): Calc!A349

## Acceptance criteria
- [ ] Desktop values for the calls above are recorded in docs/excel-semantics/financial.md, and the engine matches them
      (or a follow-up M9 task is filed to implement them).
- [ ] `make check` green; reviewers approve.

## Out of scope
- FVSCHEDULE in range.
