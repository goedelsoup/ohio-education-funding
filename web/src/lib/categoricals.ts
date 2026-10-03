/**
 * The categorical half of formula aid as its six programs, for the dashboard card and its chart.
 *
 * Out of `Categoricals.astro` so the page can read the chart's bars before the card is drawn: the
 * base-cost chart above it shares its name gutter (#660), and a frontmatter binding is not
 * something another file can import.
 */
import type { Bar } from "./chart.ts";
import { money, pct } from "./format.ts";
import type { District } from "./types.ts";

/** One program: its name, its dollars, a line on its mechanism, and its corpus node. */
export type CategoricalPart = readonly [label: string, value: number, note: string, node: string];

/**
 * The six, in the card's order.
 *
 * Each program links to the corpus node describing its mechanism. The six were one residual until
 * recently and had nothing to link to; now each is a `formula-component` in its own right, and the
 * link out from a figure to what the figure *means* is the reason the wiki exists.
 */
export function categoricalParts(d: District): readonly CategoricalPart[] {
  const c = d.categoricals;
  return [
    ["Targeted assistance", c.targeted_assistance, "Equalization for low-valuation districts. Zero once a district has enough.", "fsfp-targeted-assistance"],
    ["Special education", c.special_education, "Six weighted categories of disability, spanning a factor of sixteen.", "fsfp-special-education-weights"],
    ["Disadvantaged Pupil Impact Aid", c.dpia, "Two poverty counts blended, indexed on the state's, and squared.", "fsfp-disadvantaged-pupil-impact-aid"],
    ["Career-technical education", c.career_technical, "Five weights against a base cost 20% above the one the rest of the plan uses.", "fsfp-career-technical-weights"],
    ["Gifted", c.gifted, "Mostly staffing units, with a floor 370 districts sit on.", "fsfp-gifted-units"],
    // "unlike every other categorical" until this commit, which is the same over-claim the drill-down
    // below carried and the same one the corpus node carried: career-technical's five weights descend
    // too. What is true of this program alone is that its weights fall along a *need* gradient.
    ["English learners", c.english_learners, "Three weights that descend as need persists, alone among the six.", "fsfp-english-learner-weights"],
  ];
}

/**
 * The chart's bars: all six, a program the district receives none of included as a stub labelled `$0` (#659). The
 * chart drew only the non-zero parts under a heading saying six, and 316 of 609 districts have at
 * least one zero, so more than half the dashboards showed five bars or fewer and the reader could
 * not tell an omitted program from one the chart had no room for. In dollars rather than `0%`,
 * because `0%` is also what a program under half a percent rounds to, and this is not that.
 */
export function categoricalBars(d: District): Bar[] {
  const total = d.categorical_funding;
  return categoricalParts(d).map(([label, value]) =>
    value > 0
      ? {
          label,
          value,
          direct: pct(value / total, 0),
          hover: `${label}: ${money(value)}, ${pct(value / total, 1)} of this district's categoricals`,
        }
      : { label, value: 0, direct: money(0), hover: `${label}: ${money(0)}. This district receives none.` },
  );
}
