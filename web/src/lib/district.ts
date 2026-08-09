/**
 * The district view: what the formula computes for one district, and what it actually gets.
 *
 * Each card is exported on its own rather than assembled here. The view used to be one function
 * because it was one tab; it is now four routes — the dashboard, the outcome, the finances, and
 * the scenario — and which cards go on which page is a routing decision, made in `src/pages/`.
 *
 * Every function here is a pure string builder with no DOM behind it, which is what lets Astro
 * call them at build time and put the result in the document.
 */

import type { FanPoint } from "./chart.ts";
import { barSpec, fanSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import { count, escapeHtml, money, ordinal, pct, percentileOf, signedMoney } from "./format.ts";
import { apply, currentLaw } from "./policy.ts";
import { forecastPath, growthPrior, observations } from "./project.ts";
import { realChange, series, type Basis } from "./real.ts";
import * as routes from "./routes.ts";
import type { Bar } from "./chart.ts";
import type { Bundle, District, Statewide } from "./types.ts";

function strip(
  label: string,
  value: number | null,
  sorted: number[],
  href?: string,
): string {
  const named = href
    ? `<a href="${href}">${escapeHtml(label)}</a>`
    : escapeHtml(label);
  if (value == null) return `<p class="note">${named}: not reported.</p>`;
  const p = percentileOf(sorted, value);
  return `
    <div class="strip-row">
      <div class="strip-head">
        <span>${named}</span>
        <strong class="tnum">${money(value)}</strong>
      </div>
      <div class="strip"><div class="marker" style="left:calc(${(p * 100).toFixed(1)}% - 1.5px)"></div></div>
      <div class="scale">
        <span>${money(sorted[0] ?? 0)}</span>
        <span>${ordinal(Math.round(p * 100))} percentile</span>
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
    <div class="card">
      <h2>What a year of enrollment is worth here</h2>
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
    <div class="chartwrap" data-chart="district-fan">${renderToString(fanSpec(
      points,
      (v) => money(v),
      (p) =>
        p.observed
          ? `FY${p.year}: ${money(p.point)} at published enrollment — exact`
          : `FY${p.year}: ${money(p.low)} – ${money(p.high)}, central ${money(p.point)}`,
    ))}</div>
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
  const cashChange = latest.ending_cash - first.ending_cash;
  const yearsOfSpending =
    latest.total_expenditure > 0 ? latest.ending_cash / latest.total_expenditure : null;
  // Counted on the published figures: whether a year was run at a deficit is a fact about that
  // year's own dollars, and deflating both sides cannot change it.
  const deficits = d.finances.filter(
    (y) => y.total_expenditure > y.total_revenue,
  ).length;
  const realAid = realChange(bundle.deflator, d.finances, (y) => y.state_aid);

  const peak = shown.reduce((a, b) => (b.ending_cash > a.ending_cash ? b : a));
  const bars = shown.map((y) => ({
    label: `FY${y.fiscal_year}`,
    value: y.ending_cash,
    hover: `FY${y.fiscal_year}: ${money(y.ending_cash)} held, ${money(y.total_revenue)} in, ${money(y.total_expenditure)} out`,
    ...(y.fiscal_year === peak.fiscal_year || y.fiscal_year === latest.fiscal_year
      ? { direct: money(y.ending_cash) }
      : {}),
  }));

  return `
    <div class="card">
      <h2>What it actually received, and what it holds${
        converted ? `, in FY${base} dollars` : ""
      }</h2>
      <div class="tiles">
        <div class="tile"><div class="k">Cash on hand, FY${latest.fiscal_year}</div>
          <div class="v">${money(latest.ending_cash)}</div>
          <div class="n">${
            yearsOfSpending == null
              ? "no spending reported"
              : `${yearsOfSpending.toFixed(2)} years of spending at this rate`
          }</div></div>
        <div class="tile"><div class="k">Change since FY${first.fiscal_year}</div>
          <div class="v ${cashChange < 0 ? "loss" : "gain"}">${signedMoney(cashChange)}</div>
          <div class="n">carry-over into FY${latest.fiscal_year + 1} is
            ${money(latest.ending_cash)}</div></div>
        <div class="tile"><div class="k">State aid, FY${first.fiscal_year}–FY${latest.fiscal_year}</div>
          <div class="v ${(realAid ?? 0) < 0 ? "loss" : "gain"}">${
            realAid == null ? "—" : pct(realAid, 1)
          }</div>
          <div class="n">real; ${pct(
            d.finances[d.finances.length - 1]!.state_aid / d.finances[0]!.state_aid - 1,
            1,
          )} nominal. ${deficits} of ${d.finances.length} years run at a deficit</div></div>
      </div>

      <div class="chartwrap" data-chart="cash">${renderToString(barSpec(bars))}</div>
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

/** The three figures a reader arriving at a district came for. */
export function renderHeadline(d: District): string {
  const guaranteePP = d.realized_aid_per_pupil - d.formula_aid_per_pupil;
  return `
    <div class="tiles">
      <div class="tile"><div class="k">Base cost / pupil</div>
        <div class="v">${money(d.base_cost_per_pupil)}</div>
        <div class="n">what the plan says it costs</div></div>
      <div class="tile"><div class="k">State aid / pupil</div>
        <div class="v">${money(d.realized_aid_per_pupil)}</div>
        <div class="n">${
          d.on_guarantee
            ? money(guaranteePP) + " of it from the guarantee"
            : "all from the formula"
        }</div></div>
      <div class="tile"><div class="k">Enrolled ADM</div>
        <div class="v">${d.adm.toLocaleString("en-US", { maximumFractionDigits: 0 })}</div>
        <div class="n">base cost enrolled</div></div>
    </div>`;
}

/** Where this district's state aid comes from, and the roles it plays in the formula. */
export function renderAidSource(bundle: Bundle, d: District): string {
  const formulaPP = d.formula_aid_per_pupil;
  const realizedPP = d.realized_aid_per_pupil;
  const guaranteePP = realizedPP - formulaPP;
  const total = Math.max(realizedPP, 1);
  const formulaWidth = (formulaPP / total) * 100;
  const guaranteeWidth = (guaranteePP / total) * 100;

  const flags: string[] = [
    d.on_guarantee ? "Funded by the guarantee, not the formula" : "On formula",
  ];
  if (d.at_millage_floor) flags.push("At or below the 20-mill floor");
  else if (d.near_millage_floor) flags.push("Within a twentieth of a mill of the floor");
  if (d.at_minimum_state_share) flags.push("At the minimum state share");
  if (d.millage?.cumulative_reduction != null && d.millage.cumulative_reduction > 0.005) {
    flags.push(`${pct(d.millage.cumulative_reduction, 0)} of voted millage reduced away`);
  }
  if (d.enrollment_change != null && d.enrollment_change < 0) {
    flags.push(`Enrollment down ${pct(-d.enrollment_change)} FY2024→FY2026`);
  }

  return `
    <div class="card">
      <h2>Where the state aid comes from</h2>
      <div class="barwrap">
        <div class="bar" role="img" aria-label="Formula aid ${money(formulaPP)} per pupil, guarantee ${money(guaranteePP)} per pupil">
          <div class="seg formula ${guaranteeWidth <= 0 ? "only" : ""}" style="width:${formulaWidth}%"></div>
          ${guaranteeWidth > 0 ? `<div class="seg guarantee" style="width:${guaranteeWidth}%"></div>` : ""}
        </div>
        <div class="legend">
          <span><i class="sw formula"></i> Formula ${money(formulaPP)}/pupil</span>
          ${guaranteeWidth > 0 ? `<span><i class="sw guarantee"></i> Guarantee ${money(guaranteePP)}/pupil</span>` : ""}
        </div>
      </div>
      <p class="note">${
        d.on_guarantee
          ? `The formula computes ${money(formulaPP)} per pupil. This district receives
             ${money(realizedPP)} because the temporary transitional aid guarantee holds it at
             what it received in <strong>FY2020</strong> — a year Ohio froze funding under the
             Bridge formula rather than computing it. The formula produces
             <strong>${pct(formulaPP / realizedPP, 0)}</strong> of that level.`
          : `This district is funded by the formula, so an increase in its computed base cost
             reaches it in full — unlike the ${bundle.statewide.on_guarantee} districts held on
             the guarantee.`
      }</p>
      <div class="flags">${flags
        .map((f) => `<span class="flag">${escapeHtml(f)}</span>`)
        .join("")}</div>
    </div>`;
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
    <div class="card">
      <h2>Position among Ohio's ${bundle.statewide.districts} districts</h2>
      ${strip(
        "Assessed valuation per pupil",
        d.valuation_per_pupil,
        valuations,
        routes.metric("assessed-valuation-per-pupil"),
      )}
      ${strip(
        "Operating expenditure per pupil",
        d.operating_expenditure_per_pupil,
        expenditures,
        routes.metric("per-pupil-operating-expenditure"),
      )}
    </div>`;
}

/** Every figure the feed carries for this district, unrounded and unnarrated. */
export function renderDetail(d: District): string {
  const formulaPP = d.formula_aid_per_pupil;
  const guaranteePP = d.realized_aid_per_pupil - formulaPP;
  return `
    <div class="card">
      <h2>Detail</h2>
      <div class="scroll"><table><tbody>
        <tr><th>Base cost per pupil</th><td>${money(d.base_cost_per_pupil, 2)}</td></tr>
        <tr><th>Aggregate base cost</th><td>${money(d.aggregate_base_cost)}</td></tr>
        <tr><th>State share of base cost</th><td>${money(d.base_cost_state_share)}</td></tr>
        <tr><th>Categorical funding<div class="n">Six programs; the split is below.</div></th>
            <td>${money(d.categorical_funding)}</td></tr>
        <tr><th>Formula aid per pupil</th><td>${money(formulaPP, 2)}</td></tr>
        <tr><th>Guarantee per pupil</th><td>${money(guaranteePP, 2)}</td></tr>
        <tr><th>Guarantee, total</th><td>${money(d.guarantee)}</td></tr>
        <tr><th>Assessed valuation per pupil</th><td>${money(d.valuation_per_pupil)}</td></tr>
        <tr><th>Effective Class 1 millage</th><td>${
          d.effective_class1_millage == null ? "—" : d.effective_class1_millage.toFixed(2)
        }</td></tr>
        <tr><th>Operating expenditure per pupil</th><td>${money(d.operating_expenditure_per_pupil)}</td></tr>
        <tr><th>Economically disadvantaged</th><td>${pct(d.economically_disadvantaged)}</td></tr>
        <tr><th>Enrolled ADM FY2026</th><td>${d.current_year_adm.toLocaleString("en-US", { maximumFractionDigits: 0 })}</td></tr>
        <tr><th>Enrollment change FY2024→FY2026</th><td>${pct(d.enrollment_change)}</td></tr>
      </tbody></table></div>
      <p class="note">FY2026 enrolled ADM is partly a departmental estimate: the calculator is
        published before that fiscal year closes.</p>
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

  const parts = [
    ["Targeted assistance", c.targeted_assistance, "Equalisation for low-valuation districts. Zero once a district has enough."],
    ["Special education", c.special_education, "Six weighted categories of disability."],
    ["Disadvantaged Pupil Impact Aid", c.dpia, "Driven by the economically disadvantaged count."],
    ["Career-technical education", c.career_technical, ""],
    ["Gifted", c.gifted, "Identification and service."],
    ["English learners", c.english_learners, "Three weights by time in the country."],
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
    <div class="card">
      <h2>The categorical half, in its six parts</h2>
      <div class="chartwrap" data-chart="categoricals">${renderToString(barSpec(bars))}</div>

      <div class="scroll"><table>
        <thead><tr><th>Program</th><th class="tnum">Amount</th><th class="tnum">Share</th></tr></thead>
        <tbody>
          ${parts
            .map(
              ([label, value, note]) => `<tr${value <= 0 ? ' class="n"' : ""}>
                <th>${escapeHtml(label)}${note ? `<div class="n">${escapeHtml(note)}</div>` : ""}</th>
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
    <h3>Special education, by category</h3>
    <div class="scroll"><table>
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
