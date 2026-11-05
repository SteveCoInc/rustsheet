---
id: T-4365
title: "BINOM.DIST and NEGBINOM.DIST pmf last digits after T-2146's Loader switch: the 9,577 within-15-digit cells no task names (t2513-r3-binom-pmf-a, t2145-binomial-sweep, t2146 fresh residue, gen-m9; Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4352]
defer: phase-1b
---

## Goal
Every BINOM.DIST/BINOMDIST, NEGBINOM.DIST/NEGBINOMDIST pmf cell listed below (and the BETA.DIST density column of
t2513-r3-binom-pmf-a, which shares the kernel) matches Excel to the bit once T-4352 has identified Excel's stored Loader
chain, or the Outcome records, per case, the bit-exact count and why the rest cannot be derived.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main c5fdf6bd3, excel-graph). `make unowned-off ALL=1` leaves only the
  10 cells that open PR #3377 assigns (T-4352, T-4362), and `make exit-owners` is clean for M7 to M10. An ownership
  check over every failing cell of report.json (the `scripts/exit_owners.py` `owned` rule, open tasks of origin/main
  plus PR #3377's T-4352 and T-4362) leaves 9,582 cells with no owner. These 9,577 are BINOM-family pmf cells. Of the
  other 5, two are POISSON.DIST t4291 cells, added to T-4337 on this branch. The other three are among T-4352's 15
  decision-0293 regressed cells and stay T-4352's: it writes their gen-m9 hashes without the `gen-m9/` prefix, and
  those stems are shared with probes copies, so the rule does not credit them.
- Why no one owns them: T-2146 (done, c5fdf6bd3) owned these cases whole and moved both pmfs onto `loader.rs`
  `excel_dbinom_exponent` everywhere (decision 0293). Its Outcome routed only the visibly-off cells (to T-4352). T-3392
  (deferred) owns the Loader density probes and 19 named m9-legacy-binomdist and t1048-negbinomdist cells, not these.
- None is visibly off. All are within half a unit in the 15th digit, or are BINOM-family residue within the 13th digit
  (decision 0267 `residue13`, for instance gen-m9/c4635963fc926284 Calc!A148 `NEGBINOMDIST(6,9,1E-16)`, Excel
  3.0029999999999573E-141, ours 3.003000000000049E-141, 157 ulp). So decisions 0114 item 3 and 0188 R5 put the task under
  `defer: phase-1b`.
- Per case (failing cells no task names, median / max ulp from Excel, `make eval-show CASE=...`):
  - probes-numeric/t2513-r3-binom-pmf-a: 7,991 (all its failing cells; 3,979 BINOM.DIST pmf and 4,012 BETA.DIST
    densities at the matching shapes), 2 / 232. This task owns the whole case.
  - probes-numeric/t2145-binomial-sweep: 1,386 (all its failing cells, BINOM.DIST pmf), 2 / 396. This task owns the
    whole case.
  - probes-numeric/t2146-binom-negbinom-pmf-fresh: 134 of 265 failing (T-4352 owns the 131 visibly-off ones), 2 / 27.
  - probes-numeric/m9-legacy-binomdist: 5; probes-numeric/t1048-negbinomdist: 4; gen-m9: 57 cells in 47 cases.
- Examples (Excel vs ours):
  - t2513-r3-binom-pmf-a Calc!A1 `=BINOM.DIST(3,8,(104542602*2^-27+49421331*2^-53),FALSE)`: 0.013981433220031302 vs
    0.013981433220031293. The same row's B column is `=BETA.DIST(x,4,6,FALSE)`: 0.12583289898028172 vs
    0.12583289898028166.
  - t2145-binomial-sweep Calc!A1011 `=BINOM.DIST(5,9,0.7,FALSE)`: 0.17153224200000006 vs 0.17153224200000003.
  - gen-m9/ede0323c389fc703 Calc!A363 `=NEGBINOMDIST(4,ODD(1+2^-52),0.5)`: 0.11718750000000001 vs 0.1171875. Excel is
    one ulp above an exactly representable value. A closed form will not give that; the kernel will.
  - gen-m9/a63d1c7d6f96c56d Calc!A259 `=NEGBINOM.DIST(10,9,0.025,2)` is the one cdf among these (flag 2 is TRUE):
    2.8073841122879604E-10 vs 2.807384112287958E-10. It belongs to the bratio cdf path, not the pmf kernel. Re-measure
    it with the rest, and route it to the cdf owner of the day if it still fails.
- Rules of record: docs/excel-semantics/statistics.md "BINOM.DIST and NEGBINOM.DIST pmfs take Loader's kernel
  everywhere" and "Beta density for min(a,b) > 1 is Loader's saddle point". No new quirk; nothing to add there until a
  chain is identified.
- Split: T-4352 keeps its own named cells, including its 15 decision-0293 regressed cells, and the visibly-off
  t2146 fresh cells. T-3392 keeps its named Loader-probe cells. Depends on T-4352 because its chain is the fit; re-run
  the counts after it lands, before any further port.

Cells owned (beside the whole cases above):
- probes-numeric/t2146-binom-negbinom-pmf-fresh.case.json (134): Calc!A1:A4, A7:A8, A11:A12, A14, A23, A25, A28,
  A31, A37, A39:A40, A43:A44, A49, A53, A57:A58, A66, A75, A78, A80, A84:A85, A89:A92, A95, A97:A98, A100:A103,
  A105:A107, A116, A137, A140, A145, A147, A153, A164, A167:A168, A170:A172, A176, A179, A181:A185, A187:A193,
  A198, A201, A204:A205, A208, A213, A216:A217, A219, A222:A223, A225, A235, A238, A242, A244, A246, A248:A249,
  A252, A259, A261:A263, A266, A271, A277, A280:A282, A284, A286:A287, A289:A290, A293, A296, A300, A302:A303,
  A305, A307, A313, A316, A365:A372, A374:A375, A377, A379, A383, A387:A392, A394, A399:A400
- probes-numeric/m9-legacy-binomdist.case.json Calc!A7, A23, A24, A35, A36
- probes-numeric/t1048-negbinomdist.case.json Calc!A4, A5, B4, B5
- gen-m9/097af1ca33638670 Calc!A177
- gen-m9/09b47629f54b9817 Calc!A322
- gen-m9/0bbab6206b5c9a07 Calc!A25, A87
- gen-m9/11eb3ab4f4f8104c Calc!A312
- gen-m9/139684e68fb2fc3b Calc!A228, A380
- gen-m9/17c110a53924beb8 Calc!A11
- gen-m9/1b31092a0d79e5f1 Calc!A197
- gen-m9/1db18890916ee643 Calc!A86
- gen-m9/2a18b2f5a4dc43ac Calc!A174
- gen-m9/38e4fc309fe199ea Calc!A325
- gen-m9/43c6227e0ff368c7 Calc!A203
- gen-m9/44a8841352dc48e9 Calc!A325, A363
- gen-m9/46ac3ef5e61583ca Calc!A305
- gen-m9/583ca2baea0bb30a Calc!A30
- gen-m9/58e3253bbd3a7a82 Calc!A167
- gen-m9/5919660b7bb4d195 Calc!A276
- gen-m9/5cebac1f02bf30f2 Calc!A241
- gen-m9/607e85cfee83a689 Calc!A22, A270
- gen-m9/623663cacea53bbc Calc!A199
- gen-m9/775d053de498b8c7 Calc!A204, A390
- gen-m9/787ba8d801105a60 Calc!A71
- gen-m9/7980ffb6095a340d Calc!A257
- gen-m9/7f7ca726c3f68554 Calc!A124
- gen-m9/91e167cd8d7eb652 Calc!A183
- gen-m9/941c409532603d32 Calc!A25, A273
- gen-m9/9735c5ab72fa5ecf Calc!A368
- gen-m9/9cca17438a7e6f8a Calc!A323
- gen-m9/9e2225ead8a6a07b Calc!A275
- gen-m9/a013a45a8fbcab4f Calc!A145
- gen-m9/a5342a7e24fd803e Calc!A71, A109
- gen-m9/a63d1c7d6f96c56d Calc!A259
- gen-m9/a790a4d4a6a82756 Calc!A366
- gen-m9/afc00c8b70e78baf Calc!A353
- gen-m9/b6593d7dd13f2dac Calc!A37
- gen-m9/c18dbf21f11093f3 Calc!A45
- gen-m9/c2684066d8df263e Calc!A273
- gen-m9/c4635963fc926284 Calc!A148
- gen-m9/c468f5046aadfbbc Calc!A397
- gen-m9/c7664e494e418395 Calc!A110, A262, A300
- gen-m9/ce312817dc9c7f0b Calc!A143
- gen-m9/d208bbafc3bc1997 Calc!A307
- gen-m9/d8266bc15533702a Calc!A287
- gen-m9/d8f44d556674c2fd Calc!A167, A395
- gen-m9/d910f000b60e2eac Calc!A15
- gen-m9/ede0323c389fc703 Calc!A287, A363
- gen-m9/f82473275380882e Calc!A392

## Acceptance criteria
- [ ] After T-4352 lands, the counts above are re-measured. Each owned cell is bit-exact, or is counted in the Outcome
      per case with the reason Excel's last digits cannot be derived. The fixed cells' known-failure lines disappear
      from eval/baseline/gen-m9.txt (probes-numeric has no baseline file, so its counts go in the Outcome), with no
      new lines in any suite.
- [ ] Any change beyond porting T-4352's identified chain is a fit, and meets the decision 0188 R4 guards of
      reviewer-excel-fidelity check 7:
  - hold-out counts stated before scoring (t2513-r3-binom-pmf-hold and t2513-verify-r3-binom-pmf-h stay held out);
  - a fresh sweep drawn after the freeze and scored. Needs an oracle sweep: T-2146's fresh case is spent, so a fit
    first files a research-lane sweep task (`make new-task`, probes suite: probes-numeric is frozen by decision 0267
    item 5) and depends on it;
  - the Gate A envelope result for BINOM.DIST and NEGBINOM.DIST (the `make eval` accuracy line);
  - a `FITTED` tag and its evidenced range in `quality/fitted.json` for any fitted constant or cut;
  - the reproducibility floor stated (the 4-ulp web-desktop floor, decision 0188 R4.4);
  - no branch that only keeps already-matching cells matched.
- [ ] statistics.md's T-2146 paragraph gets the new bit-exact counts.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Visibly-off pmf cells (T-4352), BINOM.DIST.RANGE at a tiny p (T-4362), and the cdfs (bratio port follow-ups,
  T-3959, T-3961).
- The Loader density probes T-3392 names.
