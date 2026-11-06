---
id: T-4693
title: "A stored calculated-column formula mixing structured and A1 references follows edits; one default-locale parser helper (T-4274 review follow-ups)"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-4274]
---

## Goal
A stored `calculatedColumnFormula` that mixes a structured reference with A1 references follows row and column edits
like a plain one (its A1 part displaced, its structured part kept), and the parser's switch to the default locale is
one helper.

## Context
- T-4274 review (PR #3738; excel-fidelity and design, MINOR):
  - `Model::displaced_column_formulas` (calculated_columns.rs) keeps any text containing `[` unchanged, so
    `T[[#This Row],[Col]]*$B$2` stays so through a column insert at A where the first-data-cell rule predicts `$C$2`.
    The moved cell keeps `=[@Col]*$B$2` too, a cell-side gap probably shared with T-4680. Unprobed in Excel: probe a
    mixed column formula through insert_column A (excel-probe skill) before changing it.
  - The save / switch to `get_default_locale()` and `get_default_language()` / restore sequence now appears in
    `displaced_column_formulas`, `set_displaced_column_formulas`, `Model::displace_cf_ranges` (actions.rs), model.rs and
    new_empty.rs. One closure-taking `Model` helper (directive 3).
  - Optional (security): a stored formula that does not parse could print another text through
    `set_displaced_column_formulas`; keep the old text when the parse is an error node.

## Acceptance criteria
- [ ] Probe of a mixed column formula through insert_column A; the engine stores Excel's text; unit test.
- [ ] One default-locale helper used at every site above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `#This Row` cell evaluation after a column insert (T-4680).

## Blocked because
`Model::displaced_column_formulas` and its callers come from T-4274 (PR #3738), which has not merged to `main`,
so the code this task changes does not exist there yet. Depends on T-4274; resume once it lands.
