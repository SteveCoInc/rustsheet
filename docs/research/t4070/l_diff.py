"""T-4070 item 1: t2464-r2-bpser-small-shapes, w_bpser at b0 >= 8: where r2-bpser-small's GL53/glalgdiv model is
within 15 digits and the engine is not (or the reverse), diff the model's L = fl(GL(a0) + algdiv(a0, b0)) against the
engine's pieces.

The engine side needs no harness: its BETA.DIST value comes from `rustsheet-eval show --all` (the file given as
argv[1]), and its GAMMALN through the bindings (`excelmath.excel`). The model's GL is Excel's probed GAMMALN
(t2464-r2-bpser-small-gl, r2-bratio's gammaln probe), its algdiv TOMS's in x87 doubles (lib.Model.algdiv).

Usage (repo root, bindings built into the venv with maturin):
  rustsheet-eval show --all probes-numeric/t2464-r2-bpser-small-shapes.case.json > shapes.txt
  <venv>/bin/python docs/research/t4070/l_diff.py shapes.txt
"""
import re
import sys
from collections import Counter
from pathlib import Path

RESEARCH = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(RESEARCH / "t2464" / "r2-bpser-small"))
sys.path.insert(0, str(RESEARCH / "t2464"))
import excelmath as em  # noqa: E402
import lib as bsl  # noqa: E402
from score_lib import tally  # noqa: E402

V53 = bsl.V(nofpser=True, noapser=True, lg="GL53/glalgdiv")


def engine_values(path: str) -> dict:
    """{row: engine value} from `rustsheet-eval show --all` output."""
    out, cell = {}, None
    for line in open(path):
        m = re.match(r"Calc!A(\d+)\s", line)
        if m:
            cell = int(m.group(1))
        m = re.match(r"\s+actual\s*:\s*(\S+)", line)
        if m and cell is not None:
            try:
                out[cell] = float(m.group(1))
            except ValueError:
                pass
    return out


def w15(m, e) -> bool:
    return tally([(m, e)])[3] == 1


def main() -> None:
    eng = engine_values(sys.argv[1])
    rows = bsl.load_probe(("shapes",))
    meta = __import__("json").load(open(bsl.HERE / "t2464-r2-bpser-small-shapes.meta.json"))
    row_of = {(blk, x, a, b): i for i, blk, x, a, b in meta}
    pts = [(r["a"], r["b"], r["x"], r["y"]) for r in rows]
    res = bsl.run(pts, V53)
    sel = []
    for r, k in zip(rows, res):
        # score_branches.py's group: bratio's w_bpser branch, b0_class by the cell's max(a, b).
        if k[2].split("/")[0] != "w_bpser" or max(r["a"], r["b"]) < 8:  # "/sw": after bratio's swap
            continue
        i = row_of[(r["blk"], r["x"], r["a"], r["b"])]
        sel.append((i, r, k[0], eng.get(i)))
    n = len(sel)
    model = tally((m, r["excel"]) for _, r, m, _ in sel)
    engine = tally((e, r["excel"]) for _, r, _, e in sel)
    print(f"w_bpser b0>=8 cells: {n}\n  model ex/u4/w15: {model[1:]}\n  engine ex/u4/w15: {engine[1:]}")
    same = sum(m == e for _, _, m, e in sel)
    print(f"  engine == model (bits): {same} of {n}")
    # The bpser call's own (a0, b0) after bratio's swap: the trace's last call.
    diff = [(i, r, m, e, k) for (i, r, m, e), k in
            zip(sel, [res[pts.index((r["a"], r["b"], r["x"], r["y"]))] for _, r, _, _ in sel]) if m != e]
    a0s = sorted({min(k[3][-1][1], k[3][-1][2]) for *_, k in diff})
    gl_eng = dict(zip(a0s, em.excel("GAMMALN", a0s)))
    why = Counter()
    print("\n| row | a | b | x | Excel | model | engine | model w15 | engine w15 | GL(a0) model - engine (ulp) |")
    print("|---|---|---|---|---|---|---|---|---|---|")
    for i, r, m, e, k in diff:
        a0 = min(k[3][-1][1], k[3][-1][2])
        g_m, g_e = bsl.GAMMALN(a0), gl_eng[a0]
        dg = (g_m - g_e) / bsl.math.ulp(g_m) if g_m else 0.0
        why["GAMMALN(a0) differs" if g_m != g_e else "GAMMALN(a0) equal"] += 1
        if w15(m, r["excel"]) != w15(e, r["excel"]):
            print(f"| A{i} | {r['a']} | {r['b']} | {r['x']} | {r['excel']!r} | {m!r} | {e!r} | "
                  f"{w15(m, r['excel'])} | {w15(e, r['excel'])} | {dg:+.0f} |")
    print(f"\ncells where engine != model: {len(diff)}; {dict(why)}")
    probed = bsl.probed_gl()
    print(f"distinct a0 among them: {len(a0s)}; a0 in the GAMMALN probe t2464-r2-bpser-small-gl: "
          f"{sum(a in probed for a in a0s)}; GL fallback to math.lgamma: {len(bsl.bx._GL_MISSING)}")
    # Control: the model with the engine's GAMMALN at every a0 of the selection (b0 >= 8 takes algdiv, so a0 is
    # L's only GAMMALN argument) must give the engine's bits if GL(a0) is the whole difference.
    all_a0 = sorted({min(k[3][-1][1], k[3][-1][2]) for k in res if k[3]})
    bsl.bx._GL.update(zip(all_a0, em.excel("GAMMALN", all_a0)))
    bsl._MODELS.clear()
    res2 = bsl.run([(r["a"], r["b"], r["x"], r["y"]) for _, r, _, _ in sel], V53)
    same2 = sum(k[0] == e for k, (_, _, _, e) in zip(res2, sel))
    print(f"model with the engine's GAMMALN(a0): equal to the engine on {same2} of {n}; "
          f"ex/u4/w15 {tally((k[0], r['excel']) for k, (_, r, _, _) in zip(res2, sel))[1:]}")


if __name__ == "__main__":
    main()
