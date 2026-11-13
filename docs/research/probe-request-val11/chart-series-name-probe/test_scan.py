"""Checks that scan.py finds a chart's formulas as the package holds them, and says what it could not read:
python3 -I test_scan.py

It takes two of generate.py's workbooks, one whose series are names (`[0]!Label`, `[0]!Months`, `[0]!Sales`) and
the control, whose series are cells, and writes the first again in forms that mean the same: the chart
namespace under another prefix and as the default namespace, the chart part under another name with its
relationship and content type following, and a relationship with an absolute target. Then it breaks the first:
the chart part missing, not XML, its relationships not XML, and a file that is no package, each before the
untouched control. A scan that misses the names, or stops, fails here.
"""

import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
CHART, RELS = "xl/charts/chart1.xml", "xl/drawings/_rels/drawing1.xml.rels"
NAMES = ["[0]!Label", "[0]!Months", "[0]!Sales"]


def generated(directory: Path) -> tuple[Path, Path]:
    """generate.py's workbook whose series are names, and its control."""
    made = directory / "made"
    subprocess.run(
        [sys.executable, "-I", str(HERE / "generate.py"), str(made)], capture_output=True, check=True
    )
    return made / "series-workbook-name.xlsx", made / "series-control.xlsx"


def rewrite(source: Path, target: Path, change) -> None:
    """Writes `source` to `target` with `change(name, bytes)` giving each part's new (name, bytes), or None."""
    with zipfile.ZipFile(source) as old, zipfile.ZipFile(target, "w") as new:
        for name in old.namelist():
            kept = change(name, old.read(name))
            if kept is not None:
                new.writestr(*kept)


def scan(directory: Path) -> str:
    """What scan.py prints for the workbooks in `directory`: it must end normally whatever they hold."""
    command = [sys.executable, "-I", str(HERE / "scan.py"), str(directory)]
    return subprocess.run(command, capture_output=True, text=True, check=True).stdout


def prefixed(name: str, data: bytes):
    if name == CHART:
        data = (
            data.replace(b"<c:", b"<chart:")
            .replace(b"</c:", b"</chart:")
            .replace(b"xmlns:c=", b"xmlns:chart=")
        )
    return name, data


def default_namespace(name: str, data: bytes):
    if name == CHART:
        data = data.replace(b"<c:", b"<").replace(b"</c:", b"</").replace(b"xmlns:c=", b"xmlns=")
    return name, data


def renamed(name: str, data: bytes):
    data = data.replace(b"charts/chart1.xml", b"charts/series-probe.xml")
    return name.replace("charts/chart1.xml", "charts/series-probe.xml"), data


def absolute_target(name: str, data: bytes):
    if name == RELS:
        assert b'Target="../charts/chart1.xml"' in data
        data = data.replace(b'Target="../charts/chart1.xml"', b'Target="/xl/charts/chart1.xml"')
    return name, data


class Scan(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.named, self.control = generated(Path(self.scratch.name))

    def one(self, change, with_control: bool = False) -> str:
        """The scan of one rewritten copy of the named workbook, alone or before the untouched control."""
        with tempfile.TemporaryDirectory() as directory:
            if callable(change):
                rewrite(self.named, Path(directory) / "a.xlsx", change)
            else:
                (Path(directory) / "a.xlsx").write_bytes(change)
            if with_control:
                rewrite(self.control, Path(directory) / "b.xlsx", lambda name, data: (name, data))
            return scan(Path(directory))

    def test_the_names_are_found_however_the_package_holds_them(self) -> None:
        forms = {
            "as generated": lambda name, data: (name, data),
            "another prefix": prefixed,
            "the default namespace": default_namespace,
            "another part name": renamed,
            "an absolute target": absolute_target,
        }
        for form, change in forms.items():
            with self.subTest(form=form):
                output = self.one(change)
                self.assertIn("workbooks read: 1; not readable: 0; chart parts: 1", output)
                self.assertIn("series formulas that are not cells: 3", output)
                self.assertEqual([line.split(": ", 1)[1] for line in output.splitlines()[2:5]], NAMES)
                self.assertNotIn("INCOMPLETE", output)

    def test_the_control_has_a_chart_and_no_name(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            rewrite(self.control, Path(directory) / "b.xlsx", lambda name, data: (name, data))
            output = scan(Path(directory))
        self.assertIn("workbooks read: 1; not readable: 0; chart parts: 1", output)
        self.assertIn("series formulas that are not cells: 0", output)
        self.assertNotIn("INCOMPLETE", output)

    def test_what_is_linked_and_cannot_be_read_is_said(self) -> None:
        broken = {
            "the chart part missing": lambda name, data: None if name == CHART else (name, data),
            "the chart part not XML": lambda name, data: (name, b"<unclosed>" if name == CHART else data),
            "its relationships not XML": lambda name, data: (name, b"<unclosed>" if name == RELS else data),
        }
        for how, change in broken.items():
            with self.subTest(broken=how):
                output = self.one(change, with_control=True)
                self.assertIn("workbooks read: 2; not readable: 0; chart parts: 1", output)
                self.assertIn("series formulas that are not cells: 0", output)
                self.assertIn("INCOMPLETE: 1 part(s) linked and not read; 0 file(s) not readable", output)

    def test_a_file_that_is_no_package(self) -> None:
        for content in (b"not a zip file", b"PK\x03\x04 cut short"):
            with self.subTest(content=content):
                output = self.one(content, with_control=True)
                self.assertIn("workbooks read: 1; not readable: 1; chart parts: 1", output)
                self.assertIn("INCOMPLETE: 0 part(s) linked and not read; 1 file(s) not readable", output)


if __name__ == "__main__":
    unittest.main()
