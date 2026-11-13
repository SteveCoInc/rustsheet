"""Record the chart-save probe through Microsoft Graph (README "The steps", Graph route).

For each input: upload it unchanged, open a persisting session, write the number 1 into AZ200 of the sheet the
README names, close the session, wait for Excel for the web to save (the item's eTag changes), and download the
saved package as `<input stem>.web.xlsx`. A refused step is recorded as an answer in `<out>/refusals.json`.

Run from tools/:  uv run python ../docs/research/probe-request-val11/chart-save-probe/save_graph.py <inputs> <out>
"""

from __future__ import annotations

import json
import sys
import time
import uuid
from pathlib import Path
from urllib.parse import quote

from rustsheet_tools.oracle.backends import GraphBackend, GraphRequestError, OracleError

SHEET = {"56557.xlsx": "Data", "123233_charts.xlsx": "Page1_1", "64450.xlsx": "Hoja 1"}


def save_one(g: GraphBackend, src: Path) -> bytes:
    drive = g._drive()
    item = g._call(
        "PUT", f"{drive}/root:/{g.folder}/probe-{uuid.uuid4().hex[:12]}.xlsx:/content", data=src.read_bytes()
    ).json()
    url = f"{drive}/items/{item['id']}"
    try:
        sid = {
            "workbook-session-id": g._call(
                "POST", f"{url}/workbook/createSession", json={"persistChanges": True}
            ).json()["id"]
        }
        sheet = quote(SHEET.get(src.name, "Sheet1").replace("'", "''"))
        g._call(
            "PATCH",
            f"{url}/workbook/worksheets('{sheet}')/range(address='AZ200')",
            headers=sid,
            json={"values": [[1]]},
        )
        g._call("POST", f"{url}/workbook/closeSession", headers=sid)
        for _ in range(45):
            if g._call("GET", url).json().get("eTag") != item.get("eTag"):
                break
            time.sleep(2)
        else:
            raise OracleError("Excel never saved the workbook (eTag unchanged after 90 s)")
        return bytes(g._call("GET", f"{url}/content").content)
    finally:
        g._call("DELETE", url)


def main() -> None:
    inputs, out = Path(sys.argv[1]), Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    g = GraphBackend()
    refusals: dict[str, str] = {}
    for src in sorted(inputs.glob("*.xlsx")):
        try:
            (out / f"{src.stem}.web.xlsx").write_bytes(save_one(g, src))
            print("saved", src.name, flush=True)
        except (GraphRequestError, OracleError) as e:
            refusals[src.name] = str(e)
            print("REFUSED", src.name, str(e)[:300], flush=True)
    (out / "refusals.json").write_text(json.dumps(refusals, indent=2) + "\n")


if __name__ == "__main__":
    main()
