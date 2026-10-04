/**
 * `/bounds`'s census, as built: a row counted out of fewer districts than the rest says so (#710).
 *
 * Seven of the thirty-eight bounds can reach fewer than all 609 districts, and the chart ranked
 * every row on its raw count with nothing to say which. Read off the bytes, against the series
 * manifest the page is built from, in every drawing the page carries — one per layout width.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { parseHTML } from "linkedom";
import { describe, expect, test } from "vitest";

import { loadSeriesManifest } from "../../src/lib/corpusSeries.ts";

import { DIST } from "./artefact.ts";

describe("the bounds census", () => {
  const rows = loadSeriesManifest().series.find((s) => s.key === "project/bounds-census")!.rows;
  const whole = Math.max(...rows.map((row) => row.of ?? 0));
  const fewer = rows.filter((row) => row.of !== undefined && row.of !== whole);

  const { document } = parseHTML(readFileSync(join(DIST, "bounds.html"), "utf8"));
  const drawings = [...document.querySelectorAll('[data-chart="bounds-census"] svg')];

  test("every row whose population differs from the panel carries a .rank-note", () => {
    expect(fewer.length, "the census has rows out of fewer districts").toBeGreaterThan(0);
    expect(drawings.length).toBeGreaterThan(0);
    const expected = fewer.map((row) => `of ${row.of}`).sort();
    for (const svg of drawings) {
      const notes = [...svg.querySelectorAll("g.rank-note text")].map((t) => t.textContent ?? "");
      expect(notes.sort()).toEqual(expected);
    }
  });

  test("the floor is named for the rows at zero, and the 1 beside it survives", () => {
    // #709: "under 1" sat over the dots at exactly 1 and the "1" tick was dropped.
    for (const svg of drawings) {
      const foot = [...svg.querySelectorAll("g.axis-foot text")].map((t) => t.textContent ?? "");
      expect(foot).toContain("None");
      expect(foot).toContain("1");
      expect(foot.join(" ")).not.toMatch(/under 1\b/i);
    }
  });
});
