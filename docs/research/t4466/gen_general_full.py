"""Build probes/t4466-general-full, the fresh General sweep drawn after the room rule and the 15-digit E mantissa were
frozen (e85125343, 2026-11-08; decision 0188 R4.2): python3 gen_general_full.py CASE_PATH. random.Random(44665).

TEXT(v,"General") over the whole magnitude range, in eight groups of A-column rows:
- A1:A40    E only above: |v| in [1E+11, 1E+307), 1 to 15 significant digits.
- A41:A70   E only below: |v| in [1E-307, 1E-09).
- A71:A90   the 1E+11 boundary: 11-digit integers and halves near 99999999999.5, and values rounding up to 1E+11.
- A91:A110  the 1E-09 boundary: values a few units of the 9th decimal or 15th digit either side of 1E-09.
- A111:A150 ties at the 6th significant digit in the E range (7 digits ending in 5, and 6 nines ending in 5: the carry).
- A151:A170 ties at the 5th significant digit for exponents of 99 and up, either sign.
- A171:A210 the [1E-09, 1E-04) fixed-or-E band, drawn fresh.
- A211:A240 the fixed range [1E-04, 1E+11), 1 to 15 digits.
About a third of each group is negative. The frozen engine's answers are written to general_full_predictions.json
before the fill (predict with `rustsheet-eval show` on the unfilled case).
"""

import random
import sys

from caseio import write_case

rng = random.Random(44665)


def draw(lo, hi):
    """A value 10^u for u uniform in [lo, hi), cut to 1..15 significant digits."""
    v = 10 ** rng.uniform(lo, hi)
    return float(f"{v:.{rng.randint(1, 15)}g}")


def tie(exp, digits, carry=False):
    """A tie at the given significant digit: digits-1 random digits then 5, or nines then 5 for the carry."""
    if carry:
        body = "9" * (digits - 1)
    else:
        body = str(rng.randint(1, 9)) + "".join(str(rng.randint(0, 9)) for _ in range(digits - 2))
    m = body[0] + "." + body[1:] + "5"
    return float(f"{m}e{exp}")


def signed(v):
    return -v if rng.random() < 1 / 3 else v


values = []
values += [signed(draw(11, 307)) for _ in range(40)]
values += [signed(draw(-307, -9)) for _ in range(30)]
for _ in range(20):
    k = rng.choice(["int", "half", "up", "frac"])
    if k == "int":
        v = float(99999999999 - rng.randint(0, 50))
    elif k == "half":
        v = 99999999999.5 - rng.randint(0, 20)
    elif k == "up":
        v = float(f"99999{''.join(str(rng.randint(5, 9)) for _ in range(rng.randint(2, 8)))}e6")
    else:
        v = 1e11 + rng.choice([-1, 1]) * rng.randint(1, 999) / 10**rng.randint(0, 3)
    values.append(signed(v))
for _ in range(20):
    k = rng.choice(["below", "above", "digits"])
    if k == "below":
        v = 1e-9 - rng.randint(1, 50) * 1e-24
    elif k == "above":
        v = 1e-9 + rng.randint(1, 50) * 10.0 ** -rng.randint(18, 24)
    else:
        v = float(f"9.99999{rng.randint(0, 99999999)}e-10")
    values.append(signed(v))
for i in range(40):
    exp = rng.choice(list(range(11, 99)) + list(range(-98, -9)))
    values.append(signed(tie(exp, 7, carry=i % 5 == 0)))
for i in range(20):
    exp = rng.choice(list(range(99, 308)) + list(range(-307, -98)))
    values.append(signed(tie(exp, 6, carry=i % 5 == 0)))
values += [signed(draw(-9, -4)) for _ in range(40)]
values += [signed(draw(-4, 11)) for _ in range(30)]

cells = {f"A{i}": f'=TEXT({v!r},"General")' for i, v in enumerate(values, 1)}
if __name__ == "__main__":
    write_case(
        sys.argv[1],
        "T-4333: fresh General sweep over the whole magnitude range, drawn after the room rule and the 15-digit E "
        "mantissa were frozen (docs/research/t4466/gen_general_full.py; frozen engine answers in "
        "general_full_predictions.json, committed before the fill).",
        cells,
        {},
        "general_full_predictions.json",
    )
    print(len(cells), "cells")
