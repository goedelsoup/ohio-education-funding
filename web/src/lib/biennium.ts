/**
 * What a district was paid across the biennium, on two measures that disagree.
 *
 * # Why this card shows two rows and not one
 *
 * Statewide over these three years, **total state support rises $145.0M and foundation aid falls
 * $114.5M**. Both are true of the same 609 districts in the same years. A card that drew one line
 * and called it funding would have told the reader that Ohio's school funding went up, or that it
 * went down, on a choice nobody wrote down — so both are drawn, labelled, and never summed.
 *
 * The narrow measure is not a lesser version of the wide one. It is `core foundation funding +
 * guarantee`: the part `crates/project` models, the part every lever on the scenario page moves,
 * and the only part it can project past the model year. Everything the budget actually changed for the
 * districts this page gets asked about — transportation and the supplements — is outside it.
 *
 * # Why the years stop
 *
 * All three are observed department files. The feed carries no projected year here and the module
 * refuses to produce one, because a forecast exists for the narrow measure only and appending it
 * would extend a wide series with a narrow number. See `crates/project/src/biennium.rs`.
 */
import { money, signedMoney } from "./format.ts";
import * as routes from "./routes.ts";
import { anchor } from "./section.ts";
import type { District } from "./types.ts";
import { yearChip } from "./year.ts";

/** The five payment lines, in the order the card lists them. */
function lines(baseline: number): ReadonlyArray<{
  readonly label: string;
  readonly pick: (b: District["biennium"]) => number;
  readonly note: string;
}> {
  return [
    {
      label: "Foundation aid",
      pick: (b) => b.line_foundation,
      note: "Core foundation funding and the guarantee together. Zero for a district held at its base in every year.",
    },
    {
      label: "Transportation",
      pick: (b) => b.line_transportation,
      note: "Outside the foundation formula, and not moved by any lever on the scenario page.",
    },
    {
      label: "Special education transportation",
      pick: (b) => b.line_special_education_transportation,
      note: "Prorated against its appropriation, so it moves for reasons a district does not control.",
    },
    {
      label: "Preschool special education",
      pick: (b) => b.line_preschool_special_education,
      note: "Also prorated, and also outside the formula.",
    },
    {
      label: "Supplements",
      pick: (b) => b.line_supplements,
      note: `The formula transition supplement in both years, supplemental targeted assistance in FY${baseline} which H.B. 96 repealed, and the base funding and enrollment growth supplements it created.`,
    },
  ];
}

/**
 * The biennium card.
 *
 * Returns the empty string for a district the payment files do not carry, which is none of the
 * 609 the feed ships — the guard is for a future panel that adds one.
 */
export function renderBiennium(d: District): string {
  const b = d.biennium;
  if (b.total_baseline <= 0) return "";

  // From the feed, never written here: `year_*` are the model's own fiscal years.
  const years: [number, number, number] = [
    b.year_baseline,
    b.year_middle,
    b.year_terminal,
  ];
  const wide = b.total_terminal - b.total_baseline;
  const narrow = b.foundation_terminal - b.foundation_baseline;
  const measures: ReadonlyArray<[string, number, number, number, number]> = [
    [
      "Total state support",
      b.total_baseline,
      b.total_middle,
      b.total_terminal,
      wide,
    ],
    [
      "Foundation aid only",
      b.foundation_baseline,
      b.foundation_middle,
      b.foundation_terminal,
      narrow,
    ],
  ];

  return `
    <div class="card" id="biennium" data-part="biennium">
      <h2>${anchor("biennium")}Across the biennium${yearChip("biennium")}</h2>
      <p class="note">Two measures of the same district in the same years, because they do not
        agree and neither one is <em>the</em> figure. <strong>Total state support</strong> is what
        the district is paid before transfers. <strong>Foundation aid</strong> is base cost, the
        six categoricals and the guarantee — the part this site models, and the only part the
        <a href="${routes.districtScenario(d.irn)}">scenario page</a> can move or project.</p>
      <div class="scroll">
        <table>
          <thead>
            <tr><th scope="col">Measure</th>
              ${years.map((year) => `<th scope="col" class="tnum">FY${year}</th>`).join("")}
              <th scope="col" class="tnum">Change</th></tr>
          </thead>
          <tbody>
            ${measures
              .map(
                ([label, a, m, t, change]) => `
            <tr><th scope="row">${label}</th>
              <td class="tnum">${money(a)}</td>
              <td class="tnum">${money(m)}</td>
              <td class="tnum">${money(t)}</td>
              <td class="tnum">${signedMoney(change)}</td></tr>`,
              )
              .join("")}
          </tbody>
        </table>
      </div>
      <p class="note">The change column is the last year against the first. A two-year budget pays
        both of its years, so the biennium's whole departure from the year it is measured from is
        <strong>${signedMoney(b.total_middle - b.total_baseline + wide)}</strong> on total state
        support — each year counted against FY${years[0]}, not against the year before it.</p>
      <h3>What the change is made of</h3>
      <p class="note">The five lines below sum to the total state support change exactly. The
        first is the whole of what this site's model computes; the other four are not modelled
        here at all.</p>
      <div class="scroll">
        <table>
          <thead>
            <tr><th scope="col">Line</th>
              <th scope="col" class="tnum">FY${years[0]} to FY${years[2]}</th>
              <th scope="col">What it is</th></tr>
          </thead>
          <tbody>
            ${lines(years[0])
              .map(
                (line) => `
            <tr><th scope="row">${line.label}</th>
              <td class="tnum">${signedMoney(line.pick(b))}</td>
              <td class="note">${line.note}</td></tr>`,
              )
              .join("")}
          </tbody>
        </table>
      </div>
      ${
        Math.abs(narrow) < 0.005
          ? `<p class="note"><strong>This district's foundation aid did not move at all.</strong>
             It was held at its funding base in every one of the three years, so whatever the
             formula computed for it, the guarantee paid the difference and the total was the same
             number. Every lever on the scenario page acts on a figure that, for this district, is
             fixed — which is why a base cost increase can cost the state a great deal and pay this
             district nothing.</p>`
          : ""
      }
    </div>`;
}
