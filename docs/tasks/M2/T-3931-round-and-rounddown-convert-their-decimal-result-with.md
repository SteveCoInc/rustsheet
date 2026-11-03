---
id: T-3931
title: "ROUND and ROUNDDOWN convert their decimal result with Excel's x87 multiply by 10^-n"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3658]
---

## Goal
ROUND and ROUNDDOWN return the double Excel returns at every cell of probes-numeric/t3658-round-scaling, including the rare points where it is one ulp from the nearest double.

## Context
- numbers.md "Decimals reach a double through an x87 multiply by 10^-n" (T-3658): stage 2's decimal m x 10^-n
  becomes round53(round64(m x round64(10^-n))). ROUND first drops the trailing zeros of the kept digits (m = 491,
  n = 8 for `ROUND(4910.4E-9,9)`); ROUNDDOWN keeps them (m = 4910, n = 9).
- The engine returns the nearest double (`round_decimal` in crates/rustsheet-engine/src/decimal_round.rs).
  "Division rounds twice" already has a 64-bit helper (`arithmetic::extended_quotient`); reuse its approach.
- Failing cells: probes-numeric/t3658-round-scaling Calc!A1810, Calc!A1844:A1846, Calc!A1904:A1906, Calc!A1925:A1927, Calc!A1929, Hits!A1:A6, Hits!A8:A16, Hits!A18:A47, Hits!A49:A58, Hits!A60:A61, Hits!H1:H6, Hits!H8:H16, Hits!H18:H47, Hits!H49:H58, Hits!H60:H61, Hits!I1:I6, Hits!I8:I16, Hits!I18:I47, Hits!I49:I58, Hits!I60:I61, Hits!B1:B6, Hits!B8:B16, Hits!B19:B20, Hits!B22:B23, Hits!B25:B47, Hits!B52:B61
- Fresh sweep: probes-numeric/t3658-round-scaling-fresh Calc!A1:A56, Calc!A58:A126, Calc!A128:A151, Calc!A153:A155, Calc!A157:A160, Calc!A162:A164, Calc!A166:A167, Calc!A170:A172, Calc!A174, Calc!A176, Calc!A178:A181, Calc!B1:B152, Calc!B156, Calc!B161:B162, Calc!B164:B165, Calc!B167:B169, Calc!B173, Calc!B175, Calc!B177:B179, Calc!B181
- Also eval/accuracy/tables/ROUND/s1 Calc!A61 (Gate A; envelope holds it).

## Acceptance criteria
- [ ] Every ROUND and ROUNDDOWN cell of probes-numeric/t3658-round-scaling and t3658-round-scaling-fresh passes (baseline shrinks by those cells), with no regression in probes/round-family, the ROUND-family Gate A tables or gen-m5 numeric-fp.
- [ ] ROUNDUP keeps its own path ("ROUNDUP adds the unit in binary") unless evidence says otherwise.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Literal and VALUE parsing (T-3932).
- n outside 0..15 or kept digits beyond the probed range: keep the nearest double unless probed.
