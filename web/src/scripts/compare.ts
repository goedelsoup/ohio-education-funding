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
import { anchor } from "../lib/section.ts";
import { saying } from "../lib/status.ts";

/**
 * A series' label from the panel the browser fetched, or nothing if the feed omits the block.
 *
 * # Why this is four lines here rather than an import from `lib/year.ts`
 *
 * Because `year.ts` imports `loadFeed`, and `feed.ts` opens `node:fs`. Pulling it into this file
 * put the build-time feed reader into a browser bundle and the comparison table stopped rendering
 * altogether — caught by `comparison puts two districts side by side`, which is the only reason it
 * did not ship.
 *
 * `corpus.ts` carries the same warning in its docstring — "must never be imported by client code"
 * — and `year.ts` now carries it too. The lookup itself is a `find` over ten rows; sharing it is
 * not worth a module that has to stay browser-safe forever.
 */
function yearOf(panel: Panel, series: string): string {
  return panel.series_years.find((entry) => entry.series === series)?.label ?? "";
}

/**
 * A millage row's label, with the tax year it is measured in spelled out.
 *
 * `2024 tax year` in full rather than a bare `2024`, which is the rule `lib/year.ts` sets for the
 * chips and states the reason for: `FY2024` and `2024` differ by a prefix, and a prefix reads as
 * typography rather than as a claim about which eleven-month-offset period a figure covers.
 *
 * The label falls back to the bare name where the feed carries no millage block. A row saying
 * nothing about its year is what this is fixing, but a row saying "undefined tax year" is worse.
 */
function taxYearLabel(panel: Panel, name: string): string {
  const year = yearOf(panel, "millage");
  return year ? `${name}, ${year} tax year` : name;
}

const a = document.querySelector<HTMLSelectElement>("#cmp-a");
const b = document.querySelector<HTMLSelectElement>("#cmp-b");
const out = document.querySelector<HTMLElement>("#compare-out");
/*
 * What the page says once the reader has stopped changing which districts it is about.
 *
 * `out.innerHTML` below replaces the whole comparison, and nothing announced that it had. See
 * `src/lib/status.ts` for why the region is a separate sentence rather than `aria-live` on the
 * table itself.
 */
const changed = document.querySelector<HTMLElement>("#changed");
const say = changed ? saying(changed) : () => {};

/** One comparable quantity, with the formatter that makes it readable. */
interface Row {
  /**
   * A function where the label names a year, a string where it does not.
   *
   * `ROWS` is built at module load and the panel arrives over the network, so a row that wants to
   * say "FY2024 to FY2026" cannot read it at construction time. It said exactly that as a literal
   * until the year loop; the alternative to laziness here is a label that goes stale the next time
   * the calculator advances.
   */
  label: string | ((panel: Panel) => string);
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
    // Beside the effective rate rather than instead of it: the two are only interesting as a
    // pair, and the gap between them is what H.B. 920 has taken.
    //
    // Both carry the tax year they are measured in, and it is the only row family here that has
    // to say the *reckoning* as well as the digits. The footnote below the table names three
    // years — the fiscal year of the model, the profile year of the valuations, the fiscal year
    // of the expenditure — and these two rows were on a fourth, silently. A bare `2024` beside
    // `FY2027` in the same table invites exactly the reading `lib/year.ts` exists to prevent, so
    // the words go in the label where there is room for them.
    label: (panel) => taxYearLabel(panel, "Voted operating millage"),
    pick: (d) => d.voted_operating_millage,
    format: (v) => (v == null ? "—" : v.toFixed(2)),
  },
  {
    label: (panel) => taxYearLabel(panel, "Effective Class 1 millage"),
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
  {
    label: (panel) => `Enrollment change, ${yearOf(panel, "enrollment").replace("-", "–")}`,
    pick: (d) => d.enrollment_change,
    format: (v) => pct(v, 1),
  },
];

const FLAGS: { label: string; pick: (d: PanelDistrict) => boolean }[] = [
  { label: "On the guarantee", pick: (d) => d.on_guarantee },
  { label: "At or below the 20-mill floor", pick: (d) => d.at_millage_floor },
  { label: "Within a twentieth of a mill of it", pick: (d) => d.near_millage_floor },
  { label: "At the minimum state share", pick: (d) => d.at_minimum_state_share },
];

/**
 * The runtime twin of `qualifiedName` in `src/lib/feed.ts`.
 *
 * Three districts are called Green Local. Comparing two of them put the same string in both
 * column heads, which is the one table where the heads are the entire distinction between the
 * columns. The panel already carries `county`, so this asks the same question of it that the
 * build asks of the bundle, and the two answer alike because both qualify only on a repeat.
 */
let ambiguous: Set<string> | null = null;
function qualified(panel: Panel, d: PanelDistrict): string {
  if (!ambiguous) {
    const counts = new Map<string, number>();
    for (const other of panel.districts) counts.set(other.name, (counts.get(other.name) ?? 0) + 1);
    ambiguous = new Set([...counts].filter(([, n]) => n > 1).map(([name]) => name));
  }
  return ambiguous.has(d.name) ? `${d.name}, ${d.county} County` : d.name;
}

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
    const text = typeof row.label === "function" ? row.label(panel) : row.label;
    const label = row.href ? `<a href="${row.href}">${escapeHtml(text)}</a>` : escapeHtml(text);
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
    <th><a href="${routes.district(left.irn)}">${escapeHtml(qualified(panel, left))}</a></th>
    <th><a href="${routes.district(right.irn)}">${escapeHtml(qualified(panel, right))}</a></th>
    <th>Difference</th>
  </tr>`;

  out.innerHTML = `
    <div class="card" id="comparison" data-part="comparison">
      <h2>${anchor("comparison")}Side by side</h2>
      <div class="scroll"><table>
        <thead>${head}</thead>
        <tbody>${rows}${flags}</tbody>
      </table></div>
      <p class="note">Nothing in this table is coloured good or bad. Higher state aid per pupil and
        higher property wealth point in opposite directions, and which figure counts as favourable
        depends on the argument being made — which is the reason to look at two districts rather
        than one. FY${panel.fiscal_year} model; valuation is ${yearOf(panel, "profile")} and
        expenditure ${yearOf(panel, "outcome.spending")}.</p>
    </div>`;

  /*
   * And say which two districts the table is now about.
   *
   * The names rather than a tile summary, because this route has no tiles: the whole result is one
   * table of paired figures, and "the comparison is now Cleveland against Northern Local" is the
   * sentence a reader needs before they go and read it. The rest is in the table, which is a
   * keyboard region of its own.
   */
  say(`Comparison updated: ${qualified(panel, left)} against ${qualified(panel, right)}.`);
}

async function start(): Promise<void> {
  if (!a || !b || !out) return;
  const response = await fetch(`${import.meta.env.BASE_URL}data/panel.json`);
  if (!response.ok) {
    out.innerHTML = `<div class="card err" id="panel-unreachable" data-part="panel-unreachable"><p>Could not load the panel (HTTP ${response.status}).</p></div>`;
    return;
  }
  const panel = (await response.json()) as Panel;
  const byIrn = new Map(panel.districts.map((d) => [d.irn, d]));

  const params = new URLSearchParams(location.search);
  /*
   * Defaults are the corpus's own contrast pair, so an empty `/compare` still shows the argument
   * the page exists to make rather than a district against itself.
   *
   * Either side may arrive alone: every district page links here with its own IRN and no partner,
   * which is the entry point that makes this route reachable at all. So the side that was not
   * given falls back to whichever half of the pair the given one is *not* — otherwise a reader
   * arriving from Northern Local's page lands on Northern Local against Northern Local, a table
   * of "the same" in every row.
   */
  const POOR = "049056";
  const RICH = "044933";
  a.value = params.get("a") ?? (params.get("b") === POOR ? RICH : POOR);
  b.value = params.get("b") ?? (a.value === RICH ? POOR : RICH);
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

/*
 * Keep Enter from reloading the page.
 *
 * This was `onsubmit="return false"` on the form itself, which `script-src 'self'` blocks — an
 * inline event handler is inline script. The violation only appears where the CSP is actually
 * applied, which is the deployed site and never `vite preview`, so it shipped. See the built-output
 * check in `tests/e2e/`.
 */
document
  .querySelector<HTMLFormElement>("#compare-form")
  ?.addEventListener("submit", (event) => event.preventDefault());
