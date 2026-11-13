#!/usr/bin/env python3
"""Checks compare.py without Excel: python3 -I check.py

It builds a small workbook with two charts, then variants of it that stand for what a save may do, and holds
what compare.py says of each pair:
- the same package: every column `identical`;
- other series ids and nothing else: `ids only`, with how many changed;
- another gap width: `differs`, and the two lines are listed;
- the chart parts under other names: still `identical`, because a chart is found from its sheet;
- a style part dropped and a colours part changed: `gone` and `differs`;
- a frame moved: the frame `differs`;
- a chart removed: `gone`; a chart added: `new`;
- part names in another case than the relationships give them: the charts are still found;
- a saved workbook that is missing from the directory: `not returned`.
It also holds README.md's table of inputs against its list of sha256 lines.
"""

import contextlib
import io
import re
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))  # probe_package.py, beside the directories
sys.path.insert(0, str(HERE))
import compare  # noqa: E402
from probe_package import (  # noqa: E402
    HEAD,
    MAIN,
    OFFICE,
    PACKAGE,
    fail,
    workbook_relationships,
    write_package,
)

CHART = "http://schemas.openxmlformats.org/drawingml/2006/chart"
DRAWING = "http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing"
STYLE = "http://schemas.microsoft.com/office/2011/relationships"
IDS = ("{00000000-1111-4111-8111-111111111111}", "{00000001-1111-4111-8111-111111111111}")


def relationships(*entries: tuple[str, str, str]) -> str:
    """A relationships part: (id, type, target) in order."""
    body = "".join(f'<Relationship Id="{i}" Type="{kind}" Target="{target}"/>' for i, kind, target in entries)
    return f'{HEAD}<Relationships xmlns="{PACKAGE}/relationships">{body}</Relationships>'


def chart(ids: tuple[str, ...] = IDS, gap: int = 219) -> str:
    series = "".join(
        f'<c:ser><c:idx val="{n}"/><c:extLst><c:ext><c16:uniqueId val="{i}"/></c:ext></c:extLst></c:ser>'
        for n, i in enumerate(ids)
    )
    return (
        f'{HEAD}<c:chartSpace xmlns:c="{CHART}" xmlns:c16="http://schemas.microsoft.com/office/drawing/2014/chart">'
        f'<c:chart><c:plotArea><c:barChart>{series}<c:gapWidth val="{gap}"/></c:barChart></c:plotArea></c:chart>'
        "</c:chartSpace>"
    )


def anchor(reference: str, row: int) -> str:
    return (
        f"<xdr:twoCellAnchor><xdr:from><xdr:row>{row}</xdr:row></xdr:from><xdr:graphicFrame>"
        f'<a:graphicData><c:chart xmlns:c="{CHART}" r:id="{reference}"/></a:graphicData>'
        "</xdr:graphicFrame></xdr:twoCellAnchor>"
    )


def workbook(
    names: tuple[str, str] = ("chart1.xml", "chart2.xml"), **changes: object
) -> tuple[tuple[str, str], ...]:
    """The parts of the workbook: a sheet whose drawing frames two charts, the first with a style part and a
    colours part. `changes` replace a part's text by its name, or drop it with `None`; `names` are the two
    chart parts' names."""
    first, second = names
    frames = anchor("rId1", 1) + anchor("rId2", 20)
    parts = {
        "xl/workbook.xml": f'{HEAD}<workbook xmlns="{MAIN}" xmlns:r="{OFFICE}"><sheets>'
        '<sheet name="Data &amp; charts" sheetId="1" r:id="rId1"/></sheets></workbook>',
        "xl/_rels/workbook.xml.rels": workbook_relationships(1),
        "xl/worksheets/sheet1.xml": f'{HEAD}<worksheet xmlns="{MAIN}"><sheetData/></worksheet>',
        "xl/worksheets/_rels/sheet1.xml.rels": relationships(
            ("rId1", f"{OFFICE}/drawing", "../drawings/drawing1.xml")
        ),
        "xl/drawings/drawing1.xml": f'{HEAD}<xdr:wsDr xmlns:xdr="{DRAWING}" xmlns:r="{OFFICE}">{frames}</xdr:wsDr>',
        "xl/drawings/_rels/drawing1.xml.rels": relationships(
            ("rId1", f"{OFFICE}/chart", f"../charts/{first}"),
            ("rId2", f"{OFFICE}/chart", f"/xl/charts/{second}"),
        ),
        f"xl/charts/{first}": chart(),
        f"xl/charts/_rels/{first}.rels": relationships(
            ("rId1", f"{STYLE}/chartStyle", "style1.xml"), ("rId2", f"{STYLE}/chartColorStyle", "colors1.xml")
        ),
        "xl/charts/style1.xml": f"{HEAD}<cs:chartStyle/>",
        "xl/charts/colors1.xml": f"{HEAD}<cs:colorStyle/>",
        f"xl/charts/{second}": chart(IDS[:1]),
    }
    for name, text in changes.items():
        if text is None:
            del parts[name]
        else:
            parts[name] = str(text)
    return tuple(parts.items())


def written(directory: Path, name: str, parts: tuple[tuple[str, str], ...]) -> Path:
    path = directory / name
    with contextlib.redirect_stdout(
        io.StringIO()
    ):  # write_package prints the sha256, which nothing here names
        write_package(path, parts)
    return path


def cells(row: str) -> list[str]:
    return [cell.strip() for cell in row.strip("|").split("|")]


def expect(got: object, want: object, what: str) -> None:
    if got != want:
        fail(f"{what}: {got!r}, not {want!r}")


def main() -> None:
    sheet = "Data & charts"
    kept = ["identical", "identical", "identical", "none", "identical"]
    plain = ["identical", "none", "none", "none", "identical"]
    drawing = "xl/drawings/drawing1.xml"
    cases: dict[str, tuple[tuple[tuple[str, str], ...], list[list[str]]]] = {
        "the same package": (workbook(), [kept, plain]),
        "other ids": (
            workbook(**{"xl/charts/chart1.xml": chart((IDS[0], "{00000001-2222-4222-8222-222222222222}"))}),
            [["ids only (1 of 2 changed)", *kept[1:]], plain],
        ),
        "another gap width": (
            workbook(**{"xl/charts/chart2.xml": chart(IDS[:1], 150)}),
            [kept, ["differs", *plain[1:]]],
        ),
        "other part names": (workbook(("chart7.xml", "chart9.xml")), [kept, plain]),
        "style dropped, colours changed": (
            workbook(
                **{"xl/charts/style1.xml": None, "xl/charts/colors1.xml": f"{HEAD}<cs:colorStyle id='11'/>"}
            ),
            [["identical", "gone", "differs", "none", "identical"], plain],
        ),
        "a frame moved": (
            workbook(**{drawing: dict(workbook())[drawing].replace("<xdr:row>20<", "<xdr:row>21<")}),
            [kept, ["identical", "none", "none", "none", "differs"]],
        ),
        "a chart removed": (
            workbook(**{drawing: dict(workbook())[drawing].replace(anchor("rId2", 20), "")}),
            [kept, ["gone", "none", "none", "none", "gone"]],
        ),
    }
    with tempfile.TemporaryDirectory() as scratch:
        directory = Path(scratch)
        before = written(directory, "before.xlsx", workbook())
        for what, (parts, want) in cases.items():
            rows, notes = compare.compare("w", before, written(directory, "after.xlsx", parts))
            expect(
                [cells(row)[1:3] for row in rows], [[sheet, "1"], [sheet, "2"]], f"{what}: the charts found"
            )
            expect([cells(row)[3:] for row in rows], want, what)
            listed = [line.strip() for line in notes[1:]]
            gap = ['-<c:gapWidth val="219"/>', '+<c:gapWidth val="150"/>']
            expect(listed, gap if what == "another gap width" else [], f"{what}: the differences listed")

        # A chart the save added is `new`.
        fewer = written(directory, "fewer.xlsx", cases["a chart removed"][0])
        rows, _ = compare.compare("w", fewer, before)
        expect(
            [cells(row)[1:4] for row in rows],
            [[sheet, "1", "identical"], [sheet, "2", "new"]],
            "a chart added",
        )

        # A sheet part named in another case than its relationships part, as one input has them: the workbook
        # names `Sheet1.xml`, and the sheet's relationships are in `sheet1.xml.rels`.
        lower, upper = "worksheets/sheet1.xml", "worksheets/Sheet1.xml"
        renamed = tuple((name.replace(lower, upper), text.replace(lower, upper)) for name, text in workbook())
        expect(
            sorted(name for name, _ in renamed if "heet1.xml" in name),
            ["xl/worksheets/Sheet1.xml", "xl/worksheets/_rels/sheet1.xml.rels"],
            "the renamed part",
        )
        rows, _ = compare.compare("w", written(directory, "case.xlsx", renamed), before)
        expect([cells(row)[3:] for row in rows], [kept, plain], "part names in another case")

        # Two directories: `name.xlsx` against `name.web.xlsx`, and a workbook that did not come back.
        inputs, saved = directory / "inputs", directory / "saved"
        inputs.mkdir()
        saved.mkdir()
        for name in ("a.xlsx", "b.xlsx"):
            written(inputs, name, workbook())
        written(saved, "a.web.xlsx", workbook())
        found = [(label, after is not None) for label, _, after in compare.pairs(inputs, saved)]
        expect(found, [("a.xlsx", True), ("b.xlsx", False)], "the pairs of two directories")

    readme = (HERE / "README.md").read_text(encoding="utf-8")
    listed = re.findall(r"^[0-9a-f]{64}  (\S+)$", readme, re.M)
    expect(len(listed), 22, "the sha256 lines")
    expect(len(set(listed)), 22, "the sha256 lines' names, each once")
    table = readme.split("## The inputs")[1].split("```")[0]
    named = set(re.findall(r"`([\w.-]+\.xlsx)`", table))
    stray = sorted(named - set(listed))
    expect(stray, [], "inputs the table names that have no sha256 line")
    family = [name for name in listed if name.startswith("officejs-chart_")]
    expect(len(family), 10, "the officejs-chart goldens")
    unnamed = sorted(set(listed) - named - set(family))
    expect(unnamed, [], "sha256 lines the table does not name")
    print(
        f"ok: {len(cases)} kinds of save told apart, a chart added, part names in another case, two directories; "
        f"{len(listed)} inputs in the table and in the list"
    )


if __name__ == "__main__":
    main()
