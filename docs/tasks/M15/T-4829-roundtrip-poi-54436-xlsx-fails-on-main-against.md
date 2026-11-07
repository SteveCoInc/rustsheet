---
id: T-4829
title: "roundtrip-poi/54436.xlsx fails on main against its baseline (78.79% kept, item 296)"
milestone: M15
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`rustsheet-eval roundtrip --check` passes for roundtrip-poi/54436.xlsx again: either the export keeps what the baseline
records, or the case's new failure is explained and ratcheted under the roundtrip rules (SPEC-0033 "Ratchet").

## Context
- Found by T-0950 (2026-11-07). After `rustsheet-eval fetch --suite poi`, `make eval-baseline` refuses at its roundtrip
  step: `REGRESSION roundtrip-poi/54436.xlsx * 296 2cd19df138a87ec5`. eval/baseline/roundtrip-poi.txt has
  `C roundtrip-poi/54436.xlsx 995` and no F line for it.
- The same failure happens on `origin/main` (65eb15be) without T-0950's change: `rustsheet-eval roundtrip --suite
  roundtrip-poi --filter 54436` gives 995 items, 78.79% kept, 1 failing. So a change merged earlier caused it. The PR
  gate does not fetch poi; only nightly.yml does, so no PR check caught it.
- Start with `git log -- crates/rustsheet-xlsx` since roundtrip-poi.txt last changed, and the case's row in
  eval/runs/latest/roundtrip.md.

## Acceptance criteria
- [ ] `rustsheet-eval roundtrip --suite roundtrip-poi --check` reports no regression for 54436.xlsx, and the cause is in
      the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other roundtrip-poi cases.
