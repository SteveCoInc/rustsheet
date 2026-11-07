---
id: T-4758
title: "YIELD's residue after T-2788: deep-discount ulps at large ^ exponents and Gate A short-of-root extremes"
milestone: M9
lane: research
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
YIELD with more than one coupon left matches Excel bit for bit on the cells T-2788 left 1 to 2 ulps off, and its
Gate A short-of-root extremes come inside the envelope (or the task records the kernels and stops tried).

## Context
Split from T-2788 (2026-11-07). T-2788 found that PRICE's redemption term is discounted at `(n - 1) + DSC/E`, not
`(DSC/E + n) - 1`, which fixed yield-start-one-input S!B67, yield-start-term-basis S!A52 and S!B50 and
coup-calendar-edges S!J12 (financial.md "YIELD stops short of PRICE's root", T-2788 paragraph). What is left:
- Deep-discount YIELD cells `YIELD(39681,45662,R,P,102,4)` 1 to 2 ulps off, where the lab (`yield_caret`, `^`-priced)
  is off by the same 1 to 2 ulps, so the miss is not the engine's PRICE order: yield-deep-discount-cut S!A2, B2, C2,
  D5, D6, and the 100th-iterate cells B3 and C4 (5e-14 and 6e-15 relative); yield-deep-discount-sweep S!D7, D9, F3,
  H9. On D7 and F3 the engine's `^` (`arithmetic::power`) and the lab's `excel_model` differ by 1 ulp at exponents
  near 28 to 65 (T-3144's kernel question); 16 of 928 factors on T-2788's cells differ, all at bases 3 to 7.
- yield-newton-domain P!B13 `YIELD(36665,44491,30,106,100,4,0)`: Excel 14.303206872934162, ours (and the lab)
  14.30320687293416, 1 ulp low.
- Gate A YIELD short-of-root extremes (engine and Excel both short of the math root, the engine outside Excel's 15
  digits): accuracy/YIELD/s1 A230, A252, A261 and accuracy/YIELD/r4-T-2788 A252, A267 (eval/accuracy/known-failures.tsv).
  Their prices and redemptions are far from 100, the region of T-4523: re-check them after it.

- T-2788 review notes: PRICE's redemption order rests on YIELD cells only. A PRICE probe at inputs where
  `fl(fl(DSC/E + n) - 1)` and `fl((n - 1) + DSC/E)` differ would evidence it directly. `discount_overflows`
  (financial_bonds.rs) and DURATION's `add_cash_flow` still take `DSC/E + n - 1`; check whether they should follow.

## Acceptance criteria
- [ ] The deep-discount and P!B13 cells leave eval/baseline/probes.txt, or the task records the PRICE kernels, `^`
      variants and stops tried with their lab bit-exact counts (decision 0056).
- [ ] The five Gate A lines owned by this task leave eval/accuracy/known-failures.tsv, or the Outcome records why each stays.
- [ ] A PRICE probe settles the redemption order directly (cells where the two orders differ), and
      `discount_overflows` follows whichever order it shows.
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD's `#NUM!` cells (T-4523) and the `^` kernel itself (T-3144).
