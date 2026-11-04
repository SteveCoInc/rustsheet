---
id: T-4170
title: "Probe pivot grouping: numeric, date and manual groups"
milestone: M15
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4130]
---

## Goal
Evidence for numeric group labels (integer and fractional data, the last bin, `<start`/`>end`, blanks), date groups at every level (seconds to years, days with an interval, Feb 29, `<m/d/yyyy` bounds, Qtr labels), multi-level date groups, manual text groups (`Group1`), and shared group fields.

## Context
Spike run 6; oss-engines.md §0.3 (Excel's labels from corpus files); libreoffice.md §2. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Probe cases with sidecars; pivots.md entries.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
