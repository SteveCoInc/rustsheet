# T-4323: fraction digits of a second at a rounding tie

Scripts that built and scored the T-4323 probes (run from this directory with plain `python3`).

- `gen.py` built probes/t4323-fraction-ties (150 tie cells); `gen2.py`, `gen3.py` and `gen4.py`
  built the fresh sweeps 1, 2 and 3, each after the model then chosen was frozen. `meta*.json`
  map each cell to its format, scale, day and seconds offset.
- `eps.py` scores the snap candidates (15-digit rounding, `1+2^-52`), refuted by sweeps 1 and 2.
- `struct.py`, `brute.py` (with `ext.py`'s 64-bit rounding) search operation orders for the
  fraction register; none explains the serials below 1 (T-4332).
- `fu.py meta4.json:t4323-fraction-sweep-3` scores the implemented half-step model against the
  engine's previous rounding.
