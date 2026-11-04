# T-3771 r6-bpser-gpin: bpser's lnΓ(a0) on (0.5, 0.7) and the split of its ceiling

Round 6, 2026-11-03. Questions (T-3771, from T-3570's r5-bpser-lg RESULTS section 3):
1. Which lnΓ(a0) form is bpser's g just above 0.5? r5-bpser-lg found TOMS's fl(gamln1(a0) - LN a0) right more often
   there (19 a0 against 7).
2. Is the per-a0 ceiling of the bpser model in Excel's LN at x, in EXP's argument, or elsewhere?

- Python: `bindings/python/.venv/bin/python` (`make py-bindings`, then `uv pip install mpmath numpy scipy`). Run from
  this directory. `gpin.py` imports r5-bpser-lg's `pin.py`, and through it r2-bpser-small's `lib.py`. Nothing is
  copied.
- Data: r5-bpser-lg's t3570-r5-bpser-lg, 4,032 BETA.DIST cells. That is 504 a0 by single ulps at seven centers (0.15
  to 0.85), three b0 (1.61, 3.37, 5.83) and 8 x each, with Excel's GAMMALN columns. Also t3570-r5-bpser-lg-aux
  (GAMMALN(fl(1 + a0))) and this round's LN probe.
- Hold-out: r5-bpser-lg's md5 split of a0 (`pin.hold`), fixed in that round, before this one.

## Method: judge L only on cells whose z is a double

T-3769 (docs/research/t2464/r6-bpser-xlinked) showed that GL53 with the right L is bit-exact wherever
z = fl(a0 LN x) - L is a double. All its remaining misses sit on non-double z: EXP of a 54-55-bit argument, unsolved,
T-3873. So comparing L forms by bit-exact counts mixes two things: whether L is right, and how often L puts z between
doubles. A wrong L that happens to move z onto the double grid can win on counts. This is what made the gamln1 form
look right near 0.5, and T-3769's a0 = 0.5 g + 1 ulp case.

The clean test: count each form's misses on the cells where its own z is a double. A right L misses none.

## Answers

### 1. lnΓ(a0) is Excel's GAMMALN at every center, including (0.5, 0.7) (`pooled.out`)

| form | double-z cells (tune / hold-out) | misses (tune / hold-out) |
|---|---|---|
| **GL53**: fl(GAMMALN(a0) + d) | 770 / 849 | **1 / 0** |
| GL64: L kept extended | 474 / 542 | 4 / 4 |
| Ag53: g = fl(GAMMALN(fl(1 + a0)) - LN a0) | 742 / 852 | 47 / 60 |
| Ag64: that g kept extended, L stored | 595 / 700 | 50 / 69 |
| AL64: g and L kept extended | 94 / 50 | 7 / 5 |
| g1ln53: fl(fl(gamln1(a0) - LN a0) + d) | 903 / 896 | 92 / 128 |

d = fl(GAMMALN(b0) - GAMMALN(fl(a0 + b0))), all Excel's measured values. On the 0.5-0.7 band alone, GL53 misses 1 of
352 double-z cells on tune and 0 of 252 on the hold-out. gamln1 misses 64 of 437 and 53 of 319. GL53 also has the most
bit-exact cells over all (1,430 tune, 1,434 hold-out; `lforms.out`).

r5-bpser-lg's per-a0 pin (`gpin.out`) cannot see this. At many a0 near 0.5, GL53 puts all 8 z between doubles, so
nothing pins that a0 by bit-exactness. A neighbouring g (often the gamln1 form) can land z on the grid and match
there, because Excel's E for a tied z is often EXP of the neighbouring double (T-3769 `nonrepr.out`). Counted
per a0, that looks like a win for the other g. `dzmiss.out` shows the same confound from the other side: the g with
the most exact double-z cells per a0 still misses 121 double-z cells at centers 0.4-0.7.

### 2. The ceiling is EXP's argument, not LN (`split.out`)

GL53's 1,168 misses on the 4,032 cells:

| piece | tune | hold-out |
|---|---|---|
| LN (Excel's =LN(x) differs from the model) | 0 | 0 |
| EXP's argument (z = alx - L not a double) | 633 | 534 |
| s (z a double, a one-ulp move of s fixes it) | 1 | 0 |
| rest | 0 | 0 |

- LN positive control: the LN model equals Excel's =LN(x) at all 14 x of the probe (t3771-r6-bpser-gpin-ln).
- Excel's E for a non-double z is T-3873's question.
- The one s-side miss is at center 0.5 (tune). It is like T-3769's two s-side misses at (6.25, 0.7).

## Negatives

- The four other L forms above, with their double-z misses.
- A free scan over every double L within ±24 ulps of GL53's (`lforms.out`, column Lscan) finds an L that makes every
  double-z cell exact on 201 of 258 tune and 206 of 246 hold-out a0. Near 0.5 that is only 41 of 72. This scan is
  not a form, and it is confounded the same way as the per-a0 pin: it only shows the grid can be hit.

## Probe

| case | cells | what |
|---|---|---|
| `t3771-r6-bpser-gpin-ln` | 14 | `=LN(x)` at every x of t3570-r5-bpser-lg (excel-graph 2026-11-03, 0 failures) |

`mkprobe.py --write` builds it. Meta is in `t3771-r6-bpser-gpin-ln.meta.json`.

Fitting: there is no fitted value. Six fixed forms are compared on a hold-out fixed in an earlier round.

## Files

- `gpin.py`: the double-z judge, and the per-a0 pin by it → `gpin.out`.
- `dzmiss.py`: the double-z misses at the per-a0 best g → `dzmiss.out`.
- `lforms.py`: per-a0 consistency, bit-exact counts and the L scan per form → `lforms.out`.
- `pooled.py`: the pooled double-z test (answer 1) → `pooled.out`.
- `split.py`: the ceiling split (answer 2) → `split.out`.
- `mkprobe.py`: the LN probe.
