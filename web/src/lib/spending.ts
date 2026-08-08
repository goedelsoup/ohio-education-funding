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
import type { District, SpendingByFunction } from "./types.ts";

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
    return `<div class="card">
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
    <div class="card">
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
