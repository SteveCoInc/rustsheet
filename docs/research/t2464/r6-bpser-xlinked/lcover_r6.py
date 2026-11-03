"""Per fixed shape of t3769-r6-bpser-beta (L constant along x): invert every cell into its interval of L (r2's
lscan.l_interval: w3 series, alx = fl(a LN x), E = EXP of the extended z, product stored) and find the constant L
that covers the most cells. Prints that best L and GL53's L in units of 2^-53 from the accurate ln B, and how many
cells each covers. If no constant L covers nearly all cells, the x-linked residual is not in L.
usage: lcover_r6.py"""
import sys
from collections import defaultdict

import common as cm

sys.path.insert(0, str(cm.HERE.parent / "r2-bpser-small"))
import lscan  # noqa: E402


def best_cover(ivs):
    ev = sorted([(lo, 0) for lo, _ in ivs] + [(hi, 1) for _, hi in ivs])
    cur = best = 0
    bl = None
    for i, (v, t) in enumerate(ev):
        cur += 1 if t == 0 else -1
        if cur > best:
            best, bl = cur, (v, ev[i + 1][0])
    return best, bl


def main() -> None:
    shapes = defaultdict(list)
    for part, b, a, x, e in cm.beta_rows():
        shapes[(part, b, a)].append((x, e))
    U = lscan.U
    for (part, b, a), cells in sorted(shapes.items()):
        ivs = [iv for iv in (lscan.l_interval(a, b, x, e) for x, e in cells) if iv]
        best, bl = best_cover(ivs)
        acc = lscan.candidates(a, b)["acc"]
        gl = cm.MODEL.lnb_w3(a, b)
        mid = (bl[0] + bl[1]) / 2
        cov_gl = sum(lo <= gl <= hi for lo, hi in ivs)
        print(f"{part} b0={b} a0={a}: n={len(cells)} inverted={len(ivs)}  best L covers {best} at "
              f"{float((mid - acc) / U):+.3f} (width {float((bl[1] - bl[0]) / U):.3f});  GL53 L covers {cov_gl} at "
              f"{float((gl - acc) / U):+.3f}")


if __name__ == "__main__":
    main()
