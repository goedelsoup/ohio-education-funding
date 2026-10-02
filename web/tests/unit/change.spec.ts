/**
 * The `/change` tab's arithmetic, pinned on Allen County (#639).
 *
 * Allen is the county #638's crate tests already pin — nine districts, two of them held at their
 * base on foundation aid in both model years, and one, Lima City, whose formula calculates within
 * a fraction of a percent of its base. Every figure here is one of the department's own lines, so
 * the pins are the files' figures rather than this module's opinion of them.
 */
import { expect, test } from "vitest";

import { renderBiennium } from "../../src/lib/biennium.ts";
import {
  countyRank,
  largestMove,
  MEASURES,
  namedLines,
  rankPhrase,
  renderByLine,
  renderDrivers,
  renderNeighbors,
  renderPhaseIn,
  renderTransfers,
  signedPct,
  yearChange,
  yearsOf,
} from "../../src/lib/change.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import type { District } from "../../src/lib/types.ts";

const { bundle } = loadFeed();
const districts = bundle.districts;
const byIrn = (irn: string): District => districts.find((d) => d.irn === irn)!;

const LIMA = byIrn("044222");
const ELIDA = byIrn("045773");
const BATH = byIrn("045765");
const PERRY = byIrn("045781");
const DELPHOS = byIrn("043885");

test("the named lines are total state support in every year, for every district", () => {
  expect(districts).toHaveLength(609);
  for (const d of districts) {
    const total = MEASURES.total.levels(d.biennium);
    for (const i of [0, 1, 2] as const) {
      const sum = namedLines(d.biennium).reduce((acc, line) => acc + line.levels[i], 0);
      expect(Math.abs(sum - total[i]), `${d.irn} FY index ${i}`).toBeLessThan(0.03);
    }
  }
});

test("Allen County ranks on the percentage, within each direction", () => {
  const allen = districts.filter((d) => d.county === "Allen");
  expect(allen).toHaveLength(9);
  expect(rankPhrase(countyRank(BATH, districts, "total"), "Allen")).toBe(
    "the largest fall of 9 in Allen County",
  );
  expect(rankPhrase(countyRank(ELIDA, districts, "total"), "Allen")).toBe(
    "the 3rd-largest fall of 9 in Allen County",
  );
  expect(rankPhrase(countyRank(PERRY, districts, "total"), "Allen")).toBe(
    "the largest rise of 9 in Allen County",
  );
  // Lima rose least of the four that rose, and is ranked among them rather than above the falls.
  expect(rankPhrase(countyRank(LIMA, districts, "total"), "Allen")).toBe(
    "the 4th-largest rise of 9 in Allen County",
  );
  // Delphos and Shawnee are held at their base on foundation aid in both model years.
  expect(rankPhrase(countyRank(DELPHOS, districts, "foundation"), "Allen")).toBe(
    "one of 2 of 9 in Allen County that did not move",
  );
});

test("a district alone in its county is said to be, not ranked first", () => {
  expect(rankPhrase({ direction: "fall", rank: 1, alike: 1, of: 1 }, "Somewhere")).toBe(
    "the only district in Somewhere County",
  );
});

test("Lima's year on year is the department's figures, and its largest line is a supplement", () => {
  const total = yearChange(LIMA.biennium, "total");
  expect(total.dollars).toBeCloseTo(37_945_748.76 - 37_900_888.46, 2);
  expect(signedPct(total.ratio)).toBe("+0.1%");
  const foundation = yearChange(LIMA.biennium, "foundation");
  expect(foundation.dollars).toBeCloseTo(-11_434.71, 2);
  const moved = largestMove(LIMA.biennium)!;
  expect(moved.label).toBe("Base funding supplement");
  expect(moved.change).toBeCloseTo(125_122.87 - 85_158.1, 2);
});

test("the years are the feed's, never typed", () => {
  expect(yearsOf(LIMA.biennium)).toEqual([
    LIMA.biennium.year_baseline,
    LIMA.biennium.year_middle,
    LIMA.biennium.year_terminal,
  ]);
});

test("a year whose file publishes no transfers reads as unpublished, never as zero", () => {
  const html = renderTransfers(LIMA);
  const [, middle] = yearsOf(LIMA.biennium);
  expect(html).toContain("not published");
  expect(html).toContain("not itemized");
  // Lima's FY2027 service center charge, itemized.
  expect(html).toContain("−$269,453");
  // No row carries a zero dollar figure: every absent cell is a word.
  expect(html).not.toContain(">$0<");
  expect(html).toContain(`FY${middle}'s publishes none`);
});

test("the phase-in says in words when a district is at its base or near it", () => {
  const html = renderPhaseIn(LIMA);
  const [first, , last] = yearsOf(LIMA.biennium);
  expect(html).toContain(`FY${first}: at its base.`);
  expect(html).toContain(`FY${last}: at its base.`);
  expect(html).toContain("the phase-in has almost nothing to phase in");
  expect(html).toContain("$35,262,743");
});

test("every card names its measure, dates itself, and says the models are not payments", () => {
  for (const html of [
    renderByLine(LIMA),
    renderPhaseIn(LIMA),
    renderDrivers(LIMA),
    renderTransfers(LIMA),
    renderNeighbors(LIMA, districts),
  ]) {
    expect(html).toContain('class="year-chip');
    expect(html).toMatch(/total state support|foundation aid/i);
    expect(html).toMatch(/nominal dollars/);
  }
  for (const html of [renderByLine(LIMA), renderPhaseIn(LIMA), renderTransfers(LIMA)]) {
    expect(html).toContain("not payments");
  }
});

test("the drivers card ties each input to a measure and divides nothing", () => {
  const html = renderDrivers(ELIDA);
  expect(html).toContain("State share");
  expect(html).toContain("Assessed valuation");
  expect(html).toContain("Enrolled ADM");
  expect(html).toContain("nothing here divides the change between them");
  expect(html).toContain(`tax year ${ELIDA.biennium.valuation_tax_years[0]}`);
});

test("the neighbors card draws both measures, the headline checked, this district marked", () => {
  const html = renderNeighbors(LIMA, districts);
  expect(html).toContain('data-chart="neighbors-total"');
  expect(html).toContain('data-chart="neighbors-foundation"');
  expect(html).toMatch(/data-measure="total"[^>]*checked/);
  expect(html).toContain("4th-largest rise of 9 in Allen County");
});

test("the dashboard card carries the year on year, the rank, the line and the tab", () => {
  const html = renderBiennium(ELIDA, districts);
  const [, middle, last] = yearsOf(ELIDA.biennium);
  expect(html).toContain(`FY${middle} to FY${last}`);
  expect(html).toContain("the 3rd-largest fall of 9 in Allen County");
  expect(html).toContain("the line that moved it most is");
  expect(html).toContain(`href="/district/${ELIDA.irn}/change"`);
});
