--- `#This Row` cell evaluation after a column insert (T-4680).
id: T-5052
title: "A stored calculated-column formula mixing structured and A1 references follows column and row edits (T-4693 split)"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
--- `#This Row` cell evaluation after a column insert (T-4680).

## Goal
A stored `calculatedColumnFormula` that mixes a structured reference with A1 references follows row and column edits
like a plain one (its A1 part displaced, its structured part kept).

## Context
- Split from T-4693 (T-4274 review, excel-fidelity MINOR): `Model::displaced_column_formulas` (calculated_columns.rs)
  keeps any text containing `[` unchanged, so `T[[#This Row],[Col]]*$B$2` stays so through a column insert at A where
  the first-data-cell rule predicts `$C$2`. The moved cell keeps `=[@Col]*$B$2` too (a cell-side gap probably shared
  with T-4680).
- Unprobed: only desktop Excel can author the evidence (a table edited by insert_column A and saved; the cached
  `table1.xml` `calculatedColumnFormula`, as T-4692's probes/table-edits cases do); excel-graph cannot.
- Optional (security): a stored formula that does not parse could print another text through
  `set_displaced_column_formulas`; keep the old text when the parse is an error node.

## Acceptance criteria
- [ ] A desktop-Excel probe of a mixed column formula through insert_column A (and a row insert); the engine stores Excel's text; unit test.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `#This Row` cell evaluation after a column insert (T-4680).
