/**
 * "What is a floor or a ceiling, and whom does it decide?" (#718).
 *
 * # The identity
 *
 * {@link IDENTITY}: for each bound in {@link CHECKED}, the census's count of districts it is the
 * operative term for, against a recount from the feed's own columns, exactly. The census is
 * `project/bounds-census` in `crates/series.json`, which `/bounds` draws; the recount is a predicate
 * on a {@link District}. Eight of the census's rows can be recounted that way, and those are the
 * eight the page leans on. The others need inputs the feed does not carry — staffing elements,
 * capacity tiers — and the page names none of them except through the census itself.
 *
 * # The numbers
 *
 * The example is the district on the most of the eight, ties broken on IRN, so the steps are a
 * list of the bounds that decide part of one district's aid, each in the words of the bound. The
 * census-wide figures — how many bounds there are, which reaches most, which cannot bind — come
 * from `census()`, as `/bounds` reads them, so the two pages cannot disagree.
 *
 * # What it may not say
 *
 * That a district on a bound would be paid less without it. The census counts the side a provision
 * produces, not what lowering it would move: 138 districts sit on the minimum state share and far
 * fewer move when it is lowered, because the guarantee catches them. The second break says so, and
 * prices nothing.
 */

import { census } from "../bounds.ts";
import type { SeriesRow } from "../corpusSeries.ts";
import { loadFeed } from "../feed.ts";
import { count, escapeHtml, fixed, money, pct } from "../format.ts";
import { NAMES } from "../nav.ts";
import { compare } from "../order.ts";
import type { District } from "../types.ts";
import { fiscalYear } from "../yearLabel.ts";
import { blend } from "./poverty-count.ts";
import type { Identity, TopicModule } from "./topic.ts";

/** R.C. 3317.0212(F)(3)(a): the efficiency index past which the supplement stops rising. */
const EFFICIENCY_CEILING = 1.5;
/** The most the efficiency supplement adds, at that index. */
const EFFICIENCY_MOST = 0.15;
/** R.C. 3317.0212(H)(1): riders a square mile at and above which density aid is zero. */
const DENSITY_CUTOFF = 28;

/** One census row the feed can recount, and what the bound does to a district it decides. */
export interface Checked {
  /** The census row's label, as `crates/project` writes it. */
  label: string;
  on: (d: District) => boolean;
  says: (d: District, m: number) => string;
}

export const CHECKED: readonly Checked[] = [
  {
    label: "The guarantee itself",
    on: (d) => d.guarantee > 0,
    says: (d) => `The formula pays it less than its old amount, so the guarantee adds ${money(d.guarantee)}.`,
  },
  {
    label: "Minimum state share of base cost",
    on: (d) => d.at_minimum_state_share,
    says: (_, m) => `Its wealth puts its formula share below ${pct(m, 0)}, so that floor sets its share.`,
  },
  {
    label: "Transport paid on miles, not riders",
    on: (d) => d.transportation.paid_on_miles,
    says: () => "Its bus aid is counted by the mile, because the miles pay more than the riders.",
  },
  {
    label: "Transportation's own guarantee",
    on: (d) => d.transportation.guarantee > 0,
    says: (d) => `Its bus aid fell below an earlier level, so a second guarantee adds ${money(d.transportation.guarantee)}.`,
  },
  {
    label: `Efficiency adjustment capped at ${pct(EFFICIENCY_MOST, 0)}`,
    on: (d) => d.transportation.efficiency_index >= EFFICIENCY_CEILING,
    says: () => `Its buses are full enough that the efficiency bonus stops at its ceiling, ${pct(EFFICIENCY_MOST, 0)} more.`,
  },
  {
    label: `No density aid from ${count(DENSITY_CUTOFF)} riders a sq. mile`,
    on: (d) => d.transportation.district_density >= DENSITY_CUTOFF,
    says: (d) =>
      `It has ${fixed(d.transportation.district_density, 1)} riders a square mile, at least ${count(DENSITY_CUTOFF)}, so its density aid is zero.`,
  },
  {
    label: "Guarantee base held at zero",
    on: (d) => d.transition.funding_base < 0,
    says: () => "Its old base is below zero, so the guarantee holds that base at zero.",
  },
  {
    label: "Poverty aid count capped at enrollment",
    on: (d) => blend(d) > d.current_year_adm,
    says: () => "Its count of poor pupils would exceed its enrollment, so enrollment caps it.",
  },
];

/** A checked bound's census count beside the feed's recount. */
export interface Recount {
  irn: string;
  census: number;
  recount: number;
}

/** Each checked bound, its census row found by label: a renamed row should fail here, not vanish. */
export function recounts(rows: readonly SeriesRow[], districts: readonly District[]): Recount[] {
  return CHECKED.map((b) => {
    const row = rows.find((r) => r.label === b.label);
    if (!row) throw new Error(`The bounds census carries no row "${b.label}"`);
    return { irn: b.label, census: row.value, recount: districts.filter(b.on).length };
  });
}

/** Counts of districts, so there is no room either side. */
export const IDENTITY: Identity<Recount> = {
  left: (r) => r.census,
  right: (r) => r.recount,
  tolerance: 0,
};

/** The example: the district on the most checked bounds, ties broken on IRN. */
export function example(districts: readonly District[]): District {
  const on = (d: District) => CHECKED.filter((b) => b.on(d)).length;
  return [...districts].sort((a, b) => on(b) - on(a) || compare(a.irn, b.irn))[0]!;
}

export const BOUNDS: TopicModule = {
  slug: "bounds",
  group: "How the money is split",
  question: "What is a floor or a ceiling, and whom does it decide?",
  build: () => {
    const { districts, statewide } = loadFeed().bundle;
    const fy = fiscalYear(districts[0]!.biennium.year_terminal);
    const c = census();
    const m = statewide.minimum_state_share;
    const ex = example(districts);
    const name = escapeHtml(ex.name);
    const its = CHECKED.filter((b) => b.on(ex));
    const guaranteed = districts.filter(CHECKED[0]!.on).length;
    const floored = districts.filter(CHECKED[1]!.on).length;
    const inert = c.cannotBind;

    return {
      shortAnswer: {
        lead: "A floor is the least a district can get; a ceiling is the most.",
        rest: `In ${fy}, the formula holds ${count(c.rows.length)}. Each decides the amount only where a district reaches it.`,
      },
      picture:
        "It is like a minimum wage. A worker paid above it never feels it. " +
        "For a worker who would earn less, it is the whole of the pay.",
      breaks: [
        "A minimum wage is one number, and everyone knows it. The formula has dozens, and most of them sit in steps a reader never sees. " +
          `In ${fy}, the one that reaches the most districts decides ${count(c.most.value)} of ${count(c.most.of ?? c.whole)}.`,
        "And a worker on the minimum wage would earn less without it. A district on a floor might not. " +
          "Take that floor away, and another may catch the district first.",
      ],
      numbers: {
        chip: "formula",
        finding: `In ${fy}, ${name} sits on ${count(its.length)} of the bounds this page can check, and each one decides part of its aid.`,
        intro: [
          `The example is ${name}. Of the ${count(CHECKED.length)} bounds this page can check from the published figures, it sits on the most.`,
        ],
        steps: its.map((b) => b.says(ex, m)),
        after: [
          `For each of those ${count(CHECKED.length)} bounds, this page counts the districts on it, and each count matches the site's census. ` +
            `In ${fy}, the guarantee decides ${count(guaranteed)} districts and the minimum share ${count(floored)}.`,
          `Not every bound decides anyone. ${inert.length === 1 ? "One" : count(inert.length)} of the ${count(c.rows.length)} cannot bind at all, whatever a district's numbers.`,
        ],
      },
      rule: {
        notation:
          "<var>paid</var> = max(<var>amount</var>, <var>floor</var>), or min(<var>amount</var>, <var>ceiling</var>)",
        statute: "R.C. 3317",
        href: "https://codes.ohio.gov/ohio-revised-code/chapter-3317",
        symbols: [
          "<var>amount</var> is what the formula computes before the bound.",
          "<var>floor</var> is the least the law allows, and <var>ceiling</var> the most. A clamp has both.",
          `A bound decides for a district when its side wins. The minimum state share did for ${count(floored)} districts in ${fy}.`,
          "A step that pays one rate below a line and another above it is not a bound.",
        ],
      },
      howWeKnow: [
        {
          claim: "[verified] The modeled formula's bounds sit across base cost, local capacity, the categoricals, the guarantee and transportation, and exactly one of them cannot bind.",
          nodes: ["funding-regime/fair-school-funding-plan"],
        },
        {
          claim: "[verified] Every other bound is reached by at least one district in the department's model, so no bound that can bind has never bound.",
          nodes: ["funding-regime/fair-school-funding-plan"],
        },
      ],
      readNext: {
        topics: ["state-share", "guarantee"],
        data: {
          href: `${NAMES.bounds.href}#census`,
          label: "Every bound, ranked",
          note: "all of them, with the law behind each and the districts each one decides",
        },
      },
    };
  },
};
