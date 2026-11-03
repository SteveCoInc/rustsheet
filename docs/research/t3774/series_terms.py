"""T-3774 fit attempt 2, step 2: three more first-order regressors for the rest of attempt 1's fit, from bd0's series
loop (v stored, terms t0 = d v, t_j = 2k v^(2j+1)/(2j+1)):
  psum  every partial sum stored to double, from fl(t0) on (Excel keeping the running sum in memory)
  ejs   every ej = ej v^2 stored
  v2    v^2 stored once (fl(v v)) and used for every term
Each enters as the change in bd0 against the exact series (both sides, p and q), scaled like pmf_regress.py.
usage: tools/.venv/bin/python -I docs/research/t3774/series_terms.py SHOW_FILE  (from the repo root; SHOW_FILE as in
../t3447/pmf_regress.py)"""
from __future__ import annotations

import sys

sys.path.insert(0, str(__import__("pathlib").Path(__file__).resolve().parent))
import runconst as rc  # noqa: E402

pr, mpf = rc.pr, rc.mpf
EXTRA = ["psum", "ejs", "v2"]


def effects(k: float, m: mpf) -> list[mpf]:
    kk = mpf(k)
    d = kk - m
    v = pr.rnd(d / pr.rnd(kk + m, 53), 53)
    vv = v * v
    terms, ej = [d * v], 2 * kk * v
    ej_s, ej_v2, alt_s, alt_v2 = ej, ej, [], []
    v2s = pr.rnd(vv, 53)
    for j in range(1, 60):
        ej *= vv
        ej_s = pr.rnd(ej_s * vv, 53)
        ej_v2 *= v2s
        terms.append(ej / (2 * j + 1))
        alt_s.append(ej_s / (2 * j + 1))
        alt_v2.append(ej_v2 / (2 * j + 1))
        if abs(ej) < abs(d) * mpf(2) ** -80:
            break
    exact = sum(terms)
    s = pr.rnd(terms[0], 53)
    for t in terms[1:]:
        s = pr.rnd(s + t, 53)
    return [s - exact, sum(alt_s) - sum(terms[1:]), sum(alt_v2) - sum(terms[1:])]


def main() -> None:
    data = list(rc.rows(open(sys.argv[1]).read()))
    w = len(pr.NAMES)
    xs = []
    for _, _, x, _, _, (k, nk, m_p, m_q, scale) in data:
        ex = [-float(t) * scale for t in effects(k, m_p)] + [-float(t) * scale for t in effects(nk, m_q)]
        xs.append(x + ex)
    base = [0, w, 2, w + 2]
    nx = len(EXTRA)
    for label, yi in (("excel", 3), ("engine", 4)):
        ys = [r[yi] for r in data]
        print(label)
        for extra in [[]] + [[i] for i in range(nx)] + [list(range(nx))]:
            cols = base + [c for e in extra for c in (2 * w + e, 2 * w + nx + e)]
            coef, rms = pr.lstsq(xs, ys, cols)
            names = [pr.NAMES[c % w] + "pq"[c // w] if c < 2 * w else EXTRA[(c - 2 * w) % nx] + "pq"[(c - 2 * w) // nx]
                     for c in cols]
            print("  " + " ".join(f"{nm} {c:+.2f}" for nm, c in zip(names, coef)) + f"  rms {rms:.2f}")


if __name__ == "__main__":
    main()
