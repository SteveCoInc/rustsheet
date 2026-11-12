// Office.js (Script Lab or an add-in). Excel for the web is the set of record; a desktop run is a second set, labelled
// (README.md, "How"). Each case runs in a FRESH workbook and logs one JSON line; a workbook that holds a value is
// refused. The cases are those of README.md.
// Requirement sets: ExcelApi 1.9 (calculationState), 1.12 (getDirectPrecedents), 1.13 (getDirectDependents),
// 1.14 (getPrecedents), 1.15 (getDependents). A call the host lacks is logged with its error code in place of its
// addresses.
//
// Usage: one case per fresh workbook, by its button in Script Lab (the end of this file) or by run("M1").

const log = (id, data) => console.log(JSON.stringify({ id, ...data }));

async function read(context, sheet, address) {
  const r = sheet.getRange(address);
  r.load("values,formulas");
  const app = context.application;
  app.load("calculationState"); // Done | Calculating | Pending: is anything owed at this read?
  await context.sync();
  return { values: r.values, formulas: r.formulas, calculationState: app.calculationState };
}

async function setMode(context, mode) {
  context.application.calculationMode = mode; // Excel.CalculationMode.manual | automatic
  await context.sync();
}

// ---- M: Manual mode --------------------------------------------------------------------------------------------
const M = {
  // M1: a dirty cell read in Manual mode.
  M1: async (context, s) => {
    s.getRange("A1").values = [[1]];
    s.getRange("B1").formulas = [["=A1+1"]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    s.getRange("A1").values = [[5]];
    await context.sync();
    const stale = await read(context, s, "A1:B1");
    context.application.calculate(Excel.CalculationType.recalculate);
    await context.sync();
    const fresh = await read(context, s, "A1:B1");
    return { stale, fresh };
  },
  // M2: a formula ENTERED in Manual mode that reads a dirty formula cell.
  M2: async (context, s) => {
    s.getRange("A1").values = [[1]];
    s.getRange("B1").formulas = [["=A1+1"]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    s.getRange("A1").values = [[5]]; // B1 is dirty now (stored 2, would be 6)
    await context.sync();
    s.getRange("C1").formulas = [["=B1*10"]]; // 20 (stale B1), 60 (B1 calculated first), or 0 / empty?
    await context.sync();
    const afterEntry = await read(context, s, "A1:C1");
    context.application.calculate(Excel.CalculationType.recalculate);
    await context.sync();
    const afterF9 = await read(context, s, "A1:C1");
    return { afterEntry, afterF9 };
  },
  // M3: a formula entered in Manual mode that reads only literals.
  M3: async (context, s) => {
    s.getRange("A1").values = [[7]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    s.getRange("D1").formulas = [["=A1*2"]];
    await context.sync();
    const afterEntry = await read(context, s, "A1:D1");
    return { afterEntry };
  },
  // M4: an existing formula EDITED in Manual mode, with a reader.
  M4: async (context, s) => {
    s.getRange("A1").values = [[1]];
    s.getRange("B1").formulas = [["=A1+1"]];
    s.getRange("E1").formulas = [["=B1"]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    s.getRange("B1").formulas = [["=A1+2"]];
    await context.sync();
    const afterEdit = await read(context, s, "A1:E1"); // B1 3 or 2? E1 2 (stale) or 3?
    return { afterEdit };
  },
  // M6: a volatile cell in Manual mode: an unrelated edit, then F9 with nothing else dirty.
  M6: async (context, s) => {
    s.getRange("F1").formulas = [["=RAND()"]];
    s.getRange("G1").formulas = [["=F1"]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    const r0 = await read(context, s, "F1:G1");
    s.getRange("A9").values = [[1]]; // nothing reads A9
    await context.sync();
    const afterEdit = await read(context, s, "F1:G1"); // changed or not?
    context.application.calculate(Excel.CalculationType.recalculate);
    await context.sync();
    const afterF9 = await read(context, s, "F1:G1"); // changed?
    context.application.calculate(Excel.CalculationType.recalculate);
    await context.sync();
    const afterSecondF9 = await read(context, s, "F1:G1"); // changed again with nothing dirty?
    return { r0, afterEdit, afterF9, afterSecondF9 };
  },
  // M8: Range.calculate() on a cell whose precedent formula is dirty and outside the range.
  M8: async (context, s) => {
    s.getRange("A1").values = [[1]];
    s.getRange("B1").formulas = [["=A1+1"]];
    s.getRange("C1").formulas = [["=B1*10"]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    s.getRange("A1").values = [[5]];
    await context.sync();
    s.getRange("C1").calculate();
    await context.sync();
    const afterRangeCalc = await read(context, s, "A1:C1"); // C1 20 or 60? B1 2 or 6?
    return { afterRangeCalc };
  },
  // M11: back to Automatic with dirty cells: are they calculated at the switch?
  M11: async (context, s) => {
    s.getRange("A1").values = [[1]];
    s.getRange("B1").formulas = [["=A1+1"]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    s.getRange("A1").values = [[5]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.automatic);
    const afterSwitch = await read(context, s, "A1:B1"); // B1 6 at once?
    return { afterSwitch };
  },
  // M12: a row inserted in Manual mode (S1 lists "deleting or inserting a row or column" as a trigger).
  M12: async (context, s) => {
    s.getRange("A1").values = [[1]];
    s.getRange("B1").formulas = [["=A1+1"]];
    s.getRange("A3").values = [[10]];
    s.getRange("B3").formulas = [["=A3+ROW()"]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    s.getRange("A1").values = [[5]]; // B1 dirty
    await context.sync();
    s.getRange("2:2").insert(Excel.InsertShiftDirection.down); // B3 moves to B4: ROW() would be 4
    await context.sync();
    const afterInsert = await read(context, s, "A1:B4"); // B1 2 or 6? B4 13 or 14?
    return { afterInsert };
  },
  // M13: a formula copied in Manual mode.
  M13: async (context, s) => {
    s.getRange("A1:A2").values = [[1], [100]];
    s.getRange("B1").formulas = [["=A1+1"]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    s.getRange("B2").copyFrom("B1", Excel.RangeCopyType.formulas);
    await context.sync();
    const afterPaste = await read(context, s, "A1:B2"); // B2 101 (calculated), 2 (the source's value) or 0?
    return { afterPaste };
  },
  // M14: Calculate Sheet (Shift+F9) on a sheet whose dirty cells read a dirty cell of another sheet.
  M14: async (context, s) => {
    s.name = "S";
    const t = context.workbook.worksheets.add("T");
    s.getRange("A1").values = [[1]];
    s.getRange("B1").formulas = [["=A1+1"]];
    t.getRange("A1").formulas = [["=S!B1*10"]];
    t.getRange("B1").formulas = [["=A1+1"]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    s.getRange("A1").values = [[5]]; // S!B1, T!A1 and T!B1 are dirty
    await context.sync();
    t.calculate(false); // Calculate Sheet: only the cells marked dirty on T
    await context.sync();
    const afterSheet = { S: await read(context, s, "A1:B1"), T: await read(context, t, "A1:B1") };
    context.application.calculate(Excel.CalculationType.recalculate);
    await context.sync();
    const afterF9 = { S: await read(context, s, "A1:B1"), T: await read(context, t, "A1:B1") };
    return { afterSheet, afterF9 };
  },
  // M15: a defined name added in Manual mode (S3 lists "Adding, changing, or deleting defined names" with no
  // "when in automatic mode").
  M15: async (context, s) => {
    s.name = "S";
    s.getRange("A1").values = [[1]];
    s.getRange("B1").formulas = [["=A1+1"]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    s.getRange("A1").values = [[5]]; // B1 dirty
    await context.sync();
    context.workbook.names.add("n", "=S!$A$1*2");
    await context.sync();
    const afterName = await read(context, s, "A1:B1"); // B1 2 or 6?
    return { afterName };
  },
  // M16: a row hidden in Manual mode (S3 lists "Filtering, hiding, or un-hiding rows" likewise).
  M16: async (context, s) => {
    s.getRange("A1").values = [[1]];
    s.getRange("B1").formulas = [["=A1+1"]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    s.getRange("A1").values = [[5]]; // B1 dirty
    await context.sync();
    s.getRange("3:3").rowHidden = true;
    await context.sync();
    const afterHide = await read(context, s, "A1:B1"); // B1 2 or 6?
    return { afterHide };
  },
  // M17: two formulas written together in Manual mode, the first reading the second; and a pair in reading order.
  M17: async (context, s) => {
    s.getRange("A1").values = [[5]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    // B1: 11 (the pair calculated together), 1 (B2 read before it was calculated), or other?
    s.getRange("B1:B2").formulas = [["=B2+1"], ["=A1*2"]];
    s.getRange("C1:C2").formulas = [["=A1+1"], ["=C1+1"]]; // the control, in reading order: 6 and 7
    await context.sync();
    const afterEntry = await read(context, s, "A1:C2");
    context.application.calculate(Excel.CalculationType.recalculate);
    await context.sync();
    const afterF9 = await read(context, s, "A1:C2");
    return { afterEntry, afterF9 };
  },
  // M18: a dynamic array formula entered in Manual mode, then its input edited.
  M18: async (context, s) => {
    s.getRange("A1").values = [[3]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    s.getRange("D1").formulas = [["=SEQUENCE(A1)"]]; // D1:D3 1, 2, 3 at once; D1 alone; or nothing until F9?
    await context.sync();
    const afterEntry = await read(context, s, "D1:D4");
    s.getRange("A1").values = [[2]]; // the anchor is dirty: does D3 keep 3 until F9?
    await context.sync();
    const afterEdit = await read(context, s, "D1:D4");
    context.application.calculate(Excel.CalculationType.recalculate);
    await context.sync();
    const afterF9 = await read(context, s, "D1:D4");
    return { afterEntry, afterEdit, afterF9 };
  },
  // M19: Calculate Sheet (Shift+F9) with nothing dirty: a volatile cell on the sheet and its reader on another one.
  M19: async (context, s) => {
    s.name = "S";
    const t = context.workbook.worksheets.add("T");
    s.getRange("A1").values = [[1]];
    s.getRange("B1").formulas = [["=A1+1"]];
    s.getRange("C1").formulas = [["=RAND()"]];
    s.getRange("D1").formulas = [["=C1"]];
    t.getRange("A1").formulas = [["=S!C1"]];
    await context.sync();
    await setMode(context, Excel.CalculationMode.manual);
    const before = { S: await read(context, s, "A1:D1"), T: await read(context, t, "A1") };
    s.calculate(false); // Calculate Sheet on S
    await context.sync();
    // Does C1 draw? Does D1 follow it? Does T!A1 keep the old draw?
    const afterSheet = { S: await read(context, s, "A1:D1"), T: await read(context, t, "A1") };
    context.application.calculate(Excel.CalculationType.recalculate);
    await context.sync();
    const afterF9 = { S: await read(context, s, "A1:D1"), T: await read(context, t, "A1") };
    return { before, afterSheet, afterF9 };
  },
  // M20: a formula entered in Manual mode that closes a loop through a cell that was not entered, with iterative
  // calculation OFF: the rule it asks about is the non-iterative one (recalc.md). The setting is the application's
  // on the desktop and the workbook's on the web, so the case sets it, logs what it was and what it is, and puts
  // it back.
  M20: async (context, s) => {
    const iteration = context.application.iterativeCalculation;
    iteration.load("enabled");
    await context.sync();
    const iterationAtStart = iteration.enabled;
    // Everything after the setting is read is inside the try: a sync that fails once the setting is off must not
    // leave it off, on the desktop for every open workbook.
    try {
      iteration.enabled = false;
      await context.sync();
      iteration.load("enabled");
      await context.sync();
      const iterationInCase = iteration.enabled; // false, or the case answers another question
      s.getRange("A1").formulas = [["=B1+1"]]; // 1
      await context.sync();
      await setMode(context, Excel.CalculationMode.manual);
      s.getRange("B1").formulas = [["=A1"]]; // the loop A1, B1
      await context.sync();
      const afterEntry = await read(context, s, "A1:B1"); // B1 1 (A1's stored value), 0, or an error at the entry?
      context.application.calculate(Excel.CalculationType.recalculate);
      await context.sync();
      const afterF9 = await read(context, s, "A1:B1");
      return { iterationAtStart, iterationInCase, afterEntry, afterF9 };
    } finally {
      iteration.enabled = iterationAtStart;
      await context.sync();
    }
  },
  // V1 (Automatic): a volatile cell and an edit nothing reads.
  V1: async (context, s) => {
    s.getRange("F1").formulas = [["=RAND()"]];
    await context.sync();
    const r0 = await read(context, s, "F1");
    s.getRange("A9").values = [[1]];
    await context.sync();
    const afterEdit = await read(context, s, "F1"); // S1 says it changes
    return { r0, afterEdit };
  },
};

// ---- P: precedents and dependents ------------------------------------------------------------------------------
// One workbook: sheets "S", "T" and "U". The formula cells are in S column H; what each tests is in README.md.
// It is built in groups, each sent by its own sync: a group the host refuses is named in `buildErrors` and the
// others still answer.
async function buildP(context) {
  const wb = context.workbook;
  const buildErrors = [];
  const group = async (name) => {
    try {
      await context.sync();
    } catch (e) {
      buildErrors.push({ group: name, error: e.code || String(e), message: e.message });
    }
  };
  const s = wb.worksheets.getActiveWorksheet();
  s.name = "S";
  const t = wb.worksheets.add("T");
  const u = wb.worksheets.add("U");
  s.getRange("A1:A5").values = [[1], [2], [3], [4], [5]];
  s.getRange("B1:B5").values = [[10], [20], [30], [40], [50]];
  s.getRange("C1:C3").values = [[100], [200], [300]];
  s.getRange("D1").values = [["C1"]];
  t.getRange("A1").values = [[7]];
  u.getRange("A1").values = [[8]];
  await group("sheets and inputs");
  // One sync per name, as for the formulas: a name the host refuses is named, and the others are still added.
  const names = { rng: "=S!$C$1:$C$3", calc: "=SUM(S!$C$1:$C$3)*2", konst: "=42", MyFn: "=LAMBDA(x,x*S!$C$1)" };
  for (const [name, formula] of Object.entries(names)) {
    wb.names.add(name, formula);
    await group(`name ${name}`);
  }
  s.getRange("P1:R3").values = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
  await group("inputs P1:R3");
  const f = {
    H1: "=A1+B1", //                 P1  two cells
    H2: "=SUM(A1:A5)", //            P2  a range
    H3: "=SUM(A:A)", //              P3  a whole column
    H4: "=T!A1+1", //                P4  another sheet
    H5: "=SUM(rng)", //              P5  a name that is a range
    H6: "=calc+1", //                P6  a name whose body is a formula
    H7: "=INDIRECT(\"C1\")", //      P7a INDIRECT of a constant
    H8: "=INDIRECT(D1)", //          P7b INDIRECT of a cell
    H9: "=SUM(OFFSET(C1,1,0,2,1))", // P8 OFFSET: the argument, the target, or both
    H10: "=SUMIF(A1:A3,\">0\",B1)", // P9 SUMIF's resized sum range
    H11: "=IF(TRUE,A1,B1)", //       P10 the branch not taken
    H12: "=SUM(S:U!A1)", //          P12 a 3-D span
    H13: "=@A1:A5", //               P20 implicit intersection (H13 is outside rows 1-5 of column A: #VALUE!)
    H14: "=LET(x,A1,x+B1)", //       P21 LET
    H15: "=1+2", //                  P17 no reference
    H16: "=konst+A2", //             P5b a name that is a constant
    H17: "=H1+1", //                 P14 a chain: H17 -> H1 -> A1, B1
    H18: "=SUM((A1:A2,B4:B5))", //   P18 a union
    H19: "=SUM(A1:B3 B2:C5)", //     P19 an intersection
    H21: "=MyFn(A1)", //             P22 a named LAMBDA whose body reads S!C1
    H22: "=SUM(P1:Q2:R3)", //        P23 a range operator: is R1, inside the bounding rectangle only, a precedent?
    H23: "=SUMIF(A1:A3,\">0\",B1:B5)", // P9b a sum range longer than the criteria range: is B5 a precedent?
  };
  // One sync per formula: a formula the host refuses is named, and the others are still entered.
  for (const [address, formula] of Object.entries(f)) {
    s.getRange(address).formulas = [[formula]];
    await group(`formula ${address}`);
  }
  // P20b: the same implicit intersection on a row of its range (V3 is in row 3 of A1:A5).
  s.getRange("V3").formulas = [["=@A1:A5"]];
  await group("formula V3");
  // P11, a spill: J1 spills J1:J3; K1 reads the spill reference, K2 a spill cell, K3 the anchor.
  s.getRange("J1").formulas = [["=SEQUENCE(3)+A1"]];
  s.getRange("K1").formulas = [["=SUM(J1#)"]];
  s.getRange("K2").formulas = [["=J2*2"]];
  s.getRange("K3").formulas = [["=J1*2"]];
  await group("spill J1 and its readers");
  // P11f: a blocked anchor (R11 is in its way) and a reader of its spill reference.
  s.getRange("R11").values = [["x"]];
  s.getRange("R10").formulas = [["=SEQUENCE(3)"]];
  s.getRange("S10").formulas = [["=SUM(R10#)"]];
  await group("blocked spill R10");
  // P24: a conditional format's rule and a data validation's formula that read A1 (are they dependents of A1?).
  const cf = s.getRange("T1").conditionalFormats.add(Excel.ConditionalFormatType.custom);
  cf.custom.rule.formula = "=$A$1>0";
  cf.custom.format.font.color = "#FF0000";
  s.getRange("U1").dataValidation.rule = { custom: { formula: "=$A$1>0" } };
  await group("conditional format and data validation");
  // P13, a table and a structured reference.
  s.getRange("M1:N3").values = [["Item", "Qty"], ["a", 1], ["b", 2]];
  const table = s.tables.add("M1:N3", true);
  table.name = "Tb";
  s.getRange("H20").formulas = [["=SUM(Tb[Qty])"]];
  await group("table Tb");
  // P13b: [#This Row] on a data row of the table (W2) and off its rows (W5, #VALUE!).
  s.getRange("W2").formulas = [["=Tb[@Qty]*2"]];
  s.getRange("W5").formulas = [["=Tb[@Qty]*2"]];
  await group("this-row references W2 and W5");
  // P11h: a spill reference to a cell that is no anchor: X1 holds a one-cell formula, X2 a literal.
  s.getRange("X1").formulas = [["=1+1"]];
  s.getRange("X2").values = [[5]];
  s.getRange("Y1").formulas = [["=SUM(X1#)"]];
  s.getRange("Y2").formulas = [["=SUM(X2#)"]];
  await group("spill references to non-anchors Y1 and Y2");
  // P15b: a range target that holds one of its own precedents (X5 reads X4).
  s.getRange("X4").formulas = [["=A1+1"]];
  s.getRange("X5").formulas = [["=X4+1"]];
  await group("a chain inside one range X4:X5");
  return { s, t, u, buildErrors };
}

async function addresses(context, getAreas) {
  try {
    const areas = getAreas();
    areas.load("addresses");
    await context.sync();
    return areas.addresses;
  } catch (e) {
    return { error: e.code || String(e) };
  }
}

const P = {
  // Direct and all precedents of every formula cell.
  PREC: async (context) => {
    const { s, buildErrors } = await buildP(context);
    const out = { buildErrors };
    const cells = [];
    for (let r = 1; r <= 23; r++) cells.push(`H${r}`);
    cells.push("J1", "J2", "K1", "K2", "K3", "A1", "R10", "S10", "V3", "W2", "W5", "Y1", "Y2");
    for (const a of cells) {
      out[a] = {
        direct: await addresses(context, () => s.getRange(a).getDirectPrecedents()),
        all: await addresses(context, () => s.getRange(a).getPrecedents()),
      };
    }
    out["H1:H2"] = { direct: await addresses(context, () => s.getRange("H1:H2").getDirectPrecedents()) };
    out["X4:X5"] = {
      direct: await addresses(context, () => s.getRange("X4:X5").getDirectPrecedents()),
      all: await addresses(context, () => s.getRange("X4:X5").getPrecedents()),
    };
    return out;
  },
  // Direct and all dependents of the cells the formulas read.
  DEP: async (context) => {
    const { s, t, buildErrors } = await buildP(context);
    const out = { buildErrors };
    const asked = ["A1", "A2", "A3", "A5", "A9", "B1", "B2", "B3", "B5", "C1", "C2", "C3", "D1", "J1", "J2", "J3", "N2", "H1"];
    asked.push("R1", "R10", "R11", "X1", "X2"); // X1 and X2 for P11h: is Y1 a dependent of X1, and Y2 none of X2?
    for (const a of asked) {
      out[`S!${a}`] = {
        direct: await addresses(context, () => s.getRange(a).getDirectDependents()),
        all: await addresses(context, () => s.getRange(a).getDependents()),
      };
    }
    out["T!A1"] = { direct: await addresses(context, () => t.getRange("A1").getDirectDependents()) };
    out["S!A1:A2"] = { direct: await addresses(context, () => s.getRange("A1:A2").getDirectDependents()) };
    out["S!X4:X5"] = {
      direct: await addresses(context, () => s.getRange("X4:X5").getDirectDependents()),
      all: await addresses(context, () => s.getRange("X4:X5").getDependents()),
    };
    return out;
  },
};

async function run(id) {
  await Excel.run(async (context) => {
    const s = context.workbook.worksheets.getActiveWorksheet();
    const env = { build: Office.context.diagnostics.version, platform: Office.context.diagnostics.platform };
    let fresh = false;
    try {
      // One case a workbook: a case reads cells that an earlier case would have left filled, and its line would
      // not show it. A workbook that holds any value is refused, untouched.
      context.workbook.worksheets.load("items/name");
      await context.sync();
      const used = context.workbook.worksheets.items.map((sheet) => sheet.getUsedRangeOrNullObject(true));
      used.forEach((u) => u.load("address"));
      await context.sync();
      const full = used.filter((u) => !u.isNullObject).map((u) => u.address);
      if (full.length) throw { code: "WorkbookNotFresh", message: `run in a new workbook: ${full.join(", ")}` };
      fresh = true;
      // The mode is the application's on the desktop (research note, section 3): say what it was, then pin it.
      const app = context.application;
      app.load("calculationMode");
      app.iterativeCalculation.load("enabled");
      await context.sync();
      env.modeAtStart = app.calculationMode;
      env.iterationAtStart = app.iterativeCalculation.enabled; // application-wide on the desktop, as the mode is
      await setMode(context, Excel.CalculationMode.automatic);
      const data = M[id] ? await M[id](context, s) : await P[id](context);
      log(id, { ...env, data });
    } catch (e) {
      log(id, { ...env, error: e.code || String(e), message: e.message });
    } finally {
      if (fresh) {
        context.application.calculationMode = Excel.CalculationMode.automatic;
        await context.sync();
      }
    }
  });
}
// run("M1") ... run("M20"), run("V1"), run("PREC"), run("DEP"): one fresh workbook each.

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
  for (const id of [...Object.keys(M), ...Object.keys(P)]) {
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
