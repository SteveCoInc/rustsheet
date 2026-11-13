"""Hand-built probe packages: a cell's <v> with whitespace (or other padding) around its value
(poc-cfix-loadvals-cc, 2026-11-12).

Each package is a minimal SpreadsheetML package written here, with Application "RustSheet fixture"
(fixtures suite convention: placeholder cached values are never Excel evidence). One question group per
workbook, so a form Excel refuses takes only its own group down (a refusal is itself the answer).
Usage:
    python3 -I make_probes.py OUTDIR
"""

import sys
from pathlib import Path

import importlib.util

# The package writer is the one of the missing-parts probe (val10), so the two generators share it.
_SPEC = importlib.util.spec_from_file_location(
    "missing_parts_probes",
    Path(__file__).resolve().parents[2] / "probe-request-val10" / "cfix-missing-parts-probe" / "make_probes.py",
)
_BASE = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_BASE)
STYLES, CT_STYLES, sheet_xml = _BASE.STYLES, _BASE.CT_STYLES, _BASE.sheet_xml
MAIN = _BASE.MAIN
CT_SST = "application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"


def raw(ref, v, t=None):
    """A constant cell whose <v> is written exactly as `v` (already XML-escaped)."""
    ta = f' t="{t}"' if t else ""
    return f'<c r="{ref}"{ta}><v>{v}</v></c>'


def formula(ref, f):
    return f'<c r="{ref}"><f>{f}</f><v>0</v></c>'


def package(path, sheet, sst=None):
    rels = [("rId1", "worksheet", "worksheets/sheet1.xml"), ("rId2", "styles", "styles.xml")]
    parts = {"xl/styles.xml": STYLES, "xl/worksheets/sheet1.xml": sheet}
    overrides = [("/xl/styles.xml", CT_STYLES)]
    if sst is not None:
        rels.append(("rId3", "sharedStrings", "sharedStrings.xml"))
        items = "".join(f"<si><t>{s}</t></si>" for s in sst)
        parts["xl/sharedStrings.xml"] = f'<sst xmlns="{MAIN}" count="{len(sst)}" uniqueCount="{len(sst)}">{items}</sst>'
        overrides.append(("/xl/sharedStrings.xml", CT_SST))
    _BASE.package(path, [("Probe", "rId1")], rels, parts, styles=False, overrides=overrides)


def rows(values, readers, t=None):
    """Row n: A the stored form values[n-1], then one reader per column from B."""
    out = []
    for n, v in enumerate(values, start=1):
        cells = [raw(f"A{n}", v, t)]
        for col, reader in zip("BCDEFG", readers):
            cells.append(formula(f"{col}{n}", reader.format(a=f"A{n}")))
        out.append((n, cells))
    return out


NUMBER_READERS = ["ISNUMBER({a})", "{a}+1", "TYPE({a})", "ISBLANK({a})", '{a}&amp;"|"']


def main(out):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    # 1. A number with XML whitespace around it; A4 the control.
    package(out / "value-ws-numbers.xlsx", sheet_xml(rows([" 9", "9 ", "&#9;9&#10;", "9"], NUMBER_READERS)))
    # 2. Other paddings: a no-break space (Unicode, not XML, whitespace), a leading +, a space inside, a
    #    whitespace-only value.
    package(
        out / "value-ws-number-forms.xlsx", sheet_xml(rows(["&#160;9", "+9", "9 0", " "], NUMBER_READERS))
    )
    # 3. t="b": padded 1, and the xsd:boolean spellings ECMA-376 does not give a cell; A4 the control.
    package(
        out / "value-ws-booleans.xlsx",
        sheet_xml(
            rows(
                [" 1 ", "true", "TRUE", "1"], ["ISLOGICAL({a})", "{a}=TRUE", "TYPE({a})", '{a}&amp;"|"'], "b"
            )
        ),
    )
    # 4. t="e": a padded error name; A2 the control.
    package(
        out / "value-ws-errors.xlsx",
        sheet_xml(rows([" #REF! ", "#REF!"], ["ERROR.TYPE({a})", "ISERROR({a})", "TYPE({a})"], "e")),
    )
    # 6. t="d": a padded ISO date; A2 the control. Its own workbook: Excel refuses a workbook whose t="d" text it
    #    cannot read (xlsx-storage.md), so a refusal here takes nothing else down.
    package(
        out / "value-ws-date.xlsx",
        sheet_xml(rows([" 2024-01-01 ", "2024-01-01"], ["ISNUMBER({a})", "{a}+1", "TYPE({a})"], "d")),
    )
    # 7. Whitespace-only values: <v> </v> as a number, a boolean and an error; one workbook per type, so a refusal
    #    names its type.
    for t, name in ((None, "n"), ("b", "b"), ("e", "e")):
        package(
            out / f"value-ws-only-{name}.xlsx",
            sheet_xml(rows([" "], ["ISBLANK({a})", "TYPE({a})", "ISNUMBER({a})", "ISERROR({a})", "{a}&amp;\"|\""], t)),
        )
    # 5. t="s": a padded shared-string index (T-4711's shapes); A5 the control. sst 0 "zero", 1 "one".
    package(
        out / "value-ws-shared-index.xlsx",
        sheet_xml(rows([" 1", "1 ", "+1", "&#160;1", "1"], ['{a}&amp;"|"', "ISTEXT({a})", "LEN({a})"], "s")),
        sst=["zero", "one"],
    )


if __name__ == "__main__":
    main(sys.argv[1])
