/**
 * "How sure is a projection?" (#718).
 *
 * # The identity
 *
 * {@link IDENTITY}: the band's sigma, as the feed states it, against the standard deviation of the
 * districts' annual growth rates recomputed here by `growthPrior` — the same recomputation every
 * district fan runs before it draws. One row, because the band has one sigma: it is the spread of
 * districts against one another, not any one district's own, and that is the page's first break.
 *
 * # The numbers
 *
 * The example is the median district by enrollment, carried forward by `projectSeries` under the
 * feed's own method to the depth a district's fan is drawn, {@link DISTRICT_FAN_YEARS}. The check
 * on the band is `coverage()` and the check on its centre `bias()`, both read off
 * `crates/series.json`, so the coverage figure stays computed as #718 requires: a panel that grows
 * an origin moves every one of them without an edit here.
 *
 * # What it may not say
 *
 * That the band is the district's uncertainty, or that a forecast inside it is right. The band
 * prices how far districts differ, and a statewide level break — the school closures — is not in
 * it at any width. So the page states the coverage beside the bias and corrects neither, as
 * `.yidam/decisions/the-bias-published-beside-the-point.yml` decides.
 */

import { bias } from "../bias.ts";
import { coverage } from "../coverage.ts";
import { loadFeed } from "../feed.ts";
import { count, escapeHtml, fixed, pct } from "../format.ts";
import { NAMES } from "../nav.ts";
import { compare } from "../order.ts";
import { DISTRICT_FAN_YEARS, HORIZON_EXPONENT, growthPrior, observations, projectSeries, spread } from "../project.ts";
import { median } from "../stats.ts";
import type { District } from "../types.ts";
import { yearOf } from "../year.ts";
import { fiscalYear } from "../yearLabel.ts";
import type { Identity, TopicModule } from "./topic.ts";

/** The band's one sigma, as the feed states it, beside the panel it is computed from. */
export interface Spread {
  irn: string;
  stated: number;
  districts: readonly District[];
}

/** The feed prints sigma to six places, which is half a millionth of room. */
export const IDENTITY: Identity<Spread> = {
  left: (s) => s.stated,
  right: (s) => growthPrior([...s.districts], 1).sigma,
  tolerance: 5e-7,
};

/** The example: the district whose latest enrollment is the median, ties broken on IRN. */
export function example(districts: readonly District[]): District {
  const latest = (d: District) => d.adm_history.at(-1)!;
  const middle = median(districts.map(latest));
  const gap = (d: District) => Math.abs(latest(d) - middle);
  return [...districts].sort((a, b) => gap(a) - gap(b) || compare(a.irn, b.irn))[0]!;
}

export const PROJECTION: TopicModule = {
  slug: "projection",
  group: "What it does and does not show",
  question: "How sure is a projection?",
  build: () => {
    const { districts, projection } = loadFeed().bundle;
    // The feed carries the projection whenever it carries the panel; a build without it should
    // fail here rather than describe a band nobody drew.
    const meta = projection!;
    const prior = growthPrior(districts, meta.z);
    const ex = example(districts);
    const name = escapeHtml(ex.name);
    const through = meta.base_year + DISTRICT_FAN_YEARS;
    const path = projectSeries(
      observations(ex, meta.base_year),
      through,
      meta.method,
      meta.damping,
      prior,
      meta.shrink_weight,
      ex.long_run_enrollment_rate ?? null,
    );
    const end = path.at(-1)!;
    const half = Math.expm1(spread(prior, DISTRICT_FAN_YEARS));
    const one = Math.expm1(spread(prior, 1));
    const c = coverage();
    const [before, across] = bias().populations;
    const pupils = (v: number) => count(Math.round(v));
    // A log error as the share it amounts to, unsigned: the sentence says "high".
    const high = (v: number) => pct(Math.expm1(v), 1);
    const off = fixed(c.crossDistrict.worst.gap * 100, 1);
    const now = fiscalYear(meta.base_year);
    const later = fiscalYear(through);
    // The backtest scores forecasts against the Census panel, so its years are that panel's.
    const tested = yearOf("history");

    return {
      shortAnswer: {
        lead: "The range is about right. The middle runs high.",
        rest: `Tested over ${tested}, it held ${pct(c.crossDistrict.first, 1)} of forecasts at one year and ${pct(c.crossDistrict.last, 1)} at ${c.deepest} years, against ${pct(c.reference.value, 1)}.`,
      },
      picture:
        "It is like the cone on a hurricane map. Near the storm the cone is narrow. " +
        "Days out it is wide, because a small error in the path grows with every mile.",
      breaks: [
        "A hurricane cone is drawn from that storm's own track. A district has only three years of enrollment, too few for that. " +
          "So the band is built from the spread between districts instead. A district whose enrollment swings more has a wider range than this.",
        "And a cone says where the storm might go, not that its middle line is right. " +
          `Here the middle runs high. In tests over ${tested}, the forecast for the average district was ${high(across!.meanDistrict.last)} too high at ${across!.deepest} years. ` +
          `Before the school closures, it was ${high(before!.meanDistrict.last)} at ${before!.deepest} years.`,
      ],
      numbers: {
        chip: "enrollment",
        finding: `${name} enrolls ${pupils(ex.adm_history.at(-1)!)} pupils in ${now}. For ${later} the site projects ${pupils(end.point)}, in a range from ${pupils(end.low)} to ${pupils(end.high)}.`,
        intro: [
          `The example is ${name}, the middle district by enrollment of the ${count(districts.length)}.`,
        ],
        steps: [
          `The site starts from the last ${count(ex.adm_history.length)} years of enrollment at ${name}: ${pupils(ex.adm_history.at(-1)!)} pupils in ${now}.`,
          `Its trend gives ${pupils(end.point)} pupils in ${later}. That is the middle line.`,
          `Districts' yearly growth differs by about ${pct(prior.sigma, 1)}. One year out, the range is ${pct(one, 1)} each side.`,
          `It widens more slowly than the years. ${count(DISTRICT_FAN_YEARS)} years out it is ${pct(half, 1)} each side.`,
          `So ${name}'s range for ${later} runs from ${pupils(end.low)} to ${pupils(end.high)} pupils.`,
        ],
        after: [
          `A range of one spread on each side should hold about ${pct(c.reference.value, 1)} of what happens. ` +
            `Tested on every past year it could be, it held ${pct(c.crossDistrict.first, 1)} one year out and ${pct(c.crossDistrict.last, 1)} at ${c.deepest} years, never more than ${off} points off.`,
          "That test first takes out the error that each year shares. Left in, the closure years pull the middle high, which is the second break above. The site prints that bias beside its forecasts and corrects neither.",
        ],
      },
      rule: {
        notation:
          "<var>low</var>, <var>high</var> = <var>point</var> × e<sup>∓<var>σ</var> <var>h</var><sup><var>k</var></sup></sup>",
        statute: "the site's projection method",
        href: `${NAMES.method.href}#forecast-range`,
        symbols: [
          "<var>point</var> is the district's projected enrollment, from its own trend.",
          `<var>σ</var> is how much districts' yearly growth differs: ${pct(prior.sigma, 1)} over ${yearOf("enrollment")}.`,
          "<var>h</var> is the years ahead.",
          `<var>k</var> is ${HORIZON_EXPONENT}, fitted so the range holds what it claims. Below one, so the range grows more slowly than the years.`,
        ],
      },
      howWeKnow: [
        {
          claim: "[verified] Three observations cannot estimate a variance, so the interval rests on the cross-sectional spread of district growth rates as a floor; a district with volatile enrollment has a wider true interval.",
          nodes: ["scenario/guarantee-phase-out"],
        },
        {
          claim: "[verified] The widening rule was fitted to the band's measured coverage, and with each origin's mean removed the band holds close to what a one-sigma interval claims at every horizon the backtest reaches.",
          nodes: ["scenario/guarantee-phase-out"],
        },
        {
          claim: "[verified] What fails past five years is the center, not the width: the bias is a year effect from the school closures, and neither the center nor the band is corrected.",
          nodes: ["scenario/guarantee-phase-out"],
        },
      ],
      readNext: {
        topics: ["what-a-district-gets", "guarantee"],
        data: {
          href: `${NAMES.method.href}#forecast-range`,
          label: "How the forecast is checked",
          note: "the coverage and the bias at every horizon, drawn",
        },
      },
    };
  },
};
