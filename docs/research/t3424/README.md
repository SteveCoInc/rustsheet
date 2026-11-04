# T-3424: RATE's type at the noise floor

Notes from 2026-11-04, opus run. The finding is in docs/excel-semantics/financial.md, "Excel's residual and step are
the model's; the outcome of a flat secant is not". These scripts reproduce it. None of them changes the engine, and
no constant was chosen from them.

Run each script from `tools/` with this directory as its first argument, for example
`cd tools && uv run python ../docs/research/t3424/fv_trace.py ../docs/research/t3424`.

| Script | What it does |
|---|---|
| `model_trace.py` | The engine's `rate_secant` in Python: the T-2005 secant with T-3348's hold and T-3698's restart, on the fit lab's twice-rounded residual. Run it with `n pmt pv fv type [guess]` to print a trace. |
| `gen_trace_probe.py` | Writes the cells of probes/t3424-rate-noise-trace and `probe_meta.json` (`gen_trace_probe.py <dir> <out.json>`). |
| `fv_trace.py` | Excel's FV against the model's -(pv·P + pmt·A) at every probed iterate. Result: 345 of 345 equal. |
| `score_rate.py` | Loads every literal `=RATE(` oracle cell of probes and probes-numeric (2,475 cells) and scores a model variant on them. |
| `step_forms.py` | Six step forms, each with a flat secant held or `#NUM!`. The model's form is bit-exact on 1,951 cells; the others get 1,039 to 1,945. |
| `extended_iterates.py` | Iterates kept in 80-bit precision: 1,111 bit-exact (residual at the double iterate) and 1,048 (1+r and /r at the extended iterate). |
| `hold_features.py` | For the 609 flat-secant holds: the best single-threshold split of Excel's returned holds from its errors, per quantity. It never splits a tie. The best split gets 57 wrong; returning every hold gets 60. |
| `hold_ulps.py` | How far, in ulps and in which direction, Excel's value is from the model's held iterate. |
| `seeded_runs.py` | The 98 seeded RATE cells: the model's outcome against Excel's, by hold point. |
| `fv_forms.py` | Every literal `=FV(` oracle cell under the plain and the twice-rounded forms: 586 and 587 of 587 (T-4146). |

The probe writes each iterate as `(a+b)`. `a` is the iterate cut to 15 significant digits, and `b` is the rest.
Excel reads a formula literal at 15 digits, and Excel's `+` rounds `a+b` back to the iterate's double (checked for
every cell when the probe is generated).
