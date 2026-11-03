"""Generator for probes/t3820-lower-sigma-next-char (T-3820).

Run from the repository root:
`python3 docs/research/t3820/gen_case.py eval/suites/probes/cases/t3820-lower-sigma-next-char.case.json`.
"""

import json
import sys

ROWS = [
    # (label, code point)
    ("end of text", None),
    ("space", 0x20),
    ("a", 0x61),
    ("Sigma U+03A3", 0x3A3),
    ("sigma U+03C3", 0x3C3),
    ("final sigma U+03C2", 0x3C2),
    ("U+0526 Cyrillic Ext letter", 0x526),
    ("U+10C7 Georgian", 0x10C7),
    ("U+13A0 Cherokee", 0x13A0),
    ("U+AB70 Cherokee small", 0xAB70),
    ("U+1C90 Mtavruli", 0x1C90),
    ("U+A7C0 Latin Ext-D", 0xA7C0),
    ("U+10400 Deseret", 0x10400),
    ("U+05D0 alef", 0x5D0),
    ("U+0627 Arabic alef", 0x627),
    ("U+0E01 Thai ko kai", 0xE01),
    ("U+3042 hiragana a", 0x3042),
    ("U+4E00 ideograph", 0x4E00),
    ("U+AC00 Hangul", 0xAC00),
    ("U+02B0 modifier h", 0x2B0),
    ("U+00AA ordinal a", 0xAA),
    ("U+00B2 superscript 2", 0xB2),
    ("U+00BD one half", 0xBD),
    ("U+216B Roman twelve", 0x216B),
    ("U+2460 circled one", 0x2460),
    ("U+0663 Arabic-Indic 3", 0x663),
    ("U+0969 Devanagari 3", 0x969),
    ("U+FF10 fullwidth 0", 0xFF10),
    ("U+0483 Cyrillic titlo", 0x483),
    ("U+05B0 Hebrew sheva", 0x5B0),
    ("U+093E Devanagari aa sign", 0x93E),
    ("U+0E48 Thai mai ek", 0xE48),
    ("U+1AB0 comb. ext", 0x1AB0),
    ("U+1DC0 comb. suppl", 0x1DC0),
    ("U+20D0 comb. for symbols", 0x20D0),
    ("U+FE20 half mark", 0xFE20),
    ("U+0301 acute", 0x301),
    ("U+00AD soft hyphen", 0xAD),
    ("U+200B ZWSP", 0x200B),
    ("U+200C ZWNJ", 0x200C),
    ("U+200D ZWJ", 0x200D),
    ("U+2060 word joiner", 0x2060),
    ("U+FEFF BOM", 0xFEFF),
    ("U+FE0F VS16", 0xFE0F),
    ("U+2018 left quote", 0x2018),
    ("U+2019 right quote", 0x2019),
    ("U+1F600 emoji", 0x1F600),
    ("U+0027 apostrophe", 0x27),
    ("U+002D hyphen", 0x2D),
    ("U+0031 digit 1", 0x31),
]


def main() -> None:
    calc: dict[str, str] = {}
    data: dict[str, str] = {}
    for n, (label, cp) in enumerate(ROWS, start=1):
        tail = "" if cp is None else f"&UNICHAR({cp})"
        data[f"A{n}"] = f"=UNICHAR(913)&UNICHAR(931){tail}"
        data[f"B{n}"] = label
        calc[f"A{n}"] = f"=UNICODE(MID(LOWER(Data!A{n}),2,1))"
        calc[f"B{n}"] = f"=UNICODE(MID(PROPER(Data!A{n}),2,1))"
        calc[f"C{n}"] = f'=IFERROR(LEN(TEXTBEFORE("x"&Data!A{n},UNICHAR(963),1,1)),-1)'
    # BMP sweep: E(p+1) LOWER, F(p+1) PROPER over U+p00..U+pFF, one digit per code point:
    # 1 = medial sigma, 0 = final sigma, x = anything else, e = UNICHAR errors.
    for col, fn in (("E", "LOWER"), ("F", "PROPER")):
        for p in range(256):
            calc[f"{col}{p + 1}"] = (
                f"=LET(c,SEQUENCE(256,1,{p * 256}),x,UNICHAR(c),"
                f"u,IFERROR(UNICODE(MID({fn}(UNICHAR(913)&UNICHAR(931)&x),2,1)),0),"
                'CONCAT(IF(ISERROR(x),"e",IF(u=963,"1",IF(u=962,"0","x")))))'
            )
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-3820: which character after a capital sigma makes LOWER (and PROPER) give the medial sigma. "
            f"Data!A<n> = UNICHAR(913)&UNICHAR(931)&<char> (label in Data!B), rows 1..{len(ROWS)}; Calc!A<n> = "
            "UNICODE of LOWER's 2nd character (962 final, 963 medial), B the same over PROPER, C = "
            "LEN(TEXTBEFORE(\"x\"&text, medial sigma, 1, 1)) or -1. Calc!E1:E256 (LOWER) and F1:F256 (PROPER): BMP "
            "sweep, E(p+1) covers U+p00..U+pFF with the next character, one digit per code point: 1 medial, 0 final, "
            "x other, e UNICHAR error."
        ),
        "tags": ["M2", "probe", "text"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": calc}, {"name": "Data", "cells": data}],
    }
    with open(sys.argv[1], "w", encoding="utf-8") as f:
        json.dump(case, f, ensure_ascii=False, indent=1)


if __name__ == "__main__":
    main()
