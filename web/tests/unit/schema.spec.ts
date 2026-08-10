/**
 * The schemas, and the defects they exist to catch.
 *
 * Every case below is a mistake this repository actually made. They are written as tests rather
 * than as comments because the schema is only worth having if it fails on the real thing, and
 * "would this have been caught?" is answerable.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { z } from "astro/zod";
import { expect, test } from "vitest";
import YAML from "yaml";

import { loadCorpus, resolveTarget } from "../../src/lib/corpus.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import {
  NodeSchema,
  OBSERVATION_PROPERTY,
  OntologyClassSchema,
} from "../../src/lib/schema/corpus.ts";
import { BundleSchema } from "../../src/lib/schema/feed.ts";

const raw: unknown = JSON.parse(
  readFileSync(join(import.meta.dirname, "../../public/data/bundle.json"), "utf8"),
);

test("the committed feed matches the shape this site reads", () => {
  const parsed = BundleSchema.safeParse(raw);
  const problems = parsed.success
    ? []
    : parsed.error.issues.slice(0, 5).map((i) => `${i.path.join(".")}: ${i.message}`);
  expect(problems).toEqual([]);
});

test("a field the Rust adds and this mirror does not know about fails the build", () => {
  // The whole point of `.strict()`. `crates/bundle` is authoritative and this file is a
  // hand-maintained mirror of it; the failure that matters is the two drifting apart, and an
  // unexpected key is the earliest possible signal of exactly that.
  const feed = structuredClone(raw) as Record<string, unknown>;
  (feed.districts as Record<string, unknown>[])[0]!.new_rust_field = 1;
  expect(BundleSchema.safeParse(feed).success).toBe(false);
});

test("a field the Rust renames fails rather than rendering as an em dash", () => {
  // This is the failure a cast could not see. `realized_aid_per_pupil` going missing produced
  // `undefined`, which `money()` formats as "—" — indistinguishable from "not reported".
  const feed = structuredClone(raw) as Record<string, unknown>;
  const district = (feed.districts as Record<string, unknown>[])[0]!;
  district.realised_aid_per_pupil = district.realized_aid_per_pupil;
  delete district.realized_aid_per_pupil;
  expect(BundleSchema.safeParse(feed).success).toBe(false);
});

test("an IRN that is not six digits fails", () => {
  // Every route on this site is keyed on it, and 28 district names are not unique, so the IRN is
  // the only thing standing between two districts and the same page.
  const feed = structuredClone(raw) as Record<string, unknown>;
  (feed.districts as Record<string, unknown>[])[0]!.irn = "43786";
  expect(BundleSchema.safeParse(feed).success).toBe(false);
});

test("an enrollment history of the wrong length fails", () => {
  // `adm_history` is a tuple of three. A district with a short history cannot be projected, and a
  // page that quietly dropped it would report a statewide total over a subset of the panel.
  const feed = structuredClone(raw) as Record<string, unknown>;
  (feed.districts as Record<string, unknown>[])[0]!.adm_history = [1, 2];
  expect(BundleSchema.safeParse(feed).success).toBe(false);
});

test("the corpus validates with no errors", () => {
  const errors = loadCorpus().diagnostics.filter((d) => d.severity === "error");
  expect(errors.map((d) => `${d.file}: ${d.message}`)).toEqual([]);
});

test("links written as a paragraph are rejected", () => {
  // The defect that motivated all of this. `scenario/guarantee-phase-out` wrote its entire link
  // list as prose; it was valid YAML, so nothing complained, and its fifteen edges were invisible
  // to every consumer for as long as the node existed.
  const proseLinks = {
    class: "scenario",
    label: "Something",
    description: "A node.",
    properties: {},
    links: "Perturbs [a](../parameter/a.yml) and baselines on [b](../funding-regime/b.yml).",
  };
  const result = NodeSchema.safeParse(proseLinks);
  expect(result.success).toBe(false);
  if (!result.success) {
    expect(result.error.issues.some((i) => i.path.includes("links"))).toBe(true);
  }
});

test("a node with no outgoing links is rejected", () => {
  // The corpus's own rule: a node nothing reaches and that reaches nothing is a gap, not a fact.
  const orphan = { class: "metric", label: "X", description: "Y", properties: {}, links: [] };
  expect(NodeSchema.safeParse(orphan).success).toBe(false);
});

test("a property value that is not a string is rejected", () => {
  // `irn: 044933` unquoted would parse as a number in a schema that allowed one, and lose its
  // leading zero — silently turning a valid IRN into an invalid one.
  const node = {
    class: "education-agency",
    label: "X",
    description: "Y",
    properties: { irn: 44933 },
    links: [{ target: "../education-agency.ont.yml", relationship: "instance-of" }],
  };
  expect(NodeSchema.safeParse(node).success).toBe(false);
});

test("every corpus file on disk parses and validates individually", () => {
  // Belt and braces against `loadCorpus` becoming tolerant of something it should not be.
  const corpus = loadCorpus();
  const failures: string[] = [];
  for (const node of corpus.nodes) {
    const file = join(import.meta.dirname, "../../../.yidam/corpus", `${node.id}.yml`);
    const parsed = NodeSchema.safeParse(YAML.parse(readFileSync(file, "utf8")));
    if (!parsed.success) failures.push(`${node.id}: ${parsed.error.issues[0]?.message}`);
  }
  for (const entry of corpus.classes) {
    const file = join(
      import.meta.dirname,
      "../../../.yidam/corpus",
      `${entry.className}.ont.yml`,
    );
    const parsed = OntologyClassSchema.safeParse(YAML.parse(readFileSync(file, "utf8")));
    if (!parsed.success) failures.push(`${entry.className}.ont: ${parsed.error.issues[0]?.message}`);
  }
  expect(failures).toEqual([]);
});

test("every class states whether its edge vocabulary is open", () => {
  // Required with no default, so a class nobody has decided about cannot pass silently. All of
  // them currently say `characteristic`, which is what the corpus has always done and never said.
  const corpus = loadCorpus();
  for (const entry of corpus.classes) {
    expect(["characteristic", "exhaustive"]).toContain(entry.edgePolicy);
  }
  // A deliberate tripwire rather than a derived count: adding a class changes what the corpus is
  // about, and this is the assertion that makes it impossible to do quietly. Thirteen at genesis;
  // fifteen when `accountability-regime` and `intervention` were admitted, sixteen since `school`
  // joined them — all three by decision `accountability-domain`. Update it only alongside a
  // decision record.
  expect(corpus.classes).toHaveLength(16);
});

test("an ontology missing edge_policy is rejected", () => {
  const withoutPolicy = {
    class: "metric",
    label: "Metric",
    description: "A measure.",
    properties: [],
    edges: [],
  };
  expect(OntologyClassSchema.safeParse(withoutPolicy).success).toBe(false);
  expect(
    OntologyClassSchema.safeParse({ ...withoutPolicy, edge_policy: "characteristic" }).success,
  ).toBe(true);
  // And it is an enum, so a typo is not quietly treated as "open".
  expect(OntologyClassSchema.safeParse({ ...withoutPolicy, edge_policy: "open" }).success).toBe(
    false,
  );
});

test("the corpus is clean under the policy it declares", () => {
  // The point of stating the policy: what is left is signal. 46 undeclared relationships were
  // noise against an unstated assumption; the four undeclared properties were real omissions and
  // are now declared. Zero of either should remain.
  const diagnostics = loadCorpus().diagnostics;
  expect(diagnostics.map((d) => `${d.severity} ${d.file}: ${d.message}`)).toEqual([]);
});

test("a year-stamped observation is allowed where a bare property name would not be", () => {
  // `fy2024_profile` recurs across agency nodes and is a snapshot rather than a schema property.
  // Declaring it would mean editing an ontology every fiscal year to permit next year's.
  expect(OBSERVATION_PROPERTY.test("fy2024_profile")).toBe(true);
  expect(OBSERVATION_PROPERTY.test("fy2021_idea_part_b")).toBe(true);
  expect(OBSERVATION_PROPERTY.test("fy2026_27_appropriation")).toBe(true);
  // Not a licence for anything unrecognised.
  expect(OBSERVATION_PROPERTY.test("profile")).toBe(false);
  expect(OBSERVATION_PROPERTY.test("fy_profile")).toBe(false);
  expect(OBSERVATION_PROPERTY.test("notes")).toBe(false);
});

test("a bare class name resolves to that class's page", () => {
  // Two metric nodes write `target:` in the ontology's vocabulary — a class name, not a path.
  // Unhandled it emitted `<a href="education-agency">`, a relative link that 404s while looking
  // entirely normal in the markup.
  const resolved = resolveTarget("education-agency", "metric");
  expect(resolved.resolved).toBe(true);
  expect(resolved.href).toBe("/wiki/education-agency");
});

test("a target shape nothing recognises is reported rather than emitted", () => {
  const resolved = resolveTarget("../../somewhere/odd.txt", "metric");
  expect(resolved.resolved).toBe(false);
});

test("the committed JSON Schemas match the zod definitions", () => {
  // Generated and committed, so the editor can read them without a build. Which means they can go
  // stale, so they are checked like the feed is.
  for (const [file, schema] of [
    ["corpus-node.json", NodeSchema],
    ["corpus-ontology.json", OntologyClassSchema],
  ] as const) {
    // The prose fields are added by the emitter, not by zod, so they are stripped from both sides
    // rather than only from the committed copy.
    const strip = (o: Record<string, unknown>) => {
      const { $schema: _s, title: _t, description: _d, ...body } = o;
      return body;
    };
    const committed = JSON.parse(
      readFileSync(join(import.meta.dirname, "../../../.yidam/schemas", file), "utf8"),
    ) as Record<string, unknown>;
    expect(strip(committed), `${file} is stale — run \`pnpm schemas\``).toEqual(
      strip(z.toJSONSchema(schema, { io: "input" }) as Record<string, unknown>),
    );
  }
});

test("the feed the site builds from is the one the schema accepted", () => {
  // `loadFeed` parses rather than casts, so this is really asserting the wiring: what pages get is
  // schema output, not a cast over whatever was on disk.
  const { bundle } = loadFeed();
  expect(bundle.districts).toHaveLength(609);
  expect(BundleSchema.safeParse(bundle).success).toBe(true);
});

test("the base cost build-up reproduces the department's aggregate for every district", () => {
  /*
   * The claim the card makes, checked against the whole panel rather than the one district a
   * screenshot would show. These are the only per-district figures on this site that are computed
   * instead of read, so the reproduction is the licence to display them at all.
   *
   * Two dollars, not two cents: twenty-two elements each rounded where the department rounds,
   * summed. The worst district is off by $1.09 on an aggregate of $257 million. Anything larger
   * would mean the implementation and the statute had diverged rather than the arithmetic drifting.
   */
  const { bundle } = loadFeed();
  let worst = { irn: "", name: "", residual: 0 };
  let computedTotal = 0;
  let publishedTotal = 0;

  for (const d of bundle.districts) {
    const b = d.base_cost_build_up;
    const parts =
      b.teachers + b.student_support + b.district_leadership + b.building_leadership +
      b.athletic_cocurricular;
    // The five sub-components have to add to the aggregate the card prints, or the table shows
    // a total that is not the sum of the rows above it.
    expect(Math.abs(parts - b.computed_aggregate), `${d.name} parts do not sum`).toBeLessThan(0.01);
    expect(Math.abs(b.residual - (b.computed_aggregate - b.published_aggregate))).toBeLessThan(1e-6);
    expect(b.published_aggregate).toBeCloseTo(d.aggregate_base_cost, 2);

    computedTotal += b.computed_aggregate;
    publishedTotal += b.published_aggregate;
    if (Math.abs(b.residual) > Math.abs(worst.residual)) {
      worst = { irn: d.irn, name: d.name, residual: b.residual };
    }
  }

  expect(
    Math.abs(worst.residual),
    `worst residual $${worst.residual.toFixed(2)} at ${worst.name} (${worst.irn})`,
  ).toBeLessThan(2);
  // And the residuals are noise, not a bias: they cancel across the state.
  expect(Math.abs(computedTotal - publishedTotal) / publishedTotal).toBeLessThan(1e-7);
});

test("the scenario panel does not carry the build-up", () => {
  // Twenty-nine numbers per district that `policy.ts` never reads. Leaving them in would add half
  // a megabyte to the one download that happens in a browser.
  const { bundle } = loadFeed();
  const district = bundle.districts[0]!;
  expect(district).toHaveProperty("base_cost_build_up");
  const { finances: _f, outcome: _o, base_cost_build_up: _b, ...panelDistrict } = district;
  expect(Object.keys(panelDistrict)).not.toContain("base_cost_build_up");
});

test("all four tax years are present, ordered, and their classes partition the base", () => {
  // Ordered because the page reads the series as a change; reversed, every direction it reports
  // would invert silently. Partitioned because a base whose parts do not sum to its whole invites
  // shares that do not add to one.
  //
  // Four years, not two. The window exists so that recognized valuation can be reconstructed:
  // every Ohio county reappraises or updates once in TY2022-TY2024, and TY2021 makes the earliest
  // of those a change rather than a level. Anything reading this as a pair must take the LAST
  // two — see the millage tests below, which is where taking the ends went wrong.
  const { bundle } = loadFeed();
  for (const d of bundle.districts) {
    expect(d.property_tax.map((y) => y.tax_year), d.name).toEqual([2021, 2022, 2023, 2024]);
    for (const y of d.property_tax) {
      expect(Math.abs(y.class1_value + y.class2_value + y.public_utility_value - y.total_value))
        .toBeLessThan(1);
      expect(Math.abs(y.agricultural_value + y.residential_value - y.class1_value)).toBeLessThan(1);
    }
  }
});

test("SD-1's effective rate agrees with the profile report's", () => {
  // Two departments describing the same district. They are not obliged to agree, and the taxes
  // page tells readers they do — so it is checked rather than asserted.
  const { bundle } = loadFeed();
  let compared = 0;
  for (const d of bundle.districts) {
    const ty2023 = d.property_tax.find((y) => y.tax_year === 2023);
    if (!ty2023 || d.effective_class1_millage == null) continue;
    compared++;
    expect(Math.abs(ty2023.class1_rate - d.effective_class1_millage), d.name).toBeLessThan(0.01);
  }
  expect(compared).toBe(606);
});

test("the department's two spending roll-ups partition operating expenditure", () => {
  const { bundle } = loadFeed();
  let covered = 0;
  for (const d of bundle.districts) {
    const s = d.spending_by_function;
    if (!s) continue;
    covered++;
    // A cent, not nothing: the department publishes each roll-up rounded to cents independently,
    // so the pair can land a cent either side of the total it partitions. 92 districts do. The
    // Rust suite allows the same slack for the same reason.
    expect(Math.abs(s.classroom_instruction + s.nonclassroom - s.operating_per_pupil), d.name)
      .toBeLessThan(0.011);
  }
  // Two districts have no report-card spending row, and the page says so rather than showing zero.
  expect(covered).toBe(607);
});

test("the statewide tax facts the pages print are derived, and reproduce the Rust suite", () => {
  /*
   * These appear on 609 pages each. Hard-coding them is how a regenerated feed leaves a page
   * quietly wrong — which nearly happened here: the first draft of the copy said two districts are
   * charged more than they spend, and the answer is three.
   *
   * The median and the fiftieth-ranked district are the figures Policy Matters Ohio published and
   * `crates/dispersion/tests/sd1_district_taxes.rs` reproduces; matching them from the feed means
   * the web layer and the Rust agree about the same quantity.
   */
  const { tax } = loadFeed();
  expect(tax.medianChargeShare).toBeCloseTo(0.36, 2);
  expect(tax.chargedMoreThanSpent.map((d) => d.name)).toEqual([
    "Danbury Local",
    "Put-In-Bay Local",
    "Mayfield City",
  ]);

  // The floor mechanism, as a count. Reductions concentrate above the floor because that is the
  // only place reduction factors can still operate.
  //
  // Stated as a pair of rates rather than a ratio of counts. The two groups are not the same size
  // and their sizes move whenever the classification is refined, so a bare "one count exceeds the
  // other times fifty" quietly encodes the group sizes it was written against — which is how this
  // assertion came to fail on a change that made the underlying contrast stronger, not weaker.
  expect(tax.districts.atFloor + tax.districts.aboveFloor).toBe(609);
  const fellAbove = tax.rateFell.aboveFloor / tax.districts.aboveFloor;
  const fellAt = tax.rateFell.atFloor / tax.districts.atFloor;
  expect(fellAbove).toBeGreaterThan(0.6);
  expect(fellAt).toBeLessThan(0.06);
  expect(fellAbove).toBeGreaterThan(fellAt * 10);
  expect(tax.reductionsAboveFloor).toBeGreaterThan(0.95);
});

test("the floor split is taken at the start of the interval, not the end", () => {
  /*
   * The category must not be decided by the outcome it is used to measure.
   *
   * `District.at_millage_floor` reports where a district stands in the latest tax year, which is
   * what its own page should say. Using it to split "whose rate fell between the two years" lets
   * a district that fell *to* the floor be counted as an at-floor district with a falling rate —
   * and 29 districts did exactly that. The contamination is large enough to matter: it moves the
   * at-floor fall rate from 3.6% to 20.5% and drops the headline from 97.7% to 88.3%.
   */
  const { bundle, tax } = loadFeed();

  const contaminated = { atFloor: 0, aboveFloor: 0 };
  for (const d of bundle.districts) {
    // The last two, not the ends: the H.B. 920 recursion is only exact between consecutive years.
    const [before, after] = d.property_tax.slice(-2);
    if (!before || !after || d.property_tax.length < 2) continue;
    if (after.class1_rate - before.class1_rate >= -0.0005) continue;
    if (d.at_millage_floor) contaminated.atFloor++;
    else contaminated.aboveFloor++;
  }

  expect(contaminated.atFloor).toBeGreaterThan(tax.rateFell.atFloor * 4);
  expect(tax.rateFell.atFloor).toBeLessThan(10);
  expect(tax.districts.atFloor).toBeGreaterThan(bundle.statewide.at_millage_floor);
});

test("neither new block reaches the scenario panel", () => {
  const { bundle } = loadFeed();
  const district = bundle.districts[0]!;
  expect(district).toHaveProperty("property_tax");
  expect(district).toHaveProperty("spending_by_function");
  expect(district).toHaveProperty("millage");
  const {
    finances: _f,
    outcome: _o,
    base_cost_build_up: _b,
    property_tax: _p,
    spending_by_function: _s,
    millage: _m,
    ...panelDistrict
  } = district;
  for (const dropped of ["property_tax", "spending_by_function", "millage"]) {
    expect(Object.keys(panelDistrict)).not.toContain(dropped);
  }
});

test("the millage block is the crate's output, not a copy of a published column", () => {
  /*
   * Three properties that hold only if the numbers were computed. A block quietly reverted to
   * echoing Table SD-1 would still typecheck and still render; each of these would break.
   */
  const { bundle } = loadFeed();
  const blocks = bundle.districts.filter((d) => d.millage != null);
  expect(blocks.length).toBe(609);

  for (const d of blocks) {
    const m = d.millage!;
    const [before, after] = d.property_tax.slice(-2) as [
      (typeof d.property_tax)[number],
      (typeof d.property_tax)[number],
    ];

    // It reads the tax years it claims to.
    expect(m.tax_year).toBe(after.tax_year);
    expect(m.prior_rate).toBeCloseTo(before.class1_rate, 6);
    expect(m.observed_rate).toBeCloseTo(after.class1_rate, 6);

    // The residual is a derived quantity, so it has to close.
    expect(m.observed_rate - m.predicted_rate).toBeCloseTo(m.residual, 6);

    // The prediction obeys the statute: never above the prior rate, never below the floor unless
    // the prior rate already was.
    expect(m.predicted_rate).toBeLessThanOrEqual(m.prior_rate + 1e-9);
    if (m.prior_rate > 20) expect(m.predicted_rate).toBeGreaterThanOrEqual(20 - 1e-9);
  }
});

test("the two departments publish the same valuation over different pupil counts", () => {
  /*
   * The finding, as an invariant. Multiply the District Profile Report's valuation per pupil by
   * its enrolled ADM and Table SD-1's total taxable value comes back exactly — the numerators are
   * identical. The denominators are not, and the gap is charter, voucher and open-enrolment-out
   * students, so it is widest in the big-city districts where valuation per pupil does the most
   * work in the aid formula.
   *
   * The test exists because the page prints one of the two figures and it must not be positioned
   * against the other's statewide median, which is what it did until this was found.
   */
  const { bundle } = loadFeed();

  let compared = 0;
  let diverged = 0;
  for (const d of bundle.districts) {
    const ty2023 = d.property_tax.find((y) => y.tax_year === 2023);
    // Enrolled ADM FY2024, which is what the profile's valuation per pupil divides by — not
    // `d.adm`, the funded base cost count, which is a third number and reproduces nothing.
    const enrolled = d.adm_history[0];
    if (!ty2023 || d.valuation_per_pupil == null || enrolled <= 0) continue;
    compared++;

    // Same numerator, to a tenth of a percent — the profile's per-pupil times the profile's ADM.
    const fromProfile = d.valuation_per_pupil * enrolled;
    expect(Math.abs(fromProfile - ty2023.total_value) / ty2023.total_value).toBeLessThan(0.001);

    if (Math.abs(ty2023.adm / enrolled - 1) > 0.05) diverged++;
  }

  expect(compared).toBeGreaterThan(590);
  expect(diverged).toBeGreaterThan(200);

  // Columbus is the case that makes it concrete: same dollars, 1.7x the pupils.
  const columbus = bundle.districts.find((d) => d.irn === "043802")!;
  const sd1 = columbus.property_tax[columbus.property_tax.length - 1]!;
  expect(sd1.adm / columbus.adm_history[0]).toBeGreaterThan(1.6);
  expect(sd1.value_per_pupil).toBeLessThan(columbus.valuation_per_pupil! * 0.7);
});

test("the charge-off counterfactual reproduces what the regime-diff crate documents", () => {
  /*
   * Three figures the crate's own doc comments state, recomputed from the feed. If the bundle
   * wired the crate up wrongly — passed the wrong valuation, or the wrong rate — these move.
   */
  const { bundle } = loadFeed();
  const regimes = bundle.districts.map((d) => d.regime).filter((r) => r != null);
  expect(regimes.length).toBe(609);

  // Every one runs at the terminal rate, not one of the earlier ones in the series.
  for (const r of regimes) expect(r.charge_off_mills).toBe(23);

  // "65 districts are in that position at 23 mills against FY2027 base cost." It was 81 while the
  // counterfactual ran on total taxable value; the charge-off is smaller on recognized valuation,
  // so it runs past base cost in sixteen fewer places.
  expect(regimes.filter((r) => r.exceeds_base_cost).length).toBe(65);

  // "exactly zero for 465 of the 470 districts where both sides can be valued."
  const valued = regimes.filter((r) => r.residual != null);
  expect(valued.length).toBe(470);
  expect(valued.filter((r) => Math.abs(r.residual!) < 0.005).length).toBe(465);

  // The base is recognized valuation for every district, and the phase-in is a real split rather
  // than a field that is always one: three cohorts, on the county reappraisal calendar.
  const years = new Set(regimes.map((r) => r.reappraisal_year));
  expect([...years].sort()).toEqual([2022, 2023, 2024]);
  expect(regimes.filter((r) => r.recognized_share < 0.9995).length).toBeGreaterThan(400);
  for (const r of regimes) {
    expect(r.recognized_share).toBeLessThanOrEqual(1);
    expect(r.recognized_share).toBeGreaterThan(0.5);
    // Nothing is deferred once the phase-in has finished, and something is before then.
    if (r.reappraisal_year === 2022) expect(r.recognized_share).toBe(1);
  }

  /*
   * Local capacity used to be absent for 138 districts, because it was recovered by subtracting
   * state aid from base cost and that is impossible where the minimum state share binds. It is
   * no longer recovered: the department publishes it on the calculator's detail sheet, which
   * this project did not read for eleven phases. Every district has one.
   *
   * The residual is still `None` for those districts, and for the same reason — the *component*
   * comparison needs the recovered figure to line up with what `regime-diff` computed, and a
   * published number from a different route is not that. The totals were always computable.
   */
  expect(regimes.filter((r) => r.local_capacity == null).length).toBe(0);
  const censored = regimes.filter((r) => r.residual == null);
  expect(censored.length).toBeGreaterThan(130);

  // And three districts have no valuation at all, so neither side can be valued.
  expect(regimes.filter((r) => r.charge_off_local_share == null).length).toBe(3);
});

test("the counterfactual is computed on the formula's denominator, not the tax table's", () => {
  /*
   * The charge-off is subtracted from base cost per pupil, so its local share has to be per the
   * same pupil. Using Table SD-1's valuation per pupil would understate the deemed share for
   * every district whose taxation count is larger — and there are hundreds — so this pins which
   * of the two the deemed share was built from.
   */
  const { bundle } = loadFeed();
  for (const d of bundle.districts) {
    if (!d.regime?.charge_off_local_share || d.valuation_per_pupil == null) continue;
    // Times the recognized share, because the charge-off applied to recognized valuation and not
    // to the full taxable value. Dropping that factor is the error this whole phase corrected, so
    // it is pinned here as an identity rather than left to the prose.
    const expected =
      (d.valuation_per_pupil * d.regime.recognized_share * 23) / 1000;
    expect(d.regime.charge_off_local_share).toBeCloseTo(expected, 2);
  }
});

test("what one mill raises shares a denominator with the value per pupil beside it", () => {
  /*
   * Both figures appear in the same card. Dividing them by different pupil counts is the error
   * this project keeps finding in other people's tables, and the card would give no sign of it.
   */
  const { bundle } = loadFeed();
  for (const d of bundle.districts) {
    const m = d.millage;
    const y = d.property_tax[d.property_tax.length - 1];
    if (!m || !y || y.value_per_pupil <= 0) continue;

    // yield-per-mill × 1000 is the real property base per pupil, which must be the published
    // total per pupil less the public utility share of it.
    const realPerPupil = m.yield_per_mill_per_pupil * 1000;
    const expected =
      y.value_per_pupil * ((y.class1_value + y.class2_value) / y.total_value);
    // Relative, not absolute: both operands are serialized to four decimals, so on a base of a
    // few hundred thousand per pupil the last place is worth a few cents. A different pupil count
    // would be wrong by percent, not by parts per million.
    expect(Math.abs(realPerPupil - expected) / expected).toBeLessThan(1e-5);
  }
});

test("the federal split adds to the whole, and the share needs no denominator", () => {
  /*
   * Both parts are published per need-weighted pupil. The dollars therefore carry a denominator
   * that has to be named wherever they appear; their ratio does not, which is why the card leads
   * with the share. This pins both halves of that claim.
   */
  const { bundle } = loadFeed();
  const withSplit = bundle.districts.filter((d) => d.outcome?.per_equivalent_pupil_federal != null);
  expect(withSplit.length).toBe(606);

  for (const d of withSplit) {
    const o = d.outcome!;
    const parts = o.per_equivalent_pupil_federal! + o.per_equivalent_pupil_state_local!;
    expect(Math.abs(parts - o.per_equivalent_pupil!) / o.per_equivalent_pupil!).toBeLessThan(0.005);
  }

  const shares = withSplit.map((d) => d.outcome!.per_equivalent_pupil_federal! / d.outcome!.per_equivalent_pupil!);
  expect(Math.min(...shares)).toBeGreaterThan(0);
  expect(Math.max(...shares)).toBeCloseTo(bundle.statewide.outcomes!.max_federal_share, 4);
  expect(shares.filter((s) => s > 0.1).length).toBe(
    bundle.statewide.outcomes!.federal_share_above_tenth,
  );
});

test("the federal correlation is reported raw and controlled, and they differ a lot", () => {
  /*
   * Federal money follows poverty, so the raw association with attainment is mostly the poverty
   * association relabelled. Printing it alone would state a confound as a finding. The test is
   * that the two are far enough apart for the distinction to be doing work.
   */
  const o = loadFeed().bundle.statewide.outcomes!;
  expect(o.federal_share_vs_performance_raw).toBeLessThan(-0.4);
  expect(Math.abs(o.federal_share_vs_performance)).toBeLessThan(0.2);
  expect(Math.abs(o.federal_share_vs_performance_raw)).toBeGreaterThan(
    Math.abs(o.federal_share_vs_performance) * 3,
  );
});

test("growth-measure disagreement is counted over determinate pairs only", () => {
  /*
   * The defect this locks out. Value-added is published to two decimals, so a printed 0.00 covers
   * (-0.005, 0.005) and has no sign. A bare `a > 0 !== b > 0` reads all 72 of those as negative
   * and reports 76 disagreements where there are 44 — and the first version of this feature did
   * exactly that.
   */
  const { bundle } = loadFeed();
  const o = bundle.statewide.outcomes!;
  const pairs = bundle.districts
    .map((d) => d.outcome)
    .filter((x) => x?.progress_effect_size != null && x.progress_effect_size_one_year != null)
    .map((x) => [x!.progress_effect_size!, x!.progress_effect_size_one_year!] as const);

  const naive = pairs.filter(([a, b]) => a > 0 !== b > 0).length;
  const determinate = pairs.filter(([a, b]) => Math.abs(a) >= 0.005 && Math.abs(b) >= 0.005);
  const honest = determinate.filter(([a, b]) => a > 0 !== b > 0).length;

  expect(o.growth_measures_determinate).toBe(determinate.length);
  expect(o.growth_measures_disagree).toBe(honest);
  expect(naive).toBeGreaterThan(honest);

  // And the finding that makes the count readable: no disagreement survives a real threshold.
  expect(o.growth_measures_disagree_materially).toBe(0);
  for (const [a, b] of determinate.filter(([a, b]) => a > 0 !== b > 0)) {
    expect(Math.min(Math.abs(a), Math.abs(b))).toBeLessThan(0.05);
  }
});
