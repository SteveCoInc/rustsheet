---
id: T-4172
title: "rustsheet-eval roundtrip suite kind: fidelity items, structural XML differ, package validator, baselines"
milestone: M15
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4167]
spec: docs/specs/SPEC-0033-xlsx-round-trip-contract.md
---

## Goal
`rustsheet-eval roundtrip` runs round-trip suites (`kind = "roundtrip"`, under `eval/roundtrip/`):
- R0 runs on each case;
- the output is checked by the package validator of SPEC-0033 D6;
- part-level fidelity items are scored (D2 `part`: present, content type);
- known losses ratchet in an `eval/baseline/` file through the existing baseline machinery, so they can only shrink.

The vendored fixtures workbooks get the first such suite, which gates in `make eval-gates`.

## Context
Implements the first slice of SPEC-0033 (T-4167); see its D2, D6 and D8. The census prototype in
docs/research/xlsx-round-trip/ shows the shape. The task was split for size (work loop step 3):
- T-4509: element and attribute items, the P differ proper;
- T-4513: semantic items, which bring V7;
- T-4514: R1, the expected-delta rule and exceptions.

Reuse rather than add a second definition (directive 3):
- the snapshot reader's bounded part access (`xlsx_snapshot::Zip`, `read_part`, `relationships`, `resolve_target`);
- the suite manifest and case discovery (`suite.rs`);
- the cells-mode `Baseline` and its ratchet, with fidelity items in the item slot.

Round-trip suites live outside `eval/suites/` as Gate A's tables do (SPEC-0029), so no cell report or milestone
measure ever counts them.

Also carry PR #3526's review notes:
- add the round-trip kind and its location to docs/architecture/eval-harness.md;
- reword SPEC-0033 D2's "dimension-free sheet settings";
- record the baseline mode the spec's D8 calls `items` as the reused cells mode.

## Acceptance criteria
- [x] The fixtures round-trip suite runs through `rustsheet-eval roundtrip`, through `make eval`, and gates in
      `make eval-gates`. Its baseline is committed.
- [x] Unit tests: one crafted failing package per validator check V1–V6 and V8; a lost part is found as `lost`; an
      output-only part is found as `added`; part keys do not depend on part names.
- [x] `update-baseline` refuses a new loss in an unchanged case. This is the existing ratchet, tested through the
      round-trip results.
- [x] docs/architecture/eval-harness.md documents the kind, and docs/excel-semantics/xlsx-storage.md gains the
      "Round trips" section SPEC-0033 names.
- [x] `make check` green; reviewers approve.

## Out of scope
- Element and attribute items (T-4509).
- Semantic items and V7 (T-4513).
- R1 (T-4514).
- The external corpora (T-4173).
- XSD (T-4179).

## Outcome
`rustsheet-eval roundtrip` runs round-trip suites (`kind = "roundtrip"` under `eval/roundtrip/`; SPEC-0033 D8):
- R0 per case, in an isolated worker with a timeout;
- the package validator V1–V6 and V8 on the output (`roundtrip/package.rs`);
- `part` fidelity items keyed through the relationship graph, never by part name: sheets by name, other parts by
  ordinal in the parent's reference order;
- the ordinary cells-mode `Baseline` ratchet, with items in the item slot.

Reused rather than redefined:
- the snapshot reader's bounded `Zip`, `read_part`, `attr` and `relationships` (which gained `external`);
- `PartNames`, which gained `clashes` and a public `part_key`;
- `harness_options`, `panic_message` and the suite discovery.

Decision 0307 wires it into `make eval`, `make eval-gates` and `make eval-baseline`. The first suite,
`roundtrip-fixtures` (25 workbooks of eval/suites/fixtures), has 236 items and 39 known failures:
- 32 `added` parts (sharedStrings, sheetMetadata, rich-value parts the export always writes);
- 7 lost `externalLink` parts (T-4143).

No output fails the validator. Docs: eval-harness.md has a "Round-trip suites" section; xlsx-storage.md has a "Round
trips" section (N1–N12 wait on T-4503); SPEC-0033 D2 and D8 were reworded per PR #3526's review. No cell eval delta.

