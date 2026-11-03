"""Positive controls on t3769-r6-bpser-aux before any scoring: the LN model against =LN(x), and alx's two forms
(x87 stored sx("*", a, LN x), and the SSE double product) against the worksheet's =a0*LN(x).
usage: controls.py"""
import numpy as np

import common as cm


def main() -> None:
    ln = cm.measured("ln")
    print(f"LN: model == Excel on {sum(cm.LN(x) == v for x, v in ln.items())} / {len(ln)}")
    alx = cm.measured("alx")
    st = sum(cm.sx("*", a, ln[x]) == v for (a, x), v in alx.items())
    sse = sum(float(np.float64(a) * np.float64(ln[x])) == v for (a, x), v in alx.items())
    print(f"=a0*LN(x): x87 stored product {st} / {len(alx)}, double product {sse} / {len(alx)}")
    gl = cm.measured("gl")
    print(f"GAMMALN measured at {len(gl)} arguments")


if __name__ == "__main__":
    main()
