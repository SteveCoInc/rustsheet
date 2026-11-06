---
id: T-4390
title: "CUMPRINC's short-range forms: Excel's own form where neither the cut nor the geometric sum is (T-4349 follow-up)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4349]
---

## Goal
CUMPRINC and CUMIPMT over a short range (at most 32 periods) below a leading exponent of 0.3 return Excel's own
principal where neither form the engine has (the per-period sum or closed form below `GEOMETRIC_MIN_LEAD`, the
geometric sum in T-4349's band) is Excel's, or the Outcome shows which operation order Excel uses there and why no
double-precision form reproduces it.

## Context
- T-4349 compared the two short-range forms cell by cell (docs/research/t4349/dump.py, `cumulative*` fits): where
  they differ, each is bit-exact on only about 30% of the cells (265 and 286 of 923), so neither is Excel's algorithm there. Its band
  (`short_takes_geometric` in functions/financial/cumulative.rs, decision 0296) is the best selector between them it
  found, not Excel's form. The CUMIPMT cells are visibly off because the interest carries an ulp of the principal or
  of PMT as tens to thousands of its own ulps (T-3118).
- Owns the seven cells T-4349 moved from bit-exact to 1 ulp off under decision 0296 (decision 0083's conditions):
  probes/fin-cumprinc-near-1e15 Sheet1!B2, B3; probes/fin-cumprinc-single-period-rate-scan Sheet1!K52, K57, K59;
  probes/fin-cumprinc-underflow-ends Sheet1!D2, D3.
- Owns the visibly-off cells of probes-numeric/t4349-cumprinc-short-selector-fresh-sweep (190, all CUMIPMT):
  S!B2, B7, B9, B10, B11, B15, B17, B20, B21, B22, B24, B25, B26, B28, B31, B33, B35, B40, B45, B46, B48, B51, B52, B53, B55, B66, B70, B71, B72, B73, B74, B75, B79, B80, B81, B82, B84, B85, B92, B93, B100, B102, B103, B104, B105, B107, B111, B112, B116, B118, B119, B120, B122, B123, B124, B126, B127, B129, B130, B135, B139, B141, B142, B147, B148, B151, B153, B154, B155, B156, B159, B163, B164, B167, B169, B170, B172, B179, B183, B184, B185, B186, B188, B190, B194, B196, B197, B198, B201, B204, B206, B207, B210, B213, B215, B216, B218, B220, B221, B223, B225, B227, B228, B229, B231, B232, B234, B238, B242, B243, B244, B247, B248, B249, B252, B254, B255, B259, B260, B261, B262, B263, B264, B265, B266, B269, B272, B278, B279, B285, B289, B290, B292, B293, B294, B295, B296, B297, B298, B300, B303, B307, B308, B311, B313, B315, B318, B319, B321, B322, B325, B327, B328, B329, B333, B334, B336, B338, B340, B341, B343, B344, B345, B347, B350, B351, B353, B358, B359, B364, B365, B367, B368, B369, B372, B374, B375, B377, B379, B381, B382, B383, B384, B385, B386, B387, B388, B391, B395, B399.
- Owns, from T-4349, the visibly-off cells of probes-numeric/t3524-cumprinc-lead-fresh-sweep still off after it
  (57): S!B3, B9, B10, B19, B37, B45, B54, B70, B71, B76, B77, B79, B85, B86, B101, B104, B106, B110, B119, B126, B128, B131, B132, B148, B152, B154, B170, B175, B177, B186, B190, B197, B206, B214, B223, B226, B228, B233, B234, B241, B258, B275, B287, B336, B346, B379, B385, B401, A404, B415, B432, B434, B439, B456, B464, B497, B500; and of probes-numeric/t3524-cumprinc-tiny-rate-sweep (140): S!B4, B5, B9, B10, B11, B12, B15, B17, B26, B32, B33, B36, B37, B39, B50, B52, B53, B54, B55, B59, B60, B69, B70, B78, B81, B85, B89, B90, B91, B98, B100, B101, B102, B108, B109, B111, B115, B121, B122, B126, B127, B131, B139, B140, B141, B142, B144, B150, B154, B160, B161, B162, B163, B164, B166, B169, B186, B193, B195, B197, B205, B209, B215, B218, B220, B224, B226, B236, B242, B243, B247, B251, B256, B266, B267, B270, B271, B284, B286, B290, B292, B295, B297, B299, B301, B309, B312, B319, B323, B331, B332, B333, B336, B338, B340, B347, B349, B350, B351, B353, B356, B357, B358, B374, B376, B377, B378, B393, B395, B396, B410, B412, B420, B426, B428, B429, B435, B437, B446, B452, B453, B457, B462, B464, B465, B470, B474, B475, A476, B480, B482, B486, B487, B488, B490, B493, B494, B496, B499, B500.
  The long-range cells among them (rows 251-500) keep the geometric sum (T-3524); a cause found here for the short
  ones may reach them, otherwise file them on.
- Owns, from T-4110, four Gate A points (eval/accuracy/known-failures.tsv), CUMIPMT at rates 1.5E-12 to 4.9E-10 where
  `PMT·m - CUMPRINC` cancels and ours is 1E-9 to 2.5E-6 relative off Excel: accuracy/CUMIPMT/s1 Calc!A87, A90, A92 and
  A189. A90 (5 periods) and A189 (3) are short ranges. A87 (periods 152-192, 41 periods) and A92 (30-71, 42) are long
  ranges, outside the Goal's 32 periods: this task takes them on explicitly, as the same `PMT·m - CUMPRINC`
  cancellation at a tiny rate. If the short-range cause does not reach them, file them on with their known-failure
  lines.
- Candidate causes to test first: a per-period PPMT sum built as PMT less IPMT (Excel's ToolPak loop) in doubles,
  the order of `pmt·series·v^lead`, and Excel's own `ln(1+rate)` (an ulp off libm at some rates, T-2521).
- T-4349's post-freeze sweep drew only inside its band; a fresh sweep here should also take rows just outside it (5
  periods, rates just below 1E-3, leads in 0.095..0.3), where the band's edges were never probed after the freeze.
- A new selector or form chosen by scoring oracle cells is a fit: decision 0188 R4 (hold-out, a fresh sweep drawn
  after the freeze, `FITTED` tag).

## Acceptance criteria
- [ ] A candidate form beats the engine on the owned short-range cells (fit lab, hold-out counts) and on a fresh sweep
      drawn after its freeze, with no visibly-off regression beyond decision 0083's allowance; or the Outcome shows
      no candidate does, with the candidates ranked.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Type 0's underflowed principal (T-3141); long ranges unless the same cause reaches them.
