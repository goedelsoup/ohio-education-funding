/**
 * The lines behind the totals, and the two things the card must not overstate.
 *
 * Half the lines name no establishing act, and a discontinued flag is not a finding about
 * abolition. Both are easy to render past and both are tested here, because the failure in each
 * case is a table that looks complete.
 */

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";
import { byAge, ordinal, renderLineOrigins, span } from "../../src/lib/lineOrigins.ts";

const { bundle } = loadFeed();
const lines = bundle.appropriation_lines;

test("the feed carries the lines at all", () => {
  // The fixture was extracted, tested in Rust, and reachable by no reader for a phase.
  expect(lines.length).toBeGreaterThan(50);
  expect(lines.some((l) => l.general_assembly != null)).toBe(true);
  expect(lines.some((l) => l.general_assembly == null)).toBe(true);
});

test("live lines sort oldest first with the undated ones last", () => {
  /*
   * The undated lines are not "oldest" and must not sort as though a missing General Assembly
   * were a low one. A naive numeric sort on null does exactly that in JavaScript.
   */
  const sorted = byAge(lines);
  expect(sorted.every((l) => !l.discontinued)).toBe(true);

  const firstUndated = sorted.findIndex((l) => l.general_assembly == null);
  if (firstUndated !== -1) {
    expect(sorted.slice(firstUndated).every((l) => l.general_assembly == null)).toBe(true);
  }
  const dated = sorted.filter((l) => l.general_assembly != null);
  for (let i = 1; i < dated.length; i++) {
    expect(dated[i]!.general_assembly!).toBeGreaterThanOrEqual(dated[i - 1]!.general_assembly!);
  }
});

test("the department's lines span at least forty years", () => {
  // The finding the card exists for. If this narrows sharply, either the extraction lost the old
  // lines or the legislature renumbered them, and both are worth stopping for.
  const reach = span(lines)!;
  expect(reach.newest - reach.oldest).toBeGreaterThan(40);
  expect(reach.oldest).toBeLessThan(1997); // before DeRolph I was decided
});

test("ordinals read the way the Catalog writes them", () => {
  expect(ordinal(112)).toBe("112th");
  expect(ordinal(121)).toBe("121st");
  expect(ordinal(122)).toBe("122nd");
  expect(ordinal(133)).toBe("133rd");
  // The teens are the case a naive rule gets wrong.
  expect(ordinal(111)).toBe("111th");
  expect(ordinal(113)).toBe("113th");
});

test("a line with no establishing act says so rather than showing a blank", () => {
  /*
   * An empty cell reads as an oversight; "not stated" reads as the document declining to say,
   * which is what happened. The card also has to explain why the gap is not filled.
   */
  const html = renderLineOrigins(lines);
  expect(html).toContain("not stated");
  expect(html).toContain("name no establishing act");
  expect(html).toContain("numbers are reused");
});

test("the card refuses to let the discontinued flag mean abolition", () => {
  const html = renderLineOrigins(lines);
  expect(html).toContain("not a finding about");
  expect(html).toContain("folded into another is discontinued too");
});

test("an empty feed renders nothing", () => {
  expect(renderLineOrigins([])).toBe("");
  expect(span([])).toBeNull();
});
