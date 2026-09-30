/**
 * One name per place: what the bar calls a page is what the page calls itself.
 *
 * # What this caught
 *
 * The bar, the tab title and the heading were three names typed in three files. So the bar said
 * `All 609 districts` and opened `Districts`; said `History` and opened `The long view`; said
 * `Wiki` and opened `The corpus`; said `Downloads` over a page titled `Data`; and the statute
 * timeline was `Statute` in the tab and `Ohio school funding in statute` on the page. A reader who clicks
 * a word and lands on a different one has to decide whether they are where they meant to be, and
 * a reader who searches their history for the word they clicked does not find it (#547).
 *
 * The names now live in one table, `NAMES` in `src/lib/nav.ts`, and the pages read them. That
 * makes the rule true today; this makes it stay true. A page that re-types its own `<title>`, or
 * an `h1` edited to something livelier, fails here rather than drifting.
 *
 * # The rule, exactly
 *
 * For every link in the built bar:
 *
 * - the `<title>`'s stem — everything before the first ` — ` — **equals** the link's label, and
 * - the `h1` **begins with** it. An `h1` may add a clause; it may not change the name.
 *
 * A class link carries its count, `Legislation (16)`; the count is the bar's and is stripped.
 *
 * # Why it reads `dist/` and not the table
 *
 * Because the table is what the pages are supposed to read, and "supposed to" is the thing under
 * test. Comparing `NAMES` with itself proves nothing; comparing the bar a reader sees with the
 * document the link opens is the claim.
 */

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { parseHTML } from "linkedom";
import { describe, expect, test } from "vitest";

import { DIST, pages } from "./artefact.ts";

const read = (file: string) => parseHTML(readFileSync(file, "utf8")).document;

/** `/statewide` → `dist/statewide.html`, under `build.format: "file"`. */
function fileFor(href: string): string {
  const path = new URL(href, "https://example.invalid").pathname.replace(/\/$/, "");
  return join(DIST, `${path === "" ? "index" : path}.html`);
}

/** A class index, `/wiki/legislation` — the one kind of bar link that carries a count. */
const CLASS_INDEX = /^\/wiki\/[^/]+$/;

/**
 * The bar's label without the count a class link carries.
 *
 * Only a class link's. The bar named acts too until #548, and an act's label ends in its year —
 * `Am. Sub. H.B. 96 (2025)` — which is the same shape and is part of the name.
 */
const nameOf = (label: string, href: string) =>
  CLASS_INDEX.test(href) ? label.replace(/\s+\(\d+\)$/, "") : label;

/** Collapse the whitespace the template's line breaks leave inside an element. */
const text = (node: { textContent: string | null } | null) =>
  (node?.textContent ?? "").replace(/\s+/g, " ").trim();

interface Entry {
  href: string;
  label: string;
}

/**
 * Every link in the bar, read off the built homepage.
 *
 * Any page would do — the bar is the same on all of them — and the homepage is the one a gate
 * reading `dist/` is least likely to lose to a route rename.
 */
function bar(): Entry[] {
  const doc = read(join(DIST, "index.html"));
  const links = [...doc.querySelectorAll('header.site nav[aria-label="Sections"] a[href]')];
  return links.map((a) => {
    const href = a.getAttribute("href") ?? "";
    return { href, label: nameOf(text(a.querySelector(".menu-label") ?? a), href) };
  });
}

/*
 * No exemptions. The `Law` panel named seven acts by designation, each titled on its own page by
 * designation *and* subject, and they were excused here by pattern until #548 took them out of the
 * bar in favour of the class indexes. The exemption went with them: every link the bar carries now
 * is a place, and every place goes by one name.
 */
describe("one name per place", () => {
  const entries = bar();

  test("the bar was read", () => {
    // Guard against the selector going stale: every assertion below is vacuous over an empty bar.
    expect(entries.length).toBeGreaterThan(20);
  });

  test.each(entries.map((e) => [e.label, e]))(
    "%s: the bar, the title and the heading agree",
    (_label, entry) => {
      const file = fileFor(entry.href);
      expect(existsSync(file), `${entry.href} has no page at ${file}`).toBe(true);
      const doc = read(file);

      const title = text(doc.querySelector("title"));
      const stem = title.split(" — ")[0];
      expect(stem, `${entry.href}: <title> is "${title}"`).toBe(entry.label);

      const h1 = text(doc.querySelector("main h1") ?? doc.querySelector("h1"));
      expect(
        h1.startsWith(entry.label),
        `${entry.href}: the bar says "${entry.label}" and the h1 says "${h1}"`,
      ).toBe(true);
    },
  );
});

/*
 * The same rule one level down, for a district's tabs (#559). `DISTRICT_VIEWS` in `src/lib/nav.ts`
 * is the table; this reads the strip a reader sees on a built dashboard and opens every tab it
 * links. A tab's `<title>` is the district's `h1`, then the tab's label — except the dashboard's,
 * which is the district alone. The runner's tab leaves the district and is a place in the bar,
 * held by the rule above.
 *
 * Two districts: Cleveland, and one whose name another district shares, so its `h1` carries the
 * county and the title has to carry it too.
 */
describe("one name per district tab", () => {
  const SITE = " — Ohio school funding";
  const districts = ["043786", "000442"];

  test("the second was chosen for its shared name", () => {
    const h1 = text(read(fileFor("/district/000442")).querySelector("main h1"));
    expect(h1).toMatch(/, .+ County$/);
  });

  test.each(districts)("%s: each tab's label is its page's title after the district", (irn) => {
    const home = `/district/${irn}`;
    const dashboard = read(fileFor(home));
    const district = text(dashboard.querySelector("main h1"));

    const tabs = [...dashboard.querySelectorAll(".subnav a[href]")]
      .map((a) => ({ href: a.getAttribute("href") ?? "", label: text(a) }))
      .filter((tab) => tab.href === home || tab.href.startsWith(`${home}/`));
    // Guard against the selector going stale, as above: the dashboard and its three siblings.
    expect(tabs.map((t) => t.href)).toEqual([home, `${home}/outcome`, `${home}/finances`, `${home}/taxes`]);

    for (const tab of tabs) {
      const doc = read(fileFor(tab.href));
      expect(text(doc.querySelector("main h1")), tab.href).toBe(district);
      const title = text(doc.querySelector("title"));
      const expected = tab.href === home ? district : `${district} — ${tab.label}`;
      expect(title, `${tab.href}: the tab says "${tab.label}"`).toBe(`${expected}${SITE}`);
    }
  });
});

/*
 * No title says the site's name twice (#571).
 *
 * The home page passed the site name as its title and the layout appended the site name again, so
 * every browser tab and every shared link to the front door read "Ohio school funding — Ohio school
 * funding". The layout now skips the suffix when the stem is the suffix; this holds it, across
 * `<title>`, `og:title` and `twitter:title`, on every page.
 */
describe("no title repeats the site name", () => {
  const SITE = "Ohio school funding";

  test("the home page is titled once", () => {
    const doc = read(join(DIST, "index.html"));
    expect(text(doc.querySelector("title"))).toBe(SITE);
  });

  test("no page's title stem equals its suffix", () => {
    const doubled = pages().flatMap((file) => {
      const doc = read(file);
      const titles = [
        text(doc.querySelector("title")),
        doc.querySelector('meta[property="og:title"]')?.getAttribute("content") ?? "",
        doc.querySelector('meta[name="twitter:title"]')?.getAttribute("content") ?? "",
      ];
      return titles
        .filter((title) => {
          const parts = title.split(" — ");
          return parts.length > 1 && parts.at(-2) === parts.at(-1);
        })
        .map((title) => `${file.slice(DIST.length)}: "${title}"`);
    });
    expect(doubled).toEqual([]);
  });
});
