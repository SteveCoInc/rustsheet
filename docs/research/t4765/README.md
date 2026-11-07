# T-4765: GROWTH at the edges of LOGEST's factors

`gen.py` builds probes/t4765-growth-factor-edges and `predictions.tsv`. Both were committed before the excel-graph
fill. Every fit is exact (y = e^(c + Σ m_j x_j) at the known x), so LOGEST's ln coefficients are c and the m_j.

- H1: GROWTH is b · F, where b = e^c and F = e^(Σ m_j x_j) are each a double. A subnormal factor is flushed to 0, a
  subnormal product is flushed to 0, and `#NUM!` follows once b, F or the product overflows. This fits T-4590's
  Int A2:A7 zeros.
- H2: GROWTH is e^(c + Σ m_j x_j), with `#NUM!` only where b overflows. This is the engine before T-4765 (T-4590).
- H3, for the Multi sheet only: one factor per variable, e^(m_j x_j). There, e^(-720) would make the prediction 0 and
  e^(720) would make it `#NUM!`. H1 and H2 both give e^1 there, because the slope sum is 0.

Sheets:
- Cut: the zero's cut in the slopes' sum, between -700 and -709.5. H1 puts it at the DBL_MIN cut, -708.396.
- Multi: two x with opposing large terms.
- SmallB: a subnormal b with an offsetting slope.
- NoConst: const FALSE with a subnormal slopes' factor.
- Over: a slopes' factor past the double range, where the prediction is finite.

Run: `cd tools && uv run python -I ../docs/research/t4765/gen.py ../eval/suites/probes/cases/t4765-growth-factor-edges.case.json ../docs/research/t4765/predictions.tsv`
