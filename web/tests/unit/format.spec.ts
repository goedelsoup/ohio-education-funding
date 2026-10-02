/**
 * `fixed` and `signed`, and the rule that keeps the renderers using them.
 *
 * The output half is `tests/dist/numbers.spec.ts`, which reads every figure cell in the build.
 * This half runs before the build: the formatters' edge cases, and a source rule over the modules
 * #503 moved off `toFixed`, so a new call there fails here rather than in a cell nobody opens.
 */

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, test } from "vitest";

import { firstOf, lastOf } from "../../src/lib/ends.ts";
import * as format from "../../src/lib/format.ts";
import { compactMoney, compactMoneyTicks, fixed, NOT_REPORTED, pct, reported, sig, signed } from "../../src/lib/format.ts";

describe("fixed", () => {
  test("groups thousands and keeps its places", () => {
    expect(fixed(8376.39, 2)).toBe("8,376.39");
    expect(fixed(0.2104, 4)).toBe("0.2104");
    expect(fixed(20, 0)).toBe("20");
    expect(fixed(3.1, 2)).toBe("3.10");
  });

  test("writes a negative with U+2212, as `money` does", () => {
    expect(fixed(-0.03, 2)).toBe("−0.03");
    expect(fixed(-1234.5, 1)).toBe("−1,234.5");
  });

  test("drops the sign of a magnitude that rounds away", () => {
    expect(fixed(-0.001, 2)).toBe("0.00");
    expect(fixed(-0, 1)).toBe("0.0");
  });

  test("has an em dash for no value", () => {
    expect(fixed(null, 2)).toBe("—");
    expect(fixed(undefined, 2)).toBe("—");
    expect(fixed(Number.NaN, 2)).toBe("—");
    expect(fixed(Number.POSITIVE_INFINITY, 2)).toBe("—");
  });
});

describe("signed", () => {
  test("always writes the direction", () => {
    expect(signed(0.662, 3)).toBe("+0.662");
    expect(signed(-0.662, 3)).toBe("−0.662");
    expect(signed(-1500.25, 2)).toBe("−1,500.25");
  });

  test("writes no direction for a zero, after rounding", () => {
    expect(signed(0, 4)).toBe("0.0000");
    expect(signed(0.00004, 4)).toBe("0.0000");
    expect(signed(-0.00004, 4)).toBe("0.0000");
  });

  test("has an em dash for no value", () => {
    expect(signed(null, 1)).toBe("—");
  });
});

describe("pct", () => {
  test("a positive share below half a percent prints <1%, not 0%", () => {
    // #610: a bar drawn with width and labelled "0%" contradicts itself.
    expect(pct(0.004, 0)).toBe("<1%");
    expect(pct(0.0001, 0)).toBe("<1%");
    expect(pct(0.0004, 1)).toBe("<0.1%");
  });

  test("leaves zero, the boundary and negatives alone", () => {
    expect(pct(0, 0)).toBe("0%");
    expect(pct(0.005, 0)).toBe("1%");
    expect(pct(-0.004, 0)).toBe("0%");
    expect(pct(-0.02, 0)).toBe("−2%");
  });
});

describe("compactMoney", () => {
  test("is three significant figures at every scale a chart prints", () => {
    expect(compactMoney(83_470_878)).toBe("$83.5M");
    expect(compactMoney(10_480_000_000)).toBe("$10.5B");
    expect(compactMoney(1_350_078)).toBe("$1.35M");
    expect(compactMoney(200_000)).toBe("$200K");
    expect(compactMoney(-176_555_014)).toBe("−$177M");
  });

  test("keeps whole dollars under ten thousand, where the prose does", () => {
    expect(compactMoney(4229)).toBe("$4,229");
    expect(compactMoney(-47)).toBe("−$47");
  });

  test("has an em dash for no value", () => {
    expect(compactMoney(null)).toBe("—");
  });
});

describe("compactMoneyTicks", () => {
  test("writes a tick set in one style once any tick reaches ten thousand (#662)", () => {
    // `/statewide`'s wealth-offset axis read "$1,000" beside "$20K".
    expect(compactMoneyTicks([1000, 20_000])).toEqual(["$1K", "$20K"]);
    expect(compactMoneyTicks([-5000, 25_000])).toEqual(["−$5K", "$25K"]);
  });

  test("keeps a set that never reaches ten thousand whole", () => {
    expect(compactMoneyTicks([1000, 5000])).toEqual(["$1,000", "$5,000"]);
  });
});

describe("sig", () => {
  test("writes a ratio to three significant figures with a true minus", () => {
    expect(sig(2.0057)).toBe("2.01");
    expect(sig(-3.537)).toBe("−3.54");
    expect(sig(62_603.6)).toBe("62,600");
    expect(sig(0.0272)).toBe("0.0272");
  });
});

describe("firstOf and lastOf", () => {
  test("return the ends", () => {
    expect(firstOf([3, 1, 2])).toBe(3);
    expect(lastOf([3, 1, 2])).toBe(2);
    expect(lastOf(["only"])).toBe("only");
  });

  test("fail on an empty sequence rather than returning undefined", () => {
    expect(() => firstOf([])).toThrow(RangeError);
    expect(() => lastOf([])).toThrow(RangeError);
  });
});

test("the renderers #503 migrated format no number by hand", () => {
  /*
   * Named files rather than the whole tree: `toFixed` is right where the output is not a figure on
   * a page — chart geometry in `plot/spec.ts`, the terminal table in `measure.ts` — and 62 calls remain
   * outside this list and `format.ts`. These are the modules whose `toFixed` had put a hyphen minus
   * or an ungrouped thousand into a published cell, and they are held at zero.
   *
   * Comments are blanked first, so a docstring can still say what the rule forbids.
   */
  const SRC = join(import.meta.dirname, "../../src");
  // `lib/district.ts` became a directory of components in #515. Read rather than listed, so a card
  // split out of one of them is held too; the count is what stops a moved directory from passing
  // on nothing.
  const DISTRICT = readdirSync(join(SRC, "components/district"))
    .filter((name) => name.endsWith(".astro"))
    .map((name) => `components/district/${name}`);
  expect(DISTRICT).toHaveLength(22);
  const MIGRATED = [
    "lib/compare.ts",
    ...DISTRICT,
    "lib/mealProgram.ts",
    "lib/outcomes.ts",
    "lib/statewide.ts",
    "lib/tax.ts",
    "scripts/districts.ts",
  ];
  const offenders: string[] = [];
  for (const file of MIGRATED) {
    const live = readFileSync(join(SRC, file), "utf8")
      .replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, " "))
      .replace(/^\s*\/\/.*$/gm, (m) => " ".repeat(m.length));
    for (const [line, content] of live.split("\n").entries()) {
      if (/\.toFixed\(|\.toLocaleString\(/.test(content)) {
        offenders.push(`${file}:${line + 1}: ${content.trim()}`);
      }
    }
  }
  expect(offenders, "use `fixed`, `signed` or `count` from src/lib/format.ts").toEqual([]);
});

test("a log error on a chart is the percentage it amounts to (#658)", () => {
  // `exp(v) - 1`, not `100 v`: the deepest pre-closure figure is +0.0272, which is +2.8%.
  expect(format.logErrorPercent(0.0272)).toBe("+2.8%");
  expect(format.logErrorPercent(-0.0071)).toBe("−0.7%");
  // A magnitude that rounds away loses its sign.
  expect(format.logErrorPercent(-0.0001)).toBe("0.0%");
  expect(format.logErrorPercent(0)).toBe("0.0%");
});

describe("zero is not missing", () => {
  /*
   * "—" meant $0 under one column of /districts and "no figure" under the next (#597). A zero is a
   * value and is written as one; only the null branch writes a placeholder.
   */
  test("no formatter writes a zero as the dash", () => {
    const zeros = {
      money: format.money(0),
      signedMoney: format.signedMoney(0),
      millions: format.millions(0),
      compactMoney: format.compactMoney(0),
      // A zero tick in a compact set, which is the branch `compactMoney` does not reach.
      compactMoneyTicks: format.compactMoneyTicks([0, 20_000])[0]!,
      sig: format.sig(0),
      pct: format.pct(0),
      logError: format.logError(0),
      logErrorPercent: format.logErrorPercent(0),
      fixed: format.fixed(0, 2),
      signed: format.signed(0, 2),
      count: format.count(0),
      reported: format.reported(0, format.money),
    };
    // Every number formatter `format.ts` exports is in the table, so a new one cannot skip it.
    const NOT_NUMBERS = ["ordinal", "percentileOf", "escapeHtml", "fig", "NOT_REPORTED"];
    expect(Object.keys(format).filter((k) => !NOT_NUMBERS.includes(k)).sort()).toEqual(
      Object.keys(zeros).sort(),
    );
    for (const [name, text] of Object.entries(zeros)) {
      expect(text, name).not.toContain("—");
      expect(text, name).not.toBe(NOT_REPORTED);
    }
    expect(zeros.money).toBe("$0");
    expect(zeros.pct).toBe("0.0%");
  });

  test("a missing value is not reported, in words", () => {
    expect(reported(null, format.money)).toBe("not reported");
    expect(reported(undefined, format.money)).toBe("not reported");
    expect(reported(Number.NaN, (v) => pct(v, 1))).toBe("not reported");
    expect(reported(0.123, (v) => pct(v, 1))).toBe("12.3%");
  });

  test("no renderer writes a dash for a zero it tested for", () => {
    /*
     * `x <= 0 ? "—" : money(x)` was the pattern, in eleven cells across nine files. It sent a zero
     * past `money` into a dash of its own, so this holds the source rather than the formatter.
     */
    const SRC = join(import.meta.dirname, "../../src");
    const files = readdirSync(SRC, { recursive: true, encoding: "utf8" }).filter((name) =>
      /\.(astro|ts)$/.test(name),
    );
    expect(files.length).toBeGreaterThan(100);
    const ZERO_AS_DASH = /(?:<=?|===?)\s*0\s*\?\s*"—"/;
    const offenders = files.filter((name) => ZERO_AS_DASH.test(readFileSync(join(SRC, name), "utf8")));
    expect(offenders, "write the zero through `money`; a missing value through `reported`").toEqual([]);
  });
});
