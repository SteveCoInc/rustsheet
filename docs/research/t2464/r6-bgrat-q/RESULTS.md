# T-3582 r6-bgrat-q: Excel's direct bgrat increment D at b ≠ 1/2 and Excel's own incomplete gamma (negatives)

Worker r6-bgrat-q, 2026-11-02 (T-3582; split from T-3567). Method: docs/research/excel-kernel-playbook.md. Scores are
bit-exact / within 4 ulp / within half a unit of the 15th significant digit (w15). Python:
`bindings/python/.venv/bin/python` (`make py-bindings`, plus `uv pip install mpmath numpy` into it), run from this
directory. `qscan.py` needs r3-bgrat's caches first (`r3-bgrat/branches.py`, then `r3-bgrat/exact.py -a`).

## 1. Headline

1. **D's noise is not a different Q(b, z) continued fraction.** With r3-bgrat's x87 bgrat (stored ln x and z,
   `bgld.bgrat_ld`, which now takes a callable Q mode), j0 = Q/r from NR's gcf (modified Lentz, FPMIN 1e-30, the loop
   T-3571 found in Excel's bfrac) or from Cephes igamc's three-term recurrence, in doubles, scores below the exact Q
   and adds its own noise (`qscan.out`, 2,908 split points):

   | j0 from | exact/w4/w15 | run sd (30.7, 0.7) / (40, 0.03) |
   |---|---|---|
   | exact Q (baseline) | 191/1,587/2,069 | 6.60 / 8.44 |
   | grat1 CF, x87, eps 1e-15 (r3-bgrat) | 206/1,672/2,158 | 6.60 / 8.44 |
   | NR gcf, doubles, eps 2.2e-16 (fraction itself / through a rounded Q) | 145/1,195/1,859; 128/1,156/1,822 | 8.63 / 16.66; 9.08 / 16.96 |
   | NR gcf, doubles, eps 1e-15 | 132/1,146/1,833; 127/1,099/1,773 | 8.49 / 16.07; 8.96 / 16.40 |
   | NR gcf, doubles, eps 2.2e-15 | 102/946/1,409; 95/721/1,234 | 8.26 / 15.52; 8.70 / 15.80 |
   | NR gcf, eps 3e-7 (NR's float EPS) | 0/0/0 | (biased) |
   | Cephes igamc, doubles, eps 1.1e-16 (fraction / through Q) | 119/1,044/1,699; 128/1,120/1,770 | 8.41 / 21.83; 8.91 / 22.07 |
   | Cephes igamc, doubles, eps 2.2e-16 | 132/1,036/1,706; 142/1,114/1,770 | 8.32 / 20.73; 8.81 / 20.98 |

   The sd is of Excel - model, in units of 2^-53 relative. A model with Excel's arithmetic would lower it; every
   candidate raises it.
2. **Excel's own P(b, z) at b < 1 (new probe t3582-r6-gamma-runs).** GAMMA.DIST(z, b, 1, TRUE) on four runs of 1,024
   consecutive doubles in z (`pscan.out`, `pnoise.out`):

   | run | P | Excel vs exact (units of 2^-53) | Excel - CR in ulps | lag-1 autocorr |
   |---|---|---|---|---|
   | b 0.03, z0 0.3 | 0.973 | mean +0.07, sd 0.37 | 0: 799, ±1: 225 | +0.84 |
   | b 0.03, z0 1 | 0.993 | mean +0.09, sd 0.38 | 0: 775, ±1: 249 | +0.91 |
   | b 0.7, z0 0.3 | 0.420 | mean -3.67, sd 3.67 | -12 to +5 | +0.60 |
   | b 0.7, z0 1 | 0.761 | mean +2.28, sd 0.87 | 0 to +4, mostly +1/+2 | +0.18 |

   So Excel's incomplete gamma is faithfully rounded at b = 0.03 (b is the double 0.030000000000000027): every cell
   within 1 ulp, 22-24% of cells 1 ulp off the correctly rounded value, and a residual sd of 0.37-0.38 units against
   0.29-0.30 from rounding alone, so about 0.2 units of Excel's own error. It sits +2.3 units high at b = 0.7 near
   z = 1 (mostly 1-2 ulps: one excel-graph fill, inside the 4-ulp web/desktop floor of
   docs/research/oracle-calibration.md, so unconfirmed until a desktop or second fill, decision 0112), and is noisy
   with structure (FFT line 30x the mean) at b = 0.7 near z = 0.3. The engine scores 1,126/3,605/3,876 of 4,096 (CR 1,678/3,711/4,092).
3. **No series model explains the b = 0.7 residual.** NR gser, Cephes igam and the alternating z^b/Γ(b+1) series, in
   doubles and in x87, all leave the b = 0.7, z0 = 0.3 residual at sd 3.68-3.73 (CR 3.69). The x87 forms reach CR
   (alt series ld 1,684/3,703/4,091); the doubles forms are worse (NR gser 248, Cephes 376, alternating 845
   bit-exact). Regressing on the rounding of stored ln z, b·ln z, z^b (exp or pow), b ln z - z, ln z - z and e^-z
   (`preg.out`) leaves sd 3.61 at best (b·ln z - z, coefficient 1.0), so no single stored intermediate makes it.

## 2. What this says about D

D at b = 0.03 has a noise sd of 8.5 units while Excel's P at b = 0.03 adds only about 0.2 units to rounding, and D at b = 0.7 (sd 5.7)
is noisier than P's worst run (3.7). If bgrat took Q from Excel's own incomplete gamma, Q's upper-tail computation
would have to carry noise that the lower tail at the same b does not. That cannot be excluded (the worksheet cannot
show Q at b < 1, statistics.md "The b = 1/2 identity cannot be tested"), but nothing seen supports it. The source of
D's noise stays open.

## 3. Negatives (decision 0056)

| tried | score | script |
|---|---|---|
| NR gcf (modified Lentz) Q in doubles, eps 2.2e-16 / 1e-15 / 2.2e-15 / 3e-7, as j0 or through Q | 145, 132, 102, 0 bit-exact of 2,908 (best) | `qscan.out` |
| Cephes igamc Q in doubles, eps 1.1e-16 / 2.2e-16 | 119-142 bit-exact of 2,908; run sd 20.7-22.1 at (40, 0.03) | `qscan.out` |
| P by NR gser / Cephes igam / alternating series, doubles | 248 / 376 / 845 of 4,096 | `pscan.out` |
| P by the same in x87 | 122 / 138 / 1,684 of 4,096 (CR 1,678) | `pscan.out` |
| stored ln z, b·ln z, z^b, e^-z, b ln z - z, ln z - z on the b = 0.7 runs | residual sd 3.61 at best (from 3.67) | `preg.out` |

## 4. Probe

| probe (probes-numeric) | cells | what | script |
|---|---|---|---|
| t3582-r6-gamma-runs | 4,096 | GAMMA.DIST(z, b, 1, TRUE), b in {0.7, 0.03}, z0 in {0.3, 1}, 1,024 consecutive doubles each, exact double literals | `mkprobe.py` |

Filled by excel-graph on 2026-11-02. Its 220 visibly-off engine cells are owned by T-3224.

## 5. Next

- The b = 0.7, z0 = 0.3 structure: find the FFT line's period in z and match it to a stored quantity's rounding
  period (a binade of a product of z, e.g. z·k or z/(b+k) in the series).
- Runs of D and of P at the same (b, z) cannot both be had (P is near 1 where D's z lives); a run of D at small z
  (z < 1.1, where TOMS bgrat takes grat1's series) would test whether D's noise follows P's at the same b.
