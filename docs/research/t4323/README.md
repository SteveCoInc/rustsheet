# T-4323: fraction digits of a second at a rounding tie

Scripts that built and scored the T-4323 probes (run from this directory with plain `python3`).

- `gen.py` built probes/t4323-fraction-ties (150 tie cells), `gen2.py` fresh sweep 1, and
  `sweep.py` sweeps 2 and 3 and the hold-out case (seeds in its docstring), each after the model
  then chosen was frozen. `meta*.json` map each cell to its format, scale, day and seconds offset
  (meta5.json: the hold-out).
- `eps.py` scores the snap candidates (15-digit rounding, `1+2^-52`), refuted by sweeps 1 and 2.
- Searches not kept as scripts (refuted, T-4332): operation orders of the register (`frac(y)*86400*s`,
  `frac(y)*(86400*s)`, chains through 24, 60, 1440; the half step added as `0.5/S`, `1/(2S)`, in seconds,
  or after scaling), the same with every operation in 64-bit precision or rounded twice, and pre-rounding
  the serial to 15 or 16 digits. None fits the serials below 1; the best got 121 of 145 there.
- `fu.py meta4.json:t4323-fraction-sweep-3` scores the implemented half-step model against the
  engine's previous rounding; `holdout.py` gives the R4.1 train and hold-out rates.
