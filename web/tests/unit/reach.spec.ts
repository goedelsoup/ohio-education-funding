/**
 * `/reach`'s library, which had no unit test at all.
 *
 * # Why that mattered
 *
 * `reach.ts` is the route's whole argument. `envelope` measures a frame over five corner lever runs
 * so that a district which did not move looks like a district which did not move; `viewFromQuery`
 * decides what a shared link draws; `presets` decides which named positions are offered at all. The
 * behaviour was observed — `tests/e2e/app.spec.ts`'s `reach` describe covers the frame holding
 * still, the identity line's gating and the flat trails — but only through a rendered SVG, which
 * needs a build and cannot reach a quantile boundary or a declared `bounds` override directly.
 *
 * `reachability.spec.ts` is about corpus edges. The name collides and the coverage did not exist.
 *
 * # What is asserted, and what deliberately is not
 *
 * Not the values. The frame moves when the feed does and pinning `$19,175` here would rebuild in a
 * test the defect the route was rebuilt to remove. What is asserted is the *shape* of each frame —
 * which is a property of the quantity — and the invariants: that the frame is the same under every
 * lever position, that a handful of districts are outside it rather than none or most, and that
 * every offered preset is a position the sliders can actually hold.
 *
 * The feed is the committed one, loaded the way `refresh.spec.ts` and `policy.spec.ts` load it. A
 * `Bundle` is assignable to `Panel` — see the note on `PanelDistrict` — so it is passed unchanged,
 * as `reach.astro` already does.
 */

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";
import type { Dimension } from "../../src/lib/reach.ts";
import {
  DEFAULT_VIEW,
  DIMENSIONS,
  DIMENSION_KEYS,
  FRAME_QUANTILE,
  envelope,
  groups,
  inScope,
  presets,
  scopeCounties,
  viewFromQuery,
} from "../../src/lib/reach.ts";
import { MIN_CLOUD } from "../../src/lib/plot/spec.ts";
import { compare } from "../../src/lib/order.ts";
import { applyAll, currentLaw, modelOf } from "../../src/lib/policy.ts";
import { LEVER_BOUNDS, defaultLevers } from "../../src/lib/scenario.ts";
import type { Panel } from "../../src/lib/types.ts";

const panel = loadFeed().bundle as unknown as Panel;

/**
 * Every district's outcome under the enacted act, in panel order.
 *
 * The page's zero, and what a `Dimension.of` accessor is given. Computed once: it is a property of
 * the feed rather than of any test here, and running it per assertion would run the formula over
 * 609 districts a dozen times to get the same answer.
 */
const law = applyAll(panel.districts, currentLaw(modelOf(panel.statewide)), modelOf(panel.statewide));

test("a share's frame is 0 to 1 absolutely, and is not measured", () => {
  /*
   * The most stable frame there is, and the reason it is declared rather than measured: a share
   * cannot leave the unit interval, so asking the data would be spending five formula runs to
   * rediscover a definition.
   */
  for (const key of DIMENSION_KEYS) {
    if (DIMENSIONS[key].frame !== "unit") continue;
    expect(envelope(panel, key), `${key} is a share`).toEqual([0, 1]);
  }
  // And there is at least one, or the loop above asserts nothing.
  expect(DIMENSION_KEYS.filter((k) => DIMENSIONS[k].frame === "unit").length).toBeGreaterThan(0);
});

test("the typology axis takes its declared bounds, so the low tick is a locale and not a refusal", () => {
  /*
   * 1 through 8, from `Dimension.bounds`. A `zero` frame would put the floor at 0, which is the
   * code the department uses for *declining to classify* — so the low tick came out blank, because
   * there is no locale to label it with.
   */
  expect(envelope(panel, "urbanicity")).toEqual([1, 8]);
});

test("a dollar frame starts at zero and a diverging one is symmetric about it", () => {
  const [low, high] = envelope(panel, "formula")!;
  expect(low, "a per-pupil dollar figure is read as a height above the baseline").toBe(0);
  expect(high).toBeGreaterThan(0);

  const [down, up] = envelope(panel, "change")!;
  expect(down).toBeLessThan(0);
  expect(up).toBeGreaterThan(0);
  expect(down, "zero is the middle of the axis, not wherever the data puts it").toBeCloseTo(-up, 6);
});

test("both log frames are strictly positive, which is the condition the log scale is gated on", () => {
  /*
   * `renderReach` draws a log axis only where the envelope's low end clears zero, because a log
   * axis that reached zero would be a silently blank chart. If either of these came back at or
   * below zero the axis would quietly fall back to linear and a 4-pupil island would sit beside a
   * 43,000-pupil city on it.
   */
  for (const key of DIMENSION_KEYS) {
    if (DIMENSIONS[key].frame !== "log") continue;
    const measured = envelope(panel, key);
    expect(measured, `${key} carries values somewhere`).not.toBeNull();
    expect(measured![0], `${key}'s low end`).toBeGreaterThan(0);
    expect(measured![1]).toBeGreaterThan(measured![0]);
  }
  expect(DIMENSION_KEYS.filter((k) => DIMENSIONS[k].frame === "log")).toEqual([
    "valuation",
    "adm",
  ]);
});

test("the frame is the same under every lever position", () => {
  /*
   * The invariant the whole route rests on, and the one thing a movement chart must not get wrong.
   * An axis fitted to the run being drawn refits with it, so dragging base cost holds the cloud
   * still and moves the *frame* underneath — and a reader cannot tell from that whether the
   * districts moved or the ruler did.
   *
   * `envelope` takes no levers, which is what makes this true; the assertion is that it stays that
   * way. It is currently checked end-to-end by reading drawn axis labels, which needs a build and
   * can only see the two axes on screen.
   */
  const first = DIMENSION_KEYS.map((key) => envelope(panel, key));
  const again = DIMENSION_KEYS.map((key) => envelope(panel, key));
  expect(again).toEqual(first);
  // Every dimension has a frame, or a null is being read as "no opinion" somewhere downstream.
  expect(first.filter((f) => f == null)).toEqual([]);
});

test("the frame is a quantile, so a handful of districts sit outside it rather than none or most", () => {
  /*
   * Kelleys Island Local has four pupils. Its aid per pupil is far above the 99th percentile and
   * its base cost per pupil further still, and a denominator of four makes a per-pupil figure a
   * different kind of quantity — so the axis is sized to hold most districts and the rest are
   * clipped and counted.
   *
   * Both halves matter. A zero here would mean the bound had gone back to the maximum, which is
   * the defect: −$56 to $147,555, a frame in which every district in Ohio is a smudge against the
   * left edge. A large number would mean the quantile had been set somewhere useless.
   */
  const model = modelOf(panel.statewide);
  const levers = defaultLevers(model);
  const [, high] = envelope(panel, "realized")!;
  /* Widened to the interface, as `envelope` does: `satisfies` keeps each entry's literal type, on
     which the parameters it does not name do not exist. */
  const realized: Dimension = DIMENSIONS.realized;
  const outside = panel.districts.filter((d, i) => {
    const value = realized.of(d, law[i]!, levers);
    return value != null && value > high;
  }).length;
  expect(outside, "the frame holds most districts").toBeGreaterThan(0);
  expect(outside, "and is not fitted to the four-pupil island").toBeLessThan(
    panel.districts.length * (1 - FRAME_QUANTILE) * 3,
  );
});

test("every group the picker offers has a district in it, and the labels come off the feed", () => {
  const found = groups(panel.districts);
  expect(found.length).toBeGreaterThan(1);
  expect(found.map((g) => g.code)).toEqual([...found.map((g) => g.code)].sort((a, b) => a - b));
  for (const group of found) {
    expect(group.districts, `${group.code} is offered with no district in it`).toBeGreaterThan(0);
    expect(group.short.length, `${group.code} has no short name on the feed`).toBeGreaterThan(0);
  }
  // The sum is the population the picker can reach — every district the department classified.
  expect(found.reduce((n, g) => n + g.districts, 0)).toBe(
    panel.districts.filter((d) => d.typology != null).length,
  );
});

test("every preset is a position the sliders can hold", () => {
  /*
   * The module's own rule, which nothing checked. `2.65` — the scale at which Delphos City finally
   * clears its guarantee — is outside `LEVER_BOUNDS.baseCostScale`, and offering it as a button
   * would mint a position the controls cannot represent and the query string cannot carry. That a
   * preset is unreachable is the finding, and it belongs in prose rather than in a control that
   * would have to lie about where the slider is.
   */
  const offered = presets(panel);
  expect(offered.length).toBeGreaterThan(1);
  expect(offered[0]!.key, "current law is where the row starts").toBe("law");
  for (const preset of offered) {
    for (const [field, bound] of Object.entries(LEVER_BOUNDS)) {
      const value = preset.levers[field as keyof typeof LEVER_BOUNDS];
      expect(value, `${preset.key} sets no ${field}`).toBeTypeOf("number");
      expect(value, `${preset.key}'s ${field} is below the slider`).toBeGreaterThanOrEqual(
        bound.min,
      );
      expect(value, `${preset.key}'s ${field} is above the slider`).toBeLessThanOrEqual(bound.max);
    }
  }
});

test("a preset is a whole position rather than the fields it is about", () => {
  /*
   * They were partial first, and the row then reported three of them pressed at once because each
   * was still true of the fields it had named. Under a heading reading "Start from", three
   * simultaneous selections is a lie about what the reader is looking at.
   */
  const law = presets(panel)[0]!.levers;
  for (const preset of presets(panel)) {
    expect(Object.keys(preset.levers).sort(), `${preset.key} is a partial position`).toEqual(
      Object.keys(law).sort(),
    );
  }
});

test("a view falls back per field rather than all-or-nothing", () => {
  const read = viewFromQuery(new URLSearchParams("vx=poverty&vy=nonsense&c=change"));
  expect(read.x).toBe("poverty");
  expect(read.y, "an unknown dimension key falls back alone").toBe(DEFAULT_VIEW.y);
  expect(read.shading).toBe("change");
});

test("the highlight is held to the codes the department assigns", () => {
  /*
   * A query string naming code 11 would otherwise light nothing and look like a page that had lost
   * its highlight. 0 is a code the department does assign — it is *declining to classify*, three
   * districts in this panel — so it is admitted.
   */
  expect(viewFromQuery(new URLSearchParams("ty=11")).highlight).toBe(DEFAULT_VIEW.highlight);
  expect(viewFromQuery(new URLSearchParams("ty=-1")).highlight).toBe(DEFAULT_VIEW.highlight);
  expect(viewFromQuery(new URLSearchParams("ty=0")).highlight).toBe(0);
  expect(viewFromQuery(new URLSearchParams("ty=4")).highlight).toBe(4);
});

test("the highlight is read from `ty`, and a guarantee rule in `g` is not a typology code", () => {
  /*
   * The collision this is the regression test for. `toQuery` wrote the guarantee rule to `g` and
   * then, whenever the cloud was coloured by district type, overwrote it with the typology code —
   * so a reader who retired half the guarantee got a URL carrying `g=1`, `fromQuery` matched none
   * of the four rule names, and the lever fell back to `as-enacted`. The picture was plausible and
   * the setting was gone.
   *
   * Both halves are asserted: the view no longer reads `g` at all, and a `g` carrying a rule name
   * leaves the highlight at its default rather than resolving to a code by accident.
   */
  expect(viewFromQuery(new URLSearchParams("c=type&g=3")).highlight).toBe(DEFAULT_VIEW.highlight);
  expect(viewFromQuery(new URLSearchParams("c=type&g=phase-out")).highlight).toBe(
    DEFAULT_VIEW.highlight,
  );
  expect(viewFromQuery(new URLSearchParams("c=type&g=phase-out&ty=3")).highlight).toBe(3);
});

test("the trails are on unless a link says otherwise", () => {
  // Absent means the default, which is on. Only an explicit `0` turns them off — a link that had
  // to opt *in* to the trails would draw a still to anyone who shared one before the query string
  // existed.
  expect(viewFromQuery(new URLSearchParams("")).trails).toBe(true);
  expect(viewFromQuery(new URLSearchParams("t=1")).trails).toBe(true);
  expect(viewFromQuery(new URLSearchParams("t=0")).trails).toBe(false);
});

test("every dimension places every district, or says it cannot", () => {
  /*
   * Three of the eleven are nullable and the card counts them; the other eight must never be. A
   * `NaN` would be worse than a null — it is dropped by the same guard in `renderReach` and looks
   * identical in the count, but it means an accessor divided by something it did not check.
   */
  const model = modelOf(panel.statewide);
  const levers = defaultLevers(model);
  const nullable: string[] = [];
  for (const key of DIMENSION_KEYS) {
    const dim: Dimension = DIMENSIONS[key];
    const values = panel.districts.map((d, i) => dim.of(d, law[i]!, levers));
    if (values.some((v) => v == null)) nullable.push(key);
    for (const value of values) {
      if (value != null) expect(Number.isFinite(value), `${key} produced a non-finite value`).toBe(true);
    }
  }
  expect(nullable.sort()).toEqual(["poverty", "urbanicity", "valuation"]);
});

test("an absent highlight is the default, not code 0", () => {
  /*
   * `Number(null)` is `0`, and 0 is a code the department assigns — it is *declining to classify*,
   * three districts in this panel. So the absent case parsed as a real code, cleared the range
   * check and beat the default: a bare `/reach` coloured by district type lit three districts where
   * its own prose says the default is *Rural, high student poverty*, "both the largest group and
   * the one the funding argument is usually about".
   *
   * `?ty=` is the same trap by a different route, since `Number("")` is 0 as well.
   */
  expect(viewFromQuery(new URLSearchParams("")).highlight).toBe(DEFAULT_VIEW.highlight);
  expect(viewFromQuery(new URLSearchParams("c=type")).highlight).toBe(DEFAULT_VIEW.highlight);
  expect(viewFromQuery(new URLSearchParams("ty=")).highlight).toBe(DEFAULT_VIEW.highlight);
  // And 0 still reaches the highlight when a link actually names it.
  expect(viewFromQuery(new URLSearchParams("ty=0")).highlight).toBe(0);
});

/*
 * The scope: which districts the reader is asking about.
 *
 * `inScope` carries the argument for why a selection lights rather than subsets, and the two facts
 * it rests on are measurable here rather than assertable only by eye.
 *
 * What is **not** here is anything that calls `renderReach`. It renders through `plot/client.ts`,
 * which needs a real DOM because it is the browser half of the pair — `plot/ssr.ts` is the one that
 * carries its own. So the drawn consequences of a scope (the frame holding still across selections,
 * the counts restated against it, a single district still getting a cloud) are asserted in
 * `tests/e2e/app.spec.ts`'s `reach` describe, against the page. Standing up a fake document here to
 * run the browser renderer in node would be testing a fiction.
 */

test("no selection is the whole state, and an unresolvable one is no selection", () => {
  expect(inScope(panel.districts, { counties: [], districts: [] })).toBeNull();
  /*
   * `?co=nowhere` is a link to a county that does not exist, and the honest answer to it is the page
   * a bare `/reach` draws — not an empty cloud. The control cannot produce such a selection, since
   * every option comes off this panel, so this is the query-string path.
   */
  expect(inScope(panel.districts, { counties: ["nowhere"], districts: [] })).toBeNull();
  expect(inScope(panel.districts, { counties: [], districts: ["999999"] })).toBeNull();
});

test("a county scope is its districts, and a named district is added to them", () => {
  const counties = scopeCounties(panel.districts);
  const athens = counties.find((c) => c.slug === "athens")!;
  const scope = inScope(panel.districts, { counties: ["athens"], districts: [] })!;
  expect(scope.size, "every district the department attributes to the county").toBe(athens.districts);

  /*
   * Union, not intersection. The department attributes each district to exactly one county, so
   * intersecting a county with a district outside it would make the scope empty — and a reader who
   * picks Athens County and then names Cleveland has asked for both.
   */
  const cleveland = panel.districts.find((d) => d.name.startsWith("Cleveland Municipal"))!;
  expect(cleveland.county).not.toBe("Athens");
  const both = inScope(panel.districts, {
    counties: ["athens"],
    districts: [cleveland.irn],
  })!;
  expect(both.size).toBe(athens.districts + 1);
  expect(both.has(cleveland.irn)).toBe(true);
});

test("a district named inside a county it is already in is not counted twice", () => {
  const inside = panel.districts.find((d) => d.county === "Athens")!;
  const scope = inScope(panel.districts, { counties: ["athens"], districts: [inside.irn] })!;
  const alone = inScope(panel.districts, { counties: ["athens"], districts: [] })!;
  expect(scope.size).toBe(alone.size);
});

test("the counties are every one the feed carries, by name, with their district counts", () => {
  const counties = scopeCounties(panel.districts);
  expect(counties.length).toBe(new Set(panel.districts.map((d) => d.county)).size);
  expect(counties.reduce((n, c) => n + c.districts, 0)).toBe(panel.districts.length);
  expect(counties.map((c) => c.name)).toEqual([...counties.map((c) => c.name)].sort(compare));
  // Every slug round-trips through the router, and no two counties share one.
  expect(new Set(counties.map((c) => c.slug)).size).toBe(counties.length);
  for (const county of counties) expect(county.slug).toMatch(/^[a-z0-9-]+$/);
});

test("most of Ohio's counties hold too few districts to be a cloud", () => {
  /*
   * The measurement the whole design turns on, asserted so it cannot quietly stop being true. A
   * selection that *subsetted* the cloud would render nothing at all for these — `scatterSpec`
   * refuses fewer than `MIN_CLOUD` points and a null spec renders to the empty string — so the card
   * would come out as a heading, a legend and nothing between them.
   */
  const counties = scopeCounties(panel.districts);
  const small = counties.filter((c) => c.districts < MIN_CLOUD);
  expect(small.length, "a subsetting filter would blank the card for this many counties")
    .toBeGreaterThan(counties.length / 2);
  expect(counties.filter((c) => c.districts === 1).length, "and some hold exactly one")
    .toBeGreaterThan(0);
});

test("a scope is carried in the query string and held to shape", () => {
  const read = viewFromQuery(new URLSearchParams("co=athens,van-wert&d=043786,045237"));
  expect(read.counties).toEqual(["athens", "van-wert"]);
  expect(read.districts).toEqual(["043786", "045237"]);

  // An IRN is six digits, always — 28 district names in the feed are not unique, so it is the only
  // safe key and a five-digit one is not a district.
  expect(viewFromQuery(new URLSearchParams("d=43786,043786")).districts).toEqual(["043786"]);
  expect(viewFromQuery(new URLSearchParams("co=Athens County")).counties).toEqual([]);
  // A set written twice is a set.
  expect(viewFromQuery(new URLSearchParams("co=athens,athens")).counties).toEqual(["athens"]);
  expect(viewFromQuery(new URLSearchParams("")).counties).toEqual([]);
});
