/**
 * The statute timeline, and the claims its arithmetic makes about Ohio.
 *
 * # What is actually being asserted
 *
 * `/legislation` draws five formulas end to end across fifty fiscal years. Drawn that way, the
 * page is *claiming* that Ohio has always had exactly one school funding formula in force — that
 * the regimes tile the years with no gap and no overlap. That claim is not decoration and it is
 * not obviously true; it is a property of five `effective_from` / `effective_to` pairs written by
 * hand in five separate files, any one of which can be edited.
 *
 * The failure it guards is silent in both directions. A boundary off by one year draws a chart
 * with a seam too small to see, and says there was a year Ohio funded schools by nothing. A
 * boundary overlapping by one draws two bars at the same offset, and says two formulas were in
 * force at once. Neither produces an error, a missing page, or a broken link — they produce a
 * stylesheet making a statement about Ohio.
 *
 * # And the shape of the page, which is the cheaper half
 *
 * Every act placed by a date it carries, every link landing on a node that exists, and no act
 * amending one signed after it. These are the checks that catch a corpus edit before a reader
 * does, and they are here rather than in the end-to-end suite because they are questions about
 * the data and not about the markup — asking them of a built page means a fifty-second build to
 * learn that a node was renamed.
 */

import { expect, test } from "vitest";

import { loadCorpus } from "../../src/lib/corpus.ts";
import { acts, fiscalYear, regimes, renderTimeline, succession } from "../../src/lib/legislation.ts";

const corpus = loadCorpus();
const spans = regimes(corpus);
const all = acts(corpus);
const html = renderTimeline(corpus);

test("the formulas tile the years: no gap, no overlap, one open end", () => {
  /*
   * The load-bearing one. Read the diff before touching an expectation here — a failure means a
   * regime's span moved, which is a claim about which formula Ohio funded schools by in some
   * year, not a fixture that drifted.
   */
  expect(succession(spans)).toEqual([]);
  expect(spans.length).toBeGreaterThan(1);
  // Exactly one regime may be open-ended, and it must be the last.
  expect(spans.filter((r) => r.to == null).length).toBe(1);
  expect(spans[spans.length - 1]!.to).toBeNull();
});

test("the succession check can see a seam and an overlap, in both directions", () => {
  /*
   * A check whose only evidence is that it passes against correct data is not evidence of
   * anything. Three synthetic spans, each a defect the real ones could acquire from a one-year
   * edit to a single YAML file.
   */
  const at = (from: number, to: number | null) =>
    ({ id: `r${from}`, name: `R${from}`, href: "/", from, to, status: "", establishedBy: null });

  expect(succession([at(1, 4), at(5, 8), at(9, null)])).toEqual([]);
  expect(succession([at(1, 4), at(6, 8)])[0]).toMatch(/nothing runs between/);
  expect(succession([at(1, 5), at(5, 8)])[0]).toMatch(/overlap/);
  expect(succession([at(1, null), at(5, 8)])[0]).toMatch(/no end but/);
  expect(succession([at(6, 4)])[0]).toMatch(/ends before it begins/);
});

test("a fiscal-year label parses, and anything else is refused rather than guessed", () => {
  expect(fiscalYear("FY2012")).toBe(2012);
  expect(fiscalYear("  FY1976 ")).toBe(1976);
  // `current` is how the corpus writes an open end, and `null` is how this page draws one. A
  // parser that returned 0 or NaN here would sort the running regime to the start of the chart.
  expect(fiscalYear("current")).toBeNull();
  expect(fiscalYear("2012")).toBeNull();
  expect(fiscalYear("FY12")).toBeNull();
  expect(fiscalYear("")).toBeNull();
});

test("every act sits at a date it carries, and they come out in that order", () => {
  expect(all.length).toBeGreaterThan(10);
  for (const act of all) {
    expect(act.signed, act.id).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    expect(act.year, act.id).toBe(Number.parseInt(act.signed.slice(0, 4), 10));
    expect(act.designation.length, `${act.id} has no designation`).toBeGreaterThan(0);
    expect(act.assembly.length, `${act.id} has no General Assembly`).toBeGreaterThan(0);
  }
  const signed = all.map((a) => a.signed);
  expect([...signed].sort()).toEqual(signed);
  expect(new Set(all.map((a) => a.id)).size).toBe(all.length);
});

test("no act amends one signed after it", () => {
  /*
   * An `amends` edge pointing forward in time is a reciprocal written on the wrong node — the
   * shape H.B. 110 and H.B. 583 already carry correctly, one `amends` and one `amended-by`. The
   * page draws the chain under each act's own row, so a reversed pair reads as the corrective act
   * having been amended by the act it corrected.
   */
  const byHref = new Map(all.map((a) => [a.href, a]));
  for (const act of all) {
    for (const target of act.amends) {
      const earlier = byHref.get(target.href);
      expect(earlier, `${act.designation} amends something outside the class`).toBeDefined();
      expect(
        earlier!.signed < act.signed,
        `${act.designation} (${act.signed}) amends ${earlier!.designation} (${earlier!.signed})`,
      ).toBe(true);
    }
  }
});

test("a regime's establishing act was signed before the regime began", () => {
  /*
   * The join between the two halves of the page, and the one place they could disagree without
   * either being obviously wrong. H.B. 1 was signed in July 2009 and the Evidence-Based Model
   * runs from FY2010, which is what a budget act does — it is signed in the fiscal year before
   * the one it pays for. An establishing act dated *after* its regime's first year means the
   * `establishes` edge is on the wrong node.
   */
  const established = spans.filter((r) => r.establishedBy != null);
  expect(established.length).toBeGreaterThan(0);
  for (const regime of established) {
    expect(
      regime.establishedBy!.year <= regime.from,
      `${regime.establishedBy!.designation} (${regime.establishedBy!.year}) establishes ` +
        `${regime.name}, which begins FY${regime.from}`,
    ).toBe(true);
  }
});

test("a regime with no establishing act began before any act that touches a formula", () => {
  /*
   * The empty cell on the page says "older than this corpus reaches", which is a claim and not a
   * shrug — so it needs to be true of something.
   *
   * Not "older than the oldest act": the oldest act is the 1851 constitutional duty, which is
   * older than everything and establishes no formula, so that comparison is vacuous and this test
   * failed on it. The set that matters is the acts that do something to a regime, and the oldest
   * of those is H.B. 94 (2001). Equal Yield began in FY1976 and Foundation Base Cost in FY1992;
   * both are before the budget acts this corpus collects, which is why they have no establishing
   * act and never will. A regime appearing here that began *after* 2001 would be covering a
   * missing edge with a sentence about the edge of the collection.
   */
  const touching = all.filter((a) => a.action != null);
  expect(touching.length).toBeGreaterThan(0);
  const oldest = Math.min(...touching.map((a) => a.year));
  for (const regime of spans.filter((r) => r.establishedBy == null)) {
    expect(
      regime.from < oldest,
      `${regime.name} begins FY${regime.from}, after the oldest act here that touches a formula ` +
        `(${oldest}), so its missing establishing act is a gap rather than an edge`,
    ).toBe(true);
  }
});

test("every link the page draws lands on something the corpus holds", () => {
  const wiki = [...html.matchAll(/href="(\/wiki\/[^"]+)"/g)].map(([, href]) => href!);
  expect(wiki.length).toBeGreaterThan(all.length);
  const missing = wiki.filter((href) => {
    const parts = href.split("/").filter(Boolean).slice(1);
    return parts.length === 1
      ? !corpus.byClass.has(parts[0]!)
      : !corpus.byId.has(`${parts[0]}/${parts[1]}`);
  });
  expect([...new Set(missing)]).toEqual([]);
});

test("the page names every formula and every act the corpus holds", () => {
  /*
   * Against the corpus, not against `acts()`. The first version of this asserted that the page
   * named every act *the module returned*, which is a check measuring a thing against itself: a
   * `.slice(1)` in `acts()` dropped one act from the page and from the expectation together, and
   * this passed. The count has to come from somewhere the renderer cannot reach.
   */
  const heldActs = corpus.byClass.get("legislation")?.nodes ?? [];
  const heldRegimes = corpus.byClass.get("funding-regime")?.nodes ?? [];
  expect(all.length, "the timeline drops acts the corpus holds").toBe(heldActs.length);
  expect(spans.length, "the timeline drops regimes the corpus holds").toBe(heldRegimes.length);

  for (const node of heldRegimes) {
    const name = node.properties.find((p) => p.name === "name")?.value.trim() || node.label;
    expect(html, `${name} is missing from the page`).toContain(name);
  }
  for (const node of heldActs) {
    const designation =
      node.properties.find((p) => p.name === "designation")?.value.trim() || node.label;
    expect(html, `${designation} is missing from the page`).toContain(designation);
  }
});

test("the bars span the axis exactly once", () => {
  /*
   * The rendered form of the tiling claim. Offsets and widths are written into inline styles as
   * percentages, so the last bar has to end at 100 and each has to begin where the previous one
   * ended. A rounding error here is invisible; a logic error draws the chart wrong.
   */
  const bars = [...html.matchAll(/margin-left:([\d.]+)%;width:([\d.]+)%/g)].map(([, l, w]) => ({
    left: Number.parseFloat(l!),
    width: Number.parseFloat(w!),
  }));
  expect(bars.length).toBe(spans.length);
  expect(bars[0]!.left).toBe(0);
  bars.forEach((bar, index) => {
    expect(bar.width, `bar ${index} has no width`).toBeGreaterThan(0);
    const previous = bars[index - 1];
    if (previous) expect(bar.left).toBeCloseTo(previous.left + previous.width, 1);
  });
  const last = bars[bars.length - 1]!;
  expect(last.left + last.width).toBeCloseTo(100, 1);
});

test("the standing instruments are the ones that neither pay nor amend", () => {
  // The third card's whole claim. H.B. 583 appropriates nothing either and is not standing law —
  // it amends H.B. 110 — so "does not appropriate" alone would put a corrective act in a card
  // about instruments that were in force under every formula on the page.
  const standing = all.filter((a) => a.funds == null && a.amends.length === 0);
  expect(standing.length).toBeGreaterThan(0);
  // `<=` and not `<`: H.B. 920 was signed in 1976 and the Equal Yield Formula runs from FY1976.
  // Ohio's oldest surviving school tax law and its oldest recorded formula are the same year, and
  // a strict inequality here fails on that coincidence rather than on a defect.
  const begins = Math.min(...spans.map((r) => r.from));
  for (const act of standing) expect(act.year, act.designation).toBeLessThanOrEqual(begins);
  const corrective = all.filter((a) => a.funds == null && a.amends.length > 0);
  expect(corrective.length).toBeGreaterThan(0);
  for (const act of corrective) expect(standing).not.toContain(act);
});

test("this file writes no year of its own, and neither does the page", () => {
  /*
   * `yearLiterals.spec.ts` polices `src/`. The same rule matters more here than anywhere, because
   * every figure on this page is a date and a hard-coded one would look exactly like a derived
   * one. The assertion is on the *module*: nothing it emits may contain a four-digit year that is
   * not in the corpus.
   */
  const years = new Set<number>();
  for (const act of all) {
    years.add(act.year);
    /* A General Assembly names its own years — "Constitutional Convention of 1850-51" — and the
       page prints the string verbatim. A year inside a corpus-provided string is by definition
       not one this module typed, which is what is being asserted. */
    for (const [, y] of act.assembly.matchAll(/\b(1[89]\d{2}|20\d{2})\b/g)) {
      years.add(Number.parseInt(y!, 10));
    }
  }
  for (const regime of spans) {
    years.add(regime.from);
    if (regime.to != null) years.add(regime.to);
  }
  const stray = [...html.matchAll(/\b(1[89]\d{2}|20\d{2})\b/g)]
    .map(([, y]) => Number.parseInt(y!, 10))
    .filter((year) => !years.has(year));
  expect([...new Set(stray)], "a year on the page comes from neither an act nor a regime").toEqual(
    [],
  );
});

test("an empty corpus renders nothing rather than an empty chart", () => {
  const bare = { ...corpus, byClass: new Map() } as typeof corpus;
  expect(renderTimeline(bare)).toBe("");
});
