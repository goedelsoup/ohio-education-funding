/**
 * "Why don't my taxes rise when my house value does?" (#718).
 *
 * # The identity
 *
 * {@link IDENTITY}: the effective Class I rate is the voted rate less the share H.B. 920's
 * reduction factors have taken, `effective = voted × (1 − r)`, on every district carrying all
 * three. It is not true by construction. The effective rate is the District Profile Report's; the
 * share `r` is the feed's `millage.cumulative_reduction`, which `crates/bundle` computes from
 * Table SD-1's rate for the same tax year. So the identity holds the two departments' rates
 * together through the voted one, as the feed's schema says they agree.
 *
 * # What it may not say
 *
 * That a district's levy never grows. Reduction factors reach neither newly voted millage nor new
 * construction, and stop at the floor — the feed's `millage.residual` is not zero above the floor,
 * and a page claiming the rate follows from the factors alone would be wrong for most of the state.
 * So the worked example is the identity over the levy's whole life, not a one-year prediction.
 */

import { loadFeed } from "../feed.ts";
import { count, escapeHtml, fixed, pct } from "../format.ts";
import { compare } from "../order.ts";
import * as routes from "../routes.ts";
import { median } from "../stats.ts";
import type { District } from "../types.ts";
import { yearOf } from "../year.ts";
import type { Identity, TopicModule } from "./topic.ts";

/**
 * The statutory floor for a city, local or exempted village district, in mills —
 * `millage::SCHOOL_DISTRICT_FLOOR`, held to the corpus node by test.
 */
export const FLOOR = 20;

/** A district with a voted rate, an effective rate and the share between them. */
export type Levy = District & {
  voted_operating_millage: number;
  effective_class1_millage: number;
  millage: NonNullable<District["millage"]> & { cumulative_reduction: number };
};

/** Every district carrying all three figures. */
export const rows = (districts: readonly District[]): Levy[] =>
  districts.filter(
    (d): d is Levy =>
      d.voted_operating_millage != null &&
      d.effective_class1_millage != null &&
      d.millage?.cumulative_reduction != null,
  );

/**
 * `effective = voted × (1 − r)`.
 *
 * Table SD-1 prints its rate to the hundredth, which is half a hundredth of room, and the feed prints
 * `r` to four places, which is half a ten-thousandth of the voted rate: under a hundredth of a mill
 * at the highest voted rate in the state. Fifteen thousandths covers both.
 */
export const IDENTITY: Identity<Levy> = {
  left: (d) => d.effective_class1_millage,
  right: (d) => d.voted_operating_millage * (1 - d.millage.cumulative_reduction),
  tolerance: 0.015,
};

/** Above the floor in the year the reduction is measured, so the factors are still cutting. */
const aboveFloor = (d: Levy) => d.millage.prior_rate > FLOOR + 0.005;

/**
 * The example: of the districts above the floor, the one whose reduction is the median, ties broken
 * on IRN.
 *
 * Above the floor because the median district overall sits on it, where the factors have stopped,
 * and an example of a mechanism should be one where it is running.
 */
export function example(districts: readonly District[]): Levy {
  const above = rows(districts).filter(aboveFloor);
  const middle = median(above.map((d) => d.millage.cumulative_reduction));
  const gap = (d: Levy) => Math.abs(d.millage.cumulative_reduction - middle);
  return [...above].sort((a, b) => gap(a) - gap(b) || compare(a.irn, b.irn))[0]!;
}

export const REDUCTION_FACTORS: TopicModule = {
  slug: "reduction-factors",
  group: "How the money is split",
  question: "Why don't my taxes rise when my house value does?",
  build: () => {
    const { districts, statewide } = loadFeed().bundle;
    const all = rows(districts);
    const ex = example(districts);
    const name = escapeHtml(ex.name);
    const measured = `${ex.millage.tax_year - 1} tax year`;
    const latest = `${yearOf("millage")} tax year`;
    const voted = fixed(ex.voted_operating_millage, 2);
    const effective = fixed(ex.effective_class1_millage, 2);
    // To the hundredth of a per cent, so the last step checks: at whole per cents it is a mill out.
    const taken = pct(ex.millage.cumulative_reduction, 2);
    const above = all.filter(aboveFloor).length;

    return {
      shortAnswer: {
        lead: "In most of Ohio, higher home values do not raise the school levy.",
        rest: `The rate is cut; in the ${measured} the median district's was ${pct(statewide.median_millage_reduction, 0)} under what voters passed.`,
      },
      picture:
        "It is like splitting a restaurant bill that stays the same. " +
        "If everyone at the table gets a raise, each one pays a smaller slice of their pay, because the bill has not grown.",
      breaks: [
        "A bill that never grows is too simple. The cuts do not reach a new levy or the tax on a new building, so a district's take can still rise.",
        `And the slices stop shrinking at a floor. The cuts cannot push a district's operating rate below ${FLOOR} mills. ` +
          `In the ${latest}, ${count(statewide.at_millage_floor)} districts sat on it, and for them a rise in value does raise the tax.`,
      ],
      numbers: {
        chip: "profile",
        finding: `In the ${measured}, ${name}'s voters had approved ${voted} mills of operating levy, and it charged ${effective} on homes and farms.`,
        intro: [
          `The example is ${name}. Of the ${count(above)} districts above the floor, it is the middle one by how much of the voted rate the cuts have taken.`,
        ],
        steps: [
          `Voters in ${name} approved ${voted} mills. A mill is a dollar of tax on each thousand dollars of taxable value.`,
          "Each time homes were valued higher, the state cut the rate. Each levy kept raising about what it raised when passed.",
          `By the ${measured}, the cuts had taken ${taken} of it.`,
          `So homes and farms paid ${effective} mills: the ${voted} with ${taken} taken off.`,
        ],
        after: [
          `The same rule gives the department's rate for every one of the ${count(all.length)} districts it reports, to the hundredth of a mill.`,
          "Above the floor, the take grows only with a new levy, with new buildings, or on reaching the floor. So a district above it whose costs rise has to go back to its voters.",
        ],
      },
      rule: {
        notation: "<var>effective</var> = <var>voted</var> × (one − <var>r</var>)",
        statute: "R.C. 319.301",
        href: "https://codes.ohio.gov/ohio-revised-code/section-319.301",
        symbols: [
          "<var>voted</var> is the operating rate voters approved, in mills.",
          "<var>r</var> is the share of it the cuts have taken, recomputed each year values rise.",
          `<var>r</var> stops growing when <var>effective</var> reaches ${FLOOR} mills, or never starts if voters approved less.`,
          "<var>effective</var> is the rate charged on homes and farms. Other property has its own.",
        ],
      },
      howWeKnow: [
        {
          claim: "[verified] H.B. 920's reduction factors cut the rate of a voted levy as reappraisal raises the value of property already there, so it raises about the dollars it raised when passed; growth comes only from new construction, new levies, or the floor.",
          nodes: ["legislation/hb-920-1976"],
        },
        {
          claim: "[verified] The factors cannot push a district's effective operating rate below twenty mills, and at that floor a rise in value reaches revenue directly.",
          nodes: ["parameter/twenty-mill-floor"],
        },
        {
          claim: "[verified] Effective millage is reported separately for homes and farms and for other property, because the factors are computed per class.",
          nodes: ["metric/effective-operating-millage"],
        },
      ],
      readNext: {
        topics: ["sources", "state-share"],
        data: {
          href: routes.districtTaxes(ex.irn),
          label: `${name}'s taxes`,
          note: "the example district's voted and effective rates, and what the factors alone predict",
        },
      },
    };
  },
};
