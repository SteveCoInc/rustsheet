#!/usr/bin/env python3
"""Checks generate.py without Excel: python3 -I check.py

The three M10 workbooks were sent before the generators stored their packages and gave them the application
properties part (probe_package.py, `legacy`), and a file that was sent keeps its bytes. Checked here, of each:
- its parts are the five it was sent with, in that order, each deflated, and no other;
- the digest of its inflated parts is the one pinned below, which no zlib changes;
- its sha256 is the one README.md names, unless this Python deflates with zlib-ng, which writes the same parts
  as other bytes.
"""
import hashlib
import sys
import tempfile
import zipfile
import zlib
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))  # probe_package.py, beside the directories
from probe_package import fail, generate_into  # noqa: E402

PARTS = [
    "[Content_Types].xml",
    "_rels/.rels",
    "xl/workbook.xml",
    "xl/_rels/workbook.xml.rels",
    "xl/worksheets/sheet1.xml",
]
# sha256 over each part in order: its name, NUL, its inflated bytes, NUL.
DIGESTS = {
    "m10-manual-stale.xlsx": "79a8e1d251ecf00d845633b9abe84dae2e4012d618a84bda1423d357dc01dc0c",
    "m10-auto-stale.xlsx": "3d4c184d16a5ff26791e7ba39bf16715477576ae4cc8c1bc6392a5cef3e8127a",
    "m10-manual-fullcalc.xlsx": "b0be83ca9ced80c71169147ba81795860e456cbe73be276464265b6c00555129",
}


def main() -> None:
    readme = (HERE / "README.md").read_text(encoding="utf-8")
    zlib_ng = "zlib-ng" in zlib.ZLIB_RUNTIME_VERSION or hasattr(zlib, "ZLIBNG_VERSION")
    with tempfile.TemporaryDirectory() as out:
        generate_into(HERE, out)
        written = sorted(path.name for path in Path(out).iterdir())
        if written != sorted(DIGESTS):
            fail(f"generate.py wrote {written}")
        for name, digest in DIGESTS.items():
            path = Path(out, name)
            inflated = hashlib.sha256()
            with zipfile.ZipFile(path) as z:
                infos = z.infolist()
                if [info.filename for info in infos] != PARTS:
                    fail(f"{name} holds {[info.filename for info in infos]}, not the five parts it was sent with")
                if any(info.compress_type != zipfile.ZIP_DEFLATED for info in infos):
                    fail(f"{name} has a part that is not deflated")
                for info in infos:
                    inflated.update(info.filename.encode() + b"\0" + z.read(info) + b"\0")
            if inflated.hexdigest() != digest:
                fail(f"{name}'s parts are not the ones that were sent: digest {inflated.hexdigest()}")
            sha = hashlib.sha256(path.read_bytes()).hexdigest()
            if not zlib_ng and f"| `{name}` | `{sha}` |" not in readme:
                fail(
                    f"README.md does not name {name} with its sha256 {sha}: its parts are the ones sent, so "
                    "the zip around them changed, or this Python does not deflate as zlib does"
                )
    how = "their sha256s not compared (zlib-ng)" if zlib_ng else "and the sha256s README.md names"
    print(f"ok: {len(DIGESTS)} files as they were sent: five deflated parts each, the pinned digests, {how}")


if __name__ == "__main__":
    main()
