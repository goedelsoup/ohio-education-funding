/**
 * Counties as peer groups: the grouping, and the one claim the pages are built on.
 *
 * The county pages assert something specific — that districts the department places in the same
 * county stand on tax bases that differ by multiples — and these tests check that the grouping
 * behind the assertion is complete and that the assertion is true of the feed rather than of a
 * well-chosen example.
 */

import { expect, test } from "vitest";

import { counties, medianRealizedAid, slugify } from "../../src/lib/county.ts";
import { loadFeed } from "../../src/lib/feed.ts";

const { bundle } = loadFeed();
const all = counties(bundle.districts);

test("every district lands in exactly one county and none is lost", () => {
  // The grouping is a partition, and a page built on a partition that drops rows shows a county
  // missing a district without anything looking wrong.
  const grouped = all.flatMap((c) => c.districts.map((d) => d.irn));
  expect(grouped).toHaveLength(bundle.districts.length);
  expect(new Set(grouped).size).toBe(bundle.districts.length);
});

test("Ohio's 88 counties are all present", () => {
  expect(all).toHaveLength(88);
  expect(all.every((c) => c.name.length > 0)).toBe(true);
});

test("slugs are unique and URL-safe, because they are the route", () => {
  const slugs = all.map((c) => c.slug);
  expect(new Set(slugs).size).toBe(slugs.length);
  // `Van Wert` must not become a path segment with a space in it, and no slug may collide with
  // another after case folding — `/county/van-wert` is the file the static host serves.
  for (const slug of slugs) expect(slug).toMatch(/^[a-z0-9]+(-[a-z0-9]+)*$/);
  expect(slugify("Van Wert")).toBe("van-wert");
});

test("a county's pupil total is the sum of the districts it lists and nothing more", () => {
  // The pages are careful to call this a sum of the listed districts rather than the county's
  // children, because school district boundaries cross county lines. The arithmetic must match
  // the claim: this is a plain sum, with no apportionment hiding in it.
  for (const c of all) {
    const expected = c.districts.reduce((a, d) => a + d.adm, 0);
    expect(c.adm).toBeCloseTo(expected, 6);
  }
  const statewide = all.reduce((a, c) => a + c.adm, 0);
  const feed = bundle.districts.reduce((a, d) => a + d.adm, 0);
  expect(statewide).toBeCloseTo(feed, 4);
});

test("the disparity the pages lead with is real in most of the state, not just the examples", () => {
  /*
   * The claim on every county page is that neighbours are funded from unequal tax bases. If that
   * were true of two or three counties it would be an anecdote and the page framing would be
   * dishonest.
   *
   * It is true of nearly all of them: the median county's richest district has more than twice the
   * valuation per pupil of its poorest, and the widest is over seven times.
   */
  const ratios = all
    .map((c) => c.valuationRatio)
    .filter((r): r is number => r != null)
    .sort((a, b) => a - b);

  expect(ratios.length).toBeGreaterThan(80);
  const median = ratios[Math.floor(ratios.length / 2)]!;
  expect(median).toBeGreaterThan(2);
  expect(ratios[ratios.length - 1]).toBeGreaterThan(7);
});

test("a single-district county reports no ratio rather than a ratio of one", () => {
  // Four counties have one district. A ratio of 1.0 would read as "no disparity here", which is a
  // different statement from "there is nothing to compare" and would sort them among the most
  // equal counties on the index.
  const alone = all.filter((c) => c.districts.length === 1);
  expect(alone.length).toBeGreaterThan(0);
  for (const c of alone) {
    expect(c.valuationRatio).toBeNull();
    expect(c.richest).toBeNull();
    expect(c.poorest).toBeNull();
  }
});

test("the richest and poorest are the extremes of the county, not of the reporting subset", () => {
  for (const c of all) {
    if (c.valuationRatio == null) continue;
    const reported = c.districts
      .map((d) => d.valuation_per_pupil)
      .filter((v): v is number => v != null && v > 0);
    expect(c.richest!.valuation_per_pupil).toBe(Math.max(...reported));
    expect(c.poorest!.valuation_per_pupil).toBe(Math.min(...reported));
  }
});

test("the statewide median aid the pages compare against is the feed's own", () => {
  const median = medianRealizedAid(bundle.districts);
  const above = bundle.districts.filter((d) => d.realized_aid_per_pupil > median).length;
  const below = bundle.districts.filter((d) => d.realized_aid_per_pupil < median).length;
  expect(Math.abs(above - below)).toBeLessThanOrEqual(1);
});
