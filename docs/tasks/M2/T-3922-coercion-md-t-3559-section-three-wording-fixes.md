---
id: T-3922
title: "coercion.md T-3559 section: three wording fixes from the fidelity review (plane 3, sampled marks, inferred readings)"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3559]
---

## Goal
docs/excel-semantics/coercion.md's "Scripts and symbol blocks beyond T-3276 (T-3559)" section, and
the T-3276 paragraph it rewrote, claim no more than probes/t3559-collation-unprobed-blocks shows.

## Context
- The excel-fidelity reviewer of PR #2963 (T-3559) approved it with three MINOR wording findings, left
  for this task so the approved head needed no new push:
  1. The "Block order" bullet says "the astral characters below U+20000 (planes 1 and 3)". Plane 3
     (`𰀀`, U+30000) is above U+20000. The ranks (`𰀀` 150 before `가` 151; `𠀀` 153 and `𪜀` 154 after
     `一` 152) fit an order by high surrogate unit: D800-D83F and D880 up before Hangul, D840-D87F after
     the ideographs. Describe it as planes 1 and 3, or by surrogate range.
  2. The T-3276 paragraph says the marks rule (Hebrew points and cantillation, Arabic harakat, Thai
     tone marks as accents by code point) is settled. It is VERIFIED only at the sampled points
     U+0591, U+05B0, U+0610, U+065F, U+0670, U+0E4B; the extension to T-3324's whole ranges stays
     UNVERIFIED.
  3. "`™` is a letter after `"tn"`" and "no sequence is one element" (emoji) go beyond the cells:
     `™` sits between `"tn"` and `"z"` (G1:G5, ranks 49 and 50), and the ZWJ sequence ranking right
     after `👨` (147, 148) also fits an ignorable ZWJ. Word the readings as inferred (UNVERIFIED).
- Only coercion.md changes. Cite the same cells; no new probe is needed.

## Acceptance criteria
- [ ] The three passages are reworded as above, each claim citing its cells, and every inference is
      marked UNVERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New probes, and the engine (T-3917).
