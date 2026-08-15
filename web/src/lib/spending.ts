/**
 * Where a district's operating money went, by function.
 *
 * # Why this is on the finances route and not beside the audited actuals
 *
 * It is on the same page, in a separate card, with the source named — because it is a different
 * source from the five-year forecast filing above it and answers a different question. The
 * actuals are what changed hands, general fund, from the district's own filing. These are the
 * report card's FY2025 operating expenditure, per pupil, split by function. Both are records of
 * spending; neither is the funding model; and adding them would double-count.
 *
 * # The denominator, again
 *
 * This is the corpus's recurring warning arriving in a third place. These figures divide by
 * **unweighted ADM** — a headcount. The department's own headline spending figure divides by a
 * need-weighted count, and against a composition-driven outcome the two give −0.004 and −0.355
 * from the same numerator. So the basis is stated on the card rather than assumed, and the
 * headcount ADM is printed beside the figures it produced.
 *
 * # And the trap this data invites
 *
 * "Administration" is the number everyone reaches for, and it is small: general and school
 * administration together are typically under a tenth of operating spending. A reader arriving to
 * confirm that districts are administratively bloated will find the arithmetic does not support
 * it, which is worth showing plainly rather than burying — and equally worth not editorialising
 * into a claim that the composition is therefore correct.
 */

import type { Bar } from "./chart.ts";
import { count, escapeHtml, money, pct } from "./format.ts";
import { barSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import * as routes from "./routes.ts";
import type { District, OutcomeStatewide, SpendingByFunction } from "./types.ts";

/** The named functions, largest first, with what each covers. */
function functions(s: SpendingByFunction): { label: string; value: number; note?: string }[] {
  return [
    { label: "Instruction", value: s.instruction, note: "Teachers and classroom delivery." },
    {
      label: "Operations and maintenance",
      value: s.operations_maintenance,
      note: "Buildings, grounds, utilities, custodial.",
    },
    {
      label: "Pupil support",
      value: s.pupil_support,
      note: "Counselling, health, attendance, psychological services.",
    },
    { label: "Pupil transportation", value: s.pupil_transportation },
    {
      label: "School administration",
      value: s.school_admin,
      note: "Principals and building offices.",
    },
    {
      label: "Instructional staff support",
      value: s.instructional_staff_support,
      note: "Curriculum, libraries, professional development.",
    },
    {
      label: "General administration",
      value: s.general_admin,
      note: "The superintendent's and treasurer's offices, and the board.",
    },
    { label: "Other support", value: s.other_support },
    { label: "Food service", value: s.food_service },
  ].filter((f) => f.value > 0);
}

/** Render the function breakdown for one district. */
export function renderSpendingByFunction(d: District): string {
  const s = d.spending_by_function;
  if (!s) {
    return `<div class="card" data-part="spending-by-function">
      <h2>Where the money went</h2>
      <p class="note">No report-card spending row is published for this district, so its operating
        expenditure cannot be broken into functions here. It is one of two in the state.</p>
    </div>`;
  }

  const parts = functions(s);
  const total = s.operating_per_pupil;
  const bars: Bar[] = parts.map((part) => ({
    label: part.label,
    value: part.value,
    direct: pct(part.value / total, 0),
    hover: `${part.label}: ${money(part.value)} per pupil, ${pct(part.value / total, 1)} of operating spending`,
  }));

  const administration = s.general_admin + s.school_admin;

  return `
    <div class="card" data-part="spending-by-function">
      <h2>Where the money went, FY2025</h2>
      <div class="chartwrap" data-chart="functions">${renderToString(barSpec(bars))}</div>

      <div class="scroll"><table>
        <thead><tr><th>Function</th><th>Per pupil</th><th>Share</th></tr></thead>
        <tbody>
          <tr class="current">
            <th>Classroom instruction<div class="n">The department's roll-up.</div></th>
            <td class="tnum">${money(s.classroom_instruction)}</td>
            <td class="tnum">${pct(s.classroom_instruction / total, 1)}</td>
          </tr>
          <tr class="current">
            <th>Everything else<div class="n">The department's other roll-up. The two partition
              operating spending exactly.</div></th>
            <td class="tnum">${money(s.nonclassroom)}</td>
            <td class="tnum">${pct(s.nonclassroom / total, 1)}</td>
          </tr>
          ${parts
            .map(
              (part) => `<tr>
                <th>${escapeHtml(part.label)}${
                  part.note ? `<div class="n">${escapeHtml(part.note)}</div>` : ""
                }</th>
                <td class="tnum">${money(part.value)}</td>
                <td class="tnum n">${pct(part.value / total, 1)}</td>
              </tr>`,
            )
            .join("")}
          <tr class="current">
            <th>Total operating expenditure</th>
            <td class="tnum">${money(total)}</td>
            <td class="tnum">100.0%</td>
          </tr>
        </tbody>
      </table></div>

      <p class="note"><strong>Administration is ${pct(administration / total, 1)} of it</strong> —
        ${money(s.general_admin)} per pupil general and ${money(s.school_admin)} school. That is
        usually the first figure anyone looks for and it is usually small; the arithmetic does not
        support a story about administrative bloat, and it equally does not establish that the
        composition is right. It is a share, not a verdict.</p>

      <p class="note">These are the <strong>report card's</strong> FY2025 figures, divided by
        <strong>unweighted ADM</strong> — a headcount of
        ${count(Math.round(s.adm))} pupils, not the need-weighted count the department's headline
        per-pupil figure uses. The choice of denominator is not cosmetic:
        <a href="${routes.metric("expenditure-per-equivalent-pupil")}">the same numerator over a
        need-weighted count</a> behaves differently against every outcome measure. They are also
        not the <strong>audited actuals above</strong>, which are general fund totals from this
        district's own five-year forecast filing on a different basis — the two are not
        summable and neither is a check on the other.</p>
    </div>`;
}

/**
 * Where the money came from, in the one proportion that needs no denominator.
 *
 * # Why the share and not the dollars
 *
 * The report card splits its per-pupil spending figure by the origin of the money, and both parts
 * are per **need-weighted** pupil — a denominator this site has now been caught mixing up twice.
 * The ratio of two figures over the same denominator does not carry one. So the share leads, the
 * dollars follow with the basis named, and nothing here has to be reconciled against the function
 * breakdown above it, which divides by a headcount.
 *
 * # What it is for
 *
 * Every other figure on this site describes a decision Ohio made. This one describes exposure to
 * a decision Ohio does not make. Federal money is a median 4.2% of operating spending and 29.0%
 * at the top, so the same policy change reaching every district equally would not land equally.
 *
 * # And the correlation that has to be shown twice
 *
 * Federal education money is allocated substantially by poverty. So the raw association between
 * federal share and attainment is strongly negative — and says nothing, because it is the poverty
 * association wearing a different label. Holding poverty constant it nearly vanishes. Reporting
 * the raw figure alone would state a confound as a finding, which is the error this project keeps
 * finding in other people's work.
 */
export function renderFederalShare(d: District, statewide: OutcomeStatewide | null): string {
  const o = d.outcome;
  if (!o || o.per_equivalent_pupil_federal == null || o.per_equivalent_pupil == null) return "";
  const total = o.per_equivalent_pupil;
  if (total <= 0) return "";

  const federal = o.per_equivalent_pupil_federal;
  const stateLocal = o.per_equivalent_pupil_state_local ?? total - federal;
  const share = federal / total;

  const bars: Bar[] = [
    {
      label: "State and local",
      value: stateLocal,
      direct: pct(stateLocal / total, 0),
      hover: `State and local: ${money(stateLocal)} per equivalent pupil, ${pct(stateLocal / total, 1)}`,
    },
    {
      label: "Federal",
      value: federal,
      direct: pct(share, 0),
      hover: `Federal: ${money(federal)} per equivalent pupil, ${pct(share, 1)}`,
    },
  ];

  const median = statewide?.median_federal_share ?? 0;

  return `
    <div class="card" data-part="federal-share">
      <h2>Where the money came from, FY2025</h2>
      <div class="chartwrap" data-chart="origin">${renderToString(barSpec(bars))}</div>

      <div class="tiles">
        <div class="tile"><div class="k">Federal share of operating spending</div>
          <div class="v">${pct(share, 1)}</div>
          <div class="n">statewide median ${pct(median, 1)}</div></div>
        <div class="tile"><div class="k">Federal, per equivalent pupil</div>
          <div class="v">${money(federal)}</div>
          <div class="n">need-weighted count</div></div>
        <div class="tile"><div class="k">State and local</div>
          <div class="v">${money(stateLocal)}</div>
          <div class="n">need-weighted count</div></div>
      </div>

      <p class="note"><strong>${pct(share, 1)} of what this district spends on operations is
        federal money</strong>${
          share > median * 1.5
            ? ` — well above the statewide median of ${pct(median, 1)}.`
            : share < median * 0.6
              ? ` — well below the statewide median of ${pct(median, 1)}.`
              : `, against a statewide median of ${pct(median, 1)}.`
        } Statewide the figure runs from under a percent to
        ${statewide ? pct(statewide.max_federal_share, 1) : "29%"}, and
        ${statewide ? count(statewide.federal_share_above_tenth) : "49"} districts are above a
        tenth. It is the only figure on this site that measures exposure to a decision made
        outside Ohio, and the districts most exposed are not the ones with the most money.</p>

      <p class="note">The share is the reliable number here and the dollars are the awkward ones.
        Both parts are published <strong>per need-weighted pupil</strong> — the department's
        denominator, not the headcount the function breakdown above uses — so the two cards do not
        add and their per-pupil figures are not comparable. A ratio taken within one denominator
        survives that; a subtraction across two does not.</p>

      ${
        statewide
          ? `<p class="note">Across the state, federal share and the Performance Index correlate
             at <strong>${statewide.federal_share_vs_performance_raw.toFixed(3)}</strong>. That
             number is worth almost nothing on its own: federal education money is allocated
             substantially by poverty, so it is largely the poverty relationship read backwards.
             <strong>Holding poverty constant it is
             ${statewide.federal_share_vs_performance.toFixed(3)}.</strong> Neither figure
             identifies an effect in either direction, and the gap between them is the whole
             reason both are printed.</p>`
          : ""
      }
    </div>`;
}
