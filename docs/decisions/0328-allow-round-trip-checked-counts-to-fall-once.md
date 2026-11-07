# 0328 — Allow round-trip checked counts to fall once where value-less cells stop being read as the empty string

- **Status:** accepted (2026-11-06, T-4148)
- **Context:**
  - Excel reads a constant cell that stores no value as blank whatever its `t` (`<c t="str"/>`, `<c t="inlineStr"/>`
    with no `<is>`, `<c t="s"/>`, ...; fixtures/valueless-cell-forms, excel-graph, docs/excel-semantics/xlsx-storage.md
    "A cell that stores no value is blank"). The importer and `rustsheet-eval snapshot` read `t="str"` and
    `t="inlineStr"` without a value as the empty string, so the round-trip scorer (SPEC-0033) counted each such cell as
    an item it compared. T-4148 fixes both readers.
  - SPEC-0033's ratchet calls any fall of a case's checked count a regression, to catch an output that became
    unreadable, and `roundtrip --update-baseline` had no override (the spec already says `--allow-regressions` needs a
    decision record). With the fix the count falls by exactly the value-less cells of each source: 3 poi cases
    (bug66675 32,038 to 28,434, the 3,604 covered `t="str"` cells the 2026-11-04 census reported lost), 11 corpus-bench
    cases (212-45 falls 213, its count of formula-less `t="str"`/`inlineStr` cells with no value, checked by hand); Calipers is
    unchanged. The new fixtures cases of the same PR are recorded as new cases. No failing item is added and no case stops running.
- **Decision:** `rustsheet-eval roundtrip --update-baseline` gains `--allow-regressions` (refused without
  `--update-baseline`), and the T-4148 PR runs it once to record those checked counts. Any later use needs its own
  decision record, as for the cell suites.
- **Consequences:** `eval/baseline/roundtrip-{poi,corpus-bench}.txt` lower the `C` lines of the cases above and change
  nothing else (`roundtrip-fixtures.txt` only gains the new cases); reviewers confirm the baseline diff has no other line. `make eval-baseline` keeps calling
  `roundtrip --update-baseline` without the flag, so the ratchet is unchanged for every other run.
