# Probe request `coauthor-selection`: how long Excel for the web shows a co-author's selection

*From poc-design-cc, 2026-11-12, for the Excel oracle owner (routed by the coordinator, who asked for it). For
the collaborative sheet's decision D-UI-169, by which a collaborator's box over the cells is an attention cue:
it shows, holds 0.8 s and fades over 0.3 s. The decision is the UI lane's, in its tree's
apps/collab-sheet/DECISIONS.md, and was not on main when this was written. Nothing here has been run in Excel.*

## What is asked

When a co-author selects cells and then sits still, does Excel for the web go on showing their selection, or
does it show it for a moment and take it away?

Microsoft documents that the selection is shown, and words it as something that is there to point at
("Collaborate on Excel workbooks at the same time with co-authoring", support.microsoft.com, read 2026-11-12):
"with certain versions of Excel, you'll see other people's selections in different colors"; "If you see other
people's selections in different colors, they'll show up as blue, purple and so on. However, your selection
will always be green"; "If you lose track of who's who, rest your cursor over the selection, and the person's
name will be revealed. If you want to jump to where someone is working, select their picture or initials, and
then select the Go to option." The page does not say how long a selection stays while its owner does nothing,
what it looks like (an outline, a fill, a name tag), or what changes when the owner types, leaves the sheet or
goes idle. That is this request.

## How

By hand, in Excel for the web: **two people, or one person with two accounts in two browsers** (a private
window will do for the second), and one workbook stored in OneDrive or SharePoint that both open for editing.
Call them A, who watches, and B, who acts. A keeps a clock in view and takes a screenshot at each reading;
the screenshots are the evidence, and a sentence beside each is enough. Note the date and, for each side, the
browser and the build Excel for the web reports (File > About), if it reports one.

| Step | B does | A records |
|---|---|---|
| S1 | selects C3:D5 with the mouse and then touches nothing for 60 s | at 0 s, 1 s, 5 s and 60 s: is B's selection drawn? As an outline, a fill, or both, in which colour and how thick? Is B's active cell marked apart from the range? Is B's name or initials shown beside it, and for how long? Does anything fade or disappear, and when? |
| S2 | still nothing | A rests the pointer over B's selection: does B's name appear? A scrolls the selection out of view and back: is it still drawn? |
| S3 | presses the down arrow once | how soon does A see the selection move, and is the old place cleared at once? Does a name tag show again at the new place? |
| S4 | starts typing in the cell and stops before Enter, for 10 s | what marks the cell B is editing on A's screen (a tag, a lock, the text as it is typed)? Then B presses Enter: what changes? |
| S5 | goes to another sheet and selects a cell there | on the first sheet, is anything of B left? Does the sheet tab or the presence pictures say where B is? |
| S6 | comes back, selects a whole row, then a whole column, then several areas with Ctrl | is each drawn as B selected it? |
| S7 | leaves the browser tab in the background for 5 minutes | at 5 minutes: is B's last selection still drawn on A's screen? Is B still among the presence pictures? |
| S8 | nothing | A opens B's picture or initials at the top right: what does it offer (Go to), and where does it take A? |

**Return** the screenshots and the notes as `RESULTS.md` beside this file.

## What the answers decide

| # | Question | Read from | What it decides |
|---|---|---|---|
| 1 | Does a co-author's selection stay while they sit still? | S1 at 5 s and 60 s; S2 | whether D-UI-169's cue should hold while the collaborator stays, and not fade after 0.8 s |
| 2 | What is drawn: an outline, a fill, the active cell, a name? | S1 at 0 s; S6 | what the box is made of. D-UI-169 draws a 1 px outline with no fill, 2 px on the cell, and a tag |
| 3 | When does a name show? | S1, S2, S3 | whether the tag is permanent, timed, or shown on hover |
| 4 | What does editing look like to the other person? | S4 | whether a cell being edited needs a mark of its own |
| 5 | What is left when a co-author leaves the sheet or goes idle? | S5, S7, S8 | when a box is taken away, and what the presence pictures carry |

D-UI-169 is the owner's direction for the demonstration film ("the border over cell can come and then
disappear just to draw the attention quickly"), so the sheet differs from Excel there on purpose today. If S1
shows that Excel for the web keeps the selection drawn, the proposal to the UI lane is to hold the outline for
as long as the collaborator stays on the range, thin and with no fill as now, and to keep the fade only for
what Excel itself takes away; since the fade was the owner's choice, the change needs his word. If Excel takes
the selection away after a moment, D-UI-169 is Excel's behaviour too, and its 0.8 s is compared with what S1
measures.
