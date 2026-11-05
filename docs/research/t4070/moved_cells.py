"""T-4070 item 2: the probes-numeric cells T-2505's port (81c3d27e, #1991) moved off bit-exactness, scored today.

Inputs: three `rustsheet-eval run --suite probes-numeric` reports (eval/runs/latest/report.json copied aside):
  pre   the T-2505 parent 1d3ea444 (Cephes kernel), with that tree's suites
  post  T-2505 itself (81c3d27e), with that tree's suites
  now   origin/main, with today's suites
A report lists only the cells that are not bit-exact, so a cell of a case in a report but not among its cells is
bit-exact (docs/research/t3007/cell_diff.py's rule).

A moved cell is bit-exact in `pre` and not in `post`. Each is scored against r2-bpser-small's model (lib.py,
GL53/glalgdiv, no fpser or apser: the form the port follows) where its formula is a literal BETA.DIST cdf on [0, 1]:
- "now exact": today's engine is bit-exact again;
- "port's": the model is bit-exact and today's engine is not (a port bug, T-2847's rule);
- "model's too": neither is bit-exact;
- "engine only": today's engine is off, the model is too, but they give different values (counted in "model's too");
- "not modelled": not a literal BETA.DIST cdf (references, other functions, a non-unit interval); a literal is a
  number or w2-forensics' exact m*2^k (+ n*2^j) encoding.

Usage (repo root, the venv of docs/research/t4070/l_diff.py):
  <venv>/bin/python docs/research/t4070/moved_cells.py pre.json post.json now.json
"""
import json
import re
import sys
from collections import Counter, defaultdict
from fractions import Fraction
from pathlib import Path

RESEARCH = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(RESEARCH / "t2464" / "r2-bpser-small"))
sys.path.insert(0, str(RESEARCH / "t2464"))
sys.path.insert(0, str(RESEARCH / "t2464" / "r7-bpser-nondouble-z"))
import c7  # noqa: E402 - r7's split-z cell (zh + zl, T-3879's form)
import excelmath as em  # noqa: E402
import forms as r7forms  # noqa: E402
import lib as bsl  # noqa: E402

V53 = bsl.V(nofpser=True, noapser=True, lg="GL53/glalgdiv")
NUM = r"\s*(-?[0-9.]+(?:E[-+]?[0-9]+)?|\(?[0-9]+\*2\^-?[0-9]+(?:\+[0-9]+\*2\^-?[0-9]+)?\)?)\s*"
# w2-forensics' exact literal encoder writes a double as m*2^k or (m*2^k + n*2^j): both terms are exact doubles, so
# Excel's one addition rounds the exact sum once, which Fraction then float reproduces.
DYADIC = re.compile(r"([0-9]+)\*2\^(-?[0-9]+)")
BETA_CDF = re.compile(rf"^(?:_xlfn\.)?BETA\.DIST\({NUM},{NUM},{NUM},\s*TRUE\s*(?:,{NUM},{NUM})?\)$", re.I)


def load(path):
    """({case id}, {(case, sheet, cell): cell}) of one report."""
    report = json.loads(Path(path).read_text())
    cases = {c["id"] for c in report["cases"]}
    cells = {(c["id"], x["sheet"], x["cell"]): x for c in report["cases"] for x in c.get("cells", [])}
    return cases, cells


def number(text: str) -> float:
    terms = DYADIC.findall(text)
    if not terms:
        return float(text)
    return float(sum(Fraction(int(m)) * Fraction(2) ** int(k) for m, k in terms))


def literal_cdf(formula: str):
    m = BETA_CDF.match(formula.lstrip("="))
    if not m:
        return None
    x, a, b, lo, hi = m.groups()
    if (lo, hi) not in ((None, None), ("0", "1")):
        return None
    return number(x), number(a), number(b)


def main() -> None:
    (pre_cases, pre), (post_cases, post), (now_cases, now) = (load(p) for p in sys.argv[1:4])
    moved = [k for k, c in post.items() if k[0] in pre_cases and k not in pre and c.get("actual", {}).get("t") == "n"]
    print(f"cells bit-exact before T-2505 and off after it: {len(moved)}")
    pts, idx = [], []
    for k in moved:
        p = literal_cdf(post[k].get("formula", ""))
        if p:
            x, a, b = p
            pts.append((a, b, x, 1.0 - x))
            idx.append(k)
    model = dict(zip(idx, bsl.run(pts, V53)))
    cls, by_case, by_fn = Counter(), defaultdict(Counter), defaultdict(Counter)
    ulps = Counter()
    rows = []
    for k in moved:
        exp = post[k]["expected"]["v"]
        fn = re.match(r"=?(?:_xlfn\.)?([A-Z.0-9]+)\(", (post[k].get("formula") or "").upper())
        fn = fn.group(1) if fn else "other (no call)"
        if k[0] not in now_cases:
            c = "case gone"
        elif k not in now:
            c = "now exact"
        else:
            nowv = now[k]["actual"].get("v")
            if k not in model:
                c = "not modelled"
            elif model[k][0] == exp:
                c = "port's"
                rows.append((k, exp, model[k][0], nowv))
            else:
                c = "model's too"
                if model[k][0] != nowv:
                    cls["  of which engine != model"] += 1
            try:
                ulps[min(int(round(abs(float(nowv) - exp) / bsl.math.ulp(exp))), 9)] += 1
            except (TypeError, ValueError, ZeroDivisionError):
                ulps["?"] += 1
        cls[c] += 1
        by_case[k[0].split("/")[1].removesuffix(".case.json")][c] += 1
        by_fn[fn][c] += 1
    print("\n| class | cells |\n|---|---|")
    for c, n in sorted(cls.items(), key=lambda t: t[0].startswith(" ")):
        print(f"| {c} | {n} |")
    print("\nToday's ulp distance of the cells still off (9 = 9 or more):", dict(sorted(ulps.items(), key=str)))
    print("\n| function | " + " | ".join(CLS) + " |\n|---|" + "---|" * len(CLS))
    for fn, c in sorted(by_fn.items()):
        print(f"| {fn} | " + " | ".join(str(c[x]) for x in CLS) + " |")
    print("\n| case | " + " | ".join(CLS) + " |\n|---|" + "---|" * len(CLS))
    for case, c in sorted(by_case.items(), key=lambda t: -sum(t[1].values())):
        print(f"| {case} | " + " | ".join(str(c[x]) for x in CLS) + " |")
    # Control, as l_diff.py's: the model with the engine's GAMMALN at every argument it takes. A port's cell that then
    # gives the engine's value is GAMMALN's last digits, not the port.
    eng_gl: dict = {}

    def engine_gl(v: float) -> float:
        if v not in eng_gl:
            eng_gl[v] = em.excel("GAMMALN", [v])[0]
        return eng_gl[v]

    bsl.GAMMALN = bsl.bx.GAMMALN = engine_gl
    bsl._MODELS.clear()
    keys = [k for k, *_ in rows]
    swapped = {k: v[0] for k, v in zip(keys, bsl.run([pts[idx.index(k)] for k in keys], V53))}
    gl_owned = {k for k, _, _, n in rows if swapped[k] == n}
    print(f"\nport's cells that the model reproduces with the engine's GAMMALN: {len(gl_owned)} of {len(rows)}")
    # The rest: lib's model predates T-3879's split z (min(a, b) < 1, b0 < 8; the engine's `bpser_series`). r7's
    # split form with the engine's GAMMALN in L (GL53), and with Excel's (c7's own L), on the same bpser call.
    split_eng = split_exc = 0
    for k, e, _, n in rows:
        if k in gl_owned:
            continue
        res = model[k]
        _, a, b, x = res[3][-1]
        lo, hi = min(a, b), max(a, b)
        if lo >= 1.0 or hi >= 8.0:
            continue
        flip = res[2] in ("w_bpser/sw", "w1_bpser")  # the cell is the complement of this call

        def cell_value(v: float, flip: bool = flip) -> float:
            return (0.5 - v) + 0.5 if flip else v

        L = float(bsl.mpf(bsl.sx("+", engine_gl(lo), bsl.sx("-", engine_gl(hi), engine_gl(bsl.sx("+", lo, hi))))))
        v_eng = cell_value(r7forms.forms(c7.cell("t4070", a, b, x, e, L=L))[r7forms.KEY])
        if v_eng == n:
            split_eng += 1
            swapped[k] = v_eng
        split_exc += cell_value(r7forms.forms(c7.cell("t4070", a, b, x, e))[r7forms.KEY]) == e
    print(f"the others under T-3879's split z: the engine's GAMMALN gives the engine's value on {split_eng}, "
          f"Excel's GAMMALN gives Excel's on {split_exc}")
    print(f"\n## port's cells (model bit-exact, engine off today): {len(rows)}")
    print("| case | cell | Excel | model | engine today | model, engine GAMMALN |\n|---|---|---|---|---|---|")
    for (case, sheet, cell), e, m, n in rows:
        g = swapped[(case, sheet, cell)]
        print(f"| {case.split('/')[1].removesuffix('.case.json')} | {sheet}!{cell} | {e!r} | {m!r} | {n!r} | "
              f"{'= engine' if g == n else repr(g)} |")


CLS = ("now exact", "port's", "model's too", "not modelled", "case gone")

if __name__ == "__main__":
    main()
