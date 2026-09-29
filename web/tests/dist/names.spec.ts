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

import { DIST } from "./artefact.ts";

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
 * Only a class link's: an act's label ends in its year, `Am. Sub. H.B. 96 (2025)`, which is the
 * same shape and is part of the name.
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

/**
 * Links that name a single act rather than a place, which this phase leaves alone.
 *
 * The `Law` panel names seven acts by designation — `Am. Sub. H.B. 96 (2025)` — and each act's
 * page is titled by designation *and* subject. They are not places: they are the only entries the
 * bar computes out of the corpus rather than names, and #548 takes them out of the bar in favour
 * of the class indexes. Until then they are exempt, by pattern, and the exemption goes with them.
 */
const ACT = /^\/wiki\/legislation\/[^/]+$/;

describe("one name per place", () => {
  const entries = bar().filter((entry) => !ACT.test(entry.href));

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
