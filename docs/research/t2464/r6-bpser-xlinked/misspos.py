"""Where along x GL53's misses fall at the a0 = 0.3 shapes (L fits there, lcover_r6.out): one character per cell in
x order (decimals 0.0100..0.0483, then the 128-double run from 0.0371): '.' exact, '+'/'-' model 1 ulp above/below,
'#' more. Also, per miss, whether s or E moved one ulp fixes it, and E's and s's exponents (binade).
usage: misspos.py"""
import math
from collections import Counter, defaultdict

import common as cm

sx = cm.sx


def main() -> None:
    shapes = defaultdict(list)
    for part, b, a, x, e in cm.beta_rows():
        shapes[(part, b, a)].append((x, e))
    for (part, b, a), cells in sorted(shapes.items(), key=lambda kv: (kv[0][2], kv[0][1])):
        if a != 0.3:
            continue
        dec = sorted(c for c in cells if len(repr(c[0])) <= 6)
        run = sorted(c for c in cells if len(repr(c[0])) > 6)
        line, fixes, bins = [], Counter(), Counter()
        for x, e in dec + run:
            p = cm.pieces(a, b, x)
            m = p["i"]
            k = round((m - e) / math.ulp(e))
            line.append("." if k == 0 else "+" if k == 1 else "-" if k == -1 else "#")
            if k:
                f = [f"s{j:+d}" for j in (-1, 1) if sx("*", p["e"], cm.step(p["s"], j)) == e]
                f += [f"E{j:+d}" for j in (-1, 1) if sx("*", cm.step(p["e"], j), p["s"]) == e]
                fixes[" ".join(f) or "none"] += 1
                bins[(math.frexp(p["e"])[1], math.frexp(p["s"])[1], math.frexp(m)[1])] += 1
        print(f"{part} b0={b} a0={a}")
        s = "".join(line)
        for i in range(0, len(s), 128):
            print("   ", s[i:i + 128])
        print("    fixes", dict(fixes.most_common()))
        print("    miss exponents (E, s, I):", dict(bins.most_common()))


if __name__ == "__main__":
    main()
