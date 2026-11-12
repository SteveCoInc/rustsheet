// Probe request `workbook-locale`: what a regional format and a display language change (SPEC-0046).
// Office.js (Script Lab). Excel for the web is the set of record; a desktop run is a second set, labelled (README.md).
// Run it once per setting of README.md's table, each time in a FRESH workbook: await run();
// It logs one JSON line per group. Every line starts with the environment it was measured in.
// Requirement sets: ExcelApi 1.7 (numberFormatLocal), 1.11 (cultureInfo, the separators), 1.12 (datetimeFormat).
// A call the host lacks, or a write the host refuses, is logged with its error code in place of its answer.

const log = (group, env, data) => console.log(JSON.stringify({ probe: "workbook-locale", group, env, data }));
const code = (e) => ({ error: e.code || String(e), message: e.message });

// ---- ENV: which settings this run was made under ---------------------------------------------------------------
async function environment(context) {
  const env = { build: Office.context.diagnostics.version, platform: Office.context.diagnostics.platform };
  env.displayLanguage = Office.context.displayLanguage;
  env.contentLanguage = Office.context.contentLanguage;
  try {
    const app = context.application;
    app.load("decimalSeparator,thousandsSeparator,useSystemSeparators");
    app.cultureInfo.load("name");
    app.cultureInfo.numberFormat.load("numberDecimalSeparator,numberGroupSeparator");
    app.cultureInfo.datetimeFormat.load("dateSeparator,longDatePattern,shortDatePattern,timeSeparator,longTimePattern");
    await context.sync();
    env.culture = app.cultureInfo.name;
    env.excelSeparators = [app.decimalSeparator, app.thousandsSeparator, app.useSystemSeparators];
    env.systemSeparators = [app.cultureInfo.numberFormat.numberDecimalSeparator, app.cultureInfo.numberFormat.numberGroupSeparator];
    const d = app.cultureInfo.datetimeFormat;
    env.dates = [d.dateSeparator, d.shortDatePattern, d.longDatePattern, d.timeSeparator, d.longTimePattern];
  } catch (e) {
    env.cultureError = code(e);
  }
  return env;
}

// One cell: everything the API can say about it.
async function look(context, sheet, address) {
  const r = sheet.getRange(address);
  r.load("values,valueTypes,formulas,formulasLocal,numberFormat,numberFormatLocal,text");
  await context.sync();
  return {
    value: r.values[0][0], type: r.valueTypes[0][0], formula: r.formulas[0][0], formulaLocal: r.formulasLocal[0][0],
    format: r.numberFormat[0][0], formatLocal: r.numberFormatLocal[0][0], text: r.text[0][0],
    // The displayed text's code points: a group separator can be U+0020, U+00A0 or U+202F (inventory C11).
    textCodes: Array.from(String(r.text[0][0]), (c) => c.codePointAt(0).toString(16)),
  };
}

// Writes `property` of one cell and looks at it. A refused write is an answer too.
async function put(context, sheet, address, property, content) {
  try {
    sheet.getRange(address)[property] = [[content]];
    await context.sync();
    return await look(context, sheet, address);
  } catch (e) {
    return code(e);
  }
}

// ---- V: formulas stored in their en-US form, calculated under the settings (inventory groups B, C9, E, F, G) -----
// A1:A3 hold 1.5, 2.5, "WAHR"; B1 holds the date 2001-03-03 as a serial (a Saturday).
const V = {
  B1a: '=VALUE("1,5")', B1b: '=VALUE("1.5")', B1c: '=VALUE("1.234,5")', B1d: '=VALUE("1,234.5")', B1e: '=VALUE("5 €")',
  B1f: '=VALUE("$5")', B1g: '=VALUE("5%")', B1h: '=VALUE("1 234,5")',
  // The French group separator: a no-break space (U+00A0) or a narrow one (U+202F)?
  B1i: '=VALUE("1\u00a0234,5")', B1j: '=VALUE("1\u202f234,5")',
  B2a: '=DATEVALUE("1/2/2001")', B2b: '=DATEVALUE("1.2.2001")', B2c: '=DATEVALUE("1-2-2001")', B2d: '=DATEVALUE("2001-02-01")',
  B2e: '=DATEVALUE("1 March 2001")', B2f: '=DATEVALUE("1 März 2001")', B2g: '=DATEVALUE("1 mars 2001")',
  B2h: '=DATEVALUE("1 Mrz 2001")', B2i: '=TIMEVALUE("1:30 PM")', B2j: '=TIMEVALUE("13:30")',
  B3a: '="1,5"+0', B3b: '="1.5"+0', B3c: '="1.2.2001"+0', B3d: '="1/2/2001"+0',
  B4a: '=COUNTIF(A1:A2,">1,5")', B4b: '=COUNTIF(A1:A2,">1.5")', B4c: '=COUNTIF(A3,"WAHR")', B4d: '=COUNTIF(A3,"TRUE")',
  B5a: '=IF("WAHR",1,2)', B5b: '=IF("TRUE",1,2)', B5c: '=IF("VRAI",1,2)',
  B6a: '=NUMBERVALUE("1,5")', B6b: '=NUMBERVALUE("1.5")', B6c: '=NUMBERVALUE("1.234,5")',
  B7a: '=DAYS("3/2/2001","1/2/2001")',
  C9a: '=1.5&""', C9b: '=TRUE&""', C9c: '=TEXT(1.5,"General")', C9d: '=TEXT(1.5,"Standard")', C9e: '=B1&""',
  E1a: '=TEXT(B1,"dd.mm.yyyy")', E1b: '=TEXT(B1,"TT.MM.JJJJ")', E1c: '=TEXT(B1,"jj/mm/aaaa")', E1d: '=TEXT(B1,"dd/mm/aaaa")',
  E1e: '=TEXT(B1,"gg/mm/aaaa")', E1f: '=TEXT(B1,"yyyy")', E1g: '=TEXT(B1,"JJJJ")', E1h: '=TEXT(0.5625,"hh:mm")',
  E2a: '=TEXT(1234.5,"#,##0.00")', E2b: '=TEXT(1234.5,"#.##0,00")', E2c: '=TEXT(1234.5,"# ##0,00")', E2d: '=TEXT(1234.5,"0.00")',
  E2e: '=TEXT(1234.5,"0,00")',
  C5a: '=TEXT(B1,"mmmm")', C5b: '=TEXT(B1,"mmm")', C5c: '=TEXT(B1,"dddd")', C5d: '=TEXT(B1,"ddd")',
  C5e: '=TEXT(B1,"MMMM")', C5f: '=TEXT(B1,"TTTT")',
  G2a: '=TEXT(B1,"[$-407]mmmm")', G2b: '=TEXT(B1,"[$-40C]dddd")', G2c: '=TEXT(B1,"[$-409]mmmm")',
  E4a: '=TEXT(-1,"[Red]0")', E4b: '=TEXT(-1,"[Rot]0")',
  F1a: '=DOLLAR(1234.5)', F1b: '=DOLLAR(-1234.5)', F1c: '=DOLLAR(1234.5,0)',
  F2a: '=FIXED(1234.5)', F2b: '=FIXED(1234.5,1,TRUE)',
  F3a: '=UPPER("straße")', F3b: '=LOWER("ÄÖÜ")', F3c: '=PROPER("école élève")',
  F4a: '="ä"<"b"', F4b: '="ß"="ss"', F4c: '="é"="e"', F4d: '=MATCH("z",{"a","ä","b","z"},1)', F4e: '="a"<"B"',
  F5a: '=WEEKDAY(B1)', F5b: '=WEEKNUM(B1)',
  D9a: '=FORMULATEXT(H1)', D10a: '=CELL("row",A1)', D10b: '=CELL("Zeile",A1)', D10c: '=CELL("address",A1)',
  // The results of CELL and INFO, not only their type arguments: a format code, a type letter, a mode's name.
  D10d: '=CELL("format",B1)', D10e: '=CELL("type",A1)', D10f: '=INFO("recalc")',
  // R1C1 letters where they reach a value (the UI's own letters are read by a person).
  D8a: '=ADDRESS(1,1,1,FALSE)', D8b: '=INDIRECT("R1C1",FALSE)', D8c: '=INDIRECT("Z1S1",FALSE)', D8d: '=INDIRECT("L1C1",FALSE)',
  C10a: '=TEXT(B1,"m/d/yyyy")',
  F7a: '=ARRAYTOTEXT({1.5,2;"a",TRUE})', F7b: '=ARRAYTOTEXT({1.5,2;"a",TRUE},1)', F7c: '=VALUETOTEXT(1.5)', F7d: '=VALUETOTEXT(1.5,1)',
  G1a: '=ROUND(2.675,2)', G1b: '=1/3', G1c: '=DATE(2001,3,3)', G1d: '=1/0',
  G4a: '=FIND("ß","straße")', G4b: '=SEARCH("s?r","straße")', G4c: '=EXACT("a","A")', G5a: '=LENB("é")',
};

// ---- L: en-US formulas whose localized text is the answer (inventory group D) -----------------------------------
const L = {
  D1a: "=SUM(1.5,2)", D1b: "=IF(TRUE,1,2)", D1c: "=VLOOKUP(1,A1:B2,2,FALSE)", D1d: "=IFERROR(1/0,0)",
  D1e: "=XLOOKUP(1,A1:A2,A1:A2)", D1f: "=SUMIFS(A1:A2,A1:A2,1)", D1g: "=TODAY()", D1h: "=CONCAT(1,2)",
  D1i: "=LET(x,1,x+1)", D1j: "=LAMBDA(x,x+1)(1)", D1k: "=SEQUENCE(2)", D1l: "=TEXTJOIN(1,TRUE,2)",
  D3a: "=SUM((A1,A2))", D3b: "=SUM(A1:A2 A1:B1)", D4a: "=1.5+2.25E+3",
  D5a: "={1,2;3,4}", D5b: "={1.5,2.5}", D5c: '={"a","b";TRUE,#N/A}',
  D6a: "=TRUE", D6b: "=#N/A", D6c: "=IFERROR(#VALUE!,#REF!)", D6d: "=#DIV/0!", D6e: "=#NAME?", D6f: "=#NUM!", D6g: "=#NULL!",
};

// ---- T: strings typed into a cell, by both write routes (inventory group A, and unknown U1) ---------------------
const T = [
  "1,5", "1.5", "1.234,5", "1,234.5", "1 234,5", "1\u00a0234,5", "1\u202f234,5", "1.234", "1,234", "1e3", "-1,5", "(1,5)",
  "1/2/2001", "1.2.2001", "1-2-2001", "2001-02-01", "1/2", "1.2.", "1.2", "13/1/2001", "1/13/2001",
  "1 March 2001", "1 März 2001", "1 mars 2001", "1 marzo 2001", "1 Mrz 2001", "1 Mär 2001", "März 2001",
  "13:30", "1:30 PM", "1:30 nachm.", "13.30",
  "5 €", "€ 5", "5€", "$5", "£5", "5 EUR",
  "5%", "5 %",
  "WAHR", "TRUE", "VRAI", "VERDADERO", "VERO", "wahr",
  "#WERT!", "#VALUE!", "#NV", "#N/A", "#DIV/0!",
  "=SUMME(1;2)", "=SUM(1;2)", "=SUM(1,2)", "=MOYENNE(1;2)", "=SUMA(1;2)", "=SOMMA(1;2)",
  "=1,5+1", "=1.5+1", "=WENN(WAHR;1;2)", "=IF(TRUE,1,2)",
  "={1.2;3.4}", "={1\\2;3\\4}", "={1,2;3,4}", "={1;2}", "={1.2}",
  "=SUMME((A1;A2))", "=TEXT(1,5;\"0,00\")", "=TEXT(1.5,\"0.00\")",
];

// ---- D: stored number formats on stored values (inventory group C) ----------------------------------------------
// Each: [value or en-US formula, en-US format code].
const D = {
  C1a: [1234.5, "General"], C1b: [0.000012345, "General"], C1c: [123456789012, "General"],
  C2a: [1234567.891, "#,##0.00"], C2b: [-1234.5, "#,##0.00_);(#,##0.00)"], C2c: [0.5, "0%"], C2d: [1234.5, "0.00E+00"],
  C3a: [36953, "m/d/yyyy"], C3b: [36953.5625, "m/d/yyyy h:mm"], C3c: [36953, "d-mmm-yy"], C3d: [36953, "d-mmm"],
  C3e: [36953, "mmm-yy"], C3f: [0.5625, "h:mm AM/PM"], C3g: [0.5625, "h:mm:ss"], C3h: [36953, "mm-dd-yy"],
  C4a: [36953, "[$-F800]dddd, mmmm dd, yyyy"], C4b: [0.5625, "[$-F400]h:mm:ss AM/PM"],
  C5a: [36953, "mmmm"], C5b: [36953, "mmm"], C5c: [36953, "dddd"], C5d: [36953, "ddd"],
  G2a: [36953, "[$-407]dddd, d. mmmm yyyy"], G2b: [36953, "[$-40C]dddd d mmmm yyyy"], G2c: [36953, "[$-409]mmmm d, yyyy"],
  C6a: [1234.5, '"$"#,##0.00'], C6b: [-1234.5, '_("$"* #,##0.00_);_("$"* (#,##0.00);_("$"* "-"??_);_(@_)'],
  C6c: [1234.5, "[$€-407]#,##0.00"], C6d: [1234.5, "#,##0.00 [$€-407]"],
  C8a: ["=TRUE", "General"], C8b: ["=1/0", "General"], C8c: ["=NA()", "General"], C8d: ["=1=2", "General"],
  C12a: [0.125, "0.00%"], C12b: [-0.5, "0%"],
};

async function run() {
  await Excel.run(async (context) => {
    const s = context.workbook.worksheets.getActiveWorksheet();
    const env = await environment(context);
    log("ENV", env, {});

    // Inputs, written as numbers and en-US formulas so that no setting can change them.
    s.getRange("A1:A3").values = [[1.5], [2.5], ["WAHR"]];
    s.getRange("B1").values = [[36953]]; // 2001-03-03, a Saturday
    s.getRange("H1").formulas = [["=SUM(1.5,2)"]];
    await context.sync();

    let row = 10;
    const out = { V: {}, L: {}, T: {}, D: {} };
    for (const [id, formula] of Object.entries(V)) out.V[id] = { formula, ...(await put(context, s, `J${row++}`, "formulas", formula)) };
    log("V", env, out.V);
    for (const [id, formula] of Object.entries(L)) out.L[id] = { formula, ...(await put(context, s, `L${row++}`, "formulas", formula)) };
    log("L", env, out.L);
    // T: the same string by the three write routes. `formulasLocal` is the one Microsoft says is locale-aware.
    for (const typed of T) {
      out.T[typed] = {
        formulasLocal: await put(context, s, `N${row}`, "formulasLocal", typed),
        values: await put(context, s, `O${row}`, "values", typed),
        formulas: await put(context, s, `P${row}`, "formulas", typed),
      };
      row++;
    }
    log("T", env, out.T);
    for (const [id, [content, format]] of Object.entries(D)) {
      const a = `R${row++}`;
      try {
        if (typeof content === "string") s.getRange(a).formulas = [[content]];
        else s.getRange(a).values = [[content]];
        s.getRange(a).numberFormat = [[format]];
        await context.sync();
        out.D[id] = { content, format, ...(await look(context, s, a)) };
      } catch (e) {
        out.D[id] = { content, format, ...code(e) };
      }
    }
    log("D", env, out.D);

    // E5: a format code written in the locale, through numberFormatLocal, read back in its stored form.
    const local = {};
    for (const typedCode of ["TT.MM.JJJJ", "jj/mm/aaaa", "dd/mm/aaaa", "gg/mm/aaaa", "dd.mm.yyyy", "#.##0,00", "# ##0,00", "#,##0.00", "Standard", "General", "[Rot]0", "[Red]0"]) {
      const a = `T${row++}`;
      try {
        s.getRange(a).values = [[36953.5]];
        s.getRange(a).numberFormatLocal = [[typedCode]];
        await context.sync();
        local[typedCode] = await look(context, s, a);
      } catch (e) {
        local[typedCode] = code(e);
      }
    }
    log("E5", env, local);

    // D12: the separator of an inline list in a data validation, written both ways and read back.
    const lists = {};
    for (const source of ["a,b,c", "a;b;c"]) {
      const a = `V${row++}`;
      try {
        s.getRange(a).dataValidation.rule = { list: { inCellDropDown: true, source } };
        await context.sync();
        const validation = s.getRange(a).dataValidation;
        validation.load("rule");
        await context.sync();
        lists[source] = validation.rule.list.source;
      } catch (e) {
        lists[source] = code(e);
      }
    }
    log("D12", env, lists);

    // F6: the names Excel gives to new things (the display language's).
    const names = {};
    try {
      const sheet = context.workbook.worksheets.add();
      sheet.load("name");
      sheet.getRange("A1:B2").values = [[1, 2], [3, 4]];
      const table = sheet.tables.add("A1:B2", false);
      table.load("name");
      const header = table.getHeaderRowRange();
      header.load("values");
      await context.sync();
      names.sheet = sheet.name;
      names.table = table.name;
      names.headers = header.values[0];
      // D7: a structured reference as the language shows it.
      names.structured = await put(context, sheet, "D1", "formulas", `=SUM(${table.name}[[#All],[${header.values[0][0]}]])`);
      names.thisRow = await put(context, sheet, "D3", "formulas", `=${table.name}[@[${header.values[0][0]}]]`);
    } catch (e) {
      names.error = code(e);
    }
    log("F6", env, names);
  });
}
// await run();
