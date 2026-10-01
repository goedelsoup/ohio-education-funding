/**
 * Every address the build claims — for an unfurler, for a crawler, for itself.
 *
 * # Why this reads files instead of driving a browser
 *
 * Nothing a browser does exercises an `og:` tag. The consumers are Slack, Discord, Facebook,
 * LinkedIn, Bluesky and X, none of which is available to a test, and the failure they produce is
 * a blank rectangle rather than an error anyone sees. The one thing that *is* checkable is the
 * artefact: 3,430 documents, each naming a card, against 995 cards that were actually emitted.
 *
 * That pairing is the whole risk. The tags are written once in `Base.astro` and cannot be wrong
 * on one page and right on another; what can be wrong is one route family pointing at a path the
 * generator never produced — a renamed slug, a `getStaticPaths` that stopped covering something —
 * and it is invisible on every page of the site because the image is not on the page.
 *
 * The same holds one step further out. A canonical link, an `og:url` and a `<loc>` in the sitemap
 * are three statements of one address written by three different pieces of code, and nothing makes
 * them agree but the assertions below.
 */

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, test } from "vitest";

import { parseRedirects, resolveRedirect } from "../../src/lib/redirects.ts";
import { DIST, pages } from "./artefact.ts";

describe("what a link to this site looks like when it is pasted somewhere", () => {
  const meta = (body: string, key: string): string | null =>
    body.match(
      new RegExp(`<meta [^>]*(?:property|name)="${key}"[^>]*content="([^"]*)"`, "i"),
    )?.[1] ?? null;



  test("every page carries the tags an unfurler reads", () => {
    const offenders: string[] = [];
    for (const file of pages()) {
      const body = readFileSync(file, "utf8");
      for (const key of ["og:title", "og:description", "og:image", "og:url", "twitter:card"]) {
        if (!meta(body, key)) offenders.push(`${file.slice(DIST.length + 1)}: no ${key}`);
      }
    }
    expect(offenders.slice(0, 10)).toEqual([]);
  });



  test("every card a page names was actually built", () => {
    const offenders: string[] = [];
    const cards = new Set<string>();
    for (const file of pages()) {
      const image = meta(readFileSync(file, "utf8"), "og:image");
      if (!image) continue;
      const path = new URL(image).pathname;
      cards.add(path);
      if (!existsSync(join(DIST, path)))
        offenders.push(`${file.slice(DIST.length + 1)} names ${path}, which was not emitted`);
    }
    expect(offenders.slice(0, 10)).toEqual([]);
    // A sanity floor on the other side: if the endpoints silently stopped producing per-subject
    // cards, every page would fall back to the default and the check above would still pass.
    expect(cards.size).toBeGreaterThan(900);
  });



  test("the canonical link and og:url agree, and neither ends in .html", () => {
    // `build.format` is `"file"`, so `Astro.url.pathname` is the output file. Both of these are
    // that path put back — see `canonicalPath` — and the site shipped for a long time claiming
    // `/district/043786.html` while the sitemap beside it said `/district/043786`.
    const offenders: string[] = [];
    for (const file of pages()) {
      const body = readFileSync(file, "utf8");
      const canonical = body.match(/<link rel="canonical" href="([^"]*)"/i)?.[1] ?? null;
      const url = meta(body, "og:url");
      const name = file.slice(DIST.length + 1);
      if (canonical !== url) offenders.push(`${name}: canonical ${canonical} but og:url ${url}`);
      else if (canonical?.endsWith(".html")) offenders.push(`${name}: ${canonical}`);
    }
    expect(offenders.slice(0, 10)).toEqual([]);
  });



  test("the canonical URLs are the ones the sitemap lists", () => {
    // Two files a crawler reads, which have to name the same set of addresses. They are produced
    // by different code — the layout and `@astrojs/sitemap` — so nothing but this makes them agree.
    const listed = new Set(
      [...readFileSync(join(DIST, "sitemap-0.xml"), "utf8").matchAll(/<loc>([^<]*)<\/loc>/g)].map(
        (m) => m[1]!,
      ),
    );
    const offenders: string[] = [];
    for (const file of pages()) {
      // The 404 is the one page deliberately absent from the sitemap.
      if (file.endsWith("404.html")) continue;
      const canonical =
        readFileSync(file, "utf8").match(/<link rel="canonical" href="([^"]*)"/i)?.[1] ?? "";
      // The root is listed without its trailing slash; `new URL` keeps one. Compare both forms.
      if (!listed.has(canonical) && !listed.has(canonical.replace(/\/$/, "")))
        offenders.push(`${file.slice(DIST.length + 1)}: ${canonical} is not in the sitemap`);
    }
    expect(offenders.slice(0, 10)).toEqual([]);
  });



  test("the icons the layout links are all present", () => {
    for (const icon of ["favicon.svg", "icon-32.png", "apple-touch-icon.png"]) {
      expect(existsSync(join(DIST, icon)), `${icon} was not emitted`).toBe(true);
    }
  });
});

describe("an address typed without its leading zeros (#596)", () => {
  // The built file, which is what deploys: the generated rules exist nowhere else.
  const rules = parseRedirects(readFileSync(join(DIST, "_redirects"), "utf8"));
  const feed = JSON.parse(readFileSync(join(DIST, "data", "bundle.json"), "utf8")) as {
    districts: { irn: string }[];
    house_districts: { number: string }[];
    senate_districts: { number: string }[];
  };
  const keys: [string, string[]][] = [
    ["/district", feed.districts.map((d) => d.irn)],
    ["/house", feed.house_districts.map((h) => h.number)],
    ["/senate", feed.senate_districts.map((h) => h.number)],
  ];

  test("every unpadded IRN and seat number redirects to a page that exists", () => {
    const offenders: string[] = [];
    let checked = 0;
    for (const [root, list] of keys) {
      for (const key of list) {
        const bare = String(Number(key));
        if (bare === key) continue;
        checked++;
        // Unpadded, and padded to two for a three-digit seat: `/senate/1` and `/senate/01`.
        for (const typed of new Set([`${root}/${bare}`, `${root}/${bare.padStart(2, "0")}`])) {
          if (typed === `${root}/${key}`) continue;
          const hit = resolveRedirect(rules, typed, "");
          if (hit?.location !== `${root}/${key}` || hit.status !== 301)
            offenders.push(`${typed} → ${hit?.location ?? "404"}`);
          else if (!existsSync(join(DIST, `${hit.location.slice(1)}.html`)))
            offenders.push(`${typed} → ${hit.location}, which the build did not emit`);
        }
      }
    }
    expect(offenders.slice(0, 10)).toEqual([]);
    // 608 IRNs that start with a zero and all 132 seats. A floor, so a feed that
    // stopped padding would read as a failure here rather than as nothing to check.
    expect(checked).toBeGreaterThan(700);
  });

  test("no page the build emitted is itself redirected", () => {
    for (const [root, list] of keys) {
      for (const key of list) expect(resolveRedirect(rules, `${root}/${key}`, ""), key).toBeNull();
    }
  });
});
