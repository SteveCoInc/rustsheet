#!/usr/bin/env python3
"""Lattice fingerprint of Excel's central inverse-erf core and of candidate rationals (T-5112, round 8).

    python3 docs/research/t5112/nsi_kernel_lab.py excel        # Excel's three val3 ladders: error, Q, c
    python3 docs/research/t5112/nsi_kernel_lab.py remez        # (re)write remez-t-0-0.5.json (about a minute)
    python3 docs/research/t5112/nsi_kernel_lab.py candidates   # the same fingerprint for each candidate
    python3 docs/research/t5112/nsi_kernel_lab.py score-val4   # after the val4 fill: runs, controls, spectra

Needs mpmath. Run from the repository root. Nothing here is engine code and no coefficient here is Excel's: the Remez rationals are
structure generators (their doubles are not Excel's doubles; excel-kernel-playbook.md, section 1 item 5).

Model under test: x = |2p-1|, t = fl(x*x), w = fl(x*fl(P(t)/Q(t))) with P and Q evaluated by Horner in
binary64, every step stored, Q(0) = 1. The core w is read as SQRT(GAMMA.INV(x,0.5,1)).
The fingerprint of a run of consecutive doubles x_i is the pair (Q, c) that maximises
|mean exp(2 pi i (w_i - c*i)/Q)|, w_i the core as an integer count of ulps: the core moves as
c*i + Q*k_i with k_i small integers (the quantised rounding noise of the cancelling Horner sums).
"""

from __future__ import annotations

import cmath
import json
import math
import os
import statistics
import struct
import sys
from pathlib import Path

import mpmath
from mpmath import mp, mpf

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
CASES = Path(os.environ.get("NSI_CASES", ROOT / "eval/suites/probes/cases"))  # NSI_CASES: a scratch copy
REMEZ_JSON = HERE / "remez-t-0-0.5.json"
DEGREES = [(6, 6), (6, 7), (7, 6), (7, 7), (7, 8), (8, 7), (8, 8), (8, 9), (9, 8)]
mp.prec = 500


def bits(x: float) -> int:
    return struct.unpack("<q", struct.pack("<d", x))[0]


def from_bits(b: int) -> float:
    return struct.unpack("<d", struct.pack("<q", b))[0]


def ulp(x: float) -> float:
    return math.ldexp(1.0, math.frexp(abs(x))[1] - 53)


def horner(c: list[float], t: float) -> float:
    s = c[-1]
    for a in reversed(c[:-1]):
        s = s * t + a
    return s


def model(x: float, p: list[float], q: list[float]) -> float:
    t = x * x
    return x * (horner(p, t) / horner(q, t))


# ----------------------------------------------------------------------------- fingerprint
def fit_lattice(cores: list[float]) -> tuple[float, float, float]:
    """(coherence, Q, c) of a run of cores; Q in ulp, c in ulp per step."""
    b0 = bits(cores[0])
    ints = [bits(w) - b0 for w in cores]
    best = (0.0, 0.0, 0.0)
    for qi in range(700):
        quantum = 1.5 + qi * 0.025
        for ci in range(221):
            c = ci * 0.01
            s = abs(sum(cmath.exp(2j * cmath.pi * (v - c * i) / quantum) for i, v in enumerate(ints))) / len(
                ints
            )
            if s > best[0]:
                best = (s, quantum, c)
    return best


def error_ulps(xs: list[float], cores: list[float]) -> list[float]:
    return [float((mpf(w) - mpmath.erfinv(mpf(x))) / mpf(ulp(w))) for x, w in zip(xs, cores)]


def spectrum(cores: list[float], smooth: int = 2) -> list[float]:
    """Unit-norm power spectrum of the de-trended core integers, boxcar-smoothed over 2*smooth+1 bins."""
    b0 = bits(cores[0])
    ints = [bits(w) - b0 for w in cores]
    n = len(ints)
    mx, my = (n - 1) / 2, sum(ints) / n
    slope = sum((i - mx) * (y - my) for i, y in enumerate(ints)) / sum((i - mx) ** 2 for i in range(n))
    r = [y - my - slope * (i - mx) for i, y in enumerate(ints)]
    roots = [cmath.exp(-2j * cmath.pi * k / n) for k in range(n)]
    power = [abs(sum(v * roots[(k * i) % n] for i, v in enumerate(r))) ** 2 for k in range(1, n // 2 + 1)]
    if smooth:
        power = [
            sum(power[max(0, k - smooth) : k + smooth + 1])
            / (min(len(power), k + smooth + 1) - max(0, k - smooth))
            for k in range(len(power))
        ]
    norm = math.sqrt(sum(v * v for v in power))
    return [v / norm for v in power]


def overlap(a: list[float], b: list[float]) -> float:
    return sum(x * y for x, y in zip(a, b))


# ----------------------------------------------------------------------------- Excel's ladders
def excel_ladders() -> list[tuple[str, list[float], list[float]]]:
    oracle = json.loads((CASES / "val3-nsi-01.oracle.json").read_text())["sheets"]["P"]
    out = []
    for cols in ("ABCD", "GHIJ", "MNOP"):
        xs = [oracle[f"{cols[0]}{r}"]["v"] for r in range(2, 66)]
        cores = [math.sqrt(oracle[f"{cols[3]}{r}"]["v"]) for r in range(2, 66)]
        out.append((f"val3-nsi-01 P!{cols[0]}:{cols[3]}", xs, cores))
    return out


def report(name: str, xs: list[float], cores: list[float]) -> None:
    errors = error_ulps(xs, cores)
    coherence, quantum, c = fit_lattice(cores)
    print(
        f"{name}: x0={xs[0]!r} error mean {statistics.mean(errors):+.2f} sd {statistics.pstdev(errors):.2f}"
        f" min {min(errors):+.2f} max {max(errors):+.2f} ulp | Q={quantum:.2f} c={c:.2f} coherence={coherence:.2f}",
        flush=True,
    )


# ----------------------------------------------------------------------------- Remez generator
_MACLAURIN: list[mpf] = []


def scaled(t: mpf) -> mpf:
    """erfinv(sqrt t)/sqrt t, the function the rational approximates."""
    t = mpf(t)
    if t < mpf("1e-6"):
        if not _MACLAURIN:
            c = [mpf(1)]
            for k in range(1, 12):
                c.append(sum(c[m] * c[k - 1 - m] / ((m + 1) * (2 * m + 1)) for m in range(k)))
            h = mpmath.sqrt(mpmath.pi) / 2
            _MACLAURIN.extend(c[k] / (2 * k + 1) * h ** (2 * k + 1) for k in range(12))
        return sum(a * t**k for k, a in enumerate(_MACLAURIN))
    s = mpmath.sqrt(t)
    return mpmath.erfinv(s) / s


def polyval(c: list[mpf], t: mpf) -> mpf:
    return mpmath.polyval(list(reversed(c)), t)


def remez(m: int, n: int, a: float, b: float, grid: int = 3000) -> tuple[list[mpf], list[mpf], mpf]:
    """Relative-error minimax P_m/Q_n of `scaled` on [a, b], Q(0) = 1 (second Remez algorithm)."""
    a, b = mpf(a), mpf(b)
    count = m + n + 2
    ts = [(a + b) / 2 - (b - a) / 2 * mpmath.cos(mpmath.pi * i / (count - 1)) for i in range(count)]
    q = [mpf(1)] + [mpf(0)] * n
    level = mpf(0)
    mesh = [a + (b - a) * (1 - mpmath.cos(mpmath.pi * i / grid)) / 2 for i in range(grid + 1)]
    f_mesh = [scaled(t) for t in mesh]
    p: list[mpf] = []
    worst = mpf(0)
    for _ in range(30):
        fs = [scaled(t) for t in ts]
        for _ in range(60):  # P - f*Q - f*s*E*Q_prev = 0 at the reference, iterated on Q_prev
            matrix = mpmath.matrix(count, count)
            rhs = mpmath.matrix(count, 1)
            for i, (t, ft) in enumerate(zip(ts, fs)):
                for j in range(m + 1):
                    matrix[i, j] = t**j
                for j in range(1, n + 1):
                    matrix[i, m + j] = -ft * t**j
                matrix[i, m + n + 1] = -ft * (-1) ** i * polyval(q, t)
                rhs[i] = ft
            sol = mpmath.lu_solve(matrix, rhs)
            p = [sol[j] for j in range(m + 1)]
            q = [mpf(1)] + [sol[m + j] for j in range(1, n + 1)]
            done = abs(sol[m + n + 1] - level) < abs(sol[m + n + 1]) * mpf(10) ** -25
            level = sol[m + n + 1]
            if done:
                break
        err = [polyval(p, t) / polyval(q, t) / ft - 1 for t, ft in zip(mesh, f_mesh)]
        ext = [
            (mesh[i], e)
            for i, e in enumerate(err)
            if i in (0, grid) or (e - err[i - 1]) * (err[i + 1] - e) <= 0
        ]
        ext += [(t, polyval(p, t) / polyval(q, t) / ft - 1) for t, ft in zip(ts, fs)]
        ext.sort(key=lambda item: item[0])
        alt: list[tuple[mpf, mpf]] = []
        for t, e in ext:
            if alt and (alt[-1][1] > 0) == (e > 0):
                if abs(e) > abs(alt[-1][1]):
                    alt[-1] = (t, e)
            else:
                alt.append((t, e))
        while len(alt) > count:
            alt.pop(0 if abs(alt[0][1]) < abs(alt[-1][1]) else -1)
        worst = max(abs(e) for _, e in alt)
        if len(alt) < count:
            break
        ts = [t for t, _ in alt]
        if worst / min(abs(e) for _, e in alt) - 1 < mpf("1e-6"):
            break
    return p, q, worst


def write_remez() -> None:
    out = {}
    for m, n in DEGREES:
        p, q, worst = remez(m, n, 0, 0.5)
        out[f"{m},{n}"] = {
            "p": [mpmath.nstr(v, 40) for v in p],
            "q": [mpmath.nstr(v, 40) for v in q],
            "max_relative_error": mpmath.nstr(worst, 12),
        }
        print(f"({m},{n}) max relative error {mpmath.nstr(worst, 6)}", flush=True)
    REMEZ_JSON.write_text(json.dumps(out, indent=1) + "\n")


def candidates() -> dict[str, tuple[list[float], list[float]]]:
    data = json.loads(REMEZ_JSON.read_text())
    return {k: ([float(s) for s in v["p"]], [float(s) for s in v["q"]]) for k, v in data.items()}


# ----------------------------------------------------------------------------- val4 scoring
def val4_runs() -> list[tuple[str, list[float], list[float], list[float]]]:
    """(label, xs, gammas, cores) per run of the filled val4 cases; x is re-read from the formula."""
    sys.path.insert(0, str(HERE))
    import next_val4_nsi as request  # noqa: PLC0415

    out = []
    layout = [
        (f"val4-nsi-0{part + 1}", "R", [(f"A-run {16 * part + r}", 1024 * r + 1, 1024) for r in range(16)])
        for part in range(4)
    ]
    layout.append(("val4-nsi-05", "R", [("below sqrt(1/2)", 1, 8192), ("below 1/2", 8193, 8192)]))
    layout.append(("val4-nsi-06", "E", [(f"ladder {k}", 256 * k + 1, 256) for k in range(28)]))
    layout.append(("val4-nsi-07", "R", [(f"B-run {r}", 1024 * r + 1, 1024) for r in range(16)]))
    built = request.build()
    for name, sheet, runs in layout:
        sidecar = CASES / f"{name}.oracle.json"
        if not sidecar.exists():
            print(f"{name}: not filled", flush=True)
            continue
        values = json.loads(sidecar.read_text())["sheets"][sheet]
        cells = next(s for s in built[name]["sheets"] if s["name"] == sheet)["cells"]
        for label, first, length in runs:
            xs, gammas = [], []
            for row in range(first, first + length):
                formula = cells[f"A{row}"]
                hi, rest = formula.split("(")[2].split("*")
                lo, k = rest.split("+")[1].split(")/2^")
                xs.append((int(hi) * 67108864 + int(lo)) / 2.0 ** int(k.split(",")[0]))
                gammas.append(values[f"A{row}"]["v"])
            out.append((f"{name} {label}", xs, gammas, [math.sqrt(g) for g in gammas]))
    return out


def score_val4() -> None:
    cands = candidates()
    for label, xs, gammas, cores in val4_runs():
        unique = sum(w * w == g for w, g in zip(cores, gammas))
        head = cores[:64]
        coherence, quantum, c = fit_lattice(head)
        errors = error_ulps(xs[:256], cores[:256])
        line = (
            f"{label}: x0={xs[0]!r} cores squaring back {unique}/{len(cores)}; first 64 rows Q={quantum:.2f}"
            f" c={c:.2f} coherence={coherence:.2f}; first 256 rows error mean {statistics.mean(errors):+.2f}"
            f" sd {statistics.pstdev(errors):.2f} ulp"
        )
        if "A-run" in label or "below" in label:
            observed = spectrum(cores[:1024])
            scores = {
                k: overlap(observed, spectrum([model(x, p, q) for x in xs[:1024]]))
                for k, (p, q) in cands.items()
            }
            line += " | spectral overlap " + " ".join(f"({k}) {v:.2f}" for k, v in scores.items())
        print(line, flush=True)


def main() -> int:
    command = sys.argv[1] if len(sys.argv) > 1 else ""
    if command == "excel":
        for name, xs, cores in excel_ladders():
            report(name, xs, cores)
    elif command == "remez":
        write_remez()
    elif command == "candidates":
        ladders = excel_ladders()
        for key, (p, q) in candidates().items():
            for name, xs, _ in ladders:
                report(f"Remez ({key}) on the x of {name}", xs, [model(x, p, q) for x in xs])
    elif command == "score-val4":
        score_val4()
    else:
        print(__doc__)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
