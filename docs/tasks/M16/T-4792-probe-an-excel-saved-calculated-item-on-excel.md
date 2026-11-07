---
id: T-4792
title: "Probe an Excel-saved calculated item on Excel for the web"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Know whether Excel for the web opens and computes a calculated item that Excel itself saved, and if it
does, commit probes of what it computes (formula over items per record, solve order, positional
references such as `Region[-1]`, several data fields, and the restrictions with grouping).

## Context
Excel for the web refuses every case-builder file whose pivot lists a calculated item `f="1"`
(FileCorruptTryRepair), and does not compute one written `f="0"` (pivots.md "Excel for the web refuses
calculated items", T-4130; T-4505 found no other route). Its UI cannot create calculated items, the
desktop backend is out of scope (decision 0266), and the surveyed corpora hold no Excel-saved file with
one (docs/research/pivot-tables-2026-11-04/oss-engines.md §5). So the one open route is an Excel-saved
workbook with a calculated item from a public corpus with a usable licence: run it through
`rustsheet-oracle fill -b excel-graph` as an xlsx case, and if it opens, diff its parts against the
case builder's `calculated_items` to find what the builder writes differently.

Until then every calculated-item rule in pivots.md is UNVERIFIED, and T-4175 has no evidence.

## Acceptance criteria
- [ ] An Excel-saved workbook with a calculated item is run on excel-graph, or the corpora searched
      are listed in pivots.md with the answer that none has one.
- [ ] If it opens: probes of the shapes in the goal, and pivots.md entries for them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop Excel (decision 0266). Engine changes (T-4175).
