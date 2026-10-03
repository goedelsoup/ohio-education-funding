/**
 * "Why do so many districts sit on the state's floor?" (#716).
 *
 * # The identity
 *
 * R.C. 3317.017(C): the state share percentage is `(B − L) / B`, and where that is below the
 * minimum it is the minimum. {@link IDENTITY} rebuilds it from base cost per pupil and local
 * capacity per pupil and checks it against the department's published percentage for the model
 * year, on every district.
 *
 * # The reach, run rather than counted
 *
 * The floor's population is not its reach. Most districts on it are also on the guarantee, which
 * already pays them more than the formula, so lowering the floor takes nothing from them.
 * {@link reach} runs the scenario page's own model with the minimum at the lowest setting that
 * page allows, and counts the districts whose core foundation funding moves — the same run, and
 * the same `MOVED` bound, as `/scenario` and `/reach`. Nothing on this page is typed from
 * `crates/scenario-delta`'s test of it; the browser model reproduces that test, and
 * `tests/unit/explained.spec.ts` holds the two counts together through the bound figure.
 */

import { loadFeed } from "../feed.ts";
import { compactMoney, count, escapeHtml, money, pct } from "../format.ts";
import { compare } from "../order.ts";
import { scatterSpec } from "../plot/spec.ts";
import { renderToString } from "../plot/ssr.ts";
import { NAMES } from "../nav.ts";
import { MOVED, applyAll, modelOf, type Outcome } from "../policy.ts";
import { pairs } from "../relationships.ts";
import { LEVER_BOUNDS, defaultLevers, toPolicy } from "../scenario.ts";
import { median } from "../stats.ts";
import type { Bundle, District } from "../types.ts";
import { fiscalYear } from "../yearLabel.ts";
import type { Identity, TopicModule } from "./topic.ts";

/** The biennium's last column: the model year. */
const TERMINAL = 2;

/** Local capacity per pupil, which every district in the feed carries. */
function localCapacity(d: District): number {
  const lc = d.regime?.local_capacity;
  if (lc == null) throw new Error(`${d.name} (${d.irn}) carries no local capacity per pupil`);
  return lc;
}

/** What the district can raise, as a share of what the formula says its pupils cost. */
const capacityShare = (d: District) => localCapacity(d) / d.base_cost_per_pupil;

/** The state share percentage as the rule computes it: `max(1 − L/B, m)`. */
export const shareOf = (d: District, floor: number) => Math.max(1 - capacityShare(d), floor);

/** The department's published state share percentage for the model year. */
function published(d: District): number {
  const s = d.biennium.observed[TERMINAL]!.state_share;
  if (s == null) throw new Error(`${d.name} (${d.irn}) has no published state share for the model year`);
  return s;
}

/**
 * The published percentage, rebuilt: `max(1 − L/B, m)`.
 *
 * The department publishes it to seven places as the per-pupil state share over base cost per
 * pupil, and rounds the per-pupil share to the cent first. A cent over the smallest base cost in
 * the state is about a millionth and a quarter, so the bound is a millionth and a half; the widest
 * gap observed is a millionth.
 */
export function identity(floor: number): Identity<District> {
  return { left: published, right: (d) => shareOf(d, floor), tolerance: 1.5e-6 };
}

/** The floor lowered to the least the scenario page allows, run through that page's model. */
export function reach(bundle: Bundle) {
  const model = modelOf(bundle.statewide);
  const low = LEVER_BOUNDS.minimumStateShare.min;
  const outcomes = applyAll(bundle.districts, toPolicy({ ...defaultLevers(model), minimumStateShare: low }), model);
  const moved = outcomes.filter((o) => Math.abs(o.delta) > MOVED);
  const onFloor = bundle.districts.filter((d) => d.at_minimum_state_share);
  const floor = model.minimumStateShare;
  return {
    low,
    moved,
    onFloor,
    heldByGuarantee: onFloor.filter((d) => d.on_guarantee && !moved.some((o) => o.irn === d.irn)),
    saved: -moved.reduce((s, o) => s + o.delta, 0),
    // From the published share, never `m × B`: the share is paid on this year's pupils and base
    // cost on a three-year average, so a tenth of base cost is not a tenth of the share.
    naive: onFloor.reduce((s, d) => s + d.base_cost_state_share * (1 - low / floor), 0),
  };
}

/** The example: the district that moves whose loss is the median, ties broken on IRN. */
export function example(moved: readonly Outcome[]): Outcome {
  const middle = median(moved.map((o) => o.delta));
  const gap = (o: Outcome) => Math.abs(o.delta - middle);
  return [...moved].sort((a, b) => gap(a) - gap(b) || compare(a.irn, b.irn))[0]!;
}

/** What a district can raise against the share the state pays, both as shares of base cost. */
function chart(districts: readonly District[], floor: number, year: string) {
  const points = pairs(
    [...districts],
    capacityShare,
    (d) => shareOf(d, floor),
    (d, raise, share) =>
      `${d.name}: can raise ${pct(raise, 0)} of its cost, the state pays ${pct(share, 0)}` +
      (d.on_guarantee ? ", and the guarantee pays more" : ""),
    { series: (d) => (d.on_guarantee ? "guarantee" : "formula") },
  );
  const svg = renderToString(
    (w) =>
      scatterSpec(
        points,
        {
          x: { label: "what the district can raise, as a share of its cost", format: (v) => pct(v, 0) },
          y: { label: "the state's share", format: (v) => pct(v, 0) },
        },
        [],
        { width: w },
      ),
    {
      label: `The state share percentage against local capacity as a share of base cost, ${count(points.length)} districts, ${year}`,
      description: "Each district is one dot. The state's share falls as a district can raise more, then flattens at the floor.",
    },
  );
  return {
    svg,
    caption:
      "Each dot is one district: across is what it can raise as a share of its cost, up is the state's share. " +
      `The dots fall, then flatten at ${pct(floor, 0)}. Dots marked as guaranteed are paid more than the formula anyway.`,
  };
}

export const STATE_SHARE: TopicModule = {
  slug: "state-share",
  group: "How the money is split",
  question: "Why do so many districts sit on the state's floor?",
  build: () => {
    const bundle = loadFeed().bundle;
    const districts = bundle.districts;
    const floor = bundle.statewide.minimum_state_share;
    const fy = fiscalYear(districts[0]!.biennium.year_terminal);
    const r = reach(bundle);
    const lowPct = pct(r.low, 0);
    const floorPct = pct(floor, 0);
    const o = example(r.moved);
    const ex = districts.find((d) => d.irn === o.irn)!;
    const name = escapeHtml(ex.name);
    const raise = capacityShare(ex);
    const own = 1 - raise;

    return {
      shortAnswer: {
        lead: `The state pays at least ${floorPct} of what school costs in each district, however wealthy.`,
        rest: `In ${fy}, ${count(r.onFloor.length)} districts sit on that floor.`,
      },
      picture:
        "It is like a rule that every bill gets a minimum tip. " +
        "Each diner picks a tip of their own. " +
        "A diner whose tip comes to less than the minimum pays the minimum instead.",
      breaks: [
        "A minimum tip changes what a diner pays. Here, for most districts on the floor, it changes nothing. " +
          `In ${fy}, ${count(r.heldByGuarantee.length)} of the ${count(r.onFloor.length)} are on the guarantee, which already pays them more.`,
        `So if the floor were lower, most of the districts on it would not notice. At ${lowPct} in ${fy}, only ${count(r.moved.length)} would get less.`,
      ],
      numbers: {
        chip: "formula",
        finding: `In ${fy}, a floor of ${lowPct} instead of ${floorPct} would cut ${count(r.moved.length)} districts, and save the state ${compactMoney(r.saved)}.`,
        intro: [
          `The example is the middle one of the ${count(r.moved.length)} districts a lower floor would cut, ranked by what each would lose. ` +
            `It is ${name}. The numbers are the department's ${fy} calculation.`,
        ],
        steps: [
          `The formula says one pupil in ${name} costs ${money(ex.base_cost_per_pupil, 2)}. It can raise ${money(localCapacity(ex), 2)} a pupil.`,
          raise >= 1
            ? "That is more than the whole cost, so on its own the state's share would be nothing."
            : `That is ${pct(raise, 0)} of the cost, so on its own the state's share would be ${pct(own, 0)}.`,
          `The floor is higher, so the state pays ${floorPct}: ${money(ex.base_cost_state_share)} for the year.`,
          `${name} is not held up by the guarantee. At a floor of ${lowPct}, it would get ${money(-o.delta)} less.`,
        ],
        chart: chart(districts, floor, fy),
        after: [
          `The same rule gives the department's share for every district in ${fy}. ` +
            `Lowering the floor to ${lowPct} would save the state ${compactMoney(r.saved)}, not the ${compactMoney(r.naive)} a count of the floor suggests.`,
          "The gap is the guarantee. A district it already pays more than the formula loses nothing when the formula pays less.",
        ],
      },
      rule: {
        notation: "<var>share</var> = max(one − <var>L</var> ÷ <var>B</var>, <var>m</var>)",
        statute: "R.C. 3317.017",
        href: "https://codes.ohio.gov/ohio-revised-code/section-3317.017",
        symbols: [
          "<var>B</var> is base cost per pupil.",
          "<var>L</var> is local capacity per pupil, what the district is expected to raise.",
          `<var>m</var> is the minimum state share, ${floorPct} in ${fy}.`,
          "The state pays <var>share</var> × <var>B</var> for each enrolled pupil.",
        ],
      },
      howWeKnow: [
        {
          claim: `[verified] The state share is base cost less local capacity, over base cost, and never below ${floorPct}; in ${fy}, ${count(r.onFloor.length)} districts sit on the floor.`,
          nodes: ["parameter/minimum-state-share"],
          figures: ["project/districts-at-the-minimum-state-share"],
        },
        {
          claim: "[verified] Local capacity blends property valuation and two income measures, and for a district on the floor it decides nothing.",
          nodes: ["parameter/local-capacity-percentage"],
        },
      ],
      readNext: {
        topics: ["what-a-district-gets", "guarantee"],
        data: {
          href: NAMES.bounds.href,
          label: NAMES.bounds.name,
          note: "every floor and cap in the formula, and how many districts each one reaches",
        },
      },
    };
  },
};
