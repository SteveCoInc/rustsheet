// Office.js (Script Lab or an add-in). Excel for the web is the set of record; a desktop run is a second set,
// labelled (README.md, "How"). Each case runs in a NEW, EMPTY workbook and logs one JSON line. The cases are
// those of README.md. Requirement set: ExcelApi 1.9 (Workbook.usePrecisionAsDisplayed, calculationState). A
// call the host lacks is logged with its error code.
//
// "Set precision as displayed" rounds every number of the workbook for good, so run() refuses a workbook that
// holds any value, changing nothing, and puts the option and the calculation mode back as it found them.
//
// Usage: one case per new workbook, by its button in Script Lab (the end of this file) or by run("A").
// "READ" alone runs in an opened file.

const log = (id, data) => console.log(JSON.stringify({ id, ...data }));

// One row a question: a number format, a number, and the formula that gives that number (default: the literal).
// The constant goes in column A and the formula in column B, both with the format; C and D read them, in
// General.
const T = [
  // Fixed decimals: plain, ties, and doubles next to a tie.
  ["0.00", 1.2345], ["0.00", 2.675], ["0.00", 1.005], ["0.00", -1.005], ["0.00", 0.125], ["0.00", 0.135],
  ["0.00", 0.285], ["0.00", 8.325], ["0.00", 1234567.891], ["0.00", 0.004], ["0.00", -0.004],
  ["0.00", 123456789012.345], ["#,##0.00", 1234.5678],
  ["0.0", 0.25], ["0.0", 0.35], ["0.0", -0.25], ["0.0", 0.05],
  ["0", 0.5], ["0", 1.5], ["0", 2.5], ["0", -0.5], ["0", -2.5], ["0", 0.49999999999999994],
  ["0", -0.001], ["0.0", -0.04], // rounds to zero from below: is the stored number 0 or -0?
  ["0", 1.2345678901234567e19], ["0", 123456789012345.6], ["000", 1.5], ["00.0", 1.25],
  // More decimals than a double holds: is the stored number the 15 digits Excel shows?
  ["0.0000000000000000", 0.1 + 0.2, "=0.1+0.2"], ["0.000000000000000000", 1 / 3, "=1/3"],
  ["0.00000000000000", 0.1 + 0.2, "=0.1+0.2"],
  // General.
  ["General", 1.23456789012345], ["General", 0.1 + 0.2, "=0.1+0.2"], ["General", 123456789.123456],
  ["General", 1.2345678901234567e19], ["General", 0.000001234567890123],
  // Percent.
  ["0%", 0.125], ["0%", 0.135], ["0%", 0.12345], ["0.0%", 0.12345], ["0.00%", 0.123456],
  // Scientific and engineering.
  ["0.00E+00", 12345], ["0.00E+00", 0.00012345], ["0.00E+00", -98765], ["0E+0", 15000], ["0.0E+0", 0.000999],
  ["##0.0E+0", 12345], ["##0.0E+0", 123456],
  // Fractions.
  ["# ?/?", 0.3], ["# ?/?", 2.375], ["# ?/?", -0.3], ["# ?/?", 0.04], ["# ??/??", 0.3], ["# ??/??", 3.14159],
  ["# ?/8", 0.3], ["# ?/2", 0.3], ["?/?", 2.375],
  // Scaling commas.
  ["#,##0,", 1234567], ["#,##0,", 499], ["#,##0,", 500], ["0.0,,", 123456789], ['0.00,"K"', 1234567],
  // Dates and times.
  ["m/d/yyyy", 45000.75], ["yyyy", 45000.75], ["d-mmm-yy", 45000.75], ["m/d/yyyy h:mm", 45000.123456],
  ["h:mm", 0.5004], ["h:mm:ss", 0.50001], ["[h]:mm", 1.5004], ["mm:ss.0", 0.000012],
  // The Text format.
  ["@", 1.2345],
  // Sections and conditions: the section the number selects, and sections that show no digit.
  ["0.00;-0.0", 1.2345], ["0.00;-0.0", -1.2345], ["0.0;(0.00)", -1.2345],
  ["[>=100]0;0.00", 150.6], ["[>=100]0;0.00", 50.678],
  // Rounded, these two belong to the other section: 99.999 shows 100.00, and 100 shows 100; 1.455 shows 1.5,
  // and 1.5 shows 2.
  ["[>=100]0;0.00", 99.999], ["[<1.46]0.0;0", 1.455],
  ["0.00;;", -1.2345], ["0.00;;", 0], [";;;", 1.2345],
  ['0.0;-0.0;"zero"', 0.04], ['0.0;-0.0;"zero"', -0.04], ["0.0;-0.0;0.000", 0],
  // Literals, colours and padding.
  ['0.0" kg"', 1.2345], ['"abc"', 1.2345], ["[Red]0.0", 1.2345], ["[$-409]0.00", 1.2345],
  ["$#,##0.00_);[Red]($#,##0.00)", -1234.5678],
  ['_($* #,##0.00_);_($* (#,##0.00);_($* "-"??_);_(@_)', 0.004],
  ['_($* #,##0.00_);_($* (#,##0.00);_($* "-"??_);_(@_)', 1234.5678],
  // Optional digits.
  ["#.##", 1.2345], ["#.##", 0.001], ["0.??", 1.5], ["0.??", 1.2345], ["#,###", 0.4], ["#", 0.4], ["?.?", 0.04],
];
const formulaOf = (row) => row[2] || `=${row[1]}`;
const column = (m, j) => m.map((row) => row[j]);

// The calculation mode is written here only, so that run() knows whether it has to be put back.
async function setMode(context, st, mode) {
  st.modeTouched = true;
  context.application.calculationMode = mode; // Excel.CalculationMode.manual | automatic
  await context.sync();
}

// The option is written here only, so that run() knows whether it has to be put back.
async function setPad(context, st, on) {
  st.touched = true;
  context.workbook.usePrecisionAsDisplayed = on;
  await context.sync();
}

// Iterative calculation is written here only, for the same reason.
async function setIteration(context, st, to) {
  st.iterationTouched = true;
  const it = context.application.iterativeCalculation;
  it.enabled = to.enabled;
  it.maxIteration = to.maxIteration;
  it.maxChange = to.maxChange;
  await context.sync();
}

async function calc(context, type) {
  context.application.calculate(type); // Excel.CalculationType.recalculate (F9) | full (Ctrl+Alt+F9)
  await context.sync();
}

// What a range holds and shows, with the option and whether anything is owed at this read.
async function read(context, s, address) {
  const r = s.getRange(address);
  r.load("values,valueTypes,text,formulas,numberFormat");
  const app = context.application;
  app.load("calculationState,calculationMode"); // Done | Calculating | Pending
  context.workbook.load("usePrecisionAsDisplayed");
  await context.sync();
  // JSON writes -0 as 0, so the cells whose value the API gives as -0 are listed: [row, column] within the
  // range, from 0. An empty list says what the API gave, not what Excel stores (README.md, "Signed zero").
  const negativeZeros = [];
  r.values.forEach((row, i) => row.forEach((v, j) => Object.is(v, -0) && negativeZeros.push([i, j])));
  return {
    pad: context.workbook.usePrecisionAsDisplayed,
    mode: app.calculationMode,
    calculationState: app.calculationState,
    negativeZeros,
    values: r.values,
    // Double, String, Boolean, Error or Empty: JSON alone cannot tell the error #DIV/0! from that text.
    valueTypes: r.valueTypes,
    text: r.text,
    formulas: r.formulas,
    numberFormat: r.numberFormat,
  };
}

// The same, for the rows of T: A and B hold the formatted constant and formula, C and D read them.
// `columns` are single letters in sheet order; they need not be next to each other (E reads A, B, E and F).
async function readTable(context, s, columns) {
  const last = T.length + 1;
  const first = columns[0];
  const r = await read(context, s, `${first}2:${columns[columns.length - 1]}${last}`);
  const out = { pad: r.pad, mode: r.mode, calculationState: r.calculationState };
  for (const c of columns) {
    const j = c.charCodeAt(0) - first.charCodeAt(0); // the column's place in the range read, not in `columns`
    const values = column(r.values, j);
    // The sheet rows whose value the API gives as -0, which JSON would write as 0.
    const negativeZeros = values.flatMap((v, i) => (Object.is(v, -0) ? [i + 2] : []));
    out[c] = { values, valueTypes: column(r.valueTypes, j), text: column(r.text, j), negativeZeros };
  }
  return out;
}

// Rows 2 to T.length + 1 of `constants` and `formulas` (column letters) get the rows of T.
function writeFormats(s, columns) {
  const last = T.length + 1;
  for (const c of columns) s.getRange(`${c}2:${c}${last}`).numberFormat = T.map(([f]) => [f]);
}
function writeNumbers(s, constants, formulas) {
  const last = T.length + 1;
  s.getRange(`${constants}2:${constants}${last}`).values = T.map(([, n]) => [n]);
  s.getRange(`${formulas}2:${formulas}${last}`).formulas = T.map((row) => [formulaOf(row)]);
}
// Readers in General: a formula that reads a formatted cell can take its format, so General is set after it.
function writeReaders(s, readers, sources) {
  const last = T.length + 1;
  readers.forEach((c, j) => {
    const r = s.getRange(`${c}2:${c}${last}`);
    r.formulas = T.map((_, i) => [`=${sources[j]}${i + 2}`]);
    r.numberFormat = T.map(() => ["General"]);
  });
}

// F and X: what a change of format does once the option is on, in Automatic mode and in Manual mode.
async function formatSteps(context, s, st, manual) {
  s.getRange("A1:B1").numberFormat = [["0.000", "0.000"]];
  s.getRange("A1").values = [[1.23456]];
  s.getRange("B1").formulas = [["=1.23456"]];
  s.getRange("C1:D1").formulas = [["=A1*1000", "=B1*1000"]];
  s.getRange("C1:D1").numberFormat = [["General", "General"]];
  await context.sync();
  const out = { before: await read(context, s, "A1:D1") };
  if (manual) await setMode(context, st, Excel.CalculationMode.manual);
  await setPad(context, st, true);
  out.on = await read(context, s, "A1:D1");
  await calc(context, Excel.CalculationType.recalculate);
  out.onAfterF9 = await read(context, s, "A1:D1");
  // Fewer decimals. 1.235 rounds to 1.24 and 1.23456 to 1.23: B1 tells a recalculation from a second rounding.
  s.getRange("A1:B1").numberFormat = [["0.00", "0.00"]];
  await context.sync();
  out.narrowed = await read(context, s, "A1:D1");
  await calc(context, Excel.CalculationType.recalculate);
  out.narrowedAfterF9 = await read(context, s, "A1:D1");
  await calc(context, Excel.CalculationType.full);
  out.narrowedAfterFull = await read(context, s, "A1:D1");
  // More decimals: are the constant's digits gone, and is the formula calculated again, and when?
  s.getRange("A1:B1").numberFormat = [["0.0000", "0.0000"]];
  await context.sync();
  out.widened = await read(context, s, "A1:D1");
  await calc(context, Excel.CalculationType.recalculate);
  out.widenedAfterF9 = await read(context, s, "A1:D1");
  await calc(context, Excel.CalculationType.full);
  out.widenedAfterFull = await read(context, s, "A1:D1");
  return out;
}

const CASES = {
  // A: the documented path. Numbers and formats first, then the option.
  A: {
    run: async (context, s, st) => {
      writeFormats(s, ["A", "B"]);
      writeNumbers(s, "A", "B");
      writeReaders(s, ["C", "D"], ["A", "B"]);
      await context.sync();
      const off = await readTable(context, s, ["A", "B", "C", "D"]);
      await setPad(context, st, true);
      const on = await readTable(context, s, ["A", "B", "C", "D"]);
      await calc(context, Excel.CalculationType.full);
      const onAfterFull = await readTable(context, s, ["A", "B", "C", "D"]);
      // Once more: does a number that its rounding moved into another section move again?
      await calc(context, Excel.CalculationType.full);
      const onAfterSecondFull = await readTable(context, s, ["A", "B", "C", "D"]);
      const rows = T.map((row) => [row[0], row[1], formulaOf(row)]);
      return { rows, off, on, onAfterFull, onAfterSecondFull };
    },
  },
  // E: the option first. A and B get the format and then the number; E and F the number and then the format.
  E: {
    run: async (context, s, st) => {
      await setPad(context, st, true);
      writeFormats(s, ["A", "B"]);
      await context.sync();
      writeNumbers(s, "A", "B");
      writeNumbers(s, "E", "F");
      await context.sync();
      const entered = await readTable(context, s, ["A", "B", "E", "F"]);
      writeFormats(s, ["E", "F"]);
      await context.sync();
      const formatted = await readTable(context, s, ["A", "B", "E", "F"]);
      await calc(context, Excel.CalculationType.full);
      const afterFull = await readTable(context, s, ["A", "B", "E", "F"]);
      await calc(context, Excel.CalculationType.full);
      const afterSecondFull = await readTable(context, s, ["A", "B", "E", "F"]);
      return { entered, formatted, afterFull, afterSecondFull };
    },
  },
  F: { run: (context, s, st) => formatSteps(context, s, st, false) },
  X: { run: (context, s, st) => formatSteps(context, s, st, true) },
  // FMT: the control for F and X, with the option off: does a change of format alone calculate anything? F1 is
  // volatile, so it changes at every recalculation and at nothing else. (T-5187 asks the same question.)
  FMT: {
    run: async (context, s) => {
      s.getRange("B1").values = [[1.5]];
      s.getRange("B2").formulas = [["=B1*2"]];
      s.getRange("F1").formulas = [["=RAND()"]];
      await context.sync();
      const out = { offStart: await read(context, s, "B1:F3") };
      out.offNoEdit = await read(context, s, "B1:F3"); // a read alone must not change F1
      s.getRange("B1").numberFormat = [["0.0"]]; // a constant's number format
      await context.sync();
      out.offConstantFormat = await read(context, s, "B1:F3");
      s.getRange("B2").numberFormat = [["0.00"]]; // a formula cell's
      await context.sync();
      out.offFormulaFormat = await read(context, s, "B1:F3");
      s.getRange("B3:D3").numberFormat = [["0.0", "0.0", "0.0"]]; // empty cells'
      await context.sync();
      out.offEmptyFormat = await read(context, s, "B1:F3");
      s.getRange("B1").format.fill.color = "#FFFF00"; // a format that is no number format
      await context.sync();
      out.offFill = await read(context, s, "B1:F3");
      s.getRange("B1").values = [[2]]; // the control: an edit that does calculate
      await context.sync();
      out.offValueEdit = await read(context, s, "B1:F3");
      await calc(context, Excel.CalculationType.recalculate);
      out.offAfterF9 = await read(context, s, "B1:F3");
      return out;
    },
  },
  // O: the option turned off again.
  O: {
    run: async (context, s, st) => {
      s.getRange("A1:B1").numberFormat = [["0.00", "0.00"]];
      s.getRange("A1").values = [[1.23456]];
      s.getRange("B1").formulas = [["=1.23456"]];
      s.getRange("C1:D1").formulas = [["=A1*1000", "=B1*1000"]];
      s.getRange("C1:D1").numberFormat = [["General", "General"]];
      await context.sync();
      await setPad(context, st, true);
      const on = await read(context, s, "A1:D1");
      await setPad(context, st, false);
      const off = await read(context, s, "A1:D1");
      await calc(context, Excel.CalculationType.recalculate);
      const offAfterF9 = await read(context, s, "A1:D1");
      await calc(context, Excel.CalculationType.full);
      const offAfterFull = await read(context, s, "A1:D1");
      return { on, off, offAfterF9, offAfterFull };
    },
  },
  // S: a spilled array whose cells have four formats.
  S: {
    run: async (context, s, st) => {
      s.getRange("A1:A4").numberFormat = [["0.0"], ["0.00"], ["General"], ["0%"]];
      s.getRange("A1").formulas = [["=SEQUENCE(4)/3"]];
      s.getRange("C1:C3").formulas = [["=SUM(A1#)"], ["=A2*3"], ["=SUM(A1:A4)"]];
      s.getRange("C1:C3").numberFormat = [["General"], ["General"], ["General"]];
      await context.sync();
      const off = await read(context, s, "A1:C4");
      await setPad(context, st, true);
      const on = await read(context, s, "A1:C4");
      await calc(context, Excel.CalculationType.full);
      const onAfterFull = await read(context, s, "A1:C4");
      return { off, on, onAfterFull };
    },
  },
  // K: what is not a number. A2 is text that looks like a number; A5 a formula that returns such text. A7 and
  // A8 are controls for signed zero: can a read through the API show -0 at all?
  K: {
    run: async (context, s, st) => {
      const six = (f) => [[f], [f], [f], [f], [f], [f]];
      s.getRange("A1:A6").numberFormat = six("0.00");
      s.getRange("A1:A2").values = [[true], ["'1.2345"]];
      s.getRange("A3").formulas = [["=1/0"]];
      s.getRange("A5:A6").formulas = [['="1.2345"'], ["=1.2345>1"]];
      s.getRange("A7").formulas = [["=0*-1"]]; // -0 in IEEE arithmetic
      s.getRange("A8").values = [[-0]]; // -0 handed to the API
      s.getRange("C1:C6").formulas = [["=A1*1"], ["=A2*1"], ["=A3"], ["=A4*1"], ["=A5*1"], ["=A6*1"]];
      s.getRange("C1:C6").numberFormat = six("General");
      await context.sync();
      const off = await read(context, s, "A1:C8");
      await setPad(context, st, true);
      const on = await read(context, s, "A1:C8");
      return { off, on };
    },
  },
  // W: other ways a number reaches a cell while the option is on. G1 is General and holds every digit, if
  // General keeps them (A's General rows say).
  W: {
    run: async (context, s, st) => {
      await setPad(context, st, true);
      s.getRange("A1:A5").numberFormat = [["0.00"], ["0.00"], ["0.00"], ["0.00"], ["0.00"]];
      s.getRange("G1").values = [[1.23456]];
      s.getRange("G2").numberFormat = [["0.00"]]; // an empty cell that has the format, to paste from
      s.getRange("A6:B6").values = [[1.23456, null]]; // General until the paste of formats below
      s.getRange("B6").formulas = [["=1.23456"]];
      await context.sync();
      s.getRange("A1").values = [[1.23456]]; // a number
      s.getRange("A2").values = [["1.23456"]]; // text Excel reads as a number
      s.getRange("A3").formulas = [["1.23456"]]; // the same through `formulas`, as typed
      s.getRange("A4").copyFrom("G1", Excel.RangeCopyType.values); // pasted values
      s.getRange("A5").copyFrom("G1", Excel.RangeCopyType.all); // pasted with its General format
      await context.sync();
      const written = await read(context, s, "A1:A5");
      const source = await read(context, s, "G1");
      // A format that arrives by a paste of formats, over a constant and over a formula.
      const beforeFormats = await read(context, s, "A6:B6");
      s.getRange("A6").copyFrom("G2", Excel.RangeCopyType.formats);
      s.getRange("B6").copyFrom("G2", Excel.RangeCopyType.formats);
      await context.sync();
      const afterFormats = await read(context, s, "A6:B6");
      return { written, source, beforeFormats, afterFormats };
    },
  },
  // G: General in a column too narrow for the digits. Does the option follow what the cell shows?
  G: {
    run: async (context, s, st) => {
      s.getRange("A1").values = [[123456789.123456]];
      s.getRange("A2").formulas = [["=123456789.123456"]];
      s.getRange("A3").values = [[0.1234567890123]];
      s.getRange("C1:C3").formulas = [["=A1"], ["=A2"], ["=A3"]];
      s.getRange("C1:C3").numberFormat = [["General"], ["General"], ["General"]];
      s.getRange("A:A").format.columnWidth = 40; // narrower than the digits
      s.getRange("C:C").format.columnWidth = 200;
      await context.sync();
      const off = await read(context, s, "A1:C3");
      await setPad(context, st, true);
      const on = await read(context, s, "A1:C3");
      await calc(context, Excel.CalculationType.full);
      const onAfterFull = await read(context, s, "A1:C3");
      return { off, on, onAfterFull };
    },
  },
  // CF: a number format given by a conditional format. Does the option follow it, or the cell's own format?
  CF: {
    run: async (context, s, st) => {
      s.getRange("A1").values = [[1.23456]];
      s.getRange("A2").formulas = [["=1.23456"]];
      s.getRange("C1:C2").formulas = [["=A1"], ["=A2"]];
      s.getRange("C1:C2").numberFormat = [["General"], ["General"]];
      const cf = s.getRange("A1:A2").conditionalFormats.add(Excel.ConditionalFormatType.custom);
      cf.custom.rule.formula = "=TRUE";
      cf.custom.format.numberFormat = "0.0";
      // A rule that reads its own cell: shown as 1 it no longer holds, and the cell is General again.
      s.getRange("A3").values = [[1.23456]];
      s.getRange("A4").formulas = [["=1.23456"]];
      s.getRange("C3:C4").formulas = [["=A3"], ["=A4"]];
      s.getRange("C3:C4").numberFormat = [["General"], ["General"]];
      for (const a of ["A3", "A4"]) {
        const own = s.getRange(a).conditionalFormats.add(Excel.ConditionalFormatType.custom);
        own.custom.rule.formula = `=${a}>1.2`;
        own.custom.format.numberFormat = "0";
      }
      await context.sync();
      const off = await read(context, s, "A1:C4");
      await setPad(context, st, true);
      const on = await read(context, s, "A1:C4");
      await calc(context, Excel.CalculationType.recalculate);
      const onAfterF9 = await read(context, s, "A1:C4");
      await calc(context, Excel.CalculationType.full);
      const onAfterFull = await read(context, s, "A1:C4");
      return { off, on, onAfterF9, onAfterFull };
    },
  },
  // I: only what a cell stores is rounded, or the values inside a formula too?
  I: {
    run: async (context, s, st) => {
      s.getRange("A1:A2").numberFormat = [["0.00"], ["0.00"]];
      s.getRange("A1").formulas = [["=1/3"]];
      s.getRange("A2").values = [[0.3333333]];
      s.getRange("B1:B2").formulas = [["=A1*3"], ["=A2*3"]]; // 0.99 if they read 0.33
      // C1 is in 0.00: 1 unless 1/3 is rounded inside the formula.
      s.getRange("C1:C2").formulas = [["=1/3*3"], ["=SUM(A1:A2)*3"]];
      s.getRange("B1:C2").numberFormat = [["General", "0.00"], ["General", "General"]];
      await context.sync();
      const off = await read(context, s, "A1:C2");
      await setPad(context, st, true);
      const on = await read(context, s, "A1:C2");
      await calc(context, Excel.CalculationType.full);
      const onAfterFull = await read(context, s, "A1:C2");
      return { off, on, onAfterFull };
    },
  },
  // ST: a format that arrives through a named style, changed after the cells have it.
  ST: {
    run: async (context, s, st) => {
      context.workbook.styles.add("PadProbe");
      const style = context.workbook.styles.getItem("PadProbe");
      style.numberFormat = "General";
      await context.sync();
      s.getRange("A1").values = [[1.23456]];
      s.getRange("B1").formulas = [["=1.23456"]];
      s.getRange("A1:B1").style = "PadProbe";
      s.getRange("C1:D1").formulas = [["=A1*1000", "=B1*1000"]];
      s.getRange("C1:D1").numberFormat = [["General", "General"]];
      await context.sync();
      await setPad(context, st, true);
      const on = await read(context, s, "A1:D1");
      style.numberFormat = "0.00"; // every cell of the style changes format at once
      await context.sync();
      const styled = await read(context, s, "A1:D1");
      await calc(context, Excel.CalculationType.recalculate);
      const styledAfterF9 = await read(context, s, "A1:D1");
      await calc(context, Excel.CalculationType.full);
      const styledAfterFull = await read(context, s, "A1:D1");
      return { on, styled, styledAfterF9, styledAfterFull };
    },
  },
  // IT: a cell on a loop, three iterations a calculation. Rounded at each iteration (0.33, 0.66, 0.99), or
  // once when the calculation ends (1)? Iterative calculation is the application's setting on the desktop:
  // run() puts it back.
  IT: {
    iterates: true,
    run: async (context, s, st) => {
      await setIteration(context, st, { enabled: true, maxIteration: 3, maxChange: 1e-9 });
      await setPad(context, st, true);
      s.getRange("A1:B1").numberFormat = [["0.00", "General"]];
      s.getRange("A1:B1").formulas = [["=A1+1/3", "=B1+1/3"]]; // B1 is the control: General, never rounded
      await context.sync();
      const entered = await read(context, s, "A1:B1");
      await calc(context, Excel.CalculationType.recalculate);
      const afterF9 = await read(context, s, "A1:B1");
      await calc(context, Excel.CalculationType.recalculate);
      const afterSecondF9 = await read(context, s, "A1:B1");
      const iteration = { enabled: true, maxIteration: 3, maxChange: 1e-9 };
      return { iteration, entered, afterF9, afterSecondF9 };
    },
  },
  // SAVE: leaves the option ON, for the file to be saved and unzipped by hand (README.md, "By hand").
  SAVE: {
    keep: true,
    run: async (context, s, st) => {
      s.getRange("A1:B1").numberFormat = [["0.00", "0.00"]];
      s.getRange("A1").values = [[1.23456]];
      s.getRange("B1").formulas = [["=1.23456"]];
      s.getRange("C1").formulas = [["=A1*1000"]];
      s.getRange("C1").numberFormat = [["General"]];
      await context.sync();
      await setPad(context, st, true);
      return { on: await read(context, s, "A1:C1") };
    },
  },
  // READ: run in a file opened by hand (pad-unrounded.xlsx). It writes nothing but a recalculation.
  READ: {
    opened: true,
    run: async (context, s) => {
      const opened = await read(context, s, "A1:D1");
      await calc(context, Excel.CalculationType.recalculate);
      const afterF9 = await read(context, s, "A1:D1");
      await calc(context, Excel.CalculationType.full);
      const afterFull = await read(context, s, "A1:D1");
      return { opened, afterF9, afterFull };
    },
  },
};

async function run(id) {
  await Excel.run(async (context) => {
    const wb = context.workbook;
    const env = { build: Office.context.diagnostics.version, platform: Office.context.diagnostics.platform };
    // What this call changed, and what it found: the option (`touched`, `atStart`) and the calculation mode.
    const st = { touched: false, atStart: undefined, done: false, modeTouched: false, modeAtStart: undefined };
    const c = CASES[id];
    try {
      if (!c) throw { code: "UnknownCase", message: String(id) };
      const app = context.application;
      app.load("calculationMode");
      app.iterativeCalculation.load("enabled,maxIteration,maxChange");
      wb.load("usePrecisionAsDisplayed");
      wb.worksheets.load("items/name");
      await context.sync();
      env.modeAtStart = st.modeAtStart = app.calculationMode;
      env.padAtStart = st.atStart = wb.usePrecisionAsDisplayed;
      const { enabled, maxIteration, maxChange } = app.iterativeCalculation;
      env.iterationAtStart = st.iterationAtStart = { enabled, maxIteration, maxChange };
      if (!c.opened) {
        // The option rounds every number of the workbook, for good: only an empty workbook is touched.
        const used = wb.worksheets.items.map((sheet) => sheet.getUsedRangeOrNullObject(true));
        used.forEach((u) => u.load("address"));
        await context.sync();
        const full = used.filter((u) => !u.isNullObject).map((u) => u.address);
        if (full.length) {
          throw { code: "WorkbookNotEmpty", message: `run in a new, empty workbook: ${full.join(", ")}` };
        }
        await setMode(context, st, Excel.CalculationMode.automatic);
        if (st.atStart) await setPad(context, st, false); // every case starts from full precision
      }
      const data = await c.run(context, wb.worksheets.getActiveWorksheet(), st);
      st.done = true;
      log(id, { ...env, data });
    } catch (e) {
      log(id, { ...env, error: e.code || String(e), message: e.message });
    } finally {
      // The option as it was found, unless the case is there to leave it on and ran to its end; and the mode as
      // it was found. Neither is written unless this call changed it: a refused workbook is left untouched.
      if (st.touched && !(st.done && c.keep)) wb.usePrecisionAsDisplayed = st.atStart;
      if (st.iterationTouched) {
        const it = context.application.iterativeCalculation;
        it.enabled = st.iterationAtStart.enabled;
        it.maxIteration = st.iterationAtStart.maxIteration;
        it.maxChange = st.iterationAtStart.maxChange;
      }
      if (st.modeTouched) context.application.calculationMode = st.modeAtStart;
      await context.sync();
    }
  });
}
// run("A"), run("E"), run("F"), run("X"), run("FMT"), run("O"), run("S"), run("K"), run("W"), run("G"),
// run("CF"), run("I"), run("ST"), run("IT"), run("SAVE"): one new, empty workbook each. run("READ") in each of
// the two pad-unrounded files generate.py writes.

// ---- In Script Lab ---------------------------------------------------------------------------------------------
// This file is the Script tab, and the HTML tab holds one line: <div id="cases"></div><pre id="out"></pre>
// Press Run, then click one case's button: the click turns every button off, since a workbook takes one case.
// Its JSON line is logged, and shown under the buttons to copy from. The next case needs a new workbook.
if (typeof document !== "undefined" && document.getElementById("cases")) {
  const print = console.log;
  console.log = (line) => {
    print(line);
    document.getElementById("out").textContent += `${line}\n`;
  };
  const buttons = [];
  for (const id of Object.keys(CASES)) {
    const button = document.createElement("button");
    button.textContent = id;
    button.onclick = () => {
      buttons.forEach((other) => (other.disabled = true));
      return run(id);
    };
    buttons.push(button);
    document.getElementById("cases").appendChild(button);
  }
}
