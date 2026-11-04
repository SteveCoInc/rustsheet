---
id: T-4162
title: "[$-_409]h A/P prints AM/PM in Excel (6 PM), A/P in ours (6 P): probes/t1924-locale-tag-extended-hex S!B22"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(0.75,"[$-_409]h A/P")` is "6 PM", as in Excel, and the cell has an owner until then.

## Context
probes/t1924-locale-tag-extended-hex S!B22 (excel-graph) is a known failure with no open owner (gap-triager,
2026-11-04): Excel "6 PM", ours "6 P". number-formats.md "More regional variants" records that `[$-F409]` and
`[$-_409]` (extended hex 0xF409) are English with en-US halves, so `A/P` becomes `AM/PM` like `[$-1009]`; T-3666
implemented 0xF409's names, but the `A/P` widening does not reach the extended-hex path (`locale_tag::extended_hex`,
`resolve_extended`, formatter/locale_tag.rs). Find why `[$-F409]h A/P` and `[$-_409]h A/P` differ in ours.

## Acceptance criteria
- [ ] probes/t1924-locale-tag-extended-hex S!B22 passes; no other cell of that case regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other undefined sublanguages (T-1875 owns their names).
