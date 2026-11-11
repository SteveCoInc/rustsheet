#!/usr/bin/env python3
"""Split a pending rustsheet-case/v1 probe case into smaller workbooks for the excel-graph oracle.

usage: python3 split_cases.py <in.case.json> <out_dir> [--max-rows 6] [--terms 131072]

One part per sheet. A sheet holding a HYPGEOM.DIST cdf whose lower-tail sum has more than --terms
terms (x - max(0, n + K - N) + 1) is split by row instead: its other data rows go in one `-light`
part and the heavy rows in parts of at most --max-rows rows, in sheet order. Every part keeps row 1
of its sheet (the header) and the whole `Precision` sheet. Sheet names, cell addresses, formulas and
the order of `check` are the original's, so predictions keyed by Sheet!A1 apply unchanged. The
script refuses a sheet whose formulas read another row or another sheet. Standard library only.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ADDR = re.compile(r"^([A-Z]{1,3})(\d+)$")
REF = re.compile(r"(?<![A-Za-z_.!$])\$?([A-Z]{1,3})\$?(\d+)(?![\d(A-Za-z_])")
CDF = re.compile(r"HYPGEOM\.DIST\(([^,()]+),([^,()]+),([^,()]+),([^,()]+),\s*TRUE\s*\)", re.I)
ARITH = re.compile(r"^[0-9eE.+\-*/^() ]+$")
CARRIED = "Precision"


def row_of(addr: str) -> int:
    m = ADDR.match(addr)
    if not m:
        raise SystemExit(f"unexpected address {addr!r}")
    return int(m.group(2))


def number(token: str, cells: dict) -> float | None:
    token = token.strip()
    if ADDR.match(token):
        token = str(cells.get(token, ""))
    if not token or not ARITH.match(token):
        return None
    try:
        return float(eval(token.replace("^", "**"), {"__builtins__": {}}, {}))  # arithmetic only (ARITH)
    except Exception:
        return None


def cdf_terms(formula: str, cells: dict) -> list[float]:
    out = []
    for m in CDF.finditer(formula):
        args = [number(t, cells) for t in m.groups()]
        if None in args:
            out.append(float("inf"))  # unresolved: treat as heavy
            continue
        x, n, k, pop = args
        out.append(x - max(0.0, n + k - pop) + 1)
    return out


def ranges(rows: list[int]) -> str:
    rows = sorted(rows)
    parts, start, prev = [], rows[0], rows[0]
    for r in rows[1:] + [None]:
        if r is not None and r == prev + 1:
            prev = r
            continue
        parts.append(f"{start}" if start == prev else f"{start}-{prev}")
        if r is not None:
            start = prev = r
    return ", ".join(parts)


def dump(obj: dict, like: str) -> str:
    for ensure_ascii in (False, True):
        if json.dumps(json.loads(like), indent=1, ensure_ascii=ensure_ascii) == like:
            return json.dumps(obj, indent=1, ensure_ascii=ensure_ascii)
    return json.dumps(obj, indent=1, ensure_ascii=False)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("case", type=Path)
    ap.add_argument("out", type=Path)
    ap.add_argument("--max-rows", type=int, default=6)
    ap.add_argument("--terms", type=float, default=2**17)
    a = ap.parse_args()
    text = a.case.read_text()
    case = json.loads(text)
    cid = a.case.name.removesuffix(".case.json")
    assert case.get("format") == "rustsheet-case/v1", "not a rustsheet-case/v1 file"
    assert "expected" not in case or not case["expected"], "case has expected values; it is not pending"
    names = [s["name"] for s in case["sheets"]]
    carried = [s for s in case["sheets"] if s["name"] == CARRIED]
    check = case["check"]
    parts: list[tuple[str, dict, list[int] | None]] = []  # (suffix, sheet, rows or None for whole)
    for sheet in case["sheets"]:
        if sheet["name"] == CARRIED:
            continue
        cells = sheet["cells"]
        heavy_rows, rows = set(), set()
        for addr, v in cells.items():
            r = row_of(addr)
            rows.add(r)
            if isinstance(v, str) and v.startswith("="):
                if any(f"{n}!" in v for n in names) or "!" in v:
                    raise SystemExit(f"{cid} {sheet['name']}!{addr} reads another sheet")
                body = re.sub(r'"[^"]*"', "", v)
                for m in REF.finditer(body):
                    if int(m.group(2)) != r:
                        raise SystemExit(f"{cid} {sheet['name']}!{addr} reads another row: {v}")
                if any(t > a.terms for t in cdf_terms(v, cells)):
                    heavy_rows.add(r)
        slug = sheet["name"].lower()
        if not heavy_rows:
            parts.append((slug, sheet, None))
            continue
        light = sorted(rows - heavy_rows - {1})
        if light:
            parts.append((f"{slug}-light", sheet, light))
        heavy = sorted(heavy_rows)
        for i in range(0, len(heavy), a.max_rows):
            parts.append((f"{slug}-{i // a.max_rows + 1:02d}", sheet, heavy[i : i + a.max_rows]))
    a.out.mkdir(parents=True, exist_ok=True)
    seen_cells, seen_check = set(), set()
    for suffix, sheet, rows in parts:
        if rows is None:
            sub = dict(sheet["cells"])
            where = f"sheet {sheet['name']} (all of it)"
        else:
            keep = set(rows) | {1}
            sub = {k: v for k, v in sheet["cells"].items() if row_of(k) in keep}
            where = f"sheet {sheet['name']} rows {ranges(rows)} (and its header row 1)"
        part_sheets = [{"name": sheet["name"], "cells": sub}] + [json.loads(json.dumps(s)) for s in carried]
        part_check = [
            q for q in check
            if (q.split("!", 1)[0] == sheet["name"] and q.split("!", 1)[1] in sub) or q.split("!", 1)[0] == CARRIED
        ]
        pid = f"{cid}-{suffix}"
        desc = (
            f"Part of {cid} (pending since #4123): {where}"
            + (f", plus the {CARRIED} sheet" if carried else "")
            + f". The whole {cid} workbook failed twice on excel-graph (Graph's calculate or createSession, "
            f"retries exhausted; #4124), so docs/research/probe-handoff-2026-11-11-wave2.md splits it into one "
            f"workbook per sheet, with each HYPGEOM.DIST cdf row over 2^17 summed terms in groups of at most "
            f"{a.max_rows} rows. Sheet names, cell addresses and formulas are {cid}'s, so its frozen predictions "
            f"apply cell for cell. {cid}: {case['description']}"
        )
        out = {"format": case["format"], "description": desc, "tags": case["tags"], "sheets": part_sheets, "check": part_check}
        (a.out / f"{pid}.case.json").write_text(dump(out, text))
        seen_cells |= {(sheet["name"], k) for k in sub}
        seen_check |= set(part_check)
        n_cdf = sum(len(CDF.findall(v)) for v in sub.values() if isinstance(v, str))
        print(f"{pid}\t{len(part_check)} checked\t{n_cdf} cdf\t{where}")
    want_cells = {(s["name"], k) for s in case["sheets"] if s["name"] != CARRIED for k in s["cells"]}
    assert seen_cells == want_cells, "parts do not cover the original's cells exactly"
    assert seen_check == set(check), "parts do not cover the original's check list exactly"
    print(f"{cid}: {len(parts)} parts cover all {len(want_cells)} cells and {len(check)} checked cells", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
