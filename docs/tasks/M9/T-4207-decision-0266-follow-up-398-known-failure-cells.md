---
id: T-4207
title: "Decision 0266 follow-up: 398 known-failure cells only dropped M14 tasks owned (342 FORECAST.ETS cells visibly off against the web's #N/A) are unowned and back in M9's Phase 1 rate"
milestone: M9
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The known-failure cells whose only owners were the M14 tasks decision 0266 dropped have a recorded status again: each is
either an accepted cell that excel-graph cannot settle (decision 0266 rule 2: "stays as implemented ... a gap triage does
not file a task for it"), left out of the Phase 1 rate the way decision 0143 left desktop-owned cells out, or it is
counted, by a decision record that says which. `make unowned-off ALL=1` and the M9 exit row then agree with that
decision instead of reflecting a mechanism that can no longer fire.

## Context
- Gap triage 2026-11-04c (`make eval` on origin/main 9a01e9b25, excel-graph sidecars). `make unowned-off ALL=1` lists
  348 visibly-off cells with no open owner. 5 of them are owned only by deferred tasks: 4 COMPLEXs cells and
  gen-m9/60b0b6e69ad35a45 Calc!A153. The other 343 all call the FORECAST.ETS family: 253 are unattributed,
  FORECAST.ETS.STAT has 39, FORECAST.ETS.SEASONALITY 23, FORECAST.ETS.CONFINT 16 and FORECAST.ETS 12.
  One of the 343 is web-observable: gen-m9/fa25422d6f18c157 Calc!A75, Excel `#REF!`. It goes to T-4208. The other
  342 expect the web's `#N/A`. 322 are gen-m9 cells and 20 are probes cells: 12 in
  probes/forecast-ets-optional-settings-shape and 8 in probes/stat-argument-modes. Ours: 262 `#NUM!`, 62 `#VALUE!`,
  3 `#DIV/0!`, and 15 numbers from the AAA model (decision 0221), e.g. probes/stat-argument-modes H266
  `FORECAST.ETS(9,A1:A8,E1:E8)` 8.357142857142856.
- Why they are unowned: T-2425 named all of them. Decision 0266 (owner, 2026-11-04) set T-2425 and every other open M14
  task to `dropped`. scripts/desktop_owned.py (decision 0143) keeps only cells "that an open task of milestone M14
  owns", so eval/runs/desktop-owned.tsv is now empty. Re-running it with the dropped tasks counted as open gives 398
  cells: T-2425 343, T-2663 40 (calipers goldens), T-1075 11 (ironcalc-calc BESSEL, IF_ARRAY, ATAN2_POWER; poi
  EverythingTests), T-2704 4 (calipers formulas_engineering). They are listed below, 397 of them (A75 is left to
  T-4208). 342 are visibly off in this run. calipers and poi were not scored. The ironcalc-calc ones are exit exceptions or not visibly off.
- Effect today: the Phase 1 rate counts the FORECAST.ETS cells again. FORECAST.ETS.STAT (47 of 174 attributed cells
  visibly off), FORECAST.ETS.CONFINT (27/74), FORECAST.ETS.SEASONALITY (29/195) and FORECAST.ETS (27/89) are 4 of the
  53 M9 functions below the Phase 1 rate. Gate B lists them as well, as 33/157, 10/55, 21/186 and 6/64 (report.md
  "Fifteen-digit parity checkpoint"). No fix can move them. Excel for the web is `#N/A` for every call that passes the argument step
  (statistics.md "FORECAST.ETS family: excel-graph never computes it", "... on excel-graph: argument errors, text and
  lifting decide before the #N/A (T-2255)"). The engine's errors and numbers come from the model steps that statistics.md
  "FORECAST.ETS family: the model before Excel evidence" marks UNVERIFIED under decision 0221. Returning the web's `#N/A`
  would undo decision 0221, and decision 0266 rule 2 says such behavior stays as implemented.
- Stale references: eval/milestones/phase1-exceptions.tsv and exit-exceptions.tsv name T-1075, now dropped, as the
  owner of their rows. Their header comments still describe decision 0039's M14 routing. The `M14-owned, left out`
  column and the R7 rule of report.md ("cells only open M14 tasks own leave the row") can no longer be non-zero.
  T-3004 (todo) asks to give 10 probe cells to T-2425 and is moot now.
- Related open work, not duplicated: T-3641 (re-probe whether excel-graph computes the family today). If it finds that
  it does, the cells become fixable, and this decision should say what happens then. T-3994 and T-4076 cover cells
  owned only by deferred tasks, a different rule.
- Steward 2026-11-05 (second run): T-3641 landed (#3392; excel-graph still `#N/A` for the whole family), so the 40
  visibly-off cells of probes/forecast-ets-longer-series K1:K42 it owned (all but K37 and K38, all four functions; ours
  numbers or `#NUM!` from the AAA model against the web's `#N/A`) had no open owner and are the M9 exit cells `make
  exit-owners MILESTONE=M9` listed. This task owns them too, under the same rule as the 342 above.
- Code: scripts/desktop_owned.py, scripts/exit_owners.py (`read_desktop_owned`, `counted_in_rate`, `--all-off`), the
  `--desktop-owned` path of crates/rustsheet-eval (v15_checkpoint), eval/milestones/*.tsv (decision 0044 protects
  them: a line changes only in the PR of the decision that names it).

Cells (known-failure lines of eval/baseline, owned here until the decision lands). From T-2425:
- gen-m9/09e6a8028f5acfae.case.json (1): Calc!A383
- gen-m9/0a614cfe974b59c3.case.json (2): Calc!A173, A347
- gen-m9/0a9226016e8f5f33.case.json (1): Calc!A210
- gen-m9/0a957119475760c5.case.json (1): Calc!A239
- gen-m9/165db58d1540f014.case.json (2): Calc!A145, A306
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A354
- gen-m9/22656c225ff4f22a.case.json (48): Calc!A16, A20, A33, A34, A43, A52, A62, A63, A64, A66, A79, A81, A96, A99, A108, A115, A130, A135, A136, A139, A163, A176, A179, A196, A211, A215, A225, A227, A229, A230, A231, A239, A253, A254, A255, A272, A273, A283, A308, A319, A323, A326, A329, A334, A366, A374, A379, A386
- gen-m9/23d82b62e9a85bdd.case.json (1): Calc!A1
- gen-m9/30e20ba3302fea8b.case.json (1): Calc!A349
- gen-m9/31f09e6c48ca245c.case.json (1): Calc!A321
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A265
- gen-m9/3b7990bd4e3307ba.case.json (2): Calc!A252, A351
- gen-m9/43a13a091907b402.case.json (59): Calc!A4, A5, A7, A11, A15, A24, A26, A27, A35, A36, A37, A42, A61, A67, A87, A91, A95, A102, A107, A108, A113, A142, A147, A151, A155, A166, A167, A173, A175, A183, A192, A208, A212, A214, A227, A243, A244, A259, A269, A270, A280, A290, A299, A304, A306, A307, A309, A320, A323, A326, A339, A364, A366, A370, A382, A384, A387, A392, A395
- gen-m9/46ac3ef5e61583ca.case.json (1): Calc!A291
- gen-m9/5b6cc12a94f7bf14.case.json (1): Calc!A77
- gen-m9/623663cacea53bbc.case.json (1): Calc!A160
- gen-m9/67f5e0fe790e6bec.case.json (50): Calc!A8, A10, A14, A27, A37, A59, A93, A98, A99, A106, A109, A111, A121, A131, A139, A144, A156, A159, A161, A167, A168, A186, A190, A191, A197, A198, A213, A214, A218, A246, A252, A254, A272, A297, A315, A321, A329, A333, A336, A339, A350, A351, A359, A367, A368, A370, A373, A386, A388, A398
- gen-m9/73f053591a54bc43.case.json (1): Calc!A30
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A253
- gen-m9/7f7ca726c3f68554.case.json (1): Calc!A387
- gen-m9/895a33aa8352864a.case.json (1): Calc!A158
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A142
- gen-m9/932b33b1ef8b7608.case.json (1): Calc!A336
- gen-m9/941c409532603d32.case.json (1): Calc!A227
- gen-m9/9735c5ab72fa5ecf.case.json (1): Calc!A358
- gen-m9/9e2225ead8a6a07b.case.json (1): Calc!A235
- gen-m9/a013a45a8fbcab4f.case.json (1): Calc!A240
- gen-m9/a2262ea466fcc142.case.json (1): Calc!A259
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A44
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A392
- gen-m9/afc00c8b70e78baf.case.json (1): Calc!A32
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A69
- gen-m9/c2684066d8df263e.case.json (1): Calc!A241
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A116
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A187
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A61
- gen-m9/cde551ea787f713a.case.json (1): Calc!A280
- gen-m9/d208bbafc3bc1997.case.json (1): Calc!A243
- gen-m9/d5fd9e05e9e6f466.case.json (1): Calc!A129
- gen-m9/e6cf3b20a014e6a3.case.json (2): Calc!A183, A293
- gen-m9/e8d55ec393460bb7.case.json (1): Calc!A110
- gen-m9/ec2a1d9194a84396.case.json (66): Calc!A4, A8, A12, A14, A18, A26, A32, A40, A42, A47, A55, A61, A67, A68, A69, A74, A75, A88, A94, A95, A115, A117, A118, A126, A128, A136, A143, A150, A152, A158, A160, A168, A170, A171, A197, A200, A202, A207, A210, A213, A218, A219, A226, A235, A238, A242, A249, A250, A251, A254, A266, A271, A290, A293, A295, A310, A316, A346, A354, A355, A357, A367, A375, A378, A396, A398
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A312
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A214
- gen-m9/f81bd3b6dc373e5c.case.json (1): Calc!A331
- gen-m9/f82473275380882e.case.json (1): Calc!A20
- gen-m9/fa25422d6f18c157.case.json (52): Calc!A3, A6, A11, A16, A17, A29, A42, A53, A81, A91, A96, A97, A116, A119, A128, A131, A137, A144, A151, A177, A181, A186, A187, A189, A207, A215, A227, A231, A235, A248, A252, A254, A258, A263, A272, A273, A278, A280, A299, A306, A316, A320, A332, A353, A360, A365, A371, A377, A378, A382, A384, A400
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A58
- probes/forecast-ets-optional-settings-shape.case.json (12): Sheet1!M29, M30, M31, M32, M33, M34, M41, M42, M43, M44, M45, M46
- probes/stat-argument-modes.case.json (8): Sheet1!H266, H267, H270, H271, H274, H275, H277, H278
Added by gap triage 2026-11-05 (`make eval` and `make unowned-off ALL=1` on origin/main b6cf9465): one more cell of
the same class, which no open task names. `FORECAST.ETS.CONFINT(30,7,Data!C$8,Data!$B12)` (values a TRUE cell, timeline
a blank cell) inside DURATION's yld: excel-graph `#N/A` (the web's ETS answer passed through DURATION), ours `#NUM!`
from the model steps of decision 0221:
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A141

Added by gap triage 2026-11-05b (`make eval` on origin/main 8465f6f1; `scripts/exit_owners.py --all-off` and
`--milestone M9`): the 40 known-failure cells of the excel-graph probe that first showed the family is `#N/A` on the
web (statistics.md "FORECAST.ETS family: excel-graph never computes it", T-1293). Their owners were T-1553 (dropped by
decision 0266) and T-3641, which closed on 2026-11-05 after its re-probe, so they are the only unowned visibly-off
cells of the run (80 lines of `exit_owners.py --milestone M9`: each cell counts once for the probe row and once for
`rate:`). Excel `#N/A` on every cell; ours the AAA model's numbers (148.0 at K1, 4.0 at K8) or `#NUM!` (K35, K39 to K42):
- probes/forecast-ets-longer-series.case.json (40): Sheet1!K1:K36, K39:K42

One more cell of the class, owned today only through an incidental mention of its case in T-3485 (an F-quantile task):
`CORREL(FORECAST.ETS.SEASONALITY(Data!A1:B12,LOG(..),1,(-2.5)),Data!A3:C10)`, excel-graph `#N/A`, ours `#NUM!` from an
aggregation of -2.5, which the web answers with `#N/A` like every other call:
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A329

The 4 COMPLEXs.xlsx cells the Context counts as owned only by deferred tasks are T-1075's too (Mac 16.0300 cached
digits: N6 `IMREAL(IMTAN(D6))`, AL12 and AL26 `IMREAL(IMSQRT(..))`, K25 `IMAGINARY(IMCOS(D25))`, decision 0196). Their
other owner, T-3434, is deferred, so they still show in `make unowned-off ALL=1`; the ruling covers them as well:
- ironcalc-calc/ENGINEERING/COMPLEXs.xlsx (4): Sheet1!N6, AL12, K25, AL26

From T-1075:
- ironcalc-calc/ENGINEERING/BESSEL.xlsx (2): All!D18, F18
- ironcalc-calc/LOGICAL/IF_ARRAY.xlsx (3): Sheet1!C4, C11, E4
- ironcalc-calc/MATH_AND_TRIGONOMETRY/ATAN2_POWER.xlsx (1): Sheet1!D14
- poi/FormulaEvalTestData_Copy.xlsx (5): EverythingTests!G936, H936, J136, J1328, K936
From T-2663:
- calipers/pv_goldens/pv26_001/golden.xlsx (1): Ticker0001_Validation_Sheet!A2
- calipers/pv_goldens/pv26_011/golden.xlsx (2): CD market model!B3; UC market model!B3
- calipers/pv_goldens/pv26_2_001/golden.xlsx (1): Ticker0001_Validation_Sheet!A2
- calipers/pv_goldens/pv26_2_011/golden.xlsx (2): CD market model!B3; UC market model!B3
- calipers/roundtrip/formulas_math_trig/golden.xlsx (33): Sheet1!C23, C25, D22, D24, D70, E19, E22, E23, E24, E25, E46, E48, E60, E71, F22, F24, F68, G19, G23, G25, G69, G72, G75, H19, H20, H22, H23, H24, H25, H26, H71, H74, J72
- calipers/roundtrip/formulas_text/golden.xlsx (1): Text Functions!C190
From T-2704:
- calipers/roundtrip/formulas_engineering/golden.xlsx (4): Sheet1!D26, D27, J26, J27

## Acceptance criteria
- [ ] A decision record (`make new-decision`) rules how cells count when their only owner was dropped by decision 0266
      and excel-graph cannot settle them. At least two options are compared: (a) list them in
      eval/milestones/phase1-exceptions.tsv (and exit-exceptions.tsv where the exit row counts them), naming the new
      decision and no dropped task; (b) count them like any other visibly-off cell, so the four FORECAST.ETS functions
      stay below the Phase 1 rate until T-3641 or a later desktop decision. The record says what happens to them if
      T-3641 finds that the web computes the family.
- [ ] scripts/desktop_owned.py, exit_owners.py and the eval's `--desktop-owned` path either implement the ruling or are
      retired, with their tests updated. No code path depends on an open M14 task any more, since none can exist.
      The report's "M14-owned" column and the R7 wording follow the ruling.
- [ ] The rows of phase1-exceptions.tsv and exit-exceptions.tsv that name T-1075 name an owner the ruling accepts
      (decision 0044: in the PR of that decision).
- [ ] T-3004 is set to `dropped` with an Outcome that points here.
- [ ] After `make eval`, `make unowned-off ALL=1` lists none of the cells above (each is owned or excepted by the
      ruling). eval/baseline is unchanged: no F line added or removed by this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing what the FORECAST.ETS family computes (decision 0221; decision 0266 rule 2).
- gen-m9/fa25422d6f18c157 Calc!A75, which Excel gives as `#REF!` (T-4208).
- Re-probing excel-graph (T-3641).
