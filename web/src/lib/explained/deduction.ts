/**
 * "What happened to the deduction for community schools and vouchers?" (#718).
 *
 * # The identity
 *
 * {@link IDENTITY}: a district's net payment in the model year is its total state support plus the
 * two transfer lines the payment report prints, `net = total + S + T`, on every district. Those two
 * are the only lines between the two figures, and neither is named for a community school or a
 * scholarship (`crates/project/tests/the_voucher_channel_is_absent.rs`). So the page can show, from
 * the department's own columns, everything that comes off a district's payment.
 *
 * # The numbers
 *
 * The example is the district paid the most, where a deduction scaled to the payment would be
 * largest. The comparison is the statewide transfers against the community and STEM unit, both
 * from the same model year: the six units are on different years and bases, and `/statewide` sums
 * none of them, so this page sets one beside the other and adds nothing across.
 *
 * # What it may not say
 *
 * That the transfers prove the deduction gone. A charge could hide inside "other adjustments", and
 * the column alone cannot rule it out; the statute's separate units are the proof, and the page
 * cites them for it. Nor that no trace remains: the guarantee's base subtracts the deductions of
 * the year it names, so the old charge survives inside a floor. The page says that in its breaks,
 * with the one district whose base it took below zero, found rather than named.
 */

import { loadFeed } from "../feed.ts";
import { count, escapeHtml, millions, money } from "../format.ts";
import { compare } from "../order.ts";
import { NAMES } from "../nav.ts";
import type { District } from "../types.ts";
import { fiscalYear } from "../yearLabel.ts";
import { BASE_YEAR } from "./phase-in.ts";
import type { Identity, TopicModule } from "./topic.ts";

/** The model year's detail: the last of `biennium.observed`, which is `year_terminal`. */
export const terminal = (d: District) => d.biennium.observed[2]!;

/** The two transfer lines, summed: the service center charge and other adjustments. */
export const transfers = (d: District) =>
  (terminal(d).service_center_charge ?? 0) + (terminal(d).other_adjustments ?? 0);

/** Net state funding against total state support plus both transfers: three cent figures. */
export const IDENTITY: Identity<District> = {
  left: (d) => terminal(d).net_state_funding ?? Number.NaN,
  right: (d) => d.biennium.total_terminal + transfers(d),
  tolerance: 0.015,
};

/** The example: the district with the most total state support, ties broken on IRN. */
export function example(districts: readonly District[]): District {
  return [...districts].sort(
    (a, b) => b.biennium.total_terminal - a.biennium.total_terminal || compare(a.irn, b.irn),
  )[0]!;
}

export const DEDUCTION: TopicModule = {
  slug: "deduction",
  group: "How the money is split",
  question: "What happened to the deduction for community schools and vouchers?",
  build: () => {
    const { districts, funding_units: units } = loadFeed().bundle;
    const fy = fiscalYear(districts[0]!.biennium.year_terminal);
    const base = fiscalYear(BASE_YEAR);
    const ex = example(districts);
    const name = escapeHtml(ex.name);
    const o = terminal(ex);
    const total = ex.biennium.total_terminal;
    const off = transfers(ex);
    const all = districts.reduce((s, d) => s + transfers(d), 0);
    const charged = districts.filter((d) => transfers(d) < 0).length;
    // The feed carries the six units whenever it carries the model year; a build without them should
    // fail here rather than compare the transfers with nothing.
    const community = units!.units.find((u) => u.slug === "community-stem")!;
    const below = districts.filter((d) => d.transition.funding_base < 0);

    return {
      shortAnswer: {
        lead: "It is gone. The state now pays community schools and vouchers directly.",
        rest: `In ${fy}, the state takes two lines off a district's payment, and neither is for them.`,
      },
      picture:
        "It used to be like a bill that came out of one person's pay. " +
        "Now each school gets its own envelope from the state, and the district's pay comes in its own.",
      breaks: [
        "Separate envelopes mean no one can say whose pay a voucher came out of. " +
          "Nothing published says which district a scholarship is charged against, so this site prices no lever on them.",
        `And the old bill left a stain. The guarantee's floor is built from ${base}, and that year's figure subtracts the deductions paid then. ` +
          (below.length === 1
            ? `For ${escapeHtml(below[0]!.name)} they were larger than its aid, so its floor is below zero.`
            : `For ${count(below.length)} districts they were larger than their aid, so their floors are below zero.`),
      ],
      numbers: {
        chip: "formula",
        finding: `In ${fy}, ${name} is paid ${money(total)} in state support, and ${money(-off)} comes off it, none of it for a community school or a voucher.`,
        intro: [
          `The example is ${name}, the district the state pays the most. A deduction from any district's payment would show in the biggest one.`,
        ],
        steps: [
          `The formula gives ${name} ${money(total)} in total state support for ${fy}.`,
          `The payment report takes off a service center charge of ${money(-(o.service_center_charge ?? 0))}.`,
          `It takes off ${money(-(o.other_adjustments ?? 0))} in other adjustments.`,
          `That leaves ${money(o.net_state_funding)}. No line in between names a community school or a voucher.`,
        ],
        after: [
          `The same two lines give the net payment for every one of the ${count(districts.length)} districts, to the cent. ` +
            `Across the state they take off ${millions(-all).replace("+", "")}, from ${count(charged)} districts.`,
          `In ${fy}, the state pays community schools ${millions(community.amount).replace("+", "")} of their own. ` +
            "That is paid beside the districts' total, not out of it.",
        ],
      },
      rule: {
        notation: "<var>net</var> = <var>total</var> + <var>S</var> + <var>T</var>",
        statute: "R.C. 3317.022",
        href: "https://codes.ohio.gov/ohio-revised-code/section-3317.022",
        symbols: [
          "<var>total</var> is the district's total state support from the formula.",
          "<var>S</var> is the charge for its educational service center, and <var>T</var> is every other adjustment.",
          "<var>net</var> is what the state pays the district. Neither <var>S</var> nor <var>T</var> is a voucher line.",
          "R.C. 3317.022 computes community schools and each scholarship as their own units, paid beside the district's.",
        ],
      },
      howWeKnow: [
        {
          claim: "[verified] Community school and STEM school students are funded directly by the state rather than by deducting from a resident district's foundation payment.",
          nodes: ["funding-regime/fair-school-funding-plan"],
        },
        {
          claim: "[verified] Under the deduct-era design a scholarship was funded by reducing the resident district's foundation payment, so a district could lose money for a child it never educated; under the Fair School Funding Plan it is not a deduction.",
          nodes: ["program/edchoice-scholarship", "program/edchoice-expansion"],
        },
        {
          claim: `[verified] R.C. 3317.02(N)(1)(b) subtracts ${base} community school, STEM and scholarship deductions from ${base} aid, which leaves one district's guarantee base negative.`,
          nodes: ["parameter/guarantee-funding-base"],
        },
      ],
      readNext: {
        topics: ["guarantee", "what-a-district-gets"],
        data: {
          href: `${NAMES.statewide.href}#funding-units`,
          label: "The six units, side by side",
          note: "the district unit beside the community school and scholarship units, each on its own year",
        },
      },
    };
  },
};
