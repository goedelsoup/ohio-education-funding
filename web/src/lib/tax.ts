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
 * the mechanism is countable: at the time of writing, 301 of the 416 districts that *began*
 * above the floor saw their effective rate fall, against 7 of the 193 that began at it, and 97.7%
 * of every rate reduction in Ohio happened above it. Those figures are computed in `feed.ts`
 * rather than written into the copy, because they appear on 609 pages and a regenerated feed
 * would otherwise leave them wrong — as it left this comment wrong until the split was corrected.
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

/**
 * The two pupil counts, where they differ enough to matter.
 *
 * # The finding this exists to state
 *
 * Both departments publish a taxable valuation per pupil for every district, and they are not the
 * same number. The numerators are identical to the dollar — multiply the District Profile Report's
 * figure by its enrolled ADM and Table SD-1's `total_value` comes back to 1.000 for all 606
 * districts carrying both. Only the denominator differs, and it differs by a factor of 2.2 in
 * Youngstown, 1.9 in Dayton and 1.7 in Columbus.
 *
 * Taxation divides by the children who live in the district. Education divides by the ones it
 * teaches. The gap is charter, voucher and open-enrolment-out students, so it is widest in exactly
 * the districts where valuation per pupil does the most work in the aid formula — and the formula
 * reads Education's.
 *
 * The card only renders where the two are more than 5% apart, because for most districts they are
 * within a rounding of each other and a reader does not need the caution.
 */
export function renderDenominators(d: District): string {
  const latest = d.property_tax[d.property_tax.length - 1];
  // `adm_history[0]` is enrolled ADM FY2024 — the count the profile report's valuation per pupil
  // divides by, verified by multiplying back to SD-1's total value for all 606 districts. Not
  // `d.adm`, which is the base cost ADM and is a third number again.
  const enrolled = d.adm_history[0];
  if (!latest || latest.adm <= 0 || enrolled <= 0 || d.valuation_per_pupil == null) return "";

  const ratio = latest.value_per_pupil / d.valuation_per_pupil;
  if (Math.abs(ratio - 1) < 0.05) return "";

  const wider = latest.adm > enrolled;

  return `
    <div class="card">
      <h2>Two pupil counts, and why this page shows one of them</h2>
      <div class="scroll"><table>
        <thead><tr><th>Published by</th><th class="tnum">Pupils</th>
          <th class="tnum">Value per pupil</th><th>Used for</th></tr></thead>
        <tbody>
          <tr class="current">
            <th>Taxation, Table SD-1</th>
            <td class="tnum">${count(Math.round(latest.adm))}</td>
            <td class="tnum">${money(latest.value_per_pupil)}</td>
            <td class="n">Everything on this page.</td>
          </tr>
          <tr>
            <th>Education, District Profile Report</th>
            <td class="tnum">${count(Math.round(enrolled))}</td>
            <td class="tnum">${money(d.valuation_per_pupil)}</td>
            <td class="n">Enrolled ADM, FY2024. The funding formula's wealth measure, and every
              other page here.</td>
          </tr>
          <tr>
            <th>Education, base cost ADM</th>
            <td class="tnum">${count(Math.round(d.adm))}</td>
            <td class="tnum n">—</td>
            <td class="n">A third count. What base cost per pupil divides by — funded rather than
              enrolled, so it is ${pct(Math.abs(d.adm / enrolled - 1), 1)} from the row above.</td>
          </tr>
        </tbody>
      </table></div>
      <p class="note"><strong>The valuations are the same to the dollar. The pupil counts are
        not.</strong> Taxation divides by the children who live in this district; Education
        divides by the ones it teaches. The difference is students attending community schools,
        using vouchers, or open-enrolled elsewhere — the district's property base still stands
        behind them, and the two agencies disagree about whether they belong in the denominator.
        Here that makes the same district look
        ${pct(Math.abs(ratio - 1), 0)} ${ratio > 1 ? "wealthier" : "poorer"} per pupil depending
        on which table you read.</p>
      <p class="note">The formula reads Education's, so that is the figure driving this district's
        state share and the one on its
        <a href="${routes.district(d.irn)}">dashboard</a>. Neither is wrong; they answer different
        questions, and the only error available is to compare one against the other's statewide
        median. ${
          wider
            ? `Because Taxation's count here is the larger one, this page's figures are the more
               conservative reading of this district's wealth.`
            : `Because Taxation's count here is the smaller one, this page reads this district as
               wealthier per pupil than the formula does.`
        } The statewide median beside the figure above is computed on this table's basis, for
        that reason.</p>
    </div>`;
}

/**
 * What the mechanism the Fair School Funding Plan replaced would charge this district.
 *
 * # Why a counterfactual belongs on a property tax page
 *
 * The charge-off was a property tax calculation. For roughly three decades Ohio decided how much
 * of a district's cost the district should bear by multiplying a statutory millage — uniform
 * statewide, 23 mills by the end — against its assessed valuation, and subtracting the product
 * from the computed cost. Everything the rest of this page establishes about millage bears
 * directly on it.
 *
 * # And the failure the page can now count
 *
 * A uniform assumed rate only works if districts can levy at it. H.B. 920 guarantees they cannot:
 * effective rates fall as valuation rises, and a district whose own rate had dropped below the
 * charge-off rate was charged for revenue it had no way to collect. Ohio patched this with the
 * charge-off supplement — gap aid, $73.5m across 145 districts in FY2008 — rather than fixing it.
 * Against TY2024 rates, **half the state** is below the terminal charge-off rate.
 */
export function renderChargeOff(d: District, statewide: Statewide): string {
  const r = d.regime;
  if (!r || r.charge_off_local_share == null) return "";

  const short = r.mills_short_of_charge_off;
  const rate = d.millage?.observed_rate;

  return `
    <div class="card">
      <h2>What the mechanism this replaced would charge</h2>
      <p class="note">Before the Fair School Funding Plan, a district's own share of its cost was
        a flat <strong>${r.charge_off_mills.toFixed(0)} mills</strong> against its valuation —
        the same rate for every district in Ohio, whatever it could actually levy. Holding this
        district's base cost fixed and substituting that mechanism for the plan's local capacity
        measure:</p>

      <div class="scroll"><table>
        <thead><tr><th></th><th class="tnum">Per pupil</th><th>How it is arrived at</th></tr></thead>
        <tbody>
          <tr><th>Deemed local share, charge-off</th>
            <td class="tnum">${money(r.charge_off_local_share)}</td>
            <td class="n">${r.charge_off_mills.toFixed(0)} mills against
              ${money(d.valuation_per_pupil ?? 0)} of valuation per pupil — the Department of
              Education's figure, on the funding formula's pupil count, because that is the
              denominator the base cost it is subtracted from is expressed in.</td></tr>
          <tr><th>Local capacity, the plan</th>
            <td class="tnum">${
              r.local_capacity == null ? "—" : money(r.local_capacity)
            }</td>
            <td class="n">${
              r.local_capacity == null
                ? `Not recoverable. The minimum state share binds here, so all that is known is
                   that capacity exceeds a threshold — and a censored quantity is not a small one.`
                : `Property wealth blended with two income measures, recovered by subtraction from
                   base cost.`
            }</td></tr>
          <tr><th>Base cost aid, charge-off</th>
            <td class="tnum">${r.aid_charge_off == null ? "—" : money(r.aid_charge_off)}</td>
            <td class="n">${
              r.exceeds_base_cost
                ? `Nothing. The deemed share runs past the whole base cost, and the charge-off had
                   no minimum state share to stop at.`
                : `Base cost less the deemed local share, floored at zero.`
            }</td></tr>
          <tr class="current"><th>Base cost aid, the plan</th>
            <td class="tnum">${r.aid_fsfp == null ? "—" : money(r.aid_fsfp)}</td>
            <td class="n">What this district actually receives toward base cost.</td></tr>
          <tr><th>Difference</th>
            <td class="tnum">${
              r.difference == null
                ? "—"
                : `${r.difference >= 0 ? "+" : "−"}${money(Math.abs(r.difference))}`
            }</td>
            <td class="n">Plan minus charge-off. The statewide median is
              ${money(statewide.median_regime_difference)}.</td></tr>
        </tbody>
      </table></div>

      ${
        short != null && rate != null
          ? `<p class="note"><strong>This district would be charged for revenue it could not
             raise.</strong> The charge-off assumes ${r.charge_off_mills.toFixed(0)} mills; this
             district's effective Class I rate is ${rate.toFixed(2)}, which is
             ${short.toFixed(2)} mills short. That gap is the phantom revenue the mechanism became
             known for, and it is not a district-level failing — H.B. 920 produces it
             automatically, by rolling effective rates down as valuation rises while the statutory
             rate stood still. ${count(statewide.below_charge_off_rate)} of
             ${count(statewide.districts)} districts are below the rate today. Ohio's answer was a
             supplement rather than a floor: gap aid, $73.5m across 145 districts in FY2008.</p>`
          : `<p class="note">This district's effective Class I rate is at or above the
             ${r.charge_off_mills.toFixed(0)} mills the charge-off assumes, so it is one of the
             ${count(statewide.districts - statewide.below_charge_off_rate)} that could actually
             levy what the mechanism deemed it able to. The other
             ${count(statewide.below_charge_off_rate)} could not.</p>`
      }

      <p class="note"><strong>What this comparison is, and three things it is not.</strong> It is
        a counterfactual at FY2027 inputs — the plan's own computed base cost held fixed, with only
        the local share mechanism swapped. It is <em>not</em> a reconstruction of any year the
        charge-off governed: those need the era's formula amount, cost-of-doing-business factor and
        DPIA, none of which this project holds. It is <em>not</em> a full regime diff — base cost,
        the guarantee and every categorical have no declared predecessor, so one row of the
        calculation is all that is comparable. And the charge-off's base narrowed over its life
        from total taxable value to an H.B. 920-adjusted recognised valuation, which this project
        does not hold; every figure here is on the earlier, wider base and therefore overstates
        what the charge-off would have taken.</p>

      <p class="note">One seam inside the arithmetic, stated rather than smoothed over. The deemed
        local share is per <strong>enrolled</strong> ADM, because that is what the published
        valuation per pupil divides by; the base cost it is subtracted from is per
        <strong>funded</strong> base cost ADM. Those two counts differ by
        ${pct(Math.abs(d.adm / (d.adm_history[0] || d.adm) - 1), 1)} for this district and by a
        median of 1.6% statewide. The subtraction is the one <code>regime-diff</code> performs and
        the one the mechanism's own description implies, and it is not exact.</p>

      ${
        r.residual == null
          ? `<p class="note">The residual is unreportable here, because the local capacity side is
             censored. The totals still differ and the cause is not attributable to a component —
             which is the honest state of the comparison rather than a gap in it.</p>`
          : Math.abs(r.residual) < 0.005
            ? ""
            : `<p class="note">The decomposition leaves ${money(Math.abs(r.residual))} per pupil
               unexplained. Holding base cost fixed means the local share is the only thing that
               can differ, so a residual means the deemed share ran past the cost it was subtracted
               from and the floor at zero absorbed the rest. Seven districts are in this position.</p>`
      }
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
