/**
 * What year a figure is on, and the ways that used to be got wrong.
 *
 * Every case here is a defect this repository actually had. The feed's provenance paragraph named
 * every year in the feed and said *"millage is TY2023"* while all 609 districts carried
 * `tax_year: 2024`; the web layer carried about 190 four-digit year literals, one of them the
 * report card's school year typed into an Astro `<meta>` description.
 */

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";
import { renderChargeOff } from "../../src/lib/tax.ts";
import { GLOSSARY, term } from "../../src/lib/glossary.ts";
import { schoolYearBefore, seriesYear, yearChip, yearChipPair, yearTitle } from "../../src/lib/year.ts";

test("every series the feed carries names its reckoning as well as its digits", () => {
  /*
   * The whole point. A tax year is a calendar year whose revenue reaches the district in the
   * *following* fiscal year, so `2024` on a millage figure and `FY2024` on a spending figure are
   * eleven months apart. A consumer given only the digits will subtract them.
   */
  const { bundle } = loadFeed();
  expect(bundle.series_years.length).toBeGreaterThan(0);
  for (const year of bundle.series_years) {
    expect(["fiscal", "tax", "school"], year.series).toContain(year.kind);
    expect(year.label, year.series).not.toBe("");
    expect(year.source, year.series).not.toBe("");
  }
});

test("the millage year is read off the data, not off the provenance paragraph", () => {
  /*
   * The defect that motivated the block. The paragraph said TY2023; the column said 2024. A
   * sentence and a column cannot disagree in a way anything notices, and the sentence is the half
   * a reader sees.
   */
  const { bundle } = loadFeed();
  const observed = Math.max(
    ...bundle.districts.flatMap((d) => (d.millage ? [d.millage.tax_year] : [])),
  );
  expect(seriesYear("millage")?.label).toBe(String(observed));
});

test("the provenance paragraph no longer restates a year the index carries", () => {
  // One place, or the two drift. This is the check that keeps the paragraph from growing them
  // back the next time someone documents the feed in prose.
  const { bundle } = loadFeed();
  expect(bundle.provenance).not.toMatch(/\bTY\d{4}\b/);
  expect(bundle.provenance).not.toMatch(/\bFY\d{4}\b/);
});

test("a span of one year is not written as a range", () => {
  // `FY2025-FY2025` tells a reader there is a span to think about when there is not.
  for (const year of loadFeed().bundle.series_years) {
    const [first, last] = year.label.split("-");
    if (last !== undefined && first !== undefined && last.length === first.length) {
      expect(first, year.series).not.toBe(last);
    }
  }
});

test("the report card's two reckonings are carried apart", () => {
  /*
   * One download, two years: attainment for the 2024-25 school year and operating expenditure for
   * FY2025. A card showing both under one label is picking one and being wrong about the other
   * half of its own figures.
   */
  expect(seriesYear("outcome.performance")?.kind).toBe("school");
  expect(seriesYear("outcome.spending")?.kind).toBe("fiscal");
  expect(seriesYear("outcome.performance")?.label).not.toBe(
    seriesYear("outcome.spending")?.label,
  );
});

test("a chip carries the long form where the short one is ambiguous", () => {
  const tax = seriesYear("millage")!;
  expect(yearTitle(tax)).toContain("tax year");
  expect(yearTitle(tax)).toContain("calendar year");
  expect(yearTitle(tax)).toContain(tax.source);

  const chip = yearChip("millage");
  expect(chip).toContain('data-kind="tax"');
  /*
   * A button carrying the long form as its own name, and no `title`.
   *
   * `title` is why this needed changing: it reaches a mouse and nothing else, so the distinction
   * between a tax year and a fiscal year — the whole point of the chip — was unavailable to a
   * keyboard, to a screen reader and to every touch screen. A `title` here now would also be read
   * out on top of the name. Same rule `.term` follows.
   */
  expect(chip).not.toContain("title=");
  expect(chip).toContain("<button");
  expect(chip).toContain(`aria-label="${yearTitle(tax).replace(/"/g, "&quot;")}"`);
});

test("a mixed chip names both years and says which is which", () => {
  // "FY2027 · FY2022" tells a reader there are two years and not which is which.
  const pair = yearChipPair("outcome.performance", "outcome.spending", "spending");
  expect(pair).toContain(seriesYear("outcome.performance")!.label);
  expect(pair).toContain(seriesYear("outcome.spending")!.label);
  expect(pair).toContain("spending");
});

test("an absent series renders nothing rather than a placeholder", () => {
  /*
   * A missing chip is the honest rendering of "this block is not in the feed". A chip reading
   * "unknown" beside figures that are present is worse than no chip, because it attaches
   * uncertainty to the wrong thing.
   */
  const { bundle } = loadFeed();
  const absent = { ...bundle, series_years: [] };
  expect(seriesYear("millage", absent)).toBeNull();
});

test("every glossary definition is a distinction, and every link resolves to a node", async () => {
  /*
   * A term earns an entry when the everyday reading and the Ohio reading differ. The list is short
   * on purpose: a page where every third word is underlined has taught the reader to ignore them.
   */
  const { loadCorpus } = await import("../../src/lib/corpus.ts");
  const nodes = new Set(loadCorpus().nodes.map((n) => `/wiki/${n.id}`));

  expect(Object.keys(GLOSSARY).length).toBeGreaterThan(0);
  const broken: string[] = [];
  for (const [slug, entry] of Object.entries(GLOSSARY)) {
    expect(entry.definition.length, slug).toBeGreaterThan(40);
    if (entry.href && !nodes.has(entry.href)) broken.push(`${slug} -> ${entry.href}`);
  }
  expect(broken).toEqual([]);
});

test("a term renders as a focusable control with its definition in the markup", () => {
  /*
   * Three constraints and one shape that satisfies all of them: no script, no hover, and announced
   * once. A `title` would be read *as well as* the description, so there deliberately is not one.
   */
  const html = term("equivalent-pupil", "per equivalent pupil");
  expect(html).toContain('<button type="button" class="term"');
  expect(html).toContain('aria-describedby="def-equivalent-pupil"');
  expect(html).toContain('id="def-equivalent-pupil"');
  expect(html).toContain("need");
  expect(html).not.toContain("title=");
});

test("an unknown term throws rather than silently rendering bare text", () => {
  // A silent fallback lets a renamed entry strip the definitions off a page with nothing to
  // notice — the same class of failure as a link that resolves to a plausible 404.
  expect(() => term("not-a-term", "text")).toThrow(/no glossary entry/);
});

test("the cross-department agreement count is computed, not typed", () => {
  /*
   * It was the literal `219` in `taxes.astro`, beside the literals `TY2023` and `TY2024`. All
   * three were correct when written and all three move together: the profile report's column is
   * `effective_class1_millage_ty23`, a year behind Taxation's latest **by construction**, so when
   * either publisher advances the pair shifts and the copy would have gone on asserting the old
   * one. The same shape as "millage is TY2023" in the provenance paragraph.
   *
   * This asserts the count is derived from the two columns rather than pinned to a number — if it
   * were still a literal, changing the fixture would leave this passing and the page wrong.
   */
  const { bundle, tax } = loadFeed();
  const recomputed = bundle.districts.filter((d) => {
    const latest = d.property_tax[d.property_tax.length - 1];
    return (
      latest != null &&
      d.effective_class1_millage != null &&
      Math.abs(latest.class1_rate - d.effective_class1_millage) <= 0.01
    );
  }).length;

  expect(tax.agreeOnLatest).toBe(recomputed);
  // And it is a minority, which is the fact the card exists to explain: most districts disagree
  // on the latest year because only one department has published it.
  expect(tax.agreeOnLatest).toBeLessThan(bundle.statewide.districts / 2);
});

test("the recognised-valuation aggregates are computed, not typed", () => {
  /*
   * They were the literals `8.2%` and `$793m`, in a sentence that also typed `TY2024` — the same
   * three-literals-in-one-sentence shape as the `219`/`TY2023`/`TY2024` defect above, in the last
   * paragraph on that page still doing it.
   *
   * These move for a reason nothing else on the page moves for: Taxation revalues counties on a
   * staggered calendar, so which districts are mid-phase-in changes every year by construction.
   * A literal here is stale on a schedule.
   */
  const { bundle, tax } = loadFeed();

  let deferred = 0;
  let taxable = 0;
  let chargeOff = 0;
  for (const d of bundle.districts) {
    const latest = d.property_tax[d.property_tax.length - 1];
    if (d.regime?.recognized_share == null || latest == null) continue;
    const share = (1 - d.regime.recognized_share) * latest.total_value;
    deferred += share;
    taxable += latest.total_value;
    chargeOff += share * (d.regime.charge_off_mills / 1000);
  }

  expect(tax.deferredShare).toBeCloseTo(deferred / taxable, 10);
  expect(tax.deferredChargeOff).toBeCloseTo(chargeOff, 4);

  // Weighted by the panel, not averaged over districts. A plain mean of the 609 shares differs,
  // and if this ever starts matching it the weighting has been dropped.
  const shares = bundle.districts
    .filter((d) => d.regime?.recognized_share != null)
    .map((d) => 1 - d.regime!.recognized_share!);
  const unweighted = shares.reduce((a, b) => a + b, 0) / shares.length;
  expect(tax.deferredShare).not.toBeCloseTo(unweighted, 3);

  // And the deferral is real but partial: neither zero nor the whole base.
  expect(tax.deferredShare).toBeGreaterThan(0);
  expect(tax.deferredShare).toBeLessThan(0.5);
});

test("the charge-off paragraph's tax year is the one the panel carries", () => {
  /*
   * Rendered, not read off the source, because `yearLiterals.spec.ts` cannot see this one.
   * That gate exempts **files**: `lib/tax.ts` is allowlisted for FY2008 gap aid and FY2027, and
   * that licence silently covered the `TY2024` this paragraph used to type. Putting `TY2024`
   * back passes the allowlist today — which is how it got there.
   *
   * So the year is checked where it is printed, against the panel it claims to describe. What
   * that catches is the year going **stale**, which is the failure mode: typing `TY2024` back in
   * still passes here while the panel says 2024, and only a per-literal allowlist would see it.
   * That is filed rather than fixed — enumerating the 32 literals across the 11 allowlisted files
   * is its own change, and `lib/district.ts` alone carries 9 against a reason naming 4.
   */
  const { bundle, tax } = loadFeed();
  const district = bundle.districts.find((d) => d.regime?.charge_off_local_share != null);
  expect(district, "no district carries a charge-off counterfactual").toBeDefined();

  const latest = district!.property_tax[district!.property_tax.length - 1]!;
  const html = renderChargeOff(district!, bundle.statewide, tax);

  const years = [...html.matchAll(/TY(20\d\d)/g)].map((m) => Number(m[1]));
  expect(years.length, "the paragraph states no tax year at all").toBeGreaterThan(0);
  for (const year of years) expect(year).toBe(latest.tax_year);
});

test("a school year steps back without inventing a century", () => {
  // The two labels behind the current report card year were literals beside a derived one, so
  // they would have drifted apart rather than all going stale together.
  expect(schoolYearBefore("2024-25", 0)).toBe("2024-25");
  expect(schoolYearBefore("2024-25", 1)).toBe("2023-24");
  expect(schoolYearBefore("2024-25", 2)).toBe("2022-23");
  expect(schoolYearBefore("2000-01", 1)).toBe("1999-00");
  // Not `YYYY-YY` — return nothing rather than compose from whatever this is.
  expect(schoolYearBefore("FY2027", 1)).toBe("");
});

test("no corpus node states a statewide constant the feed contradicts", () => {
  /*
   * The corpus disagreed with itself, and with the feed.
   *
   * `fsfp-local-capacity-measure` was corrected from a 5% minimum state share to 10% — the
   * department's calculator says `0.1` in as many words — and `fsfp-input-year-refresh` went on
   * saying 5% in two places, one of them a worked example computing 5% of a base cost increase.
   * Nothing connected the two nodes, or either node to `bundle.statewide.minimum_state_share`.
   *
   * This is the prose-beside-data defect one level up: not a label against the column it
   * describes, but two documents describing the same constant with nothing holding them together.
   * The feed is authoritative — `crates/foundation` computes it — so the corpus may not contradict
   * it.
   *
   * Narrow on purpose. It checks the constants a node is *likely to restate in prose* and that the
   * feed carries a single unambiguous value for. A general "no number disagrees with any number"
   * check is not available and would not be worth its false positives.
   */
  const { bundle } = loadFeed();
  const { loadCorpus } = require("../../src/lib/corpus.ts") as typeof import("../../src/lib/corpus.ts");

  const percent = (v: number) => `${Number((v * 100).toFixed(2))}`.replace(/\.0+$/, "");
  const stated = percent(bundle.statewide.minimum_state_share);

  // Any other whole-percent reading of "minimum state share" is a contradiction of the feed.
  const wrong = new RegExp(`\\b(?!${stated}\\b)\\d{1,2}(\\.\\d+)?%\\s+minimum state share`, "i");

  const offenders: string[] = [];
  for (const node of loadCorpus().nodes) {
    const prose = [node.description, node.findings ?? ""].join("\n\n");
    const hit = wrong.exec(prose);
    if (hit) offenders.push(`${node.id}: "${hit[0]}" against the feed's ${stated}%`);
  }
  expect(offenders, "the feed is authoritative for this constant").toEqual([]);
});
