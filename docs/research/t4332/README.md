# T-4332: fraction digits of a second at a tie below serial 1

Run these scripts from this directory with plain `python3`. They reuse T-4323's `frozen_fmt.py`.

- `chain.py` is the model frozen before the fresh sweep. It adds the half step to the serial and cuts `frac(y)` field by
  field (24, 60, 60, then the scale), floored at each stage. `python3 chain.py` scores it against T-4323's register on
  the five t4323 cases: 134/156 below serial 1 and 469/470 from 1 up, against 101/156 and 467/470.
  `python3 chain.py meta6.json:t4332-fraction-fresh` scores the fresh sweep: 103/120, against 72/120.
- `sweep.py` generated probes/t4332-fraction-fresh after the freeze (seed 43320, 120 cells, the first 60 separating the
  two models). `meta6.json` maps its cells.
- Refuted while searching (scratch, not kept as scripts). All these use the same half step:
  - cuts through 24 and 3600, or 1440 and 60 (117 and 107 of 156 below 1);
  - the fraction cut digit by digit (131);
  - whole milliseconds or finer units first, rounded or truncated (84 to 125 below 1, but at most 363 of 470 from 1 up);
  - the cut in 64-bit extended precision (73 to 108);
  - the half step added at the seconds stage (111);
  - the larger of the cut and the register (fresh sweep 84/120).
- Open (T-4370): ties that Excel rounds up while the cut's last remainder lies 1 to 11 ulp below a whole unit.
