"""Check input pins and prediction coverage, not Excel parity."""

import datetime as dt
import hashlib
import json
from pathlib import Path

from openpyxl import load_workbook
from openpyxl.utils.datetime import to_excel

root = Path(__file__).resolve().parent
manifest = json.loads((root / "probes/probes.json").read_text())
frozen = json.loads((root / "frozen-predictions.json").read_text())
assert manifest["schema"] == "rustsheet.fill-probes/1"
assert frozen["schema"] == "rustsheet.fill-predictions/1"
assert frozen["isExcelObservation"] is False
assert len(frozen["engineCommit"]) == 40
assert len(manifest["probes"]) == 24
predictions = {entry["id"]: entry for entry in frozen["predictions"]}
assert len(predictions) == len(frozen["predictions"]) == len(manifest["probes"])


def positions(rect):
    return {(row, col) for row in range(rect["r0"], rect["r1"] + 1) for col in range(rect["c0"], rect["c1"] + 1)}


planned_cells = 0
errors = 0
for probe in manifest["probes"]:
    path = root / "probes" / probe["file"]
    assert hashlib.sha256(path.read_bytes()).hexdigest() == probe["sha256"]
    workbook = load_workbook(path, data_only=False)
    assert workbook.sheetnames == ["Fill"]
    sheet = workbook["Fill"]
    assert sorted(str(merge) for merge in sheet.merged_cells.ranges) == sorted(probe["merges"])
    for seed in probe["seeds"]:
        cell = sheet[seed["cell"]]
        actual = to_excel(cell.value, workbook.epoch) if isinstance(cell.value, dt.datetime) else cell.value
        assert actual == seed["value"], (probe["id"], seed["cell"], actual, seed["value"])
        if seed["format"]:
            expected = manifest["formats"][seed["format"]]
            assert cell.number_format == expected.get("num_format", "General")
            for property_name in ["bold", "italic"]:
                assert bool(getattr(cell.font, property_name)) == expected.get(property_name, False)
            if "bg_color" in expected:
                assert cell.fill.fgColor.rgb[-6:] == expected["bg_color"][1:]
    source = positions(probe["sourceRect"])
    target = positions(probe["targetRect"])
    assert source < target
    prediction = predictions[probe["id"]]
    assert prediction["sha256"] == probe["sha256"]
    outcome = prediction["outcome"]
    if outcome["status"] == "error":
        assert probe["id"] == "merged-source-partial"
        assert outcome["message"] and outcome["debug"]
        errors += 1
        continue
    assert outcome["status"] == "plan"
    cells = outcome["plan"]["cells"]
    assert len(cells) == len(target - source)
    assert {(cell["row"], cell["col"]) for cell in cells} == target - source
    assert all((cell["from"]["row"], cell["from"]["col"]) in source for cell in cells)
    assert outcome["plan"]["axis"] == ("rows" if probe["direction"] in ["up", "down"] else "cols")
    planned_cells += len(cells)

expected_sums = "".join(f"{entry['sha256']}  {entry['file']}\n" for entry in manifest["probes"])
assert (root / "probes/SHA256SUMS").read_text() == expected_sums
assert {probe["direction"] for probe in manifest["probes"]} == {"up", "down", "left", "right"}
assert {probe["autoFillType"] for probe in manifest["probes"]} == {"FillDefault", "FillCopy", "FillSeries", "FillDays", "FillWeekdays", "FillMonths", "FillYears"}
print(f"24 pinned workbooks; {len(predictions) - errors} plans covering {planned_cells} cells; {errors} engine refusal; no Excel observations")
