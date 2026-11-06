"""T-4437 second fresh sweep, written after the second freeze (freeze2.md): L140 bup+bgrat BETA.DIST
is 0 at n >= 3 bup terms when bup's sum W < 2^-970. Untouched shapes, walked in ln of the true tail
by 0.05 nats from ln 2^-970 + 0.525 (row 1) to ln 2^-970 - 0.525 (row 22): the rule predicts a
number through row 11 and 0 from row 12. Columns I and J are n = 2 families, evidence for T-4564
only (out of the rule's domain).
Run: cd tools && uv run python ../docs/research/t4437/gen_fresh2.py
"""

import mpmath as mp
from gen_fresh import sweep, write

FAMILIES = [("A", 1_500_000, 3.25), ("B", 6_000_000, 3.75), ("C", 2_500_000, 4.25), ("D", 7_000_000, 5.0),
            ("E", 1_200_000, 6.5), ("F", 9_000_000, 7.75), ("G", 3_500_000, 7.25), ("H", 4_500_000, 3.5),
            ("I", 3_000_000, 2.5), ("J", 7_000_000, 2.75)]


def main() -> None:
    cells, lns = sweep(FAMILIES, 22, mp.mpf("0.525"), mp.mpf("0.05"))
    write(
        "t4437-beta-zero-fresh2",
        "T-4437 second fresh sweep after the second freeze (docs/research/t4437/freeze2.md): BETA.DIST(x,a,b,TRUE) "
        "on L140 bup+bgrat at untouched shapes A (1.5E6, 3.25), B (6E6, 3.75), C (2.5E6, 4.25), D (7E6, 5), "
        "E (1.2E6, 6.5), F (9E6, 7.75), G (3.5E6, 7.25), H (4.5E6, 3.5), walked in ln of the true tail by -0.05 "
        "from ln 2^-970 + 0.525 (row 1) to ln 2^-970 - 0.525 (row 22); the rule predicts a number through row 11 "
        "and 0 from row 12. I (3E6, 2.5) and J (7E6, 2.75) are n = 2, evidence for T-4564. "
        "docs/research/t4437/gen_fresh2.py writes it.",
        cells,
        lns,
        "fresh2_ln_minus_edge.json",
    )


if __name__ == "__main__":
    main()
