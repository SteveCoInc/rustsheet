# SPEC-0018 — Nested arrays: GROUPBY and PIVOTBY cells that hold a LAMBDA's array result

- **Status:** accepted (implemented by T-2593 and T-2594). Revised by T-4661 (2026-11-07) for Excel's change of
  2026-11 (T-2609; arrays.md "Operators over nested cells, and Excel's change of 2026-11"; decision 0329): rule 4 now
  states the 2026-11-06 reads, which are the target. T-4736, T-4737 and T-4738 move the engine to them; until they
  land, ours follows the October reads that rule 4 lists as superseded.
- **Milestone / lane:** M7 / engine (implementation), research (probes)
- **Tasks:** T-2459 (this spec), T-2592 (probes), T-2602 (revision from the probe), T-2593 (representation, INDEX, TYPE,
  top-left reads), T-2594 (MAP, BYROW, BYCOL, REDUCE, the aggregates and reference cells over a nested result)

## Problem
When the function GROUPBY or PIVOTBY applies returns an array (`LAMBDA(v,v)` over a group's values), Excel keeps that array
in the result's cell: the cell shows `#VALUE!`, but `TYPE(INDEX(GROUPBY(..),r,c))` is 64 and INDEX returns the group's
values. Ours makes each such cell the error `#VALUE!` (`Model::aggregate`,
functions/lookup_and_reference/group_functions.rs), because the engine has no value that is an array inside an array:
`CalcResult::Array` holds `ArrayNode` scalars (and LAMBDA ids). Known failures this leaves (eval/baseline, 2026-10-24):

- probes/t2360-array-lambda-holidays: 47 `P!` cells (the INDEX/TYPE grid P!B3..P!E37 and the lifted counts C1, C9, C17,
  C25, C33).
- probes/t2311-pivotby-value-columns-error P!C12 (`SUM(--ISERROR(PIVOTBY(..,LAMBDA(v,v))))` is 0) and P!D12 (MAP over the
  result is an error).
- gen-m7/21b4eef18424ce94 Calc!A1504 and its spill (REGEXEXTRACT whose return_mode is such a GROUPBY gets a valid mode).
- T-2398's `LAMBDA(w,w)` cells: probes/groupby-lifted-args A153 (`SCAN(GROUPBY(..,LAMBDA(w,w)),0,MIN)` is one `#N/A`) and
  gen-m7/b24ce74f08d3f91a A1850.

## Goals / non-goals
Goals: represent a nested array where Excel makes one (GROUPBY and PIVOTBY cells), with Excel's reads of it (INDEX, TYPE,
the element-wise and aggregate reads of rule 4, the shown `#VALUE!`), bounded in memory (CLAUDE.md directive 5).

Non-goals: nested arrays anywhere Excel gives `#CALC!` instead (BYROW, BYCOL, MAP, MAKEARRAY and SCAN whose LAMBDA returns an
array: arrays.md "BYROW and BYCOL: a LAMBDA result that is an array", "MAKEARRAY with an array step" and "MAP and SCAN with a LAMBDA that
returns a multi-cell range"; a MAP step that returns its nested argument is `#CALC!` too, T-2592 A104); a nested array that
survives into a cell's stored value (Excel stores the shown `#VALUE!`: `SUM(ZZ1#)` and `INDEX(ZZ1#,2,2)` over a spilled result
are `#VALUE!`, T-2592 A19, A20).

## Excel behavior to match
Evidence: probes/t2360-array-lambda-holidays P (excel-graph, 2026-10-23, T-2360); probes/t2311-pivotby-value-columns-error
P!C12, P!D12; probes/groupby-lifted-args A153, A161, A169 (excel-graph, 2026-10-22); probes/t2592-nested-groupby-cells
(excel-graph, 2026-10-24, T-2592; cited below as A<n>; `GROUPBY(D!A1:A5,D!C1:D5,LAMBDA(v,v))`, 3x3, groups a `{1;2}`/`{10;20}`,
b `{3;4}`/`{30;40}`, Total 4x1). docs/excel-semantics/arrays.md, the
paragraph starting "A LAMBDA array result is shown as `#VALUE!`, but the cell holds the array itself".

1. **Which cells are nested.** A value cell of GROUPBY or PIVOTBY (group, subtotal and Total cells) whose function
   passes an array through holds that array, whatever its size: a one-record group is a 1x1 array (P!B3 `64|1x1|1`), a two-record
   group 2x1, the Total 4x1 (P!B21 `64|4x1|1`). A computed array result is nested the same way, 1x1 included (`LAMBDA(v,{7})`,
   `LAMBDA(v,SEQUENCE(1))` `64|1x1`, A33, A34), and so is the result of a two-parameter LAMBDA (`LAMBDA(x,y,x)` `64|2x1|3`,
   A22) and so are the leaf and subtotal cells of a two-row-field result with `total_depth` 2 (a one-record leaf `64|1x1`, a two-record subtotal `64|2x1`, the Total `64|4x1`, A35:A41). One LAMBDA over several value columns gives each column its own array
   (P!B3 vs C3). Key, header and empty filler cells are scalars (P!A3 `2|a`, P!F2 `2|Total`, the corner P!A2 `""`). One exception:
   PIVOTBY's Total columns over several value columns stay `""`, not nested (P!F3:G5 `2||#3`; arrays.md "PIVOTBY's Total
   columns are `""` only for one function over several value columns", P!Y97), while with one value column the Total
   column is nested (P!D11:D13 `64|2x1|1`, `64|4x1|1`).
2. **Shown value.** The cell displays `#VALUE!`; a spilled grid shows `#VALUE!` in each nested cell
   (probes/groupby-lifted-args Calc!B49:B52, one-record groups included).
3. **INDEX.** `INDEX(result,r,c)` that selects a nested cell returns its array: TYPE 64, ROWS/COLUMNS its shape,
   `INDEX(x,1,1)` its first value (P!B3..E37). `INDEX(GROUPBY(Data!A1:A4,Data!B1:B4,LAMBDA(w,w)),4,2)` spilling the Total's
   `{1;2;3;4}` is probes/scan-over-groupby A69:A72 (and `INDEX(..,1,2)`, group a's `{1;3}`, is `#SPILL!` at A68), landed with T-2398 (#2145).
4. **Element-wise lifts read an error; SUM is `#N/A` and COUNT skips** (2026-11-06 fills, the target under decision 0329).
   - A function that lifts element by element over the result reads each nested cell as an error, not as its top-left:
     `SUM(--ISERROR(PIVOTBY(..)))` counts every nested cell (t2360-array-lambda-holidays-refill-1106 P!C1, C9, C17, C25, C33:
     12, 9, 6, 3, 12; t2311-pivotby-value-columns-error-refill-1106 P!C12: 12). The error is not `#N/A`: over a two-level cell whose
     inner top-left is `#N/A`, ISNA is 0 and ISERROR TRUE (probes/t2609-nested-operators-and-lifts A44:A47, -one-cell A27). It is
     `#VALUE!`: the lifted fallback `"#"&ERROR.TYPE(G)` of `IFERROR(G&"",..)` gives "#3" at every nested cell, 1x1 and
     multi-cell, over a raw GROUPBY and over an INDEXed PIVOTBY row (t2475-stat-subexpressions-refill-1106 P!A64,
     t2625-nested-consumers-refill-1106 A32). A two-level cell's error type is not probed. Engine task: T-4737.
   - A lifted INDEX that selects nested cells gives each one's top-left: `INDEX(G,{2;3},2)` is `{3;1}`, SUM 4
     (t2609-nested-operators-and-lifts A56:A59). Engine task: T-4738.
   - An operator over the whole result keeps its shape: `&""` gives 3x3 (A13), and each nested cell becomes `#VALUE!`, whatever
     its size, kind or depth and whatever the operator (`+ - * / ^ % & =`, unary minus, against a scalar, a constant, a range or
     another result): probes/t2609-nested-operators-and-lifts A1:A30, -one-cell A1:A26, A28:A30 and -d04 A1:A17 (excel-graph,
     2026-11-06, T-2609). Ours does that (`own_error`, `array_node_to_number`); this did not change in November.
   - An operator on one nested cell taken out by INDEX works on its array: `+0`, `--` and `&""` leave `64|2x1|3` (A16:A18).
   - SUM over a result that holds nested cells is `#N/A`, with other arguments too (`SUM(1,GROUPBY(..),2)`); COUNT skips the nested
     cells (0 over GROUPBY, 90 over the t2625 PIVOTBY, whose scalar cells it counts); COUNTA counts each nested cell as one value
     (9 over the 3x3 GROUPBY, 28 over PIVOTBY). Evidence: t2592-nested-groupby-cells-refill-1106 A6:A8,
     t2594-nested-aggregates-refill-1106 A26:A29, t2625-nested-consumers-refill-1106 A13, A17, A20, A25, A26. Engine task: T-4736.
   - Unchanged since October: AVERAGE is `#N/A` (A9), ARRAYTOTEXT `#CALC!` (A10), and TEXTJOIN and CONCAT `#VALUE!` (A11, A12), as
     are the other aggregates of arrays.md "Aggregates over nested GROUPBY and PIVOTBY cells".
   - **Superseded (October fills, decision 0329 item 3).** On 2026-10-23/24 Excel read a lifted nested cell as its top-left
     (`SUM(--ISERROR(..))` 0: t2360-array-lambda-holidays P!C1, C9, C17, C25, and 1 at C33, where only the Total's non-top-left
     element was `#N/A`; t2311-pivotby-value-columns-error P!C12 0), and SUM, COUNT and COUNTA flattened nested cells (SUM 220,
     COUNT 16, COUNTA 19: t2592-nested-groupby-cells A6:A8; t2594-nested-aggregates A26:A29; t2625-nested-consumers A13, A17, A20,
     A25, A26), and a lifted ERROR.TYPE read a nested cell's non-error top-left, as `#N/A` (one `#N/A` for t2625 A32, a
     `64|13x3|?` listing for t2475-stat-subexpressions P!A64). These sidecars stay committed; the cells become known failures as the engine tasks
     above land. Ours follows them until then (T-2594's `NestedReading::Flatten`).
5. **MAP, BYROW, BYCOL and REDUCE hand the LAMBDA each nested cell whole.**
   - MAP, BYROW and BYCOL with a scalar step return an array of the result's shape (3x3, 3x1, 1x3; A1:A5), and a scalar step over PIVOTBY
     gives its values (A108, A109). Inside the step the nested cell is an array: `ROWS(x)` is 1 for a key, 2 for group b's cell and 4 for the
     Total's (A100:A102), `TYPE(x)` 64 (A103), `SUM(x)` 7 for `{3;4}` (A105). BYROW's row and BYCOL's column hold the nested
     cells (`TYPE(INDEX(r,1,2))` 64, A106, A107).
   - A step that returns its nested argument (`LAMBDA(x,x)`) is `#CALC!` (A104), an array step as in the non-goals. t2311
     P!D12's error is inferred to come from this (A104, A18; the oracle shows only IFERROR's "E"): its step `IFERROR(x&"",..)` keeps the nested cell an array (A18).
   - REDUCE visits every one of the 9 cells with the nested array as `x` (a count of 9, `a+ROWS(x)` 19; A14, A15).
   - TAKE, CHOOSECOLS, VSTACK, SORT, FILTER and an INDEX of a whole row or column keep the nested cells (A26:A32).
6. **SCAN** over a GROUPBY result is one cell (`#N/A` for `LAMBDA(w,w)`, 0 for SUM, the error for `LAMBDA(w,1/0)`:
   groupby-lifted-args A153, A161, A169). T-2398 owns that rule; this spec only supplies the nested cell it reads.
7. **A reference body is a reference cell, not a nested array.** With a one-cell reference body (`LAMBDA(x,D!D2)`), every
   spilled cell displays `#VALUE!` (probes/groupby-lambda-forms G!K193:K198), but INDEX reads the cell as the referenced value
   (`TYPE` 1, 10; A21). An element-wise lift and an operator read it as `#VALUE!` (`SUM(--ISERROR(..))` 6, ISNUMBER 0,
   REGEXEXTRACT `#VALUE!`, `INDEX(..+0,2,2)` `#VALUE!`), while SUM reads its value (60), COUNT gives 0 and COUNTA 9; INDEX of a
   body pointing at a blank cell is a true blank and at an `#N/A` cell `#N/A` (probes/t2609-nested-operators-and-lifts A24,
   A41:A43, A49:A54, 2026-11-06). These reads come from the 2026-11-06 fills only; SUM over the result reading the values
   (A50) is owned by T-4736.
8. **Nested cells nest again.** A LAMBDA that returns a GROUPBY gives a nested cell holding the inner 3x3 result, and that
   result's own cells are nested again (`64|3x3|a`, and the inner `INDEX(..,2,2)` `64|2x1`; A23, A24). Excel puts no limit of
   one level.

## Design
**Value.** A new `ArrayNode` variant, beside `ArrayNode::Lambda` (the precedent: a non-scalar element that each consumer
already has to place):

```rust
/// A cell of a GROUPBY or PIVOTBY result that holds its function's array result, or the value of a reference result
/// (SPEC-0018 rules 1, 7, 8). Never in a parsed array constant.
Nested(Arc<NestedArray>),

pub(crate) struct NestedArray {
    rows: Vec<Vec<ArrayNode>>, // non-empty, rectangular; elements may be `Nested` again (rule 8)
    kind: NestedKind,          // Array, or Reference: one cell read by INDEX as its value (rule 7)
    cells: usize,              // its cells plus those of every nested element, for the bound
    depth: usize,              // 1 + the deepest nested element's depth
}
```

`Arc` keeps a clone O(1) (results are cloned through LET bindings and the lifting driver), and the variant adds no size to
`ArrayNode` beyond one pointer. `NestedArray::new(rows, kind) -> Option<Self>` enforces the invariants: an empty or ragged
grid is no nested array, a `Reference` holds exactly one scalar, and `cells` and `depth` are computed once from the elements
with saturating arithmetic. `new` refuses (returns `None`, and the caller gives `#ERROR!`) a grid whose `depth` would pass
`MAX_NESTED_DEPTH` or whose `cells` would pass `limits::MAX_SIZE`, so no deeper or larger tree is ever built.
They are never recomputed by walking the tree. Elements may themselves be `Nested`, since Excel nests again (rule 8).

**One reader.** `ArrayNode::scalar(&self) -> &ArrayNode` is the one place an element-wise read of a nested element is
decided. As built (T-2593) it returns, for `Nested`, its top-left element taken recursively (a loop of at most
`MAX_NESTED_DEPTH` steps), and for a `Reference` the error `#VALUE!` (only INDEX reads its value, rule 7; a lift reads
`#VALUE!`, probes/t2609-nested-operators-and-lifts A41, A49); otherwise the node itself. Under the November reads (rule 4)
it returns an error for every `Nested` element, whatever its depth (T-4737), and the lifted INDEX of rule 4 takes the
top-left by its own read, not through `scalar` (T-4738). Every site that today matches `ArrayNode::Lambda` to
place a non-scalar, and that rule 4 shows to make an element-wise read, goes through it: the element-wise lifts (ISERROR,
REGEXEXTRACT's return_mode). Every other site keeps today's `#VALUE!` until a rule here covers it, for both kinds (a `Reference` cell included, until T-2594). The inventory is those sites (35 matches in 25 files: cast.rs, arithmetic.rs, functions/util.rs,
aggregate_args.rs, lift/kinds, text_kernel.rs, spill_functions.rs, ...); each gets an explicit `Nested` arm, either
`node.scalar()` or today's `#VALUE!`, so no site is left to a guess. Reuse an existing top-left helper of the lifting code if one fits
before adding `scalar()` (CLAUDE.md directive 3). The compiler finds every exhaustive match; a wildcard arm is audited by
`rg 'ArrayNode::Lambda'` in the same PR.

**Producers.**
- `Model::aggregate` (group_functions.rs) turns an array result into `ArrayNode::Nested` (kind `Array`) instead of `#VALUE!`.
  This covers a 1x1 array, a computed one, a two-parameter LAMBDA's result and a subtotal cell (rule 1). An array that
  already holds `Nested` elements is kept as it is (rule 8).
- A one-cell reference result becomes kind `Reference` (rule 7), holding the referenced value (T-2593, so that INDEX reads it:
  A21). Its other reads keep `#VALUE!` until T-2594.
- No other producer: BYROW, MAP and MAKEARRAY keep `#CALC!` for an array step.

**Consumers with their own rule.**
- INDEX selecting one element (`index_impl` and the lifted INDEX) returns the `Nested` element's array (rule 3), a copy
  of its rows (linear in its cells, within the cap; keeping the `Arc` until a consumer needs the rows is an optimization
  for later); an INDEX that selects a row or column of the result keeps the nested elements in it.
- TYPE of an argument that is a `Nested` element via `array_node_to_calc_result` sees an array: 64 (rule 3).
- `array_node_to_calc_result` (functions/util.rs) is the one place a single element becomes a `CalcResult`; there `Nested`
  becomes `CalcResult::Array`, so INDEX, LET and a LAMBDA parameter all hand the array on.
- Display and storage: `array_node_to_formula_value` (model.rs) and stringify show `#VALUE!` (rule 2); a spill stores the
  error, so no nested array outlives the formula's evaluation.
- INDEX of a `Reference` cell returns its value as a scalar (rule 7: TYPE 1). Its other reads are unprobed and keep
  `#VALUE!` (T-2594).
- MAP, BYROW, BYCOL and REDUCE (rule 5) pass the nested cell to the LAMBDA through `array_node_to_calc_result`, as an
  array. A step whose result is still an array is `#CALC!` through the existing array-step check. T-2594 owns this.
- SUM, COUNT and COUNTA read nested cells as rule 4 says (as built by T-2594 they flatten, the October reads; T-4736), and AVERAGE, ARRAYTOTEXT, TEXTJOIN and CONCAT give rule 4's errors. Each
  aggregate gets an explicit `Nested` arm (T-2594). Every flatten is charged to the walk budget below, and so is any other read of a nested element's contents (an aggregate that returns its error without reading them is not charged).
- TAKE, CHOOSECOLS, VSTACK, SORT, FILTER and row or column INDEX copy `ArrayNode`s, so the nested cells ride along with no
  new code (rule 5). Because of the `Arc`, each copy is O(1), so one nested cell can appear in up to 1,048,576 outer cells
  (`VSTACK(r,r,..)`, `CHOOSEROWS(r,SEQUENCE(1048576)^0)`). Copying is cheap; reading the copies is not, and the walk budget
  bounds that.
- An operator over the whole result reads each nested cell as `#VALUE!` (rule 4, T-2609): `own_error` and
  `array_node_to_number` already do, so no `Nested` arm of their own.

**Bound (directive 5).** Each GROUPBY or PIVOTBY call counts the cells it keeps against `limits::MAX_SIZE` (1,048,576, the
bound on any array the engine builds): the result's own cells plus each nested element's `cells`, which already includes
that element's own nested cells. A two-level result therefore pays for its inner cells once, where they are kept. Past it the
whole result is `#ERROR!`, as an oversized array is today (DEVIATION, arrays.md "Array results above 1,048,576 cells";
decision 0014). The check runs as each function result is kept, before the next call, so the work and memory stay within
one bound. The existing per-call input cap (`Functions::refused`) is unchanged.
The per-call cap does not bound reads, because copies share the nested array. **Walk budget.** Every operation that reads a
nested element's contents charges that element's `cells` to one per-evaluation budget, `MAX_NESTED_WALK_CELLS` (2^27, as
`limits::MAX_LIFTED_CELL_VISITS`), before it starts. The operations are:
- materializing it as a `CalcResult::Array` (`array_node_to_calc_result`: INDEX, LET, a LAMBDA parameter, the MAP, BYROW,
  BYCOL and REDUCE hand-over);
- flattening it (SUM, COUNT, COUNTA);
- comparing it by content (`PartialEq`).

The budget is kept beside the evaluation's other per-evaluation counters, as `MAX_CAPTURED_CELLS` is for LAMBDA captures. A
charge past it makes the reading call `#ERROR!` (DEVIATION, decision 0014). As built (T-2593) the only reader is INDEX
(`NestedArray::value`); `PartialEq` is not charged: it is O(1) for copies of one nested cell (pointer equality) and for cells
of different `cells`, and no evaluation path compares nested cells by content (the lifting driver compares `scalar()`s,
UNIQUE keys a nested cell as `#VALUE!`). **Kept budget.** The per-call cap does not bound a loop of calls whose results no
array size check sees (`REDUCE(..,LAMBDA(a,i,VSTACK(a,GROUPBY(..))))`), so each evaluation also charges the cells of every
nested cell's own grid (`grid_cells`: an inner level was charged when its own call kept it) against
`MAX_NESTED_KEPT_CELLS` (2^22, as `MAX_CAPTURED_CELLS`); past it the result is `#ERROR!` (DEVIATION). The charge is made per occurrence, so
`SUM(VSTACK(r,r,..))` over a nested cell of 2^20 cells is `#ERROR!` after 128 copies are read, not after 2^40 steps. Reads
that never look inside a nested element cost nothing extra: `scalar()`, TYPE, ROWS and COLUMNS of the outer array, a copy.
Nesting depth is bounded too. Each level needs one more GROUPBY call evaluated inside a LAMBDA, so the evaluator's
recursion guard (`evaluation.rs`, `DEFAULT_MAX_DEPTH` and the stack guard) already limits it. A result whose `depth` would
pass a constant `MAX_NESTED_DEPTH` (64) is `#ERROR!` anyway (DEVIATION, decision 0014), refused in `NestedArray::new`.
That way `Drop`, `PartialEq` and the hash, which recurse through nested elements, never recurse deeper than that constant,
even when the recursion guard is configured differently; `scalar()` is a loop. The cap is per call; the walk budget above is what bounds a whole evaluation once nested results are copied.

**Hash and equality.** `ArrayNode` derives `PartialEq`; `Nested` compares by content, kind included, at most
`MAX_NESTED_DEPTH` deep. The lifting driver's class hash (functions/lift/driver.rs) hashes the top-left (`scalar()`), the
value a lifted read sees.

## Alternatives considered
- **An id into a side store, as `ArrayNode::Lambda(usize)`.** Needs the store's lifetime managed per evaluation and charged
  separately; the array is plain data, so an `Arc` owns it without a store.
- **A `CalcResult::NestedArray` only at the top level.** The nested value lives inside an array (a cell of the result), so it
  has to be an `ArrayNode`.
- **Keep `#VALUE!` and special-case INDEX(GROUPBY(..)).** INDEX would have to re-run the group's function, and the lifted
  top-left reads (rule 4) would still be wrong.

## Test plan
- Unit tests: `NestedArray::new` invariants (empty, ragged, `Reference` of one scalar, `cells` and `depth`); `scalar()` through
  two levels; the cap (a LAMBDA returning `SEQUENCE(1048576)` for two groups is `#ERROR!`, and so is a two-level result whose
  inner cells pass it); the depth cap; two levels as in rule 8 (A23, A24); the walk budget (`SUM(VSTACK(r,r,..))` and
  `REDUCE(0,VSTACK(r,..),LAMBDA(a,x,a+SUM(x)))` over a result `r` holding a ~2^20-cell nested cell are `#ERROR!` in bounded
  time).
- Eval for T-2593: the 47 t2360 P! cells, t2311 P!C12, gen-m7/21b4eef18424ce94 A1504, and the t2592 cells for rules 1, 3 and 8
  (A16:A18, A22:A24, A26:A41).
- Eval for T-2594: t2311 P!D12 and the t2592 cells for rules 4, 5 and 7 (A1:A15, A21, A100:A109).
- Through T-2398: groupby-lifted-args A153 and gen-m7/b24ce74f08d3f91a A1850.
- No regressions in probes/groupby-*, pivotby-* and gen-m7 (the existing `#VALUE!` cells of rule 7 stay, and so do the `""`
  Total columns of rule 1's exception, t2360 P!F3:G5, which pass today).

## Rollout
1. T-2592 (research, S, done): the probe, probes/t2592-nested-groupby-cells, recorded in arrays.md.
2. T-2602 (research, S): this revision from T-2592's probe.
3. T-2593 (engine, M, after 2): `ArrayNode::Nested` (any depth), `scalar()`, the producer, INDEX/TYPE, display, the
   bounds. Owns rules 1-3, rule 4's top-left reads, and rule 8.
4. T-2594 (engine, S, after 3): MAP, BYROW, BYCOL and REDUCE (rule 5), the aggregates' flattening and errors (rule 4), and
   the `Reference` cell (rule 7).
5. T-2398 (SCAN over a GROUPBY result) depends on T-2593 for its `LAMBDA(w,w)` cells.
6. T-4661 (research, S, done): the revision for Excel's change of 2026-11 (rule 4; decision 0329).
7. T-4736 (engine, S, after 6): SUM `#N/A`, COUNT skipping and COUNTA one per nested cell, and SUM over a reference-body
   result (rules 4 and 7).
8. T-4737 (engine, S, after 6): element-wise lifts read a nested cell as an error (rule 4, "One reader").
9. T-4738 (engine, S, after 6): a lifted INDEX of nested cells gives each one's top-left (rule 4).
