"""Build the T-4895 upload workbooks: patched spec2xlsx output that spec2xlsx cannot write itself.

Base: gen.py's table (Cat a,b,a,c,a,b; Val 1-6; Ext 10-60; Cat keeps a and c, rows 3 and 7; Ext rejects 40, row 5)
with the edit delete_column at C, which deletes Ext, so Cat's criterion is the one kept. Variants patch
xl/tables/table1.xml (and for the blank forms xl/worksheets/sheet1.xml):
- base: unpatched (control for the delete);
- upper-case: Cat's filter lists "A" and "C" (the cells hold a and c);
- blank-kept: A6 ("a", visible) is removed and Cat's filter adds blank="1";
- blank-rejected: A6 is removed and Cat's filter is unchanged;
- custom: Cat's criterion is a custom filter, equals a or equals c;
- custom-upper: the same custom filter with operands A and C (T-4965: is a custom equal case-insensitive?).
Run (tools venv): cd tools && uv run python ../docs/research/t4895/uploads.py <out-dir>; then
`uv run rustsheet-oracle recalc -b excel-graph <out-dir>/<variant>.xlsx` for each. Prints each file's sha256.
"""

import hashlib
import io
import sys
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import gen  # noqa: E402 (gen.py sits beside this script; the path is set just above)

from rustsheet_tools.spec2xlsx import build_xlsx  # noqa: E402 (kept after the gen import it pairs with)

CAT = '<filterColumn colId="0"><filters><filter val="a"/><filter val="c"/></filters></filterColumn>'
A6 = '<c r="A6" t="s"><v>3</v></c>'
PATCHES = {
    "base": {},
    "upper-case": {"xl/tables/table1.xml": [(CAT, CAT.replace('"a"', '"A"').replace('"c"', '"C"'))]},
    "blank-kept": {
        "xl/tables/table1.xml": [(CAT, CAT.replace("<filters>", '<filters blank="1">'))],
        "xl/worksheets/sheet1.xml": [(A6, "")],
    },
    "blank-rejected": {"xl/worksheets/sheet1.xml": [(A6, "")]},
    "custom": {
        "xl/tables/table1.xml": [
            (
                CAT,
                '<filterColumn colId="0"><customFilters><customFilter val="a"/><customFilter val="c"/>'
                "</customFilters></filterColumn>",
            )
        ]
    },
    "custom-upper": {
        "xl/tables/table1.xml": [
            (
                CAT,
                '<filterColumn colId="0"><customFilters><customFilter val="A"/><customFilter val="C"/>'
                "</customFilters></filterColumn>",
            )
        ]
    },
}


def patched(raw: bytes, edits: dict[str, list[tuple[str, str]]]) -> bytes:
    src = zipfile.ZipFile(io.BytesIO(raw))
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as dst:
        for item in src.infolist():
            data = src.read(item.filename)
            for old, new in edits.get(item.filename, []):
                text = data.decode()
                if text.count(old) != 1:
                    raise SystemExit(f"{item.filename}: {old!r} found {text.count(old)} times")
                data = text.replace(old, new).encode()
            dst.writestr(item, data)
    return out.getvalue()


def main() -> None:
    out_dir = Path(sys.argv[1])
    out_dir.mkdir(parents=True, exist_ok=True)
    spec = gen.case("T-4895 upload base", [3, 5, 7], [{"op": "delete_column", "sheet": "S", "column": "C"}], {})
    raw = build_xlsx(spec)
    for name, edits in PATCHES.items():
        data = patched(raw, edits)
        (out_dir / f"{name}.xlsx").write_bytes(data)
        print(name, hashlib.sha256(data).hexdigest())


if __name__ == "__main__":
    main()
