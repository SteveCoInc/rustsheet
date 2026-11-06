---
id: T-4275
title: "Probe the lone-mark weight on the unprobed paths: lost-position ー after a Latin letter, the half-width ｰ alone, against plane 2 and the high planes (T-4258 follow-up)"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4258]
---

## Goal
Every path that gives a character the engine's `LONE_MARK` weight rests on probe evidence, or is marked
UNVERIFIED in coercion.md.

## Context
T-4258 (probes/t4258-collation-lone-mark-astral) showed that `ー ゝ ゞ ヽ ヾ 々` with no letter before them sort
above every character, so `LONE_MARK` (crates/rustsheet-engine/src/functions/util/collation.rs) moved above every
script weight. The same weight serves two paths the probe did not cover, and the T-4258 fidelity review flagged them
(MINOR):
- In `resolve`, the arm `None if lost => lone` covers the lost position, where `ー` follows a Latin letter at the first
  difference (T-3321's non-transitive rule). An example is `"aー"` against `"a𠀀"`, `"a가"` or `"a"&UNICHAR(1114109)`.
- The half-width `ｰ` U+FF70 alone, against `𠀀`, `𪜀`, U+10FFFD and `ー`.
Lone surrogates cannot be made with UNICHAR, so that half stays out of scope; mark the sentence UNVERIFIED in
coercion.md's T-4258 paragraph. Use a new `probes` case (excel-probe skill) with the layout of t4258: `A`, `B`, and
`C = (A<B)+2*(A=B)+4*(A>B)`.

## Acceptance criteria
- [ ] A new oracle-filled probe case covers the pairs above, and coercion.md records the result (VERIFIED, or a
      task filed for each visibly-off cell).
- [ ] The lone-surrogate sentence in coercion.md's T-4258 paragraph is tagged UNVERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-3381's other kana choices.
