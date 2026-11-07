"""Build the T-4939 positive-control uploads: T-4895's number-format table with its kept Ext filter listing the
text the cells display ("10.00" .. "60.00") instead of the General text spec2xlsx writes.

Base: docs/research/t4895/gen.py's t4895-two-criteria-number-format-* spec (Cat a,b,a,c,a,b; Val 1-6; Ext 10-60
formatted 0.00; Cat keeps a and c, rows 3 and 7 hidden; Ext rejects 40, row 5 hidden). Variants patch
xl/tables/table1.xml so Ext's values filter lists 10.00, 20.00, 30.00, 50.00, 60.00:
- display-control: no edit (the file's hidden rows 3, 5, 7);
- display-delete: delete_column A, which deletes Cat, so Ext's criterion is the one kept.
Run (tools venv): cd tools && uv run python ../docs/research/t4939/control.py <out-dir>; then
`uv run rustsheet-oracle recalc -b excel-graph <out-dir>/<variant>.xlsx` for each. Prints each file's sha256.
"""

import hashlib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "t4895"))
import gen  # noqa: E402 (T-4895's generator; the path is set just above)
import uploads  # noqa: E402 (T-4895's patcher, beside gen.py)

from rustsheet_tools.spec2xlsx import build_xlsx  # noqa: E402 (kept after the imports it pairs with)

GENERAL = "".join(f'<filter val="{v}"/>' for v in ("10", "20", "30", "50", "60"))
DISPLAY = "".join(f'<filter val="{v}.00"/>' for v in ("10", "20", "30", "50", "60"))
FORMATS = {f"C{r}": "0.00" for r in range(2, 8)}
VARIANTS = {"display-control": [], "display-delete": gen.DELETE_A}


def main() -> None:
    out_dir = Path(sys.argv[1])
    out_dir.mkdir(parents=True, exist_ok=True)
    for name, ops in VARIANTS.items():
        raw = build_xlsx(gen.case(f"T-4939 {name}", [3, 5, 7], ops, FORMATS))
        data = uploads.patched(raw, {"xl/tables/table1.xml": [(GENERAL, DISPLAY)]})
        (out_dir / f"{name}.xlsx").write_bytes(data)
        print(name, hashlib.sha256(data).hexdigest())


if __name__ == "__main__":
    main()
