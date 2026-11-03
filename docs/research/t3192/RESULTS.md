# T-3192: bpser's open items — BETA.DIST at a = b = 1/2

Finding (statistics.md "BETA.DIST at a = b = 1/2 is an arctangent closed form, reflected above x = 0.9"):
item 4 of T-3192 (`BETA.DIST(2.4674011002723394E-20,0.5,0.5,TRUE)`, t1745-beta-inv-roundtrip G1, 12 ulp low) is not
bpser's arithmetic but a different kernel: Excel takes an arctangent closed form at a = b = 1/2.

## Steps
1. Diagnosis: the 12 ulp come entirely from fl(LN x) (3.1E-15 off at x = 2.5E-20); L does not matter.
2. A/B of bpser with an extended LN (the full eval, `docs/research/t3007/cell_diff.py`): stored product 1 or unstored
   product 2 both move thousands of BETA.DIST/T/F cells off (variant 2: 2,489 better, 13,322 worse on BETA.DIST). So
   bpser's LN stays stored, and the a = b = 1/2 cells must take another path.
3. Tune set: the 49 a = b = 1/2 cdf cells on file. Closed forms in doubles (`brute.py`-style ranking, not committed;
   the forms are in `score_half.py`): reflected `2*atan(sqrt(x)/sqrt(y))/pi` 42, the direct form 37, the engine 16,
   correctly rounded 27. Frozen choice: reflected at x > 0.5.
4. First fresh sweep (`make_probe.py --sweep 1`, 321 x): the frozen choice 248, the direct form 270, Excel's own worksheet ATAN
   form equals BETA.DIST on 270. The reflection is right only near 1: on all 370 cells the two differ only above 0.5,
   the direct form wins up to 0.888 and the reflection from 0.900 (one exception at 0.837).
5. Switch frozen at x > 0.9, then a second fresh sweep (`make_probe.py --sweep 2`, 341 x, dense on 0.8-1): 287 bit-exact against
   228 (direct) and 253 (reflected at 0.5); every cell within 2 ulp. This is the R4 hold-out of the switch.

## Scores (bit-exact)
| set | cells | engine before (bpser) | direct | reflect > 0.5 | switch 0.9 (ported) |
|---|---|---|---|---|---|
| on file before T-3192 (tune) | 49 | 16 | 37 | 42 | 42 |
| t3192-beta-half-half (sweep 1; fixed the switch) | 321 | - | 270 | 248 | 281 |
| t3192-beta-half-half-2 (sweep 2; hold-out) | 341 | - | 228 | 253 | 287 |

Full eval A/B of the port against main (before the new cases): 54 cells better (BETA.DIST, BETADIST and
t0969-t-beta-link's df-1 cells), 0 worse.

## Open
- x = 0.5: Excel 0.5000000000000001 (5 cells), the form 0.5.
- The remaining 1-2 ulp misses (40 on sweep 1, 54 on sweep 2) have no form yet.

Usage: `tools/.venv/bin/python docs/research/t3192/score_half.py` (repo root).
