---
id: T-4336
title: "Owners for the 484 last-digit F.DIST and F.DIST.RT cells of T-3580's fresh sweep no task names (family residue, Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
Every F.DIST and F.DIST.RT cell of probes-numeric/t3580-f-fresh listed below either matches Excel bit for bit, is
moved to a narrower open task that names its `bratio::dispatch` branch, or is listed in the Outcome with Excel's
value, ours, the 50-digit math and the reason Excel's last digits cannot be derived.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 4e3993c9e). Every visibly-off cell has an open owner
  (`make unowned-off ALL=1` prints 0), but checking every failing cell of report.json against the open tasks (the
  `scripts/exit_owners.py` ownership rule) leaves 563 failing cells that no open task names, and 484 of them are
  here. T-4026 and T-4029 routed this sweep's visibly-off cells (to T-3592, T-3323, T-3712 and T-3954) and the
  sweep's fit-lab split, but nobody took its bit-level remainder.
- By function and verdict: F.DIST (column B, the cdf) 211 NumericUlp and 44 NumericClose; F.DIST.RT (column A) 202
  NumericUlp and 27 NumericClose. Median 2 ulp, maximum 151 ulp (F.DIST) and 26 ulp (F.DIST.RT).
- Where the first wrong significant digit is (triage count, the half-unit boundary of decision 0267 item 1): F.DIST 4 cells at
  the 14th digit, 12 at the 15th, the other 239 past it; F.DIST.RT none before the 16th. All are within half a unit
  of the 13th digit, so under decision 0267 item 2 (the 13-digit bar for the incomplete-beta family) they are
  Phase 1b residue (`residue13` or better), and the report counts none of them as visibly off. Decision 0267 item 4
  says Tier C (14th digit) gets no new work outside a Tier A/B task, and Tier D (15th digit) is cut. Decisions 0114
  item 3 and 0188 R5 put the task under `defer: phase-1b`.
- Examples (`make eval-show CASE=probes-numeric/t3580-f-fresh.case.json`):
  - B1037 `F.DIST(0.015715540718307,10,20,TRUE)`: Excel 5.4388358266589914E-8, ours 5.438835826658986E-8.
  - B1131 `F.DIST(0.096754,12,27,TRUE)`: Excel 7.066107301141243E-5, ours 7.066107301141227E-5.
  - A1000 `F.DIST.RT(1.603438,5,50)`: Excel 0.17638803929116312, ours 0.17638803929116317.
- Evidence already in eval/: the sweep (excel-graph), its 50-digit math sidecar
  docs/research/t3580/t3580-f-fresh.math50.json and the scorers in docs/research/t3580/ (score_train.py, split.py,
  RESULTS.md). No new Excel point is needed to label the branches. Decision 0267 item 5 freezes probes-numeric: no
  new probes-numeric case for this task.
- Related open tasks (the T-3229 split): T-3592 owns 27 visibly-off cells of this sweep and T-3323 owns 18 (bfrac
  bit level); T-3712 (L140 bup+bgrat with n >= 2 bup terms) and T-3954 (bfrac at a tiny x0) own branches some of
  these cells will land on. The cells below are none of theirs. A cell whose branch matches one of those tasks moves
  to it, and both files say so.
- Cells:
  - probes-numeric/t3580-f-fresh.case.json (484): Calc!A5, A9, A26, A29, A34, A40, A50, A52, A56, A57, A72, A84, A109, A130, A144, A165, A169, A171, A175, A177, A181, A185, A193, A201, A213, A223, A226, A237, A245, A246, A262, A266, A279, A288, A304, A308, A310, A316, A317, A334, A341, A346, A367, A374, A379, A380, A382, A393, A402, A407, A408, A410, A413, A420, A423, A424, A434, A435, A436, A440, A443, A446, A449, A450, A454, A458, A459, A461, A465, A471, A472, A479, A480, A482, A487, A489, A491, A492, A500, A501, A503, A505, A511, A512, A515, A517, A522, A526, A527, A529, A535, A537, A538, A540, A547, A553, A557, A568, A571, A573, A574, A575, A579, A584, A585, A590, A594, A605, A607, A609, A611, A612, A617, A619, A621, A622, A627, A628, A630, A638, A639, A641, A642, A653, A662, A663, A664, A665, A666, A668, A675, A676, A680, A682, A687, A688, A691, A704, A714, A737, A742, A764, A766, A780, A791, A803, A808, A826, A829, A835, A836, A841, A857, A869, A873, A875, A884, A889, A895, A898, A908, A909, A913, A921, A922, A925, A928, A956, A957, A958, A963, A968, A974, A982, A987, A989, A991, A1000, A1004, A1008, A1013, A1014, A1015, A1019, A1023, A1027, A1038, A1044, A1051, A1059, A1065, A1068, A1071, A1073, A1074, A1082, A1083, A1085, A1090, A1093, A1094, A1099, A1100, A1101, A1112, A1120, A1122, A1126, A1128, A1134, A1137, A1140, A1141, A1142, A1156, A1158, A1160, A1170, A1171, A1181, A1182, A1183, A1185, A1186, A1188, A1191, A1193, A1194, A1195, B9, B26, B29, B40, B50, B52, B56, B57, B72, B86, B96, B109, B130, B144, B165, B169, B171, B175, B181, B201, B213, B223, B226, B245, B262, B266, B279, B288, B304, B308, B310, B317, B334, B341, B346, B374, B380, B382, B391, B393, B411, B413, B416, B421, B422, B423, B424, B430, B443, B446, B456, B469, B472, B473, B484, B493, B515, B518, B523, B542, B544, B554, B572, B597, B600, B602, B608, B619, B641, B642, B656, B665, B678, B686, B701, B705, B709, B711, B712, B714, B716, B717, B718, B719, B720, B721, B725, B726, B727, B730, B734, B735, B736, B737, B738, B740, B741, B742, B744, B747, B750, B753, B755, B756, B757, B763, B764, B765, B766, B767, B769, B770, B771, B775, B776, B779, B780, B782, B784, B786, B787, B791, B793, B794, B795, B798, B800, B803, B807, B808, B811, B812, B819, B820, B822, B823, B825, B826, B827, B829, B831, B836, B839, B841, B842, B843, B845, B846, B851, B857, B861, B863, B866, B867, B869, B870, B873, B875, B878, B880, B882, B884, B885, B888, B889, B894, B896, B898, B899, B901, B908, B920, B922, B926, B931, B936, B937, B942, B950, B953, B964, B965, B969, B973, B975, B979, B982, B988, B990, B992, B993, B994, B995, B1000, B1004, B1007, B1014, B1017, B1025, B1026, B1031, B1032, B1033, B1034, B1037, B1040, B1041, B1042, B1058, B1064, B1065, B1066, B1067, B1071, B1076, B1077, B1078, B1084, B1089, B1091, B1094, B1096, B1104, B1107, B1111, B1116, B1117, B1120, B1123, B1127, B1128, B1131, B1133, B1136, B1138, B1139, B1142, B1145, B1146, B1149, B1151, B1155, B1159, B1163, B1165, B1168, B1174, B1177, B1179, B1186, B1190, B1195, B1197, B1198, B1199

## Acceptance criteria
- [ ] Each listed cell is labelled with its `bratio::dispatch` branch on `fisher.rs` `f_beta_arguments` (a
      temporary unit test or the docs/research/t3580 scripts, the method T-4029 used), and the label table goes in the
      Outcome.
- [ ] Each cell is either bit-exact, moved to the open task of its branch (named in both files), or listed in the
      Outcome with Excel, ours and the 50-digit math, and the reason its digits cannot be derived.
- [ ] Any kernel change that moves these cells is a fit only if it meets the R4 guards of reviewer-excel-fidelity
      check 7 (decision 0188): hold-out counts stated before scoring, a fresh sweep, Gate A envelope result, a
      `FITTED` tag and range in `quality/fitted.json`, the reproducibility floor, and no keep-matched branch. The
      fresh sweep needs a decision 0267 exception first (probes-numeric is frozen).
- [ ] The fixed cells' lines disappear from eval/baseline/probes-numeric.txt with no new ones, except family
      regressions allowed under decision 0267 item 6 and listed in the PR body.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The sweep's visibly-off cells (T-3592, T-3323) and the branch tasks named above.
- New probes-numeric cases (decision 0267 item 5).
