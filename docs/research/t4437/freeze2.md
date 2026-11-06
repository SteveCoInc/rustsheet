# T-4437 second freeze (before the second fresh sweep)

Written and committed before `t4437-beta-zero-fresh2` was oracle-filled; no cell of that case was
scored before this file was pushed.

## The frozen rule
On bratio's L140 bup+bgrat branch (x0 > 0.7, a0 > 15, b0 < 40 after the swap), Excel's tail is 0
when either holds:
1. bup's prefactor ret is not a normal double (T-4045, `l140_bup_prefactor_underflows`);
2. **n >= 3 bup terms** and bup's sum W = ret·S, computed as `bup.rs` `l140_bup_sum_flushed` does,
   is below 2^-970 = DBL_MIN / DBL_EPSILON.

The domain n >= 3 is part of the frozen rule. n = 2 is out of domain (T-4564 owns it): the second
sweep adds two n = 2 families as evidence for T-4564 only, and they are not scored for this rule.
Nothing in the code changes between this freeze and the sweep's scoring.

## The sweep (`gen_fresh2.py`)
Eight untouched in-domain families, n = 3 to 7, c in {0.25, 0.5, 0.75, 1} and a0 from 1.2E6 to
9E6, each walked in ln of the true tail by 0.05 nats from ln 2^-970 + 0.525 (row 1) to
ln 2^-970 - 0.525 (row 22). Prediction: a number on rows 1 to 11, 0 on rows 12 to 22.
Scored: the count of in-domain cells whose 0-or-number class matches Excel, and, on the number
cells, the Gate A envelope (15 significant digits of Excel, or 1e-9 relative of 50-digit math).
