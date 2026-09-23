/**
 * The series cross-check, and the gate broken on purpose in each position it can fail in.
 *
 * Same shape as `corpusFigures.spec.ts` and for the same reason: the real corpus is checked
 * first, so the checker is pointed at something, and then each `SeriesDiscrepancyKind` is
 * produced from a node correct in every other position and asserted to be the only thing
 * reported. A check that has only ever seen correct input has not been shown to reject anything.
 */

import { expect, test } from "vitest";

import { loadCorpus, type Node } from "../../src/lib/corpus.ts";
import { loadFigureManifest, READS_CONTRACT, type Manifest } from "../../src/lib/corpusFigures.ts";
import {
  bandsAgainstFigures,
  barsOf,
  censusAgainstFigures,
  crossCheckPageSeries,
  crossCheckSeries,
  endpoints,
  PAGE_SERIES,
  rangesOf,
  ranksOf,
  fitsAgainstFigures,
  formatRow,
  loadSeriesManifest,
  type ManifestBand,
  type ManifestCurve,
  type ManifestSeries,
  type ManifestPlane,
  type ManifestSpread,
  multiplesOf,
  panelsOf,
  linesAgainstFigures,
  placesOf,
  pointsOf,
  positionsAgainstFigures,
  READS_SERIES_CONTRACT,
  regionsOf,
  rowsAgainstFigures,
  type SeriesDiscrepancyKind,
  type SeriesManifest,
} from "../../src/lib/corpusSeries.ts";

const corpus = loadCorpus();
const manifest = loadSeriesManifest();
const figures = loadFigureManifest();

test("every series a node draws is one the crate exports, bound at both ends", () => {
  const found = crossCheckSeries(corpus.nodes, manifest);
  expect(
    found.map((entry) => `[${entry.kind}] ${entry.node} — ${entry.message}`),
    "the corpus and crates/series.json have come apart",
  ).toEqual([]);
});

test("every series a page draws exists, and its ends are bound on the node it names", () => {
  const found = crossCheckPageSeries(corpus.nodes, manifest);
  expect(
    found.map((entry) => `[${entry.kind}] ${entry.node} — ${entry.message}`),
    "a page's chart has come apart from the node that answers for its figures",
  ).toEqual([]);
});

test("the two committed manifests agree on every row that names a figure", () => {
  expect(rowsAgainstFigures(manifest, figures)).toEqual([]);
});

test("the two committed manifests agree on every number printed on a panel", () => {
  expect(fitsAgainstFigures(manifest, figures)).toEqual([]);
});

test("the two committed manifests agree on both coordinates of every position", () => {
  expect(positionsAgainstFigures(manifest, figures)).toEqual([]);
});

test("the two committed manifests agree on every row of every spread's census", () => {
  expect(censusAgainstFigures(manifest, figures)).toEqual([]);
});

test("the two committed manifests agree on both ends of every band chart's end rows", () => {
  expect(bandsAgainstFigures(manifest, figures)).toEqual([]);
});

/**
 * The coverage floor, at the value and not under it, on the figure ratchet's standing rule:
 * recount with the expressions below rather than incrementing the last stated number.
 *
 * Five series over three carriers: `dispersion/fy2016-step-by-business-class` on
 * `education-agency/toledo-city`, which is #416's finding drawn as the four bars it was found by,
 * #444's three on `formula-component/temporary-transitional-aid-guarantee` — the plane the two
 * axes disagree on, and one grouped series per axis drawn as seven panels each — and #447's band
 * chart on `formula-component/fsfp-base-cost-calculation`.
 *
 * Every computation the manifest exports is drawn *somewhere*, and there are two somewheres: a
 * node's `series:` block, and a page declared in `PAGE_SERIES`. The sum is the assertion, because
 * the defect it exists to catch — a column computed for nobody — is the same defect whichever of
 * the two is meant to be drawing it.
 */
test("the corpus draws no fewer series than it did", () => {
  const drawn = corpus.nodes.flatMap((node) => node.series);
  expect(drawn.length, "5 series bindings; raise this when you add one").toBeGreaterThanOrEqual(5);
  expect(
    corpus.nodes.filter((node) => node.series.length > 0).length,
    "3 nodes draw a series; raise this when a fourth does",
  ).toBeGreaterThanOrEqual(3);
  expect(
    new Set([...drawn.map((binding) => binding.key), ...Object.keys(PAGE_SERIES)]).size,
    "every series, cloud, plane, curve, spread and band chart the manifest exports is drawn by " +
      "some node or page",
  ).toBe(
    manifest.series.length +
      manifest.scatters.length +
      manifest.planes.length +
      manifest.curves.length +
      manifest.spreads.length +
      manifest.bands.length,
  );
  expect(
    manifest.scatters.length,
    "1 cloud: project/what-the-capacity-tiers-index-measures on formula-component/" +
      "fsfp-targeted-assistance; raise this when you add one",
  ).toBeGreaterThanOrEqual(1);
  expect(
    manifest.planes.length,
    "1 plane: project/what-each-anchor-rule-costs-on-both-axes on formula-component/" +
      "temporary-transitional-aid-guarantee; raise this when you add one",
  ).toBeGreaterThanOrEqual(1);
  expect(
    manifest.curves.length,
    "1 curve: project/what-the-band-held-at-every-horizon, drawn by /method; raise this when you " +
      "add one",
  ).toBeGreaterThanOrEqual(1);
  expect(
    manifest.spreads.length,
    "2 spreads, both on formula-component/temporary-transitional-aid-guarantee: the two terms of " +
      "every district's multiple, and the same terms cut by wealth; raise this when you add one",
  ).toBeGreaterThanOrEqual(2);
  expect(
    manifest.bands.length,
    "1 band chart: project/administrators-employed-against-administrators-funded on " +
      "formula-component/fsfp-base-cost-calculation; raise this when you add one",
  ).toBeGreaterThanOrEqual(1);
});

test("the manifest declares the contract this module reads", () => {
  expect(manifest.contract).toBe(READS_SERIES_CONTRACT);
  // Its own contract, not the figure manifest's: the two are versioned apart on purpose, and a
  // test that passed because the strings happened to coincide would hide the next divergence.
  expect(manifest.series.every((series) => series.rows.length >= 2)).toBe(true);
});

test("the endpoints of a series are its largest and smallest rows by signed value", () => {
  const series = manifest.series.find((s) => s.key === "dispersion/fy2016-step-by-business-class");
  expect(series).toBeDefined();
  // Industrial is the top of the chart and mineral the bottom; the summed share sits between,
  // which is the cancellation the chart exists to show.
  expect(endpoints(series!).map((row) => row.label)).toEqual(["Industrial", "Mineral"]);
});

// --- The page-drawn series, broken on purpose in each position it can fail in -----------------

/**
 * These three mutate the **real** corpus and manifest rather than a fixture, which is the
 * opposite of the convention below and is deliberate: `crossCheckPageSeries` reads `PAGE_SERIES`
 * out of the module, so a fixture manifest would report every page-drawn series as missing no
 * matter what the fixture said. Taking the real inputs and removing exactly one thing is the only
 * way to produce one kind at a time here.
 */
const kindsOfPages = (nodes: Node[], series: SeriesManifest): SeriesDiscrepancyKind[] =>
  crossCheckPageSeries(nodes, series).map((entry) => entry.kind);

/** The nodes `PAGE_SERIES` names as answering for a page's endpoints. */
const pageSources = new Set(Object.values(PAGE_SERIES).map((source) => source.node));

test("PAGE_SERIES is not empty, and every entry names a route and a corpus id", () => {
  // A check over an empty table passes against any corpus. Four curves are drawn today:
  // `project/bounds-census` on `/bounds`, and three on `/method` — the coverage profile and the
  // two bias populations, the last two of which share the one node that answers for them. Raise
  // this when a fifth is drawn.
  const entries = Object.entries(PAGE_SERIES);
  expect(entries.length, "4 curves are drawn; raise this when a fifth is").toBeGreaterThanOrEqual(4);
  for (const [key, source] of entries) {
    expect(source.route, key).toMatch(/^\//);
    expect(source.node, key).toMatch(/^[a-z0-9-]+\/[a-z0-9-]+$/);
  }
});

test("unknown-key: a page declares a series crates/series.json does not carry", () => {
  const without: SeriesManifest = {
    ...manifest,
    series: manifest.series.filter((series) => !(series.key in PAGE_SERIES)),
    curves: manifest.curves.filter((curve) => !(curve.key in PAGE_SERIES)),
  };
  const found = kindsOfPages(corpus.nodes, without);
  expect(found).toEqual(Object.keys(PAGE_SERIES).map(() => "unknown-key"));
});

test("page-source-missing: the node a page cites for its endpoints is not in the corpus", () => {
  const without = corpus.nodes.filter((node) => !pageSources.has(node.id));
  // One discrepancy per *entry*, not per node: two curves may name the same answering node, and
  // both of them lose their answer when it goes. `pageSources` is a set and would undercount.
  expect(kindsOfPages(without, manifest)).toEqual(
    Object.keys(PAGE_SERIES).map(() => "page-source-missing"),
  );
});

test("the node a page cites binds nothing at the chart's ends", () => {
  const stripped = corpus.nodes.map((node) =>
    pageSources.has(node.id) ? { ...node, figures: [] } : node,
  );
  const found = kindsOfPages(stripped, manifest);
  expect(found.length, "a stripped source node reports every endpoint").toBeGreaterThan(0);
  // Two kinds, because a page can draw either shape and they are read at different places: a
  // column at its largest and smallest row, a curve at each line's two ends and worst departure.
  expect(new Set(found)).toEqual(new Set(["endpoints-unbound", "line-unbound"]));
});

test("a page-drawn series is not also reported as computed for nobody", () => {
  // The two checks are halves of one rule, and the half that lives in `crossCheckSeries` is a
  // single `drawn.add`. Drop it and every page's chart is reported as a column nothing draws.
  const found = crossCheckSeries(corpus.nodes, manifest).filter(
    (entry) => entry.kind === "uncited-series",
  );
  expect(found.map((entry) => entry.key)).toEqual([]);
});

// --- The gate, broken on purpose in each position ---------------------------------------------

/** A node correct in every position, and the manifests it is correct against. */
function fixture(): { node: Node; series: SeriesManifest; figures: Manifest } {
  const node: Node = {
    id: "formula-component/example",
    className: "formula-component",
    name: "example",
    label: "Example",
    summary: "Industrial runs +0.1175 and mineral −0.0321.",
    description: "Industrial runs +0.1175 and mineral −0.0321. [verified] (`crates/dispersion`)",
    linkText: "Industrial runs +0.1175 and mineral −0.0321. [verified] (`crates/dispersion`)",
    properties: [],
    findings: null,
    revisions: [],
    unfilled: [],
    figures: [
      {
        key: "dispersion/industrial",
        value: 0.1175,
        field: "description",
        as_written: "Industrial runs +0.1175",
      },
      {
        key: "dispersion/mineral-negative",
        value: 0.0321,
        field: "description",
        as_written: "mineral −0.0321",
      },
    ],
    series: [{ key: "dispersion/by-class", field: "description" }],
    out: [],
    in: [],
  };
  const series: SeriesManifest = {
    contract: READS_SERIES_CONTRACT,
    scatters: [],
    planes: [],
    curves: [],
    spreads: [],
    bands: [],
    series: [
      {
        key: "dispersion/by-class",
        owner: "crates/dispersion",
        unit: "ratio",
        axis: "Property class",
        label: "Correlation of the step with each class",
        rows: [
          { label: "Industrial", value: 0.1175, figure: "dispersion/industrial" },
          { label: "Mineral", value: -0.0321, figure: "dispersion/mineral-negative" },
          { label: "Public utility", value: -0.0159 },
        ],
      },
    ],
  };
  const figures: Manifest = {
    contract: READS_CONTRACT,
    figures: [
      {
        key: "dispersion/industrial",
        owner: "crates/dispersion",
        unit: "ratio",
        value: 0.1175,
        label: "Against industrial share, positive",
      },
      {
        key: "dispersion/mineral-negative",
        owner: "crates/dispersion",
        unit: "ratio",
        value: 0.0321,
        label: "Against mineral share, negative",
      },
    ],
  };
  return { node, series, figures };
}

/** Run the check over one node and report only the kinds it found. */
function kinds(node: Node, series: SeriesManifest): SeriesDiscrepancyKind[] {
  return crossCheckSeries([node], series).map((entry) => entry.kind);
}

test("the fixture the mutations are cut from is itself clean", () => {
  const { node, series, figures: mine } = fixture();
  expect(kinds(node, series)).toEqual([]);
  expect(rowsAgainstFigures(series, mine)).toEqual([]);
});

test("unknown-key: the node draws a series the manifest does not carry", () => {
  const { node, series } = fixture();
  node.series[0]!.key = "dispersion/by-class-renamed";
  // The manifest's own series is then drawn by nobody, which is the second finding.
  expect(kinds(node, series)).toEqual(["unknown-key", "uncited-series"]);
});

test("field-missing: the node draws it under a field it does not have", () => {
  const { node, series } = fixture();
  node.series[0]!.field = "findings";
  expect(kinds(node, series)).toEqual(["field-missing"]);
});

test("field-missing: the node draws it under a field it has and no chart can go under", () => {
  const { node, series } = fixture();
  node.series[0]!.field = "summary";
  expect(kinds(node, series)).toEqual(["field-missing"]);
});

test("a series becomes the bars the chart draws, signed and with a spoken value", () => {
  const { series } = fixture();
  expect(barsOf(series.series[0]!)).toEqual([
    { label: "Industrial", value: 0.1175, hover: "Industrial: +0.1175" },
    { label: "Mineral", value: -0.0321, hover: "Mineral: −0.0321" },
    { label: "Public utility", value: -0.0159, hover: "Public utility: −0.0159" },
  ]);
  // A hover the manifest gives is kept as given.
  series.series[0]!.rows[0]!.hover = "Industrial, the wrong sign for a TPP reading";
  expect(barsOf(series.series[0]!)[0]!.hover).toBe("Industrial, the wrong sign for a TPP reading");
  // And each unit is written the way its readers write it.
  expect(formatRow("share", -0.281)).toBe("−28.1%");
  expect(formatRow("dollars", 5_100_000)).toBe("+$5,100,000");
  expect(formatRow("count", 65)).toBe("+65");
  expect(formatRow("ratio", 0)).toBe("0.0000");
});

test("a census becomes the ranked rows the log chart draws, with its marked row kept", () => {
  /*
   * The four members contract 3.0.0 added, in the one place they are all read. The hover is
   * composed here rather than taken from the manifest — the count, the population it is out of,
   * and the provision — because a census row's `hover` is mostly empty and what a reader wants
   * pointed at is the row of the table beside the chart. The denominator is printed on every row
   * that has one, including the rows where it is the whole model: a tooltip is read one at a
   * time, so "554" with the 609 left off is a figure a reader has to supply the basis for.
   */
  const series: ManifestSeries = {
    key: "project/example-census",
    owner: "crates/project",
    unit: "count",
    axis: "Bound",
    label: "Every bound",
    rows: [
      { label: "A floor that binds", value: 554, group: "Base cost", of: 609, cites: "R.C. 1" },
      { label: "A floor on a subset", value: 12, group: "Transport", of: 604, cites: "R.C. 2" },
      {
        label: "A floor that cannot",
        value: 0,
        group: "Base cost",
        of: 609,
        cites: "R.C. 3",
        marked: "cannot bind",
      },
    ],
  };
  expect(ranksOf(series)).toEqual([
    { label: "A floor that binds", value: 554, hover: "A floor that binds: 554 of 609 — R.C. 1" },
    {
      label: "A floor on a subset",
      value: 12,
      hover: "A floor on a subset: 12 of 604 — R.C. 2",
    },
    {
      label: "A floor that cannot",
      value: 0,
      hover: "A floor that cannot: 0 of 609 — R.C. 3",
      marked: "cannot bind",
    },
  ]);
});

test("unattributed: the node cites the owning crate in no claim tag", () => {
  const { node, series } = fixture();
  node.description = "Industrial runs +0.1175 and mineral −0.0321. [verified]";
  node.linkText = node.description;
  expect(kinds(node, series)).toEqual(["unattributed"]);
});

test("endpoints-unbound: the node binds one end of the chart and not the other", () => {
  const { node, series } = fixture();
  node.figures = node.figures.filter((figure) => figure.key !== "dispersion/mineral-negative");
  expect(kinds(node, series)).toEqual(["endpoints-unbound"]);
});

test("endpoints-unbound: the manifest names no figure at an endpoint", () => {
  const { node, series } = fixture();
  // Public utility becomes the smallest row, and it names nothing.
  series.series[0]!.rows[2]!.value = -0.5;
  expect(kinds(node, series)).toEqual(["endpoints-unbound"]);
});

test("a row that is not an endpoint may go unbound", () => {
  const { node, series } = fixture();
  // The middle row names a figure the node does not bind; the rule is about the ends.
  series.series[0]!.rows[2]!.figure = "dispersion/utility-negative";
  expect(kinds(node, series)).toEqual([]);
});

test("uncited-series: the manifest exports a series no node draws", () => {
  const { node, series } = fixture();
  series.series.push({ ...series.series[0]!, key: "dispersion/by-class-again" });
  expect(kinds(node, series)).toEqual(["uncited-series"]);
});

test("the two manifests are held to each other, row by figure", () => {
  const { series, figures: mine } = fixture();
  // A sign is not a disagreement: the row is signed and the figure is a magnitude.
  series.series[0]!.rows[0]!.value = -0.1175;
  expect(rowsAgainstFigures(series, mine)).toEqual([]);
  // A different magnitude is.
  series.series[0]!.rows[0]!.value = 0.1176;
  expect(rowsAgainstFigures(series, mine)).toHaveLength(1);
  // As is a figure the other document lacks, or one on the wrong unit.
  series.series[0]!.rows[0]!.value = 0.1175;
  series.series[0]!.rows[0]!.figure = "dispersion/absent";
  expect(rowsAgainstFigures(series, mine)).toHaveLength(1);
  series.series[0]!.rows[0]!.figure = "dispersion/industrial";
  // The unit is the series', so changing it puts both figure-naming rows on the wrong one.
  series.series[0]!.unit = "share";
  expect(rowsAgainstFigures(series, mine)).toHaveLength(2);
});

// --- The cloud, which is held by its fit rather than by its ends -------------------------------

/**
 * A node drawing a cloud, correct in every position, and the manifests it is correct against.
 *
 * Cut separately from {@link fixture} rather than added to it. A node binding both shapes would
 * report every owner-level defect twice — one finding per binding, from the same missing claim
 * tag — and every existing mutation below would then assert a count instead of a kind.
 */
function cloudFixture(): { node: Node; series: SeriesManifest; figures: Manifest } {
  const prose =
    "Total wealth runs 0.9855 on size at an r-squared of 0.8126; per pupil it runs -0.0429 at " +
    "0.0085. [verified] (`crates/project`)";
  const node: Node = {
    id: "formula-component/example",
    className: "formula-component",
    name: "example",
    label: "Example",
    summary: "The index measures size.",
    description: prose,
    linkText: prose,
    properties: [],
    findings: null,
    revisions: [],
    unfilled: [],
    figures: [
      { key: "project/total-slope", value: 0.9855, field: "description", as_written: "0.9855" },
      { key: "project/total-fit", value: 0.8126, field: "description", as_written: "0.8126" },
      {
        key: "project/per-pupil-slope-negative",
        value: 0.0429,
        field: "description",
        as_written: "-0.0429",
      },
      { key: "project/per-pupil-fit", value: 0.0085, field: "description", as_written: "0.0085" },
    ],
    series: [{ key: "project/what-it-measures", field: "description" }],
    out: [],
    in: [],
  };
  const axis = (label: string, min: number, max: number) =>
    ({ label, unit: "dollars", min, max, log: true }) as const;
  const series: SeriesManifest = {
    contract: READS_SERIES_CONTRACT,
    series: [],
    planes: [],
    curves: [],
    spreads: [],
    bands: [],
    scatters: [
      {
        key: "project/what-it-measures",
        owner: "crates/project",
        subject: "district",
        label: "What the index varies with",
        x: { label: "Enrolled ADM", unit: "pupils", min: 4, max: 62_000, log: true },
        panels: [
          {
            label: "Total weighted wealth, which the tier compares",
            y: axis("Total weighted wealth", 1e6, 1.8e10),
            fit: {
              from: [4, 1.1e6],
              to: [62_000, 1.7e10],
              slope: 0.9855,
              rSquared: 0.8126,
              slopeFigure: "project/total-slope",
              rSquaredFigure: "project/total-fit",
            },
          },
          {
            label: "Weighted wealth per resident pupil, which it names",
            y: axis("Weighted wealth per resident pupil", 1e4, 1.8e8),
            fit: {
              from: [4, 3.1e5],
              to: [62_000, 2.4e5],
              slope: -0.0429,
              rSquared: 0.0085,
              slopeFigure: "project/per-pupil-slope-negative",
              rSquaredFigure: "project/per-pupil-fit",
            },
          },
        ],
        points: [
          { label: "Kelleys Island Local", x: 4, ys: [2.1e6, 5.2e5] },
          { label: "Manchester Local", x: 673.6075, ys: [1.03e8, 1.62e5] },
          { label: "Columbus City", x: 45_000, ys: [1.6e10, 3.5e5] },
        ],
      },
    ],
  };
  const ratio = (key: string, value: number, label: string) =>
    ({ key, owner: "crates/project", unit: "ratio", value, label }) as const;
  const figures: Manifest = {
    contract: READS_CONTRACT,
    figures: [
      ratio("project/total-slope", 0.9855, "Total wealth on enrolment, slope"),
      ratio("project/total-fit", 0.8126, "Total wealth on enrolment, r-squared"),
      ratio("project/per-pupil-slope-negative", 0.0429, "Per pupil on enrolment, slope, negative"),
      ratio("project/per-pupil-fit", 0.0085, "Per pupil on enrolment, r-squared"),
    ],
  };
  return { node, series, figures };
}

test("the cloud fixture the mutations are cut from is itself clean", () => {
  const { node, series, figures: mine } = cloudFixture();
  expect(kinds(node, series)).toEqual([]);
  expect(fitsAgainstFigures(series, mine)).toEqual([]);
});

test("fit-unbound: the node does not bind a slope the panel prints", () => {
  const { node, series } = cloudFixture();
  node.figures = node.figures.filter((f) => f.key !== "project/per-pupil-slope-negative");
  expect(kinds(node, series)).toEqual(["fit-unbound"]);
});

test("fit-unbound: the node does not bind an r-squared the panel prints", () => {
  const { node, series } = cloudFixture();
  node.figures = node.figures.filter((f) => f.key !== "project/total-fit");
  expect(kinds(node, series)).toEqual(["fit-unbound"]);
});

test("unknown-key: the node draws a cloud the manifest does not carry", () => {
  const { node, series } = cloudFixture();
  node.series[0]!.key = "project/what-it-measures-renamed";
  expect(kinds(node, series)).toEqual(["unknown-key", "uncited-series"]);
});

test("unattributed: the node draws a crate's cloud and cites that crate nowhere", () => {
  const { node, series } = cloudFixture();
  node.description = node.description.replace(" (`crates/project`)", "");
  node.linkText = node.description;
  expect(kinds(node, series)).toEqual(["unattributed"]);
});

test("uncited-series: the manifest exports a cloud no node draws", () => {
  const { node, series } = cloudFixture();
  series.scatters.push({ ...series.scatters[0]!, key: "project/what-it-measures-again" });
  expect(kinds(node, series)).toEqual(["uncited-series"]);
});

test("the two manifests are held to each other, panel by figure", () => {
  const { series, figures: mine } = cloudFixture();
  // A sign is not a disagreement: the slope is signed and the figure is a magnitude, which is the
  // whole reason the key says `-negative`.
  expect(fitsAgainstFigures(series, mine)).toEqual([]);
  series.scatters[0]!.panels[1]!.fit.slope = 0.043;
  expect(fitsAgainstFigures(series, mine)).toHaveLength(1);
  // An r-squared has no direction, so it is compared as it stands and a sign on one is a defect.
  series.scatters[0]!.panels[1]!.fit.slope = -0.0429;
  series.scatters[0]!.panels[1]!.fit.rSquared = -0.0085;
  expect(fitsAgainstFigures(series, mine)).toHaveLength(1);
  // As is a figure the other document lacks, or one that is not a ratio.
  series.scatters[0]!.panels[1]!.fit.rSquared = 0.0085;
  series.scatters[0]!.panels[0]!.fit.slopeFigure = "project/absent";
  expect(fitsAgainstFigures(series, mine)).toHaveLength(1);
  series.scatters[0]!.panels[0]!.fit.slopeFigure = "project/total-slope";
  mine.figures[0]!.unit = "share";
  expect(fitsAgainstFigures(series, mine)).toHaveLength(1);
});

test("a cloud becomes one panel per outcome, every point under each, on the crate's own frame", () => {
  const { series } = cloudFixture();
  const cloud = series.scatters[0]!;
  const panels = panelsOf(cloud);
  expect(panels).toHaveLength(2);
  for (const panel of panels) {
    expect(panel.points).toHaveLength(cloud.points.length);
    // The frame is the manifest's, never fitted here: `scatterSpec` refuses a fit without both
    // domains, because a y axis fitted to its own points would redraw the slope.
    expect(panel.xDomain).toEqual([cloud.x.min, cloud.x.max]);
  }
  expect(panels[0]!.yDomain).toEqual([1e6, 1.8e10]);
  // The tooltip says which district and where it sits on both axes, each in its own unit.
  expect(panels[1]!.points[1]!.hover).toBe(
    "Manchester Local: Enrolled ADM 673.6, Weighted wealth per resident pupil $162,000",
  );
  // The line arrives as a segment in the data's own units — nothing here fits anything.
  expect(panels[1]!.fit).toEqual({
    from: { x: 4, y: 3.1e5 },
    to: { x: 62_000, y: 2.4e5 },
    slope: -0.0429,
    rSquared: 0.0085,
  });
});

test("a cloud missing a point from one panel is refused rather than drawn short", () => {
  const { series } = cloudFixture();
  series.scatters[0]!.points[1]!.ys = [1.03e8];
  expect(() => panelsOf(series.scatters[0]!)).toThrow(/carries no value under/);
});

test("the committed cloud is drawn at one scale, so a slope is an angle", () => {
  const cloud = manifest.scatters.find((c) => c.key === "project/what-the-capacity-tiers-index-measures");
  expect(cloud).toBeDefined();
  const span = Math.log(cloud!.x.max / cloud!.x.min);
  for (const panel of cloud!.panels) {
    // The crate asserts this on the way out; asserted again on the way in because it is the one
    // property that makes the pair a picture rather than two pictures.
    expect(Math.log(panel.y.max / panel.y.min) / span).toBeCloseTo(1, 12);
    // And both ends of the fitted line are inside the frame that was built to hold them.
    for (const end of [panel.fit.from, panel.fit.to]) {
      expect(end[0]).toBeGreaterThanOrEqual(cloud!.x.min);
      expect(end[0]).toBeLessThanOrEqual(cloud!.x.max);
      expect(end[1]).toBeGreaterThanOrEqual(panel.y.min);
      expect(end[1]).toBeLessThanOrEqual(panel.y.max);
    }
  }
});

// --- The plane, which is held at every position on both of its axes ----------------------------

/**
 * A node drawing a plane, correct in every position, and the manifests it is correct against.
 *
 * Cut separately from {@link fixture} and {@link cloudFixture} for the reason given above the
 * second of those: a node binding two shapes reports every owner-level defect once per binding,
 * and each mutation below would then assert a count rather than a kind.
 *
 * Three positions rather than the committed seven, and both axes signed, because the property the
 * plane exists for is that a rule can be *cheaper* on one axis and *dearer* on the other. One
 * point in each of the two quadrants that says so, and one that agrees with itself.
 */
function planeFixture(): { node: Node; series: SeriesManifest; figures: Manifest } {
  const prose =
    "The cap takes $107,611,728.73 off the guarantee and adds $10,287,443.63 to total state " +
    "support; the ratchet adds $57,298,691.08 of guarantee and $52,907,613.45 of support; the " +
    "mirror beside moves the guarantee column by $0.00 and pays $135,300,935.56. [verified] " +
    "(`crates/project`)";
  const bind = (key: string, value: number, as_written: string) =>
    ({ key, value, field: "description", as_written }) as const;
  const node: Node = {
    id: "formula-component/example-plane",
    className: "formula-component",
    name: "example-plane",
    label: "Example plane",
    summary: "The two axes disagree.",
    description: prose,
    linkText: prose,
    properties: [],
    findings: null,
    revisions: [],
    unfilled: [],
    figures: [
      bind("project/cap-guarantee-cut", 107_611_728.73, "$107,611,728.73"),
      bind("project/cap-support-gain", 10_287_443.63, "$10,287,443.63"),
      bind("project/ratchet-guarantee", 57_298_691.08, "$57,298,691.08"),
      bind("project/ratchet-support", 52_907_613.45, "$52,907,613.45"),
      bind("project/mirror-guarantee", 0, "$0.00"),
      bind("project/mirror-support-gain", 135_300_935.56, "$135,300,935.56"),
    ],
    series: [{ key: "project/what-each-rule-costs", field: "description" }],
    out: [],
    in: [],
  };
  const axis = (label: string, min: number, max: number) =>
    ({ label, unit: "dollars", min, max, log: false }) as const;
  const plane: ManifestPlane = {
    key: "project/what-each-rule-costs",
    owner: "crates/project",
    subject: "anchor rule",
    label: "What each rule costs on both axes",
    x: axis("Guarantee written", -1.2e8, 6.3e7),
    y: axis("Total state support", -1e7, 1.4e8),
    points: [
      {
        label: "1% cap at FY2036",
        x: -107_611_728.73,
        y: 10_287_443.63,
        xFigure: "project/cap-guarantee-cut",
        yFigure: "project/cap-support-gain",
      },
      {
        label: "the ratchet",
        x: 57_298_691.08,
        y: 52_907_613.45,
        xFigure: "project/ratchet-guarantee",
        yFigure: "project/ratchet-support",
      },
      {
        label: "mirror beside",
        x: 0,
        y: 135_300_935.56,
        xFigure: "project/mirror-guarantee",
        yFigure: "project/mirror-support-gain",
      },
    ],
  };
  const series: SeriesManifest = {
    contract: READS_SERIES_CONTRACT,
    series: [],
    scatters: [],
    curves: [],
    spreads: [],
    bands: [],
    planes: [plane],
  };
  const dollars = (key: string, value: number, label: string) =>
    ({ key, owner: "crates/project", unit: "dollars", value, label }) as const;
  const figures: Manifest = {
    contract: READS_CONTRACT,
    figures: [
      dollars("project/cap-guarantee-cut", 107_611_728.73, "The cap's guarantee cut"),
      dollars("project/cap-support-gain", 10_287_443.63, "The cap's support gain"),
      dollars("project/ratchet-guarantee", 57_298_691.08, "The ratchet's guarantee"),
      dollars("project/ratchet-support", 52_907_613.45, "The ratchet's support"),
      dollars("project/mirror-guarantee", 0, "The mirror beside's guarantee"),
      dollars("project/mirror-support-gain", 135_300_935.56, "The mirror beside's support gain"),
    ],
  };
  return { node, series, figures };
}

test("the plane fixture the mutations are cut from is itself clean", () => {
  const { node, series, figures: mine } = planeFixture();
  expect(kinds(node, series)).toEqual([]);
  expect(positionsAgainstFigures(series, mine)).toEqual([]);
});

test("position-unbound: the node does not bind the x of a point the plane prints", () => {
  const { node, series } = planeFixture();
  node.figures = node.figures.filter((f) => f.key !== "project/cap-guarantee-cut");
  expect(kinds(node, series)).toEqual(["position-unbound"]);
});

test("position-unbound: the node does not bind the y of a point the plane prints", () => {
  const { node, series } = planeFixture();
  node.figures = node.figures.filter((f) => f.key !== "project/mirror-support-gain");
  expect(kinds(node, series)).toEqual(["position-unbound"]);
});

test("position-unbound: a point bound on neither axis is two findings, not one", () => {
  const { node, series } = planeFixture();
  node.figures = node.figures.filter((f) => !f.key.startsWith("project/ratchet-"));
  expect(kinds(node, series)).toEqual(["position-unbound", "position-unbound"]);
});

test("unknown-key: the node draws a plane the manifest does not carry", () => {
  const { node, series } = planeFixture();
  node.series[0]!.key = "project/what-each-rule-costs-renamed";
  expect(kinds(node, series)).toEqual(["unknown-key", "uncited-series"]);
});

test("unattributed: the node draws a crate's plane and cites that crate nowhere", () => {
  const { node, series } = planeFixture();
  node.description = node.description.replace(" (`crates/project`)", "");
  node.linkText = node.description;
  expect(kinds(node, series)).toEqual(["unattributed"]);
});

test("uncited-series: the manifest exports a plane no node draws", () => {
  const { node, series } = planeFixture();
  series.planes.push({ ...series.planes[0]!, key: "project/what-each-rule-costs-again" });
  expect(kinds(node, series)).toEqual(["uncited-series"]);
});

test("the two manifests are held to each other, coordinate by figure", () => {
  const { series, figures: mine } = planeFixture();
  // A sign is not a disagreement: the coordinate is signed and the figure is a magnitude, which
  // is why the cap's x is negative and its key says `-cut`.
  expect(positionsAgainstFigures(series, mine)).toEqual([]);
  // A different magnitude is, on either axis, and one point can be wrong on both at once.
  series.planes[0]!.points[0]!.x = -107_611_728.74;
  expect(positionsAgainstFigures(series, mine)).toHaveLength(1);
  series.planes[0]!.points[0]!.y = 10_287_443.64;
  expect(positionsAgainstFigures(series, mine)).toHaveLength(2);
  series.planes[0]!.points[0]!.x = -107_611_728.73;
  series.planes[0]!.points[0]!.y = 10_287_443.63;
  // As is a figure the other document lacks, or one on the wrong unit.
  series.planes[0]!.points[1]!.yFigure = "project/absent";
  expect(positionsAgainstFigures(series, mine)).toHaveLength(1);
  series.planes[0]!.points[1]!.yFigure = "project/ratchet-support";
  // The unit is the axis', so changing it puts every point on that axis on the wrong one.
  series.planes[0]!.y.unit = "pupils";
  expect(positionsAgainstFigures(series, mine)).toHaveLength(3);
});

test("a plane becomes one named place per rule, both coordinates signed and spoken", () => {
  const { series } = planeFixture();
  const places = placesOf(series.planes[0]!);
  expect(places.map((place) => place.label)).toEqual([
    "1% cap at FY2036",
    "the ratchet",
    "mirror beside",
  ]);
  expect(places.map((place) => [place.x, place.y])).toEqual([
    [-107_611_728.73, 10_287_443.63],
    [57_298_691.08, 52_907_613.45],
    [0, 135_300_935.56],
  ]);
  // The hover carries both axes by name, because a dot on a plane says nothing on its own about
  // which measure is which — and it carries the sign, which is the finding.
  expect(places[0]!.hover).toBe(
    "1% cap at FY2036: Guarantee written −$107,611,729, Total state support +$10,287,444",
  );
});

test("the committed plane holds zero on both axes, and every position inside the frame", () => {
  const plane = manifest.planes.find(
    (p) => p.key === "project/what-each-anchor-rule-costs-on-both-axes",
  );
  expect(plane).toBeDefined();
  for (const axis of [plane!.x, plane!.y]) {
    // Zero is the rule in force. A frame that excludes it would put every rule on the same side
    // of nothing and lose the only reading the picture is for.
    expect(axis.min).toBeLessThanOrEqual(0);
    expect(axis.max).toBeGreaterThanOrEqual(0);
    expect(axis.log).toBe(false);
  }
  for (const point of plane!.points) {
    expect(point.x).toBeGreaterThanOrEqual(plane!.x.min);
    expect(point.x).toBeLessThanOrEqual(plane!.x.max);
    expect(point.y).toBeGreaterThanOrEqual(plane!.y.min);
    expect(point.y).toBeLessThanOrEqual(plane!.y.max);
  }
});

test("the committed plane is the finding: three rules cheaper on one axis and dearer on the other", () => {
  const plane = manifest.planes.find(
    (p) => p.key === "project/what-each-anchor-rule-costs-on-both-axes",
  )!;
  const disagreeing = plane.points.filter((p) => p.x < 0 && p.y > 0).map((p) => p.label);
  expect(disagreeing).toEqual([
    "A 1% cap, FY2036 undamped",
    "A rolling pupil count",
    "[M] mirrored inside [H]",
  ]);
  // And nothing runs the other way. That asymmetry is what the two-column table cannot show.
  expect(plane.points.filter((p) => p.x > 0 && p.y < 0)).toEqual([]);
});

// --- The curve, which is held at both ends of every line and at its worst departure ------------

/**
 * A node drawing a curve, correct in every position, and the manifests it is correct against.
 *
 * Cut separately from the three fixtures above for the reason each of those was: a node binding
 * two shapes reports every owner-level defect once per binding, and each mutation below would
 * then assert a count rather than a kind.
 *
 * Three positions rather than the committed thirteen, and two lines rather than one, because the
 * property the curve exists for is that two lines over one index can depart from a reference by
 * different amounts — and the departure is a maximum over the whole index, so it needs an index
 * long enough for its worst point not to be an end.
 */
function curveFixture(): { node: Node; series: SeriesManifest; figures: Manifest } {
  const prose =
    "Pooled, the band held 64.5% at one year and 60.1% at thirteen, at worst 8.9% short; with " +
    "each origin's mean removed it held 66.1% and 69.4%, within 3.0% throughout. [verified] " +
    "(`crates/project`)";
  const bind = (key: string, value: number, as_written: string) =>
    ({ key, value, field: "description", as_written }) as const;
  const node: Node = {
    id: "scenario/example-curve",
    className: "scenario",
    name: "example-curve",
    label: "Example curve",
    summary: "What the band held.",
    description: prose,
    linkText: prose,
    properties: [],
    findings: null,
    revisions: [],
    unfilled: [],
    figures: [
      bind("project/pooled-first", 0.645, "64.5%"),
      bind("project/pooled-last", 0.601, "60.1%"),
      bind("project/pooled-worst", 0.089, "8.9%"),
      bind("project/cross-first", 0.661, "66.1%"),
      bind("project/cross-last", 0.694, "69.4%"),
      bind("project/cross-worst", 0.03, "3.0%"),
    ],
    series: [{ key: "project/what-the-band-held", field: "description" }],
    out: [],
    in: [],
  };
  const curve: ManifestCurve = {
    key: "project/what-the-band-held",
    owner: "crates/project",
    subject: "horizon",
    label: "What the band held at every horizon",
    x: { label: "Years ahead", unit: "count", min: 1, max: 3, log: false },
    y: { label: "Share of forecasts inside the band", unit: "share", min: 0.55, max: 0.72, log: false },
    reference: { label: "What ±1σ claims to hold", value: 0.683 },
    lines: [
      {
        label: "Every error, origins pooled",
        points: [
          { x: 1, y: 0.645 },
          { x: 2, y: 0.594 },
          { x: 3, y: 0.601 },
        ],
        firstFigure: "project/pooled-first",
        lastFigure: "project/pooled-last",
        worst: { at: 2, gap: 0.089, figure: "project/pooled-worst" },
      },
      {
        label: "The same errors, each origin's mean removed",
        points: [
          { x: 1, y: 0.661 },
          { x: 2, y: 0.713 },
          { x: 3, y: 0.694 },
        ],
        firstFigure: "project/cross-first",
        lastFigure: "project/cross-last",
        worst: { at: 2, gap: 0.03, figure: "project/cross-worst" },
      },
    ],
  };
  const series: SeriesManifest = {
    contract: READS_SERIES_CONTRACT,
    series: [],
    scatters: [],
    planes: [],
    curves: [curve],
    spreads: [],
    bands: [],
  };
  const share = (key: string, value: number, label: string) =>
    ({ key, owner: "crates/project", unit: "share", value, label }) as const;
  const figures: Manifest = {
    contract: READS_CONTRACT,
    figures: [
      share("project/pooled-first", 0.645, "Pooled coverage at one year"),
      share("project/pooled-last", 0.601, "Pooled coverage at three years"),
      share("project/pooled-worst", 0.089, "The pooled band's worst gap"),
      share("project/cross-first", 0.661, "Cross-district coverage at one year"),
      share("project/cross-last", 0.694, "Cross-district coverage at three years"),
      share("project/cross-worst", 0.03, "The cross-district band's worst gap"),
    ],
  };
  return { node, series, figures };
}

test("the curve fixture the mutations are cut from is itself clean", () => {
  const { node, series, figures: mine } = curveFixture();
  expect(kinds(node, series)).toEqual([]);
  expect(linesAgainstFigures(series, mine)).toEqual([]);
});

test("line-unbound: the node does not bind where a line starts", () => {
  const { node, series } = curveFixture();
  node.figures = node.figures.filter((f) => f.key !== "project/pooled-first");
  expect(kinds(node, series)).toEqual(["line-unbound"]);
});

test("line-unbound: the node does not bind where a line ends", () => {
  const { node, series } = curveFixture();
  node.figures = node.figures.filter((f) => f.key !== "project/cross-last");
  expect(kinds(node, series)).toEqual(["line-unbound"]);
});

test("line-unbound: the node binds both ends and not the worst departure", () => {
  // The one an endpoint rule written for a column would miss. "Within three points at every
  // horizon" is the claim, and no coordinate on the line states it.
  const { node, series } = curveFixture();
  node.figures = node.figures.filter((f) => f.key !== "project/cross-worst");
  expect(kinds(node, series)).toEqual(["line-unbound"]);
});

test("line-unbound: a line bound nowhere is three findings, not one", () => {
  const { node, series } = curveFixture();
  node.figures = node.figures.filter((f) => !f.key.startsWith("project/pooled-"));
  expect(kinds(node, series)).toEqual(["line-unbound", "line-unbound", "line-unbound"]);
});

test("unknown-key: the node draws a curve the manifest does not carry", () => {
  const { node, series } = curveFixture();
  node.series[0]!.key = "project/what-the-band-held-renamed";
  expect(kinds(node, series)).toEqual(["unknown-key", "uncited-series"]);
});

test("unattributed: the node draws a crate's curve and cites that crate nowhere", () => {
  const { node, series } = curveFixture();
  node.description = node.description.replace(" (`crates/project`)", "");
  node.linkText = node.description;
  expect(kinds(node, series)).toEqual(["unattributed"]);
});

test("uncited-series: the manifest exports a curve no node draws", () => {
  const { node, series } = curveFixture();
  series.curves.push({ ...series.curves[0]!, key: "project/what-the-band-held-again" });
  expect(kinds(node, series)).toEqual(["uncited-series"]);
});

test("the two manifests are held to each other, line end by figure", () => {
  const { series, figures: mine } = curveFixture();
  expect(linesAgainstFigures(series, mine)).toEqual([]);
  // Each of the three is checked, and one line can be wrong at all three at once.
  series.curves[0]!.lines[0]!.points[0]!.y = 0.646;
  expect(linesAgainstFigures(series, mine)).toHaveLength(1);
  series.curves[0]!.lines[0]!.points[2]!.y = 0.602;
  series.curves[0]!.lines[0]!.worst.gap = 0.088;
  expect(linesAgainstFigures(series, mine)).toHaveLength(3);
  series.curves[0]!.lines[0]!.points[0]!.y = 0.645;
  series.curves[0]!.lines[0]!.points[2]!.y = 0.601;
  series.curves[0]!.lines[0]!.worst.gap = 0.089;
  // As is a figure the other document lacks, or one on the wrong unit. The unit is the y axis',
  // so changing it puts every one of the six on the wrong one.
  series.curves[0]!.lines[1]!.lastFigure = "project/absent";
  expect(linesAgainstFigures(series, mine)).toHaveLength(1);
  series.curves[0]!.lines[1]!.lastFigure = "project/cross-last";
  series.curves[0]!.y.unit = "pupils";
  expect(linesAgainstFigures(series, mine)).toHaveLength(6);
});

test("a curve becomes two lines over one index, and a third is refused rather than dropped", () => {
  const { series } = curveFixture();
  const points = pointsOf(series.curves[0]!);
  expect(points).toEqual([
    { at: 1, a: 0.645, b: 0.661 },
    { at: 2, a: 0.594, b: 0.713 },
    { at: 3, a: 0.601, b: 0.694 },
  ]);
  // `seriesSpec` draws a validated pair and there is no third hue to give a third line. Silently
  // drawing two of three would be a picture that omits a finding without saying so.
  series.curves[0]!.lines.push({ ...series.curves[0]!.lines[0]!, label: "A third" });
  expect(() => pointsOf(series.curves[0]!)).toThrow(/has 3 lines and seriesSpec draws two/);
});

test("the committed curve is flat where the claim is flat, and sags where it sags", () => {
  const curve = manifest.curves.find(
    (c) => c.key === "project/what-the-band-held-at-every-horizon",
  );
  expect(curve).toBeDefined();
  // The reference is the whole reading, and it is not a figure. See `ManifestReference`.
  expect(curve!.reference?.value).toBeCloseTo(0.683, 10);
  const [pooled, cross] = curve!.lines;
  // Both lines over the same index, or the second is drawn against positions the first does not
  // have and the pairing `seriesSpec` needs is a coincidence.
  expect(cross!.points.map((p) => p.x)).toEqual(pooled!.points.map((p) => p.x));
  expect(pooled!.points).toHaveLength(13);
  // The claim: removing each origin's mean is what makes it flat. Three points against nine.
  expect(cross!.worst.gap).toBeLessThan(0.031);
  expect(pooled!.worst.gap).toBeGreaterThan(0.08);
});

// --- The small multiples, which share one scale inside a set and none across two ---------------

test("a grouped series becomes one panel per group, on a scale that spans every panel", () => {
  const { series } = fixture();
  const rows = series.series[0]!.rows;
  series.series[0]!.rows = [
    { ...rows[0]!, group: "A", label: "Least wealthy", value: 166.55 },
    { ...rows[1]!, group: "A", label: "Wealthiest", value: -83.51 },
    { ...rows[2]!, group: "B", label: "Least wealthy", value: 4.06 },
  ];
  const multiples = multiplesOf(series.series[0]!)!;
  expect(multiples.panels.map((panel) => panel.label)).toEqual(["A", "B"]);
  expect(multiples.panels.map((panel) => panel.bars.length)).toEqual([2, 1]);
  // One scale across the set, taken over every row and not over the panel it lands in: B's lone
  // 4.06 is drawn against A's 166.55, which is what makes the panels comparable at a glance.
  expect(multiples.max).toBe(166.55);
  expect(multiples.min).toBe(-83.51);
  // The panel's own name is in the tooltip, because the tooltip is read with nothing beside it.
  expect(multiples.panels[1]!.bars[0]!.hover).toContain("B — Least wealthy");
});

test("an ungrouped series is not a small multiple, and says so rather than drawing one panel", () => {
  const { series } = fixture();
  expect(multiplesOf(series.series[0]!)).toBeNull();
  // Nor is a series grouped in part: a panel with no name is not a panel.
  series.series[0]!.rows[0]!.group = "A";
  expect(multiplesOf(series.series[0]!)).toBeNull();
});

test("the committed panels are grouped seven ways and scaled once per axis", () => {
  const keys = [
    "project/what-each-anchor-rule-pays-by-wealth",
    "project/what-each-anchor-rule-pays-by-disadvantaged-share",
  ];
  const scales = keys.map((key) => {
    const series = manifest.series.find((s) => s.key === key);
    expect(series, `${key} is not in the manifest`).toBeDefined();
    const multiples = multiplesOf(series!);
    expect(multiples, `${key} is not grouped, so it cannot be drawn as panels`).not.toBeNull();
    expect(multiples!.panels).toHaveLength(7);
    for (const panel of multiples!.panels) expect(panel.bars).toHaveLength(5);
    return multiples!.max;
  });
  // #444's care note: the two axes differ by two orders of magnitude in the first fifth, so one
  // scale across both would flatten the second panel into nothing. They are separate documents
  // precisely so that the scale cannot be shared — asserted here so a later merge cannot.
  expect(scales[0]).not.toBe(scales[1]);
});

test("a panel with nothing in it prints its zeros, and the set reserves the room for them", () => {
  const { series } = fixture();
  const rows = series.series[0]!.rows;
  series.series[0]!.rows = [
    { ...rows[0]!, group: "Does something", label: "Least wealthy", value: 166.55 },
    { ...rows[1]!, group: "Does something", label: "Wealthiest", value: -83.51 },
    { ...rows[2]!, group: "Does nothing", label: "Least wealthy", value: 0 },
  ];
  series.series[0]!.unit = "dollars";
  const multiples = multiplesOf(series.series[0]!)!;
  // The panel that draws bars is not labelled: the bars carry the quantity, and a number on every
  // mark is what `Bar.direct` warns against.
  expect(multiples.panels[0]!.bars.map((bar) => bar.direct)).toEqual([undefined, undefined]);
  // The panel that draws nothing is, because otherwise it draws row names against a zero rule and
  // reads as a chart that failed to render — which is exactly how #444's two inert rules shipped.
  expect(multiples.panels[1]!.bars.map((bar) => bar.direct)).toEqual(["$0"]);
  // And the room those labels want belongs to the set, not to the panel carrying them: a panel
  // with a narrower frame than its siblings draws zero at a different pixel. See `barSpec`.
  expect(multiples.labelChars).toBe(2);
});

test("the two inert anchor rules are labelled on both axes, and only they are", () => {
  for (const key of [
    "project/what-each-anchor-rule-pays-by-wealth",
    "project/what-each-anchor-rule-pays-by-disadvantaged-share",
  ]) {
    const multiples = multiplesOf(manifest.series.find((s) => s.key === key)!)!;
    const labelled = multiples.panels
      .filter((panel) => panel.bars.some((bar) => bar.direct != null))
      .map((panel) => panel.label);
    // `A dated phase-down` pays nothing to any fifth on either axis — the rule is dated out before
    // the year the panel measures — and a panel of five zeros has no ink of its own to show it.
    expect(labelled, key).toEqual(["A dated phase-down"]);
    for (const panel of multiples.panels) {
      const zeroed = panel.bars.every((bar) => bar.value === 0);
      expect(
        panel.bars.every((bar) => (bar.direct != null) === zeroed),
        `${key} — ${panel.label} labels some bars and not others`,
      ).toBe(true);
    }
    expect(multiples.labelChars).toBe(2);
  }
});

// --- The spread, whose whole reading is its census ---------------------------------------------

/**
 * A spread correct in every position, for the mutations below to be cut from.
 *
 * Two panels over one frame, three classes, one boundary, and a census that counts the whole, each
 * panel and the one region neither term carries. The panels are deliberately different sizes: a
 * fixture whose two panels are the same length cannot tell a check that counts panels from one that
 * counts the whole. One subject is unreached, which is the state the fixture exists to hold — a
 * census of 6 under a picture of 5 is the defect, and it is only visible if the sixth is named.
 */
function spreadFixture(): { node: Node; series: SeriesManifest; figures: Manifest } {
  const prose =
    "Of the 5 drawn, 3 lost pupils and 2 did not; 1 sits below the boundary where there is no " +
    "majority to take. [verified] (`crates/project`)";
  const bind = (key: string, value: number, as_written: string) =>
    ({ key, value, field: "description", as_written }) as const;
  const node: Node = {
    id: "scenario/example-spread",
    className: "scenario",
    name: "example-spread",
    label: "Example spread",
    summary: "Which term carries the majority.",
    description: prose,
    linkText: prose,
    properties: [],
    findings: null,
    revisions: [],
    unfilled: [],
    figures: [
      bind("project/drawn", 5, "Of the 5 drawn"),
      bind("project/lost", 3, "3 lost pupils"),
      bind("project/kept", 2, "2 did not"),
      bind("project/neither", 1, "1 sits below the boundary"),
    ],
    series: [{ key: "project/which-term-carries", field: "description" }],
    out: [],
    in: [],
  };
  const mark = (label: string, x: number, y: number, klass: number) => ({
    label,
    x,
    y,
    class: klass,
  });
  const spread: ManifestSpread = {
    key: "project/which-term-carries",
    owner: "crates/project",
    subject: "district",
    label: "Which of the two terms carries the majority",
    x: { label: "Enrollment term", unit: "ratio", min: -1, max: 1, log: false },
    y: { label: "Per-pupil term", unit: "ratio", min: -1, max: 1, log: false },
    classes: ["Neither: the multiple is at or below one", "Enrollment", "Per pupil"],
    boundaries: [{ label: "A multiple of one", from: [-1, 1], to: [1, -1] }],
    census: [
      { label: "Drawn", subjects: 5, figure: "project/drawn" },
      { label: "Lost pupils", subjects: 3, figure: "project/lost" },
      { label: "Kept pupils", subjects: 2, figure: "project/kept" },
      { label: "Neither term carries", subjects: 1, figure: "project/neither" },
    ],
    unreached: [{ label: "Elsewhere Local (IRN 000001)", why: "no funding base published" }],
    panels: [
      {
        label: "Lost pupils",
        marks: [mark("A", 0.4, 0.5, 1), mark("B", -0.6, -0.7, 0), mark("E", -0.1, 0.9, 2)],
      },
      { label: "Kept pupils", marks: [mark("C", 0.2, 0.6, 2), mark("D", 0.5, 0.1, 1)] },
    ],
  };
  const series: SeriesManifest = {
    contract: READS_SERIES_CONTRACT,
    series: [],
    scatters: [],
    planes: [],
    curves: [],
    spreads: [spread],
    bands: [],
  };
  const count = (key: string, value: number, label: string) =>
    ({ key, owner: "crates/project", unit: "count", value, label }) as const;
  const figures: Manifest = {
    contract: READS_CONTRACT,
    figures: [
      count("project/drawn", 5, "Districts the identity reaches"),
      count("project/lost", 3, "Districts that lost pupils"),
      count("project/kept", 2, "Districts that did not"),
      count("project/neither", 1, "Districts below the multiple of one"),
    ],
  };
  return { node, series, figures };
}

test("the spread fixture the mutations are cut from is itself clean", () => {
  const { node, series, figures: mine } = spreadFixture();
  expect(kinds(node, series)).toEqual([]);
  expect(censusAgainstFigures(series, mine)).toEqual([]);
});

test("census-unbound: the node does not bind the count of one region", () => {
  const { node, series } = spreadFixture();
  node.figures = node.figures.filter((f) => f.key !== "project/neither");
  expect(kinds(node, series)).toEqual(["census-unbound"]);
});

test("census-unbound: the node does not bind the whole of the population", () => {
  // The one that matters most and reads least like an omission: every region is bound and the
  // denominator they are all read against is not.
  const { node, series } = spreadFixture();
  node.figures = node.figures.filter((f) => f.key !== "project/drawn");
  expect(kinds(node, series)).toEqual(["census-unbound"]);
});

test("census-unbound: a census bound nowhere is one finding per region", () => {
  const { node, series } = spreadFixture();
  node.figures = [];
  expect(kinds(node, series)).toEqual([
    "census-unbound",
    "census-unbound",
    "census-unbound",
    "census-unbound",
  ]);
});

test("unknown-key: the node draws a spread the manifest does not carry", () => {
  const { node, series } = spreadFixture();
  node.series[0]!.key = "project/which-term-carries-renamed";
  expect(kinds(node, series)).toEqual(["unknown-key", "uncited-series"]);
});

test("unattributed: the node draws a crate's spread and cites that crate nowhere", () => {
  const { node, series } = spreadFixture();
  node.description = node.description.replace(" (`crates/project`)", "");
  node.linkText = node.description;
  expect(kinds(node, series)).toEqual(["unattributed"]);
});

test("uncited-series: the manifest exports a spread no node draws", () => {
  const { node, series } = spreadFixture();
  series.spreads.push({ ...series.spreads[0]!, key: "project/which-term-carries-again" });
  expect(kinds(node, series)).toEqual(["uncited-series"]);
});

test("the two manifests are held to each other, region by figure", () => {
  const { series, figures: mine } = spreadFixture();
  expect(censusAgainstFigures(series, mine)).toEqual([]);
  // A count is an integer, so the comparison is exact rather than to the figure's tolerance: a
  // tolerance here would only be a place for a district to hide.
  series.spreads[0]!.census[3]!.subjects = 2;
  expect(censusAgainstFigures(series, mine)).toHaveLength(1);
  series.spreads[0]!.census[3]!.subjects = 1;
  // A figure the other document lacks, and one on a unit that is not a count of anything.
  series.spreads[0]!.census[1]!.figure = "project/absent";
  expect(censusAgainstFigures(series, mine)).toHaveLength(1);
  series.spreads[0]!.census[1]!.figure = "project/lost";
  mine.figures[1] = { ...mine.figures[1]!, unit: "share" };
  expect(censusAgainstFigures(series, mine)).toHaveLength(1);
});

test("a panel whose size no region counts is a caption nothing pins", () => {
  const { series, figures: mine } = spreadFixture();
  // The whole is checked apart from the panels, because a panel can land on some other region's
  // count by coincidence and be wrong anyway. Drop one mark from the first panel and it draws 2,
  // which "Kept pupils" still counts — but the population is now 4, which nothing counts.
  series.spreads[0]!.panels[0]!.marks.pop();
  expect(censusAgainstFigures(series, mine)).toHaveLength(1);
  // And the other way: empty the second panel and it draws 0, which no region counts, while the
  // population falls to 2, which "Kept pupils" does. One finding again, from the other leg.
  series.spreads[0]!.panels[0]!.marks.push(mark0());
  series.spreads[0]!.panels[1]!.marks = [];
  expect(censusAgainstFigures(series, mine)).toHaveLength(1);
});

/** The mark the test above puts back, so the second leg is measured against a whole population. */
function mark0() {
  return { label: "E", x: -0.1, y: 0.9, class: 2 };
}

test("a spread that names nobody it could not place is refused", () => {
  const { series, figures: mine } = spreadFixture();
  series.spreads[0]!.unreached = [];
  expect(censusAgainstFigures(series, mine)).toHaveLength(1);
});

test("a spread becomes one panel per region, on one frame, with the boundaries on every panel", () => {
  const { series } = spreadFixture();
  const panels = regionsOf(series.spreads[0]!);
  expect(panels.map((panel) => panel.label)).toEqual(["Lost pupils", "Kept pupils"]);
  // One frame, because the panels are two views of one plane and a per-panel domain would move a
  // district between them without moving the district.
  for (const panel of panels) {
    expect(panel.xDomain).toEqual([-1, 1]);
    expect(panel.yDomain).toEqual([-1, 1]);
    expect(panel.rules.map((rule) => rule.label)).toEqual(["A multiple of one"]);
  }
  // The unclassified state is a hue of its own and not an absence — `tokens.ts` keeps the slot.
  expect(panels[0]!.points.map((point) => point.series)).toEqual([
    "formula",
    "neutral",
    "guarantee",
  ]);
  expect(panels[0]!.points[1]!.hover).toContain("Neither: the multiple is at or below one");
});

test("a mark in a hue the legend does not name is refused rather than drawn", () => {
  const { series } = spreadFixture();
  series.spreads[0]!.panels[0]!.marks[0]!.class = 3;
  expect(() => regionsOf(series.spreads[0]!)).toThrow(/is class 3 and the spread declares 3/);
});

test("the committed spreads are the guarantee's two terms, and the empty region is the finding", () => {
  const plane = manifest.spreads.find(
    (spread) => spread.key === "project/the-two-terms-of-every-districts-multiple",
  );
  expect(plane).toBeDefined();
  // Three states, not two: below a multiple of one there is no majority to take, and `origin` is
  // an Option for exactly that reason.
  expect(plane!.classes).toHaveLength(3);
  expect(plane!.panels).toHaveLength(1);
  expect(plane!.panels[0]!.marks).toHaveLength(607);
  // The two the identity does not reach, named by IRN because district names are not unique.
  expect(plane!.unreached).toHaveLength(2);
  for (const missing of plane!.unreached) expect(missing.label).toMatch(/\(IRN \d{6}\)/);
  // Both terms are signed the way the axis labels read them: positive x is fewer pupils than in
  // FY2020, positive y is the FY2027 formula paying less per pupil than the FY2020 regime did.
  const below = plane!.panels[0]!.marks.filter((mark) => mark.class === 0);
  expect(below).toHaveLength(314);
  // Class 0 is the region below the boundary, and it is that region exactly: the sum of the two
  // terms is the log of the multiple, so `class === 0` and `x + y <= 0` are the same 314.
  expect(below.every((mark) => mark.x + mark.y <= 0)).toBe(true);
  // The empty corner, which is the finding: a formula district that both lost pupils and is paid
  // less per pupil than FY2020. The four that are paid less all grew, so all four sit left of the
  // enrollment axis and none of them here.
  expect(below.filter((mark) => mark.y > 0)).toHaveLength(4);
  expect(below.filter((mark) => mark.y > 0 && mark.x > 0)).toHaveLength(0);

  const wealth = manifest.spreads.find(
    (spread) => spread.key === "project/the-two-terms-by-wealth-among-those-that-lost-pupils",
  );
  expect(wealth).toBeDefined();
  // Five fifths of the 514, and the same frame as the plane above — two views of one scatter.
  expect(wealth!.panels).toHaveLength(5);
  expect(wealth!.panels.reduce((sum, panel) => sum + panel.marks.length, 0)).toBe(514);
  // Two views of one scatter: the same axes, the same three classes, and one frame across all
  // five panels — a per-panel frame would move a district between fifths without moving it.
  expect(wealth!.classes).toEqual(plane!.classes);
  expect(wealth!.x.label).toBe(plane!.x.label);
  expect(wealth!.y.label).toBe(plane!.y.label);
  const drawn = regionsOf(wealth!);
  for (const panel of drawn) {
    expect(panel.xDomain).toEqual([wealth!.x.min, wealth!.x.max]);
    expect(panel.yDomain).toEqual([wealth!.y.min, wealth!.y.max]);
  }
  // Every district in this spread lost pupils, which is the population its census counts.
  expect(wealth!.panels.every((panel) => panel.marks.every((mark) => mark.x > 0))).toBe(true);
});

// --- The band chart, read at its two ends and at the marker ------------------------------------

/**
 * A band chart correct in every position, for the mutations below to be cut from.
 *
 * Four rows, one marker and an aggregate, which is the smallest fixture that can tell the rules
 * apart: a two-row chart is all ends and cannot show that the middle is deliberately unpinned, and
 * a chart with no marker cannot show that a marker is pinned while a boundary never is. The
 * unreached list is the shape #447 gives it — a floor the source has no column for, rather than a
 * subject the computation failed to place — and it is populated because an empty one is refused.
 */
function bandFixture(): { node: Node; series: SeriesManifest; figures: Manifest } {
  const prose =
    "The smallest fifth is funded for 5.23 and employs 6.25; the largest is funded for 20.69 " +
    "and employs 39.00. The floor binds under 1,500 pupils, and across the state 1.9664 are " +
    "employed for every one funded. [verified] (`crates/project`)";
  const bind = (key: string, value: number, as_written: string) =>
    ({ key, value, field: "description", as_written }) as const;
  const node: Node = {
    id: "scenario/example-band",
    className: "scenario",
    name: "example-band",
    label: "Example band",
    summary: "What is employed against what is funded.",
    description: prose,
    linkText: prose,
    properties: [],
    findings: null,
    revisions: [],
    unfilled: [],
    figures: [
      bind("project/funded-small", 5.23, "funded for 5.23"),
      bind("project/employed-small", 6.25, "employs 6.25"),
      bind("project/funded-large", 20.69, "funded for 20.69"),
      bind("project/employed-large", 39, "employs 39.00"),
      bind("project/threshold", 1500, "under 1,500 pupils"),
      bind("project/multiple", 1.9664, "1.9664 are employed for every one funded"),
    ],
    series: [{ key: "project/employed-against-funded", field: "description" }],
    out: [],
    in: [],
  };
  const span = (label: string, low: number, high: number) => ({
    label,
    hover: `${label}: funded for ${low}, employing ${high}`,
    low,
    high,
  });
  const band: ManifestBand = {
    key: "project/employed-against-funded",
    owner: "crates/project",
    subject: "sextile",
    label: "What districts employ against what the plan funds",
    ends: ["What the plan funds", "What the district employs"],
    measure: {
      label: "Administrators, in full-time equivalents",
      unit: "positions",
      min: 5.23,
      max: 39,
      log: true,
    },
    spans: [
      {
        ...span("552 pupils", 5.23, 6.25),
        lowFigure: "project/funded-small",
        highFigure: "project/employed-small",
      },
      span("886 pupils", 5.97, 9.7),
      span("1,663 pupils", 7.91, 15),
      {
        ...span("5,256 pupils", 20.69, 39),
        lowFigure: "project/funded-large",
        highFigure: "project/employed-large",
      },
    ],
    markers: [
      {
        label: "1,500 pupils: under it the floor funds two administrators at any enrolment.",
        value: 1500,
        at: 2.25,
        figure: "project/threshold",
      },
    ],
    whole: {
      label: "1.9664 employed for every one funded.",
      value: 1.9664,
      figure: "project/multiple",
    },
    unreached: [
      { label: "R.C. 3317.011(E)(1), guidance counselors", why: "no column of the report counts it" },
    ],
  };
  const series: SeriesManifest = {
    contract: READS_SERIES_CONTRACT,
    series: [],
    scatters: [],
    planes: [],
    curves: [],
    spreads: [],
    bands: [band],
  };
  const at = (key: string, unit: "positions" | "count" | "ratio", value: number, label: string) =>
    ({ key, owner: "crates/project", unit, value, label }) as const;
  const figures: Manifest = {
    contract: READS_CONTRACT,
    figures: [
      at("project/funded-small", "positions", 5.23, "Funded in the smallest sextile"),
      at("project/employed-small", "positions", 6.25, "Employed in the smallest sextile"),
      at("project/funded-large", "positions", 20.69, "Funded in the largest sextile"),
      at("project/employed-large", "positions", 39, "Employed in the largest sextile"),
      at("project/threshold", "count", 1500, "The floor's own threshold in pupils"),
      at("project/multiple", "ratio", 1.9664, "Employed for every one funded"),
    ],
  };
  return { node, series, figures };
}

test("the band fixture the mutations are cut from is itself clean", () => {
  const { node, series, figures: mine } = bandFixture();
  expect(kinds(node, series)).toEqual([]);
  expect(bandsAgainstFigures(series, mine)).toEqual([]);
});

test("band-unbound: the node does not bind one value of one end row", () => {
  const { node, series } = bandFixture();
  node.figures = node.figures.filter((f) => f.key !== "project/employed-large");
  expect(kinds(node, series)).toEqual(["band-unbound"]);
});

test("band-unbound: an end row is two numbers, so dropping the row is two findings", () => {
  const { node, series } = bandFixture();
  node.figures = node.figures.filter(
    (f) => f.key !== "project/funded-small" && f.key !== "project/employed-small",
  );
  expect(kinds(node, series)).toEqual(["band-unbound", "band-unbound"]);
});

test("band-unbound: the middle rows are not bound and are not meant to be", () => {
  // The rule is the first and last row, not every row. A chart of six rows bound twelve times
  // would put ten numbers in the prose that nothing is read off.
  const { node, series } = bandFixture();
  series.bands[0]!.spans[1] = {
    ...series.bands[0]!.spans[1]!,
    lowFigure: "project/nothing-binds-this",
    highFigure: "project/nor-does-anything-bind-this",
  };
  expect(kinds(node, series)).toEqual([]);
});

test("band-unbound: the marker is a statutory parameter and is pinned", () => {
  const { node, series } = bandFixture();
  node.figures = node.figures.filter((f) => f.key !== "project/threshold");
  expect(kinds(node, series)).toEqual(["band-unbound"]);
});

test("band-unbound: the aggregate is on no row, so nothing else could pin it", () => {
  const { node, series } = bandFixture();
  node.figures = node.figures.filter((f) => f.key !== "project/multiple");
  expect(kinds(node, series)).toEqual(["band-unbound"]);
});

test("band-unbound: an end row the manifest left unpinned is reported, not skipped", () => {
  const { node, series } = bandFixture();
  const last = series.bands[0]!.spans[3]!;
  series.bands[0]!.spans[3] = {
    label: last.label,
    hover: last.hover,
    low: last.low,
    high: last.high,
    lowFigure: last.lowFigure!,
  };
  expect(kinds(node, series)).toEqual(["band-unbound"]);
});

test("unknown-key: the node draws a band chart the manifest does not carry", () => {
  const { node, series } = bandFixture();
  node.series[0]!.key = "project/employed-against-funded-renamed";
  expect(kinds(node, series)).toEqual(["unknown-key", "uncited-series"]);
});

test("unattributed: the node draws a crate's band chart and cites that crate nowhere", () => {
  const { node, series } = bandFixture();
  node.description = node.description.replace(" (`crates/project`)", "");
  node.linkText = node.description;
  expect(kinds(node, series)).toEqual(["unattributed"]);
});

test("uncited-series: the manifest exports a band chart no node draws", () => {
  const { node, series } = bandFixture();
  series.bands.push({ ...series.bands[0]!, key: "project/employed-against-funded-again" });
  expect(kinds(node, series)).toEqual(["uncited-series"]);
});

test("the two manifests are held to each other at both ends, the marker and the caption", () => {
  const { series, figures: mine } = bandFixture();
  expect(bandsAgainstFigures(series, mine)).toEqual([]);
  // A value that has moved on one side and not the other, at each of the four positions the rule
  // names: an end row's low, an end row's high, the marker, and the aggregate.
  series.bands[0]!.spans[0]!.low = 5.24;
  expect(bandsAgainstFigures(series, mine)).toHaveLength(1);
  series.bands[0]!.spans[0]!.low = 5.23;
  series.bands[0]!.spans[3]!.high = 39.5;
  expect(bandsAgainstFigures(series, mine)).toHaveLength(1);
  series.bands[0]!.spans[3]!.high = 39;
  series.bands[0]!.markers[0]!.value = 1250;
  expect(bandsAgainstFigures(series, mine)).toHaveLength(1);
  series.bands[0]!.markers[0]!.value = 1500;
  series.bands[0]!.whole.value = 1.97;
  expect(bandsAgainstFigures(series, mine)).toHaveLength(1);
});

test("an end of a band drawn on a unit the figure is not is refused", () => {
  const { series, figures: mine } = bandFixture();
  // A figure the other document lacks, and one whose unit is not the axis's.
  series.bands[0]!.spans[0]!.lowFigure = "project/absent";
  expect(bandsAgainstFigures(series, mine)).toHaveLength(1);
  series.bands[0]!.spans[0]!.lowFigure = "project/funded-small";
  mine.figures[0] = { ...mine.figures[0]!, unit: "count" };
  expect(bandsAgainstFigures(series, mine)).toHaveLength(1);
});

test("a band chart that names nothing its source cannot see is refused", () => {
  const { series, figures: mine } = bandFixture();
  series.bands[0]!.unreached = [];
  expect(bandsAgainstFigures(series, mine)).toHaveLength(1);
});

test("a band becomes one row per span, with the marker carried beside them", () => {
  const { series } = bandFixture();
  const drawn = rangesOf(series.bands[0]!);
  expect(drawn.rows.map((row) => row.label)).toEqual([
    "552 pupils",
    "886 pupils",
    "1,663 pupils",
    "5,256 pupils",
  ]);
  expect(drawn.rows[0]!.low).toBe(5.23);
  expect(drawn.rows[0]!.high).toBe(6.25);
  // The axis is logarithmic and says so, which is the whole argument for the form: a row's length
  // is the ratio between its ends only where the scale is.
  expect(drawn.axis.log).toBe(true);
  expect(drawn.axis.format(6.25)).toBe("6.25");
  // The marker's position is in rows and fractional, because "16 districts into the fourth band"
  // is not a row boundary and rounding it to one would move the claim.
  expect(drawn.markers).toHaveLength(1);
  expect(drawn.markers[0]!.at).toBe(2.25);
});

test("the committed band chart is the administrator floor, drawn against what is employed", () => {
  const band = manifest.bands.find(
    (entry) => entry.key === "project/administrators-employed-against-administrators-funded",
  );
  expect(band).toBeDefined();
  // Six sextiles of the ADM distribution, and one marker: the floor's own threshold.
  expect(band!.spans).toHaveLength(6);
  expect(band!.markers).toHaveLength(1);
  expect(band!.markers[0]!.value).toBe(1500);
  // The finding, in the two numbers it is made of: districts employ about twice what the build-up
  // funds, and the gap is narrowest in the smallest sextile, where the floor binds.
  expect(band!.whole.value).toBeGreaterThan(1.9);
  expect(band!.whole.value).toBeLessThan(2);
  const multiple = (span: { low: number; high: number }) => span.high / span.low;
  expect(multiple(band!.spans[0]!)).toBeLessThan(multiple(band!.spans[5]!));
  // Six of R.C. 3317.011's seven binding floors meet no column of the District Profile Report, and
  // they are named beside the chart rather than left out of it — #447's more careful half.
  expect(band!.unreached).toHaveLength(6);
  for (const missing of band!.unreached) expect(missing.label).toMatch(/^R\.C\. 3317\.011\(/);
});
