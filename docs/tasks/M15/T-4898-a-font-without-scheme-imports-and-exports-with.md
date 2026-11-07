---
id: T-4898
title: "A font without scheme imports and exports with no scheme, not minor"
milestone: M15
lane: engine
status: blocked
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A font that has no `<scheme>` in the source keeps having none, so a non-theme font such as Arial is not saved
as following the theme's minor font.

## Context
Found by the excel-fidelity review of T-4623 (PR #3928).
- `import/styles.rs` reads a font without `<scheme>` as `FontScheme::Minor`, the default.
- `export/styles.rs` `font_xml` then writes `<scheme val="minor"/>`. Every regenerated save does this, and since
  T-4623 so do the fonts a patched save appends.
- ECMA-376 Part 1 §18.8.35: a font in the minor scheme follows the theme's minor font. Excel may therefore
  show such a font in the theme font instead of the named font. How Excel renders a name together with a
  scheme is not probed yet.

## Acceptance criteria
- [ ] Oracle evidence of how Excel renders a font whose `name` and `scheme` disagree, recorded in
      docs/excel-semantics/.
- [ ] A font without `<scheme>` round-trips without one (unit test and round-trip fixture).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Theme font resolution in the engine.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`.
