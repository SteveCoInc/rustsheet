# T-2893: the model's 1,620 ix L140 cells against the engine's 1,383

`trace.py` loads r2-bratio's ix set (`score_lib.load`, w5-triangulation `ix_evidence.tsv` rows with confidence
direct, derived or closed: 9,757 rows), runs it through the model (`bratio_excel.kernel`) and the engine's K(a, b, x, y),
and compares the L140 rows. Output: `trace.out`.

## Finding: 1,620 is all three L140 branches, 1,383 is one of them
The model's "1,011 / 1,542 / 1,585 of 1,620 ix L140 cells" (r2-bratio RESULTS.md section 1, statistics.md "Incomplete
beta L140 and bfrac") pools every ix row BEST sends to bratio's L140 (x0 <= 0.7, b0 split into n bup terms):

| BEST branch | n | BEST ex/u4/w15 |
|---|---|---|
| L140 bup+bpser | 1,383 | 905/1,318/1,354 |
| L140 bup+bgrat | 187 | 82/178/182 |
| L140 bup+bup+bgrat | 50 | 24/46/49 |
| all L140 | 1,620 | 1,011/1,542/1,585 |

T-2506 scored the engine on L140 bup+bpser only (1,383 rows), so its criterion 2 compared that branch with the
model's pooled count. There is no row-level difference to explain:
- The model sends the same 1,383 rows to L140 bup+bpser under BEST, BEST with `closed=False`, BEST with `x29=0.29`,
  TOMS_XL and R_LIKE (`trace.out`; `closed=False` moves no row in or out). verify-r2-bratio/x29check.out also
  counts 1,383.
- The engine's `bratio::dispatch` puts each of the 1,620 rows on the same L140 branch as the model (`trace.out`,
  "engine's branch for them"). The rebuilt ix set is r2-bratio's: 9,757 rows, the same as
  verify-r2-bratio/analyze.out.

## The engine on the model's 1,620 cells (criterion 2)
| model branch | n | model ex/u4/w15 | engine ex/u4/w15 |
|---|---|---|---|
| L140 bup+bpser | 1,383 | 905/1,318/1,354 | 1,133/1,343/1,363 |
| L140 bup+bgrat | 187 | 82/178/182 | 66/163/179 |
| L140 bup+bup+bgrat | 50 | 24/46/49 | 15/48/50 |
| all L140 | 1,620 | 1,011/1,542/1,585 | **1,214/1,554/1,592** |

T-2506's criterion 2 ("the ix L140 cells score at least 1,011 bit-exact of 1,620") is met on the model's own
population: 1,214. The engine is ahead on L140 bup+bpser, the branch T-2506 ported, and ahead in total. On the two
bup+bgrat branches it is bit-exact on 25 fewer cells (81 against 106), and 8 of the 237 cells are off by more than
15 digits, against the model's 6. Those two branches are owned by the bup+bgrat work: T-3568's n-term composition has
landed since T-2977 scored them at 33 and 8 bit-exact (`docs/research/t2977/score_branches.out`). The gap is too
small for a kernel follow-up of its own.

## How the engine was run
T-2977 deleted the research harness. For this run only, `functions/statistical/t2464_harness.rs` was restored from
`f6e944993^`, uncommitted, with its `BRATIO` op computing (w, w1) through today's `bratio::kernel_tails` and the
ported flag through `bratio::closed_or_ported_tails`. The `#[cfg(test)] mod t2464_harness;` line went back into
`statistical/mod.rs`, then `cargo test -p rustsheet_engine --lib --no-run`. Both changes were reverted afterwards.
The data were rebuilt as docs/research/t2506/RESULTS.md "Data" says (python3 with mpmath and numpy).
