---
id: T-4627
title: "Probe pivot field-name collisions, empty header cells and the Field<n> number"
milestone: M16
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
pivots.md settles the open point of "Pivot field names for duplicate and blank source headers": how Excel resolves a suffixed name (`Sales2`) or a `Field<n>` name that collides with a real header.

## Context
T-4585 (probes/t4585-pivot-field-names) shows `Sales2`, `Sales3`, `region2` and `Field4`. Cases:
- a source with `Sales`, `Sales2` and a second `Sales`;
- an empty-text header beside a real `Field4` header;
- a source starting at column B (B1:G3) with the empty-text header in its 4th column, sheet column E. `Field4` or `Field5` decides between position and sheet column;
- a truly empty header cell. The case builder refuses absent headers (pivot_definition.py), so this needs a builder option or a raw xlsx case;
- a constant duplicate header (`Sales` typed twice), also refused by the builder today.
- a whitespace-only header (`=" "`): kept as a name or `Field<n>`? T-4141 keeps it;
- non-ASCII case variants (`é`/`É`, `ß`/`SS`) in headers and items: T-4141 folds them with `fold_case`;
- a record cell holding a formula that returns empty text: T-4141 reads it as a text item `""`, apart from a blank.

Use the same formula-header trick: the case builder refuses duplicate or blank headers, so `=""` and `="Sales"` get past it. T-4141 implements the rule.

## Acceptance criteria
- [x] Filled probes for each case; the pivots.md entry's UNVERIFIED part closed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine refresh (T-4141).

## Outcome
probes/t4627-pivot-field-name-collisions (excel-graph, 2026-11-06) covers all eight cases, one
builder-v2 pivot per source sheet; pivots.md "Pivot field names for duplicate and blank source
headers" is rewritten from its sidecar's cache_fields. Excel names text headers first: a taken name
takes the first free `<header><k>` from 2 (`Sales`, `Sales2`, `="Sales"` → `Sales3`; `Sales`,
`="Sales"`, `Sales2` → `Sales22`); a blank (empty cell or `=""`) is `Field<n>` by source position
(Off: `Field4` at sheet column E), named after every text header (`X`, `=""`, `Field4` → `Field42`,
`Field4`); constant repeats rename like formula ones; whitespace is kept (`" "`, `" 2"`); names
fold `ß` with `SS` though items do not; a `=""` record is a text item apart from `(blank)`.
`field_names` in pivot/refresh.rs did not match (it gave `Sales2` twice and `Field4` twice), so it
now checks every taken name, names blanks last and folds names with `name_key`; a unit test pins
each probed source. The case builder gained `cache_names` (pivot_definition.py, with a pytest) for
the empty-cell and constant-repeat sources. The case's pivot cells are owned by T-4149 (rendering).
Still open, filed as T-4640: number, boolean and error headers, other multi-character folds, a
blank's name meeting a later duplicate's suffix, and two code pointers moved from this task.
