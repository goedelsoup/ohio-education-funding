/**
 * "Where does school money come from?" (#716).
 *
 * # The measure
 *
 * Total revenue as the Census Bureau's Annual Survey of School System Finances counts it, over the
 * Ohio districts the survey marks comparable — the `history` panel `/history` draws. Neither of the
 * site's two funding measures: the state's part is every state dollar, construction money
 * included, not total state support and not foundation aid. The page says so in its limits.
 *
 * # The identity
 *
 * {@link IDENTITY}: in every year the panel holds, the local, state and federal shares add to one.
 * The feed prints each to four places, so three half-steps is the room rounding has.
 *
 * # What it may not say
 *
 * The survey's state column changes basis at {@link BREAK_YEAR}: from that year the Bureau nets the
 * community-school deduct out of it, and out of total revenue with it, so the local share jumps
 * by bookkeeping. `dispersion::survey_basis` holds the correction and the feed does not carry it.
 * So the page makes no claim about the state or local share across that year — only the latest
 * year's split, that local money is the largest part in every year, and the federal share, whose
 * relief peaks dwarf anything the basis moves.
 */

import { loadFeed } from "../feed.ts";
import { count, pct } from "../format.ts";
import { RELIEF, STIMULUS, federalPeak, gaps } from "../history.ts";
import { firstOf, lastOf } from "../ends.ts";
import { NAMES } from "../nav.ts";
import * as routes from "../routes.ts";
import { median } from "../stats.ts";
import type { HistoryYear } from "../types.ts";
import { fiscalYear } from "../yearLabel.ts";
import type { Identity, TopicModule } from "./topic.ts";

/**
 * The first year the survey nets Ohio's community-school deduct out of state revenue.
 *
 * Held to the catalog entry for the survey, which says the column "changes definition at" it.
 */
export const BREAK_YEAR = 2016;

/** A year of the panel, keyed so `reconciles` can name the one that fails. */
export type SourceYear = HistoryYear & { irn: string };

/** The panel's years, keyed on the fiscal year. */
export const years = (history: readonly HistoryYear[]): SourceYear[] =>
  history.map((y) => ({ ...y, irn: String(y.fiscal_year) }));

/** The three shares add to the whole: three figures printed to four places, half a step each. */
export const IDENTITY: Identity<SourceYear> = {
  left: () => 1,
  right: (y) => y.local_share + y.state_share + y.federal_share,
  tolerance: 1.5e-4,
};

/** The federal share's usual size: the median of the years between the two relief windows. */
export function usualFederal(history: readonly HistoryYear[]): number {
  return median(
    history.filter((y) => y.fiscal_year > STIMULUS[1] && y.fiscal_year < RELIEF[0]).map((y) => y.federal_share),
  );
}

/** The pandemic relief peak, and how many times the usual share it ran at. */
export function reliefPeak(history: readonly HistoryYear[]) {
  const peak = federalPeak([...history], RELIEF)!;
  return { peak, times: peak.federal_share / usualFederal(history) };
}

export const SOURCES: TopicModule = {
  slug: "sources",
  group: "How the money is split",
  question: "Where does school money come from?",
  build: () => {
    const history = loadFeed().bundle.history;
    const last = lastOf(history);
    const fy = fiscalYear(last.fiscal_year);
    const first = fiscalYear(firstOf(history).fiscal_year);
    const usual = usualFederal(history);
    const { peak } = reliefPeak(history);
    const stimulus = federalPeak(history, STIMULUS)!;
    const missing = gaps(history).map(fiscalYear);
    const sum = last.local_share + last.state_share + last.federal_share;

    return {
      shortAnswer: {
        lead: "Ohio's schools are paid for from three places: local taxes, the state, and the federal government.",
        rest: `In ${fy}, local money was the largest part, at ${pct(last.local_share, 0)}.`,
      },
      picture:
        "It is like three taps filling one tub. Each tap runs at its own rate. " +
        "The water in the tub is what the three put in together.",
      breaks: [
        "Taps run at a steady rate. The federal one did not. " +
          `It usually gave about ${pct(usual, 0)} of the money. It rose to ${pct(stimulus.federal_share, 0)} in ${fiscalYear(stimulus.fiscal_year)} on stimulus money, ` +
          `and to ${pct(peak.federal_share, 0)} in ${fiscalYear(peak.fiscal_year)} on pandemic relief: about twice its usual rate.`,
        "And these are the Census Bureau's taps, not the formula's. The state's part here is every state dollar, building money included. " +
          `From ${fiscalYear(BREAK_YEAR)} the Bureau also counts it a new way, so the years either side of that one are not on one footing.`,
      ],
      numbers: {
        chip: "history",
        finding: `In ${fy}, local money was ${pct(last.local_share, 1)} of school revenue, the state's ${pct(last.state_share, 1)}, and federal money ${pct(last.federal_share, 1)}.`,
        intro: [
          `The example is the latest year the Census Bureau has published, across the ${count(last.districts)} Ohio districts it counts as comparable.`,
        ],
        steps: [
          `In ${fy}, local taxes and the rest of what districts raised gave ${pct(last.local_share, 1)} of school revenue.`,
          `The state gave ${pct(last.state_share, 1)}.`,
          `The federal government gave ${pct(last.federal_share, 1)}.`,
          `Together they make ${pct(sum, 0)}. Every dollar comes from one of the three.`,
        ],
        after: [
          `The three add up the same way in every year the Bureau has published, from ${first} to ${fy}. ` +
            "Local money is the largest part in every one of them." +
            (missing.length > 0 ? ` The Bureau's archive has no file for ${missing.join(" or ")}.` : ""),
          last.federal_share > usual
            ? `In ${fy} the federal share was still ${pct(last.federal_share, 1)}, above its usual ${pct(usual, 1)}, while the last of the relief money was spent.`
            : `By ${fy} the federal share was back to ${pct(last.federal_share, 1)}, near its usual ${pct(usual, 1)}.`,
        ],
      },
      rule: {
        notation: "<var>local</var> + <var>state</var> + <var>federal</var> = <var>total</var>",
        statute: "the Census Bureau's Annual Survey of School System Finances",
        href: routes.wikiSource("census-f33-school-system-finances"),
        symbols: [
          "<var>local</var> is property tax and every other source a district raises itself.",
          "<var>state</var> is every dollar from the state: the foundation formula, building money and the rest.",
          "<var>federal</var> is every dollar from Washington, including relief money that came and went.",
          "<var>total</var> is all the revenue the district takes in. Each share is one part over it.",
        ],
      },
      howWeKnow: [
        {
          claim: "[verified] Local property tax is the oldest of the three, and in Ohio an existing levy raises about the dollars it first raised, however values rise.",
          nodes: ["revenue-stream/local-property-tax"],
        },
        {
          claim: "[verified] Foundation aid is what the formula says a district costs, less what it is expected to raise, so a district that can raise less gets more of its cost from the state.",
          nodes: ["revenue-stream/state-foundation-aid"],
        },
        {
          claim: "[verified] Title I is the largest federal channel, and the pandemic relief was paid out in proportion to it.",
          nodes: ["revenue-stream/title-i", "revenue-stream/esser"],
        },
      ],
      readNext: {
        topics: ["what-a-district-gets", "state-share"],
        data: {
          href: NAMES.history.href,
          label: NAMES.history.name,
          note: "the local and state shares year by year, and how much of the gap between districts each one closes",
        },
      },
    };
  },
};
