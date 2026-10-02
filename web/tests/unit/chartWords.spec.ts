/**
 * The words a chart writes for itself (#612). See `src/lib/chartWords.ts`.
 */

import { expect, test } from "vitest";

import { axisShort, banned, noted, pointHover, sentence, unstop } from "../../src/lib/chartWords.ts";
import { DIMENSIONS } from "../../src/lib/reach.ts";

test("sentence case moves a lower-case first letter and nothing else", () => {
  expect(sentence("assessed valuation per pupil (log scale)")).toBe(
    "Assessed valuation per pupil (log scale)",
  );
  expect(sentence("local 54%")).toBe("Local 54%");
  for (const kept of ["Formula aid per pupil", "FY2024", "$1,300", "ADM", "−0.0112", ""]) {
    expect(sentence(kept)).toBe(kept);
  }
});

test("a full stop comes off before a join, and only a full stop", () => {
  expect(unstop("from 412 districts down to 0.")).toBe("from 412 districts down to 0");
  expect(unstop("ends in a figure 0.5")).toBe("ends in a figure 0.5");
  expect(unstop("no stop")).toBe("no stop");
});

test("an axis goes by the head of its title in a tooltip", () => {
  expect(axisShort("The enrollment term: fewer pupils than in FY2020, in logs")).toBe(
    "enrollment term",
  );
  expect(axisShort("Guarantee written, against the enacted anchor")).toBe("guarantee written");
  // An initialism keeps its capitals; lowering the whole title wrote "enrolled adm".
  expect(axisShort("Enrolled ADM")).toBe("enrolled ADM");
  expect(axisShort("ADM per square mile")).toBe("ADM per square mile");
});

test("a note goes after the reading, and a point's hover takes one pattern", () => {
  expect(noted("2024-25: 1,042", "cut from two rankings")).toBe(
    "2024-25: 1,042 — cut from two rankings",
  );
  expect(noted("2024-25: 1,042", undefined)).toBe("2024-25: 1,042");
  expect(
    pointHover(
      "Manchester Local",
      { label: "The enrollment term: fewer pupils, in logs", value: "+0.2270" },
      { label: "The per-pupil term: the formula against the regime", value: "−0.0312" },
      "No majority to take",
    ),
  ).toBe("Manchester Local — enrollment term: +0.2270; per-pupil term: −0.0312 — No majority to take");
});

test("a chart's banned words are caught, and their replacements are not (#658)", () => {
  // The words the issue found on the drawings, each caught.
  for (const old of ["Realized aid per pupil", "realized = formula", "On CE 46%", "On ADM 43%", "Enrolled ADM, the single year"]) {
    expect(banned(old), old).toHaveLength(1);
  }
  // A word that only contains one is not one: "CENTRAL", "ADMINISTRATORS", "Received".
  for (const kept of ["Aid received per pupil", "By access 46%", "Two other district administrators", "CENTRAL OHIO", "Received $114M"]) {
    expect(banned(kept), kept).toEqual([]);
  }
});

test("the reach chart's axes, which no dist scan can see, print none of them", () => {
  // /scenario/reach draws in the browser, so the build's markup never carries these.
  const labels = Object.values(DIMENSIONS).map((d) => d.label);
  expect(labels.length).toBeGreaterThan(5);
  expect(labels.flatMap(banned)).toEqual([]);
});
