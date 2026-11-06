---
id: T-4468
title: "Near-integer snap at the upper limit: SUBSTITUTE instance_num, FIXED/DOLLAR decimals, CHAR and FIND start just below their caps; tidy T-4453's citations"
milestone: M5
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel's answer is on record for each snapping M5 text argument at a value just below its upper cap. Since T-4453 the
engine snaps these values up to the cap and answers `#VALUE!`, where the old floor accepted them. T-4453's citation
nits are fixed in the same PR.

## Context
- T-4453 (PR #3485) made FIND/SEARCH start_num, CHAR, SUBSTITUTE instance_num and FIXED/DOLLAR decimals snap within
  2^-22 + 2^-33 below an integer (probes/t4453-near-integer-text-args). The probe covered only interior bases. At the
  cap, the snap now gives `#VALUE!` with no Excel cell behind it:
  - `SUBSTITUTE(t,"a","b",2147483647-2^-23)` (2^31-1 is rejected; text.md "SUBSTITUTE instance_num" accepts up to
    2147483646)
  - `FIXED(1,128-2^-23)` and `DOLLAR(1,128-2^-23)` (128 is `#VALUE!`)
  - `CHAR(256-2^-23)` (256 is `#VALUE!`)
  - `FIND("a","abc",5-2^-23)` (a start past LEN+1)
  The existing REPLACE 32767.9999999 caveat in text.md is the same kind of open question.
- Excel-fidelity review MINORs on #3485, both docs-only:
  - char_code.rs cites "D1:E21". UNICHAR's column ends at E14, so it should read "D1:D21, E1:E14".
  - text.md "DOLLAR and FIXED decimals": the Evidence and Tasks lines should add
    probes/t4453-near-integer-text-args G1:J21 and T-4453.

## Acceptance criteria
- [x] A committed excel-graph probe case covers each cap above: n - 2^-23 (snaps) and n - 2^-22 - 9*2^-36 (does not).
- [x] text.md's "LEFT, RIGHT and REPLACE round counts up near an integer" entry records the result. If Excel differs
      from the engine, the engine is fixed and every probe cell passes.
- [x] Both citation nits above are fixed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Interior bases (T-4453 has them).
- TEXTBEFORE/TEXTAFTER and REGEX occurrence arguments (M7).

## Handoff (resumed)
Done: committed probes/t4468-near-integer-text-caps (oracle filled, excel-graph 2026-11-06), the two citation nits, and the
text.md result. FIXED/DOLLAR(128-2^-23), CHAR(256-2^-23) and FIND start at the cap already match the engine (all
bit-exact). Only SUBSTITUTE instance_num at 2^31-1 differs: Excel snaps over a far wider window there (`#VALUE!` for
d <= 2^-18, "aaa" from 5*2^-19; cells A5, F1:F6 fail, 7 cells), the engine keeps the absolute 2^-22+2^-33 bound.
Why I stopped: finding the real bound model (absolute vs scale-dependent, and between 2^-18 and 5*2^-19) is numeric fitting
(work-loop step 4 numeric-fits guards, hold-out and fresh sweep), beyond a sonnet run. Next: one more bisection round
between 2^-18 and 5*2^-19, pick a structural model, fix `substitute_element` (util.rs) / `near_integer.rs`, then
`make eval-baseline`, run the reviewers, enqueue. Add the owner line for A5 and F1:F6 in the PR.

## Outcome
- probes/t4468-near-integer-text-caps (sonnet run): FIXED/DOLLAR(1,128-2^-23), CODE(CHAR(256-2^-23)) and
  FIND("","abc",5-2^-23) snap to the cap and are `#VALUE!` as the engine had it; only SUBSTITUTE's 2^31-1 cap
  differed (A5, F1:F6).
- Structural model (kind (c), a rounding model, no `FITTED` tag): SUBSTITUTE's instance_num is cut to 15 significant
  digits (round to nearest, `to_excel_precision`) before `near_integer_count`. Candidates ranked on the train cells
  (t4468-near-integer-text-caps A, F, G: 32 cells): absolute 2^-22+2^-33 bound 25/32; 15-digit cut 32/32; a
  relative bound 2^-48 also fits train (32/32) and predicts `#VALUE!` up to d ≈ 7.6E-6.
- Hold-out, a separate case rather than a hash split (predictions for both new cases were computed by a scratch script
  in the session before `rustsheet-oracle fill` ran; that record is not in git, the commits only show the fill and the
  model landing together; no margin was stated before scoring, and both rates are 100%, so any margin holds): probes/
  t4468-substitute-cap-window, 21 cells, decimal d from 4E-6 to 9E-6, literals and text. 15-digit cut 21/21 (with
  the engine's existing 15-digit literal truncation; B5 and C1 show a literal 2147483646.999999 reads
  2147483646.99999), absolute 19/21, relative 2^-48 11/21. The edge sits between 4.5E-6 and 4.9E-6 of the double, at
  the decimal 2147483646.999995, which only the 15-digit cut puts there.
- Fresh sweep, generated after the freeze (seed 4468): probes/t4468-substitute-cap-sweep, 60 cells (d log-uniform in
  [1E-7,1E-4] at the cap, [1E-8,1E-5] at 3): 60/60 (absolute alone 48/60; the relative bound, already ruled out
  by the hold-out, also 60/60). Scores: scratch script over the case and oracle files, one candidate per model.
  Gate A envelope: not applicable (the results are text or `#VALUE!`, no numeric point); reproducibility floor: not involved (the edge is 5E-6 wide).
- Citation nits fixed (char_code.rs "D1:D21, E1:E14"; text.md DOLLAR/FIXED Tasks line). text.md's near-integer
  entry records the caps, the model and the literal-vs-computed split.
- Eval delta: probes +3 cases (137 cells), all pass; the 7 failing cells the handoff baselined now pass, and the
  case's 7-cell visible-parity lock line is gone (re-recorded from main's file by `make eval-baseline`).
