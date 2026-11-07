#!/usr/bin/env python3
"""Lint-debt ratchet: the lint debt inherited from the IronCalc fork may only shrink.

Usage: scripts/check_lint_debt.py [--update]

Debt lives in two places, both locked in quality/lint-debt.json (decision 0335):

- per-file `LINT-DEBT` blocks in Rust sources: `#![expect(...)]` inner attributes (optionally
  `#![cfg_attr(test, expect(...))]`) between a `// LINT-DEBT-BEGIN` and a `// LINT-DEBT-END`
  line. They scope each exemption to the one module that still violates it, so new files get the
  full workspace lint set, and `expect` makes the build fail once the violation is gone (the
  entry must then be deleted);
- crate-wide `# LINT-DEBT-BEGIN` blocks in a Cargo.toml `[lints.*]` table, the pre-0335 form.
  None is left, and since none is in the lock, a new one fails.

A file or manifest not in the lock, or an entry it does not list, fails the check. Removed
entries are reported; run with --update to lock the smaller set in (commit the JSON).
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LOCK = ROOT / "quality" / "lint-debt.json"
TOML_BLOCK = re.compile(r"# LINT-DEBT-BEGIN.*?\n(.*?)# LINT-DEBT-END", re.S)
RS_BLOCK = re.compile(r"^// LINT-DEBT-BEGIN[^\n]*\n(.*?)^// LINT-DEBT-END", re.S | re.M)
# A lint path inside `expect(...)`: `clippy::name` or a bare rustc lint name.
LINT = re.compile(r"\b(?:(clippy)::)?([a-z][a-z0-9_]*)\b")
SKIP = {"target", ".git", ".venv", "node_modules"}


def _toml_entries(text: str) -> list[str]:
    entries: list[str] = []
    for section, block in zip(re.findall(r"\[lints\.(\w+)\]", text), TOML_BLOCK.findall(text), strict=False):
        for line in block.splitlines():
            key = line.split("=")[0].strip()
            if key and not key.startswith("#"):
                entries.append(f"{section}::{key}")
    return entries


def rs_entries(text: str) -> list[str]:
    """The lints a Rust source's LINT-DEBT block expects, as `clippy::x` / `rust::x`."""
    entries: set[str] = set()
    for block in RS_BLOCK.findall(text):
        for inner in re.findall(r"expect\(([^)]*)\)", block):
            for tool, name in LINT.findall(inner):
                entries.add(f"{tool or 'rust'}::{name}")
    return sorted(entries)


def _walk(pattern: str) -> list[Path]:
    return sorted(p for p in ROOT.glob(pattern) if not SKIP.intersection(p.relative_to(ROOT).parts))


def current() -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for manifest in _walk("**/Cargo.toml"):
        entries = _toml_entries(manifest.read_text())
        if entries:
            out[str(manifest.relative_to(ROOT))] = sorted(entries)
    for source in _walk("**/*.rs"):
        text = source.read_text(encoding="utf-8")
        if "LINT-DEBT-BEGIN" in text:
            out[str(source.relative_to(ROOT))] = rs_entries(text)
    return out


def main() -> int:
    now = current()
    if "--update" in sys.argv:
        LOCK.write_text(json.dumps(dict(sorted(now.items())), indent=2) + "\n")
        print(f"updated {LOCK.relative_to(ROOT)}")
        return 0
    locked: dict[str, list[str]] = json.loads(LOCK.read_text()) if LOCK.exists() else {}
    failed = False
    for path, entries in now.items():
        if path not in locked:
            failed = True
            print(f"FAIL {path}: a new LINT-DEBT block is forbidden (fix the lints instead)")
            continue
        added = sorted(set(entries) - set(locked[path]))
        if added:
            failed = True
            print(f"FAIL {path}: new lint-debt entries are forbidden: {', '.join(added)}")
    for path, entries in locked.items():
        removed = sorted(set(entries) - set(now.get(path, [])))
        if removed:
            print(f"note {path}: paid down {', '.join(removed)} — run with --update and commit")
    total = sum(len(v) for v in now.values())
    print(f"lint debt: {total} entries across {len(now)} files")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
