"""Score forms of E (and of the product E s) that carry z's low part on the cells of r6's beta probe, split by
double / non-double z and by shape. zh is z rounded to a double (ne nearest-even, dn down, up, tz toward zero),
zl = z - zh exactly (a double here: z needs 54-55 bits). E = Excel's EXP(zh) (a double argument: the measured model).
usage: forms.py [cells]  (default: r6's tune shapes; 'hold' r6's b0 = 4.75 hold-out; 'all' r6; 'r5' r5's probe)"""
import math
import sys
from collections import Counter, defaultdict

import c7

cm, mpf, fl53, fl64, sx = c7.cm, c7.mpf, c7.fl53, c7.fl64, c7.sx


KEY = "ne: fl(E s) fl(1+zl)"  # the split model


def zsplit(c: dict, how: str) -> tuple[float, float]:
    lo = c["zlo"]
    if not c["nd"]:
        return lo, 0.0
    hi = math.nextafter(lo, math.inf)
    zh = {"ne": float(c["z"]), "dn": lo, "up": hi, "tz": hi if lo < 0 else lo}[how]
    return zh, float(c["z"] - mpf(zh))


def forms(c: dict) -> dict:
    s, z = c["s"], c["z"]
    out = {"z64": sx("*", cm.EXPX(fl64(z)), s)}
    for how in ("ne", "dn", "up", "tz"):
        zh, zl = zsplit(c, how)
        E = cm.EXP(zh)
        P = sx("*", E, s)
        f = sx("+", 1.0, zl)
        out[f"{how}: E s"] = P
        out[f"{how}: fl(E s)(1+zl) ext"] = fl53(fl64(mpf(P) * fl64(1 + mpf(zl))))
        out[f"{how}: fl(E + E zl) s"] = sx("*", sx("+", E, sx("*", E, zl)), s)
        out[f"{how}: (E + E zl)ext s"] = fl53(fl64(fl64(mpf(E) + fl64(mpf(E) * zl)) * s))
        out[f"{how}: P + P zl"] = sx("+", P, sx("*", P, zl))
        out[f"{how}: P + P zl ext"] = fl53(fl64(mpf(P) + fl64(mpf(P) * zl)))
        out[f"{how}: E s + E s zl ext"] = fl53(fl64(fl64(mpf(E) * s) + fl64(fl64(mpf(E) * s) * zl)))
        out[f"{how}: E (s + s zl)"] = sx("*", E, sx("+", s, sx("*", s, zl)))
        out[f"{how}: fl(E s) EXP(zl)"] = sx("*", P, cm.EXP(zl))
        out[f"{how}: fl(E s) EXP(zl) ext"] = fl53(fl64(mpf(P) * cm.EXPX(fl64(mpf(zl)))))
        out[f"{how}: fl(E s) fl(1+zl)"] = sx("*", P, f)
        out[f"{how}: fl(E f) s"] = sx("*", sx("*", E, f), s)
        out[f"{how}: E fl(s f)"] = sx("*", E, sx("*", s, f))
        out[f"{how}: (E f)ext s"] = fl53(fl64(fl64(mpf(E) * f) * s))
        out[f"{how}: (E s)ext f"] = fl53(fl64(fl64(mpf(E) * s) * f))
    return out


def main() -> None:
    sel = sys.argv[1] if len(sys.argv) > 1 else "tune"
    tab = defaultdict(Counter)
    n = Counter()
    for c in c7.cells("r5" if sel == "r5" else "r6"):
        if sel not in ("all", "r5") and c["part"] != sel:
            continue
        kind = "nd" if c["nd"] else "dz"
        sh = f"{c['b']}:{c['a']}" if sel != "r5" else f"a0~{c['a']:.1f}"
        n[kind] += 1
        n[(kind, sh)] += 1
        for k, v in forms(c).items():
            if v == c["I"]:
                tab[k][kind] += 1
                tab[k][(kind, sh)] += 1
    shapes = sorted({k[1] for k in n if isinstance(k, tuple)})
    print(f"cells: {sel}; exact counts. dz = z is a double, nd = z is not.")
    print(f"{'form':30s} {'dz':>6s} {'nd':>6s} " + " ".join(f"{s:>9s}" for s in shapes))
    print(f"{'(cells)':30s} {n['dz']:6d} {n['nd']:6d} " + " ".join(f"{n[('nd', s)]:9d}" for s in shapes))
    for k in tab:
        t = tab[k]
        print(f"{k:30s} {t['dz']:6d} {t['nd']:6d} " + " ".join(f"{t[('nd', s)]:9d}" for s in shapes))


if __name__ == "__main__":
    main()
