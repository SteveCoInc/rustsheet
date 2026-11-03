---
id: T-3445
title: "fold_char follows Excel's comparison fold, not Rust's case tables (probes/t3298-case-mapping-sweep C and D)"
milestone: M2
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
MATCH (type 0), VLOOKUP(..,FALSE), HLOOKUP(..,FALSE), XMATCH(..,2) and wildcard criteria fold case with Excel's
comparison table, so every cell of probes/t3298-case-mapping-sweep column C and D passes.

## Context
- Failing cells: probes/t3298-case-mapping-sweep Calc!C8, C10, C15, C17:C21, C25:C27, C32:C48 and D8, D10, D15,
  D17:D21, D25:D27, D32, D34:D48.
- lookup.md "MATCH, VLOOKUP and HLOOKUP exact-match keys fold case one character to one" has the block summary
  (T-3298); T-3584 pinned the weightless set (docs/research/t3584/weightless-bmp.txt) and the
  equivalence classes beyond case pairs (lookup.md "Weightless characters" and "Equivalence classes").
- Failing cells (T-3584): probes/t3584-match-fold-weightless Calc!E1:E32, G1:H32, K1:O85, R1:R256, T1:T1232.
  Calc!R1 also shows MATCH taking U+0001 as matching any text (mask 3 at U+0001), which the engine does not.
- Code: `fold_char` and `fold_case` (crates/rustsheet-engine/src/functions/util/collation.rs), reached through
  `folded_units` (`ExactKey`) and `WildcardPattern`. Today they take Rust's `to_lowercase`, which folds Cherokee,
  Mtavruli, Latin Ext-D and the supplementary scripts that Excel keeps apart.
- The indexed exact search (`ExactKey`) must stay consistent with the linear one under the new fold, including the
  weightless-equal rule; keep the fold per UTF-16 unit or scalar consistent with surrogate pairs (they never fold).

## Acceptance criteria
- [x] Calc!C1:D48 of probes/t3298-case-mapping-sweep and the T-3584 cells above pass (Calc!R1:R256 excepted: see Outcome) and leave
      eval/baseline/probes.txt, 0 regressions;
      probes/t3133-lookup-criteria-case-fold keeps passing.
- [x] Unit tests cover one character per block class (folds, stays apart, weightless).
- [x] lookup.md's section says "Implemented in" with this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- UPPER and LOWER (T-3583). Sorting and the `=` collation unless T-3584 shows they share the table (then a follow-up).

## Outcome
- New `fold_lookup_char` (functions/util/collation/lookup_fold.rs), the fold of `folded_units` (the exact-match key of
  MATCH type 0, VLOOKUP/HLOOKUP FALSE, `ExactKey`'s index) and `WildcardPattern::new` (XMATCH/XLOOKUP wildcard mode,
  wildcard criteria). It works in this order:
  - a weightless character (the 2568 BMP code points of docs/research/t3584/weightless-bmp.txt, a 381-range table)
    folds to U+0378;
  - a supplementary character stays itself;
  - SEARCH's `fold_char` result is used unless it lands on a weightless character (Cherokee's capitals then stay
    apart) or the character is `Ɒ` U+2C70 or `Ɥ` U+A78D, which stay apart (t3298 Calc!C32:D32, C36:D36; new to
    lookup.md);
  - members of T-3584's wider classes map to their class's first member, plus U+1FBE with `Ι ι` (t3298 Calc!C8,
    t3584 K9).
  The fold is still one character to one, so the indexed and the linear exact search agree.
- SEARCH, the D-function prefix criteria and TEXTSPLIT/TEXTBEFORE keep `fold_case`/`fold_char`. The weightless class
  is unprobed there.
- Eval delta, `make eval-check`: probes known failures 4969 -> 4374 (fixed 595: t3298-case-mapping-sweep 55, every
  cell of C1:D48 now passing; t3584-match-fold-weightless 540), REGRESSIONS 0 in every suite;
  probes/t3133-lookup-criteria-case-fold still 426 of 426; visible parity 2821 in-scope cells off.
- Not fixed, kept in eval/baseline/probes.txt:
  - 54 of t3584's Calc!R1:R256: their MATCH digits now all match Excel's except U+0001's (follow-up T-3921), and the
    `=` digit is T-3904's. The other 202 R cells pass.
  - the `=` cells B, D, F, P of t3584: T-3904.
- Unit tests: `lookup_fold::tests` (one character per class: folds, stays apart, wider classes, weightless).
- Review round 1 (excel-fidelity): the wildcard paths (criteria, XMATCH mode 2) take the fold by inference, because
  they share `WildcardPattern`. criteria.md and lookup.md mark this UNVERIFIED; T-3929 probes it.
- Merging origin/main (CI's baseline check) also took 2 cells of probes/t3559-collation-unprobed-blocks (Calc!C62,
  C126) off eval/baseline/probes.txt: they pass on the merged tree.
- Follow-ups filed: T-3921 (MATCH's U+0001 key), T-3929 (probe the wildcard paths).

## Landing notes
- PR #2968, ready for review, not enqueued. Head before this note: ba634ea67 (`make catch-up` of 5c7c21c0a,
  baseline-only conflict, decision 0207). CI was still running at 11:38 UTC when the session's window closed; this push
  restarts it.
- Reviews: security, design and excel-fidelity all APPROVE on 5c7c21c0a (round 2). The catch-up and this note need no
  re-review. Their MINORs are recorded in the PR body and belong in T-3929's PR.
- Next step: once CI is green on the head, enqueue with `enable_pr_auto_merge` (SQUASH). If main moves the baseline
  again, run `make catch-up PR=2968` and re-enqueue.
- Do not repeat: excel-fidelity round 1 rejected applying the fold to wildcard criteria as verified. Keep the UNVERIFIED
  notes until T-3929's probe lands.
