/**
 * The Allen County figures, read off the built pages (#639, #642's exit gate).
 *
 * `tests/unit/change.spec.ts` pins the renderers; this pins what the build actually wrote, so a
 * page that dropped a card, or a feed that reached the page with a different year, fails here.
 */
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { parseHTML } from "linkedom";
import { describe, expect, test } from "vitest";

import { DIST } from "./artefact.ts";

const read = (path: string) =>
  parseHTML(readFileSync(join(DIST, path), "utf8")).document;
const text = (n: Element | null | undefined) => (n?.textContent ?? "").replace(/\s+/g, " ").trim();

describe("Lima City's change tab", () => {
  const document = read("district/044222/change.html");

  test("carries its five cards, each dated", () => {
    for (const id of ["by-line", "phase-in", "drivers", "transfers", "neighbors"]) {
      const card = document.querySelector(`#${id}`);
      expect(card, id).toBeTruthy();
      expect(card!.querySelector(".year-chip"), id).toBeTruthy();
    }
  });

  test("names the repealed supplement as its own line, and both measures", () => {
    const rows = [...document.querySelectorAll("#by-line tbody th")].map(text);
    expect(rows).toContain("Supplemental targeted assistance (repealed by H.B. 96)");
    expect(rows).toContain("Foundation aid (the narrow measure)");
    expect(rows).toContain("Total state support (the wide measure)");
    expect(text(document.querySelector("#by-line"))).toContain("$1,859,367");
  });

  test("says in words that it is at its base", () => {
    const card = text(document.querySelector("#phase-in"));
    expect(card).toContain("$35,262,743");
    expect(card).toContain("the phase-in has almost nothing to phase in");
  });

  test("shows FY2026 transfers as unpublished, and FY2027's items", () => {
    const card = document.querySelector("#transfers")!;
    const cells = [...card.querySelectorAll("tbody tr")].map((row) =>
      [...row.querySelectorAll("td")].map(text),
    );
    // The middle column is the year whose file publishes no transfers.
    for (const row of cells.slice(1)) expect(row[1]).toBe("not published");
    expect(text(card)).toContain("−$269,453");
    expect(text(card)).toContain("−$616,426");
    expect(text(card)).not.toMatch(/\$0\b/);
  });

  test("marks itself among Allen County's nine on both measures", () => {
    const card = document.querySelector("#neighbors")!;
    expect(card.querySelectorAll(".measure-panel")).toHaveLength(2);
    expect(text(card)).toContain("the 4th-largest rise of 9 in Allen County");
    expect(card.querySelector('input[data-measure="total"]')?.hasAttribute("checked")).toBe(true);
  });
});

describe("Elida Local's dashboard", () => {
  const document = read("district/045773.html");

  test("ranks it in its county and links to the change tab", () => {
    const card = document.querySelector("#biennium")!;
    expect(text(card)).toContain("the 3rd-largest fall of 9 in Allen County");
    expect(card.querySelector('a[href="/district/045773/change"]')).toBeTruthy();
  });
});

describe("Allen County's who-gained card (#640)", () => {
  const document = read("county/allen.html");
  const card = document.querySelector("#who-gained");

  test("is on the built page, dated, with both measures", () => {
    expect(card).toBeTruthy();
    expect(card!.querySelector(".year-chip")).toBeTruthy();
    expect(card!.querySelectorAll("table[data-sortable]")).toHaveLength(2);
  });

  test("reads each of #642's four answers", () => {
    const words = text(card);
    expect(words).toContain("The 2 that fell: Elida Local and Lima City.");
    expect(words).toContain("Supplemental targeted assistance (repealed by H.B. 96)");
    expect(words).toContain("Without supplemental targeted assistance");
    expect(words).toContain("Delphos City is at the formula's minimum state share");
  });

  test("loads the sort script", () => {
    expect(document.querySelector('script[type="module"]')).toBeTruthy();
  });
});

describe("every county's who-gained card (#661)", () => {
  const pages = readdirSync(join(DIST, "county")).filter((f) => f.endsWith(".html"));

  test("draws each measure's districts, wherever there are three to compare", () => {
    let drawn = 0;
    for (const page of pages) {
      const card = read(`county/${page}`).querySelector("#who-gained");
      if (!card) continue;
      for (const panel of card.querySelectorAll(".measure-panel")) {
        // Below three the chart would compare a district with itself or one other (#651).
        if (panel.querySelectorAll("tbody tr").length < 3) continue;
        expect(panel.querySelector('[data-chart^="neighbors-"]'), page).toBeTruthy();
        drawn++;
      }
    }
    expect(drawn, "the build has counties to draw, or this passed on nothing").toBeGreaterThan(100);
  });
});
