/**
 * The addresses that moved, and the reader that sends them on.
 *
 * `src/lib/redirects.ts` is the host's matcher rewritten for the preview server, so it is only as
 * good as its agreement with the host on the rules this site's file leans on. Those are pinned here
 * against `public/_redirects` itself — the file that deploys — rather than against a fixture that
 * could agree with the matcher and disagree with the site.
 *
 * The browser half, that each old address lands on a page that renders, is
 * `tests/e2e/redirects.spec.ts`.
 */

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, test } from "vitest";

import { parseRedirects, resolveRedirect } from "../../src/lib/redirects.ts";
import { REACH, districtScenario } from "../../src/lib/routes.ts";
import { chosenDistrict } from "../../src/lib/scenario.ts";

const FILE = readFileSync(resolve(import.meta.dirname, "../../public/_redirects"), "utf8");
const rules = parseRedirects(FILE);

describe("public/_redirects", () => {
  test("parses, and every rule is a permanent move", () => {
    expect(rules.length).toBeGreaterThan(0);
    for (const rule of rules) expect(rule.status, `${rule.from}`).toBe(301);
  });

  test("the reach view's old address lands on the runner's reach tab, levers and all", () => {
    expect(resolveRedirect(rules, "/reach", "")).toEqual({ location: REACH, status: 301 });
    expect(resolveRedirect(rules, "/reach", "?base=1.05&co=athens")?.location).toBe(
      `${REACH}?base=1.05&co=athens`,
    );
  });

  test("a district's old scenario address opens the runner on that district, levers and all", () => {
    const hit = resolveRedirect(rules, "/district/043786/scenario", "?g=removed&base=1.1");
    expect(hit?.location).toBe("/scenario?g=removed&base=1.1#d=043786");
    // And what the runner reads out of it is the district and the levers both.
    const url = new URL(hit!.location, "https://host.invalid");
    expect(chosenDistrict(url.searchParams, url.hash)).toBe("043786");
    expect(url.searchParams.get("g")).toBe("removed");
  });

  test("the pages that replaced them are not themselves redirected", () => {
    for (const path of ["/scenario", REACH, "/district/043786", "/district/043786/taxes"]) {
      expect(resolveRedirect(rules, path, ""), path).toBeNull();
    }
  });
});

describe("the matcher keeps the host's query rule", () => {
  test("a destination with its own query drops the incoming one", () => {
    // The rule the file is written around. A matcher that always carried the query would pass the
    // tests above for a file that put `?d=` in the destination, and the host would drop the levers.
    const own = parseRedirects("/old  /new?x=1  301");
    expect(resolveRedirect(own, "/old", "?keep=me")?.location).toBe("/new?x=1");
    const none = parseRedirects("/old  /new  301");
    expect(resolveRedirect(none, "/old", "?keep=me")?.location).toBe("/new?keep=me");
  });

  test("a # inside a rule is a fragment, not a comment", () => {
    const [rule] = parseRedirects("# a comment\n/a/:id  /b#id=:id\n");
    expect(rule).toEqual({ from: "/a/:id", to: "/b#id=:id", status: 302 });
  });

  test("a placeholder is one segment and a splat is the rest", () => {
    const own = parseRedirects("/a/:id/x  /b/:id  301\n/c/*  /d/:splat  301");
    expect(resolveRedirect(own, "/a/1/2/x", "")).toBeNull();
    expect(resolveRedirect(own, "/c/1/2", "")?.location).toBe("/d/1/2");
  });
});

describe("the district the runner opens on", () => {
  const at = (search: string, hash = "") => chosenDistrict(new URLSearchParams(search), hash);

  test("reads the link a district page mints", () => {
    const url = new URL(districtScenario("043786"), "https://host.invalid");
    expect(chosenDistrict(url.searchParams, url.hash)).toBe("043786");
  });

  test("takes the first of the reach view's lit districts", () => {
    expect(at("?d=044933,043786")).toBe("044933");
  });

  test("prefers the fragment a redirect wrote over the query", () => {
    expect(at("?d=044933", "#d=043786")).toBe("043786");
  });

  test("reads anything that is not an IRN as the whole state", () => {
    expect(at("")).toBe("");
    expect(at("?d=cleveland")).toBe("");
    expect(at("?d=0437861")).toBe("");
    expect(at("", "#distribution")).toBe("");
  });
});
