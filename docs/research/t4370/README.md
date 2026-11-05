# T-4370: fraction digits of a second, read at 15 digits

Run these scripts from this directory with plain `python3`. They reuse T-4332's `chain.py` and T-4323's
`frozen_fmt.py`.

- `ulpmap.py` and `ulpmap2.py` generated probes/t4370-ulp-map (340 cells) and t4370-ulp-map-2 (462): TEXT of a tie
  serial below 1 moved n units in the last place (`k/86400+n*2^(e)`, the addition exact), so the oracle places the
  flip between neighbouring doubles. `meta7.json` and `meta8.json` map their cells.
- `model.py` is the model frozen before the fresh sweep: the clock and elapsed fields as T-4332 cut them, the fraction
  digits `floor(r15*scale) mod scale` from the cut's remainder `r` of a second rounded to 15 significant digits.
  `python3 model.py` scores it: 743/746 on the six t4323/t4332 fraction cases (T-4332's cut 706; the three misses
  are `[s]` counts, T-4333) and 739/802 on the two ulp maps (554).
  `python3 model.py meta9.json:t4370-fraction-fresh` scores the fresh sweep: 120/120 (the cut 54).
- `sweep.py` generated probes/t4370-fraction-fresh after the freeze (seed 43700, 120 cells, the first 60 separating
  the model from the cut). `meta9.json` maps its cells.

What the ulp maps showed:
- The flip depends on the serial only: the same serial gives the same digits under `[s].0`, `h:mm:ss.0`, `s.0`,
  `mm:ss.0`, `[h]:mm:ss.0` and `[m]:ss.0` (t4370-ulp-map-2 A103:A162).
- The digits and the seconds field are computed apart. 2 ulp below `41.95/86400` Excel prints "41.0" under `[s].0`
  (t4370-ulp-map-2 A417): the digits wrapped to zero, the whole second did not carry; 1 ulp below it prints "42.0".
- Ties the cut truncated but Excel rounds up flip 2 to 5 ulp of the serial below the decimal tie, which is half a unit
  of the 15th significant digit of the remainder; the ties the cut got right flip within an ulp of the tie.
- Not explained (T-4389): `.000` on a remainder below about 0.06 s flips as if 1.6E-17 to 1.9E-17 s were added before
  the 15-digit cut (0.0045 s between 2.12E-17 and 2.35E-17 s below the tie, 0.0075 s between 1.91E-17 and 2.37E-17;
  0.0015 and 0.0025 round up at every offset probed, down to 1.8E-17 s below). `.0` and `.00` show no offset.

Refuted while searching (scratch, not kept as scripts), all with the cut's own digits: a relative or absolute snap of
the cut's last remainder; cut orders through 86400, 1440 and 60, 24 and 3600, or `86400*scale` at once; directed
rounding (x87 rounding control) on each stage (at best 302 of the first map's 340); 53- or 64-bit precision per stage
(288); a different half step (`0.5/86400/s` and its neighbours); the serial or the seconds rounded to 15, 16 or 17
digits before an exact rounding (at most 287); whole ticks of 1E-3 to 1E-9 s, truncated or rounded, then rounded to
the shown digits (at most 285); day-fraction constants (`1/24`, `1/1440`, `1/86400`) divided out (287). For `.000`
below 0.06 s: a decimal grid of 13 to 17 digits, and a binary snap of the remainder (2^-54 to nearest fits 788 of 802
map cells but not the s.00 tie at 0.005 s, and no operation was found that would produce it).
