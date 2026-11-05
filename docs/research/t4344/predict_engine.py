"""T-4344: record the engine's values for probes/t4344-groupby-descending-search before the fill.

Run from the repo root after `make build` and gen_probe.py, before the case has an oracle sidecar:
python3 -I docs/research/t4344/predict_engine.py. The engine is the current model (T-4342's tree, the [0, n-1]
search in each field's direction; `tree_directions` builds a table relationship and a value sort ascending), so
its values are that model's committed predictions for every cell, PIVOTBY, table and value sorts included.

`rustsheet-eval show` prints values only for a case with a sidecar, so this writes a placeholder sidecar (every
Calc formula cell the empty text), reads each cell's `actual`, and removes the placeholder again. The values go to
docs/research/t4344/predictions.tsv as JSON strings (cell, value).
"""

import json
import os
import re
import subprocess

CASE = "eval/suites/probes/cases/t4344-groupby-descending-search"
OUT = "docs/research/t4344/predictions.tsv"
# The placeholder sidecar's provenance: none of it is Excel's; the file is removed before the script ends.
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
        calc = {s["name"]: s["cells"] for s in json.load(fh)["sheets"]}["Calc"]
    cells = {c: {"t": "s", "v": ""} for c, f in calc.items() if str(f).startswith("=")}
    with open(sidecar, "w") as fh:
        json.dump({"format": "rustsheet-oracle/v1", "source": PLACEHOLDER, "sheets": {"Calc": cells}}, fh)
    try:
        shown = subprocess.run(
            [
                "target/release/rustsheet-eval",
                "show",
                "--all",
                "probes/" + os.path.basename(CASE) + ".case.json",
            ],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    finally:
        os.remove(sidecar)
    values: dict[str, str] = {}
    cell = None
    for line in shown.splitlines():
        m = re.match(r"Calc!([A-Z]+\d+)\s", line)
        if m:
            cell = m.group(1)
        m = re.match(r"\s+actual\s*:\s*(.*)$", line)
        if m and cell:
            raw = m.group(1).strip()
            values[cell] = unescape(raw) if raw.startswith('"') else raw
    missing = sorted(set(cells) - set(values))
    if missing:
        raise SystemExit(f"no engine value for {missing}")
    with open(OUT, "w") as fh:
        fh.write(
            "# T-4344: the engine's values (the current model) for probes/t4344-groupby-descending-search, written "
            "before the oracle fill (cell, value as a JSON string)\n"
        )
        for c in sorted(values, key=lambda c: (int(c[1:]), c[0])):
            fh.write(f"{c}\t{json.dumps(values[c], ensure_ascii=False)}\n")
    print(len(values), "predictions")


def run(case: str, out: str) -> None:
    """`main` for another probe: its case path (without `.case.json`) and its predictions file."""
    global CASE, OUT
    CASE, OUT = case, out
    main()


if __name__ == "__main__":
    main()
