/**
 * A build is the same build wherever it runs, and the ordering is the part that was not.
 *
 * # Why this is a test and not a convention
 *
 * `localeCompare` with no locale reads its collation out of the environment — `LANG`, `LC_ALL` —
 * and there were sixteen of them across `web/src/`, including the sort behind the district index
 * and the CSV `/data` publishes. Nothing anywhere said so, and nothing could: the code is correct
 * on the machine of whoever writes it, every time, and the disagreement only exists between two
 * machines that never compare notes.
 *
 * Measured against the real data before it was fixed: nine of thirty locales order this site's
 * strings differently, and a `cs_CZ.UTF-8` build produced ten different files from an
 * `en_US.UTF-8` one — `data/districts.csv` and `districts.html` among them.
 *
 * # Why a source rule and not two builds
 *
 * The issue proposed comparing two builds under different `LANG`, which is the more general check:
 * it would catch ordering that came from a clock or a directory listing too. It also costs two
 * full builds, and CI already builds this site twice. This holds the specific cause instead, at
 * unit-test cost, and it holds it at the moment the seventeenth site is written rather than at the
 * moment someone with a Czech laptop happens to run a build.
 *
 * The second test below is what keeps the first from being a rule about nothing.
 */

import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";
import { compare } from "../../src/lib/order.ts";

const SRC = join(import.meta.dirname, "../../src");

/**
 * Blank out comments, keeping offsets so a line number still means something.
 *
 * The same helper `yearLiterals.spec.ts` needs and for the same reason. `localeCompare` in a
 * docstring is *documentation of this rule* — `order.ts` names it a dozen times explaining why it
 * is forbidden, and `districts.ts` and `legislation.ts` each say which of the sixteen sites they
 * were. A rule that flagged those would be a rule against writing it down, which is the opposite
 * of what makes it survive.
 */
function withoutComments(source: string): string {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, " "))
    .replace(/^\s*\/\/.*$/gm, (m) => " ".repeat(m.length))
    .replace(/\{\/\*[\s\S]*?\*\/\}/g, (m) => m.replace(/[^\n]/g, " "));
}

function sources(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return sources(path);
    return /\.(ts|astro)$/.test(entry.name) ? [path] : [];
  });
}

test("nothing in the tree reads its collation out of the environment", () => {
  /*
   * A hard zero rather than an allowlist. There is no site where the *build machine's* language is
   * the right answer for how to order Ohio school districts, so unlike the year-literal rule there
   * is no legitimate case to admit — and `order.ts` gives the alternative a name, so the fix at a
   * new site is one import.
   *
   * Comments are not searched — see `withoutComments` for why that is the rule and not a
   * concession.
   */
  const offenders: string[] = [];
  for (const file of sources(SRC)) {
    const live = withoutComments(readFileSync(file, "utf8"));
    for (const [line, content] of live.split("\n").entries()) {
      if (!content.includes("localeCompare")) continue;
      offenders.push(`${file.slice(SRC.length + 1)}:${line + 1}: ${content.trim()}`);
    }
  }
  expect(offenders, "use `compare` from src/lib/order.ts").toEqual([]);
});

test("the collation that was pinned is one the environment could have changed", () => {
  /*
   * The rule above is only worth having if the thing it forbids would really have differed, so
   * this asserts the disagreement against the data the site actually ships.
   *
   * Czech and Slovak treat `ch` as one letter, sorting it after `h`. Six Ohio districts begin with
   * it, and under a Czech collation they leave their place among the C's and land after Czechia's
   * `h` — about a hundred and forty rows down the published CSV. That is not a hypothetical: it is
   * what a `cs_CZ.UTF-8` build of this repository produced.
   */
  const names = loadFeed().bundle.districts.map((d) => d.name);

  const ours = [...names].sort(compare);
  const english = [...names].sort((a, b) => a.localeCompare(b, "en"));
  expect(ours, "the site orders text as English does").toEqual(english);

  const czech = [...names].sort((a, b) => a.localeCompare(b, "cs"));
  expect(czech, "a Czech collation really does disagree").not.toEqual(ours);

  // And specifically: `Chagrin Falls` sits among the C's for us and after `Cuyahoga` for Czech.
  const chagrin = (order: string[]) => order.findIndex((n) => n.startsWith("Chagrin Falls"));
  expect(chagrin(ours)).toBeLessThan(chagrin(czech));
});

test("the comparator does not quietly re-sort numbers", () => {
  /*
   * `numeric: true` is the option a reader of `order.ts` would most plausibly reach for next, and
   * adding it would be a change rather than a fix: it puts `9` before `10` where the default puts
   * `10` first. The point of that module is to remove a difference between machines, not to
   * introduce one between builds, so the default is the behaviour and this says so out loud.
   */
  expect([..."10 9 2".split(" ")].sort(compare)).toEqual(["10", "2", "9"]);
});
