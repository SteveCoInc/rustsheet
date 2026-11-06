# Coercion and argument handling

## AND / OR / XOR evaluate every argument; the first error wins
Rule: AND/OR do not short-circuit: `=OR(TRUE,(2<COUNTIFS(…shape mismatch…)))` → `#VALUE!`.
Evidence: gen-m2/78bc88199fc96650 Calc!A217; OR: 1b6ceed7489e0800 A143, 893302d6c5b99e2e A170, dd99be1f2454159f A3; AND: 75840b9c71a50596 A56, 96a0952412cefa6c A126. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Which error: the first one, taking arguments left to right and each range or array row by row, for AND, OR and XOR
alike, whatever value decides the result: `OR(TRUE,NA(),1/0)` → `#N/A`, `OR(TRUE,1/0,NA())` → `#DIV/0!`,
`AND(FALSE,1/0,NA())` → `#DIV/0!`; with D1:F2 = TRUE, `#DIV/0!`, FALSE / `#N/A`, TRUE, `#NUM!`, `OR(D1:F2)`,
`AND(D1:F2)` and `XOR(D1:F2)` are `#DIV/0!` (row-major first) and `OR(TRANSPOSE(D1:F2))` is `#N/A`;
`OR({TRUE,#DIV/0!,#N/A})` → `#DIV/0!`; `OR(TRUE,B1:B2)` with an error in B1:B2 is that error; text before an error
in a range neither wins nor makes `#VALUE!` (`OR(H1:H2)` with H1 = "x", H2 = `#N/A` → `#N/A`; a text-only range and
direct text arguments are not covered here); an intersection `OR(D1:F2 E1:F2)` reads its own cells.
Evidence: probes/and-or-xor-error-order (30 cells; oracle: excel-graph, 2026-10-08, T-0229).
Status: VERIFIED
Implemented in: `logical_nary` (functions/logical/and_or_xor_not.rs; unit test `first_error_wins_row_by_row` in test/test_fn_or_xor.rs).
Tasks: T-0215, T-0229

## MIN / MAX coerce value arguments like SUM; ranges and arrays give only their numbers
Rule: a value argument (literal or function result, e.g. IFERROR or CONCATENATE) is coerced like
SUM's: non-numeric text is `#VALUE!` (`=MIN("a")`, `=MAX(CONCATENATE("x"))`), numeric text counts
(`=MIN("2")` → 2, `=MAX("1e2",5)` → 100), TRUE/FALSE count as 1/0 (`=MIN(TRUE,5)` → 1), and an empty
argument counts as 0 (`=MIN(5,)` → 0, `=MAX(-5,)` → 0). A reference, also one returned by IF/INDEX
(references.md), and an array contribute only their numbers: `=MIN(A1)` with A1 = "a" → 0,
`=MAX(IF(TRUE,A1),A2)` → A2, `=MIN({TRUE,5})` → 5, `=MAX({"9",-1})` → -1. IFERROR returns a value,
so `=MIN(IFERROR(A1,1))` with A1 = "a" is `#VALUE!` and an IFERROR of a blank cell is 0.
Evidence: probes/min-max-value-args (C1–C16); gen-m2/f7e6d351c8538ef5 Calc!A14,
dd99be1f2454159f A73/A264, 2cfe279fe77a000d A138, 04cfd82f54dbc2b2 A182.
Oracle: excel-graph, 2026-10-05 (probe), 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Implemented in: functions/math_and_trigonometry/mathematical.rs (`fold_min_max`)
Tasks: T-0217

## AVERAGE, MEDIAN, GEOMEAN, HARMEAN, SKEW, KURT, STDEV/VAR, COUNT value arguments
Rule: these coerce like MIN/MAX above: booleans inside an array are ignored (`=AVERAGE({TRUE,5})` → 5,
`=MEDIAN({TRUE,1,3})` → 2, `=GEOMEAN({TRUE,4})` → 4, `=STDEV({TRUE,1,3})` → 1.414...), a direct boolean counts
as 1/0 (`=AVERAGE(5,TRUE)` → 3), text inside an array is ignored, and an empty argument counts as 0
(`=AVERAGE(5,)` → 2.5, `=MEDIAN(1,)` → 0.5, `=GEOMEAN(4,)` → `#NUM!`, `=STDEV(1,3,)` → 1.5275..., VAR/VARP/STDEVP likewise,
`=COUNT(5,)` → 2, `=COUNT({TRUE,5})` → 1, HARMEAN/SKEW/SKEW.P/KURT likewise (`=HARMEAN({TRUE,4})` → 4, `=HARMEAN(4,)` → `#NUM!`, `=SKEW(1,2,4,)` → 0.7528...), `=AVERAGE(A1:A2,)` with A1 = TRUE, A2 = 5 → 2.5).
Evidence: probes/average-value-args F1–F32 (oracle: excel-graph, 2026-10-05)
Status: VERIFIED
Implemented in: `for_each_value`, `fn_count` (also HARMEAN/SKEW/SKEW.P/KURT via `for_each_value`) in functions/statistical/count_and_average.rs; `fn_stdev_s/p`, `fn_var_s/p` (standard_dev.rs, variance.rs)
Tasks: T-0236

## LEFT / RIGHT / MID numeric arguments
Rule: booleans coerce (TRUE = 1: `MID(…,TRUE,5)`); a start or length around 1E+15 is `#VALUE!`
(`MID("ü",1000000000000000.0,4)`); in LEFT a length a hair below 1 (`TRUE - 1E-15`) counts as 1
(seen for LEFT only, and the cell cannot tell whether the subtraction or LEFT rounds: probe in T-0219).
Numeric text coerces too, literal or from a cell, and is then truncated like a number:
`LEFT("a string","6")` is "a stri", `LEFT(E12,E17)` with E17 = "2.99999" (text) is "no",
`LEFT("a string",TRUE)` is "a"; RIGHT the same ("string", "ce", "g").
Evidence: gen-m2/ef5c55a81783b7d5 A40, d0c29dd5c530e28c A269, 78bc88199fc96650 A345,
b4392a6514a36ade A14 (Calc sheet). Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!E824, H824, K824 (LEFT),
E1168, H1168, K1168 (RIGHT).
Status: VERIFIED
Implemented in: `Model::array_node_to_number` (cast.rs), used by `fn_left`/`fn_right`/`fn_mid` (T-0257).
Tasks: T-0219 (huge and near-integer counts), T-0257 (numeric text and booleans)

## Text comparison uses Excel's collation, not code-point order
Rule: `<`, `>`, `>=` between texts follow Excel's collation: "x" >= "ü" is TRUE, "1.5" > "?b" is TRUE
(code-point order says otherwise for both: "x" < "ü" and "1" < "?").
Evidence: gen-m2/2cfe279fe77a000d Calc!A318, 96a0952412cefa6c Calc!A284. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED (two cells here; the full order is below, T-1469 probes/text-collation-order)
Hold-out agreement (T-3465, probes/t3465-collation-holdout, excel-graph 2026-11-03): on 600 random text pairs
(seed 3465, texts of 1 to 6 characters from 1 to 3 of 23 categories, every third pair one edit apart; generator
tools/src/rustsheet_tools/gen/collation_holdout.py, categories per row in t3465-collation-holdout.draws.tsv),
`cmp_text_excel` orders **569 of 600 pairs (94.8%)** as Excel does (1738 of 1800 `<`, `>` and `=` cells), before
any fix. Excel is antisymmetric on every pair (exactly one of A<B, B<A, A=B holds). The 31 misses, by cause:
19 pairs put a Thai or Lao digit below the letters (refuting the old UNVERIFIED choice "Thai and other digit
blocks keep code-point order"; fixed by T-3895, "Thai, Lao and the other script digits (T-3895)" below), 9 pairs ignore an inserted soft hyphen U+00AD (`"¡" = "¡"&UNICHAR(173)`; T-3662),
1 pair ignores U+034F after `»` (C237, E237; T-3305), 1 pair ranks the middle dot `·` below `Æ` (C110, D110;
T-3896) and 1 pair puts U+20000 below a leading `々` followed by marks and kana (C313, D313; a lone mark is above every
character, fixed by T-4258). No other
rule failed: kana with skipped hyphens, expansions next to marks, full-width against accented letters and the
T-3276 scripts mixed with Latin all agree.
CHAR(160) and spaces (T-0244, VERIFIED, probes/text-order-nbsp-space, 333 cells, excel-graph
2026-10-09): texts compare character by character, with the space first, CHAR(160) right after it,
then digits and letters: `" 1" < CHAR(160)&"1" < "0" < "1" < "1"&CHAR(160)`, `" " < CHAR(160)`,
`"a b" < "a"&CHAR(160)&"b" < "ab"`, `CHAR(160)&CHAR(160)&"1" < CHAR(160)&"0"`; CHAR(160) is never
equal to a space or ignored (`CHAR(160)&"1"="1"` is FALSE). `>` and COUNTIF's `">"` agree on every
pair whose operand is text (a criterion operand like `" 0"` is numeric, criteria.md). In passing:
a hyphen is skipped, `"-1" > "0"` is TRUE (Data!A17 vs C3; implemented by T-0220).
Implemented in: `functions::util::collation::cmp_text_excel` (T-0220: the three passes below), used by
`compare_values` (operators including `=`, SWITCH, SORT, XMATCH, XLOOKUP) and the ordered text criteria (`<`, `<=`, `>`, `>=`) in
functions/criteria.rs. The `=`/`<>` text criteria use it too (T-1594). MATCH(...,0) and VLOOKUP(...,FALSE) do not use the collation
(T-1590, below). Inferred, not probed: SORT orders texts the same way; control characters sort below the space (see T-1590 for CHAR(9), 10, 13).
Full order (T-1469, VERIFIED, probes/text-collation-order, 146 strings ranked by
`SUMPRODUCT(--(range<x))`, excel-graph 2026-10-13). `=`, `<` and `>` ignore case completely (`"a"<"A"` and `"a">"A"` are both FALSE) but not accents; `<`/`>` order, lowest first:
1. The empty string, then a lone `'`, `''`, `-`, `--`, then space, `  `, then the punctuation
   ``! " # $ % & ( ) * , . / : ; ? @ [ \ ] ^ _ ` { | } ~``, then `+ < = >`, then digits `0 1 2 ... 9`, then letters.
2. Hyphen and apostrophe are ignored at the first pass and only break ties afterwards (Excel's "word sort"):
   `"ab" < "a'b" < "a-b"`, `"coop" < "co-op"` (but `"co op" < "coop"`: a space is not ignored), `"1" < "'1" < "-1" < "2"`,
   `"a" < "a'" < "a-" < "'a" < "-a" < "aa"`. `"-1" > "0"` is TRUE.
3. Case is ignored at every level (`"a" = "A"`, `"ab" = "AB"`); accents are a later tiebreak, not a different letter:
   `"a" < "á" < "à"="À" < "â" < "ä" < "å" < "aa"`; `"é"="É"` sits after `"E"` and before `"F"`; `"résumé" > "resume"`.
4. Expansions and multi-letter folds: `"æ"="ae"` and `"ß"="ss"` are equal (`=` TRUE); `"ø" > "o"` but `"ø"` sorts after `"ö"` and before `"P"`
   (observed position only; whether it is a separate letter or a heavier accent of `o` is not probed); `"ÿ"` sorts after `"Y"` and before `"Z"`.
Ranks for every string are in the oracle sidecar (Calc!A, ties in Calc!B).
Fitted key (T-0220; all 486 probe cells and the gen-m2 cells above pass): compare in three passes,
case folded throughout. (1) Letter weights: control characters, space, CHAR(160), the punctuation
in the order above, digits, letters; an accented Latin-1 letter weighs as its base letter, `æ` as
`a e`, `ß` as `s s`; hyphen and apostrophe weigh nothing. (2) Accents, letter by letter: none <
acute < grave < circumflex < diaeresis < tilde < ring, then stroke (`ø`); T-3114 below extends this
order. (3) Hyphens and apostrophes as (position, weight), a later position first, apostrophe before
hyphen. `þ`, `ð`, `œ` and the Latin-1 symbols were placed later (T-1590, T-1595, T-1662, T-1669,
below), and the Latin Extended-A, Greek and Cyrillic letters by T-3114 (next section); full-width and
half-width forms and kana are probed by T-3275 (below; implemented by T-3321). Hebrew, Arabic,
Devanagari, Thai, Lao, Hangul, CJK ideographs, symbols above `€` and astral characters are probed by
T-3276 (below; implemented by T-3324). Bengali, Gurmukhi, Tamil, Telugu, Georgian, Armenian,
Ethiopic, Mongolian, Tibetan, Khmer, Myanmar, more symbol blocks, other-script digits, emoji and
planes 1 to 3 are probed by T-3559 (below, "Scripts and symbol blocks beyond T-3276"); scripts
neither probe covers stay UNVERIFIED, and `script_order` in
crates/rustsheet-engine/src/functions/util/collation/scripts.rs lists the groups they fall in.

### Letters beyond Latin-1 and combining marks (T-3114)
Evidence: probes/t3114-collation-beyond-latin1 (excel-graph, 2026-10-29), 282 strings ranked against
each other (Calc!A = count less, Calc!B = count equal) plus pairwise `<`/`=`/`>` (Calc!D:F), exact
MATCH (Calc!H) and COUNTIF `=`, `<`, `<>` (Calc!I:K); all 986 cells pass. Status: VERIFIED for these:
- Every Latin Extended-A letter probed (`ā ă ą ć č ď ē ė ę ě ğ ģ ī į ķ ĺ ľ ń ņ ň ő ŕ ř ś ş š ţ ť ū ů ű
  ų ź ż ž`) is its **base letter plus an accent**, not a letter of its own: `"ca" < "ča" < "cb"`,
  `"ea" < "ěa" < "eb"`, `"oa" < "őa" < "ob"`, `"ša" < "sb"`, `"ža" < "zb"` (Calc!A123:A139).
  Upper and lower case are equal (`"Č" = "č"`, Calc!E1).
- Accents on one base letter, in order: `a á ă ā ą`, `c ć č`, `e é ė ě ē ę`, `g ğ ģ`, `i ī į`, `l ĺ ľ`,
  `n ń ň ņ`, `o ö ő`, `r ŕ ř`, `s ś š ş`, `t ť ţ`, `u ū ů ų ű`, `z ź ż ž`. One order fits them all:
  acute < breve < dot above < caron < macron < ring < ogonek < double acute < cedilla (with diaeresis
  before ring and double acute). T-3154 ranked them against the Latin-1 accents on one letter
  and replaced this order by the Windows table's weights (next section).
- A base letter plus a combining mark equals the precomposed letter: `"a"&UNICHAR(769) = "á"`,
  `"c"&UNICHAR(780) = "č"` (also `"C"&UNICHAR(780)`), `"e"&UNICHAR(769) = "é"`,
  `"c"&UNICHAR(780)&"a" = "ča"`, and `o`+U+030B, `s`+U+030C, `z`+U+0307 tie with `ő`, `š`, `ż` (Calc!B
  counts 3 or 4, D7:F10, D25:F25). Only four combining marks are probed as such: U+0301, U+0307,
  U+030B and U+030C (UNICHAR 769, 775, 779, 780); T-3154 probes the others and stacked marks
  (next section).
- Greek: `α < ά < β`, … `ι < ί < ϊ < ΐ < κ`, … `ω < ώ` (tonos is the acute accent, dialytika the
  diaeresis); `Σ = σ = ς` (Calc!B168, B192, B216, E20), and exact MATCH of `"ς"` finds `σ` (Calc!H10). Base
  letters keep code-point order.
- Cyrillic: `а < б < … < е < ё < ж < … < и < й < к < … < я`, each upper case equal to its lower case.
  `ё` and `й` are accented `е` and `и`, not letters of their own: `"ёа" < "еб"`, `"йа" < "иб"`,
  `"ё" = "е"&UNICHAR(776)`, `"й" = "и"&UNICHAR(774)` (probes/t3154-collation-accents Calc!D8:F11).
- COUNTIF `=`, `<` and `<>` agree with the ranking above (Calc!I1:K10; `I8` = 4 and `I9` = 2 count the
  combining-mark spellings with the precomposed letters). Exact MATCH folds case and `ς` (H1:H7, H10)
  but does **not** equate a base letter plus a combining mark with the precomposed letter:
  `MATCH("c"&UNICHAR(780),r,0)` = 144 and `MATCH("a"&UNICHAR(769),r,0)` = 143 find the decomposed
  spellings, not `č` (row 57) or `á` (row 140) (H8, H9), as T-1590 found for `æ`/`ae`. XMATCH,
  XLOOKUP and COUNTIF do equate them, both ways, and VLOOKUP(...,FALSE) does not, like MATCH:
  `XMATCH("a"&UNICHAR(780),{"ǎ",…},0)` = 1, `XMATCH("ơ",{"o"&UNICHAR(795),…},0)` = 1,
  `VLOOKUP("s"&UNICHAR(806),…,FALSE)` `#N/A` (probes/t3154-collation-accents Calc!J1:J46). The engine
  agrees: MATCH and VLOOKUP exact match fold through `folded_units`, which does not decompose;
  XMATCH and XLOOKUP compare `collation_key`s (`CollatedKey`).
- Inferred, UNVERIFIED: the engine gives every other lower- or upper-case letter of Latin Extended-A
  and -B, Greek, Cyrillic and Latin Extended Additional that has a canonical decomposition
  (`ĉ ĝ ĥ ĵ ŝ ŵ ŷ ĩ ĭ ŭ ŏ ō ĕ ġ ċ ļ ŗ`, `ǐ ǒ ǔ ǖ ȁ ȩ`, `ϋ ΰ`, `ѐ ѓ ї ќ ѝ ў`, `ấ ặ ẽ ỹ`) the same
  base-plus-marks rule (its weight in the Windows table agrees for every one of them).
Implemented (T-3114): `collation.rs` case-folds, then decomposes each letter through its canonical
decomposition (`DECOMPOSED`) and gives a combining mark's accent to the letter before it.
`collation::fold_case` (the exact MATCH, wildcard and D-function prefix fold, one character to one; T-3133)
folds `ς` to `σ`, in wildcards and D-function criteria too: over `σx`, `ςx`, `Σ`, `COUNTIF(r,"ς*")` = 3,
`COUNTIF(r,"ς?")` = 2, `MATCH("ς*",r,0)` = 1, `XMATCH("ς",r,2)` = 3, `VLOOKUP("ς?",r,1,FALSE)` = `σx`,
`XLOOKUP("ς?",r,r,,2)` = `σx`, and a D-function criterion `ς` counts 3, `=ς` 1
(probes/t3154-collation-accents Calc!H1:H19; also probes/t3133-lookup-criteria-case-fold, criteria.md
"Wildcard escapes, case and trailing tilde"). VERIFIED. The D-function prefix fold of `İ` stays
inferred, UNVERIFIED (T-3180).
- `İ` (U+0130) collates as `i` + U+0307, its lower-cased canonical decomposition: `XMATCH("i"&UNICHAR(775),{"İ"})`
  is 1, while `XMATCH("I",{"İ"})` and `XMATCH("İ",{"i"})` are #N/A (probes/t3133-lookup-criteria-case-fold Calc!J1,
  J3, J4, J33, excel-graph, 2026-10-29, T-3133). Before T-3133 the engine folded it to a plain `i`. Its rank
  among `i` and the other accented `i`s, and `=`/COUNTIF equality with it, are inferred from XMATCH, UNVERIFIED
  (T-3180).
  In the same cells XMATCH equates `ẞ` and `ß` with `ss` (the `ß` expansion) and `ǅ ǆ Ǆ`, the Kelvin, Ångström
  and Ohm signs with their letters, while `ſ`/`s` and `µ`/`μ` stay apart (Calc!J1:J35).

### Accent weights add up and wrap: Windows' diacritic weights (T-3154)
Evidence: probes/t3154-collation-accents (146 strings ranked, pairs, lookups; 595 cells),
probes/t3154-collation-accent-weights (119 spellings of `a` with 1 to 100 combining marks ranked),
probes/t3154-collation-accent-pairs (28 spellings, every pair's `<`, `=`, `>`), excel-graph 2026-10-29;
Microsoft's Windows sorting weight table (`Windows Server 2022 Sorting Weight Table.txt`, download
10921), whose DW column every finding below matches. Status: VERIFIED.
- An accent is a **diacritic weight**: 2 for a plain letter, plus 12 acute, 13 grave, 14 dot above,
  16 circumflex, 17 diaeresis, 18 caron, 19 breve, 21 macron, 23 tilde, 24 ring, 25 ogonek,
  26 cedilla, 27 double acute, 78 horn, 86 dot below, 89 comma below, 141 perispomeni, … (the
  table's script-member-1 rows). Probed as marks: U+0300-U+0304, U+0306-U+030C, U+030F, U+0311,
  U+0314, U+0315, U+031B, U+0323-U+0328, U+032C, U+032D, U+0330, U+0331, U+0334, U+0342, U+0345
  (accent-weights Data!A10:A39); the other weights of U+0300-U+0362 come from the Windows table
  alone, UNVERIFIED. So on one letter `a á à â ä ǎ ă ā ã å ą ạ`, `e é è ė ê ë ě ē ę`,
  `o ó ò ô ö ō õ ő ơ`, `u ú ù û ü ū ů ų ű ư`, `s š ş ș`, `t ť ţ ț`, alone and as the first of two
  letters (Calc!A1:A145). Letters with a weight of their own keep theirs: `ª º` 3, `ı` 4, `đ` 30,
  `ł` 31, `ø` 33, `ð` 104 (all consistent with T-1590).
- Combining marks **add** their weights to the letter before them, in any order:
  `"a"&UNICHAR(769)&UNICHAR(769)` = `"å"` (12 + 12 = 24), `"a"&UNICHAR(769)&UNICHAR(768)` = `"ą"`
  (25), `"a"&UNICHAR(769)&UNICHAR(780)` = `"a"&UNICHAR(780)&UNICHAR(769)` = `"á"&UNICHAR(780)` =
  `"ǎ"&UNICHAR(769)`, `"à"&UNICHAR(803)` = `"a"&UNICHAR(803)&UNICHAR(768)`; precomposed letters
  of Latin Extended-B and Additional equal their decomposition (`ǎ`, `ș`, `ț`, `ơ`, `ư`, `ạ`) and
  `ș` (comma below) is not `ş` (cedilla).
- The sum is **one byte and wraps** past 255: two U+0342 (2 + 282 = 284, so 28) equal one cedilla
  (`"a"&UNICHAR(834)&UNICHAR(834)` = `"a"&UNICHAR(807)`), and three U+0323 (260, so 4) sort between
  `"a"` and `"á"`.
- Engine choices from the Windows table or by inference, UNVERIFIED (T-3203 probes them):
  U+0363-U+036F ("combining a" to "combining x") are the letters `a e i o u c d h m r t v x` with
  diacritic weight 3, not marks (the table gives them script member 14, a letter's alphabetic
  weight and DW 3); a mark after an expansion (`æ`, `ß`) adds its weight to the expansion's second
  letter (the table gives `ǣ`, `æ` plus macron, a row of its own); the superscript (variant) pass
  comes before the skipped-character pass.
- `ª`/`º` and `¹ ² ³` differ from `a`/`o`/digits also in a later, case-like pass (Windows case
  weight 14, the superscript bits; Excel ignores only the upper-case bit, and `ς`, case weight 10,
  still equals `σ`): `"ª"&UNICHAR(769)` (3 + 12 = 15) sorts just after `"à"` (15) (accent-weights
  Calc!A114, A10).
- **Not a total order.** A text sorts before any text it is a proper prefix of, even when the
  wrapped weight says otherwise: `"a"&UNICHAR(834)` < `"a"&UNICHAR(834)&UNICHAR(834)` although the
  latter equals `"a"&UNICHAR(807)`, which is below `"a"&UNICHAR(834)`; likewise `"a"&UNICHAR(803)` <
  three U+0323 < `"á"` < `"a"&UNICHAR(803)` (pairs Calc!S19, T18, W23, X22, AB28, AC27). The next
  bullet is the general rule; the prefix rule is its case where one side runs out. It also makes
  texts with equal wrapped sums unequal: `"a"=("a"&REPT(UNICHAR(770),16))` is FALSE and `<` TRUE
  (circumflexes 16 * 16 = 256, so the same sum as `"a"`).
  **Every caller uses it (T-3181).** Evidence: probes/t3181-collation-prefix-callers (35 cells) and
  probes/t3181-collation-prefix-arrays (13 cells), excel-graph 2026-10-30, Status: VERIFIED, on
  `"a"`, `"a"` plus 16 U+0302, U+0342 once and twice, U+0327, U+0323 once and three times and `"á"`:
  - the operators `= <> < <= > >=` (callers Calc!A1:A9);
  - COUNTIF and COUNTIFS criteria, `=` and the ordered ones (`COUNTIF(r,"a")` counts `"a"` but not
    `"a"` plus 16 U+0302; callers Calc!B1:B12);
  - approximate MATCH (1 and -1) and VLOOKUP TRUE (callers Calc!C3:C12, C14), and XMATCH next
    smaller/larger (arrays Calc!B5:B8);
  - exact XMATCH, XLOOKUP and SWITCH (`"a"` is not found in `"a"` plus 16 U+0302; arrays
    Calc!B3, B4, B11, B12); MATCH(...,0) misses both ways too (callers Calc!C1, C2), as it misses
    `"a"` against `"a"&UNICHAR(807)`'s equal `"a"&UNICHAR(834)&UNICHAR(834)` (C13): MATCH(...,0)
    is not the collation's equality (T-1590 below);
  - SORT and SORTBY: `SORT` of U+0342 once and twice gives once, twice whichever comes first
    (arrays Calc!A1, A5), descending twice, once (A6), and `"a"` before `"a"` plus 16 U+0302 (A3,
    A4). On the three-text cycle U+0342 once, twice, U+0327 SORT gives once, twice, U+0327 (A2),
    and on U+0323, three U+0323, `"á"` it gives U+0323, three U+0323, `"á"` (A8): Excel sorts with
    the pairwise comparison and its output on a cycle depends on its algorithm.
  - `UNIQUE` of two U+0342 and U+0327 (equal sums, nothing shared after the letter) is one row
    (arrays Calc!B2).
  - **SORT and SORTBY in detail (T-3211).** Evidence: probes/t3211-collation-sort-cycles, excel-graph 2026-10-31,
    Status: VERIFIED. SORT and SORTBY give the same order in all 88 probed rows (Calc!A = C, B = D). Descending is
    not the reverse of ascending: `SORT` of U+0342 once, twice, U+0327 is once, twice, U+0327 and with -1 twice,
    once, U+0327 (row 1). The order is a bottom-up natural merge sort over maximal non-descending runs only:
    the input is cut where an element compares less than the one before it (a strictly descending stretch is
    runs of one element, never reversed), and neighbouring runs are merged pairwise, round by round, a tie
    taking the left run's element; descending sorts the same way with the comparison flipped (T-3418). It
    gives every row of rows 1-44 and every row of probes/t3418-sort-cycle-permutations (excel-graph 2026-11-04:
    all 24 orders of the 4-cycles {A3, A4, A6, A7} and {A5, A6, A7, A8} and all 120 orders of {A4..A8},
    ascending and descending, 336 cells). Reversing strictly descending runs (T-3211's model) misses 28 of
    those 168 rows: on a cycle `a<b<c<d<a` with chords `c<a`, `d<b`, the input `a, c, b, d` is one strictly
    descending chain (c<a, b<c, d<b) but Excel takes it as four one-element runs, merges them into `c, a`
    and `d, b`, then into `c, d, a, b`. The probed comparison matrix (Calc!L1:T9) is the engine's on all 81
    pairs.
  - **UNIQUE groups by Excel's `=`** (T-3211, same case, Calc!G1:J15): `"ß"` and `"ss"`, `"á"` and
    `"a"&UNICHAR(769)` are one row (the first kept), as `=` finds them equal; `"Ａ"` and `"A"`, `"coop"` and
    `"co-op"` are two, as `=` finds them unequal; exactly_once drops both of an equal pair (`#CALC!` when none is
    left). On a cycle the grouping follows Excel's sort, not equality with any kept row: U+0327, U+0342, two
    U+0342 is three rows although U+0327 = two U+0342 (G4), while U+0342, two U+0342, U+0327 is two (G3).
  - **UNIQUE's rule: equal lines become one group as soon as its merge meets them** (T-4200,
    probes/t4200-unique-cycle-permutations, excel-graph, 2026-11-04: UNIQUE and exactly_once of every order of
    {A3,A4,A6,A7} and {A4..A8} and of {A3,A4,A5}, A1:B150, and two-column rows A151:B174). UNIQUE runs SORT's
    natural merge (above), except that two groups `=` finds equal become one group when they meet: as neighbours in
    a run, as the heads of two runs being merged, or as the last group out of a merge and the next one; a group is
    compared through the line that started it. Each group keeps its first line, in input order, and exactly_once
    keeps the one-line groups. Grouping the neighbours of SORT's order fits the 4- and 5-text sets but not
    {A3,A4,A5}, the cycle A5 < A3 < A4 = A5: SORT puts `{A3;A5;A4}` and `{A5;A4;A3}` in the order A5, A3, A4
    (t3211 Calc!A2, A6), yet UNIQUE groups A5 with A4 in both (rows 146, 150), since they are equal neighbours of a
    run before any merge. `{A4;A3;A5}` and `{A5;A3;A4}` keep three rows (rows 147, 149). Lines compare column by
    column, the first unequal column deciding: rows ("x",A5), ("y",A3), ("x",A4) keep two, ("x",A5) standing for
    ("x",A4), though the one column A5, A3, A4 keeps three (row 170 against row 149).
    Which line represents a merged group (its first, its smallest or its last) is not separated by
    the probe.
    Confirmed out of sample (T-4237, probes/t4237-unique-cycle-fresh, excel-graph, 2026-11-04; case and the
    engine's values committed before the fill, docs/research/t4237/): all 240 cells as predicted, on 60 seeded orders
    of the six texts A3..A8 (the {A3,A4,A5} cycle with A6, A7 and A8), 36 two-column rows where both columns hold an
    order of the cycle, and 24 rows of a number, boolean or mixed-type first column (`{1;"1";1}`,
    `{TRUE;"TRUE";TRUE}`, `{0;FALSE;""}` among them) beside an order of the cycle.
  Ours (T-3181): the operators and the criteria compare texts with `cmp_text_pairwise`, which
  applies the rule (the criteria index still hashes the `CollationKey` of the wrapped sums,
  which every pair the rule finds equal shares, and tests each candidate with the rule); so do
  all cells of the four t3154/t3201 accent cases, which compare with `<` and `=`. Since T-3210 the
  lookups do too: `compare_values_pairwise` orders texts for MATCH/VLOOKUP/HLOOKUP/XMATCH/XLOOKUP
  (approximate, descending and linear searches) and SWITCH, and the hashed exact XMATCH/XLOOKUP
  index keeps the `CollationKey` but tests the first (last) cell with the key and walks when that
  cell is not equal. Since T-3211 SORT and SORTBY compare texts with `cmp_text_pairwise` too and sort
  with `sort_indices_pairwise`, a stable natural merge sort that cannot panic on a non-total order (`slice::sort_by`
  may): every t3181 array cell passes, and since T-3418 every t3211 and t3418 sort cell. UNIQUE keys a
  text by its `CollationKey` and keeps a line unless `cmp_text_pairwise` finds it equal to
  a kept line of its bucket, each comparison charged to the formula's work budget (texts can share a key and be
  unequal), when every text of its data is ASCII (ASCII texts compare transitively); otherwise, since T-4200, it
  groups with `coalesce_equal_lines` (functions/util/pairwise_order.rs), the merge above, comparing texts with
  `cmp_text_pairwise` and other cells by their keys, each comparison charged to the budget: every t3211 and t4200
  UNIQUE cell passes. GROUPBY and PIVOTBY still group by the buckets (T-4088) and sort with the total `cmp_text`.
- **Shared leading units are skipped before the sum (T-3201).** Evidence:
  probes/t3201-collation-long-mark-runs (1,000 cells: a ladder of 246 texts `"a"` plus up to three
  marks, one per weight 2, 5, 8, 11 and 14 to 255, self-ranked in order (Calc!A1:A246); `"a"` with
  1 to 40 of U+0301, U+030A, U+0323, U+0342 ranked against it (Data!B1:B160), mixes, `"e"` runs
  and `"a"`&run&`"b"`) and probes/t3201-collation-wrapped-run-pairs (37 texts whose sums wrap to
  2 to 22, every pair, 1,369 cells), excel-graph 2026-10-30, and probes/t3304-collation-multi-letter-accents
  (four groups of texts, every pair within a group, 1,653 cells, plus ranks and COUNTIF `"="&x`;
  excel-graph 2026-10-31). Status: VERIFIED, for texts with several accented letters, marks on
  either side of a letter boundary and equal-sum runs that share a prefix too. Compare a letter's accent as a sequence of units, the
  letter itself (weight 2, or a precomposed letter's whole weight, `á` 14) and then each mark (its
  weight above): first drop the leading units the two texts share with equal weights (a mark matches
  any mark of the same weight: U+0341 matches U+0301, U+0352 matches U+0323, but precomposed `á` is
  one unit and does not match `a` plus U+0301); if one side has no units left it is less; otherwise
  compare the sums of the units left, each mod 256. Consequences, all in Excel's cells:
  - With nothing shared but the letter, every run of the same sum mod 256 is equal whatever the
    marks and counts: 22 U+0301 = 11 U+030A = 12 U+0323 = 40 U+0342 = 88 U+030D (the letter's 2
    dropped, 264 each, so 8); 18 U+0323 = 60 U+0342 = 60 U+0300 = `"á"` (wrapped-run-pairs).
  - There is no separate rule for long runs: each run is a proper prefix of every longer run of
    the same mark, so it sorts below it, and it ranks against other texts by its wrapped sum.
    22 U+0301 (2 + 264, so 10) has 43 of the 119 texts below it in t3154-collation-accent-weights Calc!A107 because
    it is above the shorter acute runs (its prefixes), above the `"a"&UNICHAR(769)&UNICHAR(X)`
    texts (they share its first acute, and its 21 acutes left, 252, beat their X), and above the
    texts whose sum is below 10; the same count explains every Excel rank of
    accent-weights Calc!A105:A112 (20, 21, 22, 40 and 100 acutes, 10 and 11 rings, 10 dots below:
    108, 110, 43, 111, 105, 110, 6, 75).
  - The skipped letter weighs nothing: `"a"` plus 85 U+030D (2 + 255, so 1 as a total) is above
    all 246 ladder texts: `"a"` is its prefix, and against each other its marks sum to 255
    (long-mark-runs Calc!B179 = 246);
    `"a"` plus 16 U+0302 (marks 256, so 0) is below every ladder text but `"a"` (its prefix) and
    the 15 that start with U+0302, which it beats on the 15 circumflexes left (240) (Calc!B173 = 16).
  - Per letter: `"a"`, 22 U+0301, `"b"` beats `"a"&UNICHAR(833)&"b"` on its 21 acutes left
    (Calc!B242); `"e"` runs rank as `"a"` runs (Calc!B181:B220).
  - A model of this rule (and the weights above) reproduces all 3,391 cells of the four t3154 and
    t3201 accent cases (pairs, weights, long-mark-runs, wrapped-run-pairs).
  - **Letter by letter, the first difference decides** (multi-letter-accents sheet G1, Data!A): a
    letter whose units are left equal after the skip (equal sums) passes the decision to the next
    letter, and the first letter that differs decides, whatever the later letters do. So
    `"áe" > "aé"` (G1!D4) and `"áe" > "aè"` (G1!J4) although the second text's accents sum higher; a
    text-wide sum or a text-wide skip would make them equal or less. `"áé"` = `"a"&UNICHAR(769)&"e"&UNICHAR(769)`
    = `"a"&UNICHAR(769)&"é"` = `"á"&"e"&UNICHAR(769)` (G1!F2:H2; COUNTIF 4, Rank!E2). `"a"` plus 16
    U+0302 (wrapped sum 2, as plain `"a"`) then `"é"` is above `"aé"` (G1!D11): `"a"` is the first
    letter's proper prefix, so the second letter is never reached. `"a"&UNICHAR(781)&UNICHAR(768)&"e"`
    = `"a"&UNICHAR(770)&"e"` (16 = 3 + 13; G1!O12) and `"a"&UNICHAR(781)&UNICHAR(768)&"e"` <
    `"a"&UNICHAR(770)&"é"` (G1!P12).
  - **The skip stops at the letter**: units are never shared across a letter boundary (sheet G2,
    Data!B). `"a"&UNICHAR(769)&"b"` > `"ab"&UNICHAR(769)` (G2!E3: the first letter has units left
    against none), where a text-wide unit stream would make them equal; `"a"&UNICHAR(769)&UNICHAR(769)&"b"`
    > `"a"&UNICHAR(769)&"b"&UNICHAR(769)&UNICHAR(769)` (G2!J8). Letters themselves come first: `"áb"`
    < `"a"&UNICHAR(769)&"c"` (G2!L2, decided by `b < c`). `"a"&UNICHAR(769)&"b"&UNICHAR(769)` =
    `"á"&"b"&UNICHAR(769)` = `"a"&UNICHAR(833)&"b"&UNICHAR(769)` = `"a"&UNICHAR(769)&"b"&UNICHAR(833)`
    (G2!F5, O5, P5; COUNTIF 4, Rank!F5).
  - **Equal sums sharing a prefix are equal** (sheet G3, Data!C): after the shared units, only the
    sums of the rest count, however many units and in whatever order. `"a"&UNICHAR(769)&UNICHAR(770)`
    = `"a"&UNICHAR(769)&UNICHAR(768)&UNICHAR(781)` = `"a"&UNICHAR(769)&UNICHAR(781)&UNICHAR(768)` =
    `"a"&UNICHAR(833)&UNICHAR(770)` = `"á"&UNICHAR(770)` = `"á"&UNICHAR(768)&UNICHAR(781)` (G3!G5:H5,
    K5, Q5, R5; COUNTIF 6, Rank!G5); `"a"&UNICHAR(769)&UNICHAR(768)&UNICHAR(768)` =
    `"a"&UNICHAR(769)&UNICHAR(807)` (26 = 13 + 13; G3!T18); `"a"&UNICHAR(769)&UNICHAR(833)&UNICHAR(768)`
    = `"a"&UNICHAR(833)&UNICHAR(769)&UNICHAR(768)` (G3!P14); `"a"&UNICHAR(833)&UNICHAR(833)` =
    `"a"&UNICHAR(769)&UNICHAR(769)` (G3!M11) and both < `"a"&UNICHAR(769)&UNICHAR(770)` (G3!F11:
    after the shared acute, 12 against 16).
  - **Accents come before case and before skipped characters**, and case never counts (sheet G4,
    Data!D): `"ab"` = `"Ab"` = `"aB"` = `"AB"` and `"áb"` = `"Áb"` = `"áB"` = `"ÁB"` (G4!B1:E1,
    F5:I5; COUNTIF 4); `"Áb"` > `"aB"` (G4!D6); `"á-b"` > `"a-b"` (G4!J10), `"a-b"` < `"áb"`
    (G4!F9), `"á-b"` > `"áb"` (G4!F10) and `"a"&UNICHAR(769)&"-b"` = `"á-b"` (G4!K12).
  - **A mark after a hyphen or apostrophe goes with that skipped character, not with the letter
    before it**: it adds nothing to the accent pass and raises the skipped character in the last
    pass. `"a-"&UNICHAR(769)&"b"` is above `"a-b"`, `"ab-"`, `"a'b"` and
    `"a'"&UNICHAR(769)&"b"` and below `"áb"`, `"á-b"`, `"a"&UNICHAR(769)&"-b"` and `"á'b"` (G4 row 11;
    COUNTIF 1, Rank!H11), and `"a'"&UNICHAR(769)&"b"` is above `"a-b"` (G4!J20): `"a'b"` < `"a-b"` <
    `"a'"&UNICHAR(769)&"b"` < `"a-"&UNICHAR(769)&"b"` < `"áb"`. One mark (U+0301) was probed: how
    other marks or several marks weigh there is open. Ours (T-3329) drops the mark from the
    accent pass and adds its weight to the skipped character's last-pass weight (`skipped`).
  - Open, UNVERIFIED: marks after `æ`/`ß` (T-3203).
  Ours (T-3181): `cmp_text_pairwise` applies this rule for the operators and criteria, so every
  cell of the four accent cases passes. The rule is not transitive (wrapped-run-pairs has 554
  intransitive triples), so the sorts keep the wrapped total with nothing skipped (`cmp_text`, a
  total order); where Excel's sorts and lookups differ is in the "Not a total order" bullet.
Implemented (T-3154): `collation_tables.rs` (`MARK_WEIGHT` from the Windows table, `DECOMPOSED`
extended to Latin Extended-B and Additional) and `collation.rs` (accent byte, variant pass).

### Full-width and half-width forms, kana, voicing and the long-vowel mark (T-3275)
Evidence: probes/t3275-collation-width-kana (137 strings in Data!A; Calc!A = count less, B = count equal,
C = exact MATCH position, D = XMATCH position, E = COUNTIF `"="&x`, G1:G34 = `(x<y)+2*(x=y)+4*(x>y)` for
chosen pairs) and probes/t3275-collation-width-kana-pairs (37 strings, every pair: Calc!<col j+1><row i> =
the same code for x = Data!A<i>, y = Data!A<j>), excel-graph 2026-10-30. Below, "rank" cites case 1 Calc!A
by Data row and a bare cell cites the pairs case. The engine passes every cell of both cases since
T-3321 (before it, 172 + 358 failed).
VERIFIED (cells cited):
- **Width is a tiebreak, not a letter.** Full-width Latin letters, digits, punctuation and the
  ideographic space U+3000 weigh as their ASCII forms: `"１a" < "1b"` (rank A37 < A38), `"Ａa" < "ab"`
  (F4), `"！a" < "!b"` (AG31 = 4 read from the `!b` side), `"　a" < " b"` (AI33), `"＄a" < "$b"` (AK35).
  Each sorts right after its ASCII form (`"0" < "０" < "1"`, `"!" < "！" < "#"`, `" " < "　" < "!"`; case 1
  G6, G8, G9) and is never equal to it (`"Ａ"="A"` FALSE, G1). Case still folds: `"Ａ"="ａ"`, `"Ａb"="ａb"`
  (case 1 Calc!B25, B31 = 2). Width is weighed after the accents: `"Ａ" < "á"` (case 1 G3), `"Ａa" < "aá"`
  (D4), and left to right: `"AＢ" < "Ａb"` (H6). The full-width apostrophe U+FF07 and hyphen U+FF0D are
  ignored like `'` and `-`, after them in the tiebreak: `"ab" < "a＇b" < "a-b" < "a－b" < "AＢ"` (ranks
  A15, A62, A61, A60, A33 in that order; case 1 G10, G11; pairs G9), and `"'" < "＇" < "-" < "－"` alone.
- **Kana**: hiragana, katakana and half-width katakana of one syllable share a letter; base order is
  the syllabary, `あ < い < う < え < お < か < き < さ < た < な < は < ま < や < ら < わ < を < ん`
  (katakana interleaved, not after the hiragana block: `"ア" < "い"`, case 1 G15; `"アあ" < "あい"`, G16).
  Kana sort after Cyrillic (`"あ" > "я"`, G32) and the full-width Latin letters (`"ｱ" > "z"`, G31), before
  the ideographs `一`, `中` (G33). Kana type is a late tiebreak, half-width < katakana < hiragana, left to
  right: `"ｱ" < "ア" < "あ"` (G13, G14, G12), `"ｱあ" < "アあ" < "あア" < "ああ"` (ranks A95, A92, A91, A90).
- **Small kana and voicing are tiebreaks.** `ぁ`/`あ`, `っ`/`つ` share a letter, small first
  (`"ぁ" < "ｱ"`, rank A99 below A86; `"っあ" < "つあ" < "つい"`). Voicing is an accent: `"か" < "が"`,
  `"は" < "ば" < "ぱ"`, and `"があ" < "かい"` (case 1 G21), `"ばあ" < "はい"`. Small-kana and the long-vowel
  mark weigh **before** kana type (`"かぁ" < "カア" < "かあ" < "カー" < "かー" < "があ"`; U23, W20, V22,
  Y21), voicing before both (it is the accent level).
- **Combining and spacing voicing marks**: `"か"&UNICHAR(12441)` (U+3099) equals `"が"` and
  `"は"&UNICHAR(12442)` equals `"ぱ"` (case 1 B96, B97 = 2; G24 = 2); COUNTIF counts both (E96, E106 = 2)
  and XMATCH finds the decomposed spelling for `"が"` (D106 = 96), but exact MATCH does not (C106 = 106),
  as T-3114 found for Latin marks. The half-width U+FF9E and the spacing U+309B also voice the
  preceding character, as an accent and not an ignored mark: `"ｶﾞa" > "ｶá"` (Q15), `"か゛a" > "かá"`
  (S17), even after a Latin letter: `"a゛b" > "Ａb"` (H19) and `> "a－b"` (J19). `"ｶﾞ"` sorts just
  before `"ガ"` (half-width tiebreak) and is not equal to it (case 1 B98 = 1). A lone `"゛"` sorts among
  the ignorable-only strings, after `"－"`, before `" "` (rank A118 = 5).
- **Long-vowel and iteration marks after a kana or kanji repeat it.** `ー` after a kana is the
  preceding vowel, sorting just after the written vowel: `"かあ" < "かー" < "かい"` (case 1 G26; ranks
  A108, A120, A109) and `"きい" < "きー"` (ranks A126 = 100, A125 = 101), so the mark takes the vowel of
  the kana before it (`あ` after `か`, `い` after `き`), not a fixed letter. `ゝ` repeats the
  kana and `ゞ` repeats it voiced: `"かか" < "かゝ" < "かが" < "かゞ"` (ranks A131, A130, A133, A132;
  AE29). `々` after `人` repeats it but sorts **before** the written repeat: `"人" < "人々" < "人人"`
  (AB26, AC27), the opposite tiebreak to `ゝ`. Alone or first (`"ー"`, `"ゝ"`, `"ゞ"`, `"々"`, `"ーa"`),
  they sort after the ideographs probed (`"ー" > "人人"`, AC25; `"ーa"`, AC37) and the four lone marks
  are **equal** to each other (case 1 B119 = 4, COUNTIF E119 = 4; MATCH and XMATCH of each find `ー`,
  C127:D129 = 119).
  **A lone mark is above every character (T-4258, VERIFIED, probes/t4258-collation-lone-mark-astral,
  105 cells, excel-graph 2026-11-05).** Each of `ー ゝ ゞ ヽ ヾ 々` alone is greater than plane 2 (`𠀀`
  U+20000, `𪜀` U+2A700, U+2FA1D), plane 3 (`𰀀` U+30000), Hangul (`가`), the BMP ideographs (`一`, U+9FFF),
  Extension A (`㐀`), the compatibility ideographs (`豈` U+F900), private use (U+E000, U+F0000), `😀`,
  `𝐀` U+1D400, U+FFFD and U+10FFFD (C1:C90), and than the two-character texts U+10FFFD U+10FFFD and `𪜀𪜀` (C104:C105). What follows
  the leading mark does not change that: `々` or `ー` followed by a combining mark, a kana, an ideograph,
  `𠀀`, `A` or another mark is still above `𠀀` (C91:C102), and so is t3465's B313 text (C103). The
  engine gives a lone mark one weight above every script weight (`LONE_MARK`). Lone surrogates, which
  UNICHAR cannot make, are not probed against it.
- **`ー` after a Latin letter is not transitive.** `"aー"` is greater than `"aa"`, `"ab"`, `"AＢ"` and
  `"a゛b"` (C8, F8, G8, T8) but less than `"Ａa"` and `"Ａb"` (E8, H8), although `"Ａa" < "ab"` (F4) and
  `"AＢ" < "Ａb"` (H6). So Excel's `<` here is no sort key; no total order reproduces all cells, and an
  implementation can only match the pairs (the pairs case holds 6 such cyclic triangles, all through `"aー"`). Exact MATCH of
  `"aー"` finds `"aa"` (case 1 C136 = 13) while `=` says no string but itself is equal (B136 = 1) and
  XMATCH finds itself (D136).
Implemented (T-3321), `collation.rs`: after the accent pass come two new passes, a **kana pass**
(small kana 1, every other letter 2, `ー` after a kana and `ゝ ゞ ヽ ヾ` 3, `々` 1) and the **variant
pass** (superscripts 1, full-width forms 2, kana type: half-width 0, katakana 1, hiragana 2), then
the skipped characters (`' ＇ - － – —`). Kana weigh as their plain hiragana, by code point (the
syllabary), voicing an accent (voiced +1, semi-voiced +2; U+3099/U+309B/U+FF9E and U+309A/U+309C/U+FF9F
are marks adding the same). The marks resolve against the letter weight before them: `ー` after a
kana is its vowel, `ゝ`/`ゞ`/`々` repeat the letter (`ゞ` voiced), and with no letter before them
they are one weight after the ideographs. `ー` after any other letter is that letter again (so
`"aー"` sorts and hashes as `"aa"`, and `"aー" < "Ａa"`), except in Excel's `<`/`=` at the first
character where the two texts differ (case-insensitively), where it has no letter before it and is
the lone weight: `"aー" > "ab"`, `"aー" > "AＢ"`, `"aー" <> "aa"`. That reproduces the six cyclic
triangles; the collation key and the sorts keep the total order (`"aー"` = `"aa"`). Exact MATCH and
VLOOKUP/HLOOKUP exact with a lookup value without wildcards fold `ー` after a letter or digit that
is not kana or an ideograph to that character and a leading `ゝ ゞ ヽ ヾ 々` to `ー`
(`collation::fold_exact_lookup`); wildcard patterns and criteria keep the plain case fold.
Engine choices, UNVERIFIED: the voicing weights against the Latin marks; the full-width variant
against the superscript one; `ー` after `ん`, after an ideograph or after a digit or symbol; `ゝ`
and `々` after a Latin letter; the katakana-only `ヷ ヸ ヹ ヺ` (voiced `わ ゐ ゑ を`), `ゕ ゖ` (small
`か け`) and `ヽ ヾ`; the marks in wildcard patterns and criteria (plain case fold, so `COUNTIF(r,"aー*")`
does not count `"aa"`).
UNVERIFIED: whether full-width punctuation other than `！＃＄％＆（，．／：＠＿＋＜＝＞` and digits other
than the probed `０ １ ９` (not `２`-`８`) follow the same rule (only these were probed); the order of the kana-type, small-kana
and long-vowel tiebreaks against the Latin width tiebreak (different scripts never tie on letters in
these cells); where `ー` after a Latin letter weighs when compared with strings without full-width
letters (it sorts above `"ab"`, below the kana: U8).

### Other scripts, their marks and digits, symbols and astral characters (T-3276)
Evidence: probes/t3276-collation-other-scripts (excel-graph, 2026-10-31): 99 strings in Data!A; Calc!A =
count less, B = count equal, C = exact MATCH position, D = XMATCH position, E = COUNTIF `"="&x`,
G1:G36 = `(x<y)+2*(x=y)+4*(x>y)` for chosen pairs, H1:H13 single cells. "Rank" cites Calc!A by Data row.
The engine passes every cell: the collation cells since T-3324, the digit cells since T-3325: `cast::number_text_in` maps the three digit blocks, so VALUE, arithmetic coercion and criteria read them. T-3446 probed the other forms and blocks (next section "Script digits in number text (T-3446)"). VERIFIED (cells cited):
- **Block order is not code-point order.** Lowest first: Latin, Greek, Cyrillic, **kana**, Hebrew,
  Arabic, Devanagari, Thai, Lao, the astral `𝐀` (U+1D400) and `😀` (U+1F600), Hangul, CJK ideographs,
  then the astral ideograph `𠀀` (U+20000): `"я" < "ア" < "あ" < "א"` (ranks A18, A20, A19, A21 = 32,
  33, 34, 35), `"ω" < "א"` (G8), `"א" < "ا"` (G2), `"ا" < "क"` (G3), `"क" < "ก"` (G4), `"ก" < "가"` (G5),
  `"가" < "一"` (G6), `"一" > "あ"` (G7), `"ເກ" < "𝐀" < "😀" < "가"` (ranks A79, A96, A95, A44 = 80, 81,
  83, 84), `"𠀀" > "龍"` (G35). Kana sit below Hebrew although their code points are higher.
- **Inside each script, code-point order**, also where a dictionary or radical order would differ:
  `א < ב < ג < ת` (ranks A21:A24 = 35, 36, 42, 43; G12), `ا < ب < ت < ي`, `क < ख < ग < ह`, `ก < ข < ฮ`,
  `가 < 각 < 나 < 힣` (G11), `一 < 丁 < 中 < 乙 < 人 < 龍` (ranks A50, A51, A54, A52, A53, A55 = 90, 93, 94, 95, 96, 97; G9, G10).
  Two-letter strings compare letter by letter (`"一一" < "一丁" < "丁"`, `"가가" < "가나" < "각"`).
- **Hebrew points, Arabic harakat and the Thai tone mark are tiebreaks** (accents) on their letter:
  `"ב" < "ב"&UNICHAR(1463) < "ב"&UNICHAR(1468) < "בא" < "ב"&UNICHAR(1468)&"א" < "בב"` (ranks A22, A60,
  A58, A25, A59, A26; G13, G14), `"ب" < "ب"&UNICHAR(1614) < "ب"&UNICHAR(1617) < "با" < "ب"&UNICHAR(1614)&"ا" < "بب"`
  (G15, G16), `"ก" < "ก"&UNICHAR(3656) < "กก" < "ก"&UNICHAR(3656)&"ก" < "กข"` (G19, G20). None is equal
  to the bare letter (Calc!B = 1 on each row). **Devanagari matras and the virama are letters**, after
  the consonants in code-point order: `"कख" < "का"` (G18), `"का" < "कि" < "क्" < "ख"`, `"क्" > "क"` (G17).
- **Prevowels are not reordered.** Thai `เ` and `แ` sort **before** every consonant probed:
  `"เก" < "เข" < "แก" < "ก"` (ranks A72, A75, A74, A39 = 64, 65, 66, 67), `"กเ" < "กก"` (rank A73 = 69 <
  A42 = 70), `"เก" < "กเ"` (G21), `"เก" < "ขก"` (G22), `"เข" < "กข"` (G23); Thai `า` keeps its code-point
  place after the consonants (`"กข" < "กา" < "ข"`). The Lao prevowel `ເ` sorts **after** the consonants:
  `"ກ" < "ກເ" < "ຂ" < "ເກ"` (G24).
- **Other-script digits are their digit plus a tiebreak**: `"3" < "٣" < "३" < "3a" < "٣a" < "३a" < "3b"`
  (ranks A10, A81, A82, A85, A83, A84, A86 = 14 to 20; G25, G26, G27), `"2" < "٢" < "3"`, `"4" < "٤" < "9"`;
  never equal under `=` (Calc!B = 1). But COUNTIF `"="&x` counts every spelling of the digit
  (E10 = E81 = E82 = 3, E9 = E87 = 2, E11 = E88 = 2) because the criterion is numeric: **VALUE reads
  Arabic-Indic, Devanagari and Thai digits as numbers**, also mixed with ASCII digits and with a decimal
  point: `VALUE("٣")` = `VALUE("३")` = `VALUE("๓")` = 3, `VALUE("٢٣")` = 23, `"٣"+0` = 3,
  `VALUE("٣.٥")` = 3.5, `VALUE("1٣")` = 13 (Calc!H1:H5, H7, H8). Full-width `VALUE("３")` is an error (H6, wrapped in IFERROR, so which error is not probed).
- **No probed symbol above `€` sorts after the letters; the arrows and dingbats sort among the punctuation.** `"~" < "€" < "+" < ">" < "←" <
  "☺" < "♥" < "✓" < "0"` (ranks A4, A7, A5, A6, A90, A94, A92, A93, A8 = 3 to 11; G29, G30, G31), with
  `← ☺ ♥ ✓` in code-point order; `∞` sorts between `"9"` and `"a"` (rank A91 = 24), `™` between `"Ab"`
  and `"ÿ"` (rank A89 = 28). `😀` sorts above `☺` (G36), so astral characters are not punctuation.
  `𝐀` is not `A`: `"𝐀" > "A"` and `"𝐀b" > "Ab"` (G33, G34), and `"𝐀" < "𝐀b" < "😀"` (ranks 81, 82, 83),
  so the astral characters are letters of their own.
- In passing: exact MATCH, VLOOKUP(...,FALSE) and wildcard XMATCH of a lone `"~"` find the empty string,
  not a `"~"` cell (`MATCH("~",{"a","","~"},0)` = 2, H9; `MATCH("~",{"a","~"},0)` an error, H10; H11, H13;
  over Data, C4 = 1 finds `""` while XMATCH finds the `"~"` row, D4 = 4). The engine already agrees.
T-3559 (next section) settles this section's open questions (`™`, `∞`, the other astral characters,
the other Thai and Lao vowels, more Hebrew and Arabic letters) and T-3324's choices where this probe
is silent: the Thai prevowels `โ ใ ไ` do go with `เ แ`; Hebrew points and cantillation, Arabic harakat
and Thai tone marks are accents by code point (sampled at U+0591, U+05B0, U+0610, U+065F, U+0670,
U+0E4B); other-script digits, Thai and Lao included, are their digit plus a tiebreak (so below the
letters, as probes/t3465-collation-holdout found: `"ณ" > "๘"`, `"NPOECV" > "໖66"`; engine gap
fixed by T-3895). REFUTED by T-3559: the arrows, miscellaneous symbols and dingbats do not sort after `•`
(they sort before it, and not all by code point), `™` is not the letters `tm` with a variant
weight (it sorts after `"tn"`), and astral characters from U+D840 do not all sort after the CJK
ideographs (plane 3's `𰀀` sorts before Hangul). Still UNVERIFIED: lone surrogate units of planes 1
to 3 other than those probed by probes/text-lone-surrogates.

### Scripts and symbol blocks beyond T-3276 (T-3559)
Evidence: probes/t3559-collation-unprobed-blocks (excel-graph, 2026-11-03): 155 strings in Data!A,
with T-3276's layout (Calc!A = count less, B = count equal, C = exact MATCH, D = XMATCH, E = COUNTIF
`"="&x`) and G1:G25 = `(x<y)+2*(x=y)+4*(x>y)` for chosen pairs. "Rank" below is Calc!A of the Data
row. VERIFIED for the cells cited:
- **Characters Excel does not know weigh the same, just above the empty string.** `➿` (U+27BF),
  `ᡸ` (U+1878) and `₿` (U+20BF) are equal to each other and to nothing else: rank 1 and count 3 on
  each (A35, A62, A126; B35 = 3), COUNTIF counts all three (E35 = 3), MATCH and XMATCH find the first
  of them (C62 = D62 = 35), and `"➿" < "0"` (G17). They are above `""` (rank 0, B1 = 1) and below
  `"!"` (rank 4). Other characters of the same blocks (`✓`, `☀`, `ᠠ`, `₹`) sort normally. Inferred,
  UNVERIFIED: Excel's table lacks these code points (they were added in Unicode 6.0, 11.0 and 10.0),
  and every code point it lacks weighs this way.
- **Symbols sort among the punctuation, not by code point and not by block.** Lowest first, with
  the ranks: `! # $ %` (4 to 7), `、` `。` (8, 9), `~` (10), `「` (11), `₩` (12), `€` (13), `₹` (14),
  `+` (15), `≠ ∑ ∫ ≤` (16 to 19), `■ ●` (20, 21), `← ⇿` (22, 23), `─ █` (24, 25), `☀ ✓` (26, 27),
  `•` (28), then the digits from `0` (29). So currency signs are not one group (`₩ < € < ₹`, with
  `₿` unknown above), math operators are not in code-point order (`≠` U+2260 < `∑` U+2211 < `∫`
  U+222B < `≤` U+2264), and `•` sorts **after** the arrows, symbols and dingbats (`"⇿" < "•"`,
  `"☀" < "•"`; G16, G18).
- **`∞` is above every punctuation mark and digit**, between `"9"` and `"a"` (rank 42; G6:G12 = 4
  against `! # % + ~ • 0`). `℃` and `№` sort after `"a"` and before `"tm"` (ranks 44, 45); whether
  they expand to letters (`°c`, `no`) is UNVERIFIED.
- **`™` is a letter after `"tn"`**: `"™" > "tm"`, `"TM"`, `"Tm"`, `"t"` and `"tn"` (G1:G5 = 4), and
  `"™" < "z"` (rank 49 against 50). So it is not `tm` with a variant weight. Where it sits between
  `"tn"` and `"z"` is UNVERIFIED. `"tm" = "TM" = "Tm"` (B15:B17 = 3).
- **Block order, lowest first, continued**: Latin, Greek, Cyrillic, **Armenian, Georgian**, kana,
  Hebrew, Arabic, Devanagari, **Bengali, Gurmukhi, Tamil, Telugu**, Thai, **Myanmar, Mongolian,
  Tibetan**, Lao, **Khmer, Ethiopic**, then the astral characters below U+20000 (planes 1 and 3),
  Hangul, the CJK ideographs, and plane 2 (`𠀀`, `𪜀`). Ranks: `я` 52 < `Ա` 53 < `ა` 56 < `あ` 59 <
  `א` 60; `ي` 83 < `क` 84 < `অ` 85; `హ` 100 < Thai `เก` 101 … `ำ` 115 < `က` 116 < `အ` 118 < `ᠠ` 119
  < `ཀ` 120 < `ཨ` 122 < `ກ` 123 … `ໃ` 132 < `ក` 133 < `អ` 136 < `ሀ` 137 < `ፚ` 139 < `𐀀` 140 < `𝐀`
  141 < … < `😀` 149 < `𰀀` 150 < `가` 151 < `一` 152 < `𠀀` 153 < `𪜀` 154. Inside the blocks the order
  is mostly code-point order, with these exceptions:
  - **Gurmukhi** puts `ਹ` (U+0A39) before `ਕ` (U+0A15): `"ਅ" < "ਹ" < "ਕ" < "ਕਿ"` (ranks 89 to 92).
  - **Georgian** puts the Mkhedruli letters before the Asomtavruli capital: `"ა" < "ჰ" < "Ⴀ"`
    (ranks 56, 57, 58), and `Ⴀ` is not equal to `ა` (B54 = 1).
  - **Lao** `ໄ` (U+0EC4) sorts before `ໃ` (U+0EC3) (ranks 131, 132).
  - **Armenian** case pairs are equal: `"Ա" = "ա"` (G15 = 2, B55 = 2), `ա < ֆ`.
  - **Hebrew** final letters equal their normal form: `"ץ" = "צ"` (G14 = 2, B97 = 2); `צ < ש < ת`.
  - **Arabic** `ﻻ` (U+FEFB, presentation form) equals `"لا"` (G13 = 2, B109 = 2; XMATCH of `"لا"`
    finds the `ﻻ` row first, D110 = 109, while exact MATCH finds its own row, C110 = 110). `ء` sorts
    before `ا` (ranks 71, 72), `ى < ي` (82, 83).
- **Marks.** Hebrew points and cantillation, Arabic harakat and the Thai tone mark `๋` are
  tiebreaks on their letter, lighter at a lower code point: `"ב" < "ב֑" < "בְ" < "בא" < "ב֑א" <
  "בְא"` (ranks 61 to 66; U+0591 before U+05B0), `"ب" < "بؐ" < "بٟ" < "بٰ" < "با" < "بؐا" < "بٰا"`
  (ranks 73 to 79), `"ก" < "ก๋" < "ก๋ก" < "กะ"` (ranks 108 to 111). The vowel signs of Bengali,
  Gurmukhi, Tamil, Telugu, Tibetan, Khmer and Myanmar follow their consonant before the next
  consonant (`"ক" < "কি"`, `"ក" < "កា" < "កិ" < "អ"`); whether they are letters or accents is not
  separated by these strings, UNVERIFIED.
- **Thai prevowels `โ ใ ไ` sort before the consonants, like `เ แ`**: `"เก" < "โ" < "โก" < "ใ" < "ใก" <
  "ไ" < "ไก" < "ก"` (ranks 101 to 108); `ะ` and `ำ` sort after the consonants (`"ข" < "ะ" < "ำ"`, ranks
  113 to 115). **Lao prevowels sort after the consonants, like `ເ`**: `"ກ" < "ກະ" < "ຂ" < "ະ" < "າ" <
  "ແ" < "ໂ" < "ໂກ" < "ໄ" < "ໃ"` (ranks 123 to 132).
- **Other-script digits are their digit plus a tiebreak by code point**, below the next digit and
  so below every letter: `"3" < "٣" < "৩" < "੩" < "௩" < "౩" < "๓" < "໓" < "༣" < "၃" < "៣" < "9"`
  (ranks 30 to 41; G21:G25 = 4), never equal under `=` (Calc!B = 1), but COUNTIF `"="&x` counts all
  eleven spellings of 3 (E11 and E136:E145 = 11): the criterion reads each as the number 3.
  Implemented by T-3917 (rule 7): `element` gives the Gurmukhi, Tamil, Telugu and Khmer digits a
  `digit_variant` weight in this order, and `script_digit_to_ascii` reads them, so number-text coercion
  takes them in every form of the T-3446 blocks. T-3958 probed them (probes/t3958-gurmukhi-tamil-telugu-khmer-digits,
  excel-graph, 2026-11-03): the forms are in "Script digits in number text (T-3446)" below, and the pairs
  (Pairs!C1:C28) place the digit 3 of each of the four blocks below `"3a"`, `"4"` and the Arabic-Indic 9, and above
  `"3"`, the Extended Arabic-Indic 3 and the Devanagari 3 (code-point order of the blocks, weight of the digit
  first). The engine passes them.
- **Astral characters and emoji sequences compare unit by unit**, no sequence is one element:
  `"𐀀" < "𝐀" < "🀄" < "🇺" < "🇺🇸" < "👍" < "👍🏽" < "👨" < "👨‍👩‍👧" < "😀"` (ranks 140 to 149);
  the skin-tone modifier and the second flag letter make the string greater (G19, G20 = 4).
  Plane 3's `𰀀` (U+30000) sorts after `😀` and before Hangul (rank 150), plane 2's `𪜀` (U+2A700)
  after `𠀀` (154).
Implemented by T-3917 (every cell of the case passes): `element` (crates/rustsheet-engine/src/functions/util/collation.rs)
skips `➿ ᡸ ₿` like U+0E83 (a stand-in for T-3904's weightless rule), places `、 。 「 ₩ ₹` in
`PUNCTUATION`, `symbol` (collation/scripts.rs) orders the math operators `≠ ∑ ∫ ≤`, the geometric shapes, arrows,
box drawing and the symbols and dingbats, then `•`; `™` is `t` with a weight above every letter, `℃` and `№` the
letters `c` and `no` with the superscript variant (their places beyond the probed neighbours are UNVERIFIED);
`ץ` weighs as `צ` and `ﻻ` as `لا`; `script_order` puts Georgian after Armenian (Mkhedruli first), reorders U+0E00
to U+18AF by Excel's block order with the Gurmukhi `ਹ` and Lao `ໄ ໃ` exceptions, and sorts plane 3 and above with
planes 1 before Hangul. The unprobed blocks keep code-point order inside the span they fall in (UNVERIFIED).

### Thai, Lao and the other script digits (T-3895)
Evidence: probes/t3895-collation-script-digits (excel-graph, 2026-11-03), 191 cells: Calc!A =
`(x<y)+2*(x=y)+4*(x>y)` for chosen pairs, COUNTIF and exact MATCH against the text `"3"`. VERIFIED (cells cited):
- **The Thai, Lao, Bengali, Tibetan, Myanmar and Extended Arabic-Indic digits are their digit plus a tiebreak**, as
  the Arabic-Indic and Devanagari digits are (T-3276): `"3" < "๓" < "4"`, `"2z" < "๓" < "3a"`, `"๙" < "a"`, `"!" <
  "๓"` for each block at digits 0, 3 and 9 (A1:A150, five rows per digit per block). Never equal under `=`.
- **The tiebreak orders the blocks by code point**, every full-width digit first: `"3" < "３" < "٣" (U+0663) < "۳"
  (U+06F3) < "३" < "৩" < "๓" < "໓" < "༣" < "၃"` (A4:A6, A11:A13, A129:A131, A151:A178). It works left to right:
  `"๓"&"1" > "3"&"๑"` (A21, A46, A71, A96, A121, A146).
- COUNTIF of the text `"3"` with `"="&"๓"` is 1 for every block (the criterion reads as the number 3; T-3446 owns
  number-text reading), exact MATCH `#N/A` (A24, A25 and the same rows of each block).
- **U+0E83** (unassigned in the Lao block, a T-3584 weightless character): a lone one is above `""` and below `" "`,
  `"!"`, `"0"`, `"a"`, `"ÿ"` and `"ກ"` (A179:A184, A189); `"a"&U+0E83 > "a"` (A187), but `U+0E83&"a" = "a"`
  (A185); `U+0E83&"໖" < "ÿ"` (A188, the hold-out's row 599); `LEN` 1, `UNICODE` 3715 (A190, A191).
Implemented in `element` (crates/rustsheet-engine/src/functions/util/collation.rs): the digits as `digit_variant`
with one variant weight per block; U+0E83 as a skipped character, a stand-in that misses A185 until T-3904's
weightless rule replaces it. The Tibetan half digits U+0F2A..U+0F33 and the other digit blocks are unprobed.

### Script digits in number text (T-3446)
Evidence: probes/t3446-script-digits-forms (excel-graph, 2026-11-03); formats from the oracle's ENTERED log,
docs/research/t3446-script-digits-forms-2026-11-03.md. Rows 1:99 are nine scripts (ASCII, then Arabic-Indic
U+0660, Extended Arabic-Indic U+06F0, Devanagari U+0966, Bengali U+09E6, Thai U+0E50, Lao U+0ED0, Tibetan
U+0F20, Myanmar U+1040) of eleven forms each (`123`, `-12`, `1.5`, `12%`, `$12`, `(12)`, `1,234`, `1e3`,
`2020-1-1`, `12:30`, `1 1/2`, digits in the row's script, the rest ASCII); A = typed, B = ISNUMBER(A),
D = `C+0`, E = VALUE(C), F = NUMBERVALUE(C), G = TEXT(C,"0.000") with C the same text as a formula string.
Status: VERIFIED. T-3958 adds the Gurmukhi (U+0A66), Tamil (U+0BE6), Telugu (U+0C66) and Khmer (U+17E0) blocks
(probes/t3958-gurmukhi-tamil-telugu-khmer-digits, same layout, four blocks of eleven forms): all twelve blocks
behave alike. `+0`, VALUE, NUMBERVALUE and TEXT read every form (the engine passes), and typed text is a number
(B1:B44 TRUE; the engine passes since T-3891). The typed formats of all four blocks are logged by
probes/t3891-typed-script-digit-shapes A43:A86 (docs/research/t3891-typed-script-digit-shapes-2026-11-04.md):
tags `pa-IN,600`, `ta-IN,900`, `te-IN,A00`, `km-KH,1200`, the same shapes as the other blocks.
- **All eight blocks read as their digits in every form**, by `+0`, VALUE, NUMBERVALUE and TEXT alike, with
  the same value as the ASCII row: `"١٢%"+0` = 0.12, `"$১২"+0` = 12, `"(๑๒)"+0` = -12, `"໑,໒໓໔"+0` = 1234,
  `"༡e༣"+0` = 1000 (the `e` stays ASCII), `"၂၀၂၀-၁-၁"+0` = 43831, `"١٢:٣٠"+0` = 0.520833… (a clock too,
  which the engine read as `#VALUE!` before T-3446), `"१ १/२"+0` = 1.5, `TEXT("١.٥","0.000")` = `1.500`.
  Scripts mix with ASCII and with each other: `"1٢3"+0` = 123, `"١२"+0` = 12 (rows 100, 101; row 102, `١.٥`, repeats row 14 and is not a mixed-script row).
  UNPROBED: criteria (COUNTIF and the other readers through functions/criteria.rs, which share `cast::number_text_in`) on the five T-3446 blocks (Extended Arabic-Indic, Bengali, Lao, Tibetan, Myanmar) are probed only for one plain digit: `COUNTIF("3","="&digit 3)` is 1 (probes/t3895-collation-script-digits Calc!A49, A74, A99, A124, A149; Thai A24). Not probed: multi-digit and the signed, decimal, percent, `$`, parentheses, group-comma, exponent and ISO-date forms in a criterion, script-digit clock and mixed-fraction criteria, and criteria readers other than COUNTIF (SUMIF, the *IFS functions, the D-functions). The engine reads them by the shared path.
- **Only the digits are read: Arabic punctuation is not.** The Arabic decimal separator U+066B (`١٫٥`), group
  separator U+066C (`١٬٢٣٤`) and percent sign U+066A (`١٢٪`) make the text non-numeric: `#VALUE!` from `+0`,
  VALUE and NUMBERVALUE, and TEXT returns the text unchanged (rows 103:105).
- **Typed into a cell, every one of these is a number** (B12:B102 TRUE, rows 100 and 101 being the mixed-script ones; B103:B105 FALSE), and the cell gets
  a **locale-tagged** number format, not the en-US one ASCII text gets: `١٢٣` → `[$-ar-SA,200]0`, `١.٥` →
  `[$-ar-SA,200]0.#`, `١٢%` → `[$-ar-SA,200]0.00%` (ASCII `12%` is `0%`), `$١٢` →
  `[$$-ar-SA,200] #,##0.00_);[Red]([$$-ar-SA,200]#,##0.00)`, `١,٢٣٤` → `[$-ar-SA,200]#,##0`, `٢٠٢٠-١-١` →
  `[$-,200]mm/dd/yyyy`, `١٢:٣٠` → `[$-,200]h:mm`; `-١٢` and `(١٢)` → `[$-ar-SA,200]0`; the exponent form keeps
  `0.00E+00` and the mixed fraction `# ?/?`, untagged. The tag per block: `ar-SA,200`, `ar-SA,300` (Extended
  Arabic-Indic), `hi-IN,400`, `bn-IN,500`, `th-TH,D00`, `lo-LA,E00`, `bo-CN,F00`, `my-MM,1000`; `1٢3` takes
  `ar-SA,200` and `١२` takes `hi-IN,400`.
- **The shapes T-3446 left open (T-3891)**, probes/t3891-typed-script-digit-shapes (excel-graph, 2026-11-04;
  ENTERED log in docs/research/t3891-typed-script-digit-shapes-2026-11-04.md), each a number (B1:B42 TRUE):
  - **The last script digit picks the tag**, whatever comes before it: `١२๓` → `th-TH,D00`, `๓२١` and `१٢3`
    → `ar-SA,200`, `1٢३` → `hi-IN,400`, `12٣` → `ar-SA,200` (A31:A39).
  - **A plain number counts its typed decimals**: `١.٢٥` → `[$-ar-SA,200]0.##`, `١.٢٥٠` → `0.###` (value 1.25),
    `١٢.٥٠` → `0.##`, `٠.٥` and `.٥` → `0.#`, `+١٢` → `0`; an integer of 12 digits `١٢٣٤٥٦٧٨٩٠١٢` →
    `0.#####E+00`, tagged (the ASCII forms are all General; 11 digits and below and long decimals unprobed).
  - **Every percent is `0.00%`** (`١٢.٥%`, `-١٢%`, `١,٢٣٤%` = 12.34); **every currency the long form**
    `[$$-ar-SA,200] #,##0.00_);[Red]([$$-ar-SA,200]#,##0.00)`: `-$١٢`, `$-١٢`, `$١,٢٣٤.٥`, `$١.٥`, `$١٢.٣٤`,
    `$(١٢)` (= -12; ASCII `$(12)` is -12 too, "Currency sign before parentheses is negative").
  - Groups as ASCII, tagged: `١,٢٣٤.٥` and `١,٢٣٤.٥٦` → `[$-ar-SA,200]#,##0.00`, `(١,٢٣٤)` → `#,##0`.
  - **Dates take the long fields under `[$-,200]`**: `١/٢/٢٠٢٠` → `mm/dd/yyyy`, `١ Jan ٢٠٢٠` and `Jan ١, ٢٠٢٠` →
    `dd-mmm-yyyy` (ASCII gives `m/d/yyyy` and `d-mmm-yy`); a date-time `٢٠٢٠-١-١ ١٢:٣٠` keeps `m/d/yyyy h:mm`,
    clocks keep theirs: `h:mm:ss`, `h:mm AM/PM` (`١٢:٣٠ PM`, `١:٣٠ AM`, `١ PM`), `mm:ss.0` (`١٢:٣٠.٥`).
  - Untagged as ASCII: `١E-٣` → `0.00E+00`, `١ ١/١٢` → `# ??/??`.
  - Thai `๑๒.๕%` → `[$-th-TH,D00]0.00%`, `๑.๒๕` → `0.##`; Devanagari `-$१२` the long currency form with `hi-IN,400`.
  How the tags print: number-formats.md "Numeral-system digits" (`locale,N00` form).
  Unprobed and inferred in ours: a date with a month name and no year (ours `dd-mmm-yyyy`), a mixed fraction
  with `%` (ours `0.00%`, tagged), the 12-digit boundary of the exponent form.
- In passing: a bare typed ASCII clock `12:30` is a number with `h:mm` in Excel (B10); the engine keeps it
  text (engine gap T-3894). `NUMBERVALUE("1 1/2")` is 46328 in 2026: NUMBERVALUE drops every space
  (text.md), so it reads the date `11/2` of the current year; the case does not score F on those rows.
Implemented in: `script_digit_to_ascii` and `ascii_script_digits` (formatter/parse_formatted.rs), applied in
`cast::number_text_in` before the formatted-number and clock readers and before both mixed-fraction readers
(`cast_number_in`, `value_number_text_with`). Typed input (T-3891): `parse_typed_number` reads the ASCII text and `script_digit_format` gives Excel's
format, the tag from `SCRIPT_DIGITS` (same file).

### Collation equality in lookups and criteria; unprobed characters (T-1590)
Evidence: probes/text-collation-lookups (66 formula cells) and probes/text-collation-unprobed (90 strings ranked by
`SUMPRODUCT(--(range<x))`), excel-graph 2026-10-15. Status: VERIFIED.
- Equality by collation (`"æ"="ae"`, `"ß"="ss"` TRUE; `"a-b"="ab"`, `"a"="á"`, `"ø"="o"`, `"co-op"="coop"` FALSE) applies in
  **XMATCH, XLOOKUP, SWITCH** (`XMATCH("ae","æ",0)` = 1, `SWITCH("ae","æ",...)` hits) and in **COUNTIF**'s text criteria (below).
  It does **not** apply in **MATCH(...,0)** and **VLOOKUP(...,FALSE)**: both give `#N/A` for `"ae"` against `"æ"` and `"ss"` against `"ß"`.
  (The engine already agrees on every lookup cell; the old "inferred: exact-match lookups treat æ and ae as equal" held for XMATCH, XLOOKUP and SWITCH only.)
- COUNTIF over {"æ","ae","a-e"}: criteria `"ae"`, `"=ae"`, `"æ"` count 2 (æ and ae); `"<>ae"` and `"<>æ"` 1 (a-e); `"<ae"` 0; `"<=ae"` and `"<=æ"` 2;
  `">ae"` 1; `">=ae"` 3; `"a-e"` 1; `"<>a-e"` 2. So the criteria use the same collation for `=`, `<>` and the ordered operators (see criteria.md). SUMIF, SUMIFS and COUNTIFS agree with COUNTIF (T-1597, criteria.md); wildcard `?`/`*` do not expand.
- Order of the characters not probed in T-1469 (lowest first, `=` marks equal; whole table in the oracle sidecar): empty, CHAR(1), CHAR(31), CHAR(127),
  `–` `—` (en and em dash behave like a hyphen: ignored, tiebreak only: `"a'b" < "a-b" < "a–b" < "a—b"`, all after `"ab"`),
  CHAR(160), CHAR(9), CHAR(10), CHAR(13) (these three sort above CHAR(160), not below the space), then
  `! ~ ¡ ¦ ¨ ¯ ´ ¸ ¿`, `‘ ’ “ ”` (`’` is an ordinary punctuation mark, not ignored: `"a’b" < "ab"`), `¢ £ ¤ ¥ €`, `+ ±`, `« »`, `× ÷`, `§ © ¬ ® ° µ`, `… •`,
  `0`, `¼ ½ ¾`, `¹ ² ³`, `9`, then letters: `a=A < ª < á < ä < ã < å`, `c < ç`, `d < đ < ð`, `i < ı < ij=ĳ`, `l < ł`, `n < ñ`, `o < º < ö < ø < œ`,
  `s`, `t < þ`, `u`, `z`, `α < β < ω`. Notes: `ĳ` equals `ij` (an expansion like `æ`/`ß`). `œ` sits between `ø` and `s`; it equals `oe` (T-1662). `ã` sorts after `ä`.
  `ç`, `đ`, `ð`, `ł`, `ñ`, `þ` and the digit-like symbols were probed as single characters only (the symbols against `0` and `9` only); the choices made for them are in the next bullet.
- Fitted in `cmp_text_excel` (T-1595; all 270 cells pass), with these choices where the probe is single-character only (each UNVERIFIED):
  control characters other than CHAR(9)/CHAR(10)/CHAR(13) are skipped like the hyphen (tiebreak weight below the apostrophe, by code point), so
  `"a"&CHAR(1)&"b"` sorts just after `"ab"`; CHAR(11) and CHAR(12) sit with CHAR(9)-CHAR(13) above CHAR(160); the en and em dash are skipped
  everywhere, also alone (`"" < "–"` by tiebreak, below CHAR(160)); the Latin-1 marks, quotes and currency signs go between `~` and `+`, the
  symbols `± « » × ÷ § © ¬ ® ° µ … •` after `>`; `¼ ½ ¾` between `0` and `1` (by value), `¹ ² ³` as `1 2 3` with a tiebreak accent;
  `ª º`, `đ ł` (stroke, like `ø`) and `ð ı` are tiebreak accents on `a o d l i` (`þ` is the expansion `th`, below), not letters of their own; `œ` = `oe` like `æ`.
- Multi-character and lookup check of those choices (T-1662; probes/text-collation-longer, 107 cells, excel-graph 2026-10-16). VERIFIED as fitted:
  `œ` = `oe` (also `Œ` = `OE`; XMATCH, XLOOKUP and COUNTIF equal, MATCH(...,0) `#N/A`); `"a"&CHAR(1)&"b"` sorts after `"ab"` and `"a b"` and before `"a'b"`, CHAR(1) above `""` and below `"'"`
  (and `COUNTIF` of `"ab"` over {CHAR(1)-text, "ab"} is 1, XMATCH `#N/A`); CHAR(11) and CHAR(12) above CHAR(160) and CHAR(9), below CHAR(13), CHAR(11) < CHAR(12);
  every one of `± « » × ÷ § © ¬ ® ° µ … •` sorts above each of `<` `=` `>`; `¼ ½ ¾` between `0` and `1` (`¾` < `2`); `¹ ² ³` above `1 2 3` but below the next
  letter position (`"¹"` > `"1"`, `"¹a"` < `"1b"`, `"¹b"` > `"1a"`; XMATCH of `"1"` against `¹` `#N/A`); `ç đ ð ł ñ ı` are accents on `c d d l n i` (`"ða"` < `"db"`, > `"da"`).
  CORRECTED: `þ` is an expansion equal to `th`, not an accent on `t` (`"þ"="th"`, `"þ"&"a"` = `"t"&"h"&"a"` TRUE, `"þa"` > `"tb"`, `XMATCH("th","þ",0)` = 1). Implemented (T-1669): `cmp_text_excel` makes `þ` an `expansion('t','h')`.
  A single-cell reference as XMATCH `lookup_array` or XLOOKUP `lookup_array`/`return_array` is a one-cell vector, not `#VALUE!`/`#N/A` (probes/text-collation-longer A96, A97, A102, A104, A105, A107).
- The middle dot `·` (U+00B7) is an ordinary symbol, not skipped and not equal to `""`: it sorts just after `µ` and before `…`
  (`§ < © < ¬ < ® < ° < µ < · < … < • < 0`; `"·" < "··" < "·a" < "·b"`, `"a·" < "a·b" < "ab"`; XMATCH/COUNTIF find it only as itself). T-3896; probes/t3896-collation-middle-dot (excel-graph 2026-11-03, Calc!A1:E54). Implemented in `PUNCTUATION`.
Tasks: T-0220, T-0244, T-1469, T-1590, T-1594, T-1595, T-1662, T-1669, T-3114, T-3154, T-3896

## Double unary minus coerces to a number
Rule: `--x` is not a no-op. Each unary minus coerces its operand to a number: `--FALSE` = 0,
`--("">1.5)` = 1, `--""` (a cell holding empty text) = `#VALUE!`, `--100` = 100.
Evidence: gen-m1/6462bf910ef9f28f Calc!A49 (`--Data!C4`, C4 = FALSE), A84 (`--Data!$A11`, A11 = ""),
A17; gen-m1/57fe331cb7611dbf Calc!A167 (oracle: excel-graph, 2026-10-05, T-0109). Upstream IronCalc's
Excel-authored `TEXT/MINUS_MINUS.xlsx` (`ironcalc@9b1ae3fd`; 72 of its 73 cells pass with T-0132)
agrees: `--TRUE` = 1, `---TRUE` = -1, `--"3"` = 3, `--"abc"` = #VALUE!, `--A1:A4` lifts element-wise.
A plus sign is a no-op (`+x` stays text or boolean).
A single unary `-` or `%` also works element by element on a range or array: `INDEX(...,1,0)%`
= 0 (gen-m2/f9d4df4efdfbcbd5 A125, oracle excel-graph); the spill shapes asserted in
`test/test_unary_operators.rs` are upstream's, not yet oracle-checked.
Status: VERIFIED
Implemented in: `Parser::parse_power` keeps one `UnaryKind::Minus` per minus sign; `Model::apply_number_unary`
evaluates it (element-wise over ranges and arrays); `stringify`/`move_formula` print `--x` as typed and
parenthesise a lower-precedence operand (T-0132, `Ported-From: ironcalc@9b1ae3fd`). `--"12:00"`
(MINUS_MINUS.xlsx C12, Excel 0.5) still fails: time text coercion is T-0146, which adds that workbook.
Tasks: T-0132

## Weightless characters in `=`, COUNTIF and XMATCH (T-3584)
Rule: the 2568 BMP code points Excel's comparison table gives no weight (lookup.md "Weightless characters",
listed in `docs/research/t3584/weightless-bmp.txt`) compare equal to one another under `=`, COUNTIF and
XMATCH(..,0), but not under EXACT; unlike exact MATCH, these three do not count them one for one:
- `UNICHAR(1316)=UNICHAR(4295)` (U+0524 with U+10C7), U+0524 with U+0525, U+0378 and U+00AD, U+AB70 with
  U+1C90: TRUE (probes/t3584-match-fold-weightless Calc!B1:B3, B8, B14, B17:B18; COUNTIF and XMATCH agree, D:F).
- `"a"&UNICHAR(1316)&"b"="ab"` and `UNICHAR(1316)&UNICHAR(4295)=UNICHAR(1317)` are TRUE (Calc!B6:B7, D6:D7,
  F6:F7), where exact MATCH says `#N/A` (E6:E7).
- But a weightless character is never equal to empty text (`UNICHAR(1316)=""` FALSE, Calc!B4, B9, B12, B19, B32),
  and `"a"&UNICHAR(1316)="a"` and `"a"&UNICHAR(173)="a"` are FALSE (Calc!B5, B13; COUNTIF 0, XMATCH `#N/A`).
- Over the whole BMP (Calc!R1:R256, digit 9 = bits 1 and 8), `x=UNICHAR(1316)` is TRUE exactly where exact MATCH
  finds U+0524 (wildcards aside).
- Not about weightless characters, found by the same probe: `=`, COUNTIF and XMATCH(..,0) also equate Greek
  Extended's dasia+oxia and psili+varia letters (`UNICHAR(7941)=UNICHAR(7946)`, U+1F05 with U+1F0A, TRUE; Calc!B21,
  D21, F21; MATCH agrees, lookup.md "Equivalence classes"), `ϵ=ε` (U+03F5, Calc!P7) and U+1FBE `=ι` (Calc!P9), and
  keep the supplementary case pair Deseret U+10400 and U+10428 apart (Calc!B25, D25, F25 FALSE and 0). UNVERIFIED:
  whether the Greek Extended class is the accent-weight rule of T-3154 (psili+varia and dasia+oxia summing to the
  same weight). The text criteria without a wildcard equate `Ͷ`/`ϳ` and `ϑ`/`ϴ` too (probes/t3929-wildcard-fold-criteria
  Calc!B29:D30, t3929-wildcard-fold-xmatch Calc!E29:E30; criteria.md "Case beyond ASCII").
  The other classes of lookup.md's "Equivalence classes" hold under `=`, `<` and COUNTIF too. In each case x = y is
  TRUE, x < y and x > y are FALSE, x&"b" < y&"a" is FALSE (they tie at the first pass), and COUNTIF counts 1. The
  pairs are:
  - the Hebrew final letters `כ ך`, `מ ם`, `נ ן`, `פ ף`, `צ ץ`;
  - Latin Extended-D `Ꜹ Ꜻ`, `ꜹ ꜻ`, `Ꝥ Ꝧ`, `Ꝯ ꝰ`;
  - Georgian `გ` with U+10FA;
  - the ypogegrammeni U+1F8D with U+1F82 and U+1F88 with U+1F80;
  - `ϴ ϑ`, `ϳ Ͷ`, `ϵ ε`, U+1FBE `ι`, and U+1F05 with U+1F0A.

  `ϴ` is not `θ`: `ϴ=θ` is FALSE and `ϴ>θ` is TRUE, and COUNTIF counts 0 (probes/t4013-class-equality-order
  Calc!B1:F18, excel-graph, 2026-11-04, T-4013).
  Implemented in: the collation's `fold` (crates/rustsheet-engine/src/functions/util/collation.rs) maps each member
  of those classes to the class's first member through the one `CLASSES` table that exact MATCH uses
  (`lookup_fold::class_first`), and leaves supplementary characters uncased (T-4013). So `=`, `<`, the criteria, the
  sorts and XMATCH's index weigh class members alike. `ϴ` therefore sorts at `ϑ`'s place, above `θ`, as Excel does.
  These do not follow MATCH: the lam-alef ligature `ﻻ` and the weightless characters (above and lookup.md).
Where a weightless character counts: settled by T-3662's prefix rule in "Format characters, U+200B and the C1
controls (T-3662)" below. It counts only when it is in the part one text has beyond the other, and the shared
part before it is plain ASCII below U+007B in both texts.
Implemented in: the collation's `elements` drops the weightless characters and `cmp_prefix` keeps a trailing one
apart under `=` (T-3904; see the T-3662 section below).
Status: VERIFIED (excel-graph, 2026-11-03) for the cells cited; the position rule VERIFIED by T-3662 (below).

## Format characters, U+200B and the C1 controls (T-3662)
Evidence: probes/t3662-collation-ignorables (17 characters × 39 questions), -prefix (152 cells) and -sweep (471
cells), excel-graph, 2026-11-04. Call a "fast unit" a UTF-16 unit below U+007B: the ASCII controls, space,
punctuation, digits and letters up to `z`, but not `{ | } ~`, DEL or anything above it.

**The format characters are weightless.** U+00AD soft hyphen, U+200C ZWNJ, U+200D ZWJ, U+200E LRM, U+200F RLM,
U+2060 word joiner, U+FEFF BOM and U+FE0F VS16 behave exactly like U+0524 and the other weightless characters
(the T-3584 list holds all of them): rows 1, 3:9 of -ignorables match rows 13:17 (U+0524, U+10C7, U+AB70, U+1C90,
U+0E83) in every column.
- Inner and leading positions are ignored: `"a"&c&"b"="ab"`, `c&"ab"="ab"`, `"a-"&c&"b"="a-b"`,
  `"a"&c&"-b"="a-b"` TRUE (Calc!B, C, O, P), and `<` and `>` tie them (R, S FALSE). Two weightless characters are
  equal, alone or in a run (`c&c=c`, `c=UNICHAR(1317)` TRUE; M, N; Calc!AC = 13: all 13 rows' a&c&b are equal).
- **The prefix rule (trailing position).** Compare the two texts unit by unit from the start while both units
  are fast and equal ignoring ASCII case. If one text runs out during that walk while the other has units left,
  the shorter text is less, whatever those units are. Otherwise, at the first unit that is not fast or the
  first difference, the whole texts are compared by the collation, where weightless characters weigh nothing.
  So:
  - `"ab"&c="ab"`, `"1"&c="1"`, `"a-"&c="a-"`, `"a "&c="a "`, `"a"&CHAR(1)&c="a"&CHAR(1)`, ``"a`"&c="a`"``,
    `c=""` and `"AB"&c="ab"` are FALSE, and `"ab"&c>"ab"` is TRUE (-ignorables D, G:L, T; -prefix C:F; -sweep
    A1:C122 and E12, every CHAR(k) for k = 1 to 122).
  - `UNICHAR(233)&c=UNICHAR(233)`, `UNICHAR(945)&c=UNICHAR(945)`, `UNICHAR(161)&c=UNICHAR(161)` (T-3465's `"¡"`),
    `"a"&UNICHAR(233)&c="a"&UNICHAR(233)`, `"a~"&c="a~"`, `"~ab"&c="~ab"`, `"a~b"&c="a~b"` and the same after
    `{ | }`, DEL and U+0080 are TRUE (-ignorables E, F, I; -prefix B, N:P; -sweep A123:C127, E1:E4, E9:E11): a
    non-fast unit anywhere in the shared start sends the texts to the collation.
  - Once both texts reach a unit, the rule no longer applies: `"ab"&c="ab"&c&c`, `"ab"&c="ab"&UNICHAR(1317)`
    and `c&"a"&c="a"` are TRUE (-prefix H, G; -ignorables Q).
  - A difference first goes to the collation: `"ab"&c<"ac"`, `"a-"&c<"ab"`, `"Ab"&c<"ac"`, `"ab"&c<"b"` TRUE and
    `"b"<"ab"&c` FALSE (-sweep F1:F6), and so do `"ab"&c<"ab"&CHAR(1)`, `"ab"&c<"ab-"` (-prefix J, K).
- COUNTIF without a wildcard and XMATCH(..,0) follow `=` here too: COUNTIF(ab&c, "ab") is 0 and XMATCH `#N/A`,
  COUNTIF(c&ab, "ab") 1, COUNTIF(aé&c, "aé") 1 (-ignorables AI, AJ, AM; -prefix R:U). EXACT is FALSE and
  exact MATCH `#N/A` throughout (lookup.md "Weightless characters": MATCH counts each unit). Wildcards count
  the character: `"a?b"` and `"a*b"` match a&c&b, `"a??b"` does not (AF:AH; criteria.md "Case beyond ASCII").
  LEN counts it (AO = 3).

**U+200B and the C1 controls are not weightless.** They have a primary weight of their own, so no cell finds
them equal to anything but themselves (-ignorables rows 2, 10:12; -prefix H10:H13 and -sweep H, M all FALSE).
- The Unicode spaces and U+200B (probes/t4039-collation-space-forms, 322 cells, excel-graph, 2026-11-04, T-4039;
  Calc!B ranks the 23 characters, C counts the equal ones): the space and U+205F are equal (C1 = C17 = 2, H17),
  then U+3000 < U+202F < CHAR(160) < CHAR(9) < U+2000 = U+2002 < U+2001 = U+2003 < U+2004 < ... < U+200A <
  U+200B (B1:B18 in row order = 0, 4, 5, 2, 6, 8, 6, 8, 10 to 16, 3, 0, 17), and U+200B sorts below `!` and every symbol tested
  (-prefix B10 FALSE, C10:N10 TRUE, P10 = 2, with CHAR(1) below it: O10 FALSE). U+200B is above CHAR(10) and
  CHAR(13) as well (-sweep G2:G4, I2:I4 FALSE), and so are U+2000 to U+200A, strictly
  (probes/t4060-collation-open-choices Calc!H8:I14 FALSE, and J8:J14 FALSE where J7,
  `CHAR(13)&"a"<CHAR(13)&"b"`, is TRUE).
- The C1 controls sort after the symbols tested and before the digits, in code-point order: U+0080, U+0081,
  U+0090, U+009E and U+009F rank 18 to 22 above U+200B (t4039 B19:B23), above `¬` and `•` (K19:L23) and below
  `0` (J19:J23). U+0081, U+0085 and U+009F are above `! # $ & * . _ ~ ¡ ¿` and CHAR(1) (-prefix B11:L13 FALSE,
  M11:N13 TRUE, P11:P13 = 12). Against `+ = > ¤ § ° ± × ÷ • → ★ © ¬`, U+0081 is strictly above (-sweep K, M)
  and U+009F at least as high (L; no `=` column). All 32 (probes/t4270-collation-c1-controls, 548 cells,
  excel-graph, 2026-11-05, T-4270): each is equal only to itself (C1:C32, E1:E32 = 1; XMATCH, COUNTIF and exact
  MATCH find its own row, F:H), and they rank 3 to 34 in code-point order among them, U+200B (0), `¬` (1), `•` (2)
  and `0` (35) (B1:B36, the same for a&c&b in D1:D36); every one is below `0` and above `¬`, `•` and U+200B
  (I:L), `"a"&c&"b"<"ab"` (M), and SORT puts them in code-point order both ways (N1:O32).
  Implemented in: `collation.rs` `element`'s arm `'\u{80}'..='\u{9f}' => single(C1_BASE + (c - 0x80))`.
- So `"a"&c&"b"<"ab"` is TRUE for U+200B and the C1 controls (-ignorables R2, R10:R12), and among the 17 a&c&b
  texts U+200B < U+0081 < U+0085 < U+009F < the weightless ones (Calc!AB).

T-4060 settled three choices those tasks made without evidence (probes/t4060-collation-open-choices, 222 cells,
excel-graph, 2026-11-04):
- **Which spaces differ from the space only at a later pass.** `x&"a"<" b"` is TRUE for U+3000, U+205F and U+202F,
  so their first-pass weight is at most the space's; with t4039's order (the space ≤ U+3000 < U+202F, Calc!B1,
  B4, B16) it is the space's. It is FALSE for CHAR(160), CHAR(9), CHAR(10), CHAR(13), U+2000 to
  U+200B and U+0081, which have weights of their own (Calc!B1:B15, K1:K15). `x&"a"<UNICHAR(8194)&"b"` is TRUE for
  U+2000 (equal to U+2002 at that pass) and FALSE for U+2001 (E8:E10).
  Implemented in: `element` weighs U+202F as `SPACE` with the variant weight `NARROW_NBSP`, above U+3000's
  `FULL_WIDTH` (T-4072).
- **The sorts and approximate lookups apply the prefix rule too**, being Excel's pairwise comparison:
  - `MATCH("ab",{"a";"ab"&U+00AD;"abc"},1)` is 1, as are VLOOKUP(..,TRUE) and XMATCH(..,-1); XMATCH(..,1) is 2 and
    the binary exact XMATCH `#N/A` (Calc!B30:B36).
  - UNIQUE keeps `ab` and `ab`&U+00AD apart and `""` and U+00AD apart, but merges `é` and `é`&U+00AD (B37:B39).
  - SORTBY's order of the five texts that tie at every pass depends on their input order (Calc!B20:D26: ascending
    6,1,2,3,4,5,7; the reversed input 6,4,5,3,1,2,7), as a non-transitive comparison gives.
  Ours: SORT, SORTBY, the lookups, SWITCH and UNIQUE compare with `cmp_text_pairwise` (`compare_values_pairwise`)
  and pass every cell. GROUPBY and PIVOTBY do not yet (T-4086 below, T-4088).
- **A mark after a weightless character is a lone mark.** `"a"&U+00AD&U+0301` is above `"a"` and below `"á"`
  (Calc!B50:B54), and so is it with U+0524 (B55:B57). With a `b` after it, it is below `"áb"` and above `"ab"`, and
  COUNTIF does not count it as `áb` (B58:B61, B63). `U+00AD&U+0301` equals a lone U+0301 (B62). So the weightless
  character ends the letter before it, and the mark weighs as a mark with no letter.
  Implemented in: `elements` yields `Element::Weightless` for it, which has no weight in any pass but ends the
  letter before it, so `accent_units` gives the mark a letter unit of its own (T-4072).

T-4086 settled what the review of T-4072 left open (probes/t4086-collation-later-passes, M2, 75 cells, and
probes/t4086-groupby-weightless, M7, 28 cells; excel-graph, 2026-11-04):
- **U+202F differs from the space at the accent pass.** `U+202F&"a"<" "&"á"` is FALSE, and so are
  `<U+3000&"á"` and `U+202F&ぁ<" "&あ` (Calc!B1, C1, F1). U+3000 and U+205F give TRUE for the first two (B2:C4).
  It is the accent pass itself, compared letter by letter: with the positions reversed, `"a"&U+202F < "á"&" "`
  is TRUE (the `á` at position 1 decides), and so is `"a"&U+202F&"b" < "á"&" b"` (B46, B48; B47 FALSE).
  **Its accent weight is 10** (probes/t4153-narrow-nbsp-accent, excel-graph, 2026-11-04, T-4153):
  - U+202F is above `" "&U+030D` (2 + 3 = 5) and below the space with each other mark probed, U+0301 to U+0328
    (14 and up) (Calc!B1:D15, E1:E15 with an `a` after);
  - it is above two U+030D (8) and below three (11) (B30:D32);
  - it equals the space with 88 U+030D, whose sum wraps to 10 in the accent byte (C34), and is above 173 and 87
    of them (9 and 7) and below 89 (13) (B35:D37).

  Three of those texts sort below it: the space, U+3000 and `" "&U+030D` (B20; B21 puts U+3000 below
  `" "&U+030D`). With a mark of its own, U+202F&U+0301
  is above `" "&U+0301` (B24:B25). Implemented in: `element` gives U+202F the space's primary and the accent
  weight `NARROW_NBSP` = 10 (T-4087, T-4153).
- **A weightless character ends a letter, not a skipped character.** A mark after `-` or `'` and a weightless
  character still joins the hyphen or apostrophe. `"a-"&U+00AD&U+0301 = "a-"&U+0301` and
  `"a'"&U+00AD&U+0301 = "a'"&U+0301` are TRUE, with `<` and `>` both FALSE. With a `b` after it, the apostrophe's
  `=` is TRUE (B50:B52); for the hyphen, `=` follows from `<` and `>` both being FALSE (B16:B17) (Calc!B10:B12,
  B21:B22). Implemented in: `skipped` reads the marks after a weightless character into the hyphen's weight, and
  `accent_units` keeps `after_skipped` across it (T-4087).
- **Width and superscripts are weighed before small kana; kana type after.**
  - `"Ａ"&ぁ > "A"&あ`, `"¹"&ぁ > "1"&あ` and `U+3000&ぁ > " "&あ` are TRUE (Calc!B30:B33, B36, F2). So the first
    character's width or superscript decides over the second's small kana.
  - `"ア"&ぁ < "あ"&あ` is TRUE (B34): the small kana still decides over the kana type, as in T-3275's
    `"かぁ" < "カア"`.
  - `"Ａ"&"a" < "A"&"á"` is TRUE (B37): the accents still come first.
  - It is a separate, earlier pass, not one pass decided at the first difference. With the width at position 2
    and the small kana at position 1, `ぁ&"Ａ" > あ&"A"`, `ぁ&"¹" > あ&"1"` and `ぁ&U+3000 > あ&" "` are still
    TRUE (B40, B42, B44; B41, B43, B45 FALSE).
  - Implemented in: the variant byte holds a width or superscript below `KANA_TYPE` and a kana's type from it up.
    `widths` is a pass between the accents and `kana_weights`; `kana_types` comes after it (T-4087). This qualifies
    "Full-width and half-width forms" above ("Width is a tiebreak"). Half-width katakana stays a kana type, not a
    width: in the width pass it would sort `"ｱ"` before `"ぁ"`, but Excel has `"ぁ" < "ｱ"` (probes/t3275-collation-
    width-kana rank A99).
- **GROUPBY and PIVOTBY group by `=`, prefix rule included.** Over `ab`, `ab`&U+00AD, `é`, `é`&U+00AD,
  U+00AD&`ab` and `AB` (values 1, 2, 4, 8, 16, 32), GROUPBY gives three groups in this order (-groupby Calc!B1,
  B4:B6, D4:D6):
  - 49 = `ab` + U+00AD&`ab` + `AB`, keyed by its first spelling `ab` (EXACT, Calc!B11:B12);
  - 2 = `ab`&U+00AD;
  - 12 = `é` + `é`&U+00AD.

  PIVOTBY gives the same groups under its header row (B2, C4:C7). By the prefix rule (a leading weightless unit
  sends both texts to the collation; no `=` cell here), U+00AD&`ab` equals both `ab` and `ab`&U+00AD, and it joins
  the first group. Implemented in (T-4088, revised by T-4195): `Grouping::new` (functions/lookup_and_reference/
  grouping.rs); see "GROUPBY on an intransitive run (T-4195)" below.
  - Single-field keys also equate what `=` equates beyond case: `ß`/`ss`, `æ`/`ae` and `á`/`a`&U+0301 make three
    groups, `{"á",48;"æ",12;"ß",3}` (probes/t4088-groupby-multifield Calc!B14:B21, B39; excel-graph, 2026-11-04).
    Ours passes them.
  - **Multi-field keys on an intransitive run do not group field by field, nor quite by whole rows.** The rows are
    (`ab`,x), (`ab`&U+00AD,y), (U+00AD&`ab`,y), with values 1, 2, 4.
    - GROUPBY shows UNIQUE's two groups, `{"ab","x",1;"ab­","y",2}` (Calc!B1:B2, B30, B34).
    - The third row, which `=` joins to the second, is in no group: COUNTA counts 1, and the aggregated values are
      "2" (B31:B32). PIVOTBY is the same (B3, B38).
    - Reversed, the shown keys are `{"ab","x",4;"ab","y",1}` (B8:B12, B33): the `ab`&U+00AD row drops out, and the
      second group shows `ab`, which is not the first spelling of either field.
    - Each field alone groups by first `=` (B35:B36). The tree model below (T-4342) passes them all.

### GROUPBY on an intransitive run (T-4195)
Evidence: probes/t4195-groupby-intransitive (56 blocks of 2 to 4 rows: every order of `ab`, `ab`&U+00AD and
U+00AD&`ab`, with `AB` and `ab`&U+00AD&U+00AD, alone, beside a second field that is x in one row, before it,
and with a third field) and probes/t4195-groupby-shown-key (18 blocks: case, U+00AD, `ß`/`ss` and `á`/`a`&U+0301
spellings of one key in different groups); excel-graph, 2026-11-04. Each block's GROUPBY lists its groups' rows
with an ARRAYTOTEXT aggregate. Write A = `ab`, B = `ab`&U+00AD, C = U+00AD&`ab`: A = C and B = C, but A < B.
- **One field: the distinct keys are UNIQUE's, sorted, and a row joins the one a binary search finds.** The
  distinct keys are the first-`=` ones; they are sorted by `<`; each row joins the key a lower-bound binary
  search of that order finds equal to it. So over B, C, A (Calc!B4) the keys are B and A, sorted A, B, and C
  joins A, not its first equal key B: `{"ab","2, 3";"ab­","1"}`. All 56 single-field cells (B1:B56) and the
  4-text runs (B31:B44; E = `ab`&U+00AD&U+00AD: E, B, C, A gives `ab`: 3, 4 and E: 1, 2, B40) follow it. The groups are in that sorted
  order: `AB` before `ab`&U+00AD (B32: `AB` < `ab`&U+00AD by the prefix rule), where `cmp_text` puts it after.
- **A group shows its key's first spelling, field by field** (first in whole-row sorted order, which is not always
  source order: T-4342's hold-out below). (`ABC`,x), (`abc`,y) gives (`ABC`,x), (`ABC`,y);
  (`ab`,x), (U+00AD&`ab`,y) gives (`ab`,x), (`ab`,y); `ß`/`ss` and `á`/`a`&U+0301 the same (-shown-key B1:B18,
  D1:D18).
- **A later field is classed within its parent.** The rows sharing the classes of the fields before it are classed
  apart from the others: with x/y first and the text second, (x,C), (y,A), (y,B) gives (y,`ab`): 2 and
  (y,`ab`&U+00AD): 3 (B55; B51:B56 are the swap rows). And `abc` under a first key other than `ABC`'s shows `abc`
  (gen-m7/8044dce50616cfe9 Calc!C833, probes/t2360-array-lambda-holidays G!B6, probes/t2625-nested-consumers P!A30).
- **A group none of UNIQUE's rows is in is not shown.** Its rows are in no group: probes/t4088-groupby-multifield
  B1:B6, B30:B32 above, where the third row's (`ab`,y) holds none of UNIQUE's rows (`ab`,x) and (`ab`&U+00AD,y).
- **Excel goes further on a multi-field run.** Rows (A,y), (B,x), (C,y) give (`ab`,y): 1, 3; (`ab`&U+00AD,x): 2;
  (`ab`&U+00AD,y): 1, 2, 3 (Calc!C8). Row 2 is in two groups, and the third group's key is no row's. UNIQUE keeps
  all three rows (D8) though (C,y) = (A,y): it keeps or drops (C,y) by the rows around it, as its merge meets them
  (T-4200), and ours matches every D cell. PIVOTBY's rows are GROUPBY's (F).
- **Excel's rule on a multi-field run: a tree over UNIQUE's sorted keys** (T-4241, probes/t4241-groupby-intransitive-runs,
  excel-graph, 2026-11-05; 48 blocks: T-4195's 18 three-row blocks again, which repeat its values exactly, and 30
  seeded four-row runs, each shown ascending (B), with `sort_order` -1 (C), 2 (D) and -2 (E), with a SUM aggregate (F)
  and through UNIQUE (G)). The model in docs/research/t4241/model.py gives every group, member list and shown key
  of all 234 blocks of both probes that use only A, B and C (t4195 C and E, t4241 B to E):
  1. The keys are UNIQUE's kept rows (its merge, T-4200), sorted by SORT's natural merge on whole rows, each field
     in its sort direction (`sort_order` -1 reverses the first field, -2 the second).
  2. Classes are built field by field over that sorted list. Under one parent, a key joins the current class when
     its field `=` the field of the class's first key, else it starts a class. A class shows its first key's
     spelling, so a group can show a key tuple no row has (C8's (`ab`&U+00AD,y) is the key (C,y)). A class can
     repeat a shown key: Bx, Ay, Cy, Cx gives (C,x): 1, 4; (C,y): 2, 3; (C,x): 1, 2, 3, 4 (t4241 Calc!B25).
  3. Each data row descends the tree. At each field, a lower-bound binary search over the parent's class keys
     finds a class, searching [0, n-1] (it never probes past the last class; T-4342, below), in the direction the
     parent's first two classes run in (the field's, unless the merge left them the other way round; T-4344,
     below). The row joins it when that key `=` the row's field. When the key is not equal the row is in no
     group: Cy, Ax, By gives (`ab`,x): 2; (`ab`,y): 1, and row 3 (B = C) is in none (t4241 Calc!B14).
  4. A leaf class no row reaches aggregates every row of the source. That is the group with all the rows in
     Calc!C8 and t4241 B25, and SUM counts such a row once in each of its groups (t4241 Calc!F2: (`ab`,y) 5,
     (`ab`&U+00AD,x) 2, (`ab`&U+00AD,y) 7 of rows worth 1, 2 and 4).
  `sort_order` 2 gave the ascending result on all 48 blocks (D = B). A table sort (`field_relationship` 1) and a sort
  by a value column on these runs: T-4344, below.
- **The hold-out (T-4342, probes/t4342-groupby-tree-holdout, excel-graph, 2026-11-05).** Written with the T-4241
  model's predictions committed before the fill (docs/research/t4342: gen_probe.py, predictions.tsv, compare.py):
  30 seeded five-row runs of two fields and 20 of three (text, x/y, text), each ascending (B), with `sort_order` -1
  (C), as SUM (D) and as PIVOTBY by a p/q column field (E); and 6 ASCII case-spelling blocks. 206 cells:
  - The frozen model scores 199. Six misses are one shape (t4342 Calc!B14, D14, E14, B48, D48, E48): two leaves
    show one key, and Excel puts the rows in the first of them where the half-open search over [0, n) finds the
    second. A search over [0, n-1] (`model.groupby(..., inclusive=True)`) gives all six and still all 234 blocks
    of t4195 and t4241, so it is step 3's search: 205 of 206.
  - The miss left is Calc!C40, three fields with `sort_order` -1: Excel puts row 1, (B,y,A), alone in (B,y,A),
    where the search finds class A first and the row in no group (T-4344 explains it, below).
  - PIVOTBY's rows are GROUPBY's groups on every block: a cell is its row group's rows in that column, and the
    group no row reaches has every row in each column (E).
  - **The shown spelling is the first in sorted order, not in source order, even in ASCII**: (`ABC`,y), (`abc`,x)
    gives (`abc`,x): 2; (`abc`,y): 1 (Calc!B51), because (`abc`,x) sorts first on the whole row; (`abc`,y),
    (`Abc`,x), (`ABC`,y), (`abc`,x) shows `Abc` (B56). With one field, or while the first spelling also sorts first,
    that is the first spelling (`East` for `East`, `east`, G!A1; -shown-key).
- **The search runs in the direction of the first two classes (T-4344, probes/t4344-groupby-descending-search,
  excel-graph, 2026-11-05).** On an intransitive run SORT's merge can leave a descending field's classes ascending:
  t4342 Calc!C40's first field has classes A, B under `sort_order` -1, and Excel's search finds B for row 1, which
  a search in the field's direction (descending) does not. Five search directions fit or nearly fit the 434 earlier
  blocks (docs/research/t4344/gen_probe.py: H0 the field's, 433; H1 first class to last, 434; H2 the same with
  ascending on a tie, 433; H3 the first two classes, 434; H4 the first class to the first later one not `=` to it,
  434, which is H3, since two adjacent classes are never `=`). The probe was written with each one's predictions
  and the engine's (hypotheses.tsv, predictions.tsv) committed before the fill, on 30 seeded five-row three-field
  runs with `sort_order` -1, -2, -3 or {-1,-3} picked where they disagree: H3 gives all 30 (H0 7, H1 15, H2 16).
  Of 4000 seeded five-row two-field runs under -1 or -2 none tells them apart. A structural choice confirmed out of
  sample (decision 0188 R4), not a fit.
  - The same probe's ASCII case-spelling blocks under `sort_order` -1 and -2 (Calc!B43:B54), PIVOTBY by x/y with
    the text as the column field ascending and `col_sort_order` -1 (B31:C42), and GROUPBY with `field_relationship`
    1 ascending and -1 (B55:C66) all match the engine, as do 22 of the 24 value sorts.
  - **A PIVOTBY column leaf no record of a row group reaches holds that row group's records** (D39: p `2`, q
    `1, 3, 4, 5`), and whether a record reaches a column leaf depends on its row group: in E38 rows 2 and 3, both
    (C,y), are p and q, and (A,y) holds all of p but only row 3 of q; in D41 the only q record with y reaches (B,y)
    in the column tree, yet q's (B,y) cell is all of q. Each of the four is the column tree's leaf no record
    reaches, holding every record of the row group as GROUPBY's such group holds every row (step 4): E38's q is
    only row 3, and D41's (B,y) is reached by no record, row 4 finding class A (C = A) and then no y leaf.
  - **The hold-out (T-4358, probes/t4358-pivotby-unreached-column-leaf, excel-graph, 2026-11-05).** 25 seeded
    five-row runs whose column tree has a leaf no record reaches, by p/q rows: ascending (B), `col_sort_order` -1
    (C), with column subtotals (D) and with x/y as the first column field (E), written with the engine's
    predictions committed before the fill (docs/research/t4358). The leaf rule gives 74 of the 75 B, C and E
    cells and every leaf cell of D. Two findings it does not cover:
    - **A first-field subtotal column holds the row group's records that reached its class**, those that reached
      no leaf under it included, and not the extra records of a leaf no record reaches: over Bx Cy Cy Ax By of
      q q q p p the `ab` subtotal of q is `2, 3` while its leaf (`ab`,x) holds nothing for q (D1); over Ax Cx Bx
      Cy Bx the `ab`&U+00AD subtotal of p is `3, 5` while its unreached (`ab`&U+00AD,y) holds `1, 3, 5` (D10).
      That rule gives 88 of 88 subtotal cells (docs/research/t4358/score.py), chosen on them, and all 75 cells of
      its hold-out, probes/t4378-pivotby-subtotal-holdout (25 fresh seeded runs with column subtotals ascending,
      `col_sort_order` -1 and as SUM, predictions committed before the fill; docs/research/t4378; excel-graph,
      2026-11-05). A structural choice confirmed out of sample (decision 0188 R4), not a fit. Implemented (T-4378):
      `Tree::descend` returns the classes a row reached, `Grouping::reached_classes` keeps them, and the
      `Slot::Subtotal` arm of `Grid::cell_rows` (pivotby.rs) takes the records whose classes start with its prefix.
    - **Excel can leave such a leaf out**: over Cx Ay Ax Bx Ax (q q p p q) the columns are (`­ab`,x), (`­ab`,y)
      only, where t4344 D39's runs of the same key tree show the repeated (C,x) (B6, D6; T-4379). The cause is
      UNIQUE, below.
  - **UNIQUE drops Bx after Cx Ay Ax** (T-4379, probes/t4379-pivotby-repeated-unreached-leaf, excel-graph,
    2026-11-05; 15 runs of B6's keys through UNIQUE (B), GROUPBY (C), PIVOTBY by p/q (D), by one row group (E) and
    with `col_sort_order` -1 (F), predictions in docs/research/t4379 committed before the fill). Over Cx Ay Ax Bx
    UNIQUE is `{"­ab","x";"ab","y"}`: Bx, which `=` Cx only, is dropped, where T-4200's merge keeps it. GROUPBY and
    PIVOTBY build their trees on those two keys, so no third (C,x) group or column exists: the leaf is not left
    out, it was never built. The same holds with a fifth row Ax, Cx or Bx after, and over Cx Ay Ax Ax Bx (rows 1,
    2, 8, 9, 10). Bx is kept over Cx Ay Bx, Cx Ay Bx Ax (Bx before the Ax), Cx Ax Ay Bx Ax (the Ax before the Ay),
    Cx Cx Bx Ay Bx and Cx Ax Cx Ay Bx, as the engine keeps it; every F cell (`col_sort_order` -1) matches. T-4393
    owns the 20 cells and t4358 B6, D6 (from T-4380, whose rule failed its hold-out, below).
  - **No coalescing merge explains UNIQUE's two-field runs** (T-4380, probes/t4380-unique-keep-head-holdout,
    excel-graph, 2026-11-05; 45 runs through UNIQUE (B) and GROUPBY (C), predictions in docs/research/t4380 committed
    before the fill). The rule chosen on t4379 (equal run heads: the right group joins the left head, which stays the
    head) fits t4379 but misses 12 of the 45 hold-out runs: over Cy Cx Ay Bx Excel keeps Cy, Cx and Bx (as T-4200's
    rule does), where the rule merges Bx into Cx. Pushing the joined head out and keeping it (`pushkeep`) misses 5;
    T-4200's rule misses 28; the right group taking the head misses more; insertion sort then grouping neighbours
    (MSVC's short `stable_sort`) misses 37 to 44. Over Cy Cx Ay Bx Ay (rows 6, 27) Excel's UNIQUE is Cy, Cx, Bx, Ay: the
    second Ay is kept apart from the first, which joined Cy, so two identical rows end in different groups. No
    sort-then-group-neighbours model under the row-by-row field comparison can do that. One-field runs never tell
    the rules apart (none up to length 7). T-4393 owns the search and these cells.
  - **A value sort's parent counts the rows that reached it but no leaf**: Bx, Ax, Ax, By, Cy by SUM of 1, 2, 4, 8,
    16 orders ascending (B,x) 1, (B,y) 8, (A,x) 6 (D56), B's 9 before A's 22, where 22 counts row 5, (C,y), which
    reaches class A (C = A) and then no y class (D56, E56).
  - **A GROUPBY parent holds the rows that reached its class, and every row when none did** (T-4359,
    probes/t4359-groupby-value-sort-parents and probes/t4359-groupby-value-sort-holdout, excel-graph, 2026-11-05):
    a value sort orders the parents by the function of those rows, a row that reached no leaf under the class
    included and the extra rows of a leaf no row reaches not, and a subtotal line shows the function of the same
    rows, under a value sort and a key sort alike. A class no row reached (a repeated class that ends the list,
    whose leaves no row reaches) holds every row, as such a leaf does: over Ax Bx Cy Bx Ay by SUM of 1 to 16 the
    subtotals are `ab`&U+00AD 10 (rows 2, 4), `ab` 21 (rows 1, 3, 5: row 3 reaches A and no y class under it) and
    the second `ab` 31, and the parents sort in that order (Calc!B4, E4).
    - The first probe (28 seeded five-row runs, 14 of two fields and 14 of three, SUM ascending and descending,
      AVERAGE, and SUM with subtotals at every level) was written with three hypotheses' predictions committed
      before the fill (docs/research/t4359/model.py: the rows of the parent's groups, R0, 24 of 84; the rows that
      reached it, R1, 52; both, R2, 65). Its subtotal cells showed the rule, R3 (R1, every row for a class none
      reached), 84 of 84, chosen after the fill. The hold-out (30 fresh runs, 12 of two fields and 18 of three,
      seed "T-4359 hold-out") was written with the engine's R3 predictions committed before its fill: 120 of 120
      value-sort cells (Calc!B:E). Its column F, a key sort with subtotals, predicted with the union of the groups'
      rows, gave 0 of 30; R3, frozen before the fill, gives 30 of 30. A structural choice confirmed out of sample
      (decision 0188 R4), not a fit.
  - **PIVOTBY's other paths on intransitive runs (T-4382)** (probes/t4382-pivotby-intransitive-paths, excel-graph,
    2026-11-05; 42 seeded five-row runs, the engine's predictions committed before the fill in docs/research/t4382;
    63 of 129 cells matched them). Findings, scored by docs/research/t4382/score.py (214 of 214 first-field
    subtotal and parent cells under the rule below, chosen on them, so it owes a hold-out: T-4401, T-4402):
    - **Row subtotals follow the column rule**: a subtotal row holds the records that reached its row class, those
      that reached no leaf under it included. Over Bx Cx Ax By Cy (q p p q q), rows the text and x/y, the `ab`
      subtotal row is p `2, 3`, q `5`, Total `2, 3, 5`: row 5, (C,y), reaches class `ab` and then no y leaf (Calc!B16;
      every subtotal row of B16:C30, ascending and `row_sort_order` -1), as GROUPBY's subtotal lines do (T-4359,
      above), whose change gives every one of these cells.
    - **A subtotal whose class no record reached holds every record of the line**, as such a leaf does (step 4):
      with three column fields and `col_total_depth` 3, over BxC BxB CyA AxB AxB (p q p q p) the level-2 subtotal
      (`ab`&U+00AD,y), which no record reaches, is p `1, 3, 5`, q `2, 4` (Calc!B31; B31:C42), as GROUPBY's class
      none reached does (T-4359). Confirmed on a hold-out of 30 fresh runs with three and four column fields
      (`col_total_depth` 3 and 4, ascending and `col_sort_order` -1; probes/t4401-pivotby-level-subtotal-holdout,
      excel-graph, 2026-11-05, 60 of 60 committed predictions matched). `Grouping::subtotal_reached` (T-4401).
    - **relative_to 3 and 4 take the parent subtotal's records by the same rule**: over Ay Ay Bx Cx Cy (q p p q q)
      the parent column total of q's (`ab`,y) cell is `1, 4, 5`, not `1, 5` (Calc!D2, a two-argument LAMBDA
      `ARRAYTOTEXT(x)&" / "&ARRAYTOTEXT(a)`); parent row totals read the parent subtotal row's cell in the same
      column, Total column included (E17). **An empty parent set is an error**: PERCENTOF `#DIV/0!` and the
      two-argument LAMBDA's cell `#VALUE!` (C7, D7: a column leaf no record reaches under an empty subtotal).
      T-4402 owns the cells.
    - **The subtotal error gate is the leaf buckets'**: with one function over two value columns, a subtotal
      whose records reached its class while every leaf under it is empty for the row group is not computed (its
      NA() makes no `#VALUE!`; Calc!B1, B3, B4 and the other `!` targets of B1:B15), as `Grid::has_records`
      decides. The three B misses (B2, B5, B14) are the
      moved subtotal cell of that layout showing `""` where Excel shows the reached-class records (`12` and
      `11, 14, 15` in B2), an engine defect on a tree with a leaf no record reaches (T-4403).

Implemented in (T-4342): `Grouping::new` (functions/lookup_and_reference/grouping.rs) takes UNIQUE's kept rows
(`unique_first_lines`, shared with `unique_of`), sorts them with `Keys::sorted` (`sort_indices_pairwise` on whole
rows in the directions `tree_directions` gives: a hierarchy's, ascending for a table or a value sort), builds the
classes with `Tree::build` and descends each row with `Tree::descend`; a leaf no row reaches is a group of every
row (`Group::reached` false, which `Grouping::group_of` maps only rows in no other group to). Groups come out in
tree order and sort by their class's rank (`Group::key_order`). While no key text is outside ASCII a row descends as
its first equal row does, so the search runs once per distinct key. `Tree::descend` searches in the direction of
the parent's first two classes (T-4344). PIVOTBY's column leaf no record reaches holds every record of the row line
(`Grouping::unreached`, the bucket fill of `grid_row` in pivotby.rs; T-4358). A column subtotal holds the records that
reached its class (T-4378). A GROUPBY (or PIVOTBY row) parent's rows, which a value sort orders by and a subtotal line
shows, are `Grouping::class_rows`: the rows that reached the class, every row when none did (T-4359; PIVOTBY's row
axis and descending key sorts unprobed, T-4388). Not implemented: UNIQUE's drop of Bx after Cx Ay Ax, which removes a
repeated leaf (T-4380) on these runs.
The T-4195 per-field model this replaces was selected on its own cells without a hold-out; the tree model was
chosen by T-4241 on seen values and confirmed by T-4342's hold-out (205 of 206 after the search revision, which
was chosen on the hold-out and checked against all 234 earlier blocks; a structural choice, decision 0188 R4,
not a fit).
Status: VERIFIED (excel-graph, 2026-11-04; the t4241, t4342, t4344, t4358, t4378 and t4359 fills, 2026-11-05) for the cells cited.

Implemented in (T-3904, T-4039, one PR, since the -ignorables Calc!AB ranks need both):
- the collation's `elements` (crates/rustsheet-engine/src/functions/util/collation.rs) drops every weightless
  character of `lookup_fold::is_weightless` (the one T-3584 table exact MATCH uses), so no pass weighs it and the
  [`CollationKey`] leaves it out;
- `cmp_prefix` applies the prefix rule before the collation in `cmp_text_pairwise` only (Excel's `<`, `=`, `>`
  and the text criteria; XMATCH's index confirms key matches with it), since the rule is not transitive;
- `element` weighs U+205F as the space, U+202F at `NARROW_NBSP`, U+2000 to U+200B at `SPACE_FORMS` and the C1
  controls at `C1_BASE`.
Status: VERIFIED (excel-graph, 2026-11-04) for the cells cited.

## Text in a range operand of arithmetic is coerced
Rule: arithmetic on a range coerces a numeric text cell to a number, so `Data!D7:D7/1` with
D7 = " 2 " (spaces around the digit) is 2 and `Data!D3:D3/0` with the same text is `#DIV/0!`. Every
observed number-producing text cell is " 2 ". That this is the same routine as scalar text
coercion is an inference for T-0131's probe to confirm.
Evidence: gen-m1/1ce5b2467b3b19fd Calc!A14, A79, A99; gen-m1/9646257ac139cd1e Calc!A25, A81 (oracle:
excel-graph, 2026-10-05, T-0109). The scalar form `-" 2 "*2` = -4 (gen-m1/10f579127ffb6d9b A106).
The same holds for a one-cell intersection of a whole row: `('Q''1'!5:5 'Q''1'!B5)+2` with B5 = " 2 "
is 4 (gen-m1/7cb99acdd82402f1 Calc!A72, A103, A217, A235; found in T-0105).
Status: VERIFIED
Implemented in: `Model::array_node_to_number` (crates/rustsheet-engine/src/cast.rs), via `cast_number`; only the " 2 " form is oracle-covered, other text forms follow the scalar path.
Tasks: T-0131

## An error in a range operand of arithmetic wins over the right operand's error
Rule: when the left operand is a one-cell range holding an error and the right operand is an error,
the result is the left error: `(Data!C1:C10 Data!B9:D9)/Data!D$5` with C9 = `#N/A` and
D5 = `#DIV/0!` is `#N/A`. `handle_arithmetic` lifts a scalar right-hand error to a 1x1 array, so the
left range's element error is seen first (`arithmetic_on_values`, case 4).
The same holds element by element for a multi-cell range. Each element shows its own error if it has
one, and otherwise the scalar's error. A text element fails with `#VALUE!` before the scalar's error.
With C8 = 7, C9 = `#N/A`, C10 = "x" and D5 = `#DIV/0!`: `=Data!C8:C9/Data!D5` is {`#DIV/0!`; `#N/A`},
`=Data!C9:C10/Data!D5` and `=Data!C9:C10+Data!D5` are {`#N/A`; `#VALUE!`}, and a scalar left error wins
over every element (`=Data!D5/Data!C8:C9` is `#DIV/0!` twice).
Evidence: gen-m1/4dc8ed1de05a61be Calc!A250 (oracle: excel-graph, 2026-10-05; found in T-0105);
probes/arith-range-error-precedence (oracle: excel-graph, 2026-10-05, T-0138).
Status: VERIFIED
Implemented in: `Model::handle_arithmetic` (crates/rustsheet-engine/src/arithmetic.rs)
Tasks: T-0137, T-0138

## Text longer than 32,767 characters
Rule: concatenation does not fail past Excel's 32,767-character cell limit, it truncates:
`LEN(CONCATENATE(REPT("a",32767),"b"))` and `LEN(REPT("a",32767)&"b")` are both 32767, and the
kept text is the leading part (`RIGHT(CONCATENATE(REPT("a",32767),"b"),1)` is "a"). REPT with a
longer result is `#VALUE!` (`REPT("a",32768)`). The limit counts UTF-16 code units, so an emoji
(a surrogate pair) counts two: `REPT("😀",16383)` is fine, `REPT("😀",16384)` is `#VALUE!`, and
`LEN(CONCATENATE(REPT("😀",16383),"ab"))` is 32767. Truncation applies per element of a spilled
CONCATENATE (`LEN(CONCATENATE({"a";"bb"},REPT("c",32767)))` is {32767;32767}).
Evidence: probes/resource-limits C9–C11; probes/concatenate-limit-spill E1–E7, E12, G85–G104
(excel-graph, 2026-10-06, T-0233). LEN counts UTF-16 units too (text.md "Text positions count
UTF-16 code units", T-0249).
Backend: excel-graph only; excel-desktop not used.
Status: VERIFIED
Implemented in: `Text::concat`/`Text::truncate_utf16` (text.rs), shared by CONCATENATE and `&` (T-0240, T-1580);
REPT's limit in `rept_element` (functions/text/common.rs); both count UTF-16 units (T-0233).
CONCAT/TEXTJOIN give `#CALC!` (probes/text-result-limit C8-C9, not implemented).
A 65,534-unit operand truncates the same way (text.md "SUBSTITUTE result longer than 32,767 characters", probes/text-long-input-concat-*, T-1913).
Tasks: T-0240, T-0233, T-0249

## Truncation inside a surrogate pair keeps a lone high surrogate
Rule: when the 32,767-unit limit falls inside a surrogate pair, Excel keeps the high surrogate:
`LEN(CONCATENATE(REPT("a",32766),"😀"))` and `LEN(REPT("😀",16383)&"😀")` are 32767, and
`UNICODE(RIGHT(CONCATENATE(REPT("a",32766),"😀"),1))` is `#VALUE!` (UNICODE of a lone surrogate).
Evidence: probes/concatenate-limit-spill E8–E11 (excel-graph, 2026-10-06, T-0233).
Ours: `&` and CONCATENATE truncate with `Text::truncate_utf16`, which keeps the high half of a cut
pair (E8, E9, E11 pass since T-1580). Decision 0033 (drop the pair whole) is superseded by
decision 0052. E10 (UNICODE of the lone half) passes since T-1581. SPEC-0010.
Arguments after the cut change nothing but an error still wins (probes/concatenate-after-cut, excel-graph,
2026-10-15, T-1612; the engine matches all nine cells): `LEN(CONCATENATE(REPT("a",32766),"😀","zz",1))` and
`LEN(CONCATENATE(REPT("a",32767),"zz",1,TRUE))` are 32767 (A1, A4), and at a character boundary the kept text still ends in "a" (A9); an error
argument after the cut is the result (`#N/A` for `...,"😀",NA())` and `...,"zz",NA())`). Among several errors the first argument's wins, wherever the cut falls: `...,1/0,NA())` is `#DIV/0!` (A6), while `...,NA(),1/0)` and `CONCATENATE(NA(),REPT("a",32767),1/0)` are `#N/A` (A7, A8). `ISERROR` of the first error case is TRUE (A10).
Status: VERIFIED
Implemented in: `concat_limited` in arithmetic.rs (`&`), `fn_concatenate` in
functions/text/concatenate.rs, `Text::truncate_utf16` in text.rs
Tasks: T-0233, T-1483, T-1580

## SUMX2MY2 / SUMX2PY2 / SUMXMY2 pairing
Rule: the two arrays are paired element by element in row-major order and need only the same
element count, not the same shape (a 3x2 range pairs with a 6x1 range); a pair where either side
is not a number (blank, text, boolean in a range) is skipped; different element counts are `#N/A`;
scalar arguments are allowed (`SUMXMY2(2,4)` = 4). The checks run in this order (T-0961,
probes/sumx-blank-text-error-order, excel-graph, 2026-10-20; all three functions alike):
1. Each argument in turn, the first one first. An error value, or a one-cell reference (`E2`, `E2:E2`) holding an
   error, is that error at once, whatever the other argument is (`SUMXMY2(1/0,C1:C3)` and `SUMXMY2(E2,C1:C3)` are
   `#DIV/0!`, not the count's `#N/A`; `SUMXMY2(1/0,NA())` `#DIV/0!`). A blank one-cell reference (`A4`, `A4:A4`,
   `IF(TRUE,A4)`) or an omitted argument (`SUMXMY2(,1)`, `SUMXMY2(1,)`) is `#VALUE!` at once: `SUMXMY2(A4,NA())` is
   `#VALUE!` but `SUMXMY2(NA(),A4)` `#N/A`, and `SUMXMY2(E1:E2,A4)` (E2 `#DIV/0!`) and `SUMXMY2(1/{1,0},A4)` are `#VALUE!`.
2. Different element counts are `#N/A`, ahead of an error inside a range or an array of either argument
   (`SUMXMY2(C1:C3,E1:E4)`, `SUMXMY2(C1:C3,1/{1,0})`, `SUMXMY2(5,1/{1,0})`).
3. Then such an error (`SUMXMY2({1,2},1/{1,0})` and `SUMXMY2(E1:E3,C1:C3)` are `#DIV/0!`). The first argument's error
   comes first, wherever it sits: `SUMXMY2({1,#N/A,3},1/{0,1,1})` is `#N/A`, `SUMXMY2(E1:E3,{1,2,#N/A})` `#DIV/0!`.
4. Then `#DIV/0!` when either argument holds no number at all (`SUMXMY2(A2:A3,B1:B2)` with A2:A3 text and TRUE,
   `SUMXMY2(C1:C2,{"a","b"})`, a one-cell text or boolean reference or a typed text or boolean against a number).
   Otherwise the result is the sum over the pairs where both sides are numbers, and it is 0 when no such pair exists:
   `SUMXMY2(A1:A2,{"a",5})` and `SUMXMY2({1,"a"},{"b",2})` are 0.
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!D1336, E1336,
G1336, I1336, J1336 (`#N/A`), K1336, E1340, F1340, H1340, J1340 (`SUMX2PY2(G7:H9,J7:J12)` =
105.7931362501), K1340, F1344, G1344, H1344 (`#N/A`), I1344, J1344.
Status: VERIFIED (Excel-cached; excel-graph for the order of checks)
Implemented in: `Model::sumx_fold` / `sumx_arg` (math_and_trigonometry/sum.rs); other `fn_get_two_matrices` callers are unchanged.
Scalar text and boolean arguments are one skipped pair: `SUMX2PY2("1",2)`, `SUMXMY2("1",2)`, `SUMX2MY2("1",2)` and the same
with `TRUE` are `#DIV/0!` (numeric text is not read as a number here) while `SUMX2PY2(1,2)` is 5; a text element inside an
array constant is skipped (`SUMX2PY2({1,"a"},{3,4})` = 10); two all-blank ranges are `#DIV/0!`. Evidence:
probes/stats-evidence-floor-t1623 Q73-Q76, Q79, Q86-Q89, Q91, Q99-Q102, Q104 (excel-graph, 2026-10-17, T-1623).
More evidence, gen-m5 (excel-graph, 2026-10-20, T-2114; all 28 cells listed in T-0961): a blank one-cell argument
(`B12`, `C8:C8`, `A$1`, or `IF(..,E5)` returning a blank cell) is `#VALUE!` whatever the other argument is: a range of another
size (`SUMXMY2(B12,D1:E8)`, gen-m5/043ee562f32eb901 Calc!A36), a number (`SUMX2MY2(YEAR(50111),D1)`, gen-m5/eb942f4a59b1c004 Calc!A231), a
text cell (`SUMXMY2(B$8,E12)` with E12 "TRUE", gen-m5/f0f205865a3a4328 Calc!A103), a boolean, or an error (`SUMX2PY2(C$4,<#NUM!>)`,
gen-m5/1962de140dae0111 Calc!A388; gen-m5/ee1667411bbe2ccc Calc!A84). A number or a range against an array of another count is `#N/A`
(`SUMX2PY2(2.5,<6-element array>)`, gen-m5/204b68ebb5812d50 Calc!A218; gen-m5/aac6230c2fb40e1e Calc!A11).
Two non-blank ranges of equal count with no numeric pair are 0, not `#DIV/0!`: `SUMX2MY2(A8:D9,C6:D9)` (gen-m5/8a387c26d182e73c
Calc!A307) and `SUMX2MY2(C10:D12,C11:E12)` (gen-m5/d1b251fd6e9c4b4b Calc!A397), whose pairs hold text, booleans and blanks only
(two all-blank ranges stay `#DIV/0!`, above): each side holds a number, so rule 4 gives 0. Implemented in T-0961.

## SUMPRODUCT: a blank one-cell argument beside other arguments is `#VALUE!` (T-2150)
Rule: SUMPRODUCT is `#VALUE!` when one argument is a blank cell given as a one-cell reference (alone, or beside single values or ranges, see the probe below), e.g. `SUMPRODUCT(E6,123456789.123)` (E6 blank; `ISERR` of it is TRUE), `SUMPRODUCT(41.42,B7:B7,3.64)`,
`SUMPRODUCT(<number>,D4:D4)`, `SUMPRODUCT(A6:A6,B4,0)` (A6 FALSE, B4 blank), `SUMPRODUCT(B1,C12,D10)` (D10 blank) and
`SUMPRODUCT(A11:A11,1.005,C11:C11)` (A11 text, C11 blank), and the `#VALUE!` reaches the caller (`SHEET(7+SUMPRODUCT(E1,100,-1))`
is `#VALUE!`, not SHEET's `#N/A`). A lone one-cell boolean is 0 (`SUMPRODUCT(B4:B4)` with B4 TRUE, gen-m5/3e134e52393061b1 Calc!A222); the probe below shows a lone blank is `#VALUE!`.
An error argument beats a shape mismatch earlier in the list: `SUMPRODUCT(TYPE(B11:B11),B9:E10,<VLOOKUP #N/A>)` is `#N/A`
(ours was `#VALUE!` before T-2150, as was `SUMPRODUCT(1,B9:E10,NA())`).
Ours (since T-2150): every argument is read in order, and the first whole-argument error or blank one-cell reference
(a cell or a one-cell range such as `D4:D4`) decides the result, even after a shape mismatch; an error element inside a
range does not decide it before a later whole-argument error or blank (see the T-2296 paragraph below, T-2450).
Evidence (excel-graph, 2026-10-20, T-2114): gen-m5/1b30e4d075b252ea Calc!A365, gen-m5/6a8bb7cd6d7769ad Calc!A79,
gen-m5/aac6230c2fb40e1e Calc!A79, gen-m5/fc81c2f704bef138 Calc!A165, gen-m5/c688ca172fe533dc Calc!A183, gen-m5/7d1a5d223624fc70 Calc!A33,
gen-m5/aac6230c2fb40e1e Calc!A64, gen-m5/f0f205865a3a4328 Calc!A396, gen-m5/399071ff6782ab16 Calc!A145.
Probed (excel-graph, 2026-10-21, T-2200, probes/t2200-blank-and-padded-m5-args, Calc!A1:A27): a blank one-cell reference is
`#VALUE!` beside a 2x2 range in either order, beside a number reference in either order, in all three positions of a
three-argument list, and alone (`SUMPRODUCT(Data!A1)`, A6). A one-cell text or boolean
reference is a 0 factor beside a number (0, either order) and alone (0), but `#VALUE!` beside a 2x2 range (a 1x1 against a 2x2
is a shape mismatch). A blank plus a shape mismatch is `#VALUE!` in every position (the cells all hold the blank, so they do not separate the two causes). The blank was a never-written cell written as a cell reference; a one-cell range to a blank is `#VALUE!` too (gen-m5
`SUMPRODUCT(<number>,D4:D4)` fc81c2f704bef138 A165, `SUMPRODUCT(A11:A11,1.005,C11:C11)` aac6230c2fb40e1e A64). An error argument ahead of the blank wins
(`SUMPRODUCT(NA(),blank)`, `SUMPRODUCT(NA(),R,blank)` are `#N/A`); the blank ahead of the error gives `#VALUE!`.
Status: VERIFIED (excel-graph); implemented in `fn_sumproduct` / `sumproduct_array` (T-2150).
Probed (excel-graph, 2026-10-23, T-2296, probes/t2296-sumproduct-error-element, Calc!A1:A16): an error *element* inside a range or array
beats a shape mismatch wherever it sits: `SUMPRODUCT(X,R)` and `SUMPRODUCT(R,X)` (X 2x1 numbers, R 2x2 with a `#DIV/0!` element) are `#DIV/0!`,
`SUMPRODUCT(X,{1,#N/A;2,3})` and `SUMPRODUCT({1,#N/A;2,3},X)` are `#N/A`, `SUMPRODUCT(X,N,R)` (N a 2x2 of numbers) is `#DIV/0!`, and
`SUMPRODUCT(B1:C4,{1,#N/A,3})` and `SUMPRODUCT({1,#N/A,3},B1:C4)` are `#N/A` (gen-m7/f7e94a817e5385ac Calc!A50 agrees). A mismatch with no error
anywhere is `#VALUE!` (`SUMPRODUCT(B1:C4,{1,2,3})`). A whole-argument error beats an error element: `SUMPRODUCT(R,X,NA())`
and `SUMPRODUCT(X,R,NA())` are `#N/A`, even with the element error ahead of it. Status: VERIFIED (excel-graph). Implemented in
`fn_sumproduct` (T-2450): every argument is read, a whole-argument error returns at once, and the first element error in argument then element
order beats a mismatch (which of several element errors wins is unprobed). A blank one-cell argument also returns at once, so it beats an element error in any position (`SUMPRODUCT(R,blank)` is `#VALUE!`): VERIFIED (excel-graph, probes/t2296-sumproduct-error-element A17-A20): `SUMPRODUCT(R1:S2,blank)`, `SUMPRODUCT(blank,R1:S2)`, `SUMPRODUCT({#N/A},blank)` and `SUMPRODUCT({1,#N/A;2,3},X1:X2,blank)` are all `#VALUE!`, in either order.
Numeric text in a one-cell argument is a 0 factor too, like non-numeric text: with a one-cell reference to the text "4",
" 4", "1.5", "1E3", "$4" or a formula's `="4"`, `SUMPRODUCT(t,n)`, `SUMPRODUCT(n,t)`, `SUMPRODUCT(t:t,n:n)`, `SUMPRODUCT(t,2)`,
`SUMPRODUCT(t,{2})`, `SUMPRODUCT(t)` and `SUMPRODUCT(t,n,m)` are 0, and so are two-cell text ranges beside numbers
(probes/t2921-sumproduct-one-cell-numeric-text A1-A46, excel-graph, 2026-10-27, T-2921; the engine already matched).
CONFLICTING (decision 0030 2: Excel 15 is below AppVersion 16; decision 0039: the web value is the Phase 1-3 target):
poi/FormulaEvalTestData_Copy.xlsx (AppVersion 15.0300, Excel 2013) caches `#VALUE!` for EverythingTests!J1328
`SUMPRODUCT(G1330:G1330,I1330:I1330)` (G1330 the text "4", I1330 the number 2), where Microsoft 365 gives 0 on the same shape
(probe A45). The engine follows M365, and J1328 stays a known failure until a 16.x desktop value decides (T-1075, M14).
Status: VERIFIED (excel-graph); CONFLICTING with the Excel 15 cached J1328.
Tasks: T-0944, T-2150, T-2296, T-2450, T-2573, T-2921

## STDEV / VAR / DEVSQ / AVEDEV / SUMSQ typed arguments; AVEDEV and DEVSQ of nothing is #NUM!
Rule: numeric text and booleans typed directly as arguments are counted (`STDEV("1","2")` =
0.7071…, `VAR(TRUE,FALSE,TRUE)` = 0.333…, `SUMSQ("2","3",4)` = 29, `AVEDEV(TRUE,FALSE)` = 0.5),
while text and booleans in ranges are ignored. With no number at all AVEDEV and DEVSQ return
`#NUM!`, whereas STDEV/VAR/VARP return `#DIV/0!` (inconsistent, but that is Excel).
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!G1288, K1288,
G1448, K1448, G1456, K1456, I392, L176, M1332, N1332; `#NUM!`: F176, G176, H392; `#DIV/0!`: F1288,
F1448, F1456, H1288.
Status: VERIFIED (Excel-cached)
Implemented in: `Model::for_each_value` users `fn_stdev_s`/`fn_stdev_p`, `fn_var_s`/`fn_var_p`,
`fn_devsq`, `fn_avedev`, `fn_sumsq`. I392 (`DEVSQ("1","2",2)`) and E392/F392/J392
pass since DEVSQ became the two-pass sum (statistics.md "DEVSQ is a two-pass sum (T-1749)"). Open: K1456
(`VARP(TRUE,FALSE,TRUE)`) differs in the last digit (accumulation order).
An empty argument is counted as 0 by DEVSQ and AVEDEV (`DEVSQ(1,)` = `AVEDEV(1,)` = 0.5; probes/stat-order-floor-batch
A274, A299; T-1748) and by STDEV/VAR (probes/average-value-args); SUMSQ(1,) is 1 either way, T-1748.
Tasks: T-0945

## MODE: a blank single-cell reference is #VALUE!; typed text and booleans are skipped
Rule: MODE, MODE.SNGL and MODE.MULT skip typed (direct) text, numeric text included, and typed booleans
(`MODE(2,"x",2)` is 2, `MODE(1,"1",1)` 1, `MODE("2","2",5)` and `MODE(TRUE,TRUE,5)` `#N/A`), and a
reference to a text cell (`MODE(K1,K2,"1",2)` is 2). A direct error propagates as itself
(`MODE(1,"1",#DIV/0!,2,3)`, `MODE(1/0,1,1)` are `#DIV/0!`, `MODE(NA(),1,1)` `#N/A`). A single-cell
reference to a blank cell and an empty argument are `#VALUE!`: `MODE(B9,2,D7)` with B9 = 2 and D7 blank
is `#VALUE!` although 2 appears twice, and so is `MODE(1,1,)`. Blanks and text inside a multi-cell range
are ignored (`MODE(C16:E16,-1.1)` = -1.1 with D16 blank). An error picked by CHOOSE propagates: legacy
`=MODE(CHOOSE(1,B1:B6*1))` off the range's rows is `#VALUE!` (references.md "Legacy `@` on CHOOSE
operator arms under aggregates"). An error in a single-cell reference, a range or a name propagates too (`MODE(K1,1,1)` with K1 = `=1/0` and `MODE(nmerr,1,1)` are `#DIV/0!`, MODE.SNGL and MODE.MULT alike), and computed text or booleans are skipped like typed ones (`MODE(1,1,"a"&"")`, `MODE(1,1,1=1)`, `MODE(1,1,"1"&"")`, a name holding text are 1; probes/mode-error-and-computed-arguments A1-A15, excel-graph, 2026-10-16, T-1291).
Typed text with no repeated number is `#N/A`, wherever it stands (`MODE(1,"1",2,3)`, `MODE("1",2,3)`,
`MODE(1,2,3,"x")`, `MODE(1.1,K3,"1",1)` with K3 = text "1.1"), and a direct error after typed text still wins
(`MODE("1",1,#DIV/0!,2,3)` is `#DIV/0!`).
Evidence: probes/mode-typed-arguments A1-A33, probes/mode-typed-text-no-mode A1-A16 and probes/mode-error-and-computed-arguments A1-A15 (excel-graph,
2026-10-16, T-1291).
CONFLICTING (decision 0030 2: Excel 15 is below AppVersion 16; decision 0039: the web value is the Phase 1-3
target; decision 0067): poi/FormulaEvalTestData_Copy.xlsx (Excel 15) caches `#VALUE!` for EverythingTests!G936
`MODE(C9,E16,"1",B8)`, H936 `MODE(B8,"1",2,3)` and K936 `MODE("1",1,#DIV/0!,2,3)`, where Microsoft 365 gives
`#N/A`, `#N/A` and `#DIV/0!` on the same inputs (probes/mode-typed-text-no-mode A3, A4, A2). The engine follows
M365, and the three cells are known failures until a 16.x desktop value decides (T-1075, M14). M936
`MODE(B9,2,D7)` (`#VALUE!`, blank single-cell reference) and D936, F936, I936, J936, L936 agree with M365.
Status: VERIFIED (excel-graph); CONFLICTING with the Excel 15 cached G936, H936, K936
Implemented in: `collect_mode_values` (mode_functions.rs); MODE.SNGL/MODE.MULT share it (typed text and
booleans and direct, referenced and named errors verified for all three, T-1291).
Tasks: T-0947, T-1291

## MAXA / MINA: single-cell references are read like ranges
Rule: a single-cell reference argument to text counts as 0 and to a boolean as 1/0, as in a range:
`MAXA(F14,E19,F12)` over three text cells is 0, `MINA(B16,B10,B9)` with B16 = TRUE is 1.
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!J888, F912.
Status: VERIFIED (Excel-cached)
Implemented in: `for_each_value_a` (statistical/count_and_average.rs); covers direct single-cell
references only. A defined name pointing to one cell and numeric text in a referenced cell (counted as
0 by inference) are unprobed. AVERAGEA shares this coercion since T-1526, and the helper reads a
cell that XLOOKUP or INDEX picks as a reference (`AVERAGEA(XLOOKUP(1,A1:A2,B1:B2),5,7)` over "x" is 4;
probes/subtotal-reference-operands-d H22). An array argument is not read like a range: see
statistics.md "AVERAGEA, MAXA and MINA over an array" (T-1555). Engine choices with no oracle cell yet
(T-1551): the same for MAXA/MINA over a picked text cell, and direct date or currency text (read by
`cast_number`).
Tasks: T-0949, T-1526, T-1555

## "inf", "NaN" and out-of-range number text are text
Rule: the words "inf", "infinity", "NaN" (any case, any sign) and number text of 1E+308 or more
are never read as numbers: arithmetic, `--`, VALUE and COUNT/SUM/AVERAGE text arguments give
#VALUE! or skip them, criteria compare them as text; `"+1"` is the number 1.
Evidence: probes/text-number-words; full rule in numbers.md "Number text of 1E+308 or more is not a number".
Oracle: excel-graph, 2026-10-05.
Status: VERIFIED
Tasks: T-0920, T-0140

## Direct text arguments of COUNT coerce like arithmetic
Rule: a direct text argument of COUNT counts whenever arithmetic would read it as a number: clock text
(`"10:00"`), date text (`"1/2/2020"`, `"1/1/1900"`), currency, percent, thousands separators,
a leading blank before plain number text (`" 1"`), `"6 PM"` and `"12:60"` all count (`COUNT("10:00","x")` = 1). "inf" and the text `"TRUE"` do
not count. Reference cells holding text still never count.
Evidence: probes/count-direct-text (excel-graph, 2026-10-06), clock-text-oddities F37.
Implemented in: `functions/statistical/count_and_average.rs` `fn_count`.
Status: VERIFIED
Tasks: T-0252

## Parenthesised number text is negative
Rule: `"(1)"` is read as -1 wherever text is coerced to a number: `0+"(1)"` = -1, `SUM("(1)")` = -1,
`0+"(1,000)"` = -1000, `COUNT("(1)","x")` = 1; but `0+"-(1)"` is #VALUE!. The engine reads it in `parse_formatted_number` (formatter/format.rs), after the currency and date forms, so VALUE and typed cell input inherit it. Probed variants (all through `0+`, `VALUE` and `ISNUMBER` of a text cell; `ISNUMBER` is FALSE for all): padding inside the parentheses is accepted (`( 1 )` = -1), currency and percent inside are accepted (`($1)` = -1, `(1%)` = -0.01), decimals are accepted (`(1.5)` = -1.5); a sign inside (`(-1)`), nested parentheses (`((1))`) and trailing text (`(1)x`) are #VALUE!. The engine matches (T-0279, T-0282). More forms (probes/paren-number-text-forms, excel-graph, T-0281): e-notation is accepted (`(1e3)` = -1000, `(1E-3)` = -0.001, `(1.5e2)` = -150), `(0)` and `($0)` are 0 (the sign of zero is not observable), `($1)` is accepted but a sign after the currency (`($-1)`, `($ -1)`), a sign before it (`(-$1)`), a plus sign (`(+1)`), a minus before e-notation (`(-1e3)`, `(-0)`) and currency after the digits (`(1$)`) are #VALUE!. The engine rejects these as Excel does (T-0282). More forms (probes/paren-number-text-currency-variants, excel-graph 2026-10-10, T-1300): `($+1)`, `(1 $)` and `(€-1)` are #VALUE!, but a trailing euro sign is accepted: `(1€)` = -1, so `€` is the one currency that may follow the digits inside parentheses (`(1$)` stays #VALUE!). Euro suffix forms (probes/paren-number-text-euro-suffix, excel-graph 2026-10-10, T-1309): `(1 €)`, `(1,000€)`, `(1.5€)`, `(1e3€)` and `(1€ )` are all negative (-1, -1000, -1.5, -1000, -1), and the unparenthesised controls `1€` and `1 €` are 1, so inside parentheses a euro suffix behaves as it does outside, and the engine matches. Other currencies after the digits are unprobed beyond `$`; the engine rejects them. Typed into a cell (probes/typed-paren-number-input, excel-graph 2026-10-07, typed entry through a Graph range `values` write, T-0280): `(1)` and `( 1 )` are -1 with format General, `(1,000)` is -1000 with `#,##0`, `($1)` is -1 and `($1,000)` -1000 with `$#,##0_);[Red]($#,##0)`, `(1%)` is -0.01 with `0%`, and `(-1)` stays text. The unparenthesised controls get the same formats: `1`, `-1` General, `1,000` `#,##0`, `$1` `$#,##0_);[Red]($#,##0)`, `1%` `0%`. So the parentheses only negate; the format comes from the inner text, and typed currency gets the red-negative accounting-style format, not `$#,##0`. The engine's values and formats all match since T-0283 ("Typed currency and percent formats"). Formats are VERIFIED by the oracle's `ENTERED` log, committed in docs/research/typed-input-formats-2026-10-07.md (not scored: `CELL("format")` is `#VALUE!` on excel-graph).
Evidence: probes/paren-negative-text, probes/paren-negative-variants, probes/paren-number-text-forms, probes/paren-number-text-currency-variants, probes/paren-number-text-euro-suffix, probes/typed-paren-number-input (excel-graph, 2026-10-07; formats: docs/research/typed-input-formats-2026-10-07.md).
Status: VERIFIED
Tasks: T-0259, T-0277, T-0280, T-0281, T-0282, T-0283, T-1300, T-1309

## Currency sign before parentheses is negative
Rule (en-US; probes/t4117-typed-currency-paren, excel-graph 2026-11-04, T-4117; ENTERED log
docs/research/t4117-typed-currency-paren-2026-11-04.md): `$`, optional spaces, then a parenthesised unsigned amount is a
negative number, typed or coerced: `$(12)`, `$ (12)`, `$( 12 )`, `$(1,234.5)`, `$(12.5)` and `$(0)` are -12, -12, -12,
-1234.5, -12.5 and 0 (typed, ISNUMBER, `"..."+0`, VALUE and NUMBERVALUE agree). Typed formats: `$#,##0_);[Red]($#,##0)`
for an integer amount, `$#,##0.00_);[Red]($#,##0.00)` when the amount has a decimal point with digits or none after it
(`$(12.0)`, `$(12.)`, `$(.5)`), `0.00E+00` for `$(1e3)`. Text: `(12$)`, `$(12`, `$12)`, `$(-12)`, `-$(12)`, `$((12))`,
`$(12)%` and `$(1,23)`. Typed `€(12)` is text with General, but `"€(12)"+0`, VALUE and NUMBERVALUE give -12
(typed input has no `€` among its currencies, as above). (NUMBERVALUE reads `$(12)%` as -0.12 and `$(1,23)` as -123, which the engine matches.)
`($12)` and `(12)` are the older paren forms above. More rows (probes/t4117-currency-parens A1:A34, excel-graph
2026-11-04; ENTERED log docs/research/t4117-currency-parens-2026-11-04.md):
- `€` takes the form only where it is a currency: typed `€(12)`, `€ (12)`, `€(1,234.5)`, `€(1e3)` and `€(1 1/2)`
  stay text, but VALUE, NUMBERVALUE and `+0` read them as negative (A15:A17, A25, A31). `£(12)` is `#VALUE!`
  everywhere (A28), as `£12` is.
- A mixed fraction is negative too: `$(1 1/2)`, `$ (1 1/2)` and `$( 1 1/2 )` are -1.5 typed with `# ?/?`, `$(3 1/12)`
  is -3.0833… with `# ??/??` (A21, A29, A33:A34), by VALUE and `+0` but not NUMBERVALUE, which drops the spaces
  (`#VALUE!`). `-$(1 1/2)` is text (A32); typed `$(1 1/2%)` stays text but takes `0.00%`, like `-$3 1/2%` (A30).
- `$(1.5E+2)` is -150 in `0.00E+00` (A19); `$(12%)`, `$(+12)`, `$()` and `$($12)` are text (A20, A22, A23, A27).
Implemented in `parse_percent_or_currency` (formatter/parse_formatted.rs) and, for fractions, `read_mixed_fraction`
(mixed_fraction.rs).
Evidence: probes/t4117-typed-currency-paren, probes/t4117-currency-parens (excel-graph, 2026-11-04).
Status: VERIFIED
Tasks: T-4117

## Typed currency and percent formats
Rule (en-US, text typed into a cell; probes/typed-currency-percent-input, excel-graph 2026-10-11, raw `ENTERED` log in docs/research/typed-currency-percent-input-2026-10-11.md, T-0283): a leading `$` (also `-$`, `$-`, and inside parentheses) gives `$#,##0_);[Red]($#,##0)`, or `$#,##0.00_);[Red]($#,##0.00)` when the text has any decimal digits (`$1.50`, `$1.0`, `$1,234.5`); a `%` gives `0%` or `0.00%` with decimals (`1.5%`, `1.0%`, `12.345%` -> `0.00%`, the value keeps all digits) and ignores grouping (`1,000%` is `0%`); scientific text keeps `0.00E+00` whichever symbol is around it (`$1e3`, `1e3%`, `-$1e3`, `($1e3)`); plain grouped text gets `#,##0` / `#,##0.00` (`(1,000.5)` -> `#,##0.00`). `-$1e3` is -1000 (the engine used to return +1000, also through `0+"-$1e3"` and VALUE, which are unprobed). A euro sign is not an en-US currency: `1€`, `1.5€`, `-1€` and `€1` stay text with General. The engine matches the `$` and percent formats (T-0283) and keeps typed euro text as text (T-1335: `set_user_input` no longer lists `€` among the currencies; `0+` coercion of `1€` still reads a number, a separate probed path (paren-number-text-euro-suffix); VALUE shares that code path (cast.rs, text/common.rs) but has no oracle cell of its own). Other prefix symbols keep the plain `{symbol}#,##0` format (unprobed).
Evidence: probes/typed-currency-percent-input (excel-graph, 2026-10-11).
Status: VERIFIED
Tasks: T-0283, T-1335

## Group commas and a leading percent sign
Rule (en-US): number text reads a comma-separated integer part in which every group after a comma has three or more
digits (not exactly three): VALUE, `A1+0` and IMREAL of "1,0000" are 10000, of "1,000,0000" 10000000, of "1,00000",
"1,000000" and "12,0000" 100000, 1000000 and 120000, of "1,0000.5" 10000.5, of "-1,0000" -10000 and of "1,0000e2"
1000000. A group of fewer than three digits or none is refused: "1,", "1,00", ",1", "2,0", "1,000,00", "1,000,0",
"1,000,", "1,,000" and "1,000.5,000" are #VALUE! (IMREAL #NUM!). An integer part of only zeros before the first comma is refused whatever the group: "0,0000", "0,000", "00,0000" and "00,000" are #VALUE!, while "01,0000" is 10000. One leading percent sign divides by
100 and may be followed by spaces, a sign, a group or a decimal: "%1", "% 1", "%-1", "%1.5" and "%1,0000" are 0.01,
0.01, -0.01, 0.015 and 100; "1,0000%" is 100. A percent sign on both ends ("%1%") and a doubled one ("2%%") are
#VALUE!. NUMBERVALUE strips its own percent signs first, so its "%%1.5" stays #VALUE! (the reader now reads the
leading one, NUMBERVALUE refuses a second).
Typed into a cell (probes/typed-group-comma-leading-percent, excel-graph 2026-11-02, T-3835, formats in docs/research/typed-group-comma-leading-percent-2026-11-02.md): the same texts give the same numbers and the refused ones stay text (General). Formats: grouped integers `#,##0` (`1,0000`, `01,0000`, `-1,0000`, `1,000,0000`), `#,##0.00` with a decimal (`1,0000.5`); a leading percent `0%` (`%1`, `%-1`, `% 1`, `%1,0000`), `0.00%` with decimals (`%1.5`); `1,0000%` `0%`. The engine matches (the reader is shared with VALUE; test `typed_group_comma_and_leading_percent_formats`).
Sign before the percent sign and a trailing `$` (probes/t3742-value-percent-currency, excel-graph 2026-11-03, T-3742): VALUE and `+0` of "-%1.5" are -0.015, like "%-1.5" and "(%1.5)"; "%%1.5", "%1.5%", "%", "%$1.5" and "$%1.5" are #VALUE!, "% 1.5" is 0.015. A trailing dollar sign is refused in every form, "1.5$", "1.5 $" and "-1.5$" are #VALUE! for VALUE, `+0` and NUMBERVALUE, while a trailing euro sign reads ("1.5€" is 1.5, as do "$1.5" and "-$1.5").
Evidence: probes/eng-inumber-grammar-value, probes/eng-number-text-group-comma-edges (excel-graph, 2026-11-02), probes/t3742-value-percent-currency (excel-graph, 2026-11-03). Sign and `$` shapes (probes/t3980-typed-percent-currency, excel-graph 2026-11-03, T-3980): VALUE and `+0` of "-% 1.5" is -0.015, "-%1e2" is -1, "+%1.5" is 0.015, and "-%-1.5" is #VALUE! (a minus after `-%` is refused). Typed into a cell, `-%1.5` and `-% 1.5` are numbers (-0.015, format `0.00%`, ENTERED log in docs/research/t3980-typed-percent-currency-2026-11-03.md); `-%-1.5`, `1.5$`, `1.5 $` and `-200$` stay General text (not formulas). The `$` refusal applies to coercion, criteria and typed input; only the formula-or-text test of a typed leading sign still reads a trailing `$`, so `-200$` is text, not a formula. COUNTIF of 1.5 against "1.5$", "=1.5$", "1.5 $", "-1.5$" and "1.5€" is 0, "$1.5" is 1, probes/t3742-value-percent-currency B1:B6). Shapes after a sign (probes/t3991-percent-shapes, excel-graph 2026-11-04, T-3991): VALUE and `+0` of "+%-1.5", "-% -1.5" and "-%+1.5" are #VALUE! (a sign before the percent sign refuses any second sign after it, unlike "%-1.5"), "+% 1.5" is 0.015 and "+%1e2" is 1. Typed into a cell, `+%1.5` is a number (0.015, format `0.00%`, TEXT("0.00%") `1.50%`), `-%1.5` shows `-1.50%`, and `3 1/2$` stays General text; VALUE("3 1/2$") is #VALUE!. Fixed in the engine (T-3991).
Oracle: excel-graph, 2026-11-02.
Status: VERIFIED
Tasks: T-3811 (implements), T-3644 (found), T-3742 (sign before the percent, trailing `$`), T-3980 (typed shapes)

## Percent text is rounded once
Rule (en-US): number text ending in `%` (VALUE, `0+"…"`, typed cell input) is the decimal text shifted two places and rounded to the nearest double once: `VALUE("23.123%")` = 0.23123, not the double 23.123 divided by 100 (0.23123000000000002). The 15-significant-digit truncation of number text applies before the shift: `12345.6789012345678%` = 123.456789012345. All three paths agree for `23.123%`, `-23.123%`, `0.07%`, `1.005%`, `12345.6789012345678%`, and the e-notation texts `23e-12%` (2.3e-13) and `1.5E-300%` (1.5e-302), and typed input is a number (ISNUMBER TRUE). The `%` operator in a formula (`=23.123%`) is a separate path and is not covered here.
Evidence: probe `probes/percent-text-paths` (VALUE, `0+`, typed input; excel-graph); ironcalc-calc TEXT/T_VALUE_VALUETOTEXT.xlsx `Sheet1!I4` (cached 0.23123).
Status: VERIFIED
Tasks: T-0929, T-2136

## AND/OR/XOR text arguments
Rule: in Microsoft 365, AND, OR and XOR skip every text argument that is not "TRUE" or "FALSE",
whether it is literal or computed, empty or not: `AND("1",TRUE)`, `AND("abc",TRUE)`, `AND(" ",TRUE)`,
`AND("",TRUE)`, `AND(""&"1",TRUE)`, `AND(IFERROR("1",0),TRUE)`, `AND(T("1"),TRUE)`, `AND(("1"),TRUE)`
and `AND("1",1)` are TRUE, and `AND("1",FALSE)` is FALSE. Text in an array literal (`{"1",TRUE}`) and in
referenced cells (A1 = "x", A2 = "1") is skipped as well. A literal "TRUE" argument counts as TRUE,
case-insensitively (`OR("true",FALSE)` = TRUE), but numeric text does not count as a number (`OR("1",FALSE)` = FALSE).
What counts is a direct scalar text whose whole value is "TRUE" or "FALSE" (any case), however it was computed: `""&"TRUE"`, `IF(TRUE,"TRUE")`, `T("TRUE")`, `LOWER("TRUE")` and `LET(s,"TRUE",s)` count as TRUE and the `&`, IF, LOWER and LET "FALSE" forms as FALSE (`T("FALSE")` is unprobed), beside a logical and alone (`AND(""&"FALSE",TRUE)` and `AND("FALSE",TRUE)` are FALSE, `XOR(T("TRUE"),TRUE)` is FALSE, `OR(LOWER("TRUE"))` is TRUE). The parse does not trim: `" TRUE"`, `"TRUE "`, `" FALSE"`, `"FALSE "` and a CHAR(160) pad are skipped like any other text (`AND(" FALSE",TRUE)` is TRUE, `OR(" TRUE")` is `#VALUE!`). Array elements are skipped whatever their text, in a constant or a computed array: `{"TRUE"}`, `{"FALSE"}`, `IF({1},"TRUE")` and `{"TRUE","FALSE"}` change nothing beside a logical and are `#VALUE!` alone. So text counts by its source (a direct value, not a reference or array element), not by the argument's syntax. Probe layout: 21 forms in rows 2-22 × AND/OR/XOR beside TRUE, beside FALSE and alone in B:J; `AND("FALSE")` alone is also FALSE in ironcalc E48.
A "TRUE"/"FALSE" text cell is skipped as well, also when a function picks it: with A1 = "TRUE" and A2 = "FALSE",
`AND`, `OR` and `XOR` of `IF(TRUE,A1)`, `INDEX(A1:A2,1)`, `XLOOKUP(1,{1},A1:A1)`, `CHOOSE(1,A1)`, `LET(x,A1,x)`,
`TAKE(A1:A2,1)` and `OFFSET(A1,0,0)` are all `#VALUE!` (picked cells are references; evidence:
probes/and-or-xor-picked-text, 70 cells, excel-graph, T-1685). A direct LAMBDA is `#VALUE!` in AND, OR and XOR
(`AND(LAMBDA(x,x),TRUE)`, `AND(LAMBDA(x,x))`), but a direct error beats it in either order (`AND(LAMBDA(x,x),1/0)`,
`AND(1/0,LAMBDA(x,x))` are `#DIV/0!`); an error cell ranks by position: before the LAMBDA it wins (`AND(A3,LAMBDA(x,x))` is `#N/A`, A3 = `#N/A`), after it it does not (`AND(LAMBDA(x,x),A3)` is `#VALUE!`; J5/J6, J11/J12, J17/J18; unlike the AVERAGE family, syntax.md T-1602); ours returned `#N/IMPL!` and skipped it, fixed in T-1685. A 3-D reference (`AND(Sheet1:Sheet2!B1)`) is read as a reference, not `#VALUE!` (K1-K9).
When nothing but skipped text is left, the result is `#VALUE!` (`AND("1")`, `AND("")`, `OR("","")`). An
error argument wins (`AND("1",#N/A)` = `#N/A`). OR and XOR behave the same.
CONFLICTING (decision 0030 2: Excel 15 is below AppVersion 16; decision 0039: the web value is the Phase 1-3
target, and the desktop check is T-1075 item 7): poi/FormulaEvalTestData_Copy.xlsx
EverythingTests!J136 caches `#VALUE!` for `AND("1",1)`, where current Excel for the web gives TRUE. The engine
follows the web, and J136 stays a known failure until a 16.x desktop value decides (M14). I136 `AND(E14,E17,"1")`
is not a conflict: E14 and E17 hold text or are empty, so every argument is skipped and both versions give `#VALUE!`.
Evidence: probes/and-or-xor-text-arguments (84 cells, excel-graph, 2026-10-11, T-0258); ironcalc-calc
LOGICAL/AND_OR_XOR.xlsx (Excel-cached) "AND XOR OR"!C46:E53; gen-m2/893302d6c5b99e2e Calc!A108 (excel-graph); probes/t3135-and-or-xor-computed-bool-text (189 cells,
excel-graph, 2026-10-29, T-3135).
Status: VERIFIED (web); CONFLICTING with the Excel 15 cached J136. The engine passes every probe cell.
Tasks: T-0258, T-1075, T-3135

## GCD / LCM / MULTINOMIAL range cells
Rule: GCD, LCM and MULTINOMIAL take ranges, but unlike SUM they do not skip what is not a number: a
text or boolean cell in a range is `#VALUE!` (`GCD(G1:G3)` with 4, "x", 6; `GCD(H1:H3)` with 4, TRUE,
6), a numeric-text cell counts as its number (`GCD(I1:I3)` with 4, "2", 6 is 2, LCM 12) and a blank
cell reads as 0 (`LCM(F1:F3)` with 4, blank, 6 is 0). Typed, numeric text is a number
(`GCD("8",4)` = 4, `MULTINOMIAL("2",3)`) but a boolean is `#VALUE!` (`GCD(TRUE,4)`,
`MULTINOMIAL(TRUE,3)`; typed LCM arguments are unprobed), whereas
`FACT(TRUE)` = 1 and `COMBIN("5",2)` = 10. SERIESSUM's coefficient range is stricter still: text,
boolean and numeric-text cells of a multi-cell range are all `#VALUE!` (a blank reads as 0), while a typed numeric-text
`x` is accepted (`SERIESSUM("1",0,1,{1,2})` = 3). 
A boolean coefficient is `#VALUE!` in every form (a cell, typed `TRUE`/`FALSE`, an array-literal element, `(B1:B3)`); a single coefficient value is read as a number, so typed `"5"` and a one-cell text reference are 5 (`SERIESSUM(1,0,1,B4)` and `B4:B4`; non-numeric single text such as `"abc"` or `""` is `#VALUE!`), while text in an array literal is `#VALUE!` (probes/seriessum-boolean-coefficients A1:A18, excel-graph, 2026-10-16, T-1191). SUMPRODUCT is the opposite: anything that is not a
number counts as 0, a typed scalar too (`SUMPRODUCT("2")` = 0, `SUMPRODUCT(TRUE)` = 0,
`SUMPRODUCT("2",3)` = 0, `SUMPRODUCT(3)` = 3), and text in a range is 0 (`SUMPRODUCT(A1:A6)` = 10).
PRODUCT and SUMSQ skip text and booleans in ranges and coerce typed ones (`PRODUCT(TRUE,3)` = 3).
PRODUCT of an array argument (`PRODUCT(IF(SEQUENCE(5),7))`) multiplies its numbers left to right
(probes/geomean-single-and-copies Copies!B1:B35). Text and booleans inside the array are skipped, an array with no
numbers gives 0 (`PRODUCT({"a"})`, `PRODUCT({TRUE,FALSE})`), and the first error element wins
(`PRODUCT({2,#N/A,#DIV/0!})` = #N/A) (probes/product-array-elements, excel-graph).
In a 2D array the first error in row-major order wins (`PRODUCT({1,#N/A;#DIV/0!,2})` = #N/A, `PRODUCT({1,#DIV/0!;#N/A,2})` = #DIV/0!). A
no-number array with a scalar gives the scalar (`PRODUCT({"a"},3)` = `PRODUCT(3,{"a"})` = `PRODUCT({TRUE,FALSE},3)` = 3). An error inside an array
ranks like a referenced-cell error, as in SUM: a direct scalar error beats it either way (`PRODUCT({1,#N/A},#NUM!,#DIV/0!)` = #NUM!,
`PRODUCT(#NUM!,{1,#N/A})` = #NUM!), and against a referenced error or another array the leftmost wins (`PRODUCT({1,#N/A},C1)` with C1=#NUM! is #N/A,
`PRODUCT(C1,{1,#N/A})` #NUM!, `PRODUCT({#N/A},{#DIV/0!})` #N/A) (probes/product-array-2d-and-scalar, excel-graph 2026-10-14, T-1287).
MDETERM, MINVERSE and MMULT are `#VALUE!` for a text cell in a matrix (arrays.md has blanks).
These decide the generator's argument modes (eval/catalog/functions.yaml `array`, T-1082); the
SUMX2MY2 family's `array` mode rests on "SUMX2MY2 / SUMX2PY2 / SUMXMY2 pairing" above, and
PERCENTOF's `aggregate` on Microsoft's page and on probes/percentof-snap-overflow-union (T-0416): direct text `"1"` and TRUE count as numbers (`PERCENTOF("1",2)` and `PERCENTOF(TRUE,2)` = 0.5), as in SUM.
A blank cell is 0 only beside another argument or inside a multi-cell range: when every argument is a lone blank cell, GCD,
LCM and MULTINOMIAL are `#VALUE!` (`MULTINOMIAL(Z200)`, `MULTINOMIAL(Z200:Z200)`, `MULTINOMIAL(Z200,Z201)`, `MULTINOMIAL(IF(TRUE,Z200))`,
`GCD(Z200)`, `GCD(Z200:Z200)`, `LCM(Z200)` with Z200 and Z201 blank), while `MULTINOMIAL(Z200,2)`, `MULTINOMIAL(2,Z200)`,
`MULTINOMIAL(Z200:Z201)` and `MULTINOMIAL(Z200:Z201,2)` are 1, `GCD(Z200,4)` and `LCM(Z200,4)` 4, and a cell holding 0 is 1
(`MULTINOMIAL(Z210)`) (probes/t2275-address-bounds J1, J15:J30, excel-graph, 2026-10-21). Ours: implemented by `Model::is_lone_blank_cell`
(functions/math_and_trigonometry/gcd_lcm.rs), called from `gcd_lcm_impl` and `fn_multinomial` (T-2294). Unprobed: how this `#VALUE!` ranks against a later negative or omitted argument.
The type check wins over a negative argument whatever the order: `GCD(-1,B1:B11)` with the text "TRUE" in B1 is
`#VALUE!`, and `MULTINOMIAL(A6:D9)` whose first cell is -2.5 and later cells hold FALSE and text is `#VALUE!`
(gen-m6/5cc04d6f42b23b51 Calc!A11, gen-m6/2c1a6e93a1974f48 Calc!A35, gen-m7/4629e931955aa4b8 Calc!A7, excel-graph; fixed in
T-1261). A negative's `#NUM!` loses to every other error, whatever the position and kind of the other argument, in GCD, LCM
and MULTINOMIAL alike: typed text (`GCD(-1,"a")`, `GCD("a",-1)`), a typed TRUE, a typed error (`GCD(-1,NA())`, `GCD(NA(),-1)`), a text,
TRUE or error cell of a range before or after it, and an array-literal element (`GCD(-1,{4,#N/A})`, `GCD(-1,{4,"a"})`, `GCD(-1,{4,TRUE})`) are
`#VALUE!` or the error itself; only with no other error is the answer `#NUM!`. Array-literal text or TRUE elements next to a negative are the same in LCM and MULTINOMIAL
(`LCM(-1,{4,"a"})`, `LCM({4,TRUE},-1)`, `MULTINOMIAL(-1,{4,#N/A})`: probes/gcd-lcm-multinomial-error-order A43:A52, T-1655). An argument above 2^53 is
`#NUM!` the same deferred way, a loser to every other error in either order, in GCD, LCM and MULTINOMIAL (typed text, `NA()` and a negative, range cells and array-literal elements are all probed, T-1661): `GCD(10^16,"a")` and `GCD("a",10^16)` are `#VALUE!`,
`GCD(10^16,NA())` and `GCD(NA(),10^16)` `#N/A` (A9:A12, A23:A26, A37:A40), and `GCD(10^16,-1)` / `GCD(-1,10^16)` are `#NUM!` (A13:A14); the engine used to return the
"Argument too large" `#NUM!` at once for GCD and LCM.
LCM's result above 2^53 is deferred the same way: `LCM(2^26+1,2^27+1)` is `#NUM!` but `LCM(2^26+1,2^27+1,"a")` is `#VALUE!`, `...,NA())` `#N/A`, and a later 0 makes the result 0 (`LCM(2^26+1,2^27+1,0)`); a later negative stays `#NUM!` (probes/gcd-lcm-overflow-ranges A1:A56, excel-graph, 2026-10-16, T-1661).
An omitted argument: the first one is `#N/A` (`GCD(,4)`, `GCD(,-1)`, `GCD(,)`, `MULTINOMIAL(,4)`), a later one is skipped, not read as 0
(`LCM(4,)` and `LCM(2,4,)` are 4, `GCD(4,)` 4, `GCD(4,,-1)` and `GCD(-1,,4)` `#NUM!`). Two different non-negative errors are judged left to right, the first argument's winning: `GCD("a",NA())` is `#VALUE!`,
`GCD(NA(),"a")` `#N/A`, `GCD(1/0,NA())` `#DIV/0!`, `GCD(NA(),1/0)` `#N/A`, and an omitted first argument is `#N/A` ahead of any later error (`GCD(,"a")`, `GCD(,NA())`) while a later omitted one is
skipped, so the earlier error shows (`GCD("a",)` is `#VALUE!`, `GCD(NA(),)` `#N/A`); LCM and MULTINOMIAL alike (A1:A8, A15:A22, A29:A36).
Evidence: probes/gcd-lcm-multinomial-negative-precedence E1:E71 (excel-graph, 2026-10-15, T-1574); engine fixed in T-1574. probes/gcd-lcm-multinomial-error-order A1:A52 (excel-graph, 2026-10-15, T-1655); GCD and LCM fixed in T-1655.
Evidence: probes/math-argument-modes Sheet1!C1:C56 (excel-graph, 2026-10-08).
Implemented in: the engine matches every cell except MULTINOMIAL numeric text (C9, C47, C52; T-1190);
SERIESSUM boolean coefficients fixed in T-1191.
Status: VERIFIED
Tasks: T-1082, T-1190, T-1191, T-1261, T-1574, T-1655, T-1661

