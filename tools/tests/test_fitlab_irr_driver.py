"""The `irr-type` fit (T-4834): the event tracer, the type score, and the path-invariant transforms."""

from __future__ import annotations

import math

import pytest

from rustsheet_tools.fitlab.irr_transforms import negate, pad_leading, pad_trailing, scale_pow2
from rustsheet_tools.fitlab.kernels import registry
from rustsheet_tools.fitlab.kernels.irr_driver import (
    BANK,
    CANDIDATES,
    CAP,
    CONVERGED,
    Options,
    irr,
    irr_candidate,
    secant,
)
from rustsheet_tools.fitlab.score import score
from rustsheet_tools.fitlab.sweep import Cell

FLOWS = [-1000.0, 300.0, 400.0, 500.0, 200.0]


def test_irr_type_is_registered_and_scores_errors() -> None:
    fit = registry()["irr-type"]
    assert fit.errors
    assert len(fit.candidates()) == len(CANDIDATES) == 8 + len(BANK)


def test_a_conventional_run_converges_and_reports_its_step() -> None:
    out = irr(FLOWS, 0.1, Options())
    assert isinstance(out.value, float)
    assert abs(out.value - 0.15322) < 1e-5
    assert out.runs[0].event == CONVERGED
    assert out.event == f"{CONVERGED}@{out.runs[0].step}"
    assert out.trace[-1] == out.value


def test_no_sign_change_is_num() -> None:
    assert irr([1.0, 2.0], 0.1, Options()).value == "#NUM!"


def test_a_tiny_cap_ends_on_the_cap_and_retries_nothing_from_0_1() -> None:
    out = irr(FLOWS, 0.1, Options(cap=1))
    assert out.value == "#NUM!"
    assert [r.event for r in out.runs] == [CAP]


def test_score_counts_types_and_lists_events() -> None:
    cells = [
        Cell("s/c", "S", "A1", "=IRR(...)", (FLOWS,), irr(FLOWS, 0.1, Options()).value),  # type: ignore[arg-type]
        Cell("s/c", "S", "A2", "=IRR(...)", ([1.0, 2.0],), math.nan, "#NUM!"),
        Cell("s/c", "S", "A3", "=IRR(...)", ([1.0, 2.0],), math.nan, "#DIV/0!"),
    ]
    s = score(cells, irr_candidate("base", Options(), "base"))
    assert s.typed == 2
    assert s.exact == 2
    assert s.events["s/c S!A1"].startswith(CONVERGED)
    assert s.events["s/c S!A2"] == "no sign change@0"
    assert "events" in s.to_json(3)


@pytest.mark.parametrize("k", [-40, -7, -1, 1, 9, 40])
def test_pow2_scaling_keeps_the_secant_iterates_bit_identical(k: int) -> None:
    # The step is a ratio of NPVs, so x2^k leaves every iterate alone; only the absolute stop tests see the
    # scale, so compare the iterates both runs reach.
    base = secant(FLOWS, 0.1, Options()).iterates
    scaled = secant(scale_pow2(FLOWS, k), 0.1, Options()).iterates
    n = min(len(base), len(scaled))
    assert n >= 3
    assert scaled[:n] == base[:n]


def test_pow2_scaling_refuses_an_inexact_scale() -> None:
    with pytest.raises(ValueError, match="not exact"):
        scale_pow2([1e308, -1.0], 10)


def test_negation_keeps_the_root_and_leading_zeros_are_invisible() -> None:
    root = irr(FLOWS, 0.1, Options()).value
    assert isinstance(root, float)
    flipped = irr(negate(FLOWS), 0.1, Options()).value
    assert isinstance(flipped, float)
    assert abs(flipped - root) < 1e-9
    padded = irr(pad_leading(FLOWS, 3), 0.1, Options())
    assert padded.value == root
    assert padded.trace == irr(FLOWS, 0.1, Options()).trace


def test_trailing_zeros_keep_the_root() -> None:
    root = irr(FLOWS, 0.1, Options()).value
    padded = irr(pad_trailing(FLOWS, 5), 0.1, Options()).value
    assert isinstance(root, float)
    assert isinstance(padded, float)
    assert abs(padded - root) < 1e-9


@pytest.mark.parametrize("name", sorted(BANK))
def test_every_bank_handler_is_inert_on_a_clean_run(name: str) -> None:
    """T-4836: each handler acts only at a flat secant, a pole crossing, the cap or a failed call."""
    assert irr(FLOWS, 0.1, BANK[name]).value == irr(FLOWS, 0.1, Options()).value


def test_the_newton_fallback_returns_where_both_secant_runs_fail() -> None:
    """H12 on `IRR({42,1,1,-1})` (financial.md "IRR's fallback when the secant fails": Excel -0.7467044766537371)."""
    flows = [42.0, 1.0, 1.0, -1.0]
    assert irr(flows, 0.1, Options(cap=200)).value == "#NUM!"
    out = irr(flows, 0.1, BANK["H12 Newton fallback"])
    assert isinstance(out.value, float)
    assert abs(out.value - -0.7467044766537371) < 1e-13
    assert out.runs[-1].event == "fallback"
