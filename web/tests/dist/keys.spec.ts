/**
 * What a chart's key and its tooltips say, held across the whole build.
 *
 * # Why this reads the artefact
 *
 * A key is a swatch beside a sentence, and the swatch's colour comes from `data-series` in a
 * template while the line it describes is coloured in `spec.ts`. Nothing joined the two: the
 * district fan keyed orange as "What the formula computes" a few hundred pixels below a bar that
 * keyed it blue (#607), and both templates were correct on their own terms. The pairing is a fact
 * about the pages together, so it is checked over the pages together.
 */

import { readFileSync } from "node:fs";

import { describe, expect, test } from "vitest";

import { DIST, pages } from "./artefact.ts";

/** A swatch and the text after it, up to the next tag. */
const SWATCH = /<i\b([^>]*)><\/i>\s*([^<]*)/g;
const SERIES_OF = /\bdata-series="([^"]+)"/;
const SW = /\bclass="[^"]*\bsw\b/;

/** Each fan's hit layer, which holds one `data-hover` per year. */
const FAN_HIT = /<g\b[^>]*\bclass="fan-hit"[^>]*>([\s\S]*?)<\/g>/g;
const HOVER = /\bdata-hover="([^"]*)"/g;

const routeOf = (file: string) =>
  `/${file.slice(DIST.length + 1).replace(/(^|\/)index\.html$/, "").replace(/\.html$/, "")}`;

describe("chart keys and readings", () => {
  const labels = new Map<string, Map<string, string>>();
  const fans: { route: string; hovers: string[] }[] = [];
  for (const file of pages()) {
    const page = readFileSync(file, "utf8");
    const route = routeOf(file);
    for (const [, attributes = "", text = ""] of page.matchAll(SWATCH)) {
      const series = SERIES_OF.exec(attributes)?.[1];
      const label = text.trim();
      if (!series || !SW.test(attributes) || !label) continue;
      const seen = labels.get(label) ?? new Map<string, string>();
      if (!seen.has(series)) seen.set(series, route);
      labels.set(label, seen);
    }
    for (const [, layer = ""] of page.matchAll(FAN_HIT)) {
      fans.push({ route, hovers: [...layer.matchAll(HOVER)].map((m) => m[1] ?? "") });
    }
  }

  test("the sweep reads keys and fans at all", () => {
    // Without these the rules below pass on a build whose markup stopped matching.
    expect(labels.size).toBeGreaterThan(5);
    expect(labels.get("What the formula computes")?.size, "the formula's key is found").toBe(1);
    // One district fan per district page, three drawings each.
    expect(fans.length).toBeGreaterThan(1_000);
  });

  test("a key's sentence names one series across the whole build", () => {
    const split = [...labels]
      .filter(([, series]) => series.size > 1)
      .map(([label, series]) => `${label}: ${[...series].map(([s, r]) => `${s} on ${r}`).join(", ")}`);
    expect(split).toEqual([]);
    expect([...(labels.get("What the formula computes")?.keys() ?? [])]).toEqual(["formula"]);
  });

  test("no fan reading prints one amount as a range", () => {
    const same = fans.flatMap(({ route, hovers }) =>
      hovers
        .filter((h) => /(\$[\d,.]+[BMK]?) – \1(?![\d,.])/.test(h))
        .map((h) => `${route}: ${h}`),
    );
    expect(same.slice(0, 5)).toEqual([]);
  });

  test("a fan that draws the formula's line names both lines in every reading", () => {
    // Columbus, which the guarantee pays: its fan is the received line and the formula's beside it.
    const columbus = fans.filter((f) => f.route === "/district/043802");
    expect(columbus.length).toBeGreaterThan(0);
    for (const { hovers } of columbus) {
      expect(hovers.length).toBeGreaterThan(1);
      for (const hover of hovers) expect(hover).toMatch(/: aid received \$[\d,]+.*; formula aid \$[\d,]+$/);
    }
  });
});
