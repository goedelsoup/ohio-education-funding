/**
 * Why a district's base cost is the number it is.
 *
 * # The one card on this site that shows arithmetic rather than a result
 *
 * Everything else per-district is the department's published model, passed through. Base cost is
 * the exception: `crates/foundation` runs the statutory staffing ratios in R.C. 3317.011 against
 * the district's own grade-band enrollment, prices them at the FY2027 statewide factor set, and
 * the feed carries all twenty-two elements.
 *
 * That is worth doing because "base cost per pupil" is the number every funding argument in Ohio
 * turns on, and as a single figure it is unarguable — there is nothing in it to agree or disagree
 * with. Broken into its parts it becomes a series of decisions: a classroom teacher for every 23
 * pupils in grades 1–3, a superintendent priced on a ramp between $80,000 and $160,000, and
 * building operation at a flat rate per pupil. Those are the things a school board is actually
 * arguing about, and they were previously visible only to someone reading Rust.
 *
 * # The reproduction is shown, not asserted
 *
 * The department publishes its own aggregate for every district. The card prints it beside the
 * computed one and states the difference. Across all 609 the worst is $1.09 on figures in the
 * millions, from twenty-two elements each rounded where the department rounds — but a page that
 * claimed to reproduce a number without showing the residual would be asking to be believed.
 */

import type { Bar } from "./chart.ts";
import { count, escapeHtml, money, pct } from "./format.ts";
import { barSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import * as routes from "./routes.ts";
import type { BaseCostBuildUp, District } from "./types.ts";

/** One element of the build-up: its statutory letter, what it pays for, and how much. */
interface Element {
  code: string;
  label: string;
  value: number;
  /** What the statute actually specifies, where a reader would otherwise have to guess. */
  note?: string;
}

/** One of the five sub-components, and the elements under it. */
interface Group {
  code: string;
  label: string;
  section: string;
  total: number;
  elements: Element[];
}

/** The build-up, grouped as R.C. 3317.011 groups it. */
export function groups(b: BaseCostBuildUp): Group[] {
  return [
    {
      code: "A",
      label: "Teachers",
      section: "3317.011(D)",
      total: b.teachers,
      elements: [
        {
          code: "A1",
          label: "Classroom teachers",
          value: b.classroom_teachers,
          note: "One per 20 pupils in kindergarten, 23 in grades 1–3, 25 in grades 4–8, 27 in grades 9–12, and 18 in career-technical.",
        },
        {
          code: "A2",
          label: "Special teachers",
          value: b.special_teachers,
          note: "Art, music and physical education, at one per 150 pupils.",
        },
        { code: "A3", label: "Substitutes", value: b.substitutes },
        {
          code: "A4",
          label: "Professional development",
          value: b.professional_development,
          note: "Four days against a 180-day year.",
        },
      ],
    },
    {
      code: "B",
      label: "Student support",
      section: "3317.011(E)",
      total: b.student_support,
      elements: [
        { code: "B1", label: "Guidance counselors", value: b.counselors },
        { code: "B2", label: "Librarians and media staff", value: b.librarians },
        { code: "B3", label: "Wellness and success staff", value: b.wellness },
        { code: "B4", label: "Academic co-curricular", value: b.academic_cocurricular },
        { code: "B5", label: "Safety and security", value: b.safety },
        { code: "B6", label: "Supplies and academic content", value: b.supplies },
        { code: "B7", label: "Student technology", value: b.technology },
      ],
    },
    {
      code: "C",
      label: "District leadership",
      section: "3317.011(F)",
      total: b.district_leadership,
      elements: [
        {
          code: "C1",
          label: "Superintendent",
          value: b.superintendent,
          note: "The only price in the formula that varies with district size: $160,000 above 4,000 pupils, $80,000 below 500, interpolated between.",
        },
        { code: "C2", label: "Treasurer", value: b.treasurer },
        {
          code: "C3",
          label: "Other administrators",
          value: b.other_administrators,
          note: "Priced at 82.8% of the superintendent band rather than looked up, so a change to that band moves this too.",
        },
        { code: "C4", label: "Fiscal support", value: b.fiscal_support },
        { code: "C5", label: "EMIS support", value: b.emis },
        { code: "C6", label: "Leadership support", value: b.leadership_support },
        { code: "C7", label: "Information technology centre", value: b.itc },
      ],
    },
    {
      code: "D",
      label: "Building leadership and operation",
      section: "3317.011(G)",
      total: b.building_leadership,
      elements: [
        {
          code: "D1",
          label: "Building leadership",
          value: b.building_leadership_staff,
          note: "Priced at 79.38% of the superintendent band.",
        },
        { code: "D2", label: "Building leadership support", value: b.building_support },
        { code: "D3", label: "Building operation", value: b.building_operation },
      ],
    },
    {
      code: "E",
      label: "Athletic co-curricular",
      section: "3317.011(H)",
      total: b.athletic_cocurricular,
      elements: [],
    },
  ];
}

/** Render the build-up for one district. */
export function renderBaseCostBuildUp(d: District): string {
  const b = d.base_cost_build_up;
  const parts = groups(b);
  const aggregate = b.computed_aggregate;

  const bars: Bar[] = parts.map((group) => ({
    label: group.label,
    value: group.total,
    direct: pct(group.total / aggregate, 0),
    hover: `${group.label} (R.C. ${group.section}): ${money(group.total)}, ${pct(group.total / aggregate, 1)} of base cost`,
  }));

  const rows = parts
    .flatMap((group) => [
      `<tr class="current">
        <th><span class="n">${group.code}</span> ${escapeHtml(group.label)}
          <span class="n">R.C. ${group.section}</span></th>
        <td class="tnum">${money(group.total)}</td>
        <td class="tnum">${pct(group.total / aggregate, 1)}</td>
      </tr>`,
      ...group.elements.map(
        (element) => `<tr>
          <th><span class="n">${element.code}</span> ${escapeHtml(element.label)}${
            element.note ? `<div class="n">${escapeHtml(element.note)}</div>` : ""
          }</th>
          <td class="tnum">${money(element.value)}</td>
          <td class="tnum n">${pct(element.value / aggregate, 1)}</td>
        </tr>`,
      ),
    ])
    .join("");

  // A dollar or so, from twenty-two roundings. Stated as a figure rather than waved at, and
  // widened to cents because at this size a percentage would read as zero and say nothing.
  const residual = Math.abs(b.residual);

  return `
    <div class="card">
      <h2>Why base cost is ${money(d.base_cost_per_pupil)} per pupil</h2>
      <p class="note">Base cost is not a rate the state sets. It is assembled for each district
        from staffing ratios written into R.C. 3317.011, applied to
        <strong>this district's own enrollment</strong> and priced at statewide average salaries.
        ${count(Math.round(b.funded_classroom_teachers))} funded classroom teachers and
        ${count(Math.round(b.funded_special_teachers))} special teachers, across
        ${count(Math.round(d.adm))} pupils.</p>

      <div class="chartwrap" data-chart="base-cost">${renderToString(barSpec(bars))}</div>

      <div class="scroll"><table>
        <thead><tr><th>Element</th><th>Amount</th><th>Share</th></tr></thead>
        <tbody>${rows}
          <tr class="current">
            <th>Aggregate base cost</th>
            <td class="tnum">${money(aggregate)}</td>
            <td class="tnum">100.0%</td>
          </tr>
        </tbody>
      </table></div>

      <p class="note"><strong>These figures are computed here, not quoted.</strong> Every other
        per-district number on this site is the department's published model passed through; this
        one is <code>crates/foundation</code> running the statute against this district's grade
        bands. The department publishes its own aggregate for the same district —
        ${money(b.published_aggregate)} — and the two differ by
        <strong>${residual < 0.005 ? "nothing" : money(residual, 2)}</strong>, which is what
        twenty-two elements each rounded at the point the department rounds them adds up to. The
        reproduction is checked across all
        ${count(609)} districts before this site is allowed to build.</p>

      <p class="note">What the state actually pays toward this is the
        <a href="${routes.metric("state-share-percentage")}">state share</a>, not the whole of it:
        ${money(d.base_cost_state_share)} of the ${money(d.aggregate_base_cost)} above. The rest is
        charged to local capacity, which is 60% assessed valuation. Changing the price inputs is
        the lever the <a href="${routes.wikiNode("scenario", "fsfp-input-year-refresh")}">cost input
        refresh</a> scenario perturbs; H.B. 96 held them at FY2022 through FY2027.</p>
    </div>`;
}
