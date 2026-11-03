"""Generator for probes/t3584-match-fold-weightless (T-3584).

Run from the repository root:
`python3 docs/research/t3584/gen_case.py eval/suites/probes/cases/t3584-match-fold-weightless.case.json`.
"""

import json
import sys

calc: dict[str, str] = {}
data: dict[str, object] = {}

# --- Data!A1 = U+0524 (reference weightless candidate), A2 = empty text
data["A1"] = "=UNICHAR(1316)"
data["A2"] = '=""'


# --- Part 1 (Calc!A1:H<n>): pairs x vs y, one row each.
# Data!C = x, Data!D = y (formulas); Calc A label, B '=', C EXACT, D COUNTIF, E MATCH,
# F XMATCH(..,0), G VLOOKUP(..,FALSE) found?, H HLOOKUP found?
def u(cp: int) -> str:
    return f"UNICHAR({cp})"


pairs = [
    ("U+0524 vs U+10C7", u(0x524), u(0x10C7)),
    ("U+0524 vs U+0525", u(0x524), u(0x525)),
    ("U+AB70 vs U+1C90", u(0xAB70), u(0x1C90)),
    ("U+0524 vs empty", u(0x524), '""'),
    ("a&U+0524 vs a", '"a"&' + u(0x524), '"a"'),
    ("a&U+0524&b vs ab", '"a"&' + u(0x524) + '&"b"', '"ab"'),
    ("U+0524&U+10C7 vs U+0525", u(0x524) + "&" + u(0x10C7), u(0x525)),
    ("U+0524 vs U+0378 (unassigned)", u(0x524), u(0x378)),
    ("U+0378 vs empty", u(0x378), '""'),
    ("U+0524 vs a", u(0x524), '"a"'),
    ("U+0524 vs U+0524", u(0x524), u(0x524)),
    ("U+00AD vs empty", u(0xAD), '""'),
    ("a&U+00AD vs a", '"a"&' + u(0xAD), '"a"'),
    ("U+0524 vs U+00AD", u(0x524), u(0xAD)),
    ("U+200B vs empty", u(0x200B), '""'),
    ("U+0524 vs U+200B", u(0x524), u(0x200B)),
    ("U+AB7B vs U+AB70", u(0xAB7B), u(0xAB70)),
    ("U+AB7B vs U+13F5", u(0xAB7B), u(0x13F5)),
    ("U+AB7B vs empty", u(0xAB7B), '""'),
    ("U+AB7B vs U+13AB", u(0xAB7B), u(0x13AB)),
    ("U+1F05 vs U+1F0A", u(0x1F05), u(0x1F0A)),
    ("U+1F05 vs U+1F0D", u(0x1F05), u(0x1F0D)),
    ("U+1F05 vs empty", u(0x1F05), '""'),
    ("U+1F0A vs U+1F02", u(0x1F0A), u(0x1F02)),
    ("U+10400 vs U+10428", u(0x10400), u(0x10428)),
    ("U+10400 vs U+10401", u(0x10400), u(0x10401)),
    ("U+10400 vs empty", u(0x10400), '""'),
    ("U+10400 vs U+0524", u(0x10400), u(0x524)),
    ("U+E000 (private use) vs empty", u(0xE000), '""'),
    ("U+E000 vs U+0524", u(0xE000), u(0x524)),
    ("U+FFFF vs U+0524", u(0xFFFF), u(0x524)),
    ("empty vs U+0524", '""', u(0x524)),
]
for i, (label, x, y) in enumerate(pairs, start=1):
    data[f"C{i}"] = "=" + x
    data[f"D{i}"] = "=" + y
    X, Y = f"Data!C{i}", f"Data!D{i}"
    calc[f"A{i}"] = label
    calc[f"B{i}"] = f"={X}={Y}"
    calc[f"C{i}"] = f"=EXACT({X},{Y})"
    calc[f"D{i}"] = f"=COUNTIF({Y},{X})"
    calc[f"E{i}"] = f"=IFNA(MATCH({X},{Y},0),0)"
    calc[f"F{i}"] = f"=IFNA(XMATCH({X},{Y},0),0)"
    calc[f"G{i}"] = f"=NOT(ISNA(VLOOKUP({X},{Y},1,FALSE)))"
    calc[f"H{i}"] = f"=NOT(ISNA(HLOOKUP({X},{Y},1,FALSE)))"
n_pairs = len(pairs)

# --- Part 2 (Calc!J1:P<n>): per-character fold of the sweep's "other" pairs.
# Data!F = the character, Data!G..I = candidates. Calc J label, K..M MATCH(char, cand) found?,
# N MATCH(char, Data!A2 (empty)) found?, O MATCH(char, Data!A1 (U+0524)) found?, P char=cand1.
fold_rows: list[tuple[int, list[int]]] = [
    (0x3D0, [0x3B2, 0x392]),
    (0x3D1, [0x3B8, 0x398, 0x3F4]),
    (0x3D5, [0x3C6, 0x3A6]),
    (0x3D6, [0x3C0, 0x3A0]),
    (0x3F0, [0x3BA, 0x39A]),
    (0x3F1, [0x3C1, 0x3A1]),
    (0x3F5, [0x3B5, 0x395]),
    (0xB5, [0x3BC, 0x39C]),
    (0x1FBE, [0x3B9, 0x399]),
    (0x345, [0x3B9, 0x399]),
    (0x3C2, [0x3C3, 0x3A3]),
    (0x3F2, [0x3C3, 0x3F9]),
    (0x1C80, [0x432, 0x412]),
    (0x1C81, [0x434, 0x414]),
    (0x1C82, [0x43E, 0x41E]),
    (0x1C83, [0x441, 0x421]),
    (0x1C84, [0x442, 0x422]),
    (0x1C85, [0x442, 0x422]),
    (0x1C86, [0x44A, 0x42A]),
    (0x1C87, [0x463, 0x462]),
    (0x1C88, [0xA64B, 0xA64A, 0x2CF2]),
    (0x131, [0x69, 0x49]),
    (0x17F, [0x73, 0x53]),
    (0x1C5, [0x1C6, 0x1C4]),
    (0x1C8, [0x1C9, 0x1C7]),
    (0x1CB, [0x1CC, 0x1CA]),
    (0x1F2, [0x1F3, 0x1F1]),
    (0x1E9B, [0x1E61, 0x1E60]),
    (0x1E9E, [0xDF]),
    (0x212A, [0x6B, 0x4B]),
    (0x212B, [0xE5, 0xC5]),
    (0x2126, [0x3C9, 0x3A9]),
    (0x1F05, [0x1F0D, 0x1F0A, 0x1F02]),
    (0x1F15, [0x1F1D, 0x1F1A]),
    (0xA73B, [0xA73A, 0xA738]),
    (0xA767, [0xA766, 0xA764]),
    (0xA699, [0xA698, 0xA660]),
    (0xA69B, [0xA69A, 0xA660]),
    (0x3F3, [0x37F, 0x376]),
]
# Georgian Mtavruli U+1C90..U+1CBF (skip unassigned 1CBB-1CBC) vs Mkhedruli U+10D0.. and U+0524
for cp in range(0x1C90, 0x1CC0):
    if cp in (0x1CBB, 0x1CBC):
        continue
    fold_rows.append((cp, [cp - 0x1C90 + 0x10D0]))
for i, (cp, cands) in enumerate(fold_rows, start=1):
    data[f"F{i}"] = f"=UNICHAR({cp})"
    for col, c in zip("GHI", cands):
        data[f"{col}{i}"] = f"=UNICHAR({c})"
    calc[f"J{i}"] = f"U+{cp:04X} vs " + ",".join(f"U+{c:04X}" for c in cands)
    for col, dcol in zip("KLM", "GHI"):
        calc[f"{col}{i}"] = f"=ISNUMBER(MATCH(Data!F{i},Data!{dcol}{i},0))" if f"{dcol}{i}" in data else None
        if calc[f"{col}{i}"] is None:
            del calc[f"{col}{i}"]
    calc[f"N{i}"] = f"=ISNUMBER(MATCH(Data!F{i},Data!$A$2,0))"
    calc[f"O{i}"] = f"=ISNUMBER(MATCH(Data!F{i},Data!$A$1,0))"
    calc[f"P{i}"] = f"=Data!F{i}=Data!G{i}"
n_fold = len(fold_rows)

# --- Part 3 (Calc!R1:R256): BMP page sweep. Cell R(p+1) covers U+p00..U+pFF, one hex digit
# per code point: bit 1 MATCH(u, U+0524 cell), bit 2 MATCH(u, empty-text cell), bit 4 u="",
# bit 8 u=U+0524; "e" where UNICHAR errors.
for p in range(256):
    calc[f"R{p + 1}"] = (
        f"=LET(c,SEQUENCE(256,1,{p * 256}),x,UNICHAR(c),"
        "m,ISNUMBER(MATCH(x,Data!$A$1,0))+2*ISNUMBER(MATCH(x,Data!$A$2,0))"
        '+4*IFERROR(x="",FALSE)+8*IFERROR(x=Data!$A$1,FALSE),'
        'CONCAT(IF(ISERROR(x),"e",DEC2HEX(m))))'
    )

# --- Part 4 (Calc!T, U): equivalence classes within blocks. Data!K holds the block characters
# (one per row); Calc!T<row> = TEXTJOIN of the hex codes in the same block that MATCH(.., 0)
# finds equal to that row's character (itself included); Calc!U<row> = its code in hex.
blocks = [
    (0x370, 0x400),
    (0x1F00, 0x2000),
    (0xA720, 0xA800),
    (0xA640, 0xA6A0),
    (0x2C80, 0x2D00),
    (0x13A0, 0x1400),
    (0xAB70, 0xABC0),
    (0x10A0, 0x1100),
    (0x1C80, 0x1CC0),
    (0x2D00, 0x2D30),
]
row = 1
for lo, hi in blocks:
    first = row
    last = row + (hi - lo) - 1
    for cp in range(lo, hi):
        data[f"K{row}"] = f"=UNICHAR({cp})"
        row += 1
    for r in range(first, last + 1):
        calc[f"U{r}"] = f"U+{lo + r - first:04X}"
        calc[f"T{r}"] = (
            f'=TEXTJOIN(",",TRUE,IF(ISNUMBER(MATCH(Data!$K${first}:$K${last},Data!K{r},0)),'
            f'DEC2HEX(SEQUENCE({hi - lo},1,{lo}),4),""))'
        )
n_class = row - 1

case = {
    "format": "rustsheet-case/v1",
    "description": (
        "T-3584: MATCH's exact-match (type 0) fold per character and the characters Excel's "
        "comparison table seems to lack. Data!A1 = UNICHAR(1316) (U+0524), Data!A2 = empty text. "
        f"Part 1, Calc!A1:H{n_pairs}: pairs Data!C (x) vs Data!D (y), A label, B x=y, C EXACT, "
        "D COUNTIF(y,x), E MATCH(x,y,0) (0 = #N/A), F XMATCH(x,y,0), G VLOOKUP(x,y,1,FALSE) found, "
        f"H HLOOKUP found. Part 2, Calc!J1:P{n_fold}: Data!F char vs candidates Data!G:I, J label, "
        "K:M MATCH(char,cand,0) found, N MATCH(char, empty text) found, O MATCH(char, U+0524) found, "
        "P char=first candidate. Part 3, Calc!R1:R256: BMP page sweep, R(p+1) covers U+p00..U+pFF, "
        "one hex digit per code point: 1 = MATCH equals U+0524, 2 = MATCH equals empty text, "
        '4 = (u=""), 8 = (u=U+0524), e = UNICHAR errors. '
        f"Part 4, Calc!T1:U{n_class}: per character of blocks Greek, Greek Extended, Latin Ext-D, "
        "Cyrillic Ext-B, Coptic, Cherokee, Cherokee Supplement, Georgian, Cyrillic Ext-C + Georgian "
        "Mtavruli, Georgian Supplement (Data!K, one per row), T = the hex codes in its block that "
        "MATCH(..,0) equals it with (itself included), U its code."
    ),
    "tags": ["M2", "probe", "text", "lookup"],
    "expected": {},
    "sheets": [
        {"name": "Calc", "cells": calc},
        {"name": "Data", "cells": data},
    ],
}
json.dump(case, open(sys.argv[1], "w"), ensure_ascii=False, indent=1)
print(n_pairs, n_fold, n_class, len(calc), len(data))
