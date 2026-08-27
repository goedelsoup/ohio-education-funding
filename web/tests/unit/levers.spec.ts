/**
 * A lever value that reached the page from a URL is held to what the page can run.
 *
 * # The defect
 *
 * The bounds lived only on the five range inputs in `ScenarioControls.astro`, which made the DOM
 * the validator: `readLevers` reads `input.value`, and a browser has already clamped that against
 * the control's own `min` and `max`. Every path went through a control, so every path was safe.
 *
 * Then `?draft=` gained the ability to render *before* the first read of one — deliberately, and
 * for a good reason: a draft's `base-cost` provision is `1.0395` and the slider steps by `0.01`, so
 * reading the control for the first render would put a number that is not the bill's under a banner
 * saying it is. The consequence was that a query string composed with a draft was validated by
 * nothing at all:
 *
 *   - `?draft=x&h=999999` reached `forecastPath` with a million-year horizon and locked the tab
 *   - `?draft=x&base=100` rendered `+$928.15B` under a slider showing `1.3` and a label reading
 *     `+9900%`, with `base=100` written back to the address bar — four readings of one scenario
 *   - `?draft=x&min=-5` ran the formula at a negative minimum state share
 *
 * # Why the step is not enforced with the ends
 *
 * Because `1.0395` has to survive. Snapping a query value to the grid would reintroduce the defect
 * the draft path exists to avoid, one layer up, and rejecting it would break every link this page
 * mints for a bill. The ends are what make a scenario runnable; the grid is what makes it
 * reachable with a mouse, and those are different claims.
 */

import { expect, test } from "vitest";

import { LEVER_BOUNDS, clampLevers, defaultLevers } from "../../src/lib/scenario.ts";

/** The feed's own two ends, as `boot` builds them. */
const HORIZON = { base: 2026, max: 2036 };

test("a value past either end comes back at the end", () => {
  expect(
    clampLevers(
      { baseCostScale: 100, minimumStateShare: -5, phaseInGeneral: 4, guaranteeArgument: -1 },
      HORIZON,
    ),
  ).toEqual({
    baseCostScale: LEVER_BOUNDS.baseCostScale.max,
    minimumStateShare: LEVER_BOUNDS.minimumStateShare.min,
    phaseInGeneral: LEVER_BOUNDS.phaseInGeneral.max,
    guaranteeArgument: LEVER_BOUNDS.guaranteeArgument.min,
  });
});

test("a value between the ends is returned unchanged, grid or no grid", () => {
  // `1.0395` is the refresh provision `hb-96-with-refreshed-inputs` prices, and the slider cannot
  // express it. A clamp that snapped would make the bill's own link stop opening the bill.
  expect(clampLevers({ baseCostScale: 1.0395 }, HORIZON)).toEqual({ baseCostScale: 1.0395 });
});

test("the horizon is held to the feed's own two ends", () => {
  expect(clampLevers({ horizon: 999_999 }, HORIZON).horizon).toBe(2036);
  expect(clampLevers({ horizon: 1900 }, HORIZON).horizon).toBe(2026);
  expect(clampLevers({ horizon: 2030 }, HORIZON).horizon).toBe(2030);
});

test("a fractional horizon is rounded, not merely bounded", () => {
  /*
   * `forecastPath` walks `baseYear + 1 .. through` by one, and `projectSeries` looks for the year
   * that *equals* `through`. A horizon of 2030.5 is inside the bounds and matches no year, so every
   * district falls through the `if (!projected) continue` and the band comes back empty — a chart
   * with no points rather than a wrong one, which is harder to recognise as a defect.
   */
  expect(clampLevers({ horizon: 2030.5 }, HORIZON).horizon).toBe(2031);
});

test("a panel with no projection collapses the horizon onto the base year", () => {
  // `max` is `panel.projection?.horizon ?? baseYear`. A feed that cannot carry enrollment forward
  // cannot be asked to, and "do not project" is exactly `horizon === base`.
  expect(clampLevers({ horizon: 2036 }, { base: 2026, max: 2026 }).horizon).toBe(2026);
});

test("a value that is not a number is dropped rather than clamped", () => {
  // There is no end of the range `NaN` was reaching for, and clamping it to one would invent an
  // intent. Dropping it lets the default stand — which is what a missing parameter already does.
  expect(clampLevers({ baseCostScale: Number.NaN, horizon: Number.NaN }, HORIZON)).toEqual({});
  expect(clampLevers({ minimumStateShare: Number.POSITIVE_INFINITY }, HORIZON)).toEqual({});
});

test("an absent lever stays absent", () => {
  expect(clampLevers({}, HORIZON)).toEqual({});
});

test("every current-law default is inside its own bounds", () => {
  /*
   * The one direction a bounds table can be wrong in without any URL being involved: a `min` above
   * the value the page opens at would clamp the reset button's own target.
   */
  const defaults = defaultLevers(0.1, HORIZON.base);
  for (const field of Object.keys(LEVER_BOUNDS) as (keyof typeof LEVER_BOUNDS)[]) {
    const { min, max } = LEVER_BOUNDS[field];
    expect(defaults[field], field).toBeGreaterThanOrEqual(min);
    expect(defaults[field], field).toBeLessThanOrEqual(max);
  }
  expect(defaults.horizon).toBeLessThanOrEqual(HORIZON.max);
});
