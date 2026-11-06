---
id: T-4185
title: "Probe structural edits that meet a pivot (T-2402) through sheet ops"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4122]
---

## Goal
Evidence for which row/column inserts, deletes and moves Excel refuses or applies around a pivot, recorded in web.md, unblocking T-2402's engine follow-up.

## Context
T-2402; sheet_ops (insert_row etc.) in the harness. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Probes with sidecars; T-2402 updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
