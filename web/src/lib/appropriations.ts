/**
 * What the General Assembly set aside for the schools, FY2002 to FY2027.
 *
 * # Why this belongs on the site at all
 *
 * Everything else here is an output of the funding system: what the formula computes, what a
 * district was paid, what its pupils achieved. This is the input — the ceiling the legislature
 * writes before any of it runs.
 *
 * # Why it cannot be shown in nominal dollars
 *
 * "Record investment in education" is a true sentence about nearly every biennium in this series
 * and an empty one. The appropriation roughly doubles across twenty-four years and CPI-U roughly
 * doubles with it, so the nominal series and the real series do not merely differ in precision:
 * one shows sustained growth and the other shows a line that gives most of it back.
 *
 * The card does not privilege either basis to make that point — `BasisToggle` is symmetric on
 * purpose, and its own note explains why neither is the default truth. What carries the point is
 * the prose: whichever basis a reader lands on, the card says plainly that the other one tells a
 * different story, and that both sentences are true.
 *
 * # What a total here is not
 *
 * Not a payment. An appropriation is a ceiling and the formula's own proration factor exists
 * because at least one line has been a residual claimant — so this and the payment reports differ
 * for ordinary reasons, and the difference between them is not a finding.
 *
 * Not per pupil, and deliberately never divided into one. A statewide appropriation over a pupil
 * count would be a per-pupil figure on a denominator nothing else here uses, sitting one division
 * away from the formula's own per-pupil numbers. See `denominators.ts`.
 */

import { escapeHtml, money, pct } from "./format.ts";
import { seriesSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import { convert, type Basis } from "./real.ts";
import type { AppropriationYear, Deflator } from "./types.ts";

/** The years only the Catalog of Budget Line Items reaches. */
export function fromCatalog(rows: AppropriationYear[]): number[] {
  return rows.filter((r) => r.source === "catalog").map((r) => r.fiscal_year);
}

/**
 * The series in `base` dollars, dropping any year the index cannot reach.
 *
 * Dropping rather than falling back to nominal, on the rule the history route already follows: a
 * real series with one un-deflated point in it is wrong in a way nothing on the page would show.
 * FY2027 is the year this loses — the index cannot reach a June that has not happened.
 */
export function inBase(
  rows: AppropriationYear[],
  deflator: Deflator | null,
  base: number,
  basis: Basis,
): AppropriationYear[] {
  if (basis === "nominal") return rows;
  if (!deflator) return [];
  return rows.flatMap((row) => {
    const enacted = convert(deflator, row.enacted, row.fiscal_year, base);
    const foundation = convert(deflator, row.foundation_funding, row.fiscal_year, base);
    if (enacted == null || foundation == null) return [];
    return [{ ...row, enacted, foundation_funding: foundation }];
  });
}

/** Growth from the first year to the last, as a multiple. */
export function growth(rows: AppropriationYear[]): number | null {
  const first = rows[0];
  const last = rows[rows.length - 1];
  if (!first || !last || first.enacted <= 0) return null;
  return last.enacted / first.enacted;
}

/** The appropriation, year by year, on one basis. */
export function renderAppropriations(
  rows: AppropriationYear[],
  deflator: Deflator | null,
  base: number | null,
  basis: Basis,
): string {
  if (rows.length < 2) return "";
  if (basis === "real" && base == null) return "";

  const shown = inBase(rows, deflator, base ?? 0, basis);
  if (shown.length < 2) return "";

  const first = shown[0]!;
  const last = shown[shown.length - 1]!;
  const multiple = growth(shown);
  const dropped = rows.length - shown.length;
  const catalogYears = fromCatalog(rows);

  const chart = renderToString(
    seriesSpec(
      shown.map((r) => ({
        year: r.fiscal_year,
        a: r.enacted / 1e9,
        b: r.foundation_funding / 1e9,
      })),
      { a: "all lines", b: "the formula" },
      (v) => `$${v.toFixed(0)}bn`,
      (p) =>
        `FY${p.year}: $${(p.a ?? 0).toFixed(2)}bn appropriated, $${(p.b ?? 0).toFixed(2)}bn of it the formula`,
    ),
  );

  const label = basis === "real" ? `constant FY${base} dollars` : "the dollars of each year";

  return `
    <div class="card" data-part="appropriations">
      <h2>What the legislature set aside</h2>
      <p class="note">Every appropriation line the Department of Education and Workforce was
        given, FY${first.fiscal_year} through FY${last.fiscal_year}, in ${escapeHtml(label)}. The
        upper line is the department's whole appropriation; the lower is the two lines the funding
        formula itself is paid from. Property tax reimbursement lines are excluded — they are
        numbered as the department's and are not its budget.</p>

      <div class="scroll">${chart}</div>

      <p class="note">${
        multiple == null
          ? ""
          : `Across this window the appropriation ${multiple >= 1.05 ? `grew ${multiple.toFixed(2)}×` : multiple <= 0.95 ? `fell to ${multiple.toFixed(2)}× its starting value` : "did not materially change"} in ${escapeHtml(label)}.`
      } The same series on the other basis tells a different story, which is the reason this card
        carries the switch: a nominal total that rises every biennium is compatible with a real
        total that does not move, and both sentences are true.${
          dropped > 0
            ? ` ${dropped} ${dropped === 1 ? "year is" : "years are"} absent here because the price index does not reach ${dropped === 1 ? "it" : "them"}.`
            : ""
        }</p>

      <div class="scroll"><table>
        <thead><tr><th>Year</th><th class="tnum">Appropriated</th>
          <th class="tnum">The formula's share</th><th class="tnum">Lines</th>
          <th>Source</th></tr></thead>
        <tbody>${shown
          .map(
            (r) => `<tr>
              <th>FY${r.fiscal_year}</th>
              <td class="tnum">${money(r.enacted)}</td>
              <td class="tnum n">${pct(r.foundation_funding / r.enacted, 0)}</td>
              <td class="tnum n">${r.items}</td>
              <td>${r.source === "catalog" ? "Catalog" : "Greenbook"}</td>
            </tr>`,
          )
          .join("")}</tbody>
      </table></div>

      <p class="note">Two publications answer for this series.
        ${
          catalogYears.length === 0
            ? ""
            : `FY${catalogYears.join(", FY")} come from the Catalog of Budget Line Items and the
        rest from LSC's greenbooks and budget workbooks — those are the years the workbooks cannot
        reach, one biennium because its greenbook has no line-item table at all and the other
        because the two workbook variants are served as the same file.`
        }
        Where both publications state a year, they agree to the cent across seventeen hundred
        line items, so the column says which document a figure came from rather than how much to
        trust it.</p>
    </div>`;
}
