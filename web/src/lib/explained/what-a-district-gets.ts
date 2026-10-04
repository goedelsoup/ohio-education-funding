/**
 * "How does Ohio decide what a district gets?" (#716).
 *
 * # The measure
 *
 * What the formula computes for a district in the model year — the state share of base cost plus
 * the six categoricals, the department's core foundation funding — before the phase-in and the
 * guarantee decide what is paid. The page says so, and sends the reader to the phase-in topic for
 * the rest, because the same district's payment can be far from this figure.
 *
 * # Two identities, term by term
 *
 * {@link shareIdentity} rebuilds the state share of base cost per pupil from base cost per pupil
 * and local capacity per pupil, with the minimum share as a `max`. {@link sumIdentity} adds the six
 * categoricals, each a published column, to that share and lands on the computed amount. The feed's
 * `categorical_funding` is not used for the second: it is the computed amount less the share, so
 * a check through it would pass by construction.
 */

import { loadFeed } from "../feed.ts";
import { compactMoney, count, escapeHtml, money, pct } from "../format.ts";
import { compare } from "../order.ts";
import { scatterSpec } from "../plot/spec.ts";
import { renderToString } from "../plot/ssr.ts";
import { pairs } from "../relationships.ts";
import * as routes from "../routes.ts";
import { median } from "../stats.ts";
import type { District } from "../types.ts";
import { fiscalYear } from "../yearLabel.ts";
import type { Identity, TopicModule } from "./topic.ts";

/** The biennium's last column: the model year, at a 100% phase-in. */
const TERMINAL = 2;

/** The six categoricals, in the order the payment report lists them, with the names a reader sees. */
export const CATEGORICALS = [
  ["targeted_assistance", "targeted assistance"],
  ["special_education", "special education"],
  ["dpia", "Disadvantaged Pupil Impact Aid"],
  ["english_learners", "English learners"],
  ["gifted", "gifted pupils"],
  ["career_technical", "career-technical education"],
] as const satisfies readonly (readonly [keyof District["categoricals"], string])[];

/** What the formula computes for the district in the model year, before the phase-in. */
export const computed = (d: District) => d.biennium.observed[TERMINAL]!.phase_in_calculated;

/** The six categoricals, summed from their own columns. */
export const categoricals = (d: District) =>
  CATEGORICALS.reduce((s, [key]) => s + d.categoricals[key], 0);

/**
 * Local capacity per pupil: what the community is expected to raise, per pupil.
 *
 * Read from the regime block, where it sits beside the charge-off it replaced. A district without
 * one would make the page's sum unprovable, so it throws rather than skipping the district.
 */
function localCapacity(d: District): number {
  const lc = d.regime?.local_capacity;
  if (lc == null) throw new Error(`${d.name} (${d.irn}) carries no local capacity per pupil`);
  return lc;
}

/** The state's share of base cost per pupil, as the rule computes it, at minimum share `floor`. */
export const statePerPupil = (d: District, floor: number) =>
  Math.max(d.base_cost_per_pupil - localCapacity(d), floor * d.base_cost_per_pupil);

/**
 * The state share of base cost per pupil, rebuilt: `max(B − L, m × B)`, to the cent.
 *
 * Per pupil, over `categorical_adm` — the `[a] Enrolled ADM` the share is paid on, which differs
 * from current-year ADM in Akron by fifty pupils and is the only count that reconciles there. The
 * tolerance is a cent: base cost per pupil is published to the cent, and the department rounds
 * the per-pupil share to the cent before it multiplies, so two half-cents can stack.
 */
export function shareIdentity(floor: number): Identity<District> {
  return {
    left: (d) => d.base_cost_state_share / d.categorical_adm,
    right: (d) => statePerPupil(d, floor),
    tolerance: 0.01,
  };
}

/**
 * The computed amount, rebuilt: the state share of base cost plus the six categoricals.
 *
 * Seven amounts each published to the cent, so half a cent apiece.
 */
export const sumIdentity: Identity<District> = {
  left: computed,
  right: (d) => d.base_cost_state_share + categoricals(d),
  tolerance: 0.035,
};

/** Computed aid per pupil, the axis the example is the median of. */
const perPupil = (d: District) => computed(d) / d.categorical_adm;

/** The example: the district whose computed aid per pupil is the median, ties broken on IRN. */
export function example(districts: readonly District[]): District {
  const middle = median(districts.map(perPupil));
  const gap = (d: District) => Math.abs(perPupil(d) - middle);
  return [...districts].sort((a, b) => gap(a) - gap(b) || compare(a.irn, b.irn))[0]!;
}

/** Local capacity against the state's share, per pupil: the split, district by district. */
function chart(districts: readonly District[], floor: number, year: string) {
  const points = pairs(
    [...districts],
    localCapacity,
    (d) => statePerPupil(d, floor),
    (d, lc, share) =>
      `${d.name}: the district can raise ${money(lc)} a pupil, the state pays ${money(share)} a pupil` +
      (d.at_minimum_state_share ? ", the least it pays" : ""),
  );
  const svg = renderToString(
    (w) =>
      scatterSpec(
        points,
        {
          x: { label: "what the district can raise, per pupil", format: compactMoney, log: true },
          y: { label: "what the state pays, per pupil", format: compactMoney, log: true },
        },
        [],
        { width: w },
      ),
    {
      label: `The state's share of base cost per pupil against local capacity per pupil, ${count(points.length)} districts, ${year}, both axes logarithmic`,
      description: "Each district is one dot. The more a district can raise, the less the state pays, down to a floor.",
    },
  );
  return {
    svg,
    caption:
      "Each dot is one district: across is what it can raise for each pupil, up is what the state pays. " +
      `The dots fall to the right, then stop at the floor of ${pct(floor, 0)}.`,
  };
}

export const WHAT_A_DISTRICT_GETS: TopicModule = {
  slug: "what-a-district-gets",
  group: "How the money is split",
  question: "How does Ohio decide what a district gets?",
  build: () => {
    const feed = loadFeed().bundle;
    const districts = feed.districts;
    const floor = feed.statewide.minimum_state_share;
    const fy = fiscalYear(districts[0]!.biennium.year_terminal);
    const ex = example(districts);
    const name = escapeHtml(ex.name);
    const b = ex.base_cost_per_pupil;
    const lc = localCapacity(ex);
    const share = statePerPupil(ex, floor);
    const extras = categoricals(ex);
    const largest = [...CATEGORICALS].sort((a, c) => ex.categoricals[c[0]] - ex.categoricals[a[0]])[0]!;
    const onFloor = districts.filter((d) => d.at_minimum_state_share);
    const totalShare = districts.reduce((s, d) => s + d.base_cost_state_share, 0);
    const totalExtras = districts.reduce((s, d) => s + categoricals(d), 0);
    const costliest = [...districts].sort((a, c) => c.base_cost_per_pupil - a.base_cost_per_pupil || compare(a.irn, c.irn));

    return {
      shortAnswer: {
        lead: "Ohio prices school in each district, takes off what the district can raise, and pays the rest.",
        rest: "Six amounts for pupils who need more come on top.",
      },
      picture:
        "It is like a group splitting a dinner bill. " +
        "Each person pays what they can afford, and a friend covers the rest. " +
        "The less someone can pay, the more the friend puts in.",
      breaks: [
        "At a dinner there is one bill. Here every district has its own. " +
          `The formula prices the staff each district needs, so one pupil costs from ${money(costliest.at(-1)!.base_cost_per_pupil)} to ${money(costliest[0]!.base_cost_per_pupil)} in ${fy}. ` +
          `The top is ${escapeHtml(costliest[0]!.name)}, with ${count(Math.round(costliest[0]!.categorical_adm))} pupils.`,
        `The friend also always pays something. The state pays at least ${pct(floor, 0)} of the cost, however much a district can raise. ` +
          `For ${count(onFloor.length)} districts in ${fy}, that floor decides the share, and what they can raise no longer matters.`,
      ],
      numbers: {
        chip: "formula",
        finding: `In ${fy}, the formula computes ${money(computed(ex))} for ${name}: the state's share of base cost, ${money(ex.base_cost_state_share)}, and ${money(extras)} for six needs.`,
        intro: [
          `The example is the middle district when all ${count(districts.length)} are ranked by what the formula computes for each pupil. ` +
            `It is ${name}. The numbers are the department's ${fy} calculation.`,
        ],
        steps: [
          `The formula says one pupil in ${name} costs ${money(b, 2)} to teach. That is the base cost.`,
          `The district can raise ${money(lc, 2)} a pupil. That is its local capacity.`,
          ex.at_minimum_state_share
            ? `The difference is less than the ${pct(floor, 0)} floor, so the state pays the floor: ${money(share, 2)} a pupil.`
            : `The state pays the difference, ${money(share, 2)} a pupil. That is above the floor of ${pct(floor, 0)}.`,
          `For ${count(Math.round(ex.categorical_adm))} pupils, that comes to ${money(ex.base_cost_state_share)}.`,
          `Six amounts for particular needs add ${money(extras)}. The largest is ${largest[1]}, at ${money(ex.categoricals[largest[0]])}.`,
          `Together they make ${money(computed(ex))}.`,
        ],
        chart: chart(districts, floor, fy),
        after: [
          `The same sum holds for every district in ${fy}, to the cent. Across the state, the six amounts are ${pct(totalExtras / (totalShare + totalExtras), 0)} of what the formula computes.`,
          "What a district is paid can still differ, because the phase-in and the guarantee come next. " +
          (ex.on_guarantee
            ? `${name} is one of those districts. The guarantee holds it at its base, which is more than the formula computes.`
            : `${name} is paid on the formula, so this is what it receives.`),
        ],
      },
      rule: {
        notation:
          "<var>computed</var> = max(<var>B</var> − <var>L</var>, <var>m</var> × <var>B</var>) × <var>N</var> + <var>C</var>",
        statute: "R.C. 3317.022",
        href: "https://codes.ohio.gov/ohio-revised-code/section-3317.022",
        symbols: [
          "<var>B</var> is base cost per pupil.",
          "<var>L</var> is local capacity per pupil, from property values and residents' incomes.",
          `<var>m</var> is the minimum state share. It is ${pct(floor, 0)} in ${fy}.`,
          "<var>N</var> is the district's enrolled pupils this year.",
          "<var>C</var> is the six extra amounts added together.",
        ],
      },
      howWeKnow: [
        {
          claim:
            "[verified] Base cost is built for each district from staffing ratios priced at statewide average salaries.",
          nodes: ["formula-component/fsfp-base-cost-calculation"],
        },
        {
          claim:
            "[verified] Local capacity blends property valuation and two income measures.",
          nodes: ["formula-component/fsfp-local-capacity-measure"],
        },
        {
          claim: `[verified] The state share of base cost is base cost less local capacity, never below ${pct(floor, 0)} of base cost in ${fy}.`,
          nodes: ["metric/state-share-percentage"],
        },
      ],
      readNext: {
        topics: ["phase-in"],
        data: {
          href: routes.district(ex.irn),
          label: `${name}'s page`,
          note: "the example district's base cost, local capacity and the six amounts",
        },
      },
    };
  },
};
