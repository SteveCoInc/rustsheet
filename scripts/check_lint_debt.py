#!/usr/bin/env python3
"""Lint-debt ratchet: the lint debt inherited from the IronCalc fork may only shrink.

Usage: scripts/check_lint_debt.py [--update]

Debt lives in two places, both locked in quality/lint-debt.json (decision 0335):

- per-file `LINT-DEBT` blocks in Rust sources: `#![expect(...)]` inner attributes (optionally
  `#![cfg_attr(test, expect(...))]`) between a `// LINT-DEBT-BEGIN` and a `// LINT-DEBT-END`
  line. They scope each exemption to the one module that still violates it, so new files get the
  full workspace lint set, and `expect` makes the build fail once the violation is gone (the
  entry must then be deleted). A block holds nothing but such attributes, and each entry is
  locked with its scope (`cfg(test) clippy::x` for a test-only one);
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
# `reason = "..."` inside an expect: not lint names.
REASON = re.compile(r'\breason\s*=\s*"(?:[^"\\]|\\.)*"')
SKIP = {"target", ".git", ".venv", "node_modules"}


def _toml_entries(text: str) -> list[str]:
    entries: list[str] = []
    for section, block in zip(re.findall(r"\[lints\.(\w+)\]", text), TOML_BLOCK.findall(text), strict=False):
        for line in block.splitlines():
            key = line.split("=")[0].strip()
            if key and not key.startswith("#"):
                entries.append(f"{section}::{key}")
    return entries


# The attributes a LINT-DEBT block may hold, and the scope each gives its lints in the lock key.
SCOPES = (
    (re.compile(r"^expect\((.*)\)$", re.S), ""),
    (re.compile(r"^cfg_attr\(\s*test\s*,\s*expect\((.*)\)\s*\)$", re.S), "cfg(test) "),
    (re.compile(r"^cfg_attr\(\s*not\(\s*test\s*\)\s*,\s*expect\((.*)\)\s*\)$", re.S), "cfg(not(test)) "),
)


def rs_parse(text: str) -> tuple[list[str], list[str]]:
    """The lints a Rust source's LINT-DEBT block expects, as `clippy::x` / `rust::x` prefixed by
    their scope (`cfg(test) clippy::x` for a test-only one), and the block's other content, which
    is forbidden: an `allow` or any other attribute would escape the ratchet."""
    entries: set[str] = set()
    problems: list[str] = []
    begins = len(re.findall(r"^// LINT-DEBT-BEGIN", text, re.M))
    ends = len(re.findall(r"^// LINT-DEBT-END", text, re.M))
    if begins != ends:
        problems.append(f"{begins} LINT-DEBT-BEGIN line(s) but {ends} LINT-DEBT-END line(s)")
    for block in RS_BLOCK.findall(text):
        body = "\n".join(line for line in block.splitlines() if not line.strip().startswith("//"))
        attrs = re.findall(r"#!\[(.*?)\]\s*(?=#!\[|$)", body.strip(), re.S)
        rest = re.sub(r"#!\[.*?\]\s*(?=#!\[|$)", "", body.strip(), flags=re.S).strip()
        if rest:
            problems.append(f"unexpected text in the block: {rest[:60]!r}")
        for attr in attrs:
            attr = attr.strip()
            for pattern, scope in SCOPES:
                found = pattern.match(attr)
                if found:
                    lints = REASON.sub("", found.group(1))
                    for tool, name in LINT.findall(lints):
                        entries.add(f"{scope}{tool or 'rust'}::{name}")
                    break
            else:
                problems.append(f"only expect(...) attributes may sit in the block, not #![{attr[:60]}]")
    return sorted(entries), problems


def rs_entries(text: str) -> list[str]:
    """The lock entries of a Rust source's LINT-DEBT block (see `rs_parse`)."""
    return rs_parse(text)[0]


def _walk(pattern: str) -> list[Path]:
    return sorted(p for p in ROOT.glob(pattern) if not SKIP.intersection(p.relative_to(ROOT).parts))


def current(problems: list[str] | None = None) -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for manifest in _walk("**/Cargo.toml"):
        entries = _toml_entries(manifest.read_text())
        if entries:
            out[str(manifest.relative_to(ROOT))] = sorted(entries)
    for source in _walk("**/*.rs"):
        text = source.read_text(encoding="utf-8")
        if "LINT-DEBT-BEGIN" in text:
            entries, found = rs_parse(text)
            out[str(source.relative_to(ROOT))] = entries
            if problems is not None:
                problems.extend(f"{source.relative_to(ROOT)}: {p}" for p in found)
    return out


def main() -> int:
    problems: list[str] = []
    now = current(problems)
    for problem in problems:
        print(f"FAIL {problem}")
    if problems:
        return 1
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
