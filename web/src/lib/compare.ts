/**
 * Two districts, side by side — the table itself, authored once for both renderers.
 *
 * # Why this left `scripts/compare.ts`
 *
 * The route used to download the whole 609-district panel to render seventeen rows about two of
 * them, and shipped an empty `#compare-out` until it landed. Measured at 400 Kbps / 400 ms RTT:
 * first paint at 1,284 ms, `panel.json` finishing at **4,131 ms**, table at 4,340 ms. Three
 * seconds of an empty box, for two districts' figures. That is the last open half of #111.
 *
 * The fix is that the *page* renders the table and the browser only changes it. So the row
 * definitions live here, where `compare.astro` can call them at build time and the client script
 * can call the same ones afterwards.
 *
 * # One rule about the difference column
 *
 * Every row states which direction is which in words as well as in colour, and no row is coloured
 * "good" or "bad". More money per pupil is not automatically better and higher property wealth is
 * not automatically worse; the whole point of the corpus's contrast pairs is that the same figure
 * means opposite things at the two ends of the distribution. So the difference is signed and
 * neutral, and the reader supplies the judgement.
 *
 * # Labels are build-time and values are not
 *
 * [`Row.label`] takes {@link Years} and is called only by the build. A district swap changes no
 * label — the tax year of the millage rows and the span of the enrollment row are properties of
 * the *feed*, not of which two districts are shown — so the browser rewrites value cells and
 * column heads and leaves every `<th>` alone. That is what lets the client carry no year data at
 * all, and it is why {@link Years} names its four series rather than taking a lookup function: a
 * row that wants a fifth year has to add it here, where `web/src/lib/year.ts`'s rule that nobody
 * hand-writes a year label can still see it.
 */

import { money, pct } from "./format.ts";
import { currentFormulaAid, currentRealizedAid } from "./policy.ts";
import * as routes from "./routes.ts";
import { anchor } from "./section.ts";
import type { PanelDistrict } from "./types.ts";

/**
 * The years the table's labels and footnote name, read from the feed at build time.
 *
 * Four series and the model's own year, spelled out rather than looked up, because a table whose
 * rows silently sit on four different reckonings is the failure `lib/year.ts` exists to prevent.
 */
export interface Years {
  /** The fiscal year of the funding model the aid rows come from. */
  fiscal: number;
  /** The tax year the two millage rows are measured in. */
  millage: string;
  /** The span the enrollment-change row covers. */
  enrollment: string;
  /** The profile-report year the valuation row is measured in. */
  profile: string;
  /** The year of the operating-expenditure row. */
  spending: string;
}

/** One comparable quantity, with the formatter that makes it readable. */
export interface Row {
  /** Stable across renders: the client finds a row's cells by it rather than by position. */
  key: string;
  label: (years: Years) => string;
  pick: (d: PanelDistrict) => number | null;
  format: (v: number | null) => string;
  /** A ratio reads better than a subtraction for wealth, which spans two orders of magnitude. */
  ratio?: boolean;
  href?: string;
}

export const ROWS: Row[] = [
  {
    key: "adm",
    label: () => "Enrolled ADM (base cost)",
    pick: (d) => d.adm,
    format: (v) => (v == null ? "—" : Math.round(v).toLocaleString("en-US")),
  },
  { key: "realized-per-pupil", label: () => "State aid per pupil", pick: (d) => d.realized_aid_per_pupil, format: (v) => money(v) },
  { key: "formula-per-pupil", label: () => "Formula aid per pupil", pick: (d) => d.formula_aid_per_pupil, format: (v) => money(v) },
  { key: "realized-total", label: () => "State aid, total", pick: (d) => currentRealizedAid(d), format: (v) => money(v) },
  { key: "formula-total", label: () => "Formula aid, total", pick: (d) => currentFormulaAid(d), format: (v) => money(v) },
  { key: "guarantee", label: () => "Guarantee, total", pick: (d) => d.guarantee, format: (v) => money(v) },
  { key: "base-cost", label: () => "Base cost per pupil", pick: (d) => d.base_cost_per_pupil, format: (v) => money(v) },
  {
    /*
     * Dated in the row header, because the card's chip cannot carry it.
     *
     * This table sits on three reckonings — the model's fiscal year, the profile report's, and the
     * report card's spending year — and `yearChipPair` names two. Naming two and letting the third
     * be read off one of them is precisely the failure `lib/year.ts` exists to stop, so the odd one
     * out says so where the millage rows already say theirs.
     */
    key: "valuation",
    label: (y) => (y.profile ? `Assessed valuation per pupil, ${y.profile}` : "Assessed valuation per pupil"),
    pick: (d) => d.valuation_per_pupil,
    format: (v) => money(v),
    ratio: true,
    href: routes.metric("assessed-valuation-per-pupil"),
  },
  {
    /*
     * Beside the effective rate rather than instead of it: the two are only interesting as a
     * pair, and the gap between them is what H.B. 920 has taken.
     *
     * Both carry the tax year they are measured in, and it is the only row family here that has
     * to say the *reckoning* as well as the digits. The footnote below the table names three
     * years — the fiscal year of the model, the profile year of the valuations, the fiscal year
     * of the expenditure — and these two rows were on a fourth, silently. A bare `2024` beside
     * `FY2027` in the same table invites exactly the reading `lib/year.ts` exists to prevent, so
     * the words go in the label where there is room for them.
     */
    key: "voted-millage",
    label: (y) => taxYear("Voted operating millage", y.millage),
    pick: (d) => d.voted_operating_millage,
    format: (v) => (v == null ? "—" : v.toFixed(2)),
  },
  {
    key: "effective-millage",
    label: (y) => taxYear("Effective Class 1 millage", y.millage),
    pick: (d) => d.effective_class1_millage,
    format: (v) => (v == null ? "—" : v.toFixed(2)),
    href: routes.metric("effective-operating-millage"),
  },
  {
    key: "spending",
    label: () => "Operating expenditure per pupil",
    pick: (d) => d.operating_expenditure_per_pupil,
    format: (v) => money(v),
    href: routes.metric("per-pupil-operating-expenditure"),
  },
  {
    key: "disadvantaged",
    label: () => "Economically disadvantaged",
    pick: (d) => d.economically_disadvantaged,
    format: (v) => pct(v, 1),
  },
  {
    key: "enrollment-change",
    label: (y) => `Enrollment change, ${y.enrollment.replace("-", "–")}`,
    pick: (d) => d.enrollment_change,
    format: (v) => pct(v, 1),
  },
];

/** A yes/no property, which has a difference but not an arithmetic one. */
export interface Flag {
  key: string;
  label: string;
  pick: (d: PanelDistrict) => boolean;
}

export const FLAGS: Flag[] = [
  { key: "on-guarantee", label: "On the guarantee", pick: (d) => d.on_guarantee },
  { key: "at-floor", label: "At or below the 20-mill floor", pick: (d) => d.at_millage_floor },
  { key: "near-floor", label: "Within a twentieth of a mill of it", pick: (d) => d.near_millage_floor },
  { key: "at-minimum", label: "At the minimum state share", pick: (d) => d.at_minimum_state_share },
];

/**
 * A millage row's label, with the tax year it is measured in spelled out.
 *
 * `2024 tax year` in full rather than a bare `2024`: `FY2024` and `2024` differ by a prefix, and a
 * prefix reads as typography rather than as a claim about which eleven-month-offset period a
 * figure covers. The label falls back to the bare name where the feed carries no millage block —
 * a row saying nothing about its year is what this is fixing, but "undefined tax year" is worse.
 */
function taxYear(name: string, year: string): string {
  return year ? `${name}, ${year} tax year` : name;
}

/** The signed, neutral difference between two districts on one row, as plain text. */
export function difference(row: Row, left: number | null, right: number | null): string {
  if (left == null || right == null) return "not comparable";
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

function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!);
}

/** One district as this table names it: the qualified name, and where its own page is. */
export interface Named {
  district: PanelDistrict;
  /** `qualifiedName` from `feed.ts` — "Green Local, Summit County" where the name repeats. */
  name: string;
}

/**
 * The whole comparison card.
 *
 * Called by `compare.astro` at build time. The client never calls it: a district swap changes
 * values and column heads and nothing else, so `scripts/compare.ts` rewrites those cells in place
 * rather than re-deriving markup the build already got right.
 *
 * `chip` is the card's year chip, built by `lib/year.ts` and passed in rather than called for:
 * that module reads the feed off the filesystem, and this one has to stay loadable in a browser.
 * It is not decoration — `tests/e2e/figures.spec.ts` requires every rendered figure to be able to
 * reach the year it is measured in, and until this table was rendered at build time the check
 * could not see it. Twenty-eight of its figures were undated the moment it became visible.
 */
export function comparison(years: Years, left: Named, right: Named, chip: string): string {
  const rows = ROWS.map((row) => {
    const l = row.pick(left.district);
    const r = row.pick(right.district);
    const text = row.label(years);
    const label = row.href ? `<a href="${row.href}">${escapeHtml(text)}</a>` : escapeHtml(text);
    return `<tr data-row="${row.key}">
      <th>${label}</th>
      <td class="tnum">${escapeHtml(row.format(l))}</td>
      <td class="tnum">${escapeHtml(row.format(r))}</td>
      <td class="n">${escapeHtml(difference(row, l, r))}</td>
    </tr>`;
  }).join("");

  const flags = FLAGS.map(
    (flag) => `<tr data-row="${flag.key}">
      <th>${escapeHtml(flag.label)}</th>
      <td>${flag.pick(left.district) ? "yes" : "no"}</td>
      <td>${flag.pick(right.district) ? "yes" : "no"}</td>
      <td class="n">${flag.pick(left.district) === flag.pick(right.district) ? "same" : "differs"}</td>
    </tr>`,
  ).join("");

  const head = (side: Named) =>
    `<th data-head="${side === left ? "a" : "b"}"><a href="${routes.district(side.district.irn)}">${escapeHtml(side.name)}</a></th>`;

  /*
   * The card names its own pair. `scripts/compare.ts` reads it to decide whether the document it
   * was served is already the comparison the query asked for — and on a bare `/compare` it is, so
   * that page does no network work at all after the HTML.
   */
  return `<div class="card" id="comparison" data-part="comparison" data-a="${left.district.irn}" data-b="${right.district.irn}">
      <h2>${anchor("comparison")}Side by side${chip}</h2>
      <div class="scroll"><table>
        <thead><tr><th></th>${head(left)}${head(right)}<th>Difference</th></tr></thead>
        <tbody>${rows}${flags}</tbody>
      </table></div>
      <p class="note">Nothing in this table is coloured good or bad. Higher state aid per pupil and
        higher property wealth point in opposite directions, and which figure counts as favourable
        depends on the argument being made — which is the reason to look at two districts rather
        than one. FY${years.fiscal} model; valuation is ${escapeHtml(years.profile)} and
        expenditure ${escapeHtml(years.spending)}.</p>
    </div>`;
}
