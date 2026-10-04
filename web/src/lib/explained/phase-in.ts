/**
 * "What does the phase-in do?" — the pilot topic (#715).
 *
 * # Why the example is a mid-phase year
 *
 * R.C. 3317.022 pays `base + p × (computed − base)`. At `p = 1` that is `computed` whatever the
 * base is, so the FY2027 panel cannot tell the interpolation from a multiplier: both readings pay
 * the same figure. The worked example and the reconciliation therefore use the biennium's middle
 * year, the department's FY2026 calculation, where the dial sits short of the end and the two
 * readings part by most of a district's base.
 *
 * # Where `p` comes from
 *
 * The feed carries the model year's policy, which is the FY2027 run at 100%. The FY2026 percentage
 * is `project/fy2026-general-phase-in` in `crates/figures.json`, solved by `crates/project` from
 * each district's own columns and identical on every district wide enough to divide. The figure's
 * key names its year, and {@link fy2026PhaseIn} refuses to run if that is not the year the
 * biennium's middle column holds, so a re-vendored feed with new years fails the build rather than
 * pairing one year's dial with another's amounts.
 *
 * # One `p` for two terms
 *
 * The statute writes two bracketed terms — the general components against the general base, and
 * DPIA against its own — each with its own percentage. The feed's biennium columns are the two
 * summed, so a single `p` reconciles them only if the two percentages are equal. That is what
 * {@link IDENTITY} asserts on every district, and it is the corpus's own claim (greenbook Table 1's
 * footnote: DPIA matches the general percentage from FY2023).
 */

import { MEASURES } from "../change.ts";
import { loadFigureManifest } from "../corpusFigures.ts";
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

/**
 * The fiscal year the base was fixed in: column `[H2]` of the payment report.
 *
 * Not in the feed, which carries the base's value and not its vintage. A constant, named, and
 * checked by `tests/unit/explained.spec.ts` against `parameter/guarantee-funding-base`, which says
 * the year in its summary — so the year on this page is the corpus's, not this file's.
 */
export const BASE_YEAR = 2020;

/** The figure that holds the mid-phase percentage, and the year its key names. */
const PHASE_IN_FIGURE = "project/fy2026-general-phase-in";

/** The biennium's middle column: the department's calculation for the mid-phase year. */
const MID = 1;

/** The phase-in percentage for the biennium's middle year, from the figure register. */
export function fy2026PhaseIn(districts: readonly District[]): { year: number; p: number } {
  const figure = loadFigureManifest().figures.find((f) => f.key === PHASE_IN_FIGURE);
  if (!figure) throw new Error(`crates/figures.json carries no ${PHASE_IN_FIGURE}`);
  const named = Number(/fy(\d{4})/.exec(PHASE_IN_FIGURE)![1]);
  const year = districts[0]!.biennium.year_middle;
  if (named !== year) {
    throw new Error(
      `${PHASE_IN_FIGURE} is a percentage for FY${named}, and the biennium's middle column is ` +
        `FY${year}. Pairing them would interpolate one year's amounts with another year's dial.`,
    );
  }
  return { year, p: figure.value };
}

/** One district's three amounts in the mid-phase year. */
function ends(d: District) {
  const o = d.biennium.observed[MID]!;
  return { base: o.funding_base, computed: o.phase_in_calculated, paid: o.phase_in_paid };
}

/** The step the phase-in weighs: computed less base, negative where the formula computes less. */
const step = (d: District) => ends(d).computed - ends(d).base;

/**
 * `paid = base + p × (computed − base)`, to the cent the department pays on, given the year's `p`.
 *
 * `tests/unit/explained.spec.ts` asserts it on every district and pairs it with a doctored row
 * that pays `p × computed`, the multiplier reading this page exists to correct.
 */
export function identity(p: number): Identity<District> {
  return {
    left: (d) => ends(d).paid,
    right: (d) => ends(d).base + p * step(d),
    tolerance: 0.01,
  };
}

/**
 * The example: the district whose step is the median step, ties broken on IRN.
 *
 * Nearest rather than equal, so that a feed with an even count — where `median` averages the two
 * middle steps and no district has that step — still names one district.
 */
export function example(districts: readonly District[]): District {
  const middle = median(districts.map(step));
  const gap = (d: District) => Math.abs(step(d) - middle);
  return [...districts].sort((a, b) => gap(a) - gap(b) || compare(a.irn, b.irn))[0]!;
}

/** Held at its base: foundation aid equal to the base, to the dollar. */
const atBase = (d: District) =>
  Math.abs(MEASURES.foundation.levels(d.biennium)[MID] - ends(d).base) < 1;

/** The districts on each side of their base, and what the guarantee pays the ones below it. */
export function sides(districts: readonly District[]) {
  const below = districts.filter((d) => step(d) < 0);
  return {
    above: districts.filter((d) => step(d) > 0),
    below,
    held: below.filter(atBase),
    restored: below.reduce((s, d) => s + ends(d).base - ends(d).paid, 0),
  };
}

/** A list of names in prose: "A", "A and B", "A, B and C". */
function names(list: readonly string[]): string {
  if (list.length < 2) return list.join("");
  return `${list.slice(0, -1).join(", ")} and ${list.at(-1)}`;
}

/** Base against computed, per pupil, with y = x: above the line moves up, below moves down. */
function chart(districts: readonly District[], year: string) {
  const perPupil = (v: number, d: District) => v / d.biennium.observed[MID]!.enrolled_adm!;
  const drawn = districts.filter((d) => ends(d).base > 0 && ends(d).computed > 0 && d.biennium.observed[MID]!.enrolled_adm);
  const left = districts.filter((d) => !drawn.includes(d));
  const points = pairs(
    drawn,
    (d) => perPupil(ends(d).base, d),
    (d) => perPupil(ends(d).computed, d),
    (d, base, computed) =>
      `${d.name}: base ${money(base)} a pupil, the formula computes ${money(computed)}` +
      (step(d) < 0 ? ", moved down and held at the base" : ", moved up"),
    { series: (d) => (step(d) < 0 ? "guarantee" : "formula") },
  );
  const svg = renderToString(
    (w) =>
      scatterSpec(
        points,
        {
          x: { label: "base per pupil", format: compactMoney, log: true },
          y: { label: "what the formula computes per pupil", format: compactMoney, log: true },
        },
        [],
        { width: w, identity: { label: "computed = base" } },
      ),
    {
      label: `What the formula computes per pupil against each district's base per pupil, ${count(points.length)} districts, ${year}, both axes logarithmic`,
      description: "Both axes are dollars per pupil on one shared domain, so the line is computed = base",
    },
  );
  const off = left.length
    ? ` ${names(left.map((d) => escapeHtml(d.name)))} ${left.length === 1 ? "has" : "have"} a base below zero and cannot sit on this scale.`
    : "";
  return {
    svg,
    caption:
      "Each dot is one district. Across is its base for each pupil, and up is what the formula computes for each pupil. " +
      `Dots above the line move up. Dots below it move down.${off}`,
  };
}

export const PHASE_IN: TopicModule = {
  slug: "phase-in",
  group: "Why it moves",
  question: "What does the phase-in do?",
  build: () => {
    const districts = loadFeed().bundle.districts;
    const { year, p } = fy2026PhaseIn(districts);
    const fy = fiscalYear(year);
    const base = fiscalYear(BASE_YEAR);
    const share = pct(p, 2);
    const ex = example(districts);
    const e = ends(ex);
    const name = escapeHtml(ex.name);
    const s = sides(districts);
    const all = s.held.length === s.below.length ? `all ${count(s.held.length)}` : count(s.held.length);

    return {
      shortAnswer: {
        lead: `The phase-in moves each district from its ${base} base toward what the formula computes.`,
        rest: `In ${fy} it goes ${share} of the way, not ${share} of the new amount.`,
      },
      picture:
        "It is like a dial turned from an old setting toward a new one. " +
        "Each year the state turns it one more notch. " +
        "Halfway means halfway between the two settings, not half of the new one.",
      breaks: [
        "A real dial has one old setting and one new one. " +
          "Here each district has its own two ends, and they can point either way. " +
          `For ${count(s.above.length)} districts in ${fy}, the formula computes more than the base, so the phase-in moves them up.`,
        `For the other ${count(s.below.length)}, the formula computes less than the base, so the phase-in moves them down. ` +
          `The guarantee then stops the fall. In ${fy} it holds ${all} of them at their base, and pays ${compactMoney(s.restored)} to do it.`,
      ],
      numbers: {
        chip: "biennium",
        finding: `In ${fy}, ${name} is paid ${money(e.paid)}: ${share} of the way from its base to what the formula computes.`,
        intro: [
          `The example is the middle district when all ${count(districts.length)} are ranked by their step. ` +
            `It is ${name}. The numbers are the department's ${fy} calculation, when the dial sat at ${share}.`,
        ],
        steps: [
          `${name}'s base, its ${base} amount, is ${money(e.base)}.`,
          `For ${fy}, the formula computes ${money(e.computed)}.`,
          `The step from the base to the computed amount is ${money(step(ex))}.`,
          `The phase-in pays ${share} of that step, which is ${money(p * step(ex))}.`,
          `The base plus that part is ${money(e.paid)}. That is what the department pays.`,
        ],
        chart: chart(districts, fy),
        after: [
          `If ${share} were a share of the new amount, ${name} would get ${money(p * e.computed)}. ` +
            `That is ${money(e.paid - p * e.computed)} less than the department pays.`,
          `The same sum holds for every district in ${fy}, to the cent. The rule below is how the law writes it.`,
        ],
      },
      rule: {
        notation:
          "<var>paid</var> = <var>base</var> + <var>p</var> × (<var>computed</var> − <var>base</var>)",
        statute: "R.C. 3317.022",
        href: "https://codes.ohio.gov/ohio-revised-code/section-3317.022",
        symbols: [
          `<var>base</var> is the district's ${base} amount, column [H2] of the department's payment report.`,
          "<var>computed</var> is what the formula computes for the district this year.",
          `<var>p</var> is the phase-in. It was ${share} in ${fy}.`,
          "The law writes two terms, one for Disadvantaged Pupil Impact Aid and one for the rest, each with its own <var>p</var>. " +
            `In ${fy} the two are equal, so one <var>p</var> covers both, and the sum above checks it on every district.`,
        ],
      },
      howWeKnow: [
        {
          claim:
            `[verified] The phase-in percentage is the weight on the distance from the ${base} base to the computed amount, ` +
            `not a multiplier on the computed amount; in ${fy} it is ${share} on every district.`,
          nodes: ["parameter/fsfp-phase-in-percentage"],
          figures: [PHASE_IN_FIGURE],
        },
        {
          claim: "[verified] The base is column [H2], and the guarantee holds a district at the same column.",
          nodes: ["parameter/guarantee-funding-base"],
        },
        {
          claim: "[verified] The computed amount is the formula's base cost with the components built on it.",
          nodes: ["formula-component/fsfp-base-cost-calculation"],
        },
      ],
      readNext: {
        topics: ["guarantee"],
        data: {
          href: routes.districtChange(ex.irn),
          label: `${name}'s Change tab`,
          note: "the example district's phase-in, year by year",
        },
      },
    };
  },
};
