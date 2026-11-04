"""T-2893: which ix rows does the r2-bratio model send to L140 bup+bpser, and under which configuration does it
count 1,620?  Usage (from the repo root, after T-2506's data rebuild; python3 with mpmath and numpy):
    T2464_HARNESS=target/debug/deps/rustsheet_engine-<hash> python3 docs/research/t2893/trace.py
The harness (`functions/statistical/t2464_harness.rs`) was deleted by T-2977; RESULTS.md says how it was restored,
uncommitted, for this run. Output: trace.out.
"""
import sys
from collections import Counter
from dataclasses import replace
from pathlib import Path

R2 = Path(__file__).resolve().parent.parent / "t2464" / "r2-bratio"
sys.path.insert(0, str(R2))
import bratio_excel as bx  # noqa: E402
from score_lib import load, tally  # noqa: E402

L140 = "L140 bup+bpser"


def branches(rows, cfg):
    out = []
    for r in rows:
        try:
            w, w1, br = bx.kernel(r["a"], r["b"], r["x"], r["y"], cfg)
        except Exception as e:  # noqa: BLE001 - a research scan: count, do not stop
            w, w1, br = float("nan"), float("nan"), f"error:{type(e).__name__}"
        out.append((w, w1, br.split("/")[0]))
    return out


def main() -> None:
    rows = load(sets=("ix",))
    print("ix rows", len(rows))
    configs = {
        "BEST": bx.BEST,
        "BEST closed=False": replace(bx.BEST, closed=False),
        "BEST x29=0.29": replace(bx.BEST, x29=0.29),
        "TOMS_XL": bx.TOMS_XL,
        "R_LIKE": bx.R_LIKE,
    }
    res = {name: branches(rows, cfg) for name, cfg in configs.items()}
    best = res["BEST"]
    for name, rs in res.items():
        idx = [i for i, t in enumerate(rs) if t[2] == L140]
        pairs = [(rs[i][0] if rows[i]["side"] == "lower" else rs[i][1], rows[i]["excel"]) for i in idx]
        print(f"{name}: L140 bup+bpser {len(idx)}; ex/u4/w15 {tally(pairs)[1:]}")
    base = {i for i, t in enumerate(best) if t[2] == L140}
    for name in ("BEST closed=False",):
        other = {i for i, t in enumerate(res[name]) if t[2] == L140}
        extra = other - base
        print(f"{name} minus BEST: {len(extra)}; BEST's branch for them: "
              f"{dict(Counter(best[i][2] for i in extra))}; minus the other way {len(base - other)}")
    print("\n## The model's ix rows by BEST branch\n| branch | n | BEST ex/u4/w15 |\n|---|---|---|")
    by_branch(rows, best)
    engine_on_model_l140(rows, best)


def engine_on_model_l140(rows, best) -> None:
    """The engine's K(a, b, x, y) (T-2506's harness run) on every ix row BEST sends to any L140 branch."""
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "t2506"))
    from score_l140 import engine  # noqa: PLC0415 - needs T2464_HARNESS

    eng = engine(rows)
    names = {10: "L140 bup+bpser", 11: "L140 bup+bgrat", 12: "L140 bup+bup+bgrat"}
    print("\n## Engine and model on the model's L140 rows (by the model's branch)")
    print("| model branch | n | engine's branch for them | model ex/u4/w15 | engine ex/u4/w15 |\n|---|---|---|---|---|")
    total_m, total_e = [], []
    for br in names.values():
        idx = [i for i, t in enumerate(best) if t[2] == br]
        side = lambda i, w, w1: w if rows[i]["side"] == "lower" else w1  # noqa: E731
        m = [(side(i, best[i][0], best[i][1]), rows[i]["excel"]) for i in idx]
        e = [(side(i, eng[i][0], eng[i][1]), rows[i]["excel"]) for i in idx]
        total_m += m
        total_e += e
        ebr = Counter(names.get(eng[i][2], str(eng[i][2])) for i in idx)
        print(f"| {br} | {len(idx)} | {dict(ebr)} | {'/'.join(map(str, tally(m)[1:]))} | "
              f"{'/'.join(map(str, tally(e)[1:]))} |")
    print(f"| all L140 | {len(total_m)} | | {'/'.join(map(str, tally(total_m)[1:]))} | "
          f"{'/'.join(map(str, tally(total_e)[1:]))} |")



def by_branch(rows, rs):
    """Each BEST branch's ix count and score (to find the population behind 1,620 / 1,011)."""
    g = {}
    for r, (w, w1, br) in zip(rows, rs):
        g.setdefault(br, []).append((w if r["side"] == "lower" else w1, r["excel"]))
    for br in sorted(g):
        print(f"| {br} | {len(g[br])} | {'/'.join(map(str, tally(g[br])[1:]))} |")


if __name__ == "__main__":
    main()
