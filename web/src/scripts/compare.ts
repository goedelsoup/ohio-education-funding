/**
 * Two districts, side by side.
 *
 * Reads the slim panel — the formula inputs, without the finances and report-card blocks — and
 * renders a difference table. The pair lives in the query string so a comparison can be sent.
 *
 * # One rule about the difference column
 *
 * Every row states which direction is which in words as well as in colour, and no row is coloured
 * "good" or "bad". More money per pupil is not automatically better and higher property wealth is
 * not automatically worse; the whole point of the corpus's contrast pairs is that the same figure
 * means opposite things at the two ends of the distribution. So the difference is signed and
 * neutral, and the reader supplies the judgement.
 */

import { money, pct } from "../lib/format.ts";
import { currentFormulaAid, currentRealizedAid } from "../lib/policy.ts";
import * as routes from "../lib/routes.ts";
import type { Panel, PanelDistrict } from "../lib/types.ts";

const a = document.querySelector<HTMLSelectElement>("#cmp-a");
const b = document.querySelector<HTMLSelectElement>("#cmp-b");
const out = document.querySelector<HTMLElement>("#compare-out");

/** One comparable quantity, with the formatter that makes it readable. */
interface Row {
  label: string;
  pick: (d: PanelDistrict) => number | null;
  format: (v: number | null) => string;
  /** A ratio reads better than a subtraction for wealth, which spans two orders of magnitude. */
  ratio?: boolean;
  href?: string;
}

const ROWS: Row[] = [
  { label: "Enrolled ADM (base cost)", pick: (d) => d.adm, format: (v) => (v == null ? "—" : Math.round(v).toLocaleString("en-US")) },
  { label: "State aid per pupil", pick: (d) => d.realized_aid_per_pupil, format: (v) => money(v) },
  { label: "Formula aid per pupil", pick: (d) => d.formula_aid_per_pupil, format: (v) => money(v) },
  { label: "State aid, total", pick: (d) => currentRealizedAid(d), format: (v) => money(v) },
  { label: "Formula aid, total", pick: (d) => currentFormulaAid(d), format: (v) => money(v) },
  { label: "Guarantee, total", pick: (d) => d.guarantee, format: (v) => money(v) },
  { label: "Base cost per pupil", pick: (d) => d.base_cost_per_pupil, format: (v) => money(v) },
  {
    label: "Assessed valuation per pupil",
    pick: (d) => d.valuation_per_pupil,
    format: (v) => money(v),
    ratio: true,
    href: routes.metric("assessed-valuation-per-pupil"),
  },
  {
    label: "Effective Class 1 millage",
    pick: (d) => d.effective_class1_millage,
    format: (v) => (v == null ? "—" : v.toFixed(2)),
    href: routes.metric("effective-operating-millage"),
  },
  {
    label: "Operating expenditure per pupil",
    pick: (d) => d.operating_expenditure_per_pupil,
    format: (v) => money(v),
    href: routes.metric("per-pupil-operating-expenditure"),
  },
  { label: "Economically disadvantaged", pick: (d) => d.economically_disadvantaged, format: (v) => pct(v, 1) },
  { label: "Enrollment change, FY2024–FY2026", pick: (d) => d.enrollment_change, format: (v) => pct(v, 1) },
];

const FLAGS: { label: string; pick: (d: PanelDistrict) => boolean }[] = [
  { label: "On the guarantee", pick: (d) => d.on_guarantee },
  { label: "At the 20-mill floor", pick: (d) => d.at_millage_floor },
  { label: "At the minimum state share", pick: (d) => d.at_minimum_state_share },
];

function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!);
}

function difference(row: Row, left: number | null, right: number | null): string {
  if (left == null || right == null) return `<span class="n">not comparable</span>`;
  if (row.ratio && right !== 0 && left !== 0) {
    const times = left / right;
    if (times >= 1) return `${times.toFixed(1)}× the second`;
    return `${(1 / times).toFixed(1)}× the first`;
  }
  const delta = left - right;
  if (Math.abs(delta) < 1e-9) return "the same";
  const sign = delta > 0 ? "+" : "−";
  return `${sign}${row.format(Math.abs(delta)).replace(/^[+−]/, "")} to the first`;
}

function render(panel: Panel, left: PanelDistrict, right: PanelDistrict): void {
  if (!out) return;
  const rows = ROWS.map((row) => {
    const l = row.pick(left);
    const r = row.pick(right);
    const label = row.href
      ? `<a href="${row.href}">${escapeHtml(row.label)}</a>`
      : escapeHtml(row.label);
    return `<tr>
      <th>${label}</th>
      <td class="tnum">${row.format(l)}</td>
      <td class="tnum">${row.format(r)}</td>
      <td class="n">${difference(row, l, r)}</td>
    </tr>`;
  }).join("");

  const flags = FLAGS.map(
    (flag) => `<tr>
      <th>${escapeHtml(flag.label)}</th>
      <td>${flag.pick(left) ? "yes" : "no"}</td>
      <td>${flag.pick(right) ? "yes" : "no"}</td>
      <td class="n">${flag.pick(left) === flag.pick(right) ? "same" : "differs"}</td>
    </tr>`,
  ).join("");

  const head = `<tr>
    <th></th>
    <th><a href="${routes.district(left.irn)}">${escapeHtml(left.name)}</a></th>
    <th><a href="${routes.district(right.irn)}">${escapeHtml(right.name)}</a></th>
    <th>Difference</th>
  </tr>`;

  out.innerHTML = `
    <div class="card">
      <h2>Side by side</h2>
      <div class="scroll"><table>
        <thead>${head}</thead>
        <tbody>${rows}${flags}</tbody>
      </table></div>
      <p class="note">Nothing in this table is coloured good or bad. Higher state aid per pupil and
        higher property wealth point in opposite directions, and which figure counts as favourable
        depends on the argument being made — which is the reason to look at two districts rather
        than one. FY${panel.fiscal_year} model; valuation is TY2023 and expenditure FY2024.</p>
    </div>`;
}

async function start(): Promise<void> {
  if (!a || !b || !out) return;
  const response = await fetch(`${import.meta.env.BASE_URL}data/panel.json`);
  if (!response.ok) {
    out.innerHTML = `<div class="card err"><p>Could not load the panel (HTTP ${response.status}).</p></div>`;
    return;
  }
  const panel = (await response.json()) as Panel;
  const byIrn = new Map(panel.districts.map((d) => [d.irn, d]));

  const params = new URLSearchParams(location.search);
  // Defaults are the corpus's own contrast pair, so an empty `/compare` still shows the argument
  // the page exists to make rather than a district against itself.
  a.value = params.get("a") ?? "049056";
  b.value = params.get("b") ?? "044933";
  if (!byIrn.has(a.value)) a.value = panel.districts[0]!.irn;
  if (!byIrn.has(b.value)) b.value = panel.districts[1]?.irn ?? panel.districts[0]!.irn;

  const update = () => {
    const left = byIrn.get(a.value);
    const right = byIrn.get(b.value);
    if (!left || !right) return;
    const next = `${location.pathname}?a=${left.irn}&b=${right.irn}`;
    if (location.pathname + location.search !== next) history.replaceState(null, "", next);
    render(panel, left, right);
  };

  a.addEventListener("change", update);
  b.addEventListener("change", update);
  update();
}

void start();
