---
id: T-4754
title: "Probe the operation order of index show-values-as with many order-discriminating cells"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4163]
---

## Goal
Excel for the web evidence for the floating-point operation order of the `index` show-values-as mode,
from many cells whose value depends on the order: `v*G/R/C`, `(v*G)/(R*C)`, `v/R*G/C`, `(v/R)/(C/G)` and
the other plausible orders each give a different double on at least several of the cells.

## Context
T-4163's engine (`crates/rustsheet-engine/src/pivot/show_as.rs`, `index`) computes
value x grand total / row total / column total, left to right. Among the probed index cells only
probes/t4129-pivot-show-as IX!B5 tells the orders apart (v=18, G=63, R=30, C=38: Excel
0.994736842105263; the other orders are 1 ulp off), and one ulp is below the R4.4 web-versus-desktop
floor. So the order is UNVERIFIED (docs/excel-semantics/pivots.md "Show-values-as edge cases (T-4161)",
**index**). A probe needs data chosen so that the candidate orders round differently on many cells,
including subtotal and grand-total cells, with the orders checked against the data before the oracle run.

## Acceptance criteria
- [ ] A probe case with a sidecar whose index cells tell the candidate orders apart on many cells.
- [ ] pivots.md records the order Excel uses, VERIFIED or re-scoped; the engine follows it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other show-values-as modes (T-4750).
