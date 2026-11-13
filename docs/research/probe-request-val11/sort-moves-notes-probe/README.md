# Data > Sort: do notes and threaded comments move with their rows? (Excel for the web, en-US)

From poc-model-cc, 2026-11-12 (Taskei P559273238). Excel for the web only (Steve's ruling, 2026-11-12). The
collaborative sheet's sort moves cell contents like a copy, and moves a note or a thread with its cell's contents
(rustsheet-sheet-ops `sort`, landing on main as the model lane's CR-12). The only source is a collaborator's report
("Excel moves notes and threaded comments with their rows"). Nothing on record probes it. Each answer is a sentence;
`[model: ...]` says what the model does today.

Setup for every step, on a fresh sheet: A1 `Key`, B1 `Value`; A2:A5 `d`, `b`, `c`, `a`; B2:B5 `4`, `2`, `3`, `1`. A
note (Review > Notes > New Note) on B2, text `note on 4`. A threaded comment (Review > New Comment) on B5, text
`thread on 1`. E1 `=B2`.

1. **Sort.** Select A1:B5, Data > Sort, by `Key`, A to Z, "My data has headers". Which cell holds the note now, and
   which the thread? What does E1 show, and what is its formula? [model: the note and the thread move with their
   values (`4` lands on row 5 and `1` on row 2, so the note is on B5 and the thread on B2), and E1 keeps `=B2`,
   which now shows 1]
2. **Hidden rows.** A fresh setup; hide row 3, then sort A1:B5 as in step 1. Does the hidden row's content move, and
   where do the note and the thread go? [model: hidden rows do not move]
3. **A sort of some columns.** A fresh setup; select A1:A5 alone, Data > Sort, and choose "Continue with the current
   selection" if Excel asks. Do the note and the thread in column B move? [model: a thread moves only when it is
   inside the sorted columns, so these stay]
