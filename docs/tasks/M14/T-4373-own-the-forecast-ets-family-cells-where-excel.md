---
id: T-4373
title: "Own the FORECAST.ETS family cells where excel-graph gives #N/A (re-probe 2026-11-05)"
milestone: M14
lane: functions-stat
status: blocked
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The 16 visibly-off cells of probes/forecast-ets-reprobe-2026-11 and the 40 of probes/forecast-ets-longer-series have
an owner: the web oracle gives `#N/A` on every FORECAST.ETS* call while the engine computes a number or a validation
error (decision 0221), so they stay known failures until a desktop-Excel backend exists.

## Blocked because
Blocked because: no oracle can evidence these cells. Excel for the web returns `#N/A` for the FORECAST.ETS family and
`#VALUE!` for CELL's desktop-only info types, and desktop Excel is out of scope (decision 0266). Unblock only if a
desktop or web oracle computes them; until then the cells are Phase 1 rate exceptions (decision 0308).

## Context
T-3641 re-probed excel-graph on 2026-11-05; see statistics.md "FORECAST.ETS family: excel-graph never computes it".
Cells: probes/forecast-ets-reprobe-2026-11 Sheet1!I1:I8 and Sheet1!J1:J8. Desktop work is out of scope (decision 0266).
- Gap triage 2026-11-05 (main 3b2f779e, clean `make eval`): probes/forecast-ets-longer-series was owned by T-2425,
  dropped by decision 0266, and was left with no owner (`make unowned-off ALL=1` listed its 40 cells, the only unowned
  visibly-off cells of the run). Same root cause: Excel (web) is `#N/A` on every cell; the engine gives numbers
  (K1 `FORECAST.ETS(25,B1:B24,A1:A24)` is 148, K14 SEASONALITY 4, K11 CONFINT 0) and `#NUM!` on the out-of-range
  arguments (K35, K39:K42). K37 and K38 already match. statistics.md cites this case (K3, K8, K14, K19, K25, K30).
- probes/forecast-ets-longer-series.case.json (40): Sheet1!K1:K36, Sheet1!K39:K42

Owns only decision 0308's and 0315's cells: the desktop-only CELL cells listed in eval/milestones/phase1-exceptions.tsv (calipers goldens,
info types color, filename, format, parentheses, prefix, protect, width), and every FORECAST.ETS-family line there.

Cell lists. The ownership check (scripts/exit_owners.py) reads the cells a task names, so they are listed here once.
Phase 1 rate exceptions under decision 0308 (eval/milestones/phase1-exceptions.tsv):
- calipers/pv_goldens/pv26_001/golden.xlsx (1): Ticker0001_Validation_Sheet!A2
- calipers/pv_goldens/pv26_011/golden.xlsx (1): CD market model!B3
- calipers/pv_goldens/pv26_011/golden.xlsx (1): UC market model!B3
- calipers/pv_goldens/pv26_2_001/golden.xlsx (1): Ticker0001_Validation_Sheet!A2
- calipers/pv_goldens/pv26_2_011/golden.xlsx (1): CD market model!B3
- calipers/pv_goldens/pv26_2_011/golden.xlsx (1): UC market model!B3
- calipers/roundtrip/formulas_information/golden.xlsx (147): Sheet1!E140, E141, E142, E143, E144, E145, E146, E147, E148, E149, E150, E151, E152, E153, E154, E155, E156, E157, E158, E159, E160, G140, G141, G142, G143, G144, G145, G146, G147, G148, G149, G150, G151, G152, G153, G154, G155, G156, G157, G158, G159, G160, H140, H141, H142, H143, H144, H145, H146, H147, H148, H149, H150, H151, H152, H153, H154, H155, H156, H157, H158, H159, H160, I140, I141, I142, I143, I144, I145, I146, I147, I148, I149, I150, I151, I152, I153, I154, I155, I156, I157, I158, I159, I160, J140, J141, J142, J143, J144, J145, J146, J147, J148, J149, J150, J151, J152, J153, J154, J155, J156, J157, J158, J159, J160, K140, K141, K142, K143, K144, K145, K146, K147, K148, K149, K150, K151, K152, K153, K154, K155, K156, K157, K158, K159, K160, N140, N141, N142, N143, N144, N145, N146, N147, N148, N149, N150, N151, N152, N153, N154, N155, N156, N157, N158, N159, N160
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
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A329
- gen-m9/5b6cc12a94f7bf14.case.json (1): Calc!A77
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A153
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
- gen-m9/9bc247733a1ca9fa.case.json (1): Calc!A10
- gen-m9/9e2225ead8a6a07b.case.json (1): Calc!A235
- gen-m9/a013a45a8fbcab4f.case.json (1): Calc!A240
- gen-m9/a2262ea466fcc142.case.json (1): Calc!A259
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A44
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A141
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
- probes/forecast-ets-longer-series.case.json (40): Sheet1!K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13, K14, K15, K16, K17, K18, K19, K20, K21, K22, K23, K24, K25, K26, K27, K28, K29, K30, K31, K32, K33, K34, K35, K36, K39, K40, K41, K42
- probes/forecast-ets-optional-settings-shape.case.json (12): Sheet1!M29, M30, M31, M32, M33, M34, M41, M42, M43, M44, M45, M46
- probes/forecast-ets-reprobe-2026-11.case.json (16): Sheet1!I1, I2, I3, I4, I5, I6, I7, I8, J1, J2, J3, J4, J5, J6, J7, J8
- probes/stat-argument-modes.case.json (8): Sheet1!H266, H267, H270, H271, H274, H275, H277, H278
Phase 1 rate exceptions under decision 0315 (calipers Mac goldens the web answers as the engine does):
- calipers/roundtrip/formulas_math_trig/golden.xlsx (16): Sheet1!C23, C25, D22, D24, E22, E23, E24, E25, F22, F24, G23, G25, H22, H23, H24, H25
- calipers/roundtrip/formulas_text/golden.xlsx (1): Text Functions!C190
- calipers/roundtrip/formula_stress_test/golden.xlsx (1): Information!B3

## Acceptance criteria
- [ ] If a desktop or web oracle ever returns numbers for the family, refill both cases and compare the cells
      (reprobe I1:I8 and J1:J8; longer-series K1:K42).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing or changing the ETS algorithm.
