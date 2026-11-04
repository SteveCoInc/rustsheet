---
id: T-3971
title: "Probe the collation neighbours T-3917 placed without evidence: Math Operators, Hebrew final letters, the place of ™, the reordered unprobed blocks"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3917]
---

## Goal
Oracle evidence (excel-graph) for where Excel sorts the characters T-3917 placed by inference next to the
probed ones, so the UNVERIFIED notes in docs/excel-semantics/coercion.md "Scripts and symbol blocks beyond
T-3276 (T-3559)" become VERIFIED or turn into an engine task.

## Context
- T-3917 (PR #3018) placed these by inference; its Excel-fidelity review (MINOR) asked for a probe:
  - Math Operators: only `≠ ∑ ∫ ≤` were ranked (t3559 ranks 16 to 19); the rest of U+2200..U+22FF (for example
    `≥` U+2265, `∀` U+2200, `∈` U+2208, `≈` U+2248) still weigh after the letters (collation.rs `other`), so `≤` and
    `≥` now sort far apart.
  - Hebrew final letters: only `ץ = צ` was probed (t3559 G14, B97); `ך ם ן ף` against `כ מ נ פ` keep code-point order.
  - `™`: above `"tn"` and below `"z"` (G1:G5); T-3917 placed it after `"tz"`, before `"u"`. Rank it against `"to"`,
    `"tz"`, `"u"`, `"tm2"`.
  - `℃` and `№`: placed as `c` and `no` with the superscript variant; rank against `"c"`, `"ca"`, `"no"`, `"noa"`.
  - The blocks T-3917 moved inside U+0E00..U+18AF with no evidence (scripts.rs `BLOCKS`): Hangul Jamo U+1100,
    Cherokee U+13A0, Canadian syllabics U+1400, Ogham U+1680, Runic U+16A0, Tagalog U+1700, against Ethiopic `ፚ` and
    Hangul `가`.
  - Geometric shapes, arrows, box drawing and dingbats were ranked at two points each; add one interior point each.
- Layout: copy probes/t3559-collation-unprobed-blocks (Data!A strings; Calc!A count less, B count equal, C exact MATCH,
  D XMATCH, E COUNTIF) with G-column pairs. Use the `excel-probe` skill.
- Design notes from T-3917's review (for the engine task, if one follows): derive `scripts::SYMBOLS` from the
  `symbol` tables instead of restating their sizes; reword the `GEORGIAN` doc.

## Acceptance criteria
- [ ] A probes case covering every item above, filled by excel-graph.
- [ ] coercion.md (and lookup.md for MATCH) records the results: VERIFIED where the engine already agrees, otherwise
      a follow-up engine task that owns the failing cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a follow-up task does that).
