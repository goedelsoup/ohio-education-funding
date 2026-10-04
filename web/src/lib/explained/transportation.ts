/**
 * "How is a bus ride paid for?" (#718).
 *
 * # The identity
 *
 * {@link IDENTITY}: the school bus payment is `max(R, M) × max(s, f)` — the greater of the rider
 * base and the mile base, at the greater of the district's state share and the transportation
 * floor — rounded *down* to the dime, against the department's published line, on every district.
 * The truncation is the department's and no other line has it; `crates/project/tests/transportation.rs`
 * found it first, and {@link dime} carries that test's epsilon for the same reason: flooring turns a
 * float's last bit into ten cents.
 *
 * # The numbers
 *
 * The example is the district whose school bus payment is the median. The rider base is checked
 * against {@link PER_RIDER} on every district, because the feed carries the riders; the mile base is
 * not, because the feed carries no miles, so {@link PER_MILE} is held to the corpus node instead.
 *
 * # What it may not say
 *
 * How much of the payment carries pupils to private and community schools. The weights put a
 * premium on them under either base, but the mile split is not in the feed, and the rider mix is
 * not the mile mix: a district paid on miles cannot be attributed by its riders, nor the state by
 * an aggregate ratio. So the page states the weights and prices nothing with them.
 */

import { loadFeed } from "../feed.ts";
import { count, escapeHtml, millions, money, pct } from "../format.ts";
import { compare } from "../order.ts";
import * as routes from "../routes.ts";
import { median } from "../stats.ts";
import type { District } from "../types.ts";
import { fiscalYear } from "../yearLabel.ts";
import type { Identity, TopicModule } from "./topic.ts";

/** Per weighted rider, as `project::panel::supplements::TRANSPORT_PER_RIDER` has it. */
export const PER_RIDER = 1337.175;
/** Per weighted bus mile, across {@link SCHOOL_DAYS} days. */
export const PER_MILE = 6.867;
/** A school year, as the mile base counts it. */
export const SCHOOL_DAYS = 180;

/** The base a district is paid on: the greater of riders and miles. */
export const base = (d: District) =>
  Math.max(d.transportation.per_rider_base, d.transportation.per_mile_base);

/** The state share transportation pays at: the district's own, or the floor where that is higher. */
export const share = (d: District, floor: number) => Math.max(d.transportation_state_share, floor);

/**
 * Rounded down to the dime, as the department rounds the school bus line. The epsilon is the width
 * of the float error being truncated, not slack: without it a product landing a bit under a dime
 * boundary loses ten cents.
 */
export const dime = (v: number) => Math.floor(v * 10 + 1e-6) / 10;

/** The published school bus payment against the rule; the feed prints it to the cent. */
export function identity(floor: number): Identity<District> {
  return {
    left: (d) => d.transportation.school_bus,
    right: (d) => dime(base(d) * share(d, floor)),
    tolerance: 0.005,
  };
}

/** The example: the district whose school bus payment is the median, ties broken on IRN. */
export function example(districts: readonly District[]): District {
  const bus = (d: District) => d.transportation.school_bus;
  const middle = median(districts.map(bus));
  const gap = (d: District) => Math.abs(bus(d) - middle);
  return [...districts].sort((a, b) => gap(a) - gap(b) || compare(a.irn, b.irn))[0]!;
}

export const TRANSPORTATION: TopicModule = {
  slug: "transportation",
  group: "How the money is split",
  question: "How is a bus ride paid for?",
  build: () => {
    const { districts, statewide } = loadFeed().bundle;
    const fy = fiscalYear(districts[0]!.biennium.year_terminal);
    const floor = statewide.transportation_floor;
    const ex = example(districts);
    const name = escapeHtml(ex.name);
    const t = ex.transportation;
    const s = ex.transportation_state_share;
    const onMiles = districts.filter((d) => d.transportation.paid_on_miles).length;
    const floored = districts.filter((d) => d.transportation_state_share < floor).length;
    const bus = districts.reduce((sum, d) => sum + d.transportation.school_bus, 0);
    const total = districts.reduce((sum, d) => sum + d.transportation.total, 0);
    const winner = t.paid_on_miles ? "the miles" : "the riders";
    const extras = t.guarantee > 0 ? "Other vehicles, two supplements and a guarantee" : "Other vehicles and two supplements";

    return {
      shortAnswer: {
        lead: "The state pays per rider or per mile, whichever is more.",
        rest: `In ${fy}, it pays at least half that, even where the formula would pay a district less.`,
      },
      picture:
        "It is like a taxi that can charge by the passenger or by the mile, and takes the higher fare. " +
        "Then a sponsor promises to pay at least half of it.",
      breaks: [
        "A taxi's meter shows which fare it ran. The payment does not: a district paid on miles and one paid on riders look the same. " +
          `In ${fy}, ${count(onMiles)} of ${count(districts.length)} districts were paid on miles.`,
        "And in a taxi every passenger counts once. Here a pupil riding to a private school counts twice, and one riding to a community school one and a half times. " +
          "The same weights apply to the miles, so the premium is there under either fare.",
      ],
      numbers: {
        chip: "formula",
        finding: `In ${fy}, ${name} is paid ${money(t.total)} for transportation. Its school bus payment, ${money(t.school_bus)}, is the middle one in the state.`,
        intro: [
          `The example is ${name}, whose school bus payment is the middle one of the ${count(districts.length)} districts.`,
        ],
        steps: [
          `${name} counts ${count(Math.round(t.weighted_riders))} weighted riders. At ${money(PER_RIDER, 3)} each, that is ${money(t.per_rider_base)}.`,
          `Counted by the mile, at ${money(PER_MILE, 3)} a mile for ${count(SCHOOL_DAYS)} days, it is ${money(t.per_mile_base)}.`,
          `The greater one wins. For ${name} that is ${winner}, at ${money(base(ex))}.`,
          s < floor
            ? `The formula's state share for ${name} is ${pct(s, 1)}. Transportation lifts it to ${pct(floor, 0)}.`
            : `The formula's state share for ${name} is ${pct(s, 1)}, above the ${pct(floor, 0)} floor.`,
          `So the school bus payment is ${money(base(ex))} at ${pct(share(ex, floor), 1)}: ${money(t.school_bus)}.`,
          `${extras} bring it to ${money(t.total)}.`,
        ],
        after: [
          `The same rule gives the school bus payment for every one of the ${count(districts.length)} districts, to the dime. ` +
            `Across the state it comes to ${millions(bus).replace("+", "")} of the ${millions(total).replace("+", "")} paid for transportation in ${fy}.`,
          `The floor does more here than in the formula, whose own minimum share is ${pct(statewide.minimum_state_share, 0)}. ` +
            `In ${fy}, ${count(floored)} districts have a share below ${pct(floor, 0)} and are paid at the floor.`,
        ],
      },
      rule: {
        notation:
          "<var>bus</var> = max(<var>R</var>, <var>M</var>) × max(<var>s</var>, <var>f</var>)",
        statute: "R.C. 3317.0212",
        href: "https://codes.ohio.gov/ohio-revised-code/section-3317.0212",
        symbols: [
          `<var>R</var> is weighted riders at ${money(PER_RIDER, 3)} each in ${fy}. A private school rider counts two, a community school rider one and a half.`,
          `<var>M</var> is weighted bus miles at ${money(PER_MILE, 3)} each, over a ${count(SCHOOL_DAYS)}-day year in ${fy}.`,
          `<var>s</var> is the district's state share from the formula, and <var>f</var> is the floor: ${pct(floor, 0)} in ${fy}.`,
          "The department rounds <var>bus</var> down to the dime.",
        ],
      },
      howWeKnow: [
        {
          claim: "[verified] The district is paid the greater of a per-rider base and a per-mile base, which one won is invisible in the amount, and the transportation minimum state share is five times the formula's.",
          nodes: ["formula-component/fsfp-transportation"],
        },
        {
          claim: "[verified] Non-public riders are weighted double and community or STEM riders one and a half, and R.C. 3317.0212(E)(1)(b) puts the same multiples on miles driven.",
          nodes: ["formula-component/fsfp-transportation"],
        },
      ],
      readNext: {
        topics: ["state-share", "what-a-district-gets"],
        data: {
          href: `${routes.district(ex.irn)}#transportation`,
          label: `${name}'s transportation`,
          note: "both bases, the share, the supplements and special education transportation, step by step",
        },
      },
    };
  },
};
