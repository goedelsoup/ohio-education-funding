/**
 * Every chart card opens on what it found, in a number (#653).
 *
 * A card's `<h2>` is a topic — "Revenue mix over time" — and it has to stay one: it is the
 * contents entry and the anchor text, and nav, contents and the OG cards all read it. So the
 * finding goes beneath it, as the bold lead-in of the card's first note, in the form
 * `CarriedForward.astro` set: "By FY2027 this district is paid $X less than its formula
 * computes". A reader who stops at that sentence has the card.
 *
 * The rule read off the bytes: a top-level `.card` that draws an `svg.plot` has a first
 * `p.note`, that note has a `<strong>`, and the strong holds a digit. A digit is the cheapest
 * test that the sentence is a finding and not a caption — "One dot per district" passes nothing.
 *
 * # Every page, not one per family
 *
 * The district, county and seat findings are computed from each page's own numbers, and several
 * fall back to nothing when a value is missing — a district with no percentile, a seat with one
 * member valued. A family's representative carries every value; the page that drops its finding
 * is somewhere in the other 2,400. So this reads all of them, and only parses those that draw.
 */

import { readFileSync } from "node:fs";
import { relative, sep } from "node:path";

import { parseHTML } from "linkedom";
import { describe, expect, test } from "vitest";

import { DIST, pages } from "./artefact.ts";

/** What marks a document as drawing a chart — the same test `tests/e2e/charted.ts` uses. */
const PLOT = /<svg\b[^>]*\bclass="plot"/;

/**
 * Cards whose chart is exempt, each with why. Keyed by route pattern and card selector.
 *
 * The wiki's "What this repository computed" card: its finding is the corpus prose beneath the
 * first note, every figure in which is bound to a crate test and checked by the binding gate. A
 * sentence generated over it would restate bound figures with no binding of its own; the first
 * note there is the card's provenance ("Not what Ohio publishes"), and is meant to be.
 */
const EXEMPT: readonly { route: RegExp; card: string; why: string }[] = [
  {
    route: /^\/wiki\//,
    card: "#findings",
    why: "the finding is the bound corpus prose; the first note states its provenance",
  },
];

const routeOf = (file: string): string =>
  "/" +
  relative(DIST, file)
    .split(sep)
    .join("/")
    .replace(/(^|\/)index\.html$/, "")
    .replace(/\.html$/, "");

describe("a chart card's finding", () => {
  test("every top-level card with a chart leads its first note with a bold sentence holding a number", () => {
    const missing: string[] = [];
    let cards = 0;
    for (const file of pages(DIST)) {
      const html = readFileSync(file, "utf8");
      if (!PLOT.test(html)) continue;
      const route = routeOf(file);
      const { document } = parseHTML(html);
      for (const card of document.querySelectorAll(".card")) {
        if (!card.querySelector("svg.plot")) continue;
        // A card nested in a card is a section of its parent, which carries the finding.
        if (card.parentElement?.closest(".card")) continue;
        if (EXEMPT.some((e) => e.route.test(route) && card.matches(e.card))) continue;
        cards += 1;
        const strong = card.querySelector("p.note")?.querySelector("strong")?.textContent ?? "";
        if (!/\d/.test(strong)) {
          missing.push(`${route}#${card.id}: ${strong.trim() || "(no bold lead-in)"}`);
        }
      }
    }
    expect(missing, missing.slice(0, 40).join("\n")).toEqual([]);
    // A sweep that matched nothing would pass on nothing.
    expect(cards).toBeGreaterThan(2400);
  });

  test("an exemption names a card that exists and draws", () => {
    for (const exempt of EXEMPT) {
      const drawn = pages(DIST).some((file) => {
        if (!exempt.route.test(routeOf(file))) return false;
        const html = readFileSync(file, "utf8");
        if (!PLOT.test(html)) return false;
        return parseHTML(html).document.querySelector(exempt.card)?.querySelector("svg.plot") != null;
      });
      expect(drawn, `${exempt.card} on ${exempt.route}: ${exempt.why}`).toBe(true);
    }
  });
});
