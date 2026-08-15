/**
 * What the department's budget is made of, and how old each piece is.
 *
 * # The finding
 *
 * The card above this one shows what the legislature set aside. This shows what the setting-aside
 * consists of: about ninety appropriation lines, created by acts spanning half a century. The
 * oldest still-live line predates every funding regime this corpus documents — including the one
 * struck down in *DeRolph*.
 *
 * That is a fact about how budgets are actually made. Nobody designed this list. Each General
 * Assembly added to what it inherited, and what a reader meets in FY2027 is the accumulation.
 *
 * # Half the lines answer "unknown", and that is not a gap in the extraction
 *
 * The Catalog prints a legal basis for every line and only sometimes an `originally established
 * by` clause; the rest cite current authority alone. Those are shown as unknown rather than
 * inferred from an earlier edition carrying the same number, because line item numbers are reused
 * — `200604` names three different programmes across three funds in this series. Inheriting an
 * origin down a number would attribute one programme's founding act to another's, and the result
 * would look complete.
 *
 * # Why the discontinued lines are here at all
 *
 * Because the Catalog still lists them and the corpus has an open question about them: the
 * department went from 109 appropriation lines to 79 across FY2014-FY2026, and whether the
 * missing ones were abolished or folded into others is unsettled. `discontinued` is the
 * publisher's label, not an answer — a line folded into another is discontinued too. The card
 * says so rather than letting the flag imply more than it carries.
 */

import { escapeHtml } from "./format.ts";
import type { AppropriationLine } from "./types.ts";
import { yearChip } from "./year.ts";

/** Live lines, oldest establishing act first; undated lines last. */
export function byAge(lines: AppropriationLine[]): AppropriationLine[] {
  return [...lines]
    .filter((l) => !l.discontinued)
    .sort((a, b) => {
      if (a.general_assembly == null && b.general_assembly == null) return a.ali < b.ali ? -1 : 1;
      if (a.general_assembly == null) return 1;
      if (b.general_assembly == null) return -1;
      if (a.general_assembly !== b.general_assembly) return a.general_assembly - b.general_assembly;
      return a.ali < b.ali ? -1 : 1;
    });
}

/** The span of General Assemblies the live, dated lines come from. */
export function span(lines: AppropriationLine[]): { oldest: number; newest: number } | null {
  const years = lines
    .filter((l) => !l.discontinued && l.convened != null)
    .map((l) => l.convened as number);
  if (years.length === 0) return null;
  return { oldest: Math.min(...years), newest: Math.max(...years) };
}

/** An ordinal in the form the Catalog writes it: `112th`, `121st`, `133rd`. */
export function ordinal(n: number): string {
  const two = n % 100;
  if (two >= 11 && two <= 13) return `${n}th`;
  return `${n}${["th", "st", "nd", "rd"][n % 10] ?? "th"}`;
}

/** The lines the department is funded through, and where each came from. */
export function renderLineOrigins(lines: AppropriationLine[]): string {
  if (lines.length === 0) return "";

  const live = byAge(lines);
  const dated = live.filter((l) => l.general_assembly != null);
  const undated = live.length - dated.length;
  const discontinued = lines.length - live.length;
  const reach = span(lines);
  const oldest = dated[0];

  return `
    <div class="card" data-part="line-origins">
      <h2>What the budget is made of${yearChip("appropriations")}</h2>
      <p class="note">The Department of Education and Workforce is funded through
        ${live.length} live appropriation lines${
          discontinued > 0
            ? `, beside ${discontinued} the Catalog still lists as discontinued`
            : ""
        }. Each was created by an act, and the Catalog names it${
          reach == null
            ? "."
            : ` — the ones that do span ${reach.newest - reach.oldest} years, from ${reach.oldest} to ${reach.newest}.`
        }</p>

      ${
        oldest == null
          ? ""
          : `<p class="note">The oldest line still being funded is
        <strong>${escapeHtml(oldest.name)}</strong>, established by
        ${escapeHtml(oldest.established_by)} — a General Assembly that convened in
        ${oldest.convened}, two decades before <em>DeRolph</em> was decided and four before the
        Fair School Funding Plan. Nobody designed this list; each legislature added to what it
        inherited.</p>`
      }

      <div class="scroll"><table>
        <thead><tr><th class="tnum">Line</th><th>Name</th><th>Fund</th>
          <th>Established by</th><th class="tnum">Convened</th></tr></thead>
        <tbody>${live
          .map(
            (l) => `<tr>
              <th class="tnum n">${escapeHtml(l.ali)}</th>
              <td>${escapeHtml(l.name)}</td>
              <td class="n">${escapeHtml(l.fund)}</td>
              <td>${l.established_by === "" ? '<span class="n">not stated</span>' : escapeHtml(l.established_by)}</td>
              <td class="tnum n">${l.convened ?? "—"}</td>
            </tr>`,
          )
          .join("")}</tbody>
      </table></div>

      <p class="note">${undated} of these lines name no establishing act. The Catalog gives every
        line a legal basis and only sometimes says which act created it; where it does not, this
        says nothing rather than reading an origin off an earlier edition with the same number.
        Line item numbers are reused — one number in this series names three different programmes
        across three funds — so inheriting an origin down a number would attribute one programme's
        founding act to another's, and the table would look complete.</p>

      <p class="note">A discontinued line is the publisher's own label and not a finding about
        abolition: a line folded into another is discontinued too. Whether the department's
        disappearing lines were abolished or consolidated is an open question this cannot settle.</p>
    </div>`;
}
