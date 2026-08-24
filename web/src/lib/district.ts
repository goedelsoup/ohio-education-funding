/**
 * The district view: what the formula computes for one district, and what it actually gets.
 *
 * Each card is exported on its own rather than assembled here. The view used to be one function
 * because it was one tab; it is now five routes — the dashboard, the outcome, the finances, the
 * property tax and the scenario — and which cards go on which page is a routing decision, made in
 * `src/pages/`. Property tax landed after the other four and this sentence went on saying four for
 * six phases, which is why it is worth writing the count down in only one place per file.
 *
 * Every function here is a pure string builder with no DOM behind it, which is what lets Astro
 * call them at build time and put the result in the document.
 */

import type { FanPoint } from "./chart.ts";
import { barSpec, distributionSpec, type Drawing, fanSpec, nearestRank } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import { count, escapeHtml, money, ordinal, pct, percentileOf, signedMoney } from "./format.ts";
import { apply, currentLaw, currentRealizedAid } from "./policy.ts";
import { forecastPath, growthPrior, observations } from "./project.ts";
import { realChange, series, type Basis } from "./real.ts";
import { hasDenominators } from "./tax.ts";
import * as routes from "./routes.ts";
import type { Bar } from "./chart.ts";
import type { Bundle, District, Statewide } from "./types.ts";
import { seriesSpanEnd, yearChip, yearOf } from "./year.ts";
import { anchor } from "./section.ts";

function strip(
  label: string,
  value: number | null,
  sorted: number[],
  population: District[],
  measure: (d: District) => number | null | undefined,
  href?: string,
): string {
  const named = href
    ? `<a href="${href}">${escapeHtml(label)}</a>`
    : escapeHtml(label);
  if (value == null) return `<p class="note">${named}: not reported.</p>`;
  const p = percentileOf(sorted, value);

  /*
   * The distribution, not a flat bar with a pin in it.
   *
   * The bar this replaced drew the minimum at one end, the maximum at the other, and nothing in
   * between — so a district at the 60th percentile and one at the 95th looked equally at home,
   * when the first is in a dense middle and the second is nearly alone. These measures are the
   * ones where that matters most: assessed valuation per pupil runs to five and a half times the
   * median, and the top of the range is one district.
   *
   * The percentile stays underneath. It is the sentence a reader repeats and the box is what makes
   * it mean something.
   */
  const box: Drawing = (w) =>
    distributionSpec(
      population
        .map((d) => ({ d, v: measure(d) }))
        .filter((row): row is { d: District; v: number } => row.v != null)
        .map(({ d, v }) => ({ value: v, hover: `${d.name}: ${money(v)}` })),
      { width: w, marker: { value, label } },
    );

  return `
    <div class="strip-row">
      <div class="strip-head">
        <span>${named}</span>
        <strong class="tnum">${money(value)}
          <span class="n">${ordinal(Math.round(p * 100))} percentile</span></strong>
      </div>
      <div class="chartwrap" data-chart="position">${renderToString(box, { label: `${label} across the ${count(sorted.length)} districts reporting it, with this district at ${money(value)}, the ${ordinal(Math.round(p * 100))} percentile`, description: `Quartiles run ${money(nearestRank(sorted, 0.25))} to ${money(nearestRank(sorted, 0.75))}, median ${money(nearestRank(sorted, 0.5))}` })}</div>
      <div class="scale">
        <span>${money(sorted[0] ?? 0)}</span>
        <span>${money(sorted[sorted.length - 1] ?? 0)}</span>
      </div>
    </div>`;
}

/**
 * What a year of enrollment is worth to this district.
 *
 * # What this is not
 *
 * It is **not** the district's actual FY2026 state aid against its actual FY2027 state aid. The
 * department publishes one funding calculator at a time and replaces it rather than archiving
 * it, so this repository has the FY2027 model and no FY2026 one; there is no retrieved source
 * anywhere in it carrying a district's FY2026 foundation payment. A card that put a number under
 * "FY2026" would be inventing it.
 *
 * What it is instead is exact and, for the question people actually ask, more useful: the
 * **FY2027 formula held completely fixed and run at each year's enrolled ADM**. Every other
 * input — base cost, local capacity, the categoricals, the guarantee baseline — is identical
 * across the rows, so the difference between them is the enrollment channel and nothing else.
 * Comparing two years of published totals could not have said that, because the formula changed
 * between them too.
 *
 * The table is exact and the band underneath it is not, so `forecastable` gates only the second.
 * A build whose projection failed its checkpoints still shows every row here: the rows are the
 * FY2027 formula run at three published enrollments, and nothing about them is a forecast.
 */
export function renderEnrollmentYears(
  bundle: Bundle,
  d: District,
  forecastable: boolean,
): string {
  const meta = bundle.projection;
  if (!meta) return "";
  const model = bundle.statewide.minimum_state_share;
  const law = currentLaw(model);
  const history = observations(d, meta.base_year);

  const rows = history.map((o) => ({
    year: o.fiscalYear,
    adm: o.value,
    aid: apply(d, law, o.value, model).realizedAid,
  }));
  const latest = rows[rows.length - 1]!;
  const prior = rows[rows.length - 2]!;
  const change = latest.aid - prior.aid;
  const admChange = latest.adm - prior.adm;

  const carried = forecastable
    ? renderCarriedForward(bundle, d)
    : `<h3>Carried forward</h3>
       <p class="note">No band is drawn. This build's projection did not reproduce the forecasts
         <code>crates/project</code> computed for it, so the enrollment carry-forward is withheld
         rather than shown unchecked. The table above is unaffected — it is the
         FY${bundle.fiscal_year} formula at three published enrollments, and nothing in it is a
         forecast.</p>`;

  return `
    <div class="card" id="enrollment" data-part="enrollment">
      <h2>${anchor("enrollment")}What a year of enrollment is worth here${yearChip("enrollment")}</h2>
      <div class="scroll"><table>
        <thead><tr>
          <th>Enrollment year</th><th>Enrolled ADM</th><th>State aid at that ADM</th>
        </tr></thead>
        <tbody>${rows
          .map(
            (r) => `<tr${r.year === meta.base_year ? ' class="current"' : ""}>
              <th>FY${r.year}${r.year === meta.base_year ? " — the model's own" : ""}</th>
              <td class="tnum">${count(Math.round(r.adm))}</td>
              <td class="tnum">${money(r.aid)}</td>
            </tr>`,
          )
          .join("")}</tbody>
      </table></div>
      <p class="note">Moving from FY${prior.year} to FY${latest.year} enrollment
        ${admChange < 0 ? "cost" : "gained"} this district
        ${count(Math.abs(Math.round(admChange)))} pupils and
        <strong class="${change < 0 ? "loss" : "gain"}">${signedMoney(change)}</strong>
        ${
          Math.abs(change) < 0.5 && d.on_guarantee
            ? `— nothing, because the guarantee holds its aid at a fixed dollar amount that
               enrollment does not enter.`
            : `, or ${signedMoney(latest.adm > 0 ? change / latest.adm : 0, 2)} per pupil.`
        }</p>
      <p class="note">These are <strong>not</strong> published FY${prior.year} and
        FY${latest.year} funding totals — the department publishes one calculator at a time and
        this repository holds the FY${bundle.fiscal_year} one, so no FY${prior.year} payment
        figure exists here to show. Every row is the FY${bundle.fiscal_year} formula held fixed
        and run at that year's enrolled ADM, which isolates the enrollment channel: two years of
        published totals could not, because the formula moved between them too.</p>
      <p class="note"><strong>The last row's enrollment is partly a departmental
        estimate.</strong> The FY${bundle.fiscal_year} calculator is published before
        FY${latest.year} closes, so FY${latest.year} enrolled ADM is not a settled count — and it
        is the row the model's own answer sits on. Every figure on this page that divides by the
        current year inherits that, which is a different kind of uncertainty from the band below:
        the band is a forecast and says so, this is a published number that is not final.</p>
      <p class="note">This card holds the formula fixed and moves enrollment. The
        <a href="${routes.districtScenario(d.irn)}">scenario for this district</a> does the
        opposite — it holds enrollment at published FY${meta.base_year} and moves the formula. The
        two were built a phase apart and neither said the other existed, which left one district's
        counterfactual on one route and one district's projection on another with no way to get
        between them.</p>

      ${carried}
    </div>`;
}

/**
 * This district's aid carried forward to its own enrollment band.
 *
 * The same verified path the statewide fan uses, over one district instead of 606. For a district
 * the guarantee pays it collapses to a flat line — its aid does not respond to its enrollment at
 * all — and that is the finding rather than a broken chart, so the copy says so and a second
 * series is added showing what the formula would have computed. The vertical gap between the two
 * is what the guarantee is worth to it.
 */
function renderCarriedForward(bundle: Bundle, d: District): string {
  const meta = bundle.projection;
  if (!meta) return "";
  const model = bundle.statewide.minimum_state_share;
  const path = forecastPath(
    [d],
    currentLaw(model),
    meta.base_year + 6,
    meta.base_year,
    meta.method,
    meta.damping,
    growthPrior(bundle.districts, meta.z),
    model,
  );
  const end = path[path.length - 1]!;
  const insensitive = end.high - end.low < Math.max(1, end.realizedAid * 0.0005);
  const points: FanPoint[] = path.map((p) => ({
    year: p.fiscalYear,
    point: p.realizedAid,
    low: p.low,
    high: p.high,
    observed: p.observed,
    // The second line, only where the first one says nothing. A guaranteed district's aid is a
    // fixed dollar amount, so its band is a horizontal line and a chart of it alone is a chart
    // of nothing; what the formula computes for it falls with enrollment, and the gap between
    // the two is the guarantee doing its job. Spread rather than `undefined`, because the
    // property is optional and an explicit undefined is not the same thing.
    ...(insensitive ? { reference: p.formulaAid } : {}),
  }));
  const gapNow = path.find((p) => p.fiscalYear === meta.base_year);
  const widening =
    gapNow && end.realizedAid - end.formulaAid - (gapNow.realizedAid - gapNow.formulaAid);

  return `
    <h3>Carried forward</h3>
    <div class="chartwrap" data-chart="district-fan">${renderToString((w) => fanSpec(
      points,
      (v) => money(v),
      (p) =>
        p.observed
          ? `FY${p.year}: ${money(p.point)} at published enrollment — exact`
          : `FY${p.year}: ${money(p.low)} – ${money(p.high)}, central ${money(p.point)}`,
      { width: w },
    ), { label: `${insensitive ? `Aid this district receives against what the formula computes for it, in dollars by enrollment year, FY${points[0]!.year} to FY${end.fiscalYear}` : `State aid in dollars by enrollment year, FY${points[0]!.year} to FY${end.fiscalYear}, with the range across projected enrollment`}`, description: `Every year is the FY${bundle.fiscal_year} formula run at that year's enrollment rather than published aid; years through FY${meta.base_year} use published enrollment and carry no interval, and the y axis is truncated to the plotted range rather than starting at zero` })}</div>
    ${
      insensitive
        ? `<div class="legend">
             <span><i class="sw formula"></i> What the district receives</span>
             <span><i class="sw guarantee"></i> What the formula computes</span>
           </div>`
        : ""
    }
    <p class="note">${
      insensitive
        ? `<strong>The flat line is what this district receives, and it is flat by
           construction.</strong> The guarantee pays a fixed dollar amount that enrollment does
           not enter, so no forecast of its enrollment moves it and it has no band. The line
           that falls is what the formula computes at that enrollment
           — ${money(end.formulaAid)} by FY${end.fiscalYear} against ${money(end.realizedAid)}
           received. The gap between them is the guarantee, and on this projection it
           ${(widening ?? 0) > 0 ? "widens by" : "narrows by"}
           ${money(Math.abs(widening ?? 0))} over the horizon.`
        : `The range, not the line, is the finding: at FY${end.fiscalYear} enrollment this
           district's aid is somewhere between ${money(end.low)} and ${money(end.high)}. The
           band is the cross-sectional spread of district enrollment growth, not this
           district's own history — three observations cannot give that.`
    }</p>`;
}

/** Fiscal years federal pandemic relief makes uncomparable to what came before and after. */
const PANDEMIC_YEARS = [2021, 2022, 2023, 2024];

/**
 * The fiscal year the casino closure lands in.
 *
 * FY2021 and not FY2020, because the fund pays in August for the half-year that ended in June: the
 * January-June 2020 collection was distributed in August 2020, which is FY2021. The two bases
 * disagree about which year the pandemic hit by exactly one year, and this constant is which one
 * the feed is on.
 */
const CASINO_CLOSURE_YEAR = 2021;

/**
 * The median district's casino distribution as a share of the state aid it booked the same year.
 *
 * Printed so a district's own share has something to be large or small against.
 */
const CASINO_MEDIAN_SHARE_OF_AID = 0.0115;

/**
 * Districts whose casino money comes out of exactly one county fund.
 *
 * This and [`CASINO_MEDIAN_SHARE_OF_AID`] are the two statewide statistics this card types rather
 * than derives, because neither is in the feed — one is a median over a join the feed does not
 * make and the other is a count over a nullable field. Both are asserted against the feed in
 * `tests/unit/casino.spec.ts`, which is the difference between a figure that is checked and one
 * that was true when it was written.
 */
const CASINO_SINGLE_COUNTY_DISTRICTS = 178;

/**
 * What the district actually received, raised, spent, and holds.
 *
 * # This is the only card on the page that is not a model
 *
 * Every other figure here is computed: what the FY2027 formula says a district is owed, what it
 * spent per pupil on the department's definitions, what its pupils achieved. These are audited
 * actuals from the district's own five-year forecast filing — money that changed hands.
 *
 * The two are differently constructed and this card never presents one as a check on the other.
 * The general fund is also not the whole budget: capital, food service, and most federal
 * programmes sit in other funds, so the spending here is not the district's total.
 *
 * Rendered once per basis and switched by `BasisToggle.astro`, which is two radio inputs and a
 * sibling selector rather than a script — so the switch works with JavaScript off. That is why
 * the basis control is no longer emitted here: the card is now the thing being toggled, not the
 * thing holding the toggle.
 */
export function renderActuals(bundle: Bundle, d: District, basis: Basis): string {
  if (d.finances.length === 0) return "";
  const { years: shown, converted, base } = series(bundle.deflator, d.finances, basis);
  const first = shown[0]!;
  const latest = shown[shown.length - 1]!;
  const cashChange =
    latest.ending_cash == null || first.ending_cash == null
      ? null
      : latest.ending_cash - first.ending_cash;
  const yearsOfSpending =
    latest.ending_cash != null && latest.total_expenditure != null && latest.total_expenditure > 0
      ? latest.ending_cash / latest.total_expenditure
      : null;
  // Counted on the published figures: whether a year was run at a deficit is a fact about that
  // year's own dollars, and deflating both sides cannot change it. Counted only over the years
  // that reported both lines — a filing that carried no expenditure line did not run a surplus.
  const comparable = d.finances.filter(
    (y) => y.total_expenditure != null && y.total_revenue != null,
  );
  const deficits = comparable.filter((y) => y.total_expenditure! > y.total_revenue!).length;
  const realAid = realChange(bundle.deflator, d.finances, (y) => y.state_aid);
  const nominalAid =
    d.finances[d.finances.length - 1]!.state_aid == null || (d.finances[0]!.state_aid ?? 0) <= 0
      ? null
      : d.finances[d.finances.length - 1]!.state_aid! / d.finances[0]!.state_aid! - 1;

  // Years with no reported balance are left out of the chart rather than drawn at zero, and
  // named under it. A bar of length nothing is a claim that the district held nothing.
  const held = shown.filter((y) => y.ending_cash != null);
  const unreported = shown.filter((y) => y.ending_cash == null);
  const peak = held.reduce<(typeof held)[number] | null>(
    (a, b) => (a == null || b.ending_cash! > a.ending_cash! ? b : a),
    null,
  );
  const bars = held.map((y) => ({
    label: `FY${y.fiscal_year}`,
    value: y.ending_cash!,
    hover: `FY${y.fiscal_year}: ${money(y.ending_cash)} held, ${money(y.total_revenue)} in, ${money(y.total_expenditure)} out`,
    ...(y.fiscal_year === peak?.fiscal_year || y.fiscal_year === latest.fiscal_year
      ? { direct: money(y.ending_cash) }
      : {}),
  }));

  return `
    <div class="card" data-part="actuals">
      <h2>${anchor("actuals")}What it actually received, and what it holds${
        converted ? `, in FY${base} dollars` : ""
      }${yearChip("formula")}</h2>
      <div class="tiles">
        <div class="tile"><div class="k">Cash on hand, FY${latest.fiscal_year}</div>
          <div class="v">${money(latest.ending_cash)}</div>
          <div class="n">${
            yearsOfSpending == null
              ? "no spending reported"
              : `${yearsOfSpending.toFixed(2)} years of spending at this rate`
          }</div></div>
        <div class="tile"><div class="k">Change since FY${first.fiscal_year}</div>
          <div class="v ${cashChange != null && cashChange < 0 ? "loss" : "gain"}">${signedMoney(cashChange)}</div>
          <div class="n">carry-over into FY${latest.fiscal_year + 1} is
            ${money(latest.ending_cash)}</div></div>
        <div class="tile"><div class="k">State aid, FY${first.fiscal_year}–FY${latest.fiscal_year}</div>
          <div class="v ${(realAid ?? 0) < 0 ? "loss" : "gain"}">${
            realAid == null ? "—" : pct(realAid, 1)
          }</div>
          <div class="n">real; ${pct(nominalAid, 1)} nominal. ${deficits} of
            ${comparable.length} years run at a deficit</div></div>
      </div>

      ${
        bars.length === 0
          ? ""
          : `<div class="chartwrap" data-chart="cash">${renderToString((w) => barSpec(bars, { width: w }), { label: `General fund cash held at 30 June by fiscal year, FY${first.fiscal_year} to FY${latest.fiscal_year}${converted ? `, in FY${base} dollars` : ""}` })}</div>`
      }
      ${
        unreported.length === 0
          ? ""
          : `<p class="note">This district's filing carries no cash balance for
            ${unreported.map((y) => `FY${y.fiscal_year}`).join(", ")}. Those years are left out
            of the chart and stand as an em dash in the table rather than being drawn at zero:
            the line is absent from the published filing, which is not the same as a district
            reporting that it held nothing.</p>`
      }
      <p class="note">Cash held at 30 June, general fund${
        converted ? `, in FY${base} dollars — deflated with ${escapeHtml(bundle.deflator?.label ?? "an index")}` : ""
      }.
        FY${PANDEMIC_YEARS[0]}–FY${PANDEMIC_YEARS[PANDEMIC_YEARS.length - 1]} are the federal
        pandemic relief years: that money was booked in the general fund by some districts and
        separately by others, so a balance rising across them is not evidence about this
        district's own position.</p>

      <div class="scroll"><table>
        <thead><tr>
          <th>Fiscal year</th><th>State aid</th><th>Local tax</th>
          <th>Revenue</th><th>Spending</th><th>Held at 30 June</th>
        </tr></thead>
        <tbody>${shown
          .map(
            (y) => `<tr>
              <th>FY${y.fiscal_year}</th>
              <td class="tnum">${money(y.state_aid)}</td>
              <td class="tnum">${money(y.local_tax)}</td>
              <td class="tnum">${money(y.total_revenue)}</td>
              <td class="tnum">${money(y.total_expenditure)}</td>
              <td class="tnum">${money(y.ending_cash)}</td>
            </tr>`,
          )
          .join("")}</tbody>
      </table></div>
      <p class="note">These are <strong>audited actuals</strong> from this district's own
        five-year forecast filing — the only figures on this page that record money changing
        hands rather than a formula's output. State aid here is unrestricted grants-in-aid as the
        treasurer books it, which is <em>not</em> the same construction as the
        FY${2027} calculator's total state support above; the two are not comparable line for
        line. Local tax is property plus income tax actually collected. General fund only, so
        capital, food service, and most federal programmes are outside it.</p>
    </div>`;
}

/**
 * One condition the formula puts this district in, and what follows from it.
 *
 * The `label` alone is what this card used to print, as a pill. "At the minimum state share" is a
 * noun phrase: it states a fact about 138 districts and gives a reader who does not already know
 * the plan no way to reach what it means for them. A pill cannot hold a clause, so these are rows.
 */
interface Condition {
  label: string;
  href?: string;
  consequence: string;
}

/**
 * Where this district's state aid comes from — the first card on the page, and the answer to it.
 *
 * # Why the headline tiles are inside this card
 *
 * They were a separate block above it, and two of the three figures were printed twice within one
 * screen: the aid-per-pupil tile and its guarantee sub-note were the bar's legend verbatim, and the
 * bar's own `aria-label` was a third printing of the same pair. Merging without deduplicating would
 * have been churn — the same figures under fewer headings — so the pair is printed once, in the
 * tiles, and the legend now names the two segments instead of restating their amounts.
 *
 * # Why the first figure changed
 *
 * It was base cost per pupil, which is an input to the formula rather than money anyone receives.
 * This page's `<meta description>`, its OG image alt text and this card's own heading all say
 * independently that the question is state aid and where it comes from, and the page opened by
 * answering a different one. Base cost keeps its own card, two below, where its twenty-two
 * elements are.
 *
 * # Why the total is summed rather than multiplied
 *
 * `realized_aid_per_pupil` divides by base cost ADM. The pupil count this page prints beside it is
 * the current-year one, and multiplying the per-pupil figure by the count next to it is wrong by a
 * median $112,601 and by $8.2m for Cleveland. `currentRealizedAid` sums the components instead,
 * which is exact — the same reason `scenario.astro` reaches for it.
 */
export function renderAidSource(bundle: Bundle, d: District): string {
  const formulaPP = d.formula_aid_per_pupil;
  const realizedPP = d.realized_aid_per_pupil;
  const guaranteePP = realizedPP - formulaPP;
  const total = Math.max(realizedPP, 1);
  const formulaWidth = (formulaPP / total) * 100;
  const guaranteeWidth = (guaranteePP / total) * 100;

  /*
   * Two chips are gone and neither is a loss. "Funded by the guarantee, not the formula" and "At or
   * below the 20-mill floor" both restated the sub-line `[irn].astro` renders directly above this
   * card, and the floor chip was strictly *less* precise than what it duplicated: the sub-line
   * separates the 21 districts genuinely below 20 mills from the 155 sitting on it, and the chip
   * collapsed all 176 into one phrase. What survives is what the sub-line does not already say.
   */
  const conditions: Condition[] = [];
  if (d.near_millage_floor && !d.at_millage_floor) {
    conditions.push({
      label: "Within a twentieth of a mill of the floor",
      href: routes.parameter("twenty-mill-floor"),
      consequence: `One more reduction and H.B. 920 stops rolling its rate back at all, after
        which a reappraisal reaches its revenue instead of being taken away again. One of 54.`,
    });
  }
  if (d.at_minimum_state_share) {
    conditions.push({
      label: "At the minimum state share",
      href: routes.metric("state-share-percentage"),
      consequence: `The formula computed a share below 10% and the statute raised it to 10%, so a
        change to the local capacity measure moves this district's aid by nothing until it is large
        enough to lift it off the floor. One of 138.`,
    });
  }
  if (d.millage?.cumulative_reduction != null && d.millage.cumulative_reduction > 0.005) {
    conditions.push({
      label: `${pct(d.millage.cumulative_reduction, 0)} of voted millage reduced away`,
      href: routes.metric("effective-operating-millage"),
      consequence: `H.B. 920's reduction factors have rolled back that much of what voters
        approved, so what a levy raises here is not what the ballot said it would.`,
    });
  }
  /*
   * Symmetric, because a fall is not the only thing that happened. This fired only on a negative,
   * so for the 109 districts whose enrollment *rose* between FY2024 and FY2026 the dashboard's sole
   * carrier of the fact was a row in the Detail card — which is why this has to change before that
   * card is deleted rather than after.
   */
  if (d.enrollment_change != null && d.enrollment_change !== 0) {
    const fell = d.enrollment_change < 0;
    conditions.push({
      label: `Enrollment ${fell ? "down" : "up"} ${pct(
        Math.abs(d.enrollment_change),
      )} ${yearOf("enrollment").replace("-", "→")}`,
      consequence: d.on_guarantee
        ? `Its aid did not move with it, because the guarantee holds a fixed dollar amount
           enrollment does not enter — the next card shows what that is worth.`
        : `Its aid ${
            fell ? "falls" : "rises"
          } with it, because this district is on the formula — the next card shows by how much.`,
    });
  }

  return `
    <div class="card" id="aid-source" data-part="aid-source">
      <h2>${anchor("aid-source")}Where the state aid comes from${yearChip("formula")}</h2>
      <div class="tiles" data-part="headline">
        <div class="tile"><div class="k">State aid, FY${bundle.fiscal_year}</div>
          <div class="v">${money(currentRealizedAid(d))}</div>
          <div class="n">every foundation dollar the model pays it</div></div>
        <div class="tile"><div class="k">State aid / pupil</div>
          <div class="v">${money(realizedPP)}</div>
          <div class="n">${
            d.on_guarantee
              ? money(guaranteePP) + " of it from the guarantee"
              : "all from the formula"
          }</div></div>
        <div class="tile"><div class="k">Base-cost ADM</div>
          <div class="v">${count(Math.round(d.adm))}</div>
          <div class="n"><a href="${routes.metric(
            "enrolled-adm",
          )}">not a headcount</a> — a three-year average</div></div>
      </div>
      ${
        /*
         * The bar is a proportion chart and 315 districts have one proportion. A full-width single
         * segment under a legend that has collapsed to one item says nothing the tiles did not, so
         * those districts get the prose and no chart. Nothing is lost by the omission: where the
         * guarantee pays zero, realized aid per pupil *is* formula aid per pupil, for all 315.
         */
        guaranteeWidth > 0
          ? `<div class="barwrap">
               <div class="bar" role="img" aria-label="${pct(
                 formulaPP / realizedPP,
                 0,
               )} of this district's aid per pupil is computed by the formula; the guarantee pays the rest">
                 <div class="seg formula" style="width:${formulaWidth}%"></div>
                 <div class="seg guarantee" style="width:${guaranteeWidth}%"></div>
               </div>
               <div class="legend">
                 <span><i class="sw formula"></i> What the formula computes</span>
                 <span><i class="sw guarantee"></i> What the guarantee adds</span>
               </div>
             </div>`
          : ""
      }
      <p class="note">${
        d.on_guarantee
          ? `The formula computes ${money(formulaPP)} per pupil. This district receives
             ${money(realizedPP)} because the <a href="${routes.wikiNode(
               "formula-component",
               "temporary-transitional-aid-guarantee",
             )}">temporary transitional aid guarantee</a> holds it at what it received in
             <strong>FY2020</strong> — a year Ohio froze funding under the Bridge formula rather
             than computing it. The formula produces
             <strong>${pct(formulaPP / realizedPP, 0)}</strong> of that level.`
          : `This district is funded by the formula, so an increase in its computed base cost
             reaches it in full — unlike the ${bundle.statewide.on_guarantee} districts held on the
             <a href="${routes.wikiNode(
               "formula-component",
               "temporary-transitional-aid-guarantee",
             )}">guarantee</a>.`
      }</p>
      ${
        // Marion Local trips none of the four, and an empty row renders as an unexplained gap
        // rather than as an absence.
        conditions.length === 0
          ? ""
          : `<ul class="conditions">${conditions
              .map(
                (c) => `<li><strong>${
                  c.href ? `<a href="${c.href}">${escapeHtml(c.label)}</a>` : escapeHtml(c.label)
                }</strong> — ${c.consequence}</li>`,
              )
              .join("")}</ul>`
      }
      ${renderHoldHarmless(d)}
    </div>`;
}

/**
 * What the guarantee actually is, where a district is touched by it.
 *
 * # It is not "hold the district at its old amount"
 *
 * Two things sit inside the figure the bar above calls "guarantee", and neither is visible in it:
 *
 * - an **open-enrolment clawback**. A guaranteed district whose open enrolment FTE has fallen by
 *   more than `max(10% of last year, 20 FTE)` has its guarantee reduced by the **statewide average
 *   base cost per pupil** for every FTE beyond that — $8,241.61 each, at full value rather than the
 *   district's state share. 43 districts, $5.1m withheld.
 * - a **second hold-harmless** above the first. The formula transition supplement holds a district
 *   at a larger FY2021 base, one that includes transportation, and reaches 144 districts of which
 *   17 draw nothing from the guarantee.
 *
 * With transportation's own guarantee that is three mechanisms anchored to FY2021, on three
 * different bases. This block renders only the parts that apply, so a district touched by none of
 * them shows nothing.
 *
 * # The guard is a union, and it has to be
 *
 * It used to be `clawback || supplement`, which meant **158 of the 294 guaranteed districts** —
 * every one carrying a guarantee and neither mechanism — had no table here at all, and their
 * guarantee dollar total appeared on the dashboard only in the Detail card. Widening it to
 * `guarantee > 0` *instead* would have been worse than leaving it: 22 districts have a clawback
 * and no guarantee, 17 draw the supplement while drawing nothing from the guarantee, and the
 * paragraph below names those 17 explicitly — so a replacement would have deleted the table from
 * precisely the districts that sentence is about. 331 districts render it now, against 173 before.
 */
function renderHoldHarmless(d: District): string {
  const t = d.transition;
  const clawback = t.open_enrollment_adjustment > 0;
  const supplement = t.transition_supplement > 0;
  if (d.guarantee <= 0 && !clawback && !supplement) return "";

  const lost = t.open_enrollment_prior - t.open_enrollment_current;
  const beyond = Math.max(lost - t.open_enrollment_threshold, 0);

  return `
    <div class="scroll"><table data-program="hold-harmless">
      <thead><tr><th>Step</th><th class="tnum">Value</th><th>What it is</th></tr></thead>
      <tbody>
        <tr><th>FY2021 funding base</th><td class="tnum">${money(t.funding_base)}</td>
          <td class="n">What the guarantee compares formula funding against.</td></tr>
        ${
          clawback
            ? `<tr class="current"><th>Open-enrolment clawback</th>
                <td class="tnum">&minus;${money(t.open_enrollment_adjustment)}</td>
                <td class="n">Its open enrolment fell ${lost.toFixed(1)} FTE against a threshold of
                  ${t.open_enrollment_threshold.toFixed(1)}, and the guarantee is cut by
                  ${money(8241.61)} — the statewide average base cost per pupil — for each of the
                  ${beyond.toFixed(1)} FTE beyond it.</td></tr>`
            : ""
        }
        <tr><th>Guarantee</th><td class="tnum">${
          d.guarantee <= 0 ? "—" : money(d.guarantee)
        }</td>
          <td class="n">${
            d.guarantee > 0
              ? "The base, less the clawback, less what the formula computes."
              : "The formula reaches its FY2021 base, so this pays nothing."
          }</td></tr>
        ${
          supplement
            ? `<tr class="current"><th><a href="${routes.wikiNode(
                "formula-component",
                "fsfp-formula-transition-supplement",
              )}">Formula transition supplement</a></th>
                <td class="tnum">${money(t.transition_supplement)}</td>
                <td class="n">A <em>second</em> hold-harmless, against a larger FY2021 base of
                  ${money(t.fy21_funding_base)} that includes transportation.</td></tr>`
            : ""
        }
      </tbody>
    </table></div>
    ${
      clawback
        ? `<p class="note"><strong>The clawback is charged at the full per-pupil base cost, not at
           this district's share of it.</strong> The state was paying its
           <em>share</em> of ${money(8241.61)} for each of those pupils — for a district at the 10%
           minimum that is about ${money(824)} — and the guarantee falls by the whole
           ${money(8241.61)} when they leave. Whether that is a deliberate incentive or an artefact of reaching for a
           convenient statewide figure is not established here.</p>`
        : ""
    }
    ${
      supplement
        ? `<p class="note">Ohio has <strong>three</strong> hold-harmless mechanisms anchored to
           FY2021, on three different bases: the guarantee above, this supplement, and a separate
           one inside transportation. They hold overlapping but different sets of districts —
           17 draw this supplement while drawing nothing from the guarantee.</p>`
        : ""
    }`;
}

/**
 * Where this district sits in the statewide distribution.
 *
 * A marker on a neutral strip rather than a chart: the question is "where does this district
 * sit", which is one value against a range and not a series. Both labels link out to the corpus
 * node that defines the measure, because the percentile is only meaningful to a reader who knows
 * what is being divided by what.
 */
export function renderPosition(
  bundle: Bundle,
  d: District,
  valuations: number[],
  expenditures: number[],
): string {
  return `
    <div class="card" id="position" data-part="position">
      <h2>${anchor("position")}Position among Ohio's ${bundle.statewide.districts} districts${yearChip("formula")}</h2>
      <p class="note">The middle half of Ohio's districts is the shaded box, the line inside it is
        the median, and the whiskers reach the last district within one and a half times the
        box's width. Anything drawn past them is its own district. The coloured rule is this one.</p>
      ${strip(
        "Assessed valuation per pupil",
        d.valuation_per_pupil,
        valuations,
        bundle.districts,
        (other) => other.valuation_per_pupil,
        routes.metric("assessed-valuation-per-pupil"),
      )}
      ${strip(
        "Operating expenditure per pupil",
        d.operating_expenditure_per_pupil,
        expenditures,
        bundle.districts,
        (other) => other.operating_expenditure_per_pupil,
        routes.metric("per-pupil-operating-expenditure"),
      )}
      <p class="note">This card places the district against all ${bundle.statewide.districts} at
        once, which is the question "is this unusual". The other question — "unusual compared to
        <em>whom</em>" — is <a href="${routes.compare(d.irn)}">the comparison table</a>, which puts
        this district's figures beside any one other district's.</p>
    </div>`;
}

/**
 * The categorical half of formula aid, as its six programs.
 *
 * # Why one number was not enough
 *
 * The corpus carried this as a residual for eight phases: core foundation funding less the state
 * share of base cost. That is exact — it reconciles to the cent for all 609 districts — and it is
 * a number a reader can do nothing with. It is also **43% of formula aid**, against a base cost
 * half this project decomposed into the twenty-two elements of R.C. 3317.011 long ago.
 *
 * The six do not behave alike, and that is the reason the sum misleads rather than merely
 * omitting. **Targeted assistance is equalisation** — it is the largest program in the state at
 * $1.36bn and it falls to exactly zero for 135 districts, because they have too much valuation to
 * qualify. **DPIA is poverty-driven.** A district whose categorical total rose could be poorer or
 * could be less wealthy, and the total cannot tell you which.
 */
export function renderCategoricals(d: District, statewide: Statewide): string {
  const c = d.categoricals;
  const total = d.categorical_funding;
  if (total <= 0) return "";

  // Each program links to the corpus node describing its mechanism. The six were one residual
  // until recently and had nothing to link to; now each is a `formula-component` in its own right,
  // and the link out from a figure to what the figure *means* is the reason the wiki exists.
  const parts = [
    ["Targeted assistance", c.targeted_assistance, "Equalisation for low-valuation districts. Zero once a district has enough.", "fsfp-targeted-assistance"],
    ["Special education", c.special_education, "Six weighted categories of disability, spanning a factor of sixteen.", "fsfp-special-education-weights"],
    ["Disadvantaged Pupil Impact Aid", c.dpia, "Two poverty counts blended, indexed on the state's, and squared.", "fsfp-disadvantaged-pupil-impact-aid"],
    ["Career-technical education", c.career_technical, "Five weights against a base cost 20% above the one the rest of the plan uses.", "fsfp-career-technical-weights"],
    ["Gifted", c.gifted, "Mostly staffing units, with a floor 370 districts sit on.", "fsfp-gifted-units"],
    // "unlike every other categorical" until this commit, which is the same over-claim the drill-down
    // below carried and the same one the corpus node carried: career-technical's five weights descend
    // too. What is true of this program alone is that its weights fall along a *need* gradient.
    ["English learners", c.english_learners, "Three weights that descend as need persists, alone among the six.", "fsfp-english-learner-weights"],
  ] as const;

  const bars: Bar[] = parts
    .filter(([, value]) => value > 0)
    .map(([label, value]) => ({
      label,
      value,
      direct: pct(value / total, 0),
      hover: `${label}: ${money(value)}, ${pct(value / total, 1)} of this district's categoricals`,
    }));

  const share = total / (total + d.base_cost_state_share);

  return `
    <div class="card" id="categoricals" data-part="categoricals">
      <h2>${anchor("categoricals")}The categorical half, in its six parts${yearChip("formula")}</h2>
      <div class="chartwrap" data-chart="categoricals">${renderToString((w) => barSpec(bars, { width: w }), { label: `This district's categorical funding by program, in dollars, ${yearOf("formula")}` })}</div>

      <div class="scroll"><table>
        <thead><tr><th>Program</th><th class="tnum">Amount</th><th class="tnum">Share</th></tr></thead>
        <tbody>
          ${parts
            .map(
              ([label, value, note, node]) => `<tr${value <= 0 ? ' class="n"' : ""}>
                <th><a href="${routes.wikiNode("formula-component", node)}">${escapeHtml(
                  label,
                )}</a>${note ? `<div class="n">${escapeHtml(note)}</div>` : ""}</th>
                <td class="tnum">${value <= 0 ? "—" : money(value)}</td>
                <td class="tnum n">${value <= 0 ? "" : pct(value / total, 1)}</td>
              </tr>`,
            )
            .join("")}
          <tr class="current"><th>Total categorical funding</th>
            <td class="tnum">${money(total)}</td><td class="tnum">100.0%</td></tr>
        </tbody>
      </table></div>

      <p class="note">${pct(share, 0)} of this district's formula aid is categorical rather than
        base cost — ${money(total)} against ${money(d.base_cost_state_share)}. ${
          c.targeted_assistance <= 0
            ? `<strong>It receives no targeted assistance.</strong> That is the largest categorical
               program in Ohio and it is equalisation: it goes to districts whose valuation per
               pupil is low enough to qualify, and this district's is not. ${count(
                 statewide.districts_without_targeted_assistance,
               )} districts are in the same position, and their categorical money is made of
               something else entirely.`
            : `Its largest single program is
               ${escapeHtml([...parts].sort((a, b) => b[1] - a[1])[0]![0])} at
               ${money([...parts].sort((a, b) => b[1] - a[1])[0]![1])}.`
        }</p>

      ${renderSpecialEducation(d)}
      ${renderTargetedAssistance(d, statewide)}
      ${renderDpia(d)}
      ${renderGifted(d)}
      ${renderCareerTechnical(d)}
      ${renderEnglishLearners(d)}

      <p class="note">These six were one number on this page until recently — a residual, computed
        as core foundation funding less the state share of base cost. The residual is exact and it
        cannot be interrogated, which matters because the programs move for opposite reasons:
        targeted assistance rises as a district gets poorer in <em>property</em>, DPIA as its
        pupils get poorer. A total that adds them describes neither.</p>
    </div>`;
}

/**
 * Special education's six categories, inside the categorical card.
 *
 * # Why the second level is worth a table of its own
 *
 * The weights are the policy and they span a factor of sixteen: Category 1 is 0.2435 and Category
 * 6 is 3.9554, each multiplying the statewide average base cost. A Category 6 pupil generates
 * nearly four times what a pupil with no disability does, so which category a child is assigned to
 * is most of what determines the money — not an administrative detail.
 *
 * And the distribution runs against the weights rather than with them. Statewide, Category 6 is
 * 15% of the pupils and 48% of the spending; Category 2 is 65% of the pupils and 34%. Two
 * categories of opposite shape are 82% of a $722m program, and the total shows neither.
 */
function renderSpecialEducation(d: District): string {
  const s = d.special_education;
  const total = s.aid.reduce((a, b) => a + b, 0);
  const pupils = s.adm.reduce((a, b) => a + b, 0);
  if (total <= 0 || pupils <= 0) return "";

  // The statutory weights, as `crates/project::panel::SPECIAL_EDUCATION_WEIGHTS` holds them and
  // as the department's own per-category amounts confirm to within one percent.
  const weights = [0.2435, 0.6179, 1.4845, 1.9812, 2.683, 3.9554];

  const rows = weights
    .map((weight, k) => ({ category: k + 1, weight, adm: s.adm[k]!, aid: s.aid[k]! }))
    .filter((r) => r.adm > 0 || r.aid > 0);

  const biggest = [...rows].sort((a, b) => b.aid - a.aid)[0];

  return `
    <h3 id="special-education">${anchor("special-education")}Special education, by category</h3>
    <div class="scroll"><table data-program="special-education">
      <thead><tr><th>Category</th><th class="tnum">Weight</th><th class="tnum">Pupils</th>
        <th class="tnum">Aid</th><th class="tnum">Share of pupils</th>
        <th class="tnum">Share of aid</th></tr></thead>
      <tbody>
        ${rows
          .map(
            (r) => `<tr${r.category === biggest?.category ? ' class="current"' : ""}>
              <th>Category ${r.category}</th>
              <td class="tnum">${r.weight.toFixed(4)}</td>
              <td class="tnum">${count(Math.round(r.adm))}</td>
              <td class="tnum">${money(r.aid)}</td>
              <td class="tnum n">${pct(r.adm / pupils, 1)}</td>
              <td class="tnum n">${pct(r.aid / total, 1)}</td>
            </tr>`,
          )
          .join("")}
        <tr class="current"><th>All categories</th><td class="tnum n">—</td>
          <td class="tnum">${count(Math.round(pupils))}</td>
          <td class="tnum">${money(total)}</td>
          <td class="tnum">100.0%</td><td class="tnum">100.0%</td></tr>
      </tbody>
    </table></div>
    <p class="note">The weight is what the category multiplies: a Category 6 pupil generates
      <strong>3.9554 times</strong> the statewide average base cost per pupil and a Category 1
      pupil <strong>0.2435 times</strong> it — a range of sixteen. Category assignment is not an
      administrative detail here, it is most of the money.${
        biggest
          ? ` This district's largest is <strong>Category ${biggest.category}</strong> at
             ${money(biggest.aid)}, which is ${pct(biggest.aid / total, 0)} of its special
             education aid from ${pct(biggest.adm / pupils, 0)} of its special education pupils.`
          : ""
      }</p>`;
}

/**
 * Targeted assistance, inside the categorical card.
 *
 * # Two tiers that measure different things
 *
 * The largest categorical in Ohio at $1.36bn, and the only one that is an equalisation rather than
 * a payment for a category of pupil. `[G]` is `[C] + [F]` and the addends do not answer the same
 * question:
 *
 * - the **capacity tier** is 0.8% of however far the district's *total* weighted wealth falls
 *   below the median district's, so it is about the size of the tax base full stop;
 * - the **wealth tier** is a rate against weighted wealth *per resident pupil*, cutting off where
 *   that reaches 1.25 times the state median.
 *
 * Statewide the split is 24% capacity and 76% wealth, but for a district it can be either, both or
 * neither — 299 draw the first, 438 the second, 135 nothing at all.
 *
 * # And two pupil counts, one line apart
 *
 * The wealth tier divides by **resident** ADM to measure and multiplies by **enrolled** ADM to pay.
 * Where a district has heavy inbound open enrolment those differ, and the card says so rather than
 * printing a per-pupil figure over whichever one came to hand.
 */
function renderTargetedAssistance(d: District, statewide: Statewide): string {
  const t = d.targeted_assistance;
  const total = d.categoricals.targeted_assistance;
  if (t.weighted_wealth <= 0) return "";

  // The statewide medians the two indices divide by, from the feed rather than from this file.
  // They were copied out of `crates/project::panel` and bound to nothing, so the shortfall below
  // was computed against whichever value the Rust held the last time somebody remembered.
  const medianWealth = statewide.targeted_assistance_median_weighted_wealth;
  const medianPerPupil = statewide.targeted_assistance_median_wealth_per_pupil;
  const enrolled = d.categorical_adm;
  const openEnrollmentGap = enrolled - t.resident_adm;

  const shortfall = medianWealth - t.weighted_wealth;
  const sizeShare =
    enrolled > 600 ? 1 : enrolled > 400 ? (0.95 * (enrolled - 400)) / 200 + 0.05 : enrolled >= 200 ? 0.05 : 0;

  return `
    <h3 id="targeted-assistance">${anchor("targeted-assistance")}Targeted assistance, in its two tiers</h3>
    <div class="scroll"><table data-program="targeted-assistance">
      <thead><tr><th>Step</th><th class="tnum">Value</th><th>What it is</th></tr></thead>
      <tbody>
        <tr><th>Assessed valuation</th><td class="tnum">${money(t.property_valuation)}</td>
          <td class="n">Weighted 60% in the wealth measure.</td></tr>
        <tr><th>Federal adjusted gross income</th><td class="tnum">${money(t.federal_gross_income)}</td>
          <td class="n">The other 40%. Wealth here is property <em>and</em> income.</td></tr>
        <tr class="current"><th>Weighted wealth</th><td class="tnum">${money(t.weighted_wealth)}</td>
          <td class="n">The blend of the two.</td></tr>
        <tr><th>Capacity index</th><td class="tnum">${t.capacity_index.toFixed(4)}</td>
          <td class="n">The median district's total wealth over this one's. Above 1 is poorer.</td></tr>
        <tr${t.capacity_amount <= 0 ? ' class="n"' : ""}><th>Capacity amount</th>
          <td class="tnum">${t.capacity_amount <= 0 ? "—" : money(t.capacity_amount)}</td>
          <td class="n">${
            shortfall <= 0
              ? "Its wealth is above the median district's, so this tier pays nothing."
              : sizeShare <= 0
                ? `Below 200 pupils. The tier pays nothing at this size however far below the
                   median the district's wealth falls.`
                : `0.8% of the ${money(shortfall)} shortfall${
                    sizeShare < 1 ? `, at ${pct(sizeShare, 0)} for a district of this size` : ""
                  }.`
          }</td></tr>
        <tr><th>Wealth per resident pupil</th><td class="tnum">${money(t.wealth_per_pupil)}</td>
          <td class="n">Weighted wealth over ${count(Math.round(t.resident_adm))} resident pupils${
            Math.abs(openEnrollmentGap) > 1
              ? ` — ${count(Math.round(Math.abs(openEnrollmentGap)))} ${
                  openEnrollmentGap > 0 ? "fewer" : "more"
                } than it enrols`
              : ""
          }.</td></tr>
        <tr><th>Wealth index</th><td class="tnum">${t.wealth_index.toFixed(4)}</td>
          <td class="n">${money(medianPerPupil)} over that. Below 0.8 the tier pays nothing.</td></tr>
        <tr${t.wealth_amount <= 0 ? ' class="n"' : ""}><th>Wealth amount</th>
          <td class="tnum">${t.wealth_amount <= 0 ? "—" : money(t.wealth_amount)}</td>
          <td class="n">${
            t.wealth_amount <= 0
              ? "Its wealth per pupil is above 1.25 times the state median."
              : `Paid on its ${count(Math.round(enrolled))} <em>enrolled</em> pupils, not the
                 resident count the rate was measured against.`
          }</td></tr>
        <tr class="current"><th>Targeted assistance</th><td class="tnum">${money(total)}</td>
          <td class="n">The two tiers, added.</td></tr>
      </tbody>
    </table></div>
    <p class="note">${
      total <= 0
        ? `This district draws neither tier. 135 are in the same position, and it is not a
           statement about their pupils — targeted assistance measures the tax base.`
        : t.capacity_amount > 0 && t.wealth_amount > 0
          ? `Both tiers pay here, ${pct(t.capacity_amount / total, 0)} capacity and
             ${pct(t.wealth_amount / total, 0)} wealth. Statewide the split is 24% and 76%.`
          : t.capacity_amount > 0
            ? `Only the capacity tier pays here. 299 districts draw it and 438 draw the other, so
               which one a district is on says something the total does not.`
            : `Only the wealth tier pays here. Its total wealth is above the median district's,
               which is what the capacity tier measures, but it is spread across enough pupils to
               qualify on the per-pupil one.`
    }${
      t.supplement_eligible
        ? ` It also qualifies for <strong>supplemental targeted assistance</strong> — a test for
           districts that were poor in FY2019 and have since lost more than an eighth of their
           pupils. 36 districts qualify. The program pays every one of them nothing.`
        : ""
    }</p>`;
}

/**
 * Disadvantaged Pupil Impact Aid, inside the categorical card.
 *
 * # The index is squared, and that is the whole character of the program
 *
 * DPIA blends two poverty counts, expresses the blend as a share of enrolment, and indexes that
 * share against the statewide 0.5334 — **squared**. Aid scales with the square of relative
 * poverty, so a district at twice the state's rate scores four times the index rather than twice.
 *
 * The two counts also disagree. Direct certification is administrative and finds a median 61% of
 * what the economically disadvantaged count does, so putting 35% of the weight on it lowers the
 * funded count almost everywhere. In three districts it runs the other way.
 */
function renderDpia(d: District): string {
  const p = d.dpia;
  const total = d.categoricals.dpia;
  if (p.weighted_adm <= 0) return "";

  const statewideShare = 0.5334;
  const linear = p.percentage / statewideShare;
  const certificationRatio =
    p.economically_disadvantaged_adm > 0
      ? p.directly_certified_adm / p.economically_disadvantaged_adm
      : 0;

  return `
    <h3 id="dpia">${anchor("dpia")}Disadvantaged Pupil Impact Aid, step by step</h3>
    <div class="scroll"><table data-program="dpia">
      <thead><tr><th>Step</th><th class="tnum">Value</th><th>What it is</th></tr></thead>
      <tbody>
        <tr><th>Economically disadvantaged ADM</th>
          <td class="tnum">${count(Math.round(p.economically_disadvantaged_adm))}</td>
          <td class="n">FY2025. Weighted 65%.</td></tr>
        <tr><th>Directly certified ADM</th>
          <td class="tnum">${count(Math.round(p.directly_certified_adm))}</td>
          <td class="n">FY2026, weighted 35% — an administrative count, ${
            certificationRatio > 0
              ? `${pct(certificationRatio, 0)} of the disadvantaged one here`
              : "smaller almost everywhere"
          }.</td></tr>
        <tr class="current"><th>Blended count</th>
          <td class="tnum">${count(Math.round(p.weighted_adm))}</td>
          <td class="n">The 65/35 mix. This is the number the money is paid on.</td></tr>
        <tr><th>As a share of enrolment</th><td class="tnum">${pct(p.percentage, 2)}</td>
          <td class="n">Against a statewide ${pct(statewideShare, 2)}.</td></tr>
        <tr class="current"><th>Index</th><td class="tnum">${p.index.toFixed(4)}</td>
          <td class="n">That share over the state's, <strong>squared</strong> — ${linear.toFixed(
            4,
          )} before squaring.</td></tr>
        <tr class="current"><th>Disadvantaged Pupil Impact Aid</th>
          <td class="tnum">${money(total)}</td>
          <td class="n">$422 per weighted pupil, scaled by the index.</td></tr>
      </tbody>
    </table></div>
    <p class="note">The squaring is the program. A district at <strong>twice</strong> the state's
      poverty rate scores <strong>four times</strong> the index, not twice — so DPIA is convex, and
      $525m distributed on a curve like that concentrates far more sharply than a per-pupil rate
      would. ${
        p.index > 1
          ? `This district is above the state's rate, so the squaring works in its favour: its
             index is ${p.index.toFixed(2)} where a linear one would be ${linear.toFixed(2)}.`
          : `This district is below the state's rate, so the squaring works against it: its index
             is ${p.index.toFixed(2)} where a linear one would be ${linear.toFixed(2)}.`
      }</p>`;
}

/**
 * Gifted, inside the categorical card.
 *
 * # The only categorical that buys staff rather than pupils
 *
 * Two per-pupil amounts and three kinds of **unit**, where a unit is a headcount entitlement priced
 * like a salary. 87% of the $54m program is unit funding.
 *
 * The units are clamped, and the clamps are the policy. Coordinator units are enrolment over 3,300,
 * floored at 0.5 and capped at 8; specialist units are identified gifted FTE over 140 in each band,
 * floored at 0.3. So a district identifying no gifted pupils still draws $93,993 before its state
 * share — gifted is the one categorical with a floor rather than a proportion. 370 districts sit on
 * the coordinator floor and three on the cap.
 */
function renderGifted(d: District): string {
  const g = d.gifted;
  const total = d.categoricals.gifted;
  if (total <= 0) return "";

  const units = [
    ["Coordinator", g.coordinator_units, g.coordinator_aid, 85_776, d.categorical_adm / 3300, 0.5, "one per 3,300 pupils, floored at 0.5 and capped at 8"],
    ["Intervention specialist, K-8", g.specialist_k8_units, g.specialist_k8_aid, 89_378, g.fte_k8 / 140, 0.3, "one per 140 identified gifted pupils, floored at 0.3"],
    ["Intervention specialist, 9-12", g.specialist_9_12_units, g.specialist_9_12_aid, 80_974, g.fte_9_12 / 140, 0.3, "the same, priced $8,404 lower"],
  ] as const;

  // Whether a unit is floored is decided by what the district *earned* against the floor, not by
  // comparing the awarded figure to the earned one. The feed rounds units to four places, so an
  // earned 5.31419 is published as an awarded 5.3142 and a naive `awarded > earned` reads that
  // rounding as a floor — which is what this card said about Cleveland's 9-12 specialists on the
  // first pass, while missing that its coordinators are at the cap.
  const floored = units.filter(([, , , , earned, floor]) => earned < floor - 1e-9);
  const atTheCap = g.coordinator_units >= 8 - 1e-9;
  const unitTotal = g.coordinator_aid + g.specialist_k8_aid + g.specialist_9_12_aid;

  return `
    <h3 id="gifted">${anchor("gifted")}Gifted: two payments and three kinds of unit</h3>
    <div class="scroll"><table data-program="gifted">
      <thead><tr><th>Component</th><th class="tnum">Units</th><th class="tnum">Earned</th>
        <th class="tnum">Amount</th><th>What it is</th></tr></thead>
      <tbody>
        <tr><th>Identification</th><td class="tnum n">—</td><td class="tnum n">—</td>
          <td class="tnum">${money(g.identification)}</td>
          <td class="n">$24 per K-6 pupil, after the state share.</td></tr>
        <tr><th>Referral</th><td class="tnum n">—</td><td class="tnum n">—</td>
          <td class="tnum">${money(g.referral)}</td>
          <td class="n">$2.50 per enrolled pupil — a different denominator, one line away.</td></tr>
        ${units
          .map(
            ([label, awarded, aid, price, earned, floor, note]) => `<tr${
              earned < floor - 1e-9 || awarded < earned - 1e-9 ? ' class="current"' : ""
            }>
              <th>${escapeHtml(label)}</th>
              <td class="tnum">${awarded.toFixed(4)}</td>
              <td class="tnum n">${earned.toFixed(4)}</td>
              <td class="tnum">${money(aid)}</td>
              <td class="n">${money(price)} each — ${escapeHtml(note)}.</td>
            </tr>`,
          )
          .join("")}
        <tr class="current"><th>Gifted</th><td class="tnum n">—</td><td class="tnum n">—</td>
          <td class="tnum">${money(total)}</td>
          <td class="n">${pct(unitTotal / total, 0)} of it is units.</td></tr>
      </tbody>
    </table></div>
    <p class="note">${
      floored.length > 0
        ? `<strong>${
            floored.length === 3
              ? "Every one of this district's three unit entitlements is"
              : floored.length === 1
                ? "One of this district's three unit entitlements is"
                : `${floored.length} of this district's three unit entitlements are`
          } a floor rather than an earned amount</strong> — its own counts would buy less. That is
           deliberate: units buy people, and half a coordinator is already the smallest thing a
           small district can be funded for. It also means this part of its gifted money does not
           move when its gifted enrolment does.`
        : atTheCap
          ? `This district is at the <strong>eight-coordinator cap</strong>. Its enrolment earns
             ${(d.categorical_adm / 3300).toFixed(2)} units and it is paid for eight, so unlike
             most of Ohio its coordinator line is a ceiling rather than a floor. Three districts
             are in that position.`
          : `Every unit here is earned rather than floored or capped.`
    } Statewide, 370 districts sit on the half-coordinator floor — three-fifths of Ohio — so for
      most of the state the coordinator line is a minimum rather than a measurement.</p>`;
}

/**
 * Career-technical education, inside the categorical card.
 *
 * # The same shape as special education, against a base cost 20% higher
 *
 * Five weights times five FTE counts times a base cost times the state share. The difference is the
 * base: CTE is weighted against a **career-technical** base cost per pupil of $9,855.62, where
 * special education and English learners use the statewide average $8,241.61.
 *
 * So a CTE weight and a special education weight are not on the same scale, and a table that lists
 * Ohio's weights together — which is the natural thing to build — understates career-technical by a
 * fifth before any pupil is counted.
 */
function renderCareerTechnical(d: District): string {
  const c = d.career_technical;
  const total = d.categoricals.career_technical;
  if (total <= 0) return "";

  // As `crates/project::panel::CTE_WEIGHTS` holds them, and as the department's own per-category
  // amounts reproduce to the cent.
  const weights = [0.623, 0.5905, 0.2154, 0.183, 0.157];
  const baseCost = 9855.62;

  const rows = weights
    .map((weight, k) => ({ category: k + 1, weight, fte: c.fte[k]!, aid: c.aid[k]! }))
    .filter((r) => r.fte > 0 || r.aid > 0);
  if (rows.length === 0) return "";
  const fte = rows.reduce((a, r) => a + r.fte, 0);

  return `
    <h3 id="career-technical">${anchor("career-technical")}Career-technical education, by category</h3>
    <div class="scroll"><table data-program="career-technical">
      <thead><tr><th>Category</th><th class="tnum">Weight</th><th class="tnum">FTE</th>
        <th class="tnum">Aid</th></tr></thead>
      <tbody>
        ${rows
          .map(
            (r) => `<tr>
              <th>Category ${r.category}</th>
              <td class="tnum">${r.weight.toFixed(4)}</td>
              <td class="tnum">${r.fte.toFixed(2)}</td>
              <td class="tnum">${money(r.aid)}</td>
            </tr>`,
          )
          .join("")}
        <tr><th>Associated services</th><td class="tnum">0.0294</td>
          <td class="tnum n">${fte.toFixed(2)}</td>
          <td class="tnum">${money(c.associated_services)}</td></tr>
        <tr class="current"><th>Career-technical education</th><td class="tnum n">—</td>
          <td class="tnum">${fte.toFixed(2)}</td><td class="tnum">${money(total)}</td></tr>
      </tbody>
    </table></div>
    <p class="note">These weights multiply a <strong>career-technical</strong> base cost of
      ${money(baseCost)} per pupil, not the ${money(8241.61)} the rest of the plan uses — 20%
      higher before any weight is applied. A Category 1 CTE pupil generates
      ${money(0.623 * baseCost)} where the same weight against the general base would give
      ${money(0.623 * 8241.61)}. Ohio's weights are not a single scale, and reading them as one
      understates this program by a fifth.</p>
    <p class="note"><strong>The counts are frozen at FY2021.</strong> The department's own column
      header reads <code>Category 1 Career Tech FTE-FY21</code>, so the FTE column above is
      enrolment as it stood six years before the year being funded, while the base cost it is
      multiplied against is built on a rolling three-year average ending ${seriesSpanEnd("enrollment")}. A district that
      has opened or closed a programme since 2021 is funded for the one it had then.</p>`;
}

/**
 * English learners, inside the categorical card.
 *
 * # The weights descend, which is the opposite of everywhere else
 *
 * 0.2104, 0.1577, 0.1053 — Category 1 is funded at twice Category 3, and Category 1 is the most
 * recently arrived learner. The program pays most in a pupil's first year and tapers over the next
 * two, so a district whose English learners are settling in sees this money fall while they are
 * still learning English.
 *
 * Special education's weights ascend from 0.2435 to 3.9554 as need deepens, which is what a reader
 * assumes Ohio's weights mean. Here they do not.
 *
 * This used to say *"every other weighted categorical in the plan runs the other way"*, and that is
 * false — [`renderCareerTechnical`] renders `[0.623, 0.5905, 0.2154, 0.183, 0.157]` five table-rows
 * above, strictly descending, in a column headed Weight. The corpus supplies the distinction the
 * claim was reaching for: career-technical's ordering "is not severity: it is programme type"
 * (`fsfp-career-technical-weights.yml`), so the two descents are not the same fact. English
 * learners is the only categorical whose weights fall along a **need** gradient, and that is the
 * claim the page can support.
 */
function renderEnglishLearners(d: District): string {
  const e = d.english_learners;
  const total = d.categoricals.english_learners;
  if (total <= 0) return "";

  const weights = [0.2104, 0.1577, 0.1053];
  const rows = weights
    .map((weight, k) => ({ category: k + 1, weight, adm: e.adm[k]!, aid: e.aid[k]! }))
    .filter((r) => r.adm > 0 || r.aid > 0);
  if (rows.length === 0) return "";
  const learners = rows.reduce((a, r) => a + r.adm, 0);

  return `
    <h3 id="english-learners">${anchor("english-learners")}English learners, by category</h3>
    <div class="scroll"><table data-program="english-learners">
      <thead><tr><th>Category</th><th class="tnum">Weight</th><th class="tnum">Pupils</th>
        <th class="tnum">Aid</th></tr></thead>
      <tbody>
        ${rows
          .map(
            (r) => `<tr>
              <th>Category ${r.category}</th>
              <td class="tnum">${r.weight.toFixed(4)}</td>
              <td class="tnum">${r.adm.toFixed(2)}</td>
              <td class="tnum">${money(r.aid)}</td>
            </tr>`,
          )
          .join("")}
        <tr class="current"><th>English learners</th><td class="tnum n">—</td>
          <td class="tnum">${learners.toFixed(2)}</td><td class="tnum">${money(total)}</td></tr>
      </tbody>
    </table></div>
    <p class="note">The weights <strong>descend</strong>. Category 1 is the most recently arrived
      learner and is funded at twice Category 3, so this program pays most in a pupil's first year
      and least in their third — it tapers as need persists rather than deepening with it. Special
      education runs the other way, ascending from 0.2435 to 3.9554 as need deepens. Career-technical's
      weights descend too, but along programme type rather than need, so this is the only categorical
      that pays less as the thing it is for persists. English learner funding reaches
      505 of Ohio's 609 districts, and 38 of them hold 80% of the $36m.</p>
    <p class="note"><strong>And the counts are frozen at FY2021.</strong> The department's own
      column header reads <code>Category 1 EL ADM-FY21</code>, so the FTE column above is enrolment
      as it stood six years before the year being funded, while base cost beside it is built on a
      rolling three-year average ending ${seriesSpanEnd("enrollment")}. For a program whose whole structure is about how
      recently a pupil arrived, that is a stranger fact than it would be anywhere else in the
      plan.</p>`;
}

/**
 * What a district receives outside the formula, and what it is not held at.
 *
 * # Two payments that behave unlike anything in the formula
 *
 * `[H] Foundation Funding` is base cost plus six categoricals, and the guarantee holds a district
 * at it. These sit in `[R] Total State Support` instead. Nothing cushions a fall in either — drop
 * a star, or slip below 3% growth, and the money is simply gone next year.
 *
 * The **performance supplement** is the only place Ohio's funding formula pays on a measured
 * outcome rather than an input, and it runs the opposite way to the rest of the formula: by
 * poverty quintile the mean payment falls from $54.74 per pupil to $23.31. The card says so on
 * every district's page rather than only on the ones where it looks generous.
 *
 * The **growth supplement** is a cliff. 3% three-year growth pays $250 on every pupil, not on the
 * pupils gained, so what the threshold costs a district has nothing to do with how close it came.
 * For a district just below, the card prints the amount forgone, which is the only honest way to
 * show a cliff.
 */
export function renderSupplements(d: District, statewide: Statewide): string {
  const s = d.supplements;
  const total = s.performance + s.base_funding + s.growth;
  if (total <= 0) return "";

  const rating = s.stars != null && s.progress != null ? Math.max(s.stars, s.progress) : null;
  const perPupil = d.categorical_adm > 0 ? s.performance / d.categorical_adm : 0;
  const nearTheCliff = !s.growth_eligible && s.enrollment_change > 0.02;

  return `
    <div class="card" id="supplements" data-part="supplements">
      <h2>${anchor("supplements")}Outside the formula${yearChip("formula")}</h2>
      <p class="note">These are paid on top of formula aid and the guarantee does not hold a
        district at them. Everything above this card is <strong>foundation funding</strong>, which
        is protected; this is not.</p>

      <div class="scroll"><table data-program="supplements">
        <thead><tr><th>Payment</th><th class="tnum">Amount</th><th>How it is decided</th></tr></thead>
        <tbody>
          <tr><th><a href="${routes.wikiNode(
            "formula-component",
            "fsfp-enrolment-supplements",
          )}">Base funding supplement</a></th><td class="tnum">${money(s.base_funding)}</td>
            <td class="n">$40 a pupil. Every district, no test of any kind.</td></tr>
          <tr${s.performance <= 0 ? ' class="n"' : ""}><th><a href="${routes.wikiNode(
            "formula-component",
            "fsfp-performance-supplement",
          )}">Performance supplement</a></th>
            <td class="tnum">${s.performance <= 0 ? "—" : money(s.performance)}</td>
            <td class="n">${
              s.performance_eligible && rating != null
                ? `$13 a pupil per rating point, on ${rating.toFixed(1)} — the greater of its
                   ${s.stars?.toFixed(1) ?? "—"}-star overall rating and its
                   ${s.progress?.toFixed(1) ?? "—"} progress rating. ${money(perPupil)} a pupil.`
                : `This district qualifies on none of the three routes: an overall rating above 3.5
                   stars, a progress rating of 3 or more, or a progress rating higher than the year
                   before.`
            }</td></tr>
          <tr${s.growth <= 0 ? ' class="n"' : ""}><th><a href="${routes.wikiNode(
            "formula-component",
            "fsfp-enrolment-supplements",
          )}">Enrollment growth supplement</a></th>
            <td class="tnum">${s.growth <= 0 ? "—" : money(s.growth)}</td>
            <td class="n">${
              s.growth_eligible
                ? `$250 on <em>every</em> pupil — not the pupils gained — because its enrolment rose
                   ${pct(s.enrollment_change, 2)} over three years, clearing the 3% threshold.`
                : s.enrollment_change > 0
                  ? `Its enrolment rose ${pct(s.enrollment_change, 2)} over three years, short of
                     the 3% this requires.`
                  : `Its enrolment fell ${pct(Math.abs(s.enrollment_change), 2)} over three years.
                     This pays only for growth.`
            }</td></tr>
          <tr class="current"><th>Outside the formula</th><td class="tnum">${money(total)}</td>
            <td class="n">Not held by the guarantee.</td></tr>
        </tbody>
      </table></div>

      ${
        nearTheCliff && s.growth_forgone != null
          ? `<p class="note"><strong>This district is just under the growth cliff.</strong> It grew
             ${pct(s.enrollment_change, 2)} against the 3% required —
             ${pct(0.03 - s.enrollment_change, 2)} short — and the supplement pays on the whole
             roll rather than on the pupils gained, so clearing it would have been worth
             <strong>${money(s.growth_forgone)}</strong>. What the threshold costs a district has
             nothing to do with how close it came.</p>`
          : ""
      }

      ${renderTransportation(d)}
      ${renderPreschoolSpecialEducation(d, statewide)}

      <p class="note">The performance supplement is the only part of Ohio's school funding paid on
        a <em>measured outcome</em> rather than on pupils, categories or a tax base — and it runs
        against the grain of everything else in the formula. Sorted by poverty, the mean payment
        per pupil falls from <strong>$54.74</strong> in the least-poor fifth of districts to
        <strong>$23.31</strong> in the poorest, and the share qualifying falls from 91% to 49%.
        Ohio's attainment measures track intake, so a program keyed to them follows intake whatever
        its intent — which explains the gradient without removing it.</p>
    </div>`;
}

/**
 * The one payment on this page that no other figure on it counts.
 *
 * # Why it gets a card rather than a row in "Outside the formula"
 *
 * That card holds the performance and enrolment supplements, which are outside *foundation
 * funding* and inside `[R] Total State Support` — the guarantee does not hold a district at them,
 * but the department pays them and the calculator computes them. This is a different kind of
 * outside. The gross casino revenue county student fund runs from the Tax Commissioner through
 * eighty-eight county funds to districts, and never becomes an appropriation to the Department of
 * Education and Workforce at all. It is therefore absent from every other number on this page —
 * from total state support, from the categorical decomposition, and from the general-fund
 * `finances` rows, which book it in a different fund.
 *
 * A reader who added up this page before this card existed was told, by omission, that they had
 * seen all the state money. They had not.
 *
 * # Why there is no per-pupil figure here
 *
 * Because there is no pupil count on this page it could honestly divide by. R.C. 5753.11 defines
 * the fund's denominator as county-resident students enrolled in a public school district —
 * which for this purpose includes community schools, STEM schools and joint vocational districts —
 * counted in October for the January payment and in May for the August one, with a pupil enrolled
 * in both a JVSD and a district counted in *both*. That is a fifth Ohio pupil count and it is
 * deliberately not a partition. Dividing by `categorical_adm` would produce a figure that sits
 * beside four other per-pupil figures looking like a fifth of the same kind.
 *
 * What is comparable is a share of money against money in the same year, which is what the tile
 * does: this fund against the state aid the treasurer booked in the same fiscal year.
 */
export function renderCasino(bundle: Bundle, d: District): string {
  if (d.casino.length === 0) return "";
  const first = d.casino[0]!;
  const latest = d.casino[d.casino.length - 1]!;
  const closure = d.casino.find((y) => y.fiscal_year === CASINO_CLOSURE_YEAR);
  const before = d.casino.find((y) => y.fiscal_year === CASINO_CLOSURE_YEAR - 1);
  const booked = d.finances.find((y) => y.fiscal_year === latest.fiscal_year);
  const shareOfAid =
    booked?.state_aid != null && booked.state_aid > 0 ? latest.amount / booked.state_aid : null;

  const statewide = bundle.casino.find((y) => y.fiscal_year === latest.fiscal_year);
  const bars: Bar[] = d.casino.map((y) => ({
    label: `FY${y.fiscal_year}`,
    value: y.amount,
    hover: `FY${y.fiscal_year}: ${money(y.amount)}${
      y.fiscal_year === CASINO_CLOSURE_YEAR ? " — the casinos were closed for a quarter of it" : ""
    }`,
    ...(y.fiscal_year === latest.fiscal_year || y.fiscal_year === CASINO_CLOSURE_YEAR
      ? { direct: money(y.amount) }
      : {}),
  }));

  return `
    <div class="card" id="casino" data-part="casino">
      <h2>${anchor("casino")}The money that reaches this district outside the formula${yearChip(
        "casino",
      )}</h2>
      <p class="note">Every other figure on this page is state aid computed by a formula or booked
        by the treasurer as grants-in-aid. This is neither. It is a share of the
        <a href="${routes.wikiNode("revenue-stream", "casino-tax-distribution")}">gross casino
        revenue county student fund</a>, which moves from the Tax Commissioner through county
        funds to districts and never becomes an appropriation to the department — so
        <strong>nothing above this card counts it</strong>.</p>

      <div class="tiles">
        <div class="tile"><div class="k">Received, FY${latest.fiscal_year}</div>
          <div class="v">${money(latest.amount)}</div>
          <div class="n">${
            statewide
              ? `of ${money(statewide.amount)} distributed statewide`
              : "paid in two instalments, January and August"
          }</div></div>
        <div class="tile"><div class="k">Against state aid booked that year</div>
          <div class="v">${shareOfAid == null ? "—" : pct(shareOfAid, 2)}</div>
          <div class="n">${
            shareOfAid == null
              ? "no FY" + latest.fiscal_year + " forecast filing to compare against"
              : `${money(booked!.state_aid)} of unrestricted grants-in-aid. Median across
                 Ohio's districts is ${pct(CASINO_MEDIAN_SHARE_OF_AID, 2)}`
          }</div></div>
        <div class="tile"><div class="k">County funds it is paid from</div>
          <div class="v">${d.casino_counties ?? "—"}</div>
          <div class="n">${
            d.casino_counties == null
              ? "not named in the most recent distribution"
              : d.casino_counties === 1
                ? `its resident pupils are all in one county, as
                   ${CASINO_SINGLE_COUNTY_DISTRICTS} of Ohio's
                   ${bundle.statewide.districts} districts' are`
                : `it has resident pupils in ${count(d.casino_counties)} counties, and is paid out
                   of each county's fund separately`
          }</div></div>
      </div>

      <div class="chartwrap" data-chart="casino">${renderToString((w) => barSpec(bars, { width: w }), { label: `Casino tax distribution received by this district, in dollars by fiscal year, FY${first.fiscal_year} to FY${latest.fiscal_year}` })}</div>

      <p class="note">${
        closure && before && before.amount > 0
          ? `<strong>FY${CASINO_CLOSURE_YEAR} is the closure.</strong> Ohio's casinos were shut by
             order from mid-March 2020, and because this fund is paid on the tax actually collected
             rather than on an appropriation, the drop arrives whole: ${money(closure.amount)}
             against ${money(before.amount)} the year before, ${pct(
               closure.amount / before.amount - 1,
               1,
             )}. Nothing cushioned it and nothing was made up afterwards — which is the difference
             between money the legislature sets and money a tax yields.`
          : `This fund is paid on the tax actually collected rather than on an appropriation, so it
             moves with gambling activity and nothing cushions a fall in it.`
      }</p>

      <div class="scroll"><table data-program="casino">
        <thead><tr><th>Fiscal year</th><th class="tnum">Distributed to this district</th>
          <th class="tnum">Statewide</th></tr></thead>
        <tbody>${d.casino
          .map((y) => {
            const all = bundle.casino.find((s) => s.fiscal_year === y.fiscal_year);
            return `<tr${y.fiscal_year === CASINO_CLOSURE_YEAR ? ' class="loss"' : ""}>
              <th>FY${y.fiscal_year}</th>
              <td class="tnum">${money(y.amount)}</td>
              <td class="tnum">${all == null ? "—" : money(all.amount)}</td>
            </tr>`;
          })
          .join("")}</tbody>
      </table></div>

      <p class="note"><strong>There is no per-pupil figure here, and one should not be computed
        from this page.</strong> The fund is apportioned on the count R.C. 5753.11 defines —
        county-resident pupils, counting community, STEM and joint vocational enrolment, with a
        pupil enrolled in both a JVSD and a district counted in both — which is a fifth Ohio pupil
        count and is deliberately not a partition of the state's students. Dividing this by the
        enrolled ADM above would put a number beside four other per-pupil figures that looks like a
        fifth of the same kind and is not one.</p>

      <p class="note">The district decides how to spend it. The Ohio Constitution restricts these
        distributions to the support of primary and secondary education and imposes no other
        condition, which makes this the least restricted money on the page — most restricted money
        in this corpus is restricted to a programme, and this is restricted only to a purpose broad
        enough to cover nearly any school expenditure.
        The series runs FY${first.fiscal_year} to FY${latest.fiscal_year} because the Department of
        Taxation's own casino page carries no distribution after January ${
          latest.fiscal_year
        }, and the ones before FY${first.fiscal_year} were published as PDFs.</p>
    </div>`;
}

/**
 * Transportation, inside the card for what sits outside the formula.
 *
 * # The second-largest program in Ohio's school funding, and it looks like nothing else
 *
 * $726m plus $183m of special education transportation — more than special education itself. Four
 * things here have no counterpart in the formula, and the card shows each as a step rather than
 * folding them into a total:
 *
 * - **two competing bases**, per weighted rider and per bus mile, with the district paid the
 *   greater. Which one wins flips for more than half the state and is invisible in the amount;
 * - **a 50% state minimum share**, five times the formula's, on which most districts sit;
 * - **two supplements pulling opposite ways** — one for filling buses, one for having few children
 *   per square mile;
 * - **a proration factor** on the special education line, which means the published figure is not
 *   what the district was owed but what there was to divide.
 */
function renderTransportation(d: District): string {
  const t = d.transportation;
  if (t.total <= 0) return "";

  const chosen = t.paid_on_miles ? t.per_mile_base : t.per_rider_base;
  const other = t.paid_on_miles ? t.per_rider_base : t.per_mile_base;
  const shortfall = t.special_education_unprorated - t.special_education;

  return `
    <h3 id="transportation">${anchor("transportation")}<a href="${routes.wikiNode(
      "formula-component",
      "fsfp-transportation",
    )}">Transportation</a></h3>
    <div class="scroll"><table data-program="transportation">
      <thead><tr><th>Step</th><th class="tnum">Value</th><th>What it is</th></tr></thead>
      <tbody>
        <tr><th>Weighted riders</th><td class="tnum">${count(Math.round(t.weighted_riders))}</td>
          <td class="n">${count(Math.round(t.public_riders))} public${
            t.nonpublic_riders > 0
              ? `, ${count(Math.round(t.nonpublic_riders))} non-public counted <strong>twice</strong>`
              : ""
          }${
            t.community_riders > 0
              ? `, ${count(Math.round(t.community_riders))} community or STEM counted 1.5 times`
              : ""
          }.</td></tr>
        <tr${t.paid_on_miles ? ' class="n"' : ' class="current"'}><th>Per-rider base</th>
          <td class="tnum">${money(t.per_rider_base)}</td>
          <td class="n">$1,337.18 a weighted rider.</td></tr>
        <tr${t.paid_on_miles ? ' class="current"' : ' class="n"'}><th>Per-mile base</th>
          <td class="tnum">${money(t.per_mile_base)}</td>
          <td class="n">$6.867 a bus mile across a 180-day year.</td></tr>
        <tr><th>School bus payment</th><td class="tnum">${money(t.school_bus)}</td>
          <td class="n">The greater of the two — <strong>${
            t.paid_on_miles ? "miles" : "riders"
          }</strong>, here — at a state share of ${pct(t.effective_state_share, 1)}.</td></tr>
        ${
          t.mass_transit > 0 || t.other > 0
            ? `<tr><th>Other vehicle types</th>
                <td class="tnum">${money(t.mass_transit + t.other)}</td>
                <td class="n">Mass transit at 35% of the rider rate, other types at 50%.</td></tr>`
            : ""
        }
        <tr${t.efficiency <= 0 ? ' class="n"' : ""}><th>Efficiency supplement</th>
          <td class="tnum">${t.efficiency <= 0 ? "—" : money(t.efficiency)}</td>
          <td class="n">${
            t.efficiency > 0
              ? `Up to 15% more for filling buses. Its index is ${t.efficiency_index.toFixed(4)}${
                  t.efficiency_index >= 1.5 ? ", at the ceiling" : ""
                }.`
              : `Its riders-per-bus index of ${t.efficiency_index.toFixed(
                  4,
                )} is below the 1.0 this starts at.`
          }</td></tr>
        <tr${t.density <= 0 ? ' class="n"' : ""}><th>Density supplement</th>
          <td class="tnum">${t.density <= 0 ? "—" : money(t.density)}</td>
          <td class="n">${
            t.density > 0
              ? `Paid for sparseness: ${t.district_density.toFixed(1)} riders a square mile against
                 the 28 this counts down from.`
              : `At ${t.district_density.toFixed(1)} riders a square mile it is above the
                 28 threshold, so this pays nothing.`
          }</td></tr>
        ${
          t.guarantee > 0
            ? `<tr class="current"><th>Transportation guarantee</th>
                <td class="tnum">${money(t.guarantee)}</td>
                <td class="n">Held at its FY2021 transportation funding of ${money(
                  t.fy21_base,
                )}, because the computed amount fell below it. A <em>second</em> guarantee,
                separate from the one on formula aid.</td></tr>`
            : ""
        }
        <tr class="current"><th>Transportation</th><td class="tnum">${money(t.total)}</td>
          <td class="n">The payments and supplements, added.</td></tr>
        <tr><th>Special education transportation</th>
          <td class="tnum">${money(t.special_education)}</td>
          <td class="n">Its reported cost at the same state share — then multiplied by
            <strong>0.91746</strong>.</td></tr>
      </tbody>
    </table></div>
    <p class="note"><strong>That last factor is not a rate.</strong> A proration below one means
      the appropriation did not cover what the formula computed, so every district's special
      education transportation was scaled down to fit. This district was computed
      ${money(t.special_education_unprorated)} and paid ${money(t.special_education)} —
      <strong>${money(shortfall)} short</strong>. The published figure is not what it was owed; it
      is what there was to divide.</p>
    <p class="note">Two things here have no counterpart in the formula. The state share floor is
      <strong>50%</strong> against the formula's 10%, so for most of Ohio local capacity does not
      determine transportation aid at all — the state equalises getting to school far harder than
      it equalises what happens there. And the two supplements reward opposite things: one pays for
      filling buses, the other for having too few children per square mile to fill them.</p>`;
}

/**
 * Preschool special education, inside the card for what sits outside the formula.
 *
 * # A flat grant and a half-weight, and the proration that no longer fits
 *
 * $148m. Each category pays a flat $4,000 a pupil — 69% of the program, and **not** reduced by the
 * state share — plus the six school-age weights at half. So this is the one component where the
 * wealthiest district and the poorest are funded identically for most of what they receive, and
 * the one where the same weight vector that makes school-age special education steeply
 * top-heavy produces something close to flat.
 *
 * The card leads with the proration because this sheet is the only place in the workbook that
 * shows what a proration *is*: the appropriation limit sits in a cell beside the factor. At the
 * stated factor the program runs $908,184 over that limit, which the card says plainly.
 */
function renderPreschoolSpecialEducation(d: District, statewide: Statewide): string {
  const p = d.preschool_special_education;
  if (p.total <= 0) return "";

  const weights = [0.2435, 0.6179, 1.4845, 1.9812, 2.683, 3.9554];
  const rows = weights
    .map((weight, k) => ({ category: k + 1, weight, adm: p.adm[k]!, aid: p.aid[k]! }))
    .filter((r) => r.adm > 0 || r.aid > 0);
  if (rows.length === 0) return "";
  const pupils = rows.reduce((a, r) => a + r.adm, 0);
  const shortfall = p.unprorated - p.total;

  return `
    <h3 id="preschool">${anchor("preschool")}<a href="${routes.wikiNode(
      "formula-component",
      "fsfp-preschool-special-education",
    )}">Preschool special education</a></h3>
    <div class="scroll"><table data-program="preschool">
      <thead><tr><th>Category</th><th class="tnum">Weight, halved</th><th class="tnum">Pupils</th>
        <th class="tnum">Aid</th></tr></thead>
      <tbody>
        ${rows
          .map(
            (r) => `<tr>
              <th>Category ${r.category}</th>
              <td class="tnum">${(r.weight / 2).toFixed(4)}</td>
              <td class="tnum">${r.adm.toFixed(2)}</td>
              <td class="tnum">${money(r.aid)}</td>
            </tr>`,
          )
          .join("")}
        <tr class="current"><th>Preschool special education</th><td class="tnum n">—</td>
          <td class="tnum">${pupils.toFixed(2)}</td><td class="tnum">${money(p.total)}</td></tr>
      </tbody>
    </table></div>
    <p class="note">Each pupil generates a <strong>flat ${money(4000)}</strong> whatever their
      category, plus the school-age weight at <strong>half</strong> against the average base cost.
      Here the flat part is ${money(p.flat_component)} — ${pct(p.flat_component / p.total, 0)} of
      the total — and it is the one payment in Ohio's school funding the state share does not
      reduce. For most of what this program pays, the wealthiest district and the poorest are
      funded identically.</p>
    <p class="note">That also flattens the weights. The same six make school-age special education
      steeply top-heavy — Category 6 is 15% of the pupils and 48% of the money — and here, halved
      and sitting on a flat ${money(4000)}, they produce something close to parity.</p>
    <p class="note"><strong>And this program is over its appropriation.</strong> Its sheet is the
      one place in the calculator that shows what a proration is, because it carries the
      appropriation limit in a cell beside the factor: ${money(statewide.preschool_appropriation)}.
      At the stated factor of ${statewide.preschool_proration}
      the program totals <strong>${money(statewide.preschool_total)}</strong> statewide —
      ${money(statewide.preschool_total - statewide.preschool_appropriation)} over. This district
      was computed ${money(p.unprorated)} and paid
      ${money(p.total)}, ${money(shortfall)} short. The calculator is a projection published before
      the fiscal year, so the factor is presumably recalibrated before payment; as published, the
      factor, the limit and the total do not agree.</p>`;
}

/**
 * Where this district sits among America's, on federal definitions.
 *
 * # The comparison every other page on this site cannot make
 *
 * Ohio describing itself cannot say whether Ohio is unusual, and every other figure here is Ohio
 * describing itself. This is 10,382 comparable school districts in every state, reported on the
 * Census Bureau's definitions rather than on each state's.
 *
 * It also relocates districts, and mostly the ordinary ones. Ohio's *median* district sits at the
 * 66th national percentile on local share; its quarter-poorest sits at the national median. The
 * tails roughly agree between the two views and the middle does not — the opposite of the
 * intuition that a national comparison mainly moves the extremes.
 *
 * # Why the figures are not the ones above them on the page
 *
 * FY2022 against the model's FY2027, and federal fall membership against Ohio's ADM. The card says
 * so rather than letting a reader compare a number here with one three cards up, which is exactly
 * the error `denominators.ts` exists because this site shipped twice.
 */
export function renderNationalPosition(d: District): string {
  const n = d.national;
  // The absent branch carries no year chip: there are no figures here to be on a year, and a chip
  // over a card explaining that a district is outside the comparison would be dating an absence.
  if (!n) {
    return `
      <div class="card" id="national" data-part="national">
        <h2>${anchor("national")}Against America</h2>
        <p class="note">This district is not in the national comparison. The Census survey's
          comparable set is unified elementary-and-secondary districts, and this is one of Ohio's
          few K-8 districts — its spending per pupil is not comparable with a district that also
          runs a high school. It is carried without a national position rather than given an
          invented one.</p>
      </div>`;
  }

  const rank = (p: number) => `${ordinal(Math.max(1, Math.round(p * 100)))} percentile`;
  const rows: [string, string, number, string][] = [
    [
      "Local share of revenue",
      pct(n.local_share, 1),
      n.local_share_percentile,
      "What DeRolph is a claim about: how much of this district's money its own community raises.",
    ],
    [
      "Revenue per pupil",
      money(n.revenue_per_pupil),
      n.revenue_per_pupil_percentile,
      "All sources, on the federal count.",
    ],
    [
      "Current spending per pupil",
      n.spending_per_pupil <= 0 ? "—" : money(n.spending_per_pupil),
      n.spending_per_pupil_percentile,
      "What it spent, as the Bureau defines spending.",
    ],
  ];

  return `
    <div class="card" id="national" data-part="national">
      <h2>${anchor("national")}Against America${yearChip("national")}</h2>
      <div class="scroll"><table data-program="national">
        <thead><tr><th>Measure</th><th class="tnum">This district</th>
          <th class="tnum">Nationally</th><th>What it is</th></tr></thead>
        <tbody>
          ${rows
            .map(
              ([label, value, percentile, note]) => `<tr>
                <th>${escapeHtml(label)}</th>
                <td class="tnum">${value}</td>
                <td class="tnum">${percentile <= 0 ? "—" : rank(percentile)}</td>
                <td class="n">${escapeHtml(note)}</td>
              </tr>`,
            )
            .join("")}
        </tbody>
      </table></div>
      <p class="note">Against <strong>10,382 school districts in every state</strong>, on the
        Census Bureau's own definitions rather than on each state's. Every other figure on this
        site is Ohio describing itself, which can say what Ohio does and not whether it is
        unusual.${
          n.local_share_percentile > 0.8
            ? ` This district is in the <strong>national top fifth</strong> for local reliance —
               ${pct(n.local_share, 0)} of its money is raised locally.`
            : n.local_share_percentile < 0.3
              ? ` Despite Ohio's reputation, this district is in the <strong>bottom third</strong>
                 nationally for local reliance: most of its money comes from the state.`
              : ""
        }</p>
      <p class="note">An ordinary Ohio district is well above the national middle, and the shift is
        largest for ordinary districts rather than extreme ones. Ohio's median district sits at the
        <strong>66th</strong> national percentile on local share; its quarter-poorest sits at the
        national median.</p>
      <p class="note"><strong>These are not the figures above.</strong> They are
        <strong>${yearOf("national")}</strong> against the model's ${yearOf("formula")}, and they divide by the federal fall
        membership rather than by Ohio's enrolled ADM. A revenue-per-pupil figure here and a
        spending-per-pupil figure elsewhere on this page are different quantities over different
        counts in different years, and subtracting one from the other would mean nothing.</p>
    </div>`;
}


/**
 * What this page is not — the closing card, and the only one on the dashboard that declares
 * collisions rather than stating figures.
 *
 * # Why the dashboard was the last route to get one
 *
 * Three of its four siblings already close this way: `finances.astro` says its actuals are not
 * comparable line for line with the calculator and points here, `outcome.astro` names two mistakes
 * it is built to prevent, `taxes.astro` names the department its figures come from and what they
 * are not. The dashboard, which has the most collisions to declare, said nothing — and
 * `finances.astro`'s note points at this page and receives no answer back.
 *
 * # The three it declares
 *
 * **Not money received.** Every figure above is the FY2027 calculator's answer, published before
 * the year it funds. Money that changed hands is on `/finances`, from the district's own five-year
 * forecast filing, and the two are differently constructed.
 *
 * **More than one pupil count.** `d.adm` is a three-year average and `d.current_year_adm` is the
 * last year alone; they round to different integers for **499 of 609** districts, and the plan
 * uses both in one calculation. `categorical_adm` is a third field carrying the current-year
 * number for 608 districts and a figure fifty pupils below it for Akron.
 *
 * **Two other years and a different denominator, two cards down.** `crates/bundle` names the
 * vintages the page does not: valuation per pupil is FY2023 and operating expenditure per pupil is
 * FY2024, two years inside one card, both on enrolled ADM FY2024 — a *fourth* count, differing
 * from base cost ADM for 601 of 609 — while `/finances` divides by the report card's FY2025
 * unweighted headcount and `/outcome` by its FY2025 need-weighted one. Every one of those is
 * declared in `denominators.ts`, which states that it cannot check prose. This card is the prose.
 */
export function renderWhatThisIsNot(bundle: Bundle, d: District): string {
  const baseCostAdm = Math.round(d.adm);
  const currentAdm = Math.round(d.current_year_adm);
  const categoricalAdm = Math.round(d.categorical_adm);

  return `
    <div class="card" id="not" data-part="not">
      <h2>${anchor("not")}What this page is not</h2>
      <p class="note"><strong>None of the figures above is money this district received.</strong>
        Every one of them is the FY${bundle.fiscal_year} funding calculator's answer for it,
        published before the year it funds. What actually changed hands is on
        <a href="${routes.districtFinances(d.irn)}">finances</a>, taken from this district's own
        five-year forecast filing — audited actuals, on a different construction, in different
        years. Reading either as a check on the other is the mistake the two routes are kept
        apart to prevent.</p>
      ${
        d.casino.length > 0
          ? `<p class="note"><strong>And there is state money in neither.</strong> This district
             received ${money(
               d.casino[d.casino.length - 1]!.amount,
             )} from the casino county student fund in
             FY${d.casino[d.casino.length - 1]!.fiscal_year}. It is not an appropriation to the
             department, so no figure above computes it; it is booked outside the general fund, so
             no row on the finances route carries it either. It is on
             <a href="${routes.districtFinances(d.irn)}#casino">finances</a>, in a card of its
             own.</p>`
          : ""
      }
      <p class="note"><strong>This page divides by more than one pupil count.</strong> Base cost
        is built on ${count(baseCostAdm)} — a three-year average, because R.C. 3317.011 funds on
        the greater of that and the current year. The state share of it, and every categorical
        above, is paid on ${count(currentAdm)}: the current year alone.
        ${
          baseCostAdm === currentAdm
            ? `Those round to the same figure here, which they do not for 499 of Ohio's
               ${bundle.statewide.districts} districts.`
            : `Those are different numbers, as they are for 499 of Ohio's
               ${bundle.statewide.districts} districts.`
        }
        ${
          categoricalAdm === currentAdm
            ? ""
            : `The categoricals are paid on a third field again, ${count(categoricalAdm)}, which
               is the same count entered twice in the department's own sheet and corrected once.`
        }
        The corpus calls conflating them
        <a href="${routes.metric("enrolled-adm")}">the single most common arithmetic error
        available in Ohio school finance</a>, and
        <a href="${
          /*
           * The section, but only for the 432 districts that render it. `renderDenominators`
           * returns "" where the two valuations agree within 5%, so for 177 of 609 the anchor the
           * plan wanted named here is simply not in the target document — and a missing fragment
           * fails silently, landing the reader at the top of a page having been promised a
           * reconciliation. The same predicate, so the two cannot drift apart.
           */
          hasDenominators(d) ? routes.at(routes.districtTaxes(d.irn), routes.SECTIONS.district.denominators) : routes.districtTaxes(d.irn)
        }">the property tax page</a> reconciles the two the Department of Taxation and the
        Department of Education each publish.</p>
      <p class="note"><strong>The position card below is two other years on a fourth count.</strong>
        Assessed valuation per pupil there is an <strong>FY2023</strong> valuation and operating
        expenditure per pupil is <strong>${yearOf("profile")}</strong> spending — two different years inside one
        card — and both divide by enrolled ADM ${yearOf("profile")}, beside a model that funds
        FY${bundle.fiscal_year}. Finances shows ${yearOf("outcome.spending")} spending on the report card's unweighted
        headcount, and Outcome shows ${yearOf("outcome.spending")} spending on its need-weighted one. Subtracting any of
        them from any other produces a number that means nothing, which is why
        <code>denominators.ts</code> exists and why it says it cannot check a sentence.</p>
      <p class="note">Every figure on this page is also a column in
        <a href="/data/districts.csv">the district export</a>, unrounded and one row per district.
        This page used to close with a thirteen-row table of the same numbers rounded, which was
        the better answer in the build where it was added and had become a restatement of ten
        figures already above it. If a fixed-shape table on every district was what you were
        reading, the CSV is where it went.</p>
    </div>`;
}
