"""T-4464: record the engine's values for probes/t4464-groupby-spent-filter-shapes before the fill.

Run from the repo root after `make build` and gen_probe.py, before the case has an oracle sidecar:
python3 -I docs/research/t4464/predict_engine.py. It works as docs/research/t4344/predict_engine.py does (a
placeholder sidecar, `rustsheet-eval show --all`, the placeholder removed again), but on sheet G and over each
call's whole 3x3 spill area (A<r>:C<r+2>), since these calls spill. A cell the engine leaves empty is recorded as
`""`. The values go to docs/research/t4464/predictions.tsv as JSON strings (cell, value).
"""

import json
import os
import re
import subprocess

CASE = "eval/suites/probes/cases/t4464-groupby-spent-filter-shapes"
OUT = "docs/research/t4464/predictions.tsv"
PLACEHOLDER = {"application": "placeholder", "input_sha256": "", "kind": "placeholder", "produced_at": "", "version": None, "workbook_name": ""}


def unescape(debug: str) -> str:
    """A Rust `{:?}` string literal's text."""
    body = debug[1:-1]
    body = re.sub(r"\\u\{([0-9a-f]+)\}", lambda m: chr(int(m.group(1), 16)), body)
    return body.replace('\\"', '"').replace("\\\\", "\\")


def main() -> None:
    sidecar = CASE + ".oracle.json"
    if os.path.exists(sidecar):
        raise SystemExit("the case already has a sidecar: predictions are written before the fill")
    with open(CASE + ".case.json") as fh:
        g = {s["name"]: s["cells"] for s in json.load(fh)["sheets"]}["G"]
    rows = [int(a[1:]) for a, f in g.items() if a.startswith("A") and str(f).startswith("=")]
    area = [f"{c}{r + d}" for r in rows for d in range(3) for c in "ABC"]
    cells = {a: {"t": "s", "v": "\u0001"} for a in area}
    with open(sidecar, "w") as fh:
        json.dump({"format": "rustsheet-oracle/v1", "source": PLACEHOLDER, "sheets": {"G": cells}}, fh)
    try:
        shown = subprocess.run(
            ["target/release/rustsheet-eval", "show", "--all", "probes/" + os.path.basename(CASE) + ".case.json"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    finally:
        os.remove(sidecar)
    values: dict[str, str] = {}
    cell = None
    for line in shown.splitlines():
        m = re.match(r"G!([A-Z]+\d+)\s", line)
        if m:
            cell = m.group(1)
        m = re.match(r"\s+actual\s*:\s*(.*)$", line)
        if m and cell:
            raw = m.group(1).strip()
            values[cell] = unescape(raw) if raw.startswith('"') else raw
    missing = sorted(set(area) - set(values))
    if missing:
        raise SystemExit(f"no engine value for {missing[:5]}")
    with open(OUT, "w") as fh:
        fh.write("# T-4464: the engine's values (the current model) for probes/t4464-groupby-spent-filter-shapes, written "
                 "before the oracle fill (cell, value as a JSON string; each call's A<r>:C<r+2>)\n")
        for c in sorted(values, key=lambda c: (int(c[1:]), c[0])):
            fh.write(f"{c}\t{json.dumps(values[c], ensure_ascii=False)}\n")
    print(len(values), "predictions")


if __name__ == "__main__":
    main()
