/**
 * The three files a machine reads that no reader ever opens.
 *
 * `robots.txt`, the JSON-LD on `/data`, and the response headers in `_headers`. Each is invisible
 * from every page of the site and from `vite preview`, and each was wrong: there was no
 * `robots.txt` at all, so Cloudflare served an auto-injected one that is entirely comments and
 * names no sitemap; there was no structured data anywhere, on a site whose `/data` route is
 * literally a dataset with three distributions; and the CSV route's `Content-Disposition` was
 * discarded by the static build, so the fiscal year the figures are on reached nobody.
 *
 * They are checked against the artefact rather than through a browser for the reason
 * `addresses.spec.ts` gives about `og:` tags: the consumers are crawlers and unfurlers, none of
 * which is available to a test, and the failure they produce is silence.
 */

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, test } from "vitest";

import { REQUIRED_CONTRACT } from "../../src/lib/types.ts";
import { DIST } from "./artefact.ts";

describe("what a machine reads", () => {
  const feed = JSON.parse(readFileSync(join(DIST, "data", "bundle.json"), "utf8"));



  test("robots.txt names the sitemap, at the host the canonical links use", () => {
    const robots = readFileSync(join(DIST, "robots.txt"), "utf8");
    const sitemap = robots.match(/^Sitemap:\s*(\S+)$/m)?.[1];
    expect(sitemap, "no Sitemap: line — the whole reason to ship this file").toBeTruthy();

    // The host is written out in `robots.txt` and configured in `astro.config.mjs`, and a crawler
    // that follows a sitemap URL on the wrong host gets nothing. Take the site's own canonical
    // link as the authority rather than restating the hostname here for a third time.
    const canonical = readFileSync(join(DIST, "index.html"), "utf8").match(
      /<link rel="canonical" href="([^"]*)"/i,
    )?.[1];
    expect(new URL(sitemap!).origin).toBe(new URL(canonical!).origin);

    // And it has to point at something that was built.
    expect(existsSync(join(DIST, new URL(sitemap!).pathname.slice(1)))).toBe(true);
  });



  test("/data describes itself as a Dataset, and every distribution it names exists", () => {
    const body = readFileSync(join(DIST, "data.html"), "utf8");
    const block = body.match(/<script type="application\/ld\+json">([\s\S]*?)<\/script>/i)?.[1];
    expect(block, "/data carries no structured data").toBeTruthy();

    const data = JSON.parse(block!);
    expect(data["@type"]).toBe("Dataset");
    // Read from the feed, not typed — so a contract bump moves this without anyone editing it,
    // which is the property that makes the block worth emitting rather than a liability.
    expect(data.version).toBe(feed.contract_version);
    expect(data.version).toBe(REQUIRED_CONTRACT);

    const named = data.distribution.map((d: { contentUrl: string }) => new URL(d.contentUrl).pathname);
    expect(named).toEqual(["/data/districts.csv", "/data/bundle.json", "/data/panel.json"]);
    for (const path of named) {
      expect(existsSync(join(DIST, path.slice(1))), `${path} is named but was not built`).toBe(true);
    }

    // The three the page links are the three it declares. A fourth download added to the table
    // and not to the block would leave the structured data quietly describing an older site.
    const linked = [...body.matchAll(/href="(\/data\/[^"]+)"/g)].map((m) => m[1]!);
    expect([...new Set(linked)].sort()).toEqual([...named].sort());
  });



  test("the CSV download's headers survive the static build", () => {
    /*
     * `output: "static"` discards the headers an endpoint's `Response` carries — Astro keeps the
     * body and writes a file. So the `Content-Disposition` in `src/pages/data/districts.csv.ts`
     * reached `astro dev` and `vite preview` and never the deploy, and the comment in that route
     * about provenance travelling in the filename described a local session.
     *
     * The `csv-download-headers` integration in `astro.config.mjs` appends them to `dist/_headers`,
     * reading the fiscal year out of the same feed. This asserts against the built `_headers`
     * because that is the file Cloudflare Pages reads; nothing else in this repository looks at it.
     */
    const headers = readFileSync(join(DIST, "_headers"), "utf8");
    expect(headers).toContain("/data/districts.csv");
    expect(headers).toContain(
      `Content-Disposition: attachment; filename="ohio-school-funding-fy${feed.fiscal_year}.csv"`,
    );
    // The block is appended, and appending to a file that had gone missing would produce a
    // `_headers` holding only this and silently dropping the CSP. The integration throws in that
    // case; this is the assertion that the throw is about something real.
    expect(headers).toContain("Content-Security-Policy:");
  });
});
