/**
 * "Does spending more raise test scores?" (#716).
 *
 * # What the page computes
 *
 * Two correlations of spending against the Performance Index, over every district that carries
 * both, and the same two inside each poverty third — #702's banding, the one `/outcomes` draws.
 * {@link measure} computes them; `tests/unit/explained.spec.ts` holds the pooled pair to the feed's
 * published `statewide.outcomes` coefficients and the enrolled one to its bound corpus figure, so a
 * page that drifted from either would fail before it rendered.
 *
 * # What it may say
 *
 * Association, never effect (#712). The page says which districts get more and what they score,
 * and that poverty sits behind both; it does not say what a dollar does. And it keeps the
 * denominator finding whole: flat per need-weighted pupil, falling per enrolled pupil. The flat one
 * is a cancellation — the least-poor third rises, the poorest falls — and the page names it as one.
 */

import { loadFeed } from "../feed.ts";
import { compactMoney, count, escapeHtml, fixed, money, pct, signed } from "../format.ts";
import { compare } from "../order.ts";
import { scatterSpec } from "../plot/spec.ts";
import { renderToString } from "../plot/ssr.ts";
import { NAMES } from "../nav.ts";
import { bands, medianTrace, pairs } from "../relationships.ts";
import { correlation, median } from "../stats.ts";
import type { District } from "../types.ts";
import { yearOf } from "../year.ts";
import type { TopicModule } from "./topic.ts";

/** The two spending measures the department publishes side by side, and what each divides by. */
const MEASURES = {
  weighted: { of: (d: District) => d.outcome?.per_equivalent_pupil, per: "need-weighted pupil" },
  enrolled: { of: (d: District) => d.outcome?.per_enrolled_pupil, per: "enrolled pupil" },
} as const;

type Measure = keyof typeof MEASURES;

const score = (d: District) => d.outcome?.performance_index;

/** The thirds' names, least poor first, as `/outcomes` draws them. */
const THIRDS = ["least poor", "middle", "poorest"] as const;

/** A correlation, signed and to three places, as `/outcomes` prints it. */
const coefficient = (v: number) => signed(v, 3);

/**
 * Every district with both spending measures, a score and a poverty third, and the third it is in.
 *
 * The thirds are drawn over every district with a poverty share, as `/outcomes` draws them, and
 * then filtered: banding the filtered list instead would move a district at a boundary.
 */
export function rows(districts: readonly District[]) {
  const third = bands([...districts], (d) => d.economically_disadvantaged);
  return districts
    .filter((d) => MEASURES.weighted.of(d) != null && MEASURES.enrolled.of(d) != null && score(d) != null)
    .flatMap((d) => {
      const t = third.get(d);
      return t == null ? [] : [{ d, third: t }];
    });
}

/** Spending against score, over the districts given. */
function r(list: readonly District[], measure: Measure): number {
  return correlation(
    list.map((d) => MEASURES[measure].of(d)!),
    list.map((d) => score(d)!),
  );
}

/** The pooled correlation and one per third, for one measure. */
export function measure(districts: readonly District[], which: Measure) {
  const all = rows(districts);
  return {
    pooled: r(all.map((x) => x.d), which),
    thirds: THIRDS.map((_, t) => r(all.filter((x) => x.third === t).map((x) => x.d), which)),
  };
}

/** The example: the district whose spending per enrolled pupil is the median, ties broken on IRN. */
export function example(districts: readonly District[]) {
  const all = rows(districts);
  const middle = median(all.map((x) => MEASURES.enrolled.of(x.d)!));
  const gap = (d: District) => Math.abs(MEASURES.enrolled.of(d)! - middle);
  return [...all].sort((a, b) => gap(a.d) - gap(b.d) || compare(a.d.irn, b.d.irn))[0]!;
}

/** Spending per need-weighted pupil against score, one median line per third and one through all. */
function chart(districts: readonly District[], spent: string, scored: string) {
  const third = new Map(rows(districts).map((x) => [x.d, x.third]));
  const points = pairs(
    [...third.keys()],
    MEASURES.weighted.of,
    score,
    (d, dollars, index) =>
      `${d.name}: ${money(dollars)} per need-weighted pupil, Performance Index ${fixed(index, 1)}, the ${THIRDS[third.get(d)!]} third`,
    { band: (d) => third.get(d) },
  );
  const xy = (list: typeof points) => list.map((p) => ({ x: p.x, y: p.y }));
  const traces = [
    { ...medianTrace(xy(points), 5, "All districts", "formula"), pooled: true },
    ...THIRDS.map((name, band) => ({
      ...medianTrace(xy(points.filter((p) => p.band === band)), 4, name[0]!.toUpperCase() + name.slice(1), "formula"),
      band,
    })),
  ];
  const svg = renderToString(
    (w) =>
      scatterSpec(
        points,
        {
          x: { label: "spending per need-weighted pupil", format: compactMoney, log: true },
          y: { label: "Performance Index", format: (v) => fixed(v, 0) },
        },
        traces,
        { width: w },
      ),
    {
      label: `Spending per need-weighted pupil against Performance Index, ${count(points.length)} districts shaded by poverty third, with a median line for each third and a dashed one through all, ${spent} spending and ${scored} scores`,
      description: "The least poor third's line rises, the poorest third's falls, and the dashed line through all of them is close to level.",
    },
  );
  return {
    svg,
    caption:
      "Each dot is one district: across is spending per need-weighted pupil, up is its score. " +
      "Each solid line is for one third of districts by poverty, and the dashed line is for all of them.",
  };
}

export const SPENDING: TopicModule = {
  slug: "spending",
  group: "What it does and does not show",
  question: "Does spending more raise test scores?",
  build: () => {
    const districts = loadFeed().bundle.districts;
    const spent = yearOf("outcome.spending");
    const scored = yearOf("outcome.performance");
    const counted = yearOf("profile");
    const all = rows(districts);
    const weighted = measure(districts, "weighted");
    const enrolled = measure(districts, "enrolled");
    const ex = example(districts);
    const d = ex.d;
    const name = escapeHtml(d.name);
    const peers = all.filter((x) => x.third === ex.third).map((x) => score(x.d)!);
    const peerScore = median(peers);
    const [rich, middle, poor] = weighted.thirds;
    const thirdMedian = (t: number, of: (d: District) => number | null | undefined) =>
      median(all.filter((x) => x.third === t).map((x) => of(x.d)!));

    return {
      shortAnswer: {
        lead: "Across Ohio, districts that spend more on each pupil do not score higher.",
        rest: "Poorer districts get more money and score lower, and that explains most of the link.",
      },
      picture:
        "It is like comparing hospitals. The ones that spend the most on each patient treat the sickest. " +
        "So more money goes with worse health, and no one thinks the money makes people ill.",
      breaks: [
        "A single number can show how closely two things move together. It runs from minus one, through zero for not at all, to one. " +
          `Spending in ${spent} against scores in ${scored} gives ${coefficient(enrolled.pooled)} for each enrolled pupil, ` +
          `and ${coefficient(weighted.pooled)} for each pupil weighted for need. The weighting counts poorer pupils as more than one.`,
        `And the near zero is not nothing. In the least poor third of districts, the number for each weighted pupil is ${coefficient(rich!)}. ` +
          `In the middle third it is ${coefficient(middle!)}, and in the poorest ${coefficient(poor!)}. Pooled, tilts that run opposite ways add up to almost nothing.`,
      ],
      numbers: {
        chip: "outcome.performance",
        finding: `In ${spent}, ${name} spent ${money(MEASURES.enrolled.of(d)!)} for each enrolled pupil, and scored ${fixed(score(d)!, 1)} in ${scored}.`,
        intro: [
          `The example is the middle one of the ${count(all.length)} districts with a score, by spending on each enrolled pupil: ${name}.`,
        ],
        steps: [
          `In ${spent}, ${name} spent ${money(MEASURES.enrolled.of(d)!)} for each pupil enrolled.`,
          `Weighting its pupils for need, that is ${money(MEASURES.weighted.of(d)!)} a pupil.`,
          `In ${counted}, ${pct(d.economically_disadvantaged!, 0)} of its pupils were economically disadvantaged: the ${THIRDS[ex.third]} third.`,
          `Its Performance Index in ${scored} was ${fixed(score(d)!, 1)}. Its third's median was ${fixed(peerScore, 1)}.`,
        ],
        chart: chart(districts, spent, scored),
        after: [
          `Across the thirds, the median score falls from ${fixed(thirdMedian(0, score), 1)} to ${fixed(thirdMedian(2, score), 1)}, ` +
            `and median spending per enrolled pupil in ${spent} rises from ${money(thirdMedian(0, MEASURES.enrolled.of))} to ${money(thirdMedian(2, MEASURES.enrolled.of))}. ` +
            "The money follows poverty, and so do the scores.",
          "None of this says what a dollar does to a score. It says which districts get more, and that poverty sits behind both.",
        ],
      },
      rule: {
        notation: "<var>PI</var> = Σ <var>w</var><sub>level</sub> × <var>n</var><sub>level</sub> ÷ <var>N</var>",
        statute: "R.C. 3302.03",
        href: "https://codes.ohio.gov/ohio-revised-code/section-3302.03",
        symbols: [
          "<var>n</var><sub>level</sub> is how many of a district's pupils scored at each level of the state tests.",
          "<var>w</var><sub>level</sub> is that level's weight: a hundred for proficient, more above it, less below.",
          "A pupil who should have been tested and was not counts as zero.",
          "<var>N</var> is the number of pupils who should have been tested.",
        ],
      },
      howWeKnow: [
        {
          claim: `[verified] Spending per enrolled pupil tracks the Performance Index at ${coefficient(enrolled.pooled)}; spending per need-weighted pupil, at ${coefficient(weighted.pooled)}.`,
          nodes: ["metric/expenditure-per-equivalent-pupil"],
          figures: ["dispersion/headcount-estimate-negative-excluding-top-spender"],
        },
        {
          claim: "[verified] The Performance Index is mostly a poverty measurement: most of how it varies between districts is the share of pupils who are economically disadvantaged.",
          nodes: ["metric/performance-index"],
        },
      ],
      readNext: {
        topics: ["what-a-district-gets", "guarantee"],
        data: {
          href: NAMES.outcomes.href,
          label: NAMES.outcomes.name,
          note: "both spending measures drawn against scores, and how much of the scores poverty explains",
        },
      },
    };
  },
};
