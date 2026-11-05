---
id: T-4076
title: "Phase 1 owner for the visibly-off cells only deferred tasks own (T-4017 F.DIST.RT sweep and others)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Every visibly-off cell has an open owner that Phase 1 can pick. The cells below are visibly off, and their only owners
are tasks with `defer: phase-1b`, which `make next-task` never offers before Phase 1 exits. That fails the Phase 1
exit criterion "owners" (decision 0187: "412 visibly-off cells with no open owner (412 owned only by a deferred
task)" on `make phase1-exit` at 30dd03e3, 2026-11-04). This task owns them so the criterion can pass. The work may
split them into narrower fix tasks: move each cell to its fix task and remove it here.

## Context
Owner request (2026-11-04): clear every known Phase 1 blocker quickly. Most of these cells (332 F.DIST.RT, 1 FDIST) come
from T-4017's new sweep (`probes-numeric/t4017-f-huge-df2-above-mean-*`, F.DIST.RT above the mean at a huge df2). It
landed with owners T-2221 and T-3199, both deferred. The rest are older cells whose fix task was deferred. Existing
non-deferred F tasks: T-3198 (F tails and densities, claimed), T-3183, T-4026. Check whether one of them already
covers a cell before splitting.

Cells (from `make unowned-off ALL=1` on 30dd03e3, the deferred-only ones):

- probes-numeric/t4017-f-huge-df2-above-mean-1e7.case.json (72; F.DIST.RT, FDIST): Calc!A5, Calc!A7, Calc!A9, Calc!A47, Calc!A53, Calc!A59, Calc!A63, Calc!A69, Calc!A75, Calc!A77, Calc!A79, Calc!A85, Calc!A87, Calc!A91, Calc!A95, Calc!A99, Calc!A101, Calc!A105, Calc!A109, Calc!A111, Calc!A113, Calc!A115, Calc!A117, Calc!A119, Calc!A121, Calc!A123, Calc!A125, Calc!A127, Calc!A129, Calc!A131, Calc!A133, Calc!A135, Calc!A137, Calc!A139, Calc!A141, Calc!A143, Calc!A145, Calc!A147, Calc!A149, Calc!A151, Calc!A153, Calc!A155, Calc!A157, Calc!A159, Calc!A161, Calc!A163, Calc!A165, Calc!A167, Calc!A169, Calc!A171, Calc!A173, Calc!A177, Calc!A179, Calc!A181, Calc!A183, Calc!A185, Calc!A187, Calc!A189, Calc!A193, Calc!A195, Calc!A197, Calc!A199, Calc!A201, Calc!A203, Calc!A205, Calc!A209, Calc!A211, Calc!A213, Calc!A215, Calc!A217, Calc!A219, Calc!A225
- probes-numeric/t4017-f-huge-df2-above-mean-3e8.case.json (67; F.DIST.RT): Calc!A5, Calc!A35, Calc!A43, Calc!A51, Calc!A53, Calc!A55, Calc!A59, Calc!A63, Calc!A75, Calc!A77, Calc!A79, Calc!A85, Calc!A87, Calc!A89, Calc!A91, Calc!A93, Calc!A95, Calc!A101, Calc!A103, Calc!A105, Calc!A107, Calc!A111, Calc!A113, Calc!A115, Calc!A117, Calc!A119, Calc!A121, Calc!A125, Calc!A127, Calc!A129, Calc!A131, Calc!A133, Calc!A135, Calc!A139, Calc!A141, Calc!A143, Calc!A145, Calc!A147, Calc!A149, Calc!A151, Calc!A153, Calc!A155, Calc!A157, Calc!A159, Calc!A163, Calc!A165, Calc!A167, Calc!A169, Calc!A173, Calc!A177, Calc!A179, Calc!A181, Calc!A183, Calc!A185, Calc!A189, Calc!A193, Calc!A195, Calc!A199, Calc!A201, Calc!A203, Calc!A205, Calc!A209, Calc!A211, Calc!A213, Calc!A215, Calc!A217, Calc!A219
- probes-numeric/t4017-f-huge-df2-above-mean-1e9.case.json (66; F.DIST.RT): Calc!A11, Calc!A35, Calc!A39, Calc!A47, Calc!A53, Calc!A55, Calc!A59, Calc!A61, Calc!A63, Calc!A67, Calc!A77, Calc!A79, Calc!A83, Calc!A87, Calc!A89, Calc!A91, Calc!A95, Calc!A99, Calc!A101, Calc!A105, Calc!A107, Calc!A109, Calc!A111, Calc!A115, Calc!A119, Calc!A121, Calc!A123, Calc!A127, Calc!A129, Calc!A131, Calc!A133, Calc!A135, Calc!A137, Calc!A139, Calc!A141, Calc!A143, Calc!A145, Calc!A147, Calc!A149, Calc!A153, Calc!A155, Calc!A157, Calc!A159, Calc!A161, Calc!A163, Calc!A165, Calc!A167, Calc!A169, Calc!A173, Calc!A177, Calc!A179, Calc!A181, Calc!A183, Calc!A185, Calc!A187, Calc!A189, Calc!A193, Calc!A195, Calc!A197, Calc!A201, Calc!A203, Calc!A205, Calc!A209, Calc!A211, Calc!A217, Calc!A219
- probes-numeric/t4017-f-huge-df2-above-mean-1e8.case.json (64; F.DIST.RT): Calc!A5, Calc!A7, Calc!A13, Calc!A29, Calc!A47, Calc!A51, Calc!A59, Calc!A61, Calc!A63, Calc!A69, Calc!A75, Calc!A77, Calc!A79, Calc!A83, Calc!A91, Calc!A99, Calc!A101, Calc!A105, Calc!A107, Calc!A109, Calc!A111, Calc!A117, Calc!A119, Calc!A123, Calc!A125, Calc!A127, Calc!A131, Calc!A133, Calc!A135, Calc!A139, Calc!A141, Calc!A143, Calc!A147, Calc!A149, Calc!A153, Calc!A155, Calc!A157, Calc!A159, Calc!A161, Calc!A163, Calc!A165, Calc!A167, Calc!A169, Calc!A171, Calc!A173, Calc!A177, Calc!A181, Calc!A183, Calc!A185, Calc!A187, Calc!A189, Calc!A193, Calc!A195, Calc!A197, Calc!A199, Calc!A201, Calc!A203, Calc!A205, Calc!A209, Calc!A211, Calc!A213, Calc!A215, Calc!A217, Calc!A219
- probes-numeric/t4017-f-huge-df2-above-mean-3e7.case.json (63; F.DIST.RT): Calc!A11, Calc!A13, Calc!A29, Calc!A35, Calc!A43, Calc!A45, Calc!A47, Calc!A55, Calc!A59, Calc!A63, Calc!A75, Calc!A77, Calc!A79, Calc!A83, Calc!A87, Calc!A91, Calc!A95, Calc!A101, Calc!A105, Calc!A107, Calc!A109, Calc!A111, Calc!A123, Calc!A125, Calc!A127, Calc!A129, Calc!A131, Calc!A133, Calc!A137, Calc!A139, Calc!A141, Calc!A143, Calc!A147, Calc!A153, Calc!A155, Calc!A157, Calc!A159, Calc!A161, Calc!A163, Calc!A167, Calc!A169, Calc!A171, Calc!A173, Calc!A177, Calc!A179, Calc!A181, Calc!A183, Calc!A185, Calc!A187, Calc!A189, Calc!A193, Calc!A195, Calc!A197, Calc!A199, Calc!A201, Calc!A203, Calc!A205, Calc!A209, Calc!A211, Calc!A213, Calc!A215, Calc!A217, Calc!A219
- probes-numeric/t3570-r5-bpser-lg-aux.case.json (45; unattributed): Calc!E130, Calc!E131, Calc!E132, Calc!E148, Calc!E149, Calc!E150, Calc!E151, Calc!E152, Calc!E153, Calc!E154, Calc!E155, Calc!E157, Calc!E158, Calc!E159, Calc!E168, Calc!E298, Calc!E299, Calc!E300, Calc!E316, Calc!E317, Calc!E318, Calc!E319, Calc!E320, Calc!E321, Calc!E322, Calc!E323, Calc!E325, Calc!E326, Calc!E327, Calc!E336, Calc!E466, Calc!E467, Calc!E468, Calc!E484, Calc!E485, Calc!E486, Calc!E487, Calc!E488, Calc!E489, Calc!E490, Calc!E491, Calc!E493, Calc!E494, Calc!E495, Calc!E504
- probes-numeric/t2220-f-dist-sweep-rt.case.json (15; F.DIST.RT): Calc!A594, Calc!A669, Calc!A680, Calc!A800, Calc!A890, Calc!A900, Calc!A954, Calc!A960, Calc!A978, Calc!A984, Calc!A990, Calc!A994, Calc!A997, Calc!A999, Calc!A1000
- probes/t3638-subnormal-upper-tail.case.json (7; CHIDIST, CHISQ.DIST.RT, POISSON): Sheet1!A17, Sheet1!A18, Sheet1!A23, Sheet1!A24, Sheet1!A36, Sheet1!A39, Sheet1!A42
- probes/t3914-imsech-sign-sweep.case.json (4; unattributed): Calc!A11, Calc!B11, Calc!D21, Calc!C21
- probes-numeric/m9-legacy-fdist.case.json (2; F.DIST.RT, FDIST): Calc!A55, Calc!A56
- probes/t3912-imcsch-band-edges.case.json (2; unattributed): Calc!A25, Calc!B25
- probes-numeric/m9-f-dist-rt.case.json (1; F.DIST.RT): Calc!A16
- probes-numeric/t1538-beta-dist-at-excel-inv.case.json (1; F.DIST.RT): Calc!A15
- probes-numeric/t3721-im-hyperbolic-small-x-fresh-sweep.case.json (1; IMSECH): S!C386
- probes/t3881-imcsch-flip-start-negative.case.json (1; unattributed): Calc!A3
- probes/t3883-im-overflow-reciprocal-sweep.case.json (1; unattributed): Calc!A101

Added by gap triage 2026-11-04 (`make eval` and `make phase1-exit` on origin/main c3d019be8, after T-4056's 13-digit
bar). Three more cells have only deferred owners (T-2221, T-3472, T-3952, T-3954, T-4016). Each is a family call with
a non-family argument function, so it keeps the 15-digit bar (SPEC-0030 "Which cells qualify"). All three are
first wrong at the 14th or 15th digit (decision 0267 Tier C/D):
- gen-m9/2801c1c11c21e134.case.json (1; unattributed, F.DIST + REDUCE): Calc!A219, Excel 4.881498042869914e-39, ours ...936e-39
- gen-m9/285d87cea1aaa73c.case.json (1; unattributed, FDIST + MINUTE): Calc!A34, Excel 6.448979666396885e-18, ours ...879e-18
- gen-m9/8f2ac25945af87da.case.json (1; unattributed, FDIST + DAYS360): Calc!A92, Excel 4.5060151487344243e-45, ours ...392e-45

Added by gap triage 2026-11-04 (`make eval`, `make exit-owners` and `make unowned-off ALL=1` on origin/main
b450050f8). Two GAMMA.DIST cells had no open owner and showed in the M9 exit row (`rate:GAMMA.DIST`, `probes`). Both
sides are within 1e-9 of the math, so they are Phase 1b evidence under decision 0188 R1(c). They are first wrong at
the 15th (A145) and 14th (A146) digit, decision 0267 Tier C/D. The fix is the deferred T-4216:
- probes/t2982-gamma-inv-huge-shape.case.json (2; GAMMA.DIST): S!A145, Excel 2.8546421399586357e-07, ours ...6256e-07; S!A146, Excel 2.8561234160188864e-07, ours ...894e-07

Added by gap triage 2026-11-05 (`make eval` and `make unowned-off ALL=1` on origin/main b6cf9465). One more
F.DIST.RT cell of T-4017's huge-df2 sweep, in the 1e10 case the list above lacks, has only deferred owners; it is first
wrong at the 13th digit (relative 1.7e-13):
- probes-numeric/t4017-f-huge-df2-above-mean-1e10.case.json (1; F.DIST.RT): Calc!A203, `F.DIST.RT(10,60,1E10)`, Excel 4.421993244304403e-90, ours 4.421993244303664e-90

The 28 BETA.DIST b = 40 cells of probes-numeric/t4080-beta-num-band-edge that only deferred T-3209 owned went to
T-4214 (decision 0267 Tier A), not here.

## Acceptance criteria
- [ ] Each cell above is within half a unit of Excel's 15th significant digit, or moved to an open, non-deferred task
      that owns it, or listed as Phase 1b evidence (both sides within 1e-9 of the 50-digit math; then it is not
      counted, per decision 0188 R1(c)).
- [ ] `make phase1-exit` no longer reports deferred-only owners for these cases.
- [ ] `make check-fast` green; reviewers approve (excel-fidelity included).

## Out of scope
- Changing `defer:` on T-2221 or T-3199 themselves.
