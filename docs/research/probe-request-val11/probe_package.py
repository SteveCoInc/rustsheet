"""What the probe requests' workbook generators share (`*/generate.py` beside this file): the output directory
argument, the parts every SpreadsheetML package has, and a zip writer that gives the same bytes at every run and
on every platform, so that a request can name a file by its sha256.

Standard library only.
"""
import hashlib
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path

MAIN = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
PACKAGE = "http://schemas.openxmlformats.org/package/2006"
OFFICE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
SHEET_TYPES = "application/vnd.openxmlformats-officedocument.spreadsheetml"
HEAD = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'

# The application properties part. Every hand-built workbook Excel for the web has opened for this repository
# carries one (eval/suites/fixtures/cases/valueless-cell-forms.xlsx and its like), and none without it has been
# tried, so every probe workbook carries it too. `write_package` adds it.
APP_PART = "docProps/app.xml"
APP = (
    f'{HEAD}<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">'
    "<Application>RustSheet probe</Application></Properties>"
)
# `legacy`, below: the three files of the t5145 request were sent before two rules of this module. They are
# deflated and carry no application properties, and they keep their bytes.


def package_relationships(legacy: bool = False) -> str:
    """`_rels/.rels`: the package's relationships, to the workbook part and to the application properties."""
    app = "" if legacy else f'<Relationship Id="rId2" Type="{OFFICE}/extended-properties" Target="{APP_PART}"/>'
    return (
        f'{HEAD}<Relationships xmlns="{PACKAGE}/relationships">'
        f'<Relationship Id="rId1" Type="{OFFICE}/officeDocument" Target="xl/workbook.xml"/>{app}</Relationships>'
    )


PACKAGE_RELATIONSHIPS = package_relationships()


def content_types(sheets: int, styles: bool = False, legacy: bool = False) -> str:
    """`[Content_Types].xml` for a workbook of `sheets` worksheets, with a styles part or without."""
    parts = "".join(
        f'<Override PartName="/xl/worksheets/sheet{i}.xml" ContentType="{SHEET_TYPES}.worksheet+xml"/>'
        for i in range(1, sheets + 1)
    )
    if styles:
        parts += f'<Override PartName="/xl/styles.xml" ContentType="{SHEET_TYPES}.styles+xml"/>'
    if not legacy:
        kind = "application/vnd.openxmlformats-officedocument.extended-properties+xml"
        parts += f'<Override PartName="/{APP_PART}" ContentType="{kind}"/>'
    return (
        f'{HEAD}<Types xmlns="{PACKAGE}/content-types">'
        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
        '<Default Extension="xml" ContentType="application/xml"/>'
        f'<Override PartName="/xl/workbook.xml" ContentType="{SHEET_TYPES}.sheet.main+xml"/>'
        f"{parts}</Types>"
    )


def workbook_relationships(sheets: int, styles: bool = False) -> str:
    """`xl/_rels/workbook.xml.rels`: rId1 onward are the worksheets in order, and the styles part comes last."""
    parts = "".join(
        f'<Relationship Id="rId{i}" Type="{OFFICE}/worksheet" Target="worksheets/sheet{i}.xml"/>'
        for i in range(1, sheets + 1)
    )
    if styles:
        parts += f'<Relationship Id="rId{sheets + 1}" Type="{OFFICE}/styles" Target="styles.xml"/>'
    return f'{HEAD}<Relationships xmlns="{PACKAGE}/relationships">{parts}</Relationships>'


def styles(cell_formats: tuple[int, ...] = (), number_formats: tuple[tuple[int, str], ...] = ()) -> str:
    """`xl/styles.xml`: cell format 0 is General, and cell format i + 1 has the number format `cell_formats[i]`.

    `number_formats` are the workbook's own (id, escaped and quoted code); ids below 164 are Excel's built-ins.
    This repository's importer refuses a workbook with no styles part (what Excel does with one is UNVERIFIED:
    docs/excel-semantics/xlsx-storage.md, "A package with no styles part is refused here"; M10 of the t5145
    probe asks), so every probe workbook that the engine is to load carries one.
    """
    own = "".join(f'<numFmt numFmtId="{number}" formatCode={code}/>' for number, code in number_formats)
    declared = f'<numFmts count="{len(number_formats)}">{own}</numFmts>' if number_formats else ""
    formats = "".join(
        f'<xf numFmtId="{number}" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/>'
        for number in cell_formats
    )
    return (
        f'{HEAD}<styleSheet xmlns="{MAIN}">{declared}'
        '<fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>'
        '<fills count="2"><fill><patternFill patternType="none"/></fill>'
        '<fill><patternFill patternType="gray125"/></fill></fills>'
        '<borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>'
        '<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>'
        f'<cellXfs count="{len(cell_formats) + 1}">'
        '<xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>'
        f"{formats}</cellXfs>"
        '<cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>'
        "</styleSheet>"
    )


def output_directory() -> Path:
    """The one argument of a generator, created if it is missing."""
    if len(sys.argv) != 2:
        sys.exit(f"usage: python3 -I {Path(sys.argv[0]).name} <output directory>")
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    return out


def write_package(path: Path, parts: tuple[tuple[str, str], ...], legacy: bool = False) -> None:
    """Writes the parts, (name, text) in order, as a package, and prints its sha256 and name.

    The application properties part is added after them, unless the parts hold one. The parts are stored, not
    deflated: zlib and zlib-ng deflate the same part to different bytes, so a deflated package's sha256 would
    depend on the Python that wrote it. `legacy` writes the t5145 probe's files as they were sent.
    """
    method = zipfile.ZIP_DEFLATED if legacy else zipfile.ZIP_STORED
    if not legacy and all(name != APP_PART for name, _ in parts):
        parts = (*parts, (APP_PART, APP))
    with zipfile.ZipFile(path, "w", method) as z:
        for name, text in parts:
            # A fixed timestamp, so that two runs write the same bytes and the request can name a sha256.
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = method
            info.create_system = 3  # zipfile writes 0 on Windows: keep the bytes the same on every platform
            z.writestr(info, text)
    print(hashlib.sha256(path.read_bytes()).hexdigest(), path.name)


def fail(message: str) -> None:
    """Ends a probe's check.py with what it found."""
    sys.exit(f"check.py: {message}")


def generate_into(directory: Path, out: str) -> None:
    """Runs the `generate.py` of a probe directory into `out`, as the request tells its reader to run it."""
    command = [sys.executable, "-I", str(directory / "generate.py"), out]
    run = subprocess.run(command, capture_output=True, text=True, check=False)
    if run.returncode != 0:
        fail(f"generate.py failed: {run.stderr}")


def generated(directory: Path) -> dict[str, tuple[str, dict[str, bytes]]]:
    """What the `generate.py` of a probe directory writes: for each workbook, its sha256 and its parts.

    The generator runs into a scratch directory.
    """
    with tempfile.TemporaryDirectory() as out:
        generate_into(directory, out)
        packages = {}
        for path in sorted(Path(out).iterdir()):
            with zipfile.ZipFile(path) as z:
                parts = {name: z.read(name) for name in z.namelist()}
            packages[path.name] = (hashlib.sha256(path.read_bytes()).hexdigest(), parts)
    return packages
