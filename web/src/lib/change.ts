/**
 * Why a district's funding moved from one year of the biennium to the next — the `/change` tab.
 *
 * # What this page may say and what it may not
 *
 * The three years are three department files: FY2025's is the final payment report, and FY2026's
 * and FY2027's are the department's models of those years, not payments. Every figure here is a
 * line one of those files publishes, in nominal dollars, and every change is attributed to a line
 * or a provision — never to an intent, and never to an input the files do not let it be split
 * across. The FY2025 report carries no capacity inputs at all, so the drivers card starts its
 * state share and its enrollment at FY2026 rather than inventing a first year.
 *
 * # Two measures, still
 *
 * Total state support is the headline and foundation aid is one control away, as on the
 * dashboard's biennium card (`biennium.ts`). The two disagree in sign statewide, so nothing here
 * ranks, charts or sums a district without naming which of them it is on.
 *
 * # Why a missing year is a word
 *
 * FY2026's file publishes no transfers and FY2025's no state share. Those cells read "not
 * published", because a zero would be a claim that the district had no transfers, and the
 * difference between the two is the whole of what a reader checking this page would want to know.
 */
import type { Bar } from "./chart.ts";
import { escapeHtml, fixed, money, ordinal, pct, signedMoney } from "./format.ts";
import { barSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import * as routes from "./routes.ts";
import { anchor } from "./section.ts";
import type { District } from "./types.ts";
import { yearChip } from "./year.ts";

type Biennium = District["biennium"];

/** The two measures a change can be on. `total` is the headline. */
export type MeasureKey = "total" | "foundation";

/** Each measure's name and its three levels, by position — index 0 is `year_baseline`. */
export const MEASURES: Record<
  MeasureKey,
  { readonly name: string; readonly levels: (b: Biennium) => [number, number, number] }
> = {
  total: {
    name: "total state support",
    levels: (b) => [b.total_baseline, b.total_middle, b.total_terminal],
  },
  foundation: {
    name: "foundation aid",
    levels: (b) => [b.foundation_baseline, b.foundation_middle, b.foundation_terminal],
  },
};

/** The model's three fiscal years, read from the feed. */
export function yearsOf(b: Biennium): [number, number, number] {
  return [b.year_baseline, b.year_middle, b.year_terminal];
}

/** A change as a percentage with its sign, `+0.1%` or `−3.1%`. */
export function signedPct(v: number): string {
  return v > 0 ? `+${pct(v, 1)}` : pct(v, 1);
}

/** A position in the three years: 0 is `year_baseline`, 2 is `year_terminal`. */
export type YearIndex = 0 | 1 | 2;

/** One year to a later one, in dollars and as a share of the earlier year. */
export function levelChange(
  b: Biennium,
  measure: MeasureKey,
  from: YearIndex,
  to: YearIndex,
): { dollars: number; ratio: number } {
  const levels = MEASURES[measure].levels(b);
  const dollars = levels[to] - levels[from];
  return { dollars, ratio: levels[from] > 0 ? dollars / levels[from] : 0 };
}

/** The middle year to the last, in dollars and as a share of the middle year. */
export function yearChange(b: Biennium, measure: MeasureKey): { dollars: number; ratio: number } {
  return levelChange(b, measure, 1, 2);
}

/** Under a cent is no change: the files publish to the cent. */
export const UNMOVED = 0.005;

/**
 * Where a district's change sits among its county's.
 *
 * `rank` counts within the district's own direction — "2nd-largest fall" — so a district that
 * fell least is not described as ranking above every district that rose. Ranked on the
 * percentage, because a city and a village in one county differ in size by an order of magnitude
 * and a dollar ranking would only restate that.
 */
export interface CountyRank {
  direction: "rise" | "fall" | "none";
  /** 1 is the largest move in `direction`. 0 where the district did not move. */
  rank: number;
  /** Districts in the county moving the same way, this one included. */
  alike: number;
  /** Districts in the county, this one included. */
  of: number;
}

export function countyRank(
  d: District,
  districts: readonly District[],
  measure: MeasureKey,
): CountyRank {
  const peers = districts.filter((p) => p.county === d.county);
  const moved = (p: District) => yearChange(p.biennium, measure);
  const own = moved(d);
  const direction =
    Math.abs(own.dollars) < UNMOVED ? "none" : own.dollars > 0 ? "rise" : "fall";
  const alike = peers.filter((p) => {
    const c = moved(p);
    if (direction === "none") return Math.abs(c.dollars) < UNMOVED;
    return direction === "rise" ? c.dollars >= UNMOVED : c.dollars <= -UNMOVED;
  });
  const rank =
    direction === "none"
      ? 0
      : 1 +
        alike.filter((p) => Math.abs(moved(p).ratio) > Math.abs(own.ratio)).length;
  return { direction, rank, alike: alike.length, of: peers.length };
}

/** "the 2nd-largest fall of 9 in Allen County", read off {@link countyRank}. */
export function rankPhrase(rank: CountyRank, county: string): string {
  const where = `${county} County`;
  if (rank.of <= 1) return `the only district in ${where}`;
  if (rank.direction === "none") {
    return rank.alike === 1
      ? `the only one of ${rank.of} in ${where} that did not move`
      : `one of ${rank.alike} of ${rank.of} in ${where} that did not move`;
  }
  const largest = rank.rank === 1 ? "largest" : `${ordinal(rank.rank)}-largest`;
  return `the ${largest} ${rank.direction} of ${rank.of} in ${where}`;
}

/**
 * One named payment line across the three years.
 *
 * Foundation aid, the three lines outside the formula, and the five supplements — which between
 * them are total state support in every year, to within cent rounding. Supplemental targeted
 * assistance keeps its own row in the years after H.B. 96 repealed it, so the reader sees what
 * went rather than a supplements total that quietly lost a part.
 */
export interface Line {
  label: string;
  levels: [number, number, number];
  /** Foundation aid is the narrow measure itself, and is marked as one. */
  measure?: MeasureKey;
}

export function namedLines(b: Biennium): Line[] {
  const at = (pick: (o: Biennium["observed"][number]) => number): [number, number, number] => [
    pick(b.observed[0]!),
    pick(b.observed[1]!),
    pick(b.observed[2]!),
  ];
  return [
    { label: "Foundation aid", levels: MEASURES.foundation.levels(b), measure: "foundation" },
    { label: "Transportation", levels: at((o) => o.transportation) },
    { label: "Special education transportation", levels: at((o) => o.special_education_transportation) },
    { label: "Preschool special education", levels: at((o) => o.preschool_special_education) },
    { label: "Supplemental targeted assistance (repealed by H.B. 96)", levels: at((o) => o.targeted_assistance) },
    { label: "Formula transition supplement", levels: at((o) => o.formula_transition) },
    { label: "Base funding supplement", levels: at((o) => o.base_funding) },
    { label: "Enrollment growth supplement", levels: at((o) => o.enrollment_growth) },
    { label: "Performance supplement", levels: at((o) => o.performance) },
  ];
}

/**
 * The line that moved most from the middle year to the last, by the size of the move.
 *
 * `null` when no line moved by a cent — a district held at its base with flat outside lines.
 */
export function largestMove(b: Biennium): { label: string; change: number } | null {
  return largestMoveBetween(b, 1, 2);
}

/** {@link largestMove} between any two of the three years. */
export function largestMoveBetween(
  b: Biennium,
  from: YearIndex,
  to: YearIndex,
): { label: string; change: number } | null {
  let best: { label: string; change: number } | null = null;
  for (const line of namedLines(b)) {
    const change = line.levels[to] - line.levels[from];
    if (Math.abs(change) < UNMOVED) continue;
    if (!best || Math.abs(change) > Math.abs(best.change)) best = { label: line.label, change };
  }
  return best;
}

/** A column head naming the year and whether its file is a payment or a model. */
function yearHead(years: [number, number, number], i: number): string {
  return `FY${years[i]} ${i === 0 ? "paid" : "model"}`;
}

/** The note every card repeats in substance: which files, which basis. */
function filesNote(years: [number, number, number]): string {
  return `FY${years[0]} is the department's final payment report; FY${years[1]} and FY${years[2]}
    are its models of those years, not payments. Every figure is in nominal dollars.`;
}

/** Section 1: every named line in the three years, and both measures labelled. */
export function renderByLine(d: District): string {
  const b = d.biennium;
  const years = yearsOf(b);
  const total = MEASURES.total.levels(b);
  const row = (label: string, levels: [number, number, number], strong = false) => {
    const cells = levels.map((v) => `<td class="tnum">${money(v)}</td>`).join("");
    const head = strong ? `<strong>${escapeHtml(label)}</strong>` : escapeHtml(label);
    return `<tr><th scope="row">${head}</th>${cells}<td class="tnum">${signedMoney(levels[2] - levels[1])}</td></tr>`;
  };
  return `
    <div class="card" id="by-line" data-part="by-line">
      <h2>${anchor("by-line")}By line${yearChip("biennium")}</h2>
      <p class="note">Every line the department pays this district through, in the three years.
        <strong>Total state support</strong>, the last row, is all of them; <strong>foundation
        aid</strong>, the first, is the narrow measure — core foundation funding and the guarantee,
        the part this site models. ${filesNote(years)}</p>
      <div class="scroll">
        <table>
          <thead>
            <tr><th scope="col">Line</th>
              ${years.map((_, i) => `<th scope="col" class="tnum">${yearHead(years, i)}</th>`).join("")}
              <th scope="col" class="tnum">FY${years[1]} to FY${years[2]}</th></tr>
          </thead>
          <tbody>
            ${namedLines(b)
              .map((line) =>
                row(line.measure ? `${line.label} (the narrow measure)` : line.label, line.levels, line.measure != null),
              )
              .join("")}
            ${row("Total state support (the wide measure)", total, true)}
          </tbody>
        </table>
      </div>
      <p class="note">The lines sum to total state support in every year, to within cent rounding.
        Supplemental targeted assistance keeps its row after FY${years[0]} because its going is
        part of the change: H.B. 96 repealed it, and the base funding and enrollment growth
        supplements it created are separate lines, not its replacement under another name.</p>
    </div>`;
}

/**
 * Section 2: the phase-in's three published stages, and what the guarantee added.
 *
 * R.C. 3317.022 pays `base + rate × (calculated − base)`. A district whose formula calculates
 * about its base gains little however far the rate rises, which this card says in words where
 * it is so rather than leaving the reader to subtract.
 */
export function renderPhaseIn(d: District): string {
  const b = d.biennium;
  const years = yearsOf(b);
  const foundation = MEASURES.foundation.levels(b);
  const stage = (label: string, pick: (i: number) => number) =>
    `<tr><th scope="row">${label}</th>${years.map((_, i) => `<td class="tnum">${money(pick(i))}</td>`).join("")}</tr>`;
  const sentences = years.map((year, i) => {
    const o = b.observed[i]!;
    const above = o.phase_in_paid - o.funding_base;
    const level = foundation[i]!;
    const guarantee = level - o.phase_in_paid;
    if (Math.abs(level - o.funding_base) < 1) {
      return `<li><strong>FY${year}: at its base.</strong> The formula calculates
        ${money(o.phase_in_calculated)} against a ${money(o.funding_base)} base, and foundation aid
        is the base itself${guarantee >= UNMOVED ? ` — the guarantee pays the ${money(guarantee)} the phase-in leaves short` : ""}.
        A higher phase-in rate would not reach it.</li>`;
    }
    if (above > 0) {
      return `<li><strong>FY${year}: ${signedMoney(above)} above its base.</strong> The formula
        calculates ${money(o.phase_in_calculated)}, ${money(o.phase_in_calculated - o.funding_base)}
        over a ${money(o.funding_base)} base, and the phase-in pays ${money(above)} of that
        gap.${guarantee >= UNMOVED ? ` The guarantee adds ${money(guarantee)}.` : ""}</li>`;
    }
    return `<li><strong>FY${year}: below its base.</strong> The phase-in pays
      ${money(o.phase_in_paid)} against a ${money(o.funding_base)} base${guarantee >= UNMOVED ? `, and the guarantee adds ${money(guarantee)}` : ""}.</li>`;
  });
  const last = b.observed[2]!;
  const near =
    last.funding_base > 0 &&
    Math.abs(last.phase_in_calculated - last.funding_base) / last.funding_base < 0.01;
  return `
    <div class="card" id="phase-in" data-part="phase-in">
      <h2>${anchor("phase-in")}The phase-in${yearChip("biennium")}</h2>
      <p class="note">Foundation aid is paid as the funding base plus a share of the gap between
        what the formula calculates and that base, and the guarantee holds it up where the result
        would fall. These are the department's own three figures for each year.
        ${filesNote(years)}</p>
      <div class="scroll">
        <table>
          <thead>
            <tr><th scope="col">Stage</th>
              ${years.map((_, i) => `<th scope="col" class="tnum">${yearHead(years, i)}</th>`).join("")}</tr>
          </thead>
          <tbody>
            ${stage("Funding base", (i) => b.observed[i]!.funding_base)}
            ${stage("Calculated by the formula", (i) => b.observed[i]!.phase_in_calculated)}
            ${stage("Paid through the phase-in", (i) => b.observed[i]!.phase_in_paid)}
            ${stage("Guarantee", (i) => foundation[i]! - b.observed[i]!.phase_in_paid)}
            ${stage("<strong>Foundation aid</strong>", (i) => foundation[i]!)}
          </tbody>
        </table>
      </div>
      <ul class="note">${sentences.join("")}</ul>
      ${
        near
          ? `<p class="note"><strong>The formula calculates within ${pct(Math.abs(last.phase_in_calculated - last.funding_base) / last.funding_base, 1)}
             of this district's base in FY${years[2]}</strong> — ${money(last.phase_in_calculated)}
             against ${money(last.funding_base)} — so the phase-in has almost nothing to phase in,
             and a change to its rate moves this district by little or nothing.</p>`
          : ""
      }
    </div>`;
}

/**
 * Section 3: state share, valuation and enrollment, each beside the dollars it bears on.
 *
 * Beside, not apportioned. The three files publish the inputs and the payment, never how much of
 * the payment's change each input is, so each sentence states one input's movement and the
 * measure it reaches through, and the card says plainly that the split is not here.
 */
export function renderDrivers(d: District): string {
  const b = d.biennium;
  const years = yearsOf(b);
  const [, middle, last] = b.observed;
  const foundation = yearChange(b, "foundation");
  const [v0, , v2] = b.valuation;
  const [t0, , t2] = b.valuation_tax_years;
  const share =
    middle!.state_share != null && last!.state_share != null
      ? `<li><strong>State share</strong> went from ${pct(middle!.state_share, 1)} in FY${years[1]} to
          ${pct(last!.state_share, 1)} in FY${years[2]}. It is the part of base cost the state pays
          after the district's local capacity is charged, so it reaches the dollars through
          foundation aid, which moved ${signedMoney(foundation.dollars)} over the same two
          years.</li>`
      : `<li><strong>State share</strong> is not published for one of FY${years[1]} and
          FY${years[2]} for this district.</li>`;
  const valuation = `<li><strong>Assessed valuation</strong> went from ${money(v0)} in tax year ${t0}
      to ${money(v2)} in tax year ${t2}${v0! > 0 ? ` (${signedPct(v2! / v0! - 1)})` : ""}. Valuation is
      the larger part of local capacity, which is what sets the state share above — the route by
      which a reappraisal reaches foundation aid at all.</li>`;
  const adm =
    middle!.enrolled_adm != null && last!.enrolled_adm != null
      ? `<li><strong>Enrolled ADM</strong> went from ${fixed(middle!.enrolled_adm, 1)} in
          FY${years[1]} to ${fixed(last!.enrolled_adm, 1)} in FY${years[2]}, as each year's file
          states it${middle!.enrolled_adm > 0 ? ` (${signedPct(last!.enrolled_adm / middle!.enrolled_adm - 1)})` : ""}.
          Base cost is computed per pupil, so enrollment reaches the dollars through the formula's
          calculated figure on the phase-in card.</li>`
      : `<li><strong>Enrolled ADM</strong> is not published for one of FY${years[1]} and
          FY${years[2]} for this district.</li>`;
  const held = Math.abs(b.foundation_terminal - b.observed[2]!.funding_base) < 1;
  return `
    <div class="card" id="drivers" data-part="drivers">
      <h2>${anchor("drivers")}What moved underneath${yearChip("biennium")}</h2>
      <p class="note">Three inputs and the measure each reaches the dollars through. The files
        publish the inputs and the payment but not how much of the payment's change each input is,
        so nothing here divides the change between them. The FY${years[0]} payment report carries
        no capacity inputs at all, so state share and enrollment start at FY${years[1]}. Dollar
        figures are in nominal dollars.</p>
      <ul class="note">${share}${valuation}${adm}</ul>
      ${
        held
          ? `<p class="note"><strong>This district is at its base in FY${years[2]}</strong>, so none
             of the three reaches its foundation aid that year: whatever they did, the guarantee
             paid the difference.</p>`
          : ""
      }
    </div>`;
}

/** Section 4: transfers and net funding — FY2025's total, FY2027's items, FY2026 unpublished. */
export function renderTransfers(d: District): string {
  const b = d.biennium;
  const years = yearsOf(b);
  const total = MEASURES.total.levels(b);
  const cell = (v: number | null, missing: string) =>
    v == null ? `<td class="n">${missing}</td>` : `<td class="tnum">${money(v)}</td>`;
  // A year whose file publishes no transfers at all is "not published"; one that publishes the
  // total without its items is "not itemized". Neither is a zero.
  const missing = (i: number) => (b.observed[i]!.transfers == null ? "not published" : "not itemized");
  const row = (label: string, pick: (o: Biennium["observed"][number], i: number) => number | null) =>
    `<tr><th scope="row">${label}</th>${years.map((_, i) => cell(pick(b.observed[i]!, i), missing(i))).join("")}</tr>`;
  return `
    <div class="card" id="transfers" data-part="transfers">
      <h2>${anchor("transfers")}Transfers and net funding${yearChip("biennium")}</h2>
      <p class="note">What the department deducts from total state support before it pays the
        district — the educational service center's charge and other adjustments — and what is
        left. FY${years[0]}'s report publishes the transfers as one total; FY${years[2]}'s itemizes
        them; FY${years[1]}'s publishes none, and that column says so rather than showing a zero.
        ${filesNote(years)}</p>
      <div class="scroll">
        <table>
          <thead>
            <tr><th scope="col"></th>
              ${years.map((_, i) => `<th scope="col" class="tnum">${yearHead(years, i)}</th>`).join("")}</tr>
          </thead>
          <tbody>
            <tr><th scope="row">Total state support</th>${total.map((v) => `<td class="tnum">${money(v)}</td>`).join("")}</tr>
            ${row("Service center charge", (o) => o.service_center_charge)}
            ${row("Other adjustments", (o) => o.other_adjustments)}
            ${row("Transfers, in total", (o) => o.transfers)}
            ${row("<strong>Net state funding</strong>", (o) => o.net_state_funding)}
          </tbody>
        </table>
      </div>
    </div>`;
}

/** One measure's chart of the county, this district marked. */
function neighborsChart(d: District, peers: readonly District[], measure: MeasureKey): string {
  const b = d.biennium;
  const years = yearsOf(b);
  const name = MEASURES[measure].name;
  const ranked = peers
    .map((p) => ({ p, change: yearChange(p.biennium, measure) }))
    .sort((x, y) => y.change.ratio - x.change.ratio);
  const bars: Bar[] = ranked.map(({ p, change }) => ({
    label: p.name,
    value: change.ratio,
    hover: `${p.name}: ${signedPct(change.ratio)} (${signedMoney(change.dollars)}) in ${name}, FY${years[1]} to FY${years[2]}`,
    current: p.irn === d.irn,
  }));
  const label = `Change in ${name} from FY${years[1]} to FY${years[2]}, as a percentage of FY${years[1]}, for each of the ${peers.length} districts in ${d.county} County; ${d.name} is marked`;
  return `<div class="chartwrap" data-chart="neighbors-${measure}">${renderToString(
    (w) => barSpec(bars, { width: w, scale: { format: (v) => signedPct(v), says: `change in ${name}` } }),
    { label },
  )}</div>`;
}

/**
 * Section 5: every county peer's change on the selected measure.
 *
 * The measure is a pair of radios and two panels switched by a sibling selector, as
 * `BasisToggle.astro` switches dollar bases, so the control works with no script running.
 */
export function renderNeighbors(d: District, districts: readonly District[]): string {
  const peers = districts.filter((p) => p.county === d.county);
  const years = yearsOf(d.biennium);
  const panel = (measure: MeasureKey) => {
    const change = yearChange(d.biennium, measure);
    const rank = countyRank(d, districts, measure);
    return `<div class="measure-panel ${measure}">
        <p class="note">On <strong>${MEASURES[measure].name}</strong>, ${escapeHtml(d.name)} moved
          ${signedPct(change.ratio)} (${signedMoney(change.dollars)}) from FY${years[1]} to
          FY${years[2]}: ${rankPhrase(rank, d.county)}.</p>
        ${neighborsChart(d, peers, measure)}
      </div>`;
  };
  return `
    <div class="card measure-scope" id="neighbors" data-part="neighbors">
      <h2>${anchor("neighbors")}Across ${escapeHtml(d.county)} County${yearChip("biennium")}</h2>
      <input class="vh" type="radio" name="neighbors-measure" id="neighbors-total"
        data-measure="total" aria-label="Measure: Total state support" checked />
      <input class="vh" type="radio" name="neighbors-measure" id="neighbors-foundation"
        data-measure="foundation" aria-label="Measure: Foundation aid" />
      <div class="basis">
        <label class="pill" for="neighbors-total" data-measure="total">Total state support</label>
        <label class="pill" for="neighbors-foundation" data-measure="foundation">Foundation aid</label>
        <span class="n">FY${years[1]} and FY${years[2]} models, nominal dollars</span>
      </div>
      ${panel("total")}
      ${panel("foundation")}
      <p class="note">Every district the department attributes to
        <a href="${routes.county(routes.slugify(d.county))}">${escapeHtml(d.county)} County</a>,
        as a percentage of its own FY${years[1]} figure, so a city and a village can be read on
        one scale. The two measures can rank the same county in different orders: a district held
        at its base does not move on foundation aid at all, whatever its other lines do.</p>
    </div>`;
}
