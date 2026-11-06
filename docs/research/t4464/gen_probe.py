"""T-4464: write probes/t4464-groupby-spent-filter-shapes (GROUPBY's spent filter beside a lift, three shapes).

Run from the repo root: python3 -I docs/research/t4464/gen_probe.py. The Data sheet is
probes/t3840-groupby-lifted-filter-consumed's (the gen-m7 batch Data sheet), so the cells compare with T-3840's.
Each call is at A<r> on sheet G with its label in E<r>; calls are 4 rows apart, so a result of up to 3x3 fits.

Filters: DROP (`Data!C1:C12<0.2`) drops rows, as in T-3840. EMPTY (`Data!D1:D12="zz"`) keeps no row: column D holds
numbers and booleans only. GT99 is the task's `Data!C1:C12>99`, which is not a no-row filter in Excel: besides C10
(123456789.123), column C's text and FALSE compare greater than any number, so it keeps five rows (C1, C3, C5, C10,
C11).
"""

import json

SRC = "eval/suites/probes/cases/t3840-groupby-lifted-filter-consumed.case.json"
OUT = "eval/suites/probes/cases/t4464-groupby-spent-filter-shapes.case.json"

DROP = "(Data!C1:C12<0.2)"
EMPTY = '(Data!D1:D12="zz")'
GT99 = "(Data!C1:C12>99)"
SUM = "_xleta.SUM"


def gb(rel: str = "", hdr: str = "0", depth: str = "1", filt: str = "", keys: str = "Data!C1:C12") -> str:
    """A GROUPBY call over the T-3840 data; empty arguments are omitted, as T-3840's calls omit them."""
    args = [keys, "Data!D1:D12", SUM, hdr, depth, "", filt, rel]
    while args and args[-1] == "":
        args.pop()
    return "_xlfn.GROUPBY(" + ",".join(args) + ")"


def calls() -> list[tuple[str, str]]:
    out: list[tuple[str, str]] = []
    # (a) a filter that keeps no row, beside a lifted field_relationship, field_headers and total depth.
    for rel in ("{0,1;0,1}", '{"x",0;0,1}', "{2,0;0,1}"):
        for name, filt in (("none", ""), ("drop", DROP), ("empty", EMPTY)):
            out.append((gb(rel=rel, filt=filt), f"(a) rel {rel} filter {name}"))
    for hdr in ("{0,1;0,0}", '{"x",0;0,0}'):
        for name, filt in (("none", ""), ("drop", DROP), ("empty", EMPTY)):
            out.append((gb(hdr=hdr, filt=filt), f"(a) hdr {hdr} filter {name}"))
    for depth in ("{1,2;1,1}", "{5,1;1,1}"):
        for name, filt in (("none", ""), ("drop", DROP), ("empty", EMPTY)):
            out.append((gb(depth=depth, filt=filt), f"(a) depth {depth} filter {name}"))
    out.append((gb(filt=EMPTY), "(a) scalar call, filter empty"))
    out.append((gb(rel="{0,1;0,1}", filt=GT99), "(a) rel {0,1;0,1} filter C>99 (keeps C10 and the text/FALSE rows)"))
    # (b) a lifted table relationship (1) with total depth 1, 2 and -1 beside a dropping filter; one and two key
    # columns (a subtotal needs a second key column).
    for keys in ("Data!C1:C12", "Data!B1:C12"):
        for rel in ("{1,0;0,1}", "{0,1;0,1}", '{"x",1;0,1}'):
            for depth in ("1", "2", "-1"):
                for name, filt in (("none", ""), ("drop", DROP)):
                    out.append((gb(rel=rel, depth=depth, filt=filt, keys=keys), f"(b) keys {keys} rel {rel} depth {depth} filter {name}"))
    # (c) a lifted argument that is itself the result of an inner lifted GROUPBY or a MAP of GROUPBY calls.
    inner = {
        "none": "0*IFERROR(" + gb(rel="{0,1;0,1}") + ",0)",
        "drop": "0*IFERROR(" + gb(rel="{0,1;0,1}", filt=DROP) + ",0)",
    }
    mapped = {
        name: "0*IFERROR(MAP({0,1;0,1},LAMBDA(r,INDEX(" + gb(rel="r", filt=filt) + ",1,1))),0)"
        for name, filt in (("none", ""), ("drop", DROP))
    }
    for kind, table in (("inner lift", inner), ("inner MAP", mapped)):
        for iname, arg in table.items():
            for oname, filt in (("none", ""), ("drop", DROP)):
                out.append((gb(rel=arg, filt=filt), f"(c) rel = {kind} (inner filter {iname}), outer filter {oname}"))
                out.append((gb(depth="1+" + arg, filt=filt), f"(c) depth = 1+{kind} (inner filter {iname}), outer filter {oname}"))
    # An inner GROUPBY inside the outer call's function: it runs during each outer element's call.
    for iname, irel in (("scalar", ""), ("lifted", "{0,1;0,1}")):
        fn = "LAMBDA(v,ROWS(" + gb(rel=irel, filt=DROP) + "))"
        for oname, filt in (("none", ""), ("drop", DROP)):
            call = gb(rel="{0,1;0,1}", filt=filt).replace(SUM, fn)
            out.append((call, f"(c) function = LAMBDA with {iname} inner GROUPBY (inner filter drop), outer filter {oname}"))
    # The inner MAP alone: does a scalar GROUPBY call spend its filter for the next MAP iteration?
    for name, filt in (("none", ""), ("drop", DROP)):
        out.append(("MAP({0,1;0,1},LAMBDA(r,INDEX(" + gb(rel="r", filt=filt) + ",1,1)))", f"(c) MAP of scalar GROUPBY, filter {name}"))
    return out


def main() -> None:
    with open(SRC) as fh:
        data = next(s for s in json.load(fh)["sheets"] if s["name"] == "Data")
    cells: dict[str, object] = {}
    for i, (formula, label) in enumerate(calls()):
        r = 1 + 4 * i
        cells[f"A{r}"] = "=" + formula
        cells[f"E{r}"] = label
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "GROUPBY's spent filter beside a lift (T-4464): (a) a filter_array that keeps no row beside a lifted "
            "field_relationship, field_headers or total depth; (b) a lifted table field_relationship (1) with total "
            "depth 1, 2 and -1 beside a dropping filter; (c) a lifted argument computed by an inner lifted GROUPBY or "
            "a MAP of GROUPBY calls. Each call spills at A<r> (up to 3x3), label in E<r>. Data is the gen-m7 batch "
            "Data sheet. Generated by docs/research/t4464/gen_probe.py."
        ),
        "tags": ["M7", "probe", "dynamic-arrays", "T-4464"],
        "expected": {},
        "sheets": [{"name": "G", "cells": cells}, data],
    }
    with open(OUT, "w") as fh:
        json.dump(case, fh, indent=1, ensure_ascii=False)
        fh.write("\n")
    print(len(calls()), "calls")


if __name__ == "__main__":
    main()
