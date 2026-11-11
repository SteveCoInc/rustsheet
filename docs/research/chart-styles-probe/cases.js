// The data every chart of the Chart Styles probe is made from: A1:D5 on each type's sheet.
// Row 1 is the series names, column A the categories (text), the rest numbers. Every type charts the same range.
export const SHEETS = [
  "Clustered Column",
  "Line with Markers",
  "Pie",
  "Clustered Bar",
  "Area",
  "Doughnut",
  "Scatter",
  "Combo",
];
export const DATA = [
  ["Region", "Q1", "Q2", "Q3"],
  ["North", 120, 135, 150],
  ["South", 90, 80, 105],
  ["East", 60, 75, 70],
  ["West", 45, 50, 65],
];
