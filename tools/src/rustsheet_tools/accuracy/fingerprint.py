"""What a math sidecar's values were computed from, so `math --check` can tell stale values (T-4231).

An eval case's math sidecar (`eval_math`) records two fingerprints in its `source`, and `math --check`
recomputes both from the current code without mpmath:
- `definitions`: for each function whose definition one of the case's candidates calls, a hash of the
  code that computes it. That is the module of the definition's `value`, `domain` and `tail`, and every
  `rustsheet_tools` module those modules import, transitively, but the generator's machinery
  (`MACHINERY`). Each module is hashed as its parsed syntax tree with docstrings removed, so a comment or
  docstring edit changes nothing and any code edit does. The tree is dumped so that the same code gives
  the same hash on Python 3.11 to 3.13 (`_stable_dump`); a Python whose parser reads the same source
  into another tree would flag every sidecar, and the fix is to regenerate them on it. A shared helper (`defs/_common.py`, a fit-lab
  kernel such as `fitlab.kernels.beta`) is covered through its importers.
- `arguments_sha256`: a hash of every computable candidate's cell, formula and arguments as the
  definitions receive them (exact `repr`s, so a reader change such as a subnormal flush shows), and of
  each composite's whole parsed tree.

The machinery (`MACHINERY`, and `composite`'s and `eval_math`'s own evaluation) is not hashed, since
nearly every definition imports it: an edit there that alters values bumps `tables.VERSION`, which the
check compares too. A definition change names the functions to regenerate (`math --suite S --function
F`, T-4483), not the whole suite.
"""

from __future__ import annotations

import ast
import functools
import hashlib
import importlib
import importlib.util
import inspect
from collections.abc import Callable, Iterable, Iterator
from typing import Any

from rustsheet_tools.accuracy import composite
from rustsheet_tools.accuracy.model import Definition

PACKAGE = "rustsheet_tools"
# Modules (nearly) every definition imports that compute no definition's value: an edit to them that
# alters values bumps tables.VERSION instead (see the module doc). The fit lab's sweep reader is the
# argument reader, which `arguments_sha256` covers through what it reads; its scoring modules and the
# snapshot reader come in only through a kernel's imports of their types (`fitlab/kernels/beta.py`).
MACHINERY = frozenset(
    {
        "rustsheet_tools",
        "rustsheet_tools.fitlab.fit",
        "rustsheet_tools.fitlab.holdout",
        "rustsheet_tools.fitlab.score",
        "rustsheet_tools.fitlab.sweep",
        "rustsheet_tools.snapshot",
        "rustsheet_tools.accuracy",
        "rustsheet_tools.accuracy.defs",
        "rustsheet_tools.accuracy.literal",
        "rustsheet_tools.accuracy.model",
        "rustsheet_tools.accuracy.sampler",
        "rustsheet_tools.accuracy.tables",
    }
)
# The hex digits kept of each hash: enough to make an accidental match negligible, short enough to read.
DIGITS = 16


def _statements(body: list[ast.stmt]) -> Iterator[ast.stmt]:
    """Every statement of `body` and of the blocks nested in it (function and class bodies, `if`,
    `try`, `with`, loop and `match` blocks). Docstrings and imports are statements, so this walks far
    fewer nodes than `ast.walk`, which also visits every expression."""
    for stmt in body:
        yield stmt
        for name in ("body", "orelse", "finalbody"):
            block = getattr(stmt, name, None)
            if isinstance(block, list):
                yield from _statements(block)
        for handler in getattr(stmt, "handlers", None) or []:
            yield from _statements(handler.body)
        for case in getattr(stmt, "cases", None) or []:
            yield from _statements(case.body)


def _strip_docstrings(tree: ast.Module) -> ast.Module:
    """`tree` without the docstring of the module, of each class and of each function (in place)."""
    holders = [
        tree,
        *(
            s
            for s in _statements(tree.body)
            if isinstance(s, ast.ClassDef | ast.FunctionDef | ast.AsyncFunctionDef)
        ),
    ]
    for node in holders:
        body = node.body
        first = body[0] if body else None
        if (
            isinstance(first, ast.Expr)
            and isinstance(first.value, ast.Constant)
            and isinstance(first.value.value, str)
        ):
            node.body = body[1:] or [ast.Pass()]
    return tree


def _stable_dump(node: Any, out: list[str]) -> None:
    """Appends a dump of `node` that is the same on every Python the tools run on: like `ast.dump`
    without positions, but an empty field (None or []) is left out, so a field a newer Python adds
    empty (3.12's `type_params` on every function and class) does not change it. CI's Python (the
    runner's) and a local one can differ by a minor version."""
    if isinstance(node, ast.AST):
        out.append(type(node).__name__ + "(")
        for name in node._fields:
            value = getattr(node, name, None)
            if value is None or value == []:
                continue
            out.append(name + "=")
            _stable_dump(value, out)
            out.append(",")
        out.append(")")
    elif isinstance(node, list):
        out.append("[")
        for item in node:
            _stable_dump(item, out)
            out.append(",")
        out.append("]")
    else:
        out.append(repr(node))


def _tree_digest(tree: ast.AST) -> str:
    """The hash of a parsed module's `_stable_dump`."""
    out: list[str] = []
    _stable_dump(tree, out)
    return hashlib.sha256("".join(out).encode("utf-8")).hexdigest()


def source_digest(source: str) -> str:
    """The hash of Python `source`'s code: its syntax tree, docstrings removed (the dump leaves out
    positions and comments, so only a code edit changes it)."""
    return _tree_digest(_strip_docstrings(ast.parse(source)))


@functools.cache
def _module_tree(name: str) -> ast.Module:
    """Module `name`'s syntax tree without docstrings, parsed once for its digest and its imports
    (`math --check` parses every definition module in each worker: the parse is most of the cost)."""
    return _strip_docstrings(ast.parse(inspect.getsource(importlib.import_module(name))))


@functools.cache
def _module_digest(name: str) -> str:
    """`source_digest` of module `name`, from its one parse."""
    return _tree_digest(_module_tree(name))


def clear_caches() -> None:
    """Forgets every cached parse and digest (a test that edits a module's source needs it)."""
    for cached in (_module_tree, _module_digest, _imports, _modules_digest):
        cached.cache_clear()


def _ours(name: str) -> bool:
    """Whether `name` is a module of this package."""
    return name == PACKAGE or name.startswith(PACKAGE + ".")


def _submodule(package: str, name: str) -> str | None:
    """`package.name` when `package` is a package with that submodule (`from pkg import mod`), else None."""
    if not hasattr(importlib.import_module(package), "__path__"):
        return None
    sub = f"{package}.{name}"
    return sub if importlib.util.find_spec(sub) is not None else None


@functools.cache
def _imports(name: str) -> tuple[str, ...]:
    """The `rustsheet_tools` modules that module `name` imports directly, less `MACHINERY`."""
    found: set[str] = set()
    for node in _statements(_module_tree(name).body):
        if isinstance(node, ast.ImportFrom) and node.module and node.level == 0 and _ours(node.module):
            found.add(node.module)
            found.update(sub for a in node.names if (sub := _submodule(node.module, a.name)))
        elif isinstance(node, ast.Import):
            found.update(a.name for a in node.names if _ours(a.name))
    return tuple(sorted(found - MACHINERY - {name}))


def _closure(roots: Iterable[str]) -> list[str]:
    """`roots` and every module they import (`_imports`), transitively, sorted."""
    seen: set[str] = set()
    todo = list(roots)
    while todo:
        name = todo.pop()
        if name not in seen:
            seen.add(name)
            todo.extend(_imports(name))
    return sorted(seen)


def _code_modules(definition: Definition) -> list[str]:
    """The modules of the definition's callables (a builtin or C function has no module to hash)."""
    parts: list[Callable[..., Any] | None] = [definition.value, definition.domain, definition.tail]
    return [m for p in parts if p is not None and (m := getattr(p, "__module__", None))]


def definition_digest(definition: Definition) -> str:
    """The fingerprint of the code that computes `definition` (see the module doc)."""
    return _modules_digest(tuple(_code_modules(definition)))


@functools.cache
def _modules_digest(roots: tuple[str, ...]) -> str:
    """The fingerprint of `roots` and their imports, cached by module names: hashing a `Definition`
    itself would walk all its fields on every lookup."""
    h = hashlib.sha256()
    for name in _closure(roots):
        h.update(f"{name}\0{_module_digest(name)}\n".encode())
    return h.hexdigest()[:DIGITS]


def _node_shape(arg: Any) -> Any:
    """A composite node by its kind and parts, the called definition by its function name (a node's
    own `repr` would print the definition's callables with their addresses); a literal argument
    (`literal.Arg`) as it is, since its `repr` is exact (a double's `repr` round-trips, `-0.0`
    included, and an int, a bool and a text print differently)."""
    if isinstance(arg, composite.Const):
        return ("const", arg.value)
    if isinstance(arg, composite.Neg):
        return ("neg", _node_shape(arg.operand))
    if isinstance(arg, composite.Op):
        return ("op", arg.op, _node_shape(arg.left), _node_shape(arg.right))
    if isinstance(arg, composite.Call):
        return ("call", arg.definition.function, *map(_node_shape, arg.args))
    return arg


def _calls(node: Any) -> Iterable[Definition]:
    """The definitions a composite tree calls."""
    if isinstance(node, composite.Call):
        yield node.definition
        for a in node.args:
            yield from _calls(a)
    elif isinstance(node, composite.Neg):
        yield from _calls(node.operand)
    elif isinstance(node, composite.Op):
        yield from _calls(node.left)
        yield from _calls(node.right)


def fingerprints(cands: Iterable[Any]) -> dict[str, Any]:
    """`definitions` and `arguments_sha256` for a case's candidates (`eval_math.Candidate`): only the
    ones whose arguments fit, since the others get no value and no `skipped` entry."""
    definitions: dict[str, str] = {}
    lines: list[str] = []
    for c in cands:
        if isinstance(c.args, str):
            continue
        if c.tree is not None:
            called = list(_calls(c.tree))
            shape: Any = ("tree", _node_shape(c.tree))
        else:
            called = [c.definition]
            shape = ("args", c.args, c.scale)
        for d in called:
            if d.function not in definitions:  # a `Definition`'s own hash walks all its fields
                definitions[d.function] = definition_digest(d)
        # `repr` of these shapes is exact and the same in every run (`_node_shape`)
        lines.append(repr((c.sheet, c.a1, c.text, shape)))
    digest = hashlib.sha256("\n".join(lines).encode("utf-8")).hexdigest()
    return {"definitions": dict(sorted(definitions.items())), "arguments_sha256": digest}
