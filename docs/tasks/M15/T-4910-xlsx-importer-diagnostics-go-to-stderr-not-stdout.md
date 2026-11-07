---
id: T-4910
title: "xlsx importer diagnostics go to stderr, not stdout"
milestone: M15
lane: engine
status: blocked
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The xlsx importer never writes to stdout, so a CLI's machine-readable output, such as
`rustsheet-eval package-diff --json`, stays parseable.

## Context
Found by the T-4890 probe:
- `package-diff --json` of calipers roundtrip/rich_text began with `Unexpected feature XmlNode { name: "vertAlign" ... }`.
- That line comes from `println!` diagnostics inherited from IronCalc:
  - `import/styles.rs` (unexpected font feature, unexpected pattern);
  - `import/util.rs` (unexpected color node);
  - `import/worksheets.rs` (unexpected cell type).
- `import/pivots.rs` already reports to stderr with `eprintln!`.

## Acceptance criteria
- [ ] No `println!` in `crates/rustsheet-xlsx/src` outside tests. The diagnostics go to stderr or are dropped, and
      a ratchet or lint keeps them out.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A structured warnings API for import.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`.
