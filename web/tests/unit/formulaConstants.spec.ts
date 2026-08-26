/**
 * Every constant in a formula is written down in a parameter node, or licensed with a reason.
 *
 * # What this closes
 *
 * The `parameter` class is where a counterfactual binds: `.yidam/corpus/parameter/README.md` says
 * the component describes how the calculation runs and the parameter says what number it runs on,
 * "and only the second is what a proposal changes". For four years the class held six nodes while
 * **33 distinct statutory rates sat as bare literals inside the components' own `function` fields**
 * — every special education multiple, every career-technical multiple, the transportation rates,
 * the DPIA amount. Exactly one of the 33, the statewide average base cost per pupil, had a node.
 *
 * A constant with no node cannot be perturbed, cannot carry a `kind`, cannot carry a series, and
 * cannot say what would have to happen for it to change. It reads as arithmetic when it is policy.
 * #204 named the gap and made growing this class the precondition for linking a formula's terms to
 * the nodes that define them.
 *
 * # What is checked, and why it is checked against the TeX
 *
 * `function_tex` is the statement of the formula the site actually renders, so a literal there is a
 * number a reader meets. For each component, every literal in it must appear somewhere in the text
 * of a `parameter` node the component declares a `governed-by` edge to.
 *
 * That is a weaker claim than "this literal is that parameter's value" and it is the strongest one
 * available: the TeX writes `0.2435` and the node writes it inside a table of six, and no parse
 * relates them. What it does catch is the failure that actually happened — a number in a formula
 * with no node behind it anywhere.
 *
 * Small whole numbers are excluded. `\sum_{k=1}^{6}`, a `\max(\ldots, 6)`, a five-day substitute
 * allowance and a grade band are structure rather than policy, and there is no way to tell them
 * from a policy constant that happens to be small. Twelve is the cut: it admits every staffing
 * divisor in R.C. 3317.011 and excludes the indices. A policy constant below twelve — the 5% the
 * plan was enacted with, say — is written `0.05` and is caught.
 *
 * # It found four things on its first run, and two were missing edges
 *
 * `fsfp-preschool-special-education` restates the six special education multiples in its TeX and
 * declared no edge to them, because R.C. 3317.0213 names each division of R.C. 3317.013 rather
 * than repeating the numbers — so the corpus had the relationship in prose and not in the graph.
 * `guarantee-open-enrolment-clawback` charges $8,241.61 per FTE and declared no `governed-by` at
 * all. Both are edges now. The other two are the exemptions below.
 */

import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { expect, test } from "vitest";

const CORPUS = join(import.meta.dirname, "../../../.yidam/corpus");

/** Below this, a whole number in a formula is structure — an index, a bound, a count of terms. */
const STRUCTURAL_MAX = 12;

/** A licence for a literal with no parameter node behind it. */
interface Allowance {
  /** Why there is no node, and what it would take to write one. */
  reason: string;
  /** Exactly the literals the reason argues for, as the TeX writes them. */
  literals: string[];
}

/**
 * Constants that are not in any parameter node, keyed by the component that states them.
 *
 * The reason is the point, and each of these names the work that would close it. An entry reading
 * "it is a number in a spreadsheet" is not a licence, it is an unwritten node.
 */
const UNNODED: Record<string, Allowance> = {
  "fsfp-base-cost-calculation": {
    literals: ["18", "20", "23", "25", "90", "150", "180"],
    reason:
      "R.C. 3317.011's build-up constants: staffing divisors of 20, 23, 25, 27 and 18 for kindergarten, grades 1-3, 4-8, 9-12 and career-technical, one special teacher per 150, a $90 substitute daily rate and a 180-day contract year. Seven are listed here and the eighth, 27, is not — a node this component declares happens to state 27 somewhere else, so the check is satisfied by coincidence. That is the known looseness of a value-level match and it is why this list exists rather than the check standing alone. The section also carries a 1.16 benefit multiplier, salary caps at $160,000, $130,000, $80,000 and $60,000, and ADM thresholds at 4,000 and 500. Writing them as one `base-cost-build-up` parameter is the largest single piece of the class still missing, and it is a section-length reading rather than a transcription — the caps interpolate between district sizes and the section applies only to FY2026-27, so the node has to state a schedule and an expiry. Deferred deliberately: `base-cost-per-pupil` already carries the build-up's *result* and its sensitivity, so the gap is the inputs' provenance rather than the figure a reader meets.",
  },
  "fsfp-targeted-assistance": {
    literals: ["1.6", "0.88"],
    reason:
      "The eligibility test for targeted assistance's *supplemental* tier — a FY2019 wealth index above 1.6 and FY2019 enrolled ADM below 88% of the district's total. The tier pays zero to every district in the FY2027 model, so this is a dormant provision: a parameter node for a threshold that currently selects nobody would state a value with no consequence and no series. It becomes a node the year the tier pays anyone.",
  },
  "guarantee-open-enrolment-clawback": {
    literals: ["20"],
    reason:
      "The clawback's own trigger: a district's guarantee is reduced only where its entering open-enrolment FTE falls by more than the greater of 10% and 20 FTE. Only the 20 is listed. The 0.1 is not, because `base-cost-per-pupil` — which this component declares, for the $8,241.61 the clawback charges per FTE — states 0.1 as the minimum state share, and a value-level check cannot tell one tenth from another. The collision is recorded here rather than worked around: a licence may not cover a literal the check does not report. R.C. 3317.019 supplies the mechanism and the department's FY2027 workbook supplies the rate, so the pair is part legislated and part delegated in the way `dpia-per-pupil-amount` is — which is the argument for a node rather than against one. Left unwritten here because the threshold is read by one component and by nothing else, and the phase that writes it should write the clawback's rate beside it from the enrolled act rather than from the workbook.",
  },
};

/** Every literal a formula states that is not structure. */
function literals(tex: string): string[] {
  const plain = tex.replaceAll("\\$", "$").replaceAll("{,}", "").replaceAll(",", "");
  const found = new Set<string>();
  for (const match of plain.matchAll(/(?<![\w.])\d+(?:\.\d+)?(?![\w])/g)) {
    const text = match[0];
    const value = Number(text);
    if (Number.isInteger(value) && value <= STRUCTURAL_MAX) continue;
    found.add(text);
  }
  return [...found].sort((a, b) => Number(a) - Number(b));
}

/**
 * Every number a node's prose states, as values rather than as text.
 *
 * Numeric and not a substring match, because both directions of a text match are wrong here.
 * A substring is too loose: `0.1` occurs inside `0.10`, `0.15` and `20.1`, and
 * `base-cost-per-pupil` contains all three, so a bare `includes` let the open-enrolment clawback's
 * 10% threshold count as covered by a node that says nothing about it. And an exact string match
 * is too tight: the local capacity blend is `0.6` in the TeX and `0.60` in the node, the
 * preschool halving is `0.5` against `0.50`, and neither difference means anything.
 *
 * A percentage contributes both readings. `50%` in prose covers `0.5` in a formula and `50` in
 * another one, because the corpus writes rates both ways and a check that forced one notation
 * would be enforcing a house style rather than finding a missing node.
 */
function stated(prose: string): Set<number> {
  const values = new Set<number>();
  for (const match of prose.matchAll(/(?<![\w.])(\d+(?:\.\d+)?)(%?)/g)) {
    const value = Number(match[1]);
    values.add(value);
    if (match[2] === "%") values.add(value / 100);
  }
  return values;
}

function block(source: string, field: string): string | null {
  const match = new RegExp(`^  ${field}: \\|\\n((?:    .*\\n|\\n)+)`, "m").exec(source);
  return match?.[1] ?? null;
}

test("every constant in a formula is written down in a parameter node, or licensed with a reason", () => {
  const parameters = new Map(
    readdirSync(join(CORPUS, "parameter"))
      .filter((name) => name.endsWith(".yml"))
      .map((name) => [
        name.slice(0, -".yml".length),
        readFileSync(join(CORPUS, "parameter", name), "utf8").replaceAll(",", ""),
      ]),
  );

  const unnoded: string[] = [];
  const used = new Set<string>();

  for (const name of readdirSync(join(CORPUS, "formula-component"))) {
    if (!name.endsWith(".yml")) continue;
    const component = name.slice(0, -".yml".length);
    const source = readFileSync(join(CORPUS, "formula-component", name), "utf8");
    const tex = block(source, "function_tex");
    if (tex === null) continue;

    const governs = [
      ...source.matchAll(/- target: \.\.\/parameter\/([a-z0-9-]+)\.yml\n\s+relationship: governed-by/g),
    ].map(([, target]) => target!);
    const known = stated(governs.map((target) => parameters.get(target) ?? "").join("\n"));
    const allowed = UNNODED[component];

    for (const literal of literals(tex)) {
      if (known.has(Number(literal))) continue;
      if (allowed?.literals.includes(literal)) {
        used.add(`${component}: ${literal}`);
        continue;
      }
      unnoded.push(
        `${component}: ${literal} — stated in the formula, in none of the ${governs.length} ` +
          `parameter nodes it declares (${governs.join(", ") || "none"})`,
      );
    }
  }

  expect(
    unnoded,
    "a constant a formula states with no parameter node behind it: write the node and declare a `governed-by` edge, or license it here with what it would take",
  ).toEqual([]);

  /* Both directions, the shape `yearLiterals.spec.ts` uses: a licence covering nothing is one
     waiting to cover the next constant somebody adds. */
  const unused = Object.entries(UNNODED).flatMap(([component, allowance]) =>
    allowance.literals
      .filter((literal) => !used.has(`${component}: ${literal}`))
      .map((literal) => `${component}: ${literal}`),
  );
  expect(unused, "a licence for a constant the formula no longer states — delete it").toEqual([]);
});

test("every formula component declares the parameters it runs on", () => {
  /*
   * The coarse half, and it is not redundant with the check above: a component whose TeX happens to
   * state no literal above twelve would pass that one while declaring no parameter at all, which is
   * the state `guarantee-open-enrolment-clawback` was in.
   *
   * `temporary-transitional-aid-guarantee` is the shape this catches. Its formula is a `max` over
   * two quantities with no constant in it, and it runs on the FY2020 funding base — which is a
   * parameter, and one of the four the scenario runner can move.
   */
  const undeclared: string[] = [];
  for (const name of readdirSync(join(CORPUS, "formula-component"))) {
    if (!name.endsWith(".yml")) continue;
    const source = readFileSync(join(CORPUS, "formula-component", name), "utf8");
    if (block(source, "function_tex") === null) continue;
    if (!/relationship: governed-by/.test(source)) undeclared.push(name.slice(0, -".yml".length));
  }
  expect(
    undeclared,
    "a formula component that names no parameter: a calculation with no dial is not one a proposal can change",
  ).toEqual([]);
});
