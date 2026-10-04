/**
 * "Why are so many districts paid an old amount?" (#716).
 *
 * # The identity
 *
 * Line `[I]` of the payment report: `max([H2] − [I1] − [H], 0)`, the base less the open enrollment
 * charge less foundation aid, never below zero. {@link IDENTITY} rebuilds it from the three
 * published columns and checks it against the published guarantee on every district, in the model
 * year, where foundation aid is what the formula computes because the phase-in sits at 100%.
 *
 * # The years on the page
 *
 * The base year is {@link BASE_YEAR}, the phase-in topic's, which `tests/unit/explained.spec.ts`
 * holds to `parameter/guarantee-funding-base`. {@link ORIGIN_YEAR} is where the corpus traces the
 * chain of bases under it, and is held to the same node, which says "bottoms out at" that year.
 * The corpus gives the chain as the worst case, not as every district's history, so the page says
 * the chain "can reach back" and no further.
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
import { BASE_YEAR } from "./phase-in.ts";
import type { Identity, TopicModule } from "./topic.ts";

/** The fiscal year the corpus traces the chain of guarantee bases back to. */
export const ORIGIN_YEAR = 2011;

/** The biennium's last column: the model year, at a 100% phase-in. */
const TERMINAL = 2;

/** Foundation aid in the model year: `[H]`, what the formula computes at a 100% phase-in. */
export const computed = (d: District) => d.biennium.observed[TERMINAL]!.phase_in_paid;

/** The floor the guarantee holds a district at: `[H2] − [I1]`, never below zero. */
export const floorOf = (d: District) =>
  Math.max(d.transition.funding_base - d.transition.open_enrollment_adjustment, 0);

/** What the district receives on the two lines: foundation aid plus the guarantee. */
const received = (d: District) => computed(d) + d.guarantee;

/**
 * `[I] = max([H2] − [I1] − [H], 0)`, rebuilt from its columns.
 *
 * Four amounts each published to the cent; the rebuild lands within a ten-millionth of a dollar on
 * every district, so a cent is room for the publication and nothing else.
 */
export const IDENTITY: Identity<District> = {
  left: (d) => d.guarantee,
  right: (d) => Math.max(floorOf(d) - computed(d), 0),
  tolerance: 0.01,
};

/** How much of what a guaranteed district receives is the guarantee. */
const guaranteedShare = (d: District) => d.guarantee / received(d);

/** The example: the guaranteed district whose guaranteed share is the median, ties broken on IRN. */
export function example(districts: readonly District[]): District {
  const held = districts.filter((d) => d.on_guarantee);
  const middle = median(held.map(guaranteedShare));
  const gap = (d: District) => Math.abs(guaranteedShare(d) - middle);
  return [...held].sort((a, b) => gap(a) - gap(b) || compare(a.irn, b.irn))[0]!;
}

/** Computed against received, per pupil: districts on the formula sit on the line, the rest above. */
function chart(districts: readonly District[], year: string) {
  const per = (v: number, d: District) => v / d.categorical_adm;
  const points = pairs(
    [...districts],
    (d) => per(computed(d), d),
    (d) => per(received(d), d),
    (d, c, r) =>
      `${d.name}: the formula computes ${money(c)} a pupil, the district receives ${money(r)}` +
      (d.on_guarantee ? ", held up by the guarantee" : ""),
    { series: (d) => (d.on_guarantee ? "guarantee" : "formula") },
  );
  const svg = renderToString(
    (w) =>
      scatterSpec(
        points,
        {
          x: { label: "what the formula computes per pupil", format: compactMoney, log: true },
          y: { label: "what the district receives per pupil", format: compactMoney, log: true },
        },
        [],
        { width: w, identity: { label: "receives = computed" } },
      ),
    {
      label: `What each district receives per pupil against what the formula computes per pupil, ${count(points.length)} districts, ${year}, both axes logarithmic`,
      description: "Both axes are dollars per pupil on one shared domain, so the line is receives = computed",
    },
  );
  return {
    svg,
    caption:
      "Each dot is one district. Across is what the formula computes for each pupil, and up is what the district receives. " +
      "Dots on the line are paid on the formula. Dots above it are held up by the guarantee.",
  };
}

export const GUARANTEE: TopicModule = {
  slug: "guarantee",
  group: "Why it moves",
  question: "Why are so many districts paid an old amount?",
  build: () => {
    const districts = loadFeed().bundle.districts;
    const fy = fiscalYear(districts[0]!.biennium.year_terminal);
    const base = fiscalYear(BASE_YEAR);
    const origin = fiscalYear(ORIGIN_YEAR);
    const held = districts.filter((d) => d.on_guarantee);
    const total = held.reduce((s, d) => s + d.guarantee, 0);
    const underHalf = held.filter((d) => computed(d) < floorOf(d) / 2);
    const charged = held.filter((d) => d.transition.open_enrollment_adjustment > 0);
    const ex = example(districts);
    const name = escapeHtml(ex.name);
    const charge = ex.transition.open_enrollment_adjustment;

    return {
      shortAnswer: {
        lead: `A guarantee pays each district at least its ${base} amount.`,
        rest: `In ${fy}, the formula computes less than that for ${count(held.length)} of ${count(districts.length)} districts.`,
      },
      picture:
        "It is like a pay floor in an old work contract. " +
        "However the pay rules change, no one earns less than they did when the contract was signed. " +
        "If the new rules pay less, the employer makes up the gap.",
      breaks: [
        "A contract has the same floor for everyone. Here each district has its own floor: what it was paid in " +
          `${base}. For ${count(held.length)} districts in ${fy}, the formula computes less, so the floor decides what they get.`,
        "A contract floor is reset when the contract is renewed. This one has not been reset in three budgets. " +
          `And the ${base} amount was built on older amounts, in a chain that can reach back to ${origin}.`,
      ],
      numbers: {
        chip: "formula",
        finding: `In ${fy}, the guarantee pays ${name} ${money(ex.guarantee)}, ${pct(guaranteedShare(ex), 0)} of what it receives.`,
        intro: [
          `The example is the middle district when the ${count(held.length)} on the guarantee are ranked by how much of their money it pays. ` +
            `It is ${name}. The numbers are the department's ${fy} calculation.`,
        ],
        steps: [
          charge > 0
            ? `${name}'s ${base} amount is ${money(ex.transition.funding_base)}. Pupils lost to open enrollment take off ${money(charge)}.`
            : `${name}'s base, its ${base} amount, is ${money(ex.transition.funding_base)}.`,
          `For ${fy}, the formula computes ${money(computed(ex))}.`,
          `That is ${money(floorOf(ex) - computed(ex))} short of the base.`,
          `The guarantee pays that gap, so ${name} receives ${money(received(ex))}, its base.`,
        ],
        chart: chart(districts, fy),
        after: [
          `The same sum holds for every district in ${fy}, to the cent. ` +
            `Across the state, the guarantee pays ${compactMoney(total)} to ${count(held.length)} districts.`,
          `For ${count(underHalf.length)} of them, the formula computes less than half the base. ` +
            "Those districts would need the formula to grow a long way before it paid more than the guarantee.",
        ],
      },
      rule: {
        notation:
          "<var>guarantee</var> = max(<var>base</var> − <var>open</var> − <var>computed</var>, zero)",
        statute: "R.C. 3317.019",
        href: "https://codes.ohio.gov/ohio-revised-code/section-3317.019",
        symbols: [
          `<var>base</var> is the district's ${base} amount, column [H2] of the department's payment report.`,
          `<var>open</var> is a charge for pupils lost to open enrollment past a limit. In ${fy}, ${count(charged.length)} guaranteed districts pay it.`,
          "<var>computed</var> is what the formula computes for the district this year.",
          "The guarantee is never below zero, so a district the formula pays more gets nothing from it.",
        ],
      },
      howWeKnow: [
        {
          claim: `[verified] Where the formula computes less than the base, the state pays the base instead; in ${fy} that is ${compactMoney(total)} to ${count(held.length)} districts.`,
          nodes: ["formula-component/temporary-transitional-aid-guarantee"],
        },
        {
          claim: `[verified] The base is the ${base} amount, published for every district, and it has not been reset since.`,
          nodes: ["parameter/guarantee-funding-base"],
        },
        {
          claim: `[verified] The ${base} amount carried earlier amounts forward, in a chain that ends in ${origin}.`,
          nodes: ["funding-regime/bridge-formula"],
        },
      ],
      readNext: {
        topics: ["phase-in", "what-a-district-gets"],
        data: {
          href: routes.districtFinances(ex.irn),
          label: `${name}'s finances`,
          note: "the example district's state aid, and how much of it the guarantee pays",
        },
      },
    };
  },
};
