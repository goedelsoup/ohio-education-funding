/**
 * The preview cards, and the address a page claims for itself.
 *
 * # What this suite can check, and what it deliberately leaves to the artefact scan
 *
 * A card is an image. Nothing here asserts that it *looks* right — that is a job for opening one,
 * and `tests/e2e/app.spec.ts` covers the half that is mechanically checkable: that every page names
 * a card and that every card it names was actually emitted.
 *
 * What is checkable here is the arithmetic and the strings, which is where the defects have
 * actually been. The percentage on the district bar shipped at a hundred times its value — "426%
 * from the guarantee" on a district whose guarantee is 4% of its aid — because `pct` takes a
 * fraction and was handed a percentage. It rendered, it was plausible at a glance on a card nobody
 * reads twice, and it was wrong on 609 images. That is the shape of bug this file is for.
 */

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { expect, test, describe } from "vitest";

import { loadCorpus } from "../../src/lib/corpus.ts";
import { counties } from "../../src/lib/county.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import { clamp, render, WIDTH, HEIGHT } from "../../src/lib/og/card.ts";
import { CARD, SOURCE_PROPERTY } from "../../src/lib/og/palette.ts";
import { pageCards } from "../../src/lib/og/pages.ts";
import { canonicalPath, og } from "../../src/lib/routes.ts";
import { districtCard } from "../../src/pages/og/district/[irn].png.ts";
import { countyCard } from "../../src/pages/og/county/[slug].png.ts";
import { seatCard } from "../../src/pages/og/[chamber]/[number].png.ts";
import { nodeCard, sourceCard } from "../../src/pages/og/wiki/[class]/[node].png.ts";
import { classCard } from "../../src/pages/og/wiki/[class].png.ts";

const { bundle } = loadFeed();
const corpus = loadCorpus();

describe("the address a page claims for itself", () => {
  /*
   * `build.format` is `"file"`, so during the build `Astro.url.pathname` is the output file and
   * not the served path. Both the canonical link and `og:url` are that path put back, and the
   * shipped site got it wrong in one direction for as long as it had a canonical link: it said
   * `/district/043786.html` while the sitemap beside it said `/district/043786`.
   */
  test("an output file becomes the path it is served at", () => {
    expect(canonicalPath("/index.html")).toBe("/");
    expect(canonicalPath("/district/043786.html")).toBe("/district/043786");
    expect(canonicalPath("/district/043786/finances.html")).toBe("/district/043786/finances");
    expect(canonicalPath("/wiki/metric/performance-index.html")).toBe(
      "/wiki/metric/performance-index",
    );
  });

  test("a path that is already served passes through", () => {
    expect(canonicalPath("/")).toBe("/");
    expect(canonicalPath("/method")).toBe("/method");
  });

  test("no trailing slash survives, because the static host 404s on one", () => {
    expect(canonicalPath("/counties/")).toBe("/counties");
    expect(canonicalPath("/")).toBe("/");
  });
});

describe("the card palette", () => {
  /**
   * The light-mode `:root` block of the token sheet, as a map of custom property to value.
   *
   * Read rather than imported: this is a stylesheet, and the point of this test is that the two
   * files are *separately* maintained and must agree anyway.
   *
   * It read `app.css` until the tokens moved into `tokens/colors.css`, and it failed the moment
   * they did — which is the check working. A drift test that silently stops finding its source
   * stops being a drift test, so this resolves the file it actually needs and would fail loudly
   * again if that one moved too.
   */
  const root = (): Map<string, string> => {
    const path = resolve(process.cwd(), "src/styles/tokens/colors.css");
    const css = readFileSync(path, "utf8");
    // The first `:root` block only. The three after it are the dark overrides and the two explicit
    // theme choices; a card is a PNG baked at build time and is always the light palette.
    const start = css.indexOf(":root {");
    expect(start, `no :root block in ${path} — the palette this test compares against is gone`)
      .toBeGreaterThan(-1);
    const block = css.slice(start, css.indexOf("}", start));
    return new Map(
      [...block.matchAll(/(--[a-z0-9-]+):\s*(#[0-9a-f]{3,8})\s*;/gi)].map((m) => [
        m[1]!,
        m[2]!.toLowerCase(),
      ]),
    );
  };

  test("has not drifted from the stylesheet it was copied from", () => {
    const declared = root();
    for (const [name, value] of Object.entries(CARD)) {
      const property = SOURCE_PROPERTY[name as keyof typeof CARD];
      expect(
        declared.get(property),
        `CARD.${name} is ${value}, but ${property} in app.css is ${declared.get(property)}. A ` +
          `card is a PNG and cannot re-render when the theme changes, so the copy here is the ` +
          `only place it gets the colour — update it, or the cards keep the old palette.`,
      ).toBe(value.toLowerCase());
    }
  });
});

describe("clamping", () => {
  test("leaves anything that fits alone", () => {
    expect(clamp("Lima City", 40)).toBe("Lima City");
  });

  test("cuts on a word boundary when there is one worth cutting on", () => {
    expect(clamp("Cuyahoga Heights Local School District", 24)).toBe("Cuyahoga Heights Local…");
  });

  test("cuts mid-word rather than losing most of the string", () => {
    // No space in the last 40% of the cut, so a word boundary would throw away too much.
    expect(clamp("Antidisestablishmentarianism", 12)).toBe("Antidisesta…");
  });
});

describe("the district card", () => {
  const cards = bundle.districts.map((d) => districtCard(d, bundle.fiscal_year));

  test("states the guarantee as a share of aid, not a multiple of it", () => {
    for (const card of cards) {
      expect(card.bar).toBeDefined();
      expect(card.bar!.share).toBeGreaterThanOrEqual(0);
      expect(card.bar!.share).toBeLessThanOrEqual(1);
      const stated = card.bar!.label.match(/^(\d+)%/);
      if (stated) expect(Number(stated[1])).toBeLessThanOrEqual(100);
    }
  });

  test("a district on the guarantee says how much of its aid that is", () => {
    const cleveland = bundle.districts.find((d) => d.irn === "043786")!;
    const card = districtCard(cleveland, bundle.fiscal_year);
    const share =
      (cleveland.realized_aid_per_pupil - cleveland.formula_aid_per_pupil) /
      cleveland.realized_aid_per_pupil;
    expect(card.bar!.tone).toBe("guarantee");
    expect(card.bar!.share).toBeCloseTo(share, 6);
  });

  test("a district on the formula says so, and fills the bar", () => {
    const onFormula = bundle.districts.find((d) => !d.on_guarantee)!;
    const card = districtCard(onFormula, bundle.fiscal_year);
    expect(card.bar).toEqual({ share: 1, label: "Funded by the formula", tone: "formula" });
    expect(card.figureNote).toContain("all of it computed by the formula");
  });

  /*
   * A district whose formula amount exceeds what it is paid would give a negative share, which
   * would render as a bar drawn leftwards from nothing. `render` clamps it; this checks the input
   * side, so the clamp is a backstop rather than the thing keeping the card honest.
   */
  test("never claims a negative share", () => {
    for (const card of cards) expect(card.bar!.share).toBeGreaterThanOrEqual(0);
  });
});

describe("every card the site builds", () => {
  /** The cards whose figures come out of the funding panel. */
  const fromFeed = () => [
    ...Object.values(pageCards()),
    ...bundle.districts.map((d) => districtCard(d, bundle.fiscal_year)),
    ...counties(bundle.districts).map((c) => countyCard(c, bundle.fiscal_year)),
    ...[...bundle.house_districts, ...bundle.senate_districts].map((s) =>
      seatCard(s, "house", bundle.fiscal_year),
    ),
  ];

  /** And the ones whose figures count corpus objects, which no fiscal year applies to. */
  const fromCorpus = () => [
    ...corpus.classes.map((c) => classCard(c)),
    ...corpus.nodes.map((n) => nodeCard(n, n.className)),
    ...corpus.sources.map((s) => sourceCard(s)),
  ];

  /** One of each kind, including the extremes that decide whether the layout holds. */
  const every = () => [...fromFeed(), ...fromCorpus()];

  test("lays out without throwing, at every extreme in the feed and the corpus", () => {
    for (const card of every()) expect(() => render(card)).not.toThrow();
  });

  test("carries a headline that has been cut to something a card can hold", () => {
    for (const card of every()) {
      // `clamp` is applied by `render`, so this is about the input: a headline arriving longer
      // than the third size step still fits, but only just, and one arriving empty is a defect.
      expect(card.headline.length).toBeGreaterThan(0);
      expect(clamp(card.headline, 58).length).toBeLessThanOrEqual(58);
    }
  });

  /*
   * A card outlives the page it was shared from — that is the whole point of one — so a number on
   * a card is a number that will be read a year from now with nothing around it. Every card
   * carrying one has to say which year it is from.
   *
   * The figure slot does not always hold a number. "The corpus", "Apportioned, not published" and
   * "JSON and CSV" are phrases, and a phrase does not date. So the rule is about digits, not about
   * whether the slot is filled.
   *
   * It applies to the feed's cards and not the corpus's. "12 nodes" is a count of an ontology
   * class, and an ontology class is not a thing a fiscal year happened to.
   */
  test("dates every figure that is actually a number", () => {
    for (const card of fromFeed()) {
      if (!card.figure || !/\d/.test(card.figure)) continue;
      expect(
        `${card.meta ?? ""} ${card.figureNote ?? ""}`,
        `"${card.figure}" on the ${card.headline} card is a number with no year beside it`,
      ).toMatch(/FY\d{4}|contract \d/i);
    }
  });
});

describe("the card routes", () => {
  test("mirror the page routes they belong to", () => {
    expect(og.district("043786")).toBe("/og/district/043786.png");
    expect(og.county("cuyahoga")).toBe("/og/county/cuyahoga.png");
    expect(og.chamberDistrict("house", "024")).toBe("/og/house/024.png");
    expect(og.wikiClass("metric")).toBe("/og/wiki/metric.png");
    expect(og.wikiNode("metric", "performance-index")).toBe(
      "/og/wiki/metric/performance-index.png",
    );
    expect(og.wikiSource("ocg-white-paper-013")).toBe("/og/wiki/source/ocg-white-paper-013.png");
  });

  test("every slug a page asks for is one the table can render", () => {
    // `og.page()` builds a path from a key; the endpoint builds its paths from the same table. A
    // slug referenced by a page but absent here is a card that never gets emitted, which is a
    // preview that comes back blank on a page nobody thought to re-check.
    const table = pageCards();
    for (const slug of [
      "statewide",
      "districts",
      "counties",
      "compare",
      "outcomes",
      "history",
      "scenario",
      "method",
      "data",
      "wiki",
      "sources",
      "house",
      "senate",
      "site",
    ]) {
      expect(table[slug], `no card in pageCards() for the slug "${slug}"`).toBeDefined();
    }
  });
});

test("the card is the size every consumer of one has settled on", () => {
  // 1.91:1 for Facebook and LinkedIn, over X's 300px floor, under everyone's file-size ceiling.
  expect(WIDTH).toBe(1200);
  expect(HEIGHT).toBe(630);
});
