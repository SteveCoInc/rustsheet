---
id: T-4561
title: "Source-preserving export 3: metadata for rich values and dynamic arrays, and the calcChain rule, in patched packages (SPEC-0034 D3.4, D3.5, E2)"
milestone: M15
lane: engine
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-4183, T-4560]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
A patched package can hold dynamic-array anchors and rich-value errors, and can add or remove formula cells:
- a source with no `metadata.xml` of its own takes the export's metadata parts;
- calcChain is dropped, with its relationship and content-type override, when the set of formula cells changed.

This covers SPEC-0034 D3.4, D3.5 and E2. The styles half (D3.3) is T-4623.

## Context
SPEC-0034; T-4560 left these as fallbacks. The three external-link fixtures fall back because a recalculated cell
becomes a rich error, and those sources have no `metadata.xml`.

## Acceptance criteria
- [x] A source without metadata parts takes the export's (`xl/metadata.xml`, and `xl/richData/*` for rich errors),
      with a relationship and content-type override for each. Its cells use the export's `vm`/`cm` (integration
      test).
- [x] A source with its own metadata keeps the T-4560 rules: same-kind rich errors keep the source `vm`, and
      anything else falls back.
- [x] Adding or removing a formula cell drops calcChain, with its relationship and override, and changing only
      values keeps it (integration tests).
- [x] The round-trip baselines are rerecorded.
- [x] `make check` green; reviewers approve.

## Out of scope
- New styles (T-4623).
- Structural edits.

## Outcome
- `export/patch/package.rs`:
  - `add_parts` adds a relationship (fresh `rIdP<n>` ids) and an override per new part;
  - `remove_part` cuts calcChain's `<Relationship>` and `<Override>`;
  - `rewrite_zip` drops removed entries and appends added ones.
- `export/patch/strings.rs` holds the shared-string helpers, which keeps patch.rs at 900 lines.
- `try_patch`:
  - A source with none of `xl/metadata.xml` and the two rich-value parts is on the export's layout. Its whole-cell
    rewrites and value patches carry the export's `vm`/`cm`.
  - When any patched sheet then holds `vm` or `cm`, the export's `metadata.xml` (plus the rich-value parts when a
    rich error exists) is added.
  - A changed set of formula cells drops calcChain instead of falling back.
- Round-trip kept rates (weighted, SPEC-0033 D8), with T-4560 → with T-4561:

  | suite | failing cases | kept |
  |---|---|---|
  | roundtrip-calipers | 53 → 53 | 99.986% → 99.986% |
  | roundtrip-poi | 21 → 21 (+14 not scored) | 99.993% → 99.997% |
  | roundtrip-corpus-bench | 20 → 20 | 99.997% → 99.99999% |
  | roundtrip-fixtures | 3 → 3 | 95.75% → 100.00% |

- The fixtures' three failing cases now hold only `added` parts: the metadata parts their new rich errors need. No
  source item is lost.
