/**
 * "How does Ohio count poor students?" (#718).
 *
 * # The identity
 *
 * {@link IDENTITY}: DPIA's count is `min((1 − w) × E + w × C, N)` — the FY2025 economically
 * disadvantaged ADM and the FY2026 directly certified ADM, blended at H.B. 96's weight and capped
 * at the district's enrolled ADM — against the department's published `d1`, on every district.
 * The weight is the scenario model's `DPIA_BLEND`, so the page and the lever cannot disagree.
 *
 * # The meal program, and which years it may not speak for
 *
 * The count the department's definition grew out of is free and reduced-price lunch applications,
 * which the MR-81 published every October. The feed keys each October on the fiscal year it falls
 * in (#705: October 2020 is FY2021). The page names two breaks, both from the feed rather than
 * typed: the first year published as more than one file, when community eligibility schools stopped
 * collecting forms, and the years the feed marks not comparable, when the federal waivers let
 * almost every sponsor stop filing.
 *
 * # What it may not say
 *
 * How the department builds its count today. R.C. 3317.03(B)(21) delegates the definition, and the
 * corpus records which test the department now uses as open. So the page says who sets each count,
 * not what is inside the first one.
 */

import { loadFeed } from "../feed.ts";
import { count, escapeHtml, fixed, pct } from "../format.ts";
import { compare } from "../order.ts";
import { DPIA_BLEND } from "../policy.ts";
import * as routes from "../routes.ts";
import { median } from "../stats.ts";
import type { District, MealProgramYear } from "../types.ts";
import { fiscalYear } from "../yearLabel.ts";
import type { Identity, TopicModule } from "./topic.ts";

/** The blended count before the cap. */
export const blend = (d: District, weight = DPIA_BLEND) =>
  (1 - weight) * d.dpia.economically_disadvantaged_adm + weight * d.dpia.directly_certified_adm;

/** The blend, capped at the district's enrolled ADM. */
export const counted = (d: District) => Math.min(blend(d), d.current_year_adm);

/** The published count against the rebuilt one: `d1` is printed to the hundredth. */
export const IDENTITY: Identity<District> = {
  left: (d) => d.dpia.weighted_adm,
  right: counted,
  tolerance: 0.005,
};

/** The example: the district whose blended share of enrollment is the median, ties broken on IRN. */
export function example(districts: readonly District[]): District {
  const middle = median(districts.map((d) => d.dpia.percentage));
  const gap = (d: District) => Math.abs(d.dpia.percentage - middle);
  return [...districts].sort((a, b) => gap(a) - gap(b) || compare(a.irn, b.irn))[0]!;
}

/** The meal program's two breaks: the first year in several files, and the years not comparable. */
export function breaks(meal: readonly MealProgramYear[]) {
  const split = meal.find((y) => y.streams > 1)!;
  const waived = meal.filter((y) => !y.comparable);
  const after = meal.find((y) => y.fiscal_year > waived.at(-1)!.fiscal_year)!;
  return { split, waived, after };
}

export const POVERTY_COUNT: TopicModule = {
  slug: "poverty-count",
  group: "How the money is split",
  question: "How does Ohio count poor students?",
  build: () => {
    const { districts, meal_program: meal } = loadFeed().bundle;
    const fy = fiscalYear(districts[0]!.biennium.year_terminal);
    const ex = example(districts);
    const name = escapeHtml(ex.name);
    const e = ex.dpia.economically_disadvantaged_adm;
    const c = ex.dpia.directly_certified_adm;
    const first = 1 - DPIA_BLEND;
    const heavy = pct(first, 0);
    const light = pct(DPIA_BLEND, 0);
    const capped = districts.filter((d) => blend(d) > d.current_year_adm);
    const { split, waived, after } = breaks(meal);
    const waivedYears = waived.map((y) => fiscalYear(y.fiscal_year)).join(" and ");
    const allE = districts.reduce((s, d) => s + d.dpia.economically_disadvantaged_adm, 0);
    const allC = districts.reduce((s, d) => s + d.dpia.directly_certified_adm, 0);

    return {
      shortAnswer: {
        lead: "Ohio blends two counts of poor pupils.",
        rest: `One is the department's; the other counts pupils whose families get aid. In ${fy} they weigh ${heavy} and ${light}.`,
      },
      picture:
        "It is like counting a town's hungry from two lists. One is everyone who signed up at the food bank. " +
        "The other is everyone already on the welfare rolls. Neither list is the whole town, so the count leans on both.",
      breaks: [
        "For years the food bank's list was the count: families filed a form for free or reduced-price lunch. " +
          `From ${fiscalYear(split.fiscal_year)}, schools that feed every child free stopped collecting forms, so the list stopped seeing them.`,
        `And in ${waivedYears}, federal waivers let every school feed every child free. ` +
          `Most stopped filing at all: ${count(waived[0]!.sponsors)} lunch sponsors filed in ${fiscalYear(waived[0]!.fiscal_year)}, against ${count(after.sponsors)} in ${fiscalYear(after.fiscal_year)}. Those years count who filed, not who was poor.`,
      ],
      numbers: {
        chip: "formula",
        finding: `In the ${fy} formula, ${name} counts ${fixed(ex.dpia.weighted_adm, 2)} poor pupils, ${pct(ex.dpia.percentage, 1)} of its enrollment.`,
        intro: [
          `The example is ${name}. Its share of poor pupils is the middle one of the ${count(districts.length)} districts in the department's ${fy} calculation.`,
        ],
        steps: [
          `The department counts ${fixed(e, 2)} of ${name}'s pupils as economically disadvantaged. That count is from ${fiscalYear(districts[0]!.biennium.year_terminal - 2)}.`,
          `It counts ${fixed(c, 2)} as directly certified from other aid rolls. That count is from ${fiscalYear(districts[0]!.biennium.year_terminal - 1)}.`,
          `The formula takes ${heavy} of the first and ${light} of the second: ${fixed(ex.dpia.weighted_adm, 2)} pupils.`,
          `That is ${pct(ex.dpia.percentage, 1)} of the ${fixed(ex.current_year_adm, 2)} pupils it enrolls.`,
        ],
        after: [
          `The same blend gives the department's count for every district in ${fy}, but it may not exceed a district's enrollment. ` +
            (capped.length === 1
              ? `It does in one, ${escapeHtml(capped[0]!.name)}, which is counted at its enrollment instead.`
              : `It does in ${count(capped.length)}, which are counted at their enrollment instead.`),
          `The two lists are far apart. Across the state they hold ${count(Math.round(allE))} and ${count(Math.round(allC))} pupils, so the weight between them moves money as well as the count.`,
        ],
      },
      rule: {
        notation:
          "<var>count</var> = min((one − <var>w</var>) × <var>E</var> + <var>w</var> × <var>C</var>, <var>N</var>)",
        statute: "H.B. 96",
        href: routes.wikiNode("legislation", "hb-96-2025"),
        symbols: [
          "<var>E</var> is the pupils the department counts as economically disadvantaged.",
          "<var>C</var> is the pupils directly certified because their family is on another aid roll.",
          `<var>w</var> is the weight on the second count: ${light} in ${fy}.`,
          "<var>N</var> is the district's enrolled pupils. The count may not exceed them.",
        ],
      },
      howWeKnow: [
        {
          claim: "[verified] Disadvantaged pupil impact aid blends the department's economically disadvantaged count with the directly certified count, caps the blend at enrollment, and squares the district's share over the state's.",
          nodes: ["formula-component/fsfp-disadvantaged-pupil-impact-aid"],
        },
        {
          claim: "[verified] The blend's weights are set by H.B. 96 in a table of its own, not by the department and not in the Revised Code.",
          nodes: ["legislation/hb-96-2025"],
        },
      ],
      readNext: {
        topics: ["what-a-district-gets", "state-share"],
        data: {
          href: routes.districtFinances(ex.irn),
          label: `${name}'s finances`,
          note: "the example district's state aid, including what its poor pupils bring",
        },
      },
    };
  },
};
