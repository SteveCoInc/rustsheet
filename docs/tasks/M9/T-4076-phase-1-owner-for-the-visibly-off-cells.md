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

## Acceptance criteria
- [ ] Each cell above is within half a unit of Excel's 15th significant digit, or moved to an open, non-deferred task
      that owns it, or listed as Phase 1b evidence (both sides within 1e-9 of the 50-digit math; then it is not
      counted, per decision 0188 R1(c)).
- [ ] `make phase1-exit` no longer reports deferred-only owners for these cases.
- [ ] `make check-fast` green; reviewers approve (excel-fidelity included).

## Out of scope
- Changing `defer:` on T-2221 or T-3199 themselves.
