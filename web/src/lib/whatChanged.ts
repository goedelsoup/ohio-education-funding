/**
 * What changed across the biennium, statewide and by provision — the `/what-changed` page (#641).
 *
 * # What this page may say and what it may not
 *
 * The terms of a district's `/change` tab (`change.ts`), over all 609 at once. FY2025 is the
 * department's final payment report; FY2026 and FY2027 are its models of those years, not
 * payments. Every figure is a line one of those files publishes, summed or counted here, in
 * nominal dollars. A change is put down to a line or a provision — the repealed supplement, the
 * DPIA blend, local capacity, the phase-in — and never to what anyone meant by it.
 *
 * # Two measures, still
 *
 * Total state support rose across the biennium and foundation aid fell, so every money figure
 * here names which of the two it is on, and the distribution card is one measure at a time under
 * the same no-script toggle the county page uses.
 *
 * # Why the provisions are side by side and not a waterfall
 *
 * The files publish each line and each input, and never how much of a district's change each
 * input caused. A repealed supplement is its own line and can be subtracted; DPIA, capacity and
 * the phase-in all reach the payment through one calculated figure, and splitting that figure
 * among them would be this page's model and not the department's. So each card states what its
 * provision did and where it reached, and none of them adds up to the total.
 */
import {
  levelChange,
  MEASURES,
  type MeasureKey,
  signedPct,
  UNMOVED,
  type YearIndex,
  yearsOf,
} from "./change.ts";
import { tally, withoutTargetedAssistance } from "./countyChange.ts";
import { qualifiedName } from "./feed.ts";
import { count, escapeHtml, millions, money, pct } from "./format.ts";
import { compare } from "./order.ts";
import * as routes from "./routes.ts";
import { anchor } from "./section.ts";
import { median, percentile } from "./stats.ts";
import type { District } from "./types.ts";
import { yearChip } from "./year.ts";

const SECTION = routes.SECTIONS.changed;

/** The three steps, earliest first, the whole biennium last — as on the county page. */
const STEPS: readonly (readonly [YearIndex, YearIndex])[] = [
  [0, 1],
  [1, 2],
  [0, 2],
];

/** The feed's three fiscal years. Every district carries the same three. */
function years(districts: readonly District[]): [number, number, number] {
  return yearsOf(districts[0]!.biennium);
}

const span = (y: readonly number[], from: YearIndex, to: YearIndex) => `FY${y[from]} to FY${y[to]}`;

/** "$145.0M" with a verb for its sign, where a signed figure would read as a typo in a sentence. */
function moved(v: number): string {
  const magnitude = millions(Math.abs(v)).replace("+", "");
  return Math.abs(v) < UNMOVED ? "did not move" : v > 0 ? `rose ${magnitude}` : `fell ${magnitude}`;
}

/** One measure's statewide total in each of the three years. */
export function statewideLevels(districts: readonly District[], measure: MeasureKey): [number, number, number] {
  const out: [number, number, number] = [0, 0, 0];
  for (const d of districts) {
    const levels = MEASURES[measure].levels(d.biennium);
    for (const i of [0, 1, 2] as const) out[i] += levels[i];
  }
  return out;
}

/** The page's one-sentence summary, which is also its meta description. */
export function summary(districts: readonly District[]): string {
  const y = years(districts);
  const [total, foundation] = (["total", "foundation"] as const).map((m) => {
    const l = statewideLevels(districts, m);
    return l[2] - l[0];
  });
  return `From FY${y[0]} to FY${y[2]}, total state support to Ohio's ${count(districts.length)} districts ${moved(total!)} and foundation aid ${moved(foundation!)}, provision by provision.`;
}

/** A district's foundation aid is its funding base, to the dollar: the guarantee holds it there. */
function atBase(d: District, i: YearIndex): boolean {
  return Math.abs(MEASURES.foundation.levels(d.biennium)[i] - d.biennium.observed[i]!.funding_base) < 1;
}

/** The formula calculates above the district's base, so the phase-in has something to pay. */
function aboveBase(d: District, i: YearIndex): boolean {
  const o = d.biennium.observed[i]!;
  return o.phase_in_calculated > o.funding_base;
}

const th = (label: string) => `<th scope="col" class="tnum">${label}</th>`;
const td = (text: string) => `<td class="tnum">${text}</td>`;

/** Section 1: the two measures, three years and three steps each. */
export function renderTwoMeasures(districts: readonly District[]): string {
  const id = SECTION.twoMeasures;
  const y = years(districts);
  const [total, foundation] = [statewideLevels(districts, "total"), statewideLevels(districts, "foundation")];
  const others = total[2] - total[0] - (foundation[2] - foundation[0]);
  const rows = (["total", "foundation"] as const).map((m) => {
    const l = m === "total" ? total : foundation;
    return `<tr><th scope="row">${MEASURES[m].name[0]!.toUpperCase()}${MEASURES[m].name.slice(1)}</th>
      ${l.map((v) => td(millions(v).replace("+", ""))).join("")}
      ${STEPS.map(([a, b]) => td(`${millions(l[b] - l[a])}<div class="n">${signedPct((l[b] - l[a]) / l[a])}</div>`)).join("")}
    </tr>`;
  });
  return `
    <div class="card" id="${id}" data-part="${id}">
      <h2>${anchor(id)}The two measures${yearChip("biennium")}</h2>
      <p class="note">${escapeHtml(summary(districts))} <strong>Total state support</strong> is every
        line the department pays a district through; <strong>foundation aid</strong> is core
        foundation funding and the guarantee alone. The difference between them is transportation,
        the two special education lines and the named supplements, and from FY${y[0]} to FY${y[2]}
        those together ${moved(others)}.</p>
      <div class="scroll"><table>
        <thead><tr><th scope="col">Measure, all ${count(districts.length)} districts</th>
          ${y.map((v, i) => th(`FY${v} ${i === 0 ? "paid" : "model"}`)).join("")}
          ${STEPS.map(([a, b]) => th(span(y, a, b))).join("")}
        </tr></thead>
        <tbody>${rows.join("")}</tbody>
      </table></div>
      <p class="note">FY${y[0]} is the department's final payment report; FY${y[1]} and FY${y[2]}
        are its models of those years, not payments. The population is the ${count(districts.length)}
        districts all three files carry, in nominal dollars.</p>
    </div>`;
}

/** One measure's distribution: who rose and who fell on each step, and by how much. */
function distributionPanel(districts: readonly District[], measure: MeasureKey): string {
  const y = years(districts);
  const rows = STEPS.map(([a, b]) => {
    const t = tally(districts, measure, a, b);
    const ratios = districts.map((d) => levelChange(d.biennium, measure, a, b).ratio);
    return `<tr><th scope="row">${span(y, a, b)}</th>
      ${td(count(t.rise.length))}${td(count(t.fall.length))}${td(count(t.none.length))}
      ${td(signedPct(percentile(ratios, 0.25)))}${td(signedPct(median(ratios)))}${td(signedPct(percentile(ratios, 0.75)))}
    </tr>`;
  });
  return `<div class="measure-panel ${measure}">
      <div class="scroll"><table>
        <thead><tr><th scope="col">${MEASURES[measure].name[0]!.toUpperCase()}${MEASURES[measure].name.slice(1)}</th>
          ${th("Rose")}${th("Fell")}${th("Did not move")}
          ${th("25th percentile")}${th("Median")}${th("75th percentile")}
        </tr></thead>
        <tbody>${rows.join("")}</tbody>
      </table></div>
    </div>`;
}

/** Section 2: how many districts gained and lost on each measure, and by how much. */
export function renderDistribution(districts: readonly District[]): string {
  const id = SECTION.distribution;
  const y = years(districts);
  return `
    <div class="card measure-scope" id="${id}" data-part="${id}">
      <h2>${anchor(id)}Who gained and who lost${yearChip("biennium")}</h2>
      <p class="note">The ${count(districts.length)} districts on the measure selected. The
        percentiles are of each district's change as a share of its own earlier year, so a city and
        a village count alike; a change under a cent is no change.</p>
      <input class="vh" type="radio" name="${id}-measure" id="${id}-total"
        data-measure="total" aria-label="Measure: Total state support" checked />
      <input class="vh" type="radio" name="${id}-measure" id="${id}-foundation"
        data-measure="foundation" aria-label="Measure: Foundation aid" />
      <div class="basis">
        <label class="pill" for="${id}-total" data-measure="total">Total state support</label>
        <label class="pill" for="${id}-foundation" data-measure="foundation">Foundation aid</label>
        <span class="n">FY${y[0]} paid, FY${y[1]} and FY${y[2]} models</span>
      </div>
      ${distributionPanel(districts, "total")}
      ${distributionPanel(districts, "foundation")}
      <p class="note">Every district moves on total state support, because transportation alone
        changes everywhere. On foundation aid a district held at its funding base does not move at
        all, which is why that measure has a column of districts that did neither.</p>
    </div>`;
}

/** The repealed supplement, statewide. */
export function targetedAssistance(districts: readonly District[]) {
  const recipients = districts.filter((d) => d.biennium.observed[0]!.targeted_assistance >= UNMOVED);
  const rest = districts.filter((d) => !recipients.includes(d));
  const cut = (d: District) => levelChange(d.biennium, "total", 0, 1).dollars < 0;
  const without = recipients.map((d) => withoutTargetedAssistance(d)!);
  return {
    dollars: recipients.reduce((s, d) => s + d.biennium.observed[0]!.targeted_assistance, 0),
    recipients: recipients.length,
    recipientsCut: recipients.filter(cut).length,
    rest: rest.length,
    restCut: rest.filter(cut).length,
    cutWithout: without.filter((x) => x.dollars < 0).length,
  };
}

/** Section 3: supplemental targeted assistance, which H.B. 96 repealed after the first year. */
export function renderTargetedAssistance(districts: readonly District[]): string {
  const id = SECTION.targetedAssistance;
  const y = years(districts);
  const t = targetedAssistance(districts);
  const total = statewideLevels(districts, "total");
  const share = (n: number, of: number) => `${count(n)} of ${count(of)} (${pct(n / of, 0)})`;
  return `
    <div class="card" id="${id}" data-part="${id}">
      <h2>${anchor(id)}The repealed supplement${yearChip("biennium")}</h2>
      <p class="note"><strong>Supplemental targeted assistance</strong> paid
        ${money(t.dollars)} to ${count(t.recipients)} districts in FY${y[0]}, and
        <a href="${routes.wikiNode("legislation", "hb-96-2025")}">H.B. 96</a> repealed it: neither
        model year carries it. It is a line of total state support and not of foundation aid, so it
        is the one provision here whose whole effect can be read off by subtraction.</p>
      <div class="scroll"><table>
        <thead><tr><th scope="col">Districts</th>
          ${th(`Total state support cut, ${span(y, 0, 1)}`)}
        </tr></thead>
        <tbody>
          <tr><th scope="row">Paid the supplement in FY${y[0]}</th>${td(share(t.recipientsCut, t.recipients))}</tr>
          <tr><th scope="row">Every other district</th>${td(share(t.restCut, t.rest))}</tr>
          <tr><th scope="row">Paid it, with the supplement taken out of both years</th>${td(share(t.cutWithout, t.recipients))}</tr>
        </tbody>
      </table></div>
      <p class="note">With the supplement taken out of FY${y[0]}, statewide total state support
        from FY${y[0]} to FY${y[1]} reads ${millions(total[1] - total[0] + t.dollars)} rather than
        ${millions(total[1] - total[0])}. The last row is what the rest of a recipient's payment
        did: ${count(t.recipientsCut)} recipients were cut with the supplement counted, and
        ${count(t.cutWithout)} without it.</p>
    </div>`;
}

/** DPIA in the two model years that itemize it, split by where the FY2027 payment sits. */
export function dpia(districts: readonly District[]) {
  const at = (d: District, i: 1 | 2) => d.biennium.observed[i]!.dpia ?? 0;
  const group = (list: readonly District[]) => ({
    districts: list.length,
    levels: [list.reduce((s, d) => s + at(d, 1), 0), list.reduce((s, d) => s + at(d, 2), 0)] as const,
  });
  return {
    all: group(districts),
    fell: districts.filter((d) => at(d, 2) - at(d, 1) < -UNMOVED).length,
    formula: group(districts.filter((d) => aboveBase(d, 2))),
    held: group(districts.filter((d) => !aboveBase(d, 2))),
  };
}

/** Section 4: the change in how DPIA counts directly certified students. */
export function renderDirectCertification(districts: readonly District[]): string {
  const id = SECTION.directCertification;
  const y = years(districts);
  const x = dpia(districts);
  const row = (label: string, g: { levels: readonly [number, number] }) => {
    const [a, b] = g.levels;
    return `<tr><th scope="row">${label}</th>${td(millions(a).replace("+", ""))}${td(millions(b).replace("+", ""))}
      ${td(`${millions(b - a)}<div class="n">${signedPct(a > 0 ? (b - a) / a : 0)}</div>`)}</tr>`;
  };
  return `
    <div class="card" id="${id}" data-part="${id}">
      <h2>${anchor(id)}Direct certification in DPIA${yearChip("biennium")}</h2>
      <p class="note"><a href="${routes.wikiNode("formula-component", "fsfp-disadvantaged-pupil-impact-aid")}">Disadvantaged
        Pupil Impact Aid</a> counts a district's poor pupils as a blend of its economically
        disadvantaged count and its directly certified count. H.B. 96 gives the directly certified
        count more of the blend in FY${y[2]} than in FY${y[1]}, and the count the FY${y[2]} model
        carries is lower than the one in the FY${y[1]} model; the node sets out both, with the act's
        text.</p>
      <div class="scroll"><table>
        <thead><tr><th scope="col">DPIA as the formula computes it</th>
          ${th(`FY${y[1]} model`)}${th(`FY${y[2]} model`)}${th("Change")}
        </tr></thead>
        <tbody>
          ${row(`All ${count(districts.length)} districts`, x.all)}
          ${row(`The ${count(x.formula.districts)} whose FY${y[2]} formula calculates above their base`, x.formula)}
          ${row(`The ${count(x.held.districts)} whose FY${y[2]} formula calculates at or below it`, x.held)}
        </tbody>
      </table></div>
      <p class="note">DPIA fell in ${count(x.fell)} of the ${count(districts.length)}. It is a
        component of the formula's calculated aid, not a payment line, so it is never added to
        either measure. Where the FY${y[2]} formula calculates above a district's base, the phase-in
        is complete and calculated aid is paid, so the fall reaches foundation aid. Where it
        calculates at or below the base, the guarantee holds the district up and a smaller DPIA
        deepens the shortfall the guarantee pays rather than the payment. The FY${y[0]} payment
        report does not itemize DPIA, so its year is not a column here.</p>
    </div>`;
}

/**
 * The state-share bands. The first is the formula's minimum, read off the data as the smallest
 * share any district has; the cut points after it are where this page draws the bands, not the
 * statute's.
 */
const BAND_EDGES = [0.3, 0.5, 0.7] as const;

/** A district's FY2026 state share band, as a label. */
export function shareBands(districts: readonly District[]): { label: string; districts: District[] }[] {
  const shares = districts.map((d) => d.biennium.observed[1]!.state_share).filter((s): s is number => s != null);
  const minimum = Math.min(...shares);
  const share = (d: District) => d.biennium.observed[1]!.state_share;
  const edges = [minimum, ...BAND_EDGES, Infinity];
  const bands = [
    { label: `At the minimum, ${pct(minimum, 0)}`, districts: districts.filter((d) => share(d) === minimum) },
  ];
  for (let i = 0; i + 1 < edges.length; i++) {
    const [lo, hi] = [edges[i]!, edges[i + 1]!];
    bands.push({
      label: hi === Infinity ? `${pct(lo, 0)} and over` : i === 0 ? `Over the minimum, under ${pct(hi, 0)}` : `${pct(lo, 0)} to under ${pct(hi, 0)}`,
      districts: districts.filter((d) => {
        const s = share(d);
        return s != null && (i === 0 ? s > lo : s >= lo) && s < hi;
      }),
    });
  }
  return bands;
}

/** Section 5: reappraisal through local capacity, by state-share band. */
export function renderReappraisal(districts: readonly District[]): string {
  const id = SECTION.reappraisal;
  const y = years(districts);
  const ty = districts[0]!.biennium.valuation_tax_years;
  const rows = shareBands(districts).map(({ label, districts: band }) => {
    const valuation = band
      .map((d) => d.biennium.valuation)
      .filter((v) => v[0] != null && v[2] != null && v[0] > 0)
      .map((v) => v[2]! / v[0]! - 1);
    const share = band.map((d) => {
      const o = d.biennium.observed;
      return o[2]!.state_share! / o[1]!.state_share! - 1;
    });
    const fell = band.filter((d) => levelChange(d.biennium, "foundation", 1, 2).dollars < -UNMOVED).length;
    const held = band.filter((d) => atBase(d, 2)).length;
    return `<tr><th scope="row">${label}</th>${td(count(band.length))}
      ${td(signedPct(median(valuation)))}${td(signedPct(median(share)))}
      ${td(count(fell))}${td(count(held))}</tr>`;
  });
  return `
    <div class="card" id="${id}" data-part="${id}">
      <h2>${anchor(id)}Reappraisal and the state share${yearChip("biennium")}</h2>
      <p class="note">A district's <a href="${routes.metric("state-share-percentage")}">state share</a>
        is the part of base cost left for the state after its
        <a href="${routes.wikiNode("formula-component", "fsfp-local-capacity-measure")}">local capacity</a>
        is charged, and a reappraisal raises the valuation that capacity is measured on. The same
        rise in capacity takes the same dollars off every district's share, which is a much larger
        part of a small share than of a large one. So the lower a district's share starts, the more
        of its aid a valuation rise takes.</p>
      <div class="scroll"><table>
        <thead><tr><th scope="col">State share, FY${y[1]}</th>${th("Districts")}
          ${th(`Median valuation change, TY${ty[0]} to TY${ty[2]}`)}
          ${th(`Median state share change, ${span(y, 1, 2)}`)}
          ${th(`Foundation aid fell, ${span(y, 1, 2)}`)}
          ${th(`At their base, FY${y[2]}`)}
        </tr></thead>
        <tbody>${rows.join("")}</tbody>
      </table></div>
      <p class="note">The state share change is relative to the share itself. At the minimum the share cannot fall further, and a district held at its base
        loses no foundation aid when its share falls, because the guarantee pays what the formula no
        longer does, so the bands set those apart. Valuation
        is the department's three tax years and the share its two model years; the files publish
        both and not how much of a share's fall the valuation caused, which also moves with
        enrollment and income. The county table below puts each district's share beside its
        neighbors'.</p>
    </div>`;
}

/** Section 6: the phase-in, and the districts it does not reach. */
export function phaseIn(districts: readonly District[]) {
  return ([0, 1, 2] as const).map((i) => {
    const above = districts.filter((d) => aboveBase(d, i));
    const rates = above
      .map((d) => d.biennium.observed[i]!)
      .filter((o) => o.phase_in_calculated - o.funding_base >= 1)
      .map((o) => (o.phase_in_paid - o.funding_base) / (o.phase_in_calculated - o.funding_base));
    return {
      above: above.length,
      atBase: districts.filter((d) => atBase(d, i)).length,
      rate: median(rates),
      gap: above.reduce((s, d) => s + d.biennium.observed[i]!.phase_in_calculated - d.biennium.observed[i]!.funding_base, 0),
      paid: above.reduce((s, d) => s + d.biennium.observed[i]!.phase_in_paid - d.biennium.observed[i]!.funding_base, 0),
    };
  });
}

/** Section 6, rendered. */
export function renderPhaseIn(districts: readonly District[]): string {
  const id = SECTION.phaseIn;
  const y = years(districts);
  const p = phaseIn(districts);
  const row = (label: string, f: (x: (typeof p)[number]) => string) =>
    `<tr><th scope="row">${label}</th>${p.map((x) => td(f(x))).join("")}</tr>`;
  const [first, , last] = p as [(typeof p)[0], (typeof p)[0], (typeof p)[0]];
  return `
    <div class="card" id="${id}" data-part="${id}">
      <h2>${anchor(id)}The phase-in${yearChip("biennium")}</h2>
      <p class="note">Foundation aid is paid as the funding base plus a share of the gap between
        what the formula calculates and that base — the
        <a href="${routes.parameter("fsfp-phase-in-percentage")}">phase-in percentage</a>. The rate
        below is read off the files, as what each district above its base was paid of its gap.</p>
      <div class="scroll"><table>
        <thead><tr><th scope="col">All ${count(districts.length)} districts</th>
          ${y.map((v, i) => th(`FY${v} ${i === 0 ? "paid" : "model"}`)).join("")}
        </tr></thead>
        <tbody>
          ${row("Phase-in rate", (x) => pct(x.rate, 1))}
          ${row("Districts whose formula calculates above their base", (x) => count(x.above))}
          ${row("Their gap above base, calculated", (x) => millions(x.gap).replace("+", ""))}
          ${row("Of it, paid through the phase-in", (x) => millions(x.paid).replace("+", ""))}
          ${row("Districts whose foundation aid is their base", (x) => count(x.atBase))}
        </tbody>
      </table></div>
      <p class="note">The rate rose from ${pct(first.rate, 1)} to ${pct(last.rate, 1)}, and the
        phase-in paid ${millions(Math.abs(last.paid - first.paid)).replace("+", "")}
        ${last.paid < first.paid ? "less" : "more"} above bases in FY${y[2]} than in FY${y[0]},
        because the gap it pays a share of ${last.gap < first.gap ? "shrank" : "grew"} from
        ${millions(first.gap).replace("+", "")} to ${millions(last.gap).replace("+", "")}, and
        ${count(Math.abs(first.above - last.above))} ${first.above > last.above ? "fewer" : "more"} districts
        had one at all. A district whose formula calculates at or below its base gains nothing from a
        higher rate: ${count(last.atBase)} are paid exactly their base in FY${y[2]}, and the phase-in
        does not reach them.</p>
    </div>`;
}

/** Section 7: every district, filterable by county, each linked to its change tab. */
export function renderByCounty(districts: readonly District[]): string {
  const id = SECTION.byCounty;
  const y = years(districts);
  const counties = [...new Set(districts.map((d) => d.county))].sort(compare);
  const cell = (text: string, sort: number | null, loss = false) =>
    `<td class="tnum${loss ? " loss" : ""}" data-sort-value="${sort ?? ""}">${text}</td>`;
  const head = (label: string, key: string, sorted?: "ascending") =>
    `<th scope="col" aria-sort="${sorted ?? "none"}"><button type="button" data-sort="${key}">${label}</button></th>`;
  const rows = [...districts]
    .sort((a, b) => compare(a.county, b.county) || compare(a.name, b.name))
    .map((d) => {
      const changes = (["total", "foundation"] as const).flatMap((m) =>
        STEPS.map(([a, b]) => levelChange(d.biennium, m, a, b).ratio),
      );
      const o = d.biennium.observed;
      const share =
        o[1]!.state_share == null || o[2]!.state_share == null ? null : o[2]!.state_share - o[1]!.state_share;
      return `<tr data-county="${escapeHtml(d.county)}">
        <th scope="row" data-sort-value="${escapeHtml(d.name)}"><a href="${routes.districtChange(d.irn)}">${escapeHtml(qualifiedName(d))}</a></th>
        <td data-sort-value="${escapeHtml(d.county)}">${escapeHtml(d.county)}</td>
        ${changes.map((r) => cell(signedPct(r), r, r < 0)).join("")}
        ${cell(share == null ? "not published" : `${share > 0 ? "+" : ""}${pct(share, 1).replace("%", "")} pts`, share, share != null && share < 0)}
      </tr>`;
    });
  return `
    <div class="card" id="${id}" data-part="${id}">
      <h2>${anchor(id)}By county${yearChip("biennium")}</h2>
      <p class="note">Every district, under the county the department attributes it to. Each name
        opens the district's change tab, which splits its own figures by line; each county's page
        compares its districts on both measures and all three steps.</p>
      <form class="filters" id="${id}-filter" role="search">
        <div class="field">
          <label for="${id}-county">County</label>
          <select id="${id}-county">
            <option value="">All ${count(counties.length)}</option>
            ${counties.map((c) => `<option value="${escapeHtml(c)}">${escapeHtml(c)}</option>`).join("")}
          </select>
        </div>
        <p class="count" id="${id}-count" aria-live="polite">${count(districts.length)} districts</p>
      </form>
      <div class="scroll"><table data-sortable>
        <thead><tr>
          ${head("District", "name")}${head("County", "name", "ascending")}
          ${(["total", "foundation"] as const)
            .flatMap((m) => STEPS.map(([a, b]) => head(`${m === "total" ? "Total state support" : "Foundation aid"}, ${span(y, a, b)}`, `${m}-${a}-${b}`)))
            .join("")}
          ${head(`State share, ${span(y, 1, 2)}`, "share")}
        </tr></thead>
        <tbody>${rows.join("")}</tbody>
      </table></div>
      <p class="note">Each change is the district's own, as a share of its level in the earlier year;
        the state share change is in percentage points, and the FY${y[0]} payment report does not
        publish a share.</p>
    </div>`;
}
