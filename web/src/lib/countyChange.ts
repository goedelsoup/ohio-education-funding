/**
 * Who in a county gained and who lost across the biennium — the county page's `who-gained` card.
 *
 * # Why a table of every pair of years, on both measures
 *
 * "Lima City was one of only two Allen districts cut" is true on total state support from FY2025
 * to FY2026, and on nothing else: on foundation aid Lima is flat and four of its neighbours fall,
 * across FY2025 to FY2027 four or five Allen districts are cut, and without the supplement H.B. 96
 * repealed Lima rose. Each of those is one row and one column of the same comparison, so the card
 * prints all three steps for every district on the selected measure, and says in words who rose
 * and who fell on each, instead of choosing the step that makes the sharpest sentence.
 *
 * # What it does not say
 *
 * The same terms as the district's `/change` tab (`change.ts`): every figure is a line the
 * department publishes, FY2026 and FY2027 are its models and not payments, and a change is put
 * down to a line or a provision, never to an intent. A county is the department's attribution of
 * each district to one county (see `county.ts`), so this compares districts and never sums them.
 */
import {
  largestMoveBetween,
  levelChange,
  MEASURES,
  type MeasureKey,
  neighborsChart,
  signedPct,
  UNMOVED,
  type YearIndex,
  yearsOf,
} from "./change.ts";
import type { County } from "./county.ts";
import { count, escapeHtml, money, pct, signedMoney } from "./format.ts";
import * as routes from "./routes.ts";
import { anchor } from "./section.ts";
import type { District } from "./types.ts";
import { yearChip } from "./year.ts";

/** The three steps the card reports, earliest first, the whole biennium last. */
const STEPS: readonly (readonly [YearIndex, YearIndex])[] = [
  [0, 1],
  [1, 2],
  [0, 2],
];

/** A district's direction on one step: under a cent is no change, as on the `/change` tab. */
function direction(dollars: number): "rise" | "fall" | "none" {
  return Math.abs(dollars) < UNMOVED ? "none" : dollars > 0 ? "rise" : "fall";
}

/** Who rose, who fell and who did not move on one measure and one step, in the county's order. */
export function tally(
  districts: readonly District[],
  measure: MeasureKey,
  from: YearIndex,
  to: YearIndex,
): Record<"rise" | "fall" | "none", District[]> {
  const out: Record<"rise" | "fall" | "none", District[]> = { rise: [], fall: [], none: [] };
  for (const d of districts) out[direction(levelChange(d.biennium, measure, from, to).dollars)].push(d);
  return out;
}

/**
 * Total state support from the first year to the second, without supplemental targeted assistance.
 *
 * H.B. 96 repealed the supplement, so the first step carries its whole FY2025 amount as a loss for
 * every district that had one. Taking it out of both years asks what the rest of a district's
 * payment did. `null` for a district that had none in the first year, where the question is empty.
 * The same definition as `Row::without_targeted_assistance` in crates/project, and both are pinned
 * on Lima City's +1.95%.
 */
export function withoutTargetedAssistance(d: District): { dollars: number; ratio: number } | null {
  const b = d.biennium;
  const [first, second] = [b.observed[0]!, b.observed[1]!];
  if (first.targeted_assistance < UNMOVED) return null;
  const before = b.total_baseline - first.targeted_assistance;
  const after = b.total_middle - second.targeted_assistance;
  return { dollars: after - before, ratio: before > 0 ? (after - before) / before : 0 };
}

/** "Lima City and Elida Local", from a list of districts. */
function names(list: readonly District[]): string {
  const each = list.map((d) => escapeHtml(d.name));
  if (each.length <= 2) return each.join(" and ");
  return `${each.slice(0, -1).join(", ")} and ${each[each.length - 1]}`;
}

/**
 * One step in a sentence: how many rose and fell, naming the smaller side where it is short enough
 * to read. A list of twenty names is a table, and the table is right below it.
 */
export function stepSentence(
  districts: readonly District[],
  measure: MeasureKey,
  from: YearIndex,
  to: YearIndex,
): string {
  const years = yearsOf(districts[0]!.biennium);
  const t = tally(districts, measure, from, to);
  const of = districts.length;
  const when = `FY${years[from]} to FY${years[to]}`;
  if (of === 1) {
    const only = t.rise.length ? "rose" : t.fall.length ? "fell" : "did not move";
    return `<li><strong>${when}:</strong> ${names(districts)} ${only}.</li>`;
  }
  const parts = [
    `${count(t.rise.length)} rose`,
    `${count(t.fall.length)} fell`,
    ...(t.none.length ? [`${count(t.none.length)} did not move`] : []),
  ];
  const fewer = t.fall.length <= t.rise.length ? { list: t.fall, verb: "fell" } : { list: t.rise, verb: "rose" };
  const named =
    fewer.list.length === 0
      ? ""
      : fewer.list.length === 1
        ? ` Only ${names(fewer.list)} ${fewer.verb}.`
        : fewer.list.length <= 5
          ? ` The ${count(fewer.list.length)} that ${fewer.verb}: ${names(fewer.list)}.`
          : "";
  return `<li><strong>${when}:</strong> of ${count(of)}, ${parts.join(", ")}.${named}</li>`;
}

/** A cell with its sort key, which the formatted text is not. */
function cell(text: string, sortValue: number | string | null): string {
  return `<td class="tnum" data-sort-value="${sortValue == null ? "" : escapeHtml(String(sortValue))}">${text}</td>`;
}

/** A sortable column head, `aria-sort` on the `th` as on the districts index. */
function head(label: string, key: string, sorted?: "descending"): string {
  return `<th scope="col" class="tnum" aria-sort="${sorted ?? "none"}"><button type="button" data-sort="${key}">${label}</button></th>`;
}

/**
 * One measure's sentences, its chart and its table, in that order (#661): the chart draws the
 * whole biennium the table is sorted on, so its bars read top to bottom in the table's order.
 */
function panel(c: County, measure: MeasureKey): string {
  const years = yearsOf(c.districts[0]!.biennium);
  const name = MEASURES[measure].name;
  const step = (from: YearIndex, to: YearIndex) => `FY${years[from]} to FY${years[to]}`;
  // Written in the order the reader is most likely to ask about: the whole biennium, largest
  // rise first. The sort buttons reorder it; without a script this is the order there is.
  const rows = [...c.districts].sort(
    (a, b) => levelChange(b.biennium, measure, 0, 2).ratio - levelChange(a.biennium, measure, 0, 2).ratio,
  );
  const body = rows
    .map((d) => {
      const b = d.biennium;
      const levels = MEASURES[measure].levels(b);
      const changes = STEPS.map(([from, to]) => levelChange(b, measure, from, to));
      const [, middle, last] = b.observed;
      const moved = largestMoveBetween(b, 0, 2);
      return `<tr>
        <th scope="row" data-sort-value="${escapeHtml(d.name)}"><a href="${routes.districtChange(d.irn)}">${escapeHtml(d.name)}</a></th>
        ${levels.map((v) => cell(money(v), v)).join("")}
        ${changes.map((x) => cell(`${signedPct(x.ratio)}<div class="n">${signedMoney(x.dollars)}</div>`, x.ratio)).join("")}
        ${cell(middle!.state_share == null ? "not published" : pct(middle!.state_share, 1), middle!.state_share)}
        ${cell(
          last!.state_share == null
            ? "not published"
            : `${pct(last!.state_share, 1)}${d.at_minimum_state_share ? `<div class="n">the minimum</div>` : ""}`,
          last!.state_share,
        )}
        ${
          measure === "total"
            ? `<td data-sort-value="${moved ? escapeHtml(moved.label) : ""}">${
                moved ? `${escapeHtml(moved.label)}<div class="n tnum">${signedMoney(moved.change)}</div>` : "none moved"
              }</td>`
            : ""
        }
      </tr>`;
    })
    .join("");
  return `<div class="measure-panel ${measure}">
      <p class="note">On <strong>${name}</strong>:</p>
      <ul class="note">${STEPS.map(([from, to]) => stepSentence(c.districts, measure, from, to)).join("")}</ul>
      ${neighborsChart(c.districts, measure, { from: 0, to: 2 })}
      <div class="scroll"><table data-sortable>
        <thead><tr>
          <th scope="col" aria-sort="none"><button type="button" data-sort="name">District</button></th>
          ${years.map((y, i) => head(`FY${y} ${i === 0 ? "paid" : "model"}`, `level-${i}`)).join("")}
          ${STEPS.map(([from, to]) => head(step(from, to), `step-${from}-${to}`, from === 0 && to === 2 ? "descending" : undefined)).join("")}
          ${head(`State share, FY${years[1]}`, "share-1")}
          ${head(`State share, FY${years[2]}`, "share-2")}
          ${measure === "total" ? `<th scope="col" aria-sort="none"><button type="button" data-sort="line">Line that moved most, ${step(0, 2)}</button></th>` : ""}
        </tr></thead>
        <tbody>${body}</tbody>
      </table></div>
    </div>`;
}

/** What the state share did from the middle year to the last, in a sentence. */
function shareSentence(c: County): string {
  const years = yearsOf(c.districts[0]!.biennium);
  const pairs = c.districts.filter(
    (d) => d.biennium.observed[1]!.state_share != null && d.biennium.observed[2]!.state_share != null,
  );
  const fell = pairs.filter((d) => d.biennium.observed[2]!.state_share! < d.biennium.observed[1]!.state_share!);
  const minimum = c.districts.filter((d) => d.at_minimum_state_share);
  const all = pairs.length === c.districts.length;
  const share =
    fell.length === pairs.length && pairs.length > 1
      ? `falls in every district ${all ? "here" : "that publishes it in both years"}`
      : `falls in ${count(fell.length)} of the ${count(pairs.length)} districts ${all ? "here" : "that publish it in both years"}`;
  return `<p class="note"><strong>State share</strong>, from FY${years[1]} to FY${years[2]},
    ${share}.${
      minimum.length
        ? ` ${names(minimum)} ${minimum.length === 1 ? "is" : "are"} at the formula's minimum state
          share in FY${years[2]}.`
        : ""
    } It is the part of base cost the state pays after a district's local capacity is charged, so
    it reaches the dollars through foundation aid; the FY${years[0]} payment report does not
    publish it.</p>`;
}

/** The repealed supplement, taken out of the first step for each district that had it. */
function withoutSentence(c: County): string {
  const years = yearsOf(c.districts[0]!.biennium);
  const had = c.districts
    .map((d) => ({ d, x: withoutTargetedAssistance(d) }))
    .filter((r): r is { d: District; x: { dollars: number; ratio: number } } => r.x != null);
  if (had.length === 0) return "";
  const each = had
    .map(
      ({ d, x }) =>
        `${escapeHtml(d.name)} ${signedPct(x.ratio)} (${money(d.biennium.observed[0]!.targeted_assistance)} of it in FY${years[0]})`,
    )
    .join("; ");
  return `<p class="note"><strong>Without supplemental targeted assistance</strong>, which H.B. 96
    repealed after FY${years[0]}, total state support from FY${years[0]} to FY${years[1]} moved:
    ${each}. ${had.length === 1 ? "That district's" : "Those districts'"} first step on total state
    support carries the whole supplement as a loss; this is what the rest of the payment did.</p>`;
}

/**
 * The card's finding (#653), on the step and measure its first chart draws: the whole biennium,
 * on total state support. The bullets beneath split it by step and the toggle by measure.
 */
export function finding(c: County): string {
  const years = yearsOf(c.districts[0]!.biennium);
  const when = `from FY${years[0]} to FY${years[2]}`;
  const name = MEASURES.total.name;
  if (c.districts.length === 1) {
    const d = c.districts[0]!;
    const x = levelChange(d.biennium, "total", 0, 2);
    const moved = { rise: `rose ${signedPct(x.ratio)}`, fall: `fell ${signedPct(x.ratio)}`, none: "did not move" };
    return `${escapeHtml(d.name)}'s ${name} ${moved[direction(x.dollars)]} ${when}.`;
  }
  const t = tally(c.districts, "total", 0, 2);
  const none = t.none.length ? `, and ${count(t.none.length)} did not move` : "";
  return `On ${name} ${when}, ${count(t.rise.length)} of the ${count(c.districts.length)} districts here rose and ${count(t.fall.length)} fell${none}.`;
}

/**
 * The card: both measures under the no-script toggle `renderNeighbors` uses, the state share
 * beside them, and the county attribution said where the reader is comparing.
 */
export function renderWhoGained(c: County): string {
  const id = routes.SECTIONS.county.whoGained;
  const years = yearsOf(c.districts[0]!.biennium);
  return `
    <div class="card measure-scope" id="${id}" data-part="${id}">
      <h2>${anchor(id)}Who gained, who lost${yearChip("biennium")}</h2>
      <p class="note"><strong>${finding(c)}</strong> Every district here in the three years, on the
        measure selected. FY${years[0]}
        is the department's final payment report; FY${years[1]} and FY${years[2]} are its models of
        those years, not payments. Changes are percentages of each district's own earlier year, so
        a city and a village read on one scale. Every figure is in nominal dollars.</p>
      <input class="vh" type="radio" name="${id}-measure" id="${id}-total"
        data-measure="total" aria-label="Measure: Total state support" checked />
      <input class="vh" type="radio" name="${id}-measure" id="${id}-foundation"
        data-measure="foundation" aria-label="Measure: Foundation aid" />
      <div class="basis">
        <label class="pill" for="${id}-total" data-measure="total">Total state support</label>
        <label class="pill" for="${id}-foundation" data-measure="foundation">Foundation aid</label>
        <span class="n">FY${years[0]} paid, FY${years[1]} and FY${years[2]} models</span>
      </div>
      ${panel(c, "total")}
      ${panel(c, "foundation")}
      <p class="note">The two measures can disagree in sign. <strong>Total state support</strong> is
        every line the department pays a district through; <strong>foundation aid</strong> is core
        foundation funding and the guarantee alone, and a district held at its base does not move on
        it whatever its other lines do. Each district's name opens its change tab, which splits its
        own figures by line.</p>
      ${withoutSentence(c)}
      ${shareSentence(c)}
      <p class="note">A district whose territory crosses a county line is filed under the one
        county the department assigns it, so it appears on that county's page and on no other.</p>
    </div>`;
}
