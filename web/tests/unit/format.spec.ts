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
import { fixed, signed } from "../../src/lib/format.ts";

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
