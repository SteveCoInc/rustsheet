#!/usr/bin/env python3
"""Checks generate.py without Excel: python3 -I check.py

It writes the workbook and its control to a scratch directory and reads them back. It says nothing about
Excel's values: those are what the request asks for. Checked here:
- each file's sha256 is the one README.md names;
- the workbook holds the sheets L and E, in that order, and only L has `transitionEvaluation="1"`;
- the control is the same package but for that attribute, which neither of its sheets has;
- both sheets hold the same input cells, and the ones listed below are of the kind README.md says: a text that
  looks like a number or like a formula is text, an empty string is a cell, an empty cell is no cell;
- every row of README.md's table is on both sheets, its id in column P and its formula in column Q, from row 2
  in the table's order, with `OTHER` naming the other sheet;
- row X3 reads the other sheet's C1, wherever the table puts it;
- the two names are defined as README.md says;
- baseline-rustsheet.tsv has one line for each row of the table, in the table's order.
"""
import re
import sys
from pathlib import Path
from xml.etree import ElementTree

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))  # probe_package.py, beside the directories
from probe_package import MAIN, fail, generated  # noqa: E402

NS = {"m": MAIN}

# Address, kind and content of the input cells whose kind matters. "absent" is an empty cell.
KINDS = """
A1 text abc
A2 text 12
A3 text
A4 number 5
A5 boolean 1
A6 absent
A8 error #N/A
B1 text <one space>
B4 text TRUE
B5 text 0
B6 text 5
B7 formula ""
K2 text =Ap
X2 text =a
U4 number 5
U6 text
U7 absent
AA2 formula U2="a"
S1 number 5
N1 text 12
"""


def cells_of(xml: bytes) -> dict[str, tuple[str, str]]:
    """address -> (kind, content) for every cell of a sheet part."""
    out = {}
    for c in ElementTree.fromstring(xml).iterfind(".//m:c", NS):
        formula, value, inline = c.find("m:f", NS), c.find("m:v", NS), c.find("m:is/m:t", NS)
        if formula is not None:
            out[c.get("r")] = ("formula", formula.text or "")
        elif c.get("t") == "inlineStr":
            out[c.get("r")] = ("text", (inline.text or "") if inline is not None else "")
        else:
            kind = {"b": "boolean", "e": "error"}.get(c.get("t"), "number")
            out[c.get("r")] = (kind, value.text if value is not None else "")
    return out


def main() -> None:
    readme = (HERE / "README.md").read_text(encoding="utf-8")
    rows = re.findall(r"^\| ([A-Z]\d+) \| `(.*)` \| .* \|$", readme, flags=re.MULTILINE)
    if len(rows) != 135:
        fail(f"README.md lists {len(rows)} rows, not the 135 this check was written for")
    packages = generated(HERE)
    if list(packages) != ["transition-evaluation-control.xlsx", "transition-evaluation.xlsx"]:
        fail(f"generate.py wrote {list(packages)}")
    control_sha, control = packages["transition-evaluation-control.xlsx"]
    sha, parts = packages["transition-evaluation.xlsx"]
    flagged = parts["xl/worksheets/sheet1.xml"].replace(b'<sheetPr transitionEvaluation="1"/>', b"", 1)
    if control != {**parts, "xl/worksheets/sheet1.xml": flagged} or b"sheetPr" in flagged:
        fail("the control is not the workbook without the attribute")

    workbook = ElementTree.fromstring(parts["xl/workbook.xml"])
    if [s.get("name") for s in workbook.iterfind("m:sheets/m:sheet", NS)] != ["L", "E"]:
        fail("the workbook's sheets are not L and E")
    names = {n.get("name"): n.text for n in workbook.iterfind("m:definedNames/m:definedName", NS)}
    if names != {"OnL": "L!$A$1+1", "OnE": "E!$A$1+1"}:
        fail(f"the defined names are {names}")
    sheets = {"L": parts["xl/worksheets/sheet1.xml"], "E": parts["xl/worksheets/sheet2.xml"]}
    flags = {}
    for name, xml in sheets.items():
        properties = ElementTree.fromstring(xml).find("m:sheetPr", NS)
        flags[name] = None if properties is None else properties.get("transitionEvaluation")
    if flags != {"L": "1", "E": None}:
        fail(f"transitionEvaluation is {flags}")

    held = {name: cells_of(xml) for name, xml in sheets.items()}
    table = {f"{column}{i + 2}" for i in range(len(rows)) for column in "PQ"}
    inputs = {name: {a: v for a, v in cells.items() if a not in table} for name, cells in held.items()}
    if inputs["L"] != inputs["E"] or len(inputs["L"]) != 70:
        fail(f"the sheets hold {len(inputs['L'])} and {len(inputs['E'])} input cells, or they differ")
    for line in KINDS.strip().splitlines():
        address, kind, *content = line.split(" ", 2)
        want = (kind, (content[0] if content else "").replace("<one space>", " "))
        got = inputs["L"].get(address, ("absent", ""))
        if got != want:
            fail(f"{address} is {got}, not {want}")

    c1 = [row_id for row_id, _ in rows].index("C1") + 2
    for name, other in (("L", "E"), ("E", "L")):
        for i, (row_id, formula) in enumerate(rows):
            row = i + 2
            if held[name].get(f"P{row}") != ("text", row_id):
                fail(f"{name}!P{row} is not the id {row_id}")
            want = formula.removeprefix("=").replace("OTHER", other)
            if held[name].get(f"Q{row}") != ("formula", want):
                fail(f"{name}!Q{row} is {held[name].get(f'Q{row}')}, not the formula of {row_id}")
            if row_id == "X3" and want != f"TYPE({other}!Q{c1})":
                fail(f"X3 reads {want}, and C1 is in row {c1}")
    baseline = (HERE / "baseline-rustsheet.tsv").read_text(encoding="utf-8").splitlines()
    if [line.split("\t")[0] for line in baseline] != ["id", *(row_id for row_id, _ in rows)]:
        fail("baseline-rustsheet.tsv's ids are not the table's, in its order")
    # Last, so that a change of the workbook is first told by what changed in it.
    named = re.sub(r"\s+", " ", readme)
    for name, digest in (("transition-evaluation-control.xlsx", control_sha), ("transition-evaluation.xlsx", sha)):
        if f"`{name}` (sha256 `{digest}`)" not in named:
            fail(f"README.md does not name {name} with its sha256 {digest}")
    print(f"ok: {len(rows)} rows on two sheets, {len(inputs['L'])} input cells each, and the control; sha256 {sha}")


if __name__ == "__main__":
    main()
