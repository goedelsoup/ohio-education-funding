/**
 * The measure report's arithmetic.
 *
 * `collect` is not tested here and cannot be: it reads `getComputedStyle` and
 * `getBoundingClientRect`, neither of which linkedom implements, and a fake that returned plausible
 * numbers would be testing the fake. It is exercised by running `scripts/measure.ts` against a real
 * build in a real browser, which is the only place those values exist.
 *
 * What IS tested here is everything a later redesign phase will lean on when it turns a row of the
 * report into a threshold — the gap arithmetic, the breach detection, and the fact that an unset
 * threshold grades nothing. Those are the parts that decide whether a phase lands, so they are the
 * parts that need to be right before any of them does.
 */

import { describe, expect, test } from "vitest";
import {
  ROUTES,
  THRESHOLDS,
  WIDTHS,
  formatReport,
  violations,
  widestGap,
  type Measured,
  type Report,
} from "../../src/lib/measure.ts";

/** A row with today's district-page figures, which every case below varies one field of. */
function row(overrides: Partial<Measured> = {}): Measured {
  return {
    route: "/district/043786.html",
    width: 1280,
    // The district page's real census before the redesign: thirteen sizes, eleven of them inside
    // 3.7px, then the jump to the h1. Every threshold below is written against these figures.
    sizes: [11.5, 12, 12.5, 13, 13.5, 14, 14, 14.5, 15, 15, 15.5, 22.5, 24],
    headingRatio: 1.49,
    boxes: { count: 30, decorative: 11, maxDepth: 2 },
    rightAlignedProse: 31,
    measure: { median: 75, p90: 92, max: 110, over78: 3, count: 42 },
    headerHeight: 51,
    firstContentY: 79,
    zeroAdvance: 8.34,
    bodyFont: "ui-sans-serif, system-ui",
    ...overrides,
  };
}

const report = (...rows: Measured[]): Report => ({ measuredAt: "2026-08-25T00:00:00.000Z", rows });

describe("the route set", () => {
  test("names one route per page genre, and no duplicates", () => {
    expect(new Set(ROUTES).size).toBe(ROUTES.length);
  });

  test("every route is a path a static host serves, not an extensionless address", () => {
    // `build.format` is "file", so the artefact holds `district/043786.html`. A route written
    // without the extension 404s against the build and the run silently reports on fewer genres.
    for (const route of ROUTES) {
      expect(route, route).toMatch(/^\/[^?#]*\.html$/);
    }
  });

  test("the narrowest width is the one the chart suite already uses", () => {
    // 375 is where the e2e suite asserts chart text stays at 9px, and the two should not drift:
    // a chrome measurement at one width and a chart measurement at another cannot be reconciled.
    expect(WIDTHS[0]).toBe(375);
  });
});

describe("the widest step in a size census", () => {
  test("is the ratio between the two neighbours furthest apart", () => {
    // 15.2 -> 22.4 is the jump the district page actually has, and it is what "no middle register"
    // means as a number: every other pair on that page is inside 1.15.
    expect(widestGap([11, 12, 13.6, 15.2, 22.4])).toBeCloseTo(22.4 / 15.2, 5);
  });

  test("does not depend on the order it is given the sizes in", () => {
    expect(widestGap([22.4, 11, 15.2, 13.6, 12])).toBeCloseTo(widestGap([11, 12, 13.6, 15.2, 22.4]), 10);
  });

  test("is 1 for a census with no gap in it", () => {
    // The identity, and it matters: a page with one text size must not read as an infinite jump.
    expect(widestGap([15])).toBe(1);
    expect(widestGap([])).toBe(1);
  });

  test("ignores a zero, rather than returning infinity for it", () => {
    // A `font-size: 0` is a real thing on this site — it is how one visually-hidden label is
    // built — and dividing by it would make the gap metric useless on whatever page carries one.
    expect(Number.isFinite(widestGap([0, 12, 15]))).toBe(true);
  });
});

describe("the thresholds a phase has set", () => {
  test("carry one entry per phase that has landed, and no others", () => {
    // #183 shipped the instrument grading nothing; each phase since fills in the row it is about.
    // Asserted as a whole object so a threshold cannot be added without a phase behind it.
    expect(THRESHOLDS).toEqual({
      boxDecorative: 0, // #185
      sizeCount: 10, //    #186
      headingRatio: 2, //  #186
      measureMax: 80, //   #186, font-sensitive
      rightAlignedProse: 0, // #188
      firstContentY: 110, //   #189, font-sensitive
    });
  });

  test("and #186 leaves sizeGap unset, because a display size is a leap by design", () => {
    // The metric was built for "a cluster and then a leap"; the crowding was the defect and the
    // leap never was. Asserted so that setting it later is a decision rather than a tidy-up.
    expect(THRESHOLDS.sizeGap).toBeUndefined();
  });

  test("and bite on the figures they were written against", () => {
    // `row()` is the pre-redesign district page: 30 boxes with 11 decorative, 13 sizes, an h1 at
    // 1.49x body and a paragraph at 110ch. Every threshold set should have something to say.
    const metrics = violations(report(row())).map((v) => v.metric).sort();
    expect(metrics).toEqual([
      "boxDecorative",
      "headingRatio",
      "measureMax",
      "rightAlignedProse",
      "sizeCount",
    ]);
  });

  test("and pass the page that came out of them", () => {
    const after = row({
      boxes: { count: 19, decorative: 0, maxDepth: 1 },
      sizes: [11.5, 12.5, 14, 15, 17, 21.5, 24, 38.5],
      headingRatio: 2.56,
      measure: { median: 71, p90: 75, max: 75, over78: 0, count: 42 },
      // #188. `alignColumns` decides this per column from what the column holds, so the page that
      // came out of the phase carries none rather than carrying few.
      rightAlignedProse: 0,
    });
    expect(violations(report(after))).toEqual([]);
  });

  test("does not constrain the total, which is mostly affordances", () => {
    // A threshold on `count` could only be met by removing controls' boundaries, which WCAG 1.4.11
    // requires. Asserted so that adding one later is a deliberate act rather than a tidy-up.
    expect(THRESHOLDS.boxCount).toBeUndefined();
    expect(THRESHOLDS.boxDepth).toBeUndefined();
  });
});

describe("once a phase sets one", () => {
  test("#188's hard zero catches the right-aligned prose cells", () => {
    const found = violations(report(row()), { rightAlignedProse: 0 });
    expect(found).toHaveLength(1);
    expect(found[0]?.metric).toBe("rightAlignedProse");
    expect(found[0]?.measured).toBe(31);
    expect(found[0]?.message).toContain("31");
  });

  test("and passes a page that has none", () => {
    expect(violations(report(row({ rightAlignedProse: 0 })), { rightAlignedProse: 0 })).toEqual([]);
  });

  test("#185's box budget catches count and depth separately", () => {
    const found = violations(report(row()), { boxCount: 12, boxDepth: 1 });
    expect(found.map((v) => v.metric).sort()).toEqual(["boxCount", "boxDepth"]);
  });

  test("#186's heading ratio is a floor, not a ceiling", () => {
    // The defect is a heading too SMALL relative to body, so the comparison runs the other way
    // from every other threshold here. Getting this backwards would grade the fix as the failure.
    expect(violations(report(row({ headingRatio: 1.49 })), { headingRatio: 2 })).toHaveLength(1);
    expect(violations(report(row({ headingRatio: 2.4 })), { headingRatio: 2 })).toEqual([]);
  });

  test("a measure breach says which font produced the number", () => {
    // The whole reason `bodyFont` is carried on the row: 110ch under SF Pro and 110ch under DejaVu
    // are different amounts of text, and a message that omits it sends someone chasing a
    // regression that is a platform difference.
    const found = violations(report(row()), { measureMax: 78 });
    expect(found[0]?.message).toContain("ui-sans-serif");
  });

  test("the chrome budget applies only at the narrowest width", () => {
    // 166px of chrome at 390px is the defect. The same header is 51px at 1280 and entirely fine,
    // so a threshold applied at every width would report a failure on the desktop layout.
    const narrow = row({ width: 375, firstContentY: 194 });
    const wide = row({ width: 1280, firstContentY: 194 });
    expect(violations(report(narrow), { firstContentY: 96 })).toHaveLength(1);
    expect(violations(report(wide), { firstContentY: 96 })).toEqual([]);
  });

  test("reports every breach rather than stopping at the first", () => {
    // A redesign phase wants to see all of what it moved. Failing fast turns one run into six.
    const found = violations(report(row(), row({ route: "/statewide.html" })), {
      rightAlignedProse: 0,
      boxCount: 12,
    });
    expect(found).toHaveLength(4);
    expect(new Set(found.map((v) => v.route)).size).toBe(2);
  });

  test("a null measurement is not a breach", () => {
    // `/districts` is a table and carries no substantial paragraph, so `measure` is null there.
    // Treating absent as zero would report the table as the best-set page on the site.
    expect(violations(report(row({ measure: null, headingRatio: null })), {
      measureMax: 78,
      headingRatio: 2,
    })).toEqual([]);
  });
});

describe("the printed table", () => {
  const text = formatReport(report(row({ width: 375 }), row({ width: 1280 })));

  test("groups by width and names the font behind the font-sensitive columns", () => {
    expect(text).toContain("375px");
    expect(text).toContain("1280px");
    expect(text).toContain("ui-sans-serif");
    expect(text).toContain("8.34px");
  });

  test("says outright which columns do not survive a change of platform", () => {
    // The one line in the output that stops a cross-machine diff being read as a regression.
    expect(text).toMatch(/DEPEND ON THE PLATFORM'S FONT/);
  });

  test("prints an em dash where a page has no measurement, not a zero", () => {
    expect(formatReport(report(row({ measure: null })))).toContain("—");
  });
});
