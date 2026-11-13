#!/usr/bin/env python3
"""Writes the workbooks of the probe request (README.md): one sheet of data, three defined names and one bar
chart whose series gives its name, categories and values each by a formula. The control spells the three as
cell ranges; one file spells the values alone as a defined name; each other file spells all three as names, in
one way.

Every cache in a chart part is stale on purpose (other texts and numbers than the cells hold), so that what
Excel plots says whether it read the formula or kept the cache. The chart part has the shape of the one
tools/src/rustsheet_tools/pivot_chart.py writes, which Excel for the web opens (probes/t4130-pivot-chart-cache).

Standard library only, and the same bytes at every run.
Usage: python3 -I generate.py <output directory>
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # probe_package.py, beside the directories
from probe_package import HEAD, MAIN, OFFICE, PACKAGE_RELATIONSHIPS  # noqa: E402
from probe_package import content_types, output_directory, relationships, styles  # noqa: E402
from probe_package import workbook_relationships, write_package  # noqa: E402

CHART = "http://schemas.openxmlformats.org/drawingml/2006/chart"
DRAWING = "http://schemas.openxmlformats.org/drawingml/2006/main"
SHEET_DRAWING = "http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing"
DRAWING_TYPES = "application/vnd.openxmlformats-officedocument"

# The cells: a header row, four categories, their values, and a second column of values for the step that
# redefines a name.
CATEGORIES, VALUES, OTHER = ("a", "b", "c", "d"), (10, 20, 15, 25), (1, 2, 3, 4)
# The names, and the cells each one is: the series' name, its categories and its values.
NAMES = (("Label", "Sheet1!$B$1"), ("Months", "Sheet1!$A$2:$A$5"), ("Sales", "Sheet1!$B$2:$B$5"))
# What the caches hold: nothing a cell holds.
STALE_NAME, STALE_CATEGORIES, STALE_VALUES = "stale", ("w", "x", "y", "z"), (901, 902, 903, 904)

# Each file: whether its names are the workbook's or the sheet's, how a chart formula spells a name, and which
# of the series' three formulas are spelled so (the others spell the cells).
ALL = tuple(name for name, _ in NAMES)
FILES = {
    "series-control.xlsx": ("workbook", "{}", ()),
    "series-workbook-name-values.xlsx": ("workbook", "[0]!{}", ("Sales",)),
    "series-workbook-name.xlsx": ("workbook", "[0]!{}", ALL),
    "series-sheet-name.xlsx": ("sheet", "Sheet1!{}", ALL),
    "series-bare-name.xlsx": ("workbook", "{}", ALL),
    "series-prefixed-workbook-name.xlsx": ("workbook", "Sheet1!{}", ALL),
}


def workbook(scope: str) -> str:
    local = ' localSheetId="0"' if scope == "sheet" else ""
    names = "".join(f'<definedName name="{name}"{local}>{cells}</definedName>' for name, cells in NAMES)
    return (
        f'{HEAD}<workbook xmlns="{MAIN}" xmlns:r="{OFFICE}">'
        '<sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets>'
        f'<definedNames>{names}</definedNames><calcPr calcId="191029" fullCalcOnLoad="1"/></workbook>'
    )


def sheet() -> str:
    def text(cell: str, value: str) -> str:
        return f'<c r="{cell}" t="inlineStr"><is><t>{value}</t></is></c>'

    rows = [f'<row r="1">{text("A1", "Month")}{text("B1", "Value")}{text("C1", "Other")}</row>']
    for r, (category, value, other) in enumerate(zip(CATEGORIES, VALUES, OTHER), start=2):
        rows.append(
            f'<row r="{r}">{text(f"A{r}", category)}<c r="B{r}"><v>{value}</v></c><c r="C{r}"><v>{other}</v></c></row>'
        )
    return (
        f'{HEAD}<worksheet xmlns="{MAIN}" xmlns:r="{OFFICE}"><dimension ref="A1:C5"/>'
        f'<sheetData>{"".join(rows)}</sheetData><drawing r:id="rId1"/></worksheet>'
    )


def points(values: tuple, number: bool) -> str:
    code = "<c:formatCode>General</c:formatCode>" if number else ""
    listed = "".join(f'<c:pt idx="{i}"><c:v>{value}</c:v></c:pt>' for i, value in enumerate(values))
    return f'{code}<c:ptCount val="{len(values)}"/>{listed}'


def chart(form: str, named: tuple[str, ...]) -> str:
    """The chart part: a clustered column chart of one series, with a legend so that the series' name shows."""
    label, months, sales = (form.format(name) if name in named else cells for name, cells in NAMES)
    series = (
        '<c:ser><c:idx val="0"/><c:order val="0"/>'
        f"<c:tx><c:strRef><c:f>{label}</c:f><c:strCache>{points((STALE_NAME,), False)}</c:strCache></c:strRef></c:tx>"
        '<c:invertIfNegative val="0"/>'
        f"<c:cat><c:strRef><c:f>{months}</c:f><c:strCache>{points(STALE_CATEGORIES, False)}</c:strCache>"
        "</c:strRef></c:cat>"
        f"<c:val><c:numRef><c:f>{sales}</c:f><c:numCache>{points(STALE_VALUES, True)}</c:numCache></c:numRef></c:val>"
        "</c:ser>"
    )

    def axis(kind: str, own: int, other: int, position: str) -> str:
        return (
            f'<c:{kind}><c:axId val="{own}"/><c:scaling><c:orientation val="minMax"/></c:scaling>'
            f'<c:delete val="0"/><c:axPos val="{position}"/><c:crossAx val="{other}"/></c:{kind}>'
        )

    return (
        f'{HEAD}<c:chartSpace xmlns:c="{CHART}" xmlns:a="{DRAWING}" xmlns:r="{OFFICE}"><c:roundedCorners val="0"/>'
        '<c:chart><c:autoTitleDeleted val="1"/><c:plotArea><c:layout/>'
        '<c:barChart><c:barDir val="col"/><c:grouping val="clustered"/><c:varyColors val="0"/>'
        f'{series}<c:gapWidth val="150"/><c:axId val="1"/><c:axId val="2"/></c:barChart>'
        f'{axis("catAx", 1, 2, "b")}{axis("valAx", 2, 1, "l")}</c:plotArea>'
        '<c:legend><c:legendPos val="r"/><c:overlay val="0"/></c:legend><c:plotVisOnly val="1"/></c:chart>'
        "</c:chartSpace>"
    )


# The drawing part: one chart, anchored at E3 to L18.
DRAWING_PART = (
    f'{HEAD}<xdr:wsDr xmlns:xdr="{SHEET_DRAWING}" xmlns:a="{DRAWING}" xmlns:r="{OFFICE}"><xdr:twoCellAnchor>'
    "<xdr:from><xdr:col>4</xdr:col><xdr:colOff>0</xdr:colOff><xdr:row>2</xdr:row><xdr:rowOff>0</xdr:rowOff></xdr:from>"
    "<xdr:to><xdr:col>11</xdr:col><xdr:colOff>0</xdr:colOff><xdr:row>17</xdr:row><xdr:rowOff>0</xdr:rowOff></xdr:to>"
    '<xdr:graphicFrame macro=""><xdr:nvGraphicFramePr><xdr:cNvPr id="2" name="Chart 1"/>'
    '<xdr:cNvGraphicFramePr/></xdr:nvGraphicFramePr><xdr:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/></xdr:xfrm>'
    f'<a:graphic><a:graphicData uri="{CHART}"><c:chart xmlns:c="{CHART}" r:id="rId1"/></a:graphicData></a:graphic>'
    "</xdr:graphicFrame><xdr:clientData/></xdr:twoCellAnchor></xdr:wsDr>"
)


def main() -> None:
    out = output_directory()
    types = content_types(
        1,
        styles=True,
        more=(
            ("/xl/drawings/drawing1.xml", f"{DRAWING_TYPES}.drawing+xml"),
            ("/xl/charts/chart1.xml", f"{DRAWING_TYPES}.drawingml.chart+xml"),
        ),
    )
    for name, (scope, form, named) in FILES.items():
        write_package(
            out / name,
            (
                ("[Content_Types].xml", types),
                ("_rels/.rels", PACKAGE_RELATIONSHIPS),
                ("xl/workbook.xml", workbook(scope)),
                ("xl/_rels/workbook.xml.rels", workbook_relationships(1, styles=True)),
                ("xl/styles.xml", styles()),
                ("xl/worksheets/sheet1.xml", sheet()),
                ("xl/worksheets/_rels/sheet1.xml.rels", relationships(("drawing", "../drawings/drawing1.xml"))),
                ("xl/drawings/drawing1.xml", DRAWING_PART),
                ("xl/drawings/_rels/drawing1.xml.rels", relationships(("chart", "../charts/chart1.xml"))),
                ("xl/charts/chart1.xml", chart(form, named)),
            ),
        )


if __name__ == "__main__":
    main()
