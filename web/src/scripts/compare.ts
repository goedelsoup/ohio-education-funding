/**
 * Two districts, side by side — the part that changes which two.
 *
 * The table itself is `lib/compare.ts`, rendered by `compare.astro` at build time. This file is
 * what a reader's choice does to it, and what it deliberately does *not* do is re-derive markup:
 * a district swap changes value cells and column heads and nothing else. Every `<th>` label, the
 * footnote and its four years are authored once, at the build, where the feed is.
 *
 * # What this used to cost
 *
 * It fetched `/data/panel.json` — 641,042 B, 127,961 gzipped — and rendered seventeen rows about
 * two of the 609 districts in it. At 400 Kbps / 400 ms RTT: first paint 1,284 ms, panel finishing
 * 4,131 ms, table 4,340 ms. Three seconds of an empty box. That was the last open half of #111.
 *
 * Now the picker's `<option>` list is the only district index the browser needs — it already holds
 * every name, qualified, because `compare.astro` writes it — and a pair costs two files of about a
 * kilobyte each. A bare `/compare` fetches **nothing**: the document it was served is already the
 * comparison it is about, which the card says in `data-a`/`data-b`.
 */

import { difference, FLAGS, ROWS } from "../lib/compare.ts";
import * as routes from "../lib/routes.ts";
import { saying } from "../lib/status.ts";
import type { PanelDistrict } from "../lib/types.ts";

const a = document.querySelector<HTMLSelectElement>("#cmp-a");
const b = document.querySelector<HTMLSelectElement>("#cmp-b");
const out = document.querySelector<HTMLElement>("#compare-out");
/*
 * What the page says once the reader has stopped changing which districts it is about.
 *
 * The table below is rewritten in place and nothing announced that it had. See `src/lib/status.ts`
 * for why the region is a separate sentence rather than `aria-live` on the table itself.
 */
const changed = document.querySelector<HTMLElement>("#changed");
const say = changed ? saying(changed) : () => {};

/**
 * The qualified name the build already wrote into the picker.
 *
 * Three districts are called Green Local, and comparing two of them would put the same string in
 * both column heads — the one table where the heads are the entire distinction between the
 * columns. `qualifiedName` in `lib/feed.ts` settles that at build time for the `<option>` text, so
 * the browser reads the answer rather than recomputing it, and the two cannot disagree.
 */
function nameOf(select: HTMLSelectElement, irn: string): string {
  return select.querySelector<HTMLOptionElement>(`option[value="${irn}"]`)?.textContent?.trim() ?? irn;
}

/** Whether the picker offers a district at all, which is the only validation an IRN needs here. */
function known(select: HTMLSelectElement, irn: string | null): irn is string {
  return irn !== null && select.querySelector(`option[value="${irn}"]`) !== null;
}

/**
 * One district's formula inputs, fetched once.
 *
 * Held rather than refetched: a reader comparing three districts against one flips back and forth,
 * and the second look at a district should not be a second request. The map is bounded by what
 * they touch.
 */
const held = new Map<string, PanelDistrict>();
async function districtOf(irn: string): Promise<PanelDistrict | null> {
  const have = held.get(irn);
  if (have) return have;
  const response = await fetch(`${import.meta.env.BASE_URL}data/district/${irn}.json`);
  if (!response.ok) return null;
  const district = (await response.json()) as PanelDistrict;
  held.set(irn, district);
  return district;
}

/** Rewrite the table's values and heads. Every label, and the footnote, stay as the build wrote them. */
function paint(left: PanelDistrict, right: PanelDistrict): void {
  const card = out?.querySelector<HTMLElement>("#comparison");
  if (!card || !a || !b) return;

  for (const [key, side, select] of [
    ["a", left, a],
    ["b", right, b],
  ] as const) {
    const head = card.querySelector<HTMLAnchorElement>(`th[data-head="${key}"] a`);
    if (!head) continue;
    head.href = routes.district(side.irn);
    head.textContent = nameOf(select, side.irn);
  }

  for (const row of ROWS) {
    const cells = card.querySelectorAll<HTMLTableCellElement>(`tr[data-row="${row.key}"] td`);
    if (cells.length < 3) continue;
    const l = row.pick(left);
    const r = row.pick(right);
    // `textContent` rather than `innerHTML`: a value cell holds a formatted number and a phrase,
    // never markup, so nothing here has to be escaped and nothing here can inject.
    cells[0]!.textContent = row.format(l);
    cells[1]!.textContent = row.format(r);
    cells[2]!.textContent = difference(row, l, r);
  }

  for (const flag of FLAGS) {
    const cells = card.querySelectorAll<HTMLTableCellElement>(`tr[data-row="${flag.key}"] td`);
    if (cells.length < 3) continue;
    cells[0]!.textContent = flag.pick(left) ? "yes" : "no";
    cells[1]!.textContent = flag.pick(right) ? "yes" : "no";
    cells[2]!.textContent = flag.pick(left) === flag.pick(right) ? "same" : "differs";
  }

  card.dataset.a = left.irn;
  card.dataset.b = right.irn;

  /*
   * And say which two districts the table is now about.
   *
   * The names rather than a tile summary, because this route has no tiles: the whole result is one
   * table of paired figures, and "the comparison is now Cleveland against Northern Local" is the
   * sentence a reader needs before they go and read it.
   */
  say(`Comparison updated: ${nameOf(a, left.irn)} against ${nameOf(b, right.irn)}.`);
}

function unreachable(irn: string, select: HTMLSelectElement): void {
  if (!out) return;
  out.innerHTML =
    `<div class="card err" id="district-unreachable" data-part="district-unreachable"><p>Could not ` +
    `load the figures for ${nameOf(select, irn)}.</p></div>`;
}

/** Fetch whichever of the pair is not already held, then rewrite the table. */
async function show(): Promise<void> {
  if (!a || !b) return;
  const next = `${location.pathname}?a=${a.value}&b=${b.value}`;
  if (location.pathname + location.search !== next) history.replaceState(null, "", next);

  const [left, right] = await Promise.all([districtOf(a.value), districtOf(b.value)]);
  if (!left) return unreachable(a.value, a);
  if (!right) return unreachable(b.value, b);
  paint(left, right);
}

function start(): void {
  if (!a || !b || !out) return;
  const card = out.querySelector<HTMLElement>("#comparison");

  const params = new URLSearchParams(location.search);
  /*
   * Defaults are the pair the document was built with, so an empty `/compare` is already showing
   * the argument the page exists to make.
   *
   * Either side may arrive alone: every district page links here with its own IRN and no partner,
   * which is the entry point that makes this route reachable at all. So the side that was not
   * given falls back to whichever half of the seeded pair the given one is *not* — otherwise a
   * reader arriving from Northern Local's page lands on Northern Local against Northern Local, a
   * table of "the same" in every row.
   */
  const POOR = card?.dataset.a ?? "";
  const RICH = card?.dataset.b ?? "";
  const asked = { a: params.get("a"), b: params.get("b") };
  a.value = known(a, asked.a) ? asked.a : asked.b === POOR ? RICH : POOR;
  b.value = known(b, asked.b) ? asked.b : a.value === RICH ? POOR : RICH;

  a.addEventListener("change", () => void show());
  b.addEventListener("change", () => void show());

  /*
   * The whole point of the build-time render: when the reader asked for the pair the document
   * already holds, there is nothing to fetch and nothing to rewrite. That is every arrival at
   * `/compare` with no query, which is where the route's own navigation sends them.
   */
  if (a.value === POOR && b.value === RICH) return;
  void show();
}

start();
