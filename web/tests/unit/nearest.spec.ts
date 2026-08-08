/**
 * The 404's nearest-match suggestions.
 *
 * Unit-tested rather than exercised in a browser for a specific reason: a static preview server
 * answers an unmatched path with the front page rather than with `404.html`, so the one place this
 * behaviour can be checked is here. That is also why the logic is a pure function taking a path
 * instead of a script reading `location`.
 */

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";
import { nearest, type Candidate } from "../../src/lib/nearest.ts";
import * as routes from "../../src/lib/routes.ts";

const districts: Candidate[] = loadFeed().alphabetical.map((d) => ({
  t: d.name,
  u: routes.district(d.irn),
  a: d.irn,
}));

test("one wrong digit puts the intended district first", () => {
  // Cleveland Municipal is 043786. A reader who typed 043785 or 043787 meant this.
  for (const typo of ["043785", "043787"]) {
    const [first] = nearest(`/district/${typo}`, districts);
    expect(first?.a, `nearest to ${typo}`).toBe("043786");
  }
});

test("an exact IRN in the path returns that district first", () => {
  const [first] = nearest("/district/049056", districts);
  expect(first?.t).toBe("Northern Local");
});

test("a transposed digit still lands in the neighbourhood", () => {
  const suggestions = nearest("/district/043768", districts);
  expect(suggestions.length).toBeGreaterThan(0);
  // Not necessarily first — a transposition moves further than a substitution — but the correct
  // district should be among the handful offered.
  expect(suggestions.map((s) => s.a)).toContain("043786");
});

test("a name in the path is matched as a substring", () => {
  const suggestions = nearest("/district/upper-arlington", districts);
  expect(suggestions.map((s) => s.t)).toContain("Upper Arlington City");
});

test("a path with nothing to go on suggests nothing rather than guessing", () => {
  // An empty 404 beats a confident wrong answer: the suggestions are links a reader will trust.
  expect(nearest("/", districts)).toEqual([]);
  expect(nearest("/xy", districts)).toEqual([]);
});

test("the suggestion list is bounded", () => {
  expect(nearest("/district/000000", districts, 6)).toHaveLength(6);
  expect(nearest("/district/000000", districts, 3)).toHaveLength(3);
});
