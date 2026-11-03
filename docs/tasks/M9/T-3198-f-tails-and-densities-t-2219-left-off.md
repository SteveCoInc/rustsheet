---
id: T-3198
title: "F tails and densities T-2219 left off: F.DIST.RT(3,50,100) on L140 and F.DIST densities at (50,100); T-2219's gen-m9 cells below a + b = 10"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
campaign: accuracy
depends_on: []
---

## Goal
The F cells T-2219 left visibly off, and its gen-m9 cells, agree with Excel to 15 significant digits.

## Context
T-2219 moved half-integer shapes with 10 <= a + b < MAXGAM, outside the closed forms and the ported
bratio branches, to an accurate double-double kernel (`incomplete_beta.rs::half_integer_tail`;
docs/excel-semantics/statistics.md "Incomplete beta in the ironcalc-calc workbooks", bullet
"Accurate tails at half-integer shapes from a + b = 10"). Cells of its table it did not reach:
- probes-numeric/m9-f-dist-rt A19 `F.DIST.RT(3,50,100)`: Excel 1.516786664496894e-6, ours
  1.5167866644968792e-6 (mpmath 1.5167866644968894697e-6): bratio's dispatch is L140 bup + bpser,
  which is ported (`bratio/bup.rs`), so the error is that branch's (T-2506 family).
- probes-numeric/m9-f-dist A26 `F.DIST(0.5,50,100,FALSE)` (Excel 0.08394970207752739, ours
  0.08394970207752746) and A30 `F.DIST(3,50,100,FALSE)` (Excel 1.05292998522241E-05, ours
  1.0529299852224087E-05): the density is Excel's own chain model (`f_density.rs`, T-2586/T-3034),
  which reaches 15 digits only where |c ln x| is small.
- T-2219's 26 gen-m9 cells that stay off moved to T-2221 on 2026-11-03 (see "Triage (2026-11-03)" below).

## Acceptance criteria
- [ ] m9-f-dist-rt A19 and m9-f-dist A26, A30 within half a unit of the 15th digit of Excel.
- [x] Every gen-m9 cell above within 15 digits of Excel, or named in the Outcome and handed to the
      task that owns its branch (T-2221 for Excel's own last bits): all 26 handed to T-2221, triage below.
- [ ] No gated known-failure line added; `make check` green; reviewers approve.

## Out of scope
- The F quantiles (T-2511) and bit-level fits of Excel's formulation (T-2221).

## Triage (2026-11-03)
This is an opus run's triage. The task stays `todo` for the three probe cells. 50-digit references come from mpmath 1.3 at
mp.dps = 50: `betainc(.., regularized=True)` for the tails, and the closed-form density. Each relative error is against that
reference; values are Excel's and ours on main 93a939d2.

| cell | call | Excel rel. error | ours rel. error |
|---|---|---|---|
| gen-m9 2801c1c11c21e134 A374 | `FDIST(3.14159,9,6)` | -1.8e-16 | -1.1e-15 |
| gen-m9 35e8168aae2b00f8 A123 | `F.DIST(100,4,10,FALSE)` | -3.5e-15 | -3.3e-15 |
| gen-m9 3a25a03eb94fcfbd A49 | `FDIST(123456789.123,10,7)` | -1.06e-14 | -1.05e-14 |
| gen-m9 68d8ffb69ff739d8 A273 | `F.DIST.RT(2.5,1E+10,6)` | -1.5e-15 | -2.0e-15 |
| gen-m9 7bd219ec0c0c5e22 A349 | `FDIST(7,10,1E+8)` | +6.7e-15 | -2.1e-15 |
| gen-m9 8dd2312c96effb1b A144 | `FDIST(2.5,8,9)` | -2.1e-16 | -3.5e-16 |
| gen-m9 a5828d3925575713 A235 | `FDIST(42,4,9)` | -1.5e-15 | -1.3e-15 |
| gen-m9 b8ee55934c5b824c A22 | `FDIST(37.5,9,7)` | -3.2e-15 | -2.4e-15 |
| gen-m9 d8266bc15533702a A337 | `F.DIST.RT(1E+10,8,6)` | -1.4e-15 | -1.9e-15 |
| gen-m9 ee2497a5e683c5d9 A118 | `FDIST(3,5,30)` | +3.1e-16 | +7.1e-16 |
| gen-m9 60b0b6e69ad35a45 A151 | `F.DIST.RT(1E+10,9,9)` | +3.8e-15 | +4.7e-15 |
| probes-numeric m9-f-dist-rt A19 | `F.DIST.RT(3,50,100)` | +3.0e-15 | -6.8e-15 |
| probes-numeric m9-f-dist A26 | `F.DIST(0.5,50,100,FALSE)` | +3.2e-16 | +1.2e-15 |
| probes-numeric m9-f-dist A30 | `F.DIST(3,50,100,FALSE)` | +1.03e-14 | +9.0e-15 |

- **gen-m9.**
  - Ours is as close to the reference as Excel is, and usually on the same side. In 7bd219ec0c0c5e22 A349 ours is closer.
  - The other 15 cells take computed arguments and differ from Excel by 1e-16 to 9e-15. None is visibly off.
  - Matching them means fitting Excel's own last bits, not making ours more accurate, so all 26 moved to T-2221's list
    (its "Cells added from T-3198 (2026-11-03)"). The one exception is 2801c1c11c21e134 A374, where ours is 6x further
    from the reference than Excel. It sits at a + b = 15 on the L140 (bup + bpser) path, the same as A19 below.
- **A19.** Ours is 2.3x further from the reference than Excel.
  - The correctly rounded reference is within half a unit of Excel's 15th digit (3.0e-15 against a half unit of 3.3e-15).
  - So routing bratio's L140 dispatch for shapes of 10 <= a + b < MAXGAM to T-2219's double-double kernel
    (`incomplete_beta.rs::half_integer_tail`) would likely meet the criterion.
  - It also moves every cell bup + bpser now matches bit for bit. It needs a full eval and the R4 guards if any cut is
    chosen by scoring, so it was not attempted in this run.
- **A26.** Ours is 1.2e-15 off the reference and Excel 3.2e-16 off, inside Excel's chain-model density (`f_density.rs`).
- **A30.** Both are about 1e-14 off on the same side, so it is likely the chain model's own error.
