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

  test("links each provision that moved it to the card that reads it statewide (#694)", () => {
    const supplement = document.querySelector('#by-line th a[href="/what-changed#targeted-assistance"]');
    expect(text(supplement)).toBe("Supplemental targeted assistance (repealed by H.B. 96)");
    expect(document.querySelector('#phase-in a[href="/what-changed#phase-in"]')).toBeTruthy();
    expect(document.querySelector('#drivers a[href="/what-changed#reappraisal"]')).toBeTruthy();
  });

  test("lands every one of those links on a card the built page has", () => {
    const changed = read("what-changed.html");
    const fragments = [...document.querySelectorAll('a[href^="/what-changed#"]')].map(
      (a) => a.getAttribute("href")!.split("#")[1]!,
    );
    expect(fragments).toHaveLength(3);
    for (const id of fragments) expect(changed.getElementById(id), id).toBeTruthy();
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
    expect(card.querySelector('a[href="/what-changed"]')).toBeTruthy();
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
    expect(card!.querySelector('a[href="/what-changed"]')).toBeTruthy();
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

describe("/what-changed's distribution card (#697)", () => {
  const card = read("what-changed.html").querySelector("#distribution");

  test("draws a histogram for each step in both measures, before the table", () => {
    expect(card).toBeTruthy();
    const panels = [...card!.querySelectorAll(".measure-panel")];
    expect(panels.map((p) => p.getAttribute("data-measure"))).toEqual(["total", "foundation"]);
    for (const panel of panels) {
      const charts = panel.querySelectorAll('[data-chart^="change-"]');
      expect(charts).toHaveLength(3);
      for (const chart of charts) {
        // Every width drawn, the phone's among them.
        expect(chart.querySelector('.chart-at[data-at="narrow"] svg.plot .hist')).toBeTruthy();
        expect(chart.querySelector('.chart-at[data-at="wide"] svg.plot .hist')).toBeTruthy();
      }
    }
  });

  test("draws the districts held at base on foundation aid as their own mark", () => {
    const foundation = card!.querySelector('.measure-panel[data-measure="foundation"]')!;
    const stems = [...foundation.querySelectorAll('.chart-at[data-at="wide"] .hist-held > *')];
    expect(stems).toHaveLength(3);
    expect(stems.map((s) => s.getAttribute("data-hover"))).toContain(
      "219 districts did not move: foundation aid, FY2026 to FY2027 models",
    );
  });
});
