---
id: T-3984
title: "arrays.md: name the probed reference forms in the T-3109 GROUPBY/PIVOTBY bullet and cite A88"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3109]
---

## Goal
The arrays.md bullet T-3109 added ("GROUPBY and PIVOTBY lift over an array field_headers or total depth") claims only
what its probe shows.

## Context
- MINOR findings of the T-3109 excel-fidelity review (PR #3034): the bullet opens with "Every reference-returning form
  lifts the same way", but probes/t3109-groupby-options-ref-forms covers only OFFSET, CHOOSE, IF and INDEX (INDIRECT,
  XLOOKUP and the `:` range operator are unprobed); and it does not cite A88:B88 (PIVOTBY field_headers
  `INDEX(Data!J1:K1,0)` over 1, 3 is `""`, `""`: the corner stays empty for both elements, H88 1x2).
- Docs-only; no new probe is needed.

## Acceptance criteria
- [ ] The bullet names the probed forms (OFFSET, CHOOSE, IF, INDEX) and calls the other reference-returning
      functions unprobed.
- [ ] It cites A88:B88 and H88 of probes/t3109-groupby-options-ref-forms.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
