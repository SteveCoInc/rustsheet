---
id: T-4192
title: "XLOOKUP with an error return_array and with wildcard match_mode under a binary search_mode: probe and match"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
XLOOKUP gives Excel's result, and Excel's error order, in two cases. One is a return_array that is an error, or a
range or array holding errors. The other is match_mode 2 (wildcard) combined with search_mode 2 or -2 (binary).
docs/excel-semantics/lookup.md records both as unprobed.

## Context
- Gap triage 2026-11-04, opus pass (`make eval` on main 5941bb95). Every visibly-off cell has an open owner, and the
  functions-lookup lane has no ready task. lookup.md "XLOOKUP: return_array may be wider than the lookup vector"
  ends: "Still unprobed: an error return_array, and wildcard match mode with a binary search mode."
- Ours, error return_array: `Model::xlookup_table` (crates/rustsheet-engine/src/functions/xlookup.rs) evaluates
  lookup_array first and then return_array, and returns an error-valued return_array at once. That is before the
  shape check and before the lookup value is searched. So `XLOOKUP(9,A1:A3,1/0,"nf")` (not found) is `#DIV/0!` in
  ours, not "nf". It is also unprobed whether an error lookup_value beats it (`XLOOKUP(NA(),A1:A3,1/0)`). Probe:
  `1/0`, `NA()` and `#REF!` as return_array, found and not found, with and without if_not_found. Also an error lookup
  array beside an error return array, to get their order. Also a return range that holds an error cell, at the
  matched row and at another row. Also an array return `{1;#N/A;3}` picked at the error and away from it. Also a 1x1
  error array `{#N/A}` over a one-cell lookup vector, which follows the scalar return rule (t3317-xlookup-scalar-return).
- Ours, wildcard with binary search: `binary_hit` returns `BinaryHit::Pattern` (`#VALUE!`) for match_mode 2 or 3
  under search_mode 2 or -2, whatever the lookup value. The only evidence is regex mode 3
  (probes/t2669-xlookup-regex-mode D22, D23). Probe match_mode 2 with search_mode 2 and -2 in several forms: a lookup
  value holding `*`, `?` or `~`; plain text with no wildcard character; a number; a boolean; a blank. Also probe the
  order against a not-found value, against if_not_found, and against a size mismatch (`#VALUE!` for both, so add
  ERROR.TYPE-free contrasts, such as a found key whose returned cell is a number). Also probe a lifted search_mode
  `{1;2}` with match_mode 2.
- Related open tasks, not duplicated: T-2865 (XLOOKUP if_not_found, match_mode and search_mode as empty text or an
  error: argument kinds, research). Leave those kinds to it.

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph, canary-verified, tag M6) holds the cells above.
- [ ] lookup.md records both rules with the cells, and the "Still unprobed" sentence is removed or narrowed.
- [ ] The engine matches every new cell, in `xlookup_table` / `binary_hit` (XMATCH shares `binary_hit`'s rules: if
      Excel's XMATCH differs, the PR probes XMATCH's wildcard + binary cells too and keeps one helper).
- [ ] probes/t2669-xlookup-regex-mode, t3317-xlookup-scalar-return, xlookup-array-return, xlookup-row-column-picks and
      t2746-xlookup-error-lookup-array-modes still pass. Eval known failures only shrink (`make eval-baseline`,
      0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- XLOOKUP argument kinds (T-2865), regex match mode (evidenced), and binary-search landing over unsorted data
  (evidenced in lookup.md "XMATCH / XLOOKUP binary search over runs of equal values").
