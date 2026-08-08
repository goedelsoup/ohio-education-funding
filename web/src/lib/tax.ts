/**
 * The local half of the formula: what property in a district is worth, and what is charged on it.
 *
 * # Why this needed its own view
 *
 * Local capacity is 60% assessed valuation and decides the state's residual share, so property is
 * most of what determines a district's funding. The site represented all of it with one number —
 * valuation per pupil — which is the local side of Ohio school funding rendered as a scalar.
 *
 * The data is Table SD-1, published by the **Department of Taxation** rather than the Department
 * of Education. Two departments describing the same district are not obliged to agree, and where
 * these overlap they do: SD-1's effective Class I rate matches the District Profile Report's for
 * all 606 districts carrying both, to 0.01 mills. That is worth stating on the page, because a
 * reader has no other way to know the two halves of the state are consistent here.
 *
 * # And why two tax years rather than one
 *
 * H.B. 920 is a mechanism that only exists as a change. Its reduction factors roll a district's
 * effective rate back as valuation rises, holding revenue from existing levies roughly flat in
 * nominal terms — but they cannot roll a rate below twenty mills. So a reappraisal does opposite
 * things on either side of that floor, and a single year cannot show it. With TY2023 and TY2024
 * the mechanism is countable: at the time of writing, 304 of the 439 districts above the floor
 * saw their effective rate fall against 4 of the 170 at it, and 98.7% of every rate reduction in
 * Ohio happened above it. Those figures are computed in `feed.ts` rather than written into the
 * copy, because they appear on 609 pages and a regenerated feed would otherwise leave them wrong.
 */

import type { Bar } from "./chart.ts";
import { count, escapeHtml, money, pct } from "./format.ts";
import { barSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import * as routes from "./routes.ts";
import type { TaxStatewide } from "./feed.ts";
import type { District, PropertyTaxYear, Statewide } from "./types.ts";

/**
 * The statutory reduction-factor floor, in mills — `millage::SCHOOL_DISTRICT_FLOOR`.
 *
 * Restated here because this module renders prose that names it, not because it classifies with
 * it: `at_millage_floor` and `near_millage_floor` arrive from the feed already decided by the
 * Rust crate that cites R.C. 319.301. Nothing below re-derives that judgement.
 */
const FLOOR = 20;

/** A signed change, written so a reader does not have to work out the direction. */
function change(from: number, to: number): string {
  if (from === 0) return "—";
  const delta = to / from - 1;
  if (Math.abs(delta) < 0.00005) return "no change";
  return `${delta > 0 ? "+" : "−"}${pct(Math.abs(delta), 2)}`;
}

/** The property classes, in the order the statute treats them. */
function classes(y: PropertyTaxYear): { label: string; value: number; group: string }[] {
  return [
    { label: "Residential", value: y.residential_value, group: "Class I" },
    { label: "Agricultural", value: y.agricultural_value, group: "Class I" },
    { label: "Commercial", value: y.commercial_value, group: "Class II" },
    { label: "Industrial", value: y.industrial_value, group: "Class II" },
    { label: "Mineral", value: y.mineral_value, group: "Class II" },
    { label: "Railroad", value: y.railroad_value, group: "Class II" },
    { label: "Public utility", value: y.public_utility_value, group: "Neither" },
  ];
}

/** What the district's tax base is made of. */
export function renderTaxBase(d: District): string {
  const latest = d.property_tax[d.property_tax.length - 1];
  if (!latest) return "";

  const parts = classes(latest).filter((c) => c.value > 0);
  const bars: Bar[] = parts.map((part) => ({
    label: part.label,
    value: part.value,
    direct: pct(part.value / latest.total_value, 0),
    hover: `${part.label} (${part.group}): ${money(part.value)}, ${pct(part.value / latest.total_value, 1)} of the base`,
  }));

  const residentialShare = (latest.residential_value + latest.agricultural_value) / latest.total_value;

  return `
    <div class="card">
      <h2>What the tax base is made of, TY${latest.tax_year}</h2>
      <div class="chartwrap" data-chart="tax-base">${renderToString(barSpec(bars))}</div>
      <p class="note">Total taxable value ${money(latest.total_value)}, or
        ${money(latest.value_per_pupil)} per pupil.
        <strong>${pct(residentialShare, 0)}</strong> of it is Class I — residential and
        agricultural — which carries its own reduction factor separate from everything else.</p>
      <p class="note">The split matters because the two classes are reduced separately. A district
        whose base is mostly residential and one whose base is mostly commercial respond
        differently to the same reappraisal, and their effective rates diverge over time even if
        voters approved identical millage.</p>
    </div>`;
}

/**
 * What changed between the two tax years, and what the floor did about it.
 *
 * The centrepiece. For a district at the floor, reduction factors have stopped operating and
 * revenue rises with valuation; for one above it, they roll the rate back and revenue does not.
 * Which of those a district is on is decided by its levy history rather than by any current
 * decision, and it is the highest-leverage single fact about its finances.
 */
export function renderTaxChange(d: District, statewide: TaxStatewide): string {
  if (d.property_tax.length < 2) return "";
  const [before, after] = [d.property_tax[0]!, d.property_tax[d.property_tax.length - 1]!];

  const rateMoved = after.class1_rate - before.class1_rate;
  const atFloor = d.at_millage_floor;
  // Twenty-one districts charge under twenty mills. The floor guarantees what twenty mills would
  // raise, so a district below it was never subject to a reduction at all — it reads as being at
  // the floor and arrives there by a different route, which is worth distinguishing. Keyed off
  // the charged rate rather than the voted one so that Kelleys Island, which has no profile row,
  // is not misdescribed by the fallback.
  const belowFloor = after.class1_rate < FLOOR - 0.005;
  const voted = d.voted_operating_millage;
  const valueGrowth = before.class1_value > 0 ? after.class1_value / before.class1_value - 1 : 0;
  const chargeGrowth =
    before.class1_taxes_charged > 0
      ? after.class1_taxes_charged / before.class1_taxes_charged - 1
      : 0;

  const rows = [
    ["Class I taxable value", before.class1_value, after.class1_value, money],
    ["Class I effective millage", before.class1_rate, after.class1_rate, (v: number) => v.toFixed(4)],
    ["Class I tax charged", before.class1_taxes_charged, after.class1_taxes_charged, money],
    ["Class II taxable value", before.class2_value, after.class2_value, money],
    ["Class II effective millage", before.class2_rate, after.class2_rate, (v: number) => v.toFixed(4)],
    ["Class II tax charged", before.class2_taxes_charged, after.class2_taxes_charged, money],
    [
      "Real property tax charged",
      before.real_property_taxes_charged,
      after.real_property_taxes_charged,
      money,
    ],
    ["Value per pupil", before.value_per_pupil, after.value_per_pupil, money],
  ] as const;

  return `
    <div class="card">
      <h2>TY${before.tax_year} to TY${after.tax_year}</h2>
      <div class="scroll"><table>
        <thead><tr>
          <th></th><th>TY${before.tax_year}</th><th>TY${after.tax_year}</th><th>Change</th>
        </tr></thead>
        <tbody>${rows
          .map(
            ([label, from, to, format]) => `<tr>
              <th>${escapeHtml(label)}</th>
              <td class="tnum">${format(from)}</td>
              <td class="tnum">${format(to)}</td>
              <td class="tnum n">${change(from, to)}</td>
            </tr>`,
          )
          .join("")}</tbody>
      </table></div>

      <p class="note">${
        belowFloor
          ? `<strong>This district charges less than twenty mills, so H.B. 920's reduction factors
             are not operating on it — and never have.</strong>
             <a href="${routes.parameter("twenty-mill-floor")}">The floor</a> is a guarantee of
             what twenty mills would raise rather than a rate anyone is held to, so a district
             already below it has nothing that can be reduced.${
               voted != null
                 ? ` Its voters approved ${voted.toFixed(2)} mills and it charges
                    ${after.class1_rate.toFixed(2)}.`
                 : ""
             } Twenty-one districts in Ohio charge under twenty effective Class I mills. Its Class
             I value moved ${pct(valueGrowth, 2)} and its charge moved ${pct(chargeGrowth, 2)}: a
             reappraisal reaches revenue here, for a different reason than at the floor.`
          : atFloor
            ? `<strong>This district is at the
             <a href="${routes.parameter("twenty-mill-floor")}">20-mill floor</a>, so H.B. 920's
             reduction factors have stopped operating on it.</strong> Its Class I value moved
             ${pct(valueGrowth, 2)} and its Class I charge moved ${pct(chargeGrowth, 2)} — the two
             track each other, because there is no reduction left to absorb the difference. A
             reappraisal is a revenue event here.`
          : d.near_millage_floor
            ? `<strong>This district is above the
             <a href="${routes.parameter("twenty-mill-floor")}">20-mill floor</a> by
             ${(after.class1_rate - FLOOR).toFixed(4)} mills</strong> — close enough that the
             distinction carries little meaning. Reduction factors are technically operative and
             have almost nothing left to operate on: its rate moved
             ${rateMoved >= 0 ? "+" : "−"}${Math.abs(rateMoved).toFixed(4)} mills against a
             ${pct(valueGrowth, 2)} change in value, and its charge tracked its value to within
             ${pct(Math.abs(chargeGrowth - valueGrowth), 2)}. Read it as a floor district whose
             rate happens to round up.`
            : `<strong>This district is above the
             <a href="${routes.parameter("twenty-mill-floor")}">20-mill floor</a>, so H.B. 920's
             reduction factors are fully operative.</strong> Its Class I effective rate moved
             ${rateMoved >= 0 ? "+" : "−"}${Math.abs(rateMoved).toFixed(4)} mills against a
             ${pct(valueGrowth, 2)} change in value.
             ${
               rateMoved < -0.0005
                 ? `That is the factors doing what they were written to do: as valuation rises they
                    roll the rate back, so revenue from existing levies stays roughly flat in
                    nominal terms and the district has to return to voters to keep pace.`
                 : `The factors reduce the rate on <em>existing</em> levies against
                    <em>existing</em> property, and neither newly voted millage nor new
                    construction is subject to them — so a rate that holds or rises above the floor
                    means one of those outweighed the reduction, not that the reduction stopped.`
             }`
      }</p>

      <p class="note">Statewide the split is stark. Of the
        ${count(statewide.districts.aboveFloor)} districts that began TY${before.tax_year} above
        the floor, <strong>${count(statewide.rateFell.aboveFloor)} saw their effective Class I rate
        fall</strong> by TY${after.tax_year}; of the ${count(statewide.districts.atFloor)} that
        began at it, <strong>${count(statewide.rateFell.atFloor)} did</strong>.
        ${pct(statewide.reductionsAboveFloor, 1)} of every effective-rate reduction in Ohio happens
        above the floor — the mechanism working exactly as written, and the reason floor status is
        the highest-leverage single fact about a district's local revenue.</p>

      <p class="note">With one qualification worth making, because the split above invites
        treating floor status as a fixed property of a district and it is not.
        <strong>${count(statewide.nearFloor)} districts sit above the floor by less than a
        twentieth of a mill</strong>, and
        <strong>${count(statewide.crossedTheFloor)} crossed 20.0000 in one direction or the other
        between these two tax years</strong> — most of them by under five hundredths. For those
        the binary is decided in the fourth decimal place and will likely read the other way next
        year. It is a real threshold with real consequences, and for roughly a district in eight
        it is also a coin toss.</p>

      <p class="note">Tax <em>charged</em> is not money <em>received</em>. A tax year is collected
        across the following calendar year in two settlements, so a levy passed in November is
        charged in full for that tax year and arrives across two fiscal years. Any comparison of a
        charge to a year's spending inherits that gap, and it is largest exactly where a levy has
        just passed.</p>
    </div>`;
}

/**
 * What voters approved against what the reduction factors left, and the gap between them.
 *
 * # The number the site could not previously state
 *
 * Every page here has been able to say that H.B. 920 reduces effective millage as valuation
 * rises. None could say by how much, because the voted rate — the rate on the ballot — sat
 * unparsed in column 6 of the profile CSV. With it the mechanism stops being a description: the
 * median Ohio district has voted 42.32 mills of current operating levy and collects 23.40, and
 * the median district has lost 42% of what its voters approved.
 *
 * # And the prediction, which is the calculator rather than the table
 *
 * The `millage` crate applied to two tax years gives the rate reduction factors alone would
 * produce. The residual against the charged rate is what they do not reach — new construction and
 * newly voted millage, both exempt by statute. It is signed, and the sign is informative: a
 * positive residual is a district that grew or voted, a negative one is a levy that expired.
 */
export function renderMillage(d: District, statewide: Statewide): string {
  const m = d.millage;
  if (!m) return "";

  const voted = d.voted_operating_millage;
  const reduced = m.cumulative_reduction;
  const residual = m.residual;

  // Under a hundredth of a mill is the precision Table SD-1 publishes to, so anything smaller is
  // the prediction and the charge being the same number written twice.
  const explained = Math.abs(residual) < 0.01;

  // The floor clamped the prediction only if it landed exactly on the floor having started above
  // it. A district already below twenty mills was never clamped by anything.
  const floorClamped =
    Math.abs(m.predicted_rate - FLOOR) < 0.005 && m.prior_rate > FLOOR + 0.005;

  return `
    <div class="card">
      <h2>What voters approved, and what the factors left</h2>

      ${
        voted == null || reduced == null
          ? `<p class="note">The District Profile Report carries no voted operating millage for
             this district, so the reduction cannot be stated as a share here.</p>`
          : `<div class="tiles">
              <div class="tile"><div class="k">Voted current operating millage</div>
                <div class="v">${voted.toFixed(2)}</div>
                <div class="n">TY2023, the rate on the ballot</div></div>
              <div class="tile"><div class="k">Effective Class I millage</div>
                <div class="v">${m.observed_rate.toFixed(2)}</div>
                <div class="n">TY${m.tax_year}, the rate anyone pays</div></div>
              <div class="tile"><div class="k">Taken by reduction factors</div>
                <div class="v">${pct(reduced, 0)}</div>
                <div class="n">statewide median ${pct(statewide.median_millage_reduction, 0)}</div></div>
            </div>
            <p class="note">${
              reduced < 0.005
                ? `Voters approved <strong>${voted.toFixed(2)} mills</strong> and the district
                   charges <strong>${m.observed_rate.toFixed(2)}</strong> — the same rate, and one
                   of very few places in Ohio where that is true. Reduction factors have taken
                   nothing, because
                   <a href="${routes.parameter("twenty-mill-floor")}">the twenty-mill floor</a>
                   guarantees what twenty mills would raise and this district has never voted
                   twenty. There is no reduction to make. Everywhere else the two numbers diverge:
                   the median district voted
                   ${statewide.median_voted_millage.toFixed(2)} mills, collects
                   ${statewide.median_effective_millage.toFixed(2)}, and has lost
                   ${pct(statewide.median_millage_reduction, 0)} of what its voters approved.`
                : `Voters approved <strong>${voted.toFixed(2)} mills</strong> and the district
                   charges <strong>${m.observed_rate.toFixed(2)}</strong>. That gap is not a rebate
                   or a rollback anyone voted for: H.B. 920's reduction factors have removed
                   ${pct(reduced, 0)} of the approved rate over the life of the levies, a hundredth
                   of a mill at a time, as reappraisals raised the value of property that was
                   already there. Statewide the median district has lost
                   ${pct(statewide.median_millage_reduction, 0)} the same way — the median voted
                   rate is ${statewide.median_voted_millage.toFixed(2)} mills against an effective
                   ${statewide.median_effective_millage.toFixed(2)}.`
            }</p>`
      }

      <h3>What the factors alone predict</h3>
      <div class="scroll"><table>
        <thead><tr><th></th><th class="tnum">Mills</th><th>What it is</th></tr></thead>
        <tbody>
          <tr><th>Charged TY${m.tax_year - 1}</th>
            <td class="tnum">${m.prior_rate.toFixed(4)}</td>
            <td class="n">Table SD-1, the year before.</td></tr>
          <tr><th>Predicted TY${m.tax_year}</th>
            <td class="tnum">${m.predicted_rate.toFixed(4)}</td>
            <td class="n">The prior rate scaled by the change in Class I value${
              // The floor only appears in the prediction when it actually clamped it. Saying it
              // "binds here" for a district charging under twenty mills would be backwards: the
              // floor is what it never reached, not what stopped it.
              floorClamped ? ", held at the statutory floor" : ""
            }.</td></tr>
          <tr class="current"><th>Charged TY${m.tax_year}</th>
            <td class="tnum">${m.observed_rate.toFixed(4)}</td>
            <td class="n">Table SD-1, observed.</td></tr>
          <tr><th>Residual</th>
            <td class="tnum">${residual >= 0 ? "+" : "−"}${Math.abs(residual).toFixed(4)}</td>
            <td class="n">What reduction factors cannot account for.</td></tr>
        </tbody>
      </table></div>

      <p class="note">${
        explained
          ? `<strong>The reduction factors account for this district's rate exactly.</strong>
             Nothing happened here in ${m.tax_year} that they do not explain — no levy passed, no
             levy expired, and new construction was too small to register against a base this
             size.`
          : residual > 0
            ? `<strong>The charged rate is ${Math.abs(residual).toFixed(2)} mills above what the
               factors alone predict.</strong> They apply to existing levies on existing property
               and to nothing else, so the excess is millage they never reached: a levy passed in
               the interval, or new construction added to the base after the reduction was
               computed. This page cannot tell you which — Table SD-1 publishes the outcome, not
               the levy history that produced it — but it can tell you the reduction factors are
               not the reason.`
            : `<strong>The charged rate is ${Math.abs(residual).toFixed(2)} mills below what the
               factors alone predict.</strong> Reduction factors cannot do that; they hold a
               levy's yield roughly flat, they do not retire it. A rate that falls faster than
               they explain means millage came off the books — a levy reaching its term, or one
               replaced at a lower rate.`
      }</p>

      <div class="tiles">
        <div class="tile"><div class="k">What one mill raises here</div>
          <div class="v">${money(m.yield_per_mill_per_pupil)}</div>
          <div class="n">per pupil, TY${m.tax_year}</div></div>
        <div class="tile"><div class="k">Statewide median</div>
          <div class="v">${money(statewide.median_yield_per_mill)}</div>
          <div class="n">per pupil, per mill</div></div>
        <div class="tile"><div class="k">To raise a median mill</div>
          <div class="v">${(statewide.median_yield_per_mill / m.yield_per_mill_per_pupil).toFixed(2)}</div>
          <div class="n">mills needed here</div></div>
      </div>
      <p class="note">This is the local half of the formula in one number, and the reason a
        millage comparison between districts means almost nothing on its own. One mill raises
        ${money(m.yield_per_mill_per_pupil)} per pupil here against a statewide median of
        ${money(statewide.median_yield_per_mill)}, so
        ${
          m.yield_per_mill_per_pupil >= statewide.median_yield_per_mill
            ? `this district reaches the median district's revenue with
               ${(statewide.median_yield_per_mill / m.yield_per_mill_per_pupil).toFixed(2)} mills
               of effort. A lower rate here is not necessarily a lower commitment.`
            : `matching the median district's revenue takes
               ${(statewide.median_yield_per_mill / m.yield_per_mill_per_pupil).toFixed(2)} mills
               here. A higher rate is not necessarily a higher commitment — it can be the same
               commitment against a smaller base.`
        }
        Across the state one mill runs from
        ${money(statewide.min_yield_per_mill)} per pupil to
        ${money(statewide.max_yield_per_mill)}, a spread of
        ${Math.round(statewide.max_yield_per_mill / statewide.min_yield_per_mill)}
        times, which is the inequality
        <a href="${routes.parameter("state-share-percentage")}">the state share</a> exists to
        offset.</p>
    </div>`;
}

/** How the charge compares to what the district spends — with the caveat that makes it readable. */
export function renderTaxAgainstSpending(d: District, statewide: TaxStatewide): string {
  const latest = d.property_tax[d.property_tax.length - 1];
  const spending = d.spending_by_function;
  if (!latest || !spending || spending.adm <= 0) return "";

  const operating = spending.operating_per_pupil * spending.adm;
  if (operating <= 0) return "";
  const share = latest.real_property_taxes_charged / operating;

  return `
    <div class="card">
      <h2>Against what the district spends</h2>
      <div class="tiles">
        <div class="tile"><div class="k">Real property tax charged, TY${latest.tax_year}</div>
          <div class="v">${money(latest.real_property_taxes_charged)}</div>
          <div class="n">current expenses, excluding joint vocational levies</div></div>
        <div class="tile"><div class="k">Operating spending, FY2025</div>
          <div class="v">${money(operating)}</div>
          <div class="n">${money(spending.operating_per_pupil)} × ${count(Math.round(spending.adm))} pupils</div></div>
        <div class="tile"><div class="k">Charge as a share of spending</div>
          <div class="v">${pct(share, 0)}</div>
          <div class="n">statewide median is
            ${pct(statewide.medianChargeShare, 0)}</div></div>
      </div>
      <p class="note">A ratio between two different things, and worth reading as one. The
        numerator is a <strong>tax year</strong> charge and the denominator a <strong>fiscal
        year</strong> of spending; they overlap but do not coincide, and a district that passed a
        levy in the interval shows a numerator its denominator's year never fully received.</p>
      <p class="note">${
        statewide.chargedMoreThanSpent.length === 0
          ? "No district is charged more in real property tax than it spends on operations."
          : `<strong>${count(statewide.chargedMoreThanSpent.length)} districts are charged more
             than they spend</strong> —
             ${statewide.chargedMoreThanSpent
               .map((x) => `${escapeHtml(x.name)} at ${pct(x.share, 0)}`)
               .join(", ")}. For the first two that is arithmetic rather than policy: a district
             whose valuation per pupil pins it to the minimum state share has to raise essentially
             all of its own cost, and the residual definition of state aid leaves nothing else for
             it to do. The third is the timing gap above — a levy passed in November 2024 is
             charged in full for TY2024, and roughly half of it did not reach the district until
             the following fiscal year.`
      }</p>
    </div>`;
}
