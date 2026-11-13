# Probe request `review-comments`: comments, notes and Show Changes in Excel for the web

> **val11 note:** `64759.xlsx` is an Apache POI test file (Apache-2.0), so this change does not carry it: `python3 -I ../fetch_public_inputs.py review-comments-probe .` downloads it from POI at a pinned revision and checks its sha256.

*From poc-design-cc, 2026-11-12, for the Excel oracle owner (routed by the coordinator). Asked by the UI lane
(poc-ui-b-cc) for the Review area's specs, `docs/ui/specs/review/` on its branch `ui-b/freeze-docs` (dcace040b,
not on main when this was written): its `probes.md` lists five requests, R1 to R5, and this is all five, written
as steps. Taskei P561736218. Nothing here has been run in Excel, and nothing waits for it today.*

## What is asked, and what is not

The Review area's specs mark as UNVERIFIED every expected result that rests on what Excel does with a comment,
a note or the change list. Microsoft documents part of it, and the steps below ask only the rest. Read on
2026-11-12:

- **"Insert comments and notes in Excel"**, its Web tab
  (https://support.microsoft.com/en-us/office/insert-comments-and-notes-in-excel-bdcc9f5d-38e2-45b4-9a92-0b2b5c7bf6f8):
  - a comment is made with New Comment on the right-click menu and posted with Ctrl+Enter, a reply with New Reply;
  - "You can only edit comments you've created.";
  - a thread is deleted with Delete Comment on the cell's right-click menu, a reply with Delete on the reply;
  - "Anyone who has write access to a workbook can resolve or reopen comments.";
  - a note is made with New Note on the right-click menu or Shift+F2, edited with Notes > Edit Note, and deleted
    with Delete Note;
  - "When you copy and paste threaded comments to other cells, Excel for the web will also paste the cell
    values." and "Copied comments replace any existing comments in the destination area.";
  - "When a cell has a comment, an indicator appears in the corner of the cell." Its shape and colour on the web
    are not given.
- **"The difference between threaded comments and notes"**
  (https://support.microsoft.com/en-us/office/the-difference-between-threaded-comments-and-notes-75a51eec-4092-42ab-abf8-7669077b7be3):
  - "Keep in mind that it's possible for others to edit your comments." and "anyone with edit access to your file
    can edit your comment", which the first page's sentence contradicts. Part 1 asks which holds on the web;
  - "Notes can be pinned to the grid in Excel for the web, but only for the current session", "can be resized,
    but only for the current session", "Notes cannot be moved in Excel for the web", and "Editing Notes in
    Excel for the web will remove any existing rich text the note may have contained."
- **"Show changes made in a workbook"** and **"Get help with Show Changes in Excel"**
  (https://support.microsoft.com/en-us/excel/show-changes-that-were-made-in-a-workbook,
  https://support.microsoft.com/en-us/excel/get-help-with-show-changes-in-excel):
  - "Each entry shows who made the change, where and when they made it, and what they changed.", with "the
    previous cell value for reference";
  - "The Changes pane shows changes to cell values and formulas, including moving, sorting, inserting, or
    deleting cells or ranges." Not shown: "Chart, shape, and other object edits", "PivotTable operations",
    "Formatting changes", "Hiding cells or ranges, or filtering";
  - "To view all changes in a bulk edit, select See changes on the bulk card.";
  - a selection's changes: "Right-click the selection, and then select Show Changes.", or Filter in the pane,
    by Range or Sheet;
  - "You can view changes from the past 365 days.";
  - "Show Changes tracks changes made in Excel for the web, Excel for Windows, Excel for Mac, Excel for iPad,
    Excel for iPhone, and Excel for Android."

Not documented, and asked here: what a delete does on several cells, and whether Undo brings a thread back;
whether a comment or a note follows its cell through a cut, a sort and a row delete; what the two cell marks
look like; what a saved file holds for a note and for a thread with a reply and a mention; and how a fill and a
sort are grouped in the change list. Whether the people list jumps to a person's cell and whether a sheet tab
shows who is on it are steps S5 and S8 of `docs/research/coauthor-selection-probe`, and are not asked twice.

## How

By hand, in Excel for the web. The five parts are independent: any one can be run alone, in the order that
suits. Parts 1 and 5 need **two people, or one person with two accounts in two browsers** (a private window
will do), called A and B; the others need one. Please note the date, the browser and the build Excel reports
(File > About), and write a sentence for each step. A screenshot is the best evidence where a message or a
mark is asked. **Return** the notes as `RESULTS.md` beside this file, with the screenshots and the two
downloaded workbooks of Parts 3 and 4.

The workbook: `review-moves.xlsx` (sha256 `465dc9afe8829fff23c14f63e931335f77dcc9cc96b1f4ec4e125f5a27870e24`),
written by `python3 -I generate.py <dir>`. Sheet `Moves` holds A1:D10: column A counts down from 10 to 1, and
every cell of B1:D10 holds its own first address (`B2` in B2), so a cell that moved says where it came from.
Sorted by column A, smallest first, row 2 goes to row 9 and row 3 to row 8. Sheet `Long` holds one text of
32,767 characters in A1, the longest a cell holds, which ends in `z`. `python3 -I check.py` holds all of this
without Excel.

### Part 1: deleting and editing comments (R1; A and B; a new workbook)

| Step | Do | Write down |
|---|---|---|
| 1.1 | A writes a comment on B2 (`one`), on B3 (`two`) and on B4 (`three`). B replies `reply` on B3. | Nothing, unless something is refused. |
| 1.2 | A edits her comment on B2 to `one, edited`. | Is the comment marked as edited, and how? What does B see, and when? |
| 1.3 | A edits her comment on B3, the one B replied to. | Is Edit offered after a reply? |
| 1.4 | B opens A's comment on B4. | Is Edit offered to B at all? If it is, B changes the text to `three, by B`: whose name does the comment then carry? |
| 1.5 | B deletes his own reply on B3. Then B replies again, and A tries to delete B's reply. After recording both results, B posts a fresh reply `reply-for-undo` on B3; both confirm that it is visible before continuing. | Can a reply be deleted on its own? By its author, by the thread's author, by both? |
| 1.6 | B right-clicks B4 and looks for Delete Comment. | Is it offered to B for A's thread? If it is, do not use it. |
| 1.7 | A selects B2:B4 and uses Delete Comment, from the right-click menu, or from the Review tab if the menu has none. | Is a confirmation asked? Which of the three threads are gone: all, the active cell's, none? |
| 1.8 | A presses Ctrl+Z once. | Do the deleted threads come back, including `reply-for-undo` on B3 if B3's thread was deleted? What does B see? If the threads came back, can B's Ctrl+Z take them away again? |
| 1.9 | Both reload the page. | The comments each sees in the Comments pane. |

### Part 2: comments and notes when content moves (R2; one person; `review-moves.xlsx`, sheet `Moves`)

Before each of steps 2.2 to 2.7, verify the baseline: cell values match the generated workbook, `c-B2` is on B2,
`n-C3` is on C3, and the range comment is as observed in 2.1. After each step, record the result and try Undo to
restore that baseline (2.6 and 2.7 already ask for Undo). Record any failure to restore a comment or note before
repairing the state. If the complete baseline cannot be restored, open a fresh copy of `review-moves.xlsx` and
repeat 2.1 before the next step. Do not measure a later operation on an annotation left missing or changed by
an earlier step.

| Step | Do | Write down |
|---|---|---|
| 2.1 | Write a comment `c-B2` on B2 and a note `n-C3` on C3. Then select A5:B6 and choose New Comment, text `c-range`. | Which cell or cells carry `c-range`: A5 alone, or the range? What does the Comments pane say its place is? |
| 2.2 | Cut B2 and paste it at F10. | Where is `c-B2` now: F10, B2, or gone? |
| 2.3 | Copy A1 (it has no comment) and paste it onto B2. | Does B2 keep `c-B2`? |
| 2.4 | Write a comment `c-D4` on D4. Copy D4 and paste it onto B2. | What does B2 carry now: `c-D4`, `c-B2`, or both? (Microsoft: copied comments replace the ones in the destination.) |
| 2.5 | Select A1:D10 and sort by column A, smallest to largest. The range has no header row: say so if Excel asks. | Where are `c-B2` and `n-C3`, and where are the cells that read `B2` and `C3`? If the comment and the note moved with their rows, each is on the cell that reads its old address: B9 and C8, or B10 and C9 if Excel kept row 1 as a header. |
| 2.6 | Delete row 2 (right-click its header). | Is `c-B2` gone from the sheet? Does the Comments pane still list it, and under which place? Then Ctrl+Z: is it back on B2? |
| 2.7 | Delete row 3 the same way, then Ctrl+Z. | The same two questions for the note `n-C3`. |

### Part 3: notes (R3; one person; `review-moves.xlsx`, sheet `Moves`, then sheet `Long`)

| Step | Do | Write down |
|---|---|---|
| 3.1 | On a fresh copy of `review-moves.xlsx`, write a note `n-B2` on B2. Open the Review tab and the right-click menus of B2 (with a note) and B3 (without a note). | Every command about notes, by its name: New Note, Edit Note, Show/Hide Note, Show All Notes, Convert to Comments, Previous and Next, or whichever are there. Does Shift+F2 open a new note? |
| 3.2 | Write a comment `c-B4` on B4. Zoom to 400% and take one screenshot of B2:B4. | The two marks: shape, colour and corner of each. |
| 3.3 | Rest the pointer on B2 without clicking. | Does the note show? After about how long? Does it carry an author line, and is that line bold? |
| 3.4 | Pin the note on B2 (Show/Hide Note, if there is one), drag its box larger, then click another cell. | Does it stay open? Was a resize possible? |
| 3.5 | On sheet `Long`, select A1 and copy it. On sheet `Moves`, make a new note on B6 and paste. | Is the paste accepted? Does the note's text end in `z`? |
| 3.6 | Make a new note on B7, paste the same text, and type `!` after it. | Is the last character accepted, refused, or is something cut? Any message, word for word. |
| 3.7 | File > Save As > Download a Copy. | Return the file as `saved-notes.xlsx`. |

From `saved-notes.xlsx` we read what you cannot see: whether the note's text starts with the author and a colon
as a bold run, whether the drawing keeps the pinned state and the box's size (Microsoft says both last "only for
the current session"), and how many characters the notes on B6 and B7 hold.

### Part 4: a thread, a reply, a mention and a resolved thread in a saved file (R4; one person; a new workbook)

| Step | Do | Write down |
|---|---|---|
| 4.1 | Write a comment on C3 that mentions a person of your organisation: `please check @` and pick the person. Reply to it once: `done`. | How the mention looks in the posted comment. |
| 4.2 | Write a comment `closed` on C5 and resolve the thread. | How a resolved thread looks on the cell and in the pane. |
| 4.3 | File > Save As > Download a Copy. | Return the file as `saved-threads.xlsx`. It will hold the mentioned person's name and account id, as Excel stores them: say so if it must not be kept in the repository, and we keep only what we read from it. |
| 4.4 | Open `64759.xlsx` in Excel for the web (below). | How the author of the comment on C2 is shown: by name, as an unknown person, by initials, with a picture? Can you reply to it? What does the note on B3 show? |

`64759.xlsx` (sha256 `43f3c765678160e26fab0e3963782a941fb3d41a6a4a9904a48656aa284d68e8`) is a workbook of Apache
POI's test data, saved by Excel for Mac 16.03, which the repository fetches as part of its `poi` suite
(`eval/.cache/suites/poi/64759.xlsx` after `rustsheet-eval fetch`). It has one threaded comment, on C2, whose
author is a person of another organisation's directory, and one note, on B3. A copy goes with this request.

### Part 5: Show Changes (R5; A and B; a new workbook that B creates and shares)

| Step | Do | Write down |
|---|---|---|
| 5.1 | **Before A opens the workbook**, B types 1, 2, 3 into A1:A3 and `=SUM(A1:A3)` into A4. | Nothing. |
| 5.2 | A opens the workbook. A types 10 into C1 and changes A2 to 20. | Nothing. |
| 5.3 | B selects E1, types 1, and fills it down to E100 (drag the fill handle, or copy E1 and paste on E2:E100). | Which operation was used: fill-handle drag or copy/paste. The grouping observed in 5.5 applies only to that operation. |
| 5.4 | A sorts A1:A3 from largest to smallest. B makes C1 bold. | Nothing. |
| 5.5 | A opens Review > Show Changes. | The entries, newest first, as the pane words them: one line for each. Are B's edits of step 5.1, made before A opened the file, listed? How is the fill shown: one card or 100 entries, and what does "See changes" open? How is the sort shown? Is the bold listed? (Microsoft: formatting is not.) Does A4's change of value appear, or only cells someone typed in? |
| 5.6 | A closes the pane, selects A1:A3, right-clicks and chooses Show Changes. Then A uses Filter in the pane, by Sheet and by Range. | Is the right-click command there? What does the pane list for the selection? What does each filter offer? |
| 5.7 | A scrolls to the end of the list. | Whether the pane says how far back it goes, or that older changes are elsewhere, word for word. |

## What the answers decide

The case numbers are the UI lane's (`docs/ui/specs/review/`). For every row, what Excel for the web does
decides the case: where it differs from what a case proposes, the case is revised, and a difference that has to
stay is a known gap with a follow-up task, not a choice of the sheet's (decision 0052).

| # | Question | Read from | Spec cases |
|---|---|---|---|
| 1 | What does Delete Comment do on several selected cells, and does it ask? | 1.7 | S4.4 (the targets, the confirmation) |
| 2 | Does Undo bring back a deleted thread, for whom? | 1.8 | S4.6 |
| 3 | Who may edit a comment, who may delete a reply or a thread? | 1.3 to 1.6 | S4.3, S6.2; the two Microsoft pages disagree on editing |
| 4 | Is an edited comment marked? | 1.2 | S6.1 |
| 5 | Does a comment move with a cut, and stay under a paste? | 2.2 to 2.4 | S7.1, S7.4 |
| 6 | Do a comment and a note move with a sorted row? | 2.5 | S7.5, S2.5 |
| 7 | What becomes of a comment or a note when its row is deleted, and on Undo? | 2.6, 2.7 | S8.1, S8.2 |
| 8 | Can a comment be on more than one cell? | 2.1 | S7.2, S8.3 (cases about a thread on a range) |
| 9 | Which note commands and keys are there? | 3.1 | S2.3 |
| 10 | What do the two marks look like? When does a note show on hover? | 3.2, 3.3 | S2.1, S2.2 |
| 11 | What does a saved note hold: the author run, the pinned state, the box? | `saved-notes.xlsx` | S2.7 |
| 12 | How long may a note be? | 3.5, 3.6, `saved-notes.xlsx` | S2.8 |
| 13 | How are a reply, a resolved thread and a mention stored? Are placeholder notes written for each? | `saved-threads.xlsx` | S3.3, S3.6 |
| 14 | How is an author shown who is not in the organisation? | 4.4 | S3.1 |
| 15 | What does Show Changes list, how are a fill and a sort grouped, and what can it be narrowed to? | 5.5 to 5.7 | S9.1, S9.3, S9.5 |

Each answer becomes an entry of `docs/excel-semantics/` with this request as its evidence; the UI lane writes
the entries and updates its cases in the same change.
