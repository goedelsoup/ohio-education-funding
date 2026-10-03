/**
 * "Why is 2011 still inside the formula?" (#718).
 *
 * # What the page walks
 *
 * The anchor chain `funding-regime/bridge-formula` sets out: each budget re-read the guarantee's
 * base as the year before it began, and each base carried the previous guarantee inside it, so a
 * district on the guarantee throughout still carries the first. {@link CHAIN} is that table, one
 * link per act, and `tests/unit/explained.spec.ts` holds every link to the node's own lines, so the
 * page and the corpus cannot disagree about a year or an act.
 *
 * # The numbers
 *
 * How many districts the base holds in the model year, computed from the published guarantee, and
 * how far prices have moved since the year the base names, from the feed's deflator. The base has
 * no escalator, so that is what a floor written in dollars loses by standing still.
 *
 * # What it may not say
 *
 * That any one district is still paid on its {@link ORIGIN_YEAR} figure. The corpus gives the
 * chain as the worst case and records how many districts are still on it as open: a district that
 * cleared its floor in a later formula run was re-based on that run. So the page says the floor
 * "can trace back", names no district, and says in its breaks why not.
 */

import { loadFeed } from "../feed.ts";
import { count, pct } from "../format.ts";
import { NAMES } from "../nav.ts";
import { lastOf } from "../ends.ts";
import type { Deflator } from "../types.ts";
import { fiscalYear } from "../yearLabel.ts";
import { ORIGIN_YEAR } from "./guarantee.ts";
import { BASE_YEAR } from "./phase-in.ts";
import type { TopicModule } from "./topic.ts";

/**
 * Each act, and the fiscal year it set the floor at: H.B. 153 kept the first year's aid, and every
 * act after it the amount paid the year before its budget began.
 */
export const CHAIN = [
  { year: ORIGIN_YEAR, act: "H.B. 153" },
  { year: 2013, act: "H.B. 59" },
  { year: 2015, act: "H.B. 64" },
  { year: 2017, act: "H.B. 49" },
  { year: 2019, act: "H.B. 166" },
  { year: BASE_YEAR, act: "H.B. 110" },
] as const;

/** H.B. 49's one exception: the floor fell to this share for districts that lost this share of pupils. */
export const EMPTYING_FLOOR = 0.95;
export const EMPTYING_LOSS = 0.1;

/** How far prices moved from the base year to the latest year the deflator covers. */
export function inflation(deflator: Deflator) {
  const at = (y: number) => deflator.points.find((p) => p.fiscal_year === y)!.index;
  const last = lastOf(deflator.points).fiscal_year;
  return { last, rise: at(last) / at(BASE_YEAR) - 1 };
}

export const ANCHOR_CHAIN: TopicModule = {
  slug: "anchor-chain",
  group: "Why it moves",
  question: `Why is ${fiscalYear(ORIGIN_YEAR)} still inside the formula?`,
  build: () => {
    const { districts, deflator } = loadFeed().bundle;
    const terminal = districts[0]!.biennium.year_terminal;
    const fy = fiscalYear(terminal);
    const [first, , , , frozen, base] = CHAIN;
    const y = (link: { year: number }) => fiscalYear(link.year);
    const held = districts.filter((d) => d.guarantee > 0).length;
    // The feed carries the deflator whenever it carries the panel this page reads; a build without
    // it should fail here rather than print a price rise of nothing.
    const { last, rise } = inflation(deflator!);

    return {
      shortAnswer: {
        lead: "Because each budget copied its floor from the one before.",
        rest: `In ${fy}, the floor that paid ${pct(held / districts.length, 0)} of districts can trace back to ${y(first)}.`,
      },
      picture:
        "It is like a photocopy of a photocopy. Each copy is made from the one before, never from the original. " +
        "Long after the original is gone, its words are still there.",
      breaks: [
        "A photocopy fades a little with each copy. This one keeps every digit of its dollar figure, because the floor is never raised. " +
          `The dollar is what fades: prices rose ${pct(rise, 0)} from ${y(base)} to ${fiscalYear(last)}, so it buys less every year.`,
        "And not every district's copy goes back to the first page. A district whose formula paid more than its floor started a fresh copy that year. " +
          "The chain is the longest a floor can be, not every district's.",
      ],
      numbers: {
        chip: "formula",
        finding: `In ${fy}, ${count(held)} of ${count(districts.length)} districts were paid their ${y(base)} floor, a figure that can trace back to ${y(first)}.`,
        intro: [
          "Each step below is one budget setting the floor. Each floor included the top-up paid under the one before, so a district on it every time carries the first copy the whole way.",
        ],
        steps: [
          `In ${y(first)}, the state paid each district under a formula called the Evidence-Based Model.`,
          `${first.act} repealed it, but kept each district's ${y(first)} aid as its floor, without the federal stimulus money.`,
          `The next three budgets each set the floor at what a district got in ${y(CHAIN[1])}, ${y(CHAIN[2])}, then ${y(CHAIN[3])}.`,
          `For ${y(base)}, ${frozen.act} ran no formula at all. Every district got its ${y(frozen)} amount again.`,
          `${base.act}, the plan in force now, took that ${y(base)} amount as its floor. It has never been reset.`,
        ],
        after: [
          `That is ${count(CHAIN.length)} copies across ${count(terminal - first.year)} years. ` +
            "Even the measure of property wealth in the first copy, its charge-off valuation, belonged to the formula that was repealed.",
          `Only one of those budgets treated districts differently: ${CHAIN[3].act} set the floor at ${pct(EMPTYING_FLOOR, 0)} for districts that had lost ${pct(EMPTYING_LOSS, 0)} of their pupils. ` +
            "Every other copy changed the date, not the rule.",
        ],
      },
      rule: {
        notation:
          "<var>paid</var><sub><var>t</var></sub> = max(<var>formula</var><sub><var>t</var></sub>, <var>paid</var><sub><var>b</var></sub>)",
        statute: "R.C. 3317.02",
        href: "https://codes.ohio.gov/ohio-revised-code/section-3317.02",
        symbols: [
          "<var>t</var> is a year, and <var>b</var> is the year before its budget began.",
          "<var>formula</var> is what that year's formula computes for the district.",
          "<var>paid</var><sub><var>b</var></sub> already holds the floor before it, so the first floor rides along inside it.",
          `Since ${y(base)}, <var>b</var> has not moved: R.C. 3317.02 names that year's amount as the base.`,
        ],
      },
      howWeKnow: [
        {
          claim: "[verified] Each budget from H.B. 59 to H.B. 166 re-read the guarantee's anchor year and left the mechanism alone, and each base included the previous budget's guarantee, so a district guaranteed throughout carries its earliest anchor forward.",
          nodes: ["funding-regime/bridge-formula"],
        },
        {
          claim: `[verified] H.B. 153 built each district's base from its ${y(first)} state aid per pupil, with a floor at ${y(first)} aid net of federal stimulus, and repealed the Evidence-Based Model that had computed it.`,
          nodes: ["legislation/hb-153-2011", "funding-regime/evidence-based-model"],
        },
        {
          claim: `[verified] Column [H2] is the ${y(base)} amount, which H.B. 166 set at each district's ${y(frozen)} allocation unchanged, and it has never been re-based.`,
          nodes: ["parameter/guarantee-funding-base"],
        },
      ],
      readNext: {
        topics: ["guarantee", "phase-in"],
        data: {
          href: NAMES.legislation.href,
          label: NAMES.legislation.name,
          note: "every act in the chain, in the order it was signed",
        },
      },
    };
  },
};
