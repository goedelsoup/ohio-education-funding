/**
 * The panel as one CSV row per district.
 *
 * The form most people actually want. A journalist checking a claim about the guarantee, or
 * legislative staff sorting by exposure to a phase-in, needs a table and a spreadsheet — not 609
 * web pages and not a nested JSON document.
 *
 * Every column is a figure the site already shows, so this cannot disagree with the pages: both
 * come out of the same feed in the same build.
 */

import type { APIRoute } from "astro";

import { loadFeed } from "../../lib/feed.ts";
import { currentFormulaAid, currentRealizedAid } from "../../lib/policy.ts";

/** RFC 4180: quote anything containing a comma, a quote, or a newline; double the quotes. */
function cell(value: string | number | boolean | null | undefined): string {
  if (value == null) return "";
  const text = String(value);
  return /[",\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
}

const COLUMNS = [
  "irn",
  "name",
  "adm_base_cost",
  "adm_current_year",
  "base_cost_per_pupil",
  "aggregate_base_cost",
  "base_cost_state_share",
  "categorical_funding",
  "formula_aid_total",
  "realized_aid_total",
  "formula_aid_per_pupil",
  "realized_aid_per_pupil",
  "guarantee_total",
  "on_guarantee",
  "at_millage_floor",
  "near_millage_floor",
  "at_minimum_state_share",
  "valuation_per_pupil",
  "voted_operating_millage",
  "effective_class1_millage",
  "millage_reduced_away",
  "yield_per_mill_per_pupil",
  "class1_rate_predicted",
  "class1_rate_residual",
  "operating_expenditure_per_pupil",
  "economically_disadvantaged",
  "enrollment_change_fy24_fy26",
  "performance_index",
  "progress_effect_size",
  "spending_per_enrolled_pupil",
  "spending_per_equivalent_pupil",
  "latest_closed_fiscal_year",
  "latest_state_aid",
  "latest_local_tax",
  "latest_total_revenue",
  "latest_total_expenditure",
  "latest_ending_cash",
] as const;

export const GET: APIRoute = () => {
  const { bundle, alphabetical } = loadFeed();

  const rows = alphabetical.map((d) => {
    const latest = d.finances[d.finances.length - 1];
    const o = d.outcome;
    return [
      d.irn,
      d.name,
      d.adm,
      d.current_year_adm,
      d.base_cost_per_pupil,
      d.aggregate_base_cost,
      d.base_cost_state_share,
      d.categorical_funding,
      // Summed from the components, as everywhere else on this site: the per-pupil figures are
      // published on base cost enrolled ADM, so multiplying one by the current-year count is a
      // different quantity rather than a rounding difference.
      currentFormulaAid(d),
      currentRealizedAid(d),
      d.formula_aid_per_pupil,
      d.realized_aid_per_pupil,
      d.guarantee,
      d.on_guarantee,
      d.at_millage_floor,
      d.near_millage_floor,
      d.at_minimum_state_share,
      d.valuation_per_pupil,
      d.voted_operating_millage,
      d.effective_class1_millage,
      d.millage?.cumulative_reduction,
      d.millage?.yield_per_mill_per_pupil,
      d.millage?.predicted_rate,
      d.millage?.residual,
      d.operating_expenditure_per_pupil,
      d.economically_disadvantaged,
      d.enrollment_change,
      o?.performance_index,
      o?.progress_effect_size,
      o?.per_enrolled_pupil,
      o?.per_equivalent_pupil,
      latest?.fiscal_year,
      latest?.state_aid,
      latest?.local_tax,
      latest?.total_revenue,
      latest?.total_expenditure,
      latest?.ending_cash,
    ].map(cell);
  });

  // A comment line before the header would break every naive parser, so the provenance travels in
  // the filename and on `/data` instead. What goes in the file is the table.
  const csv = [COLUMNS.join(","), ...rows.map((r) => r.join(","))].join("\n") + "\n";

  return new Response(csv, {
    headers: {
      "content-type": "text/csv; charset=utf-8",
      "content-disposition": `attachment; filename="ohio-school-funding-fy${bundle.fiscal_year}.csv"`,
    },
  });
};
