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
  barsOf,
  crossCheckPageSeries,
  crossCheckSeries,
  endpoints,
  PAGE_SERIES,
  ranksOf,
  fitsAgainstFigures,
  formatRow,
  loadSeriesManifest,
  type ManifestSeries,
  panelsOf,
  READS_SERIES_CONTRACT,
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

/**
 * The coverage floor, at the value and not under it, on the figure ratchet's standing rule:
 * recount with the expressions below rather than incrementing the last stated number.
 *
 * One series and one carrier: `dispersion/fy2016-step-by-business-class` on
 * `education-agency/toledo-city`, which is #416's finding drawn as the four bars it was found by.
 *
 * Every computation the manifest exports is drawn *somewhere*, and there are two somewheres: a
 * node's `series:` block, and a page declared in `PAGE_SERIES`. The sum is the assertion, because
 * the defect it exists to catch — a column computed for nobody — is the same defect whichever of
 * the two is meant to be drawing it.
 */
test("the corpus draws no fewer series than it did", () => {
  const drawn = corpus.nodes.flatMap((node) => node.series);
  expect(drawn.length, "1 series binding; raise this when you add one").toBeGreaterThanOrEqual(1);
  expect(
    corpus.nodes.filter((node) => node.series.length > 0).length,
    "1 node draws a series; raise this when a second does",
  ).toBeGreaterThanOrEqual(1);
  expect(
    new Set([...drawn.map((binding) => binding.key), ...Object.keys(PAGE_SERIES)]).size,
    "every series and cloud the manifest exports is drawn by some node or page",
  ).toBe(manifest.series.length + manifest.scatters.length);
  expect(
    manifest.scatters.length,
    "1 cloud: project/what-the-capacity-tiers-index-measures on formula-component/" +
      "fsfp-targeted-assistance; raise this when you add one",
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
  // A check over an empty table passes against any corpus. One page draws a series today:
  // `project/bounds-census` on `/bounds`. Raise this when a second does.
  const entries = Object.entries(PAGE_SERIES);
  expect(entries.length, "1 page draws a series; raise this when a second does").toBeGreaterThanOrEqual(1);
  for (const [key, source] of entries) {
    expect(source.route, key).toMatch(/^\//);
    expect(source.node, key).toMatch(/^[a-z0-9-]+\/[a-z0-9-]+$/);
  }
});

test("unknown-key: a page declares a series crates/series.json does not carry", () => {
  const without: SeriesManifest = {
    ...manifest,
    series: manifest.series.filter((series) => !(series.key in PAGE_SERIES)),
  };
  const found = kindsOfPages(corpus.nodes, without);
  expect(found).toEqual(Object.keys(PAGE_SERIES).map(() => "unknown-key"));
});

test("page-source-missing: the node a page cites for its endpoints is not in the corpus", () => {
  const without = corpus.nodes.filter((node) => !pageSources.has(node.id));
  expect(kindsOfPages(without, manifest)).toEqual([...pageSources].map(() => "page-source-missing"));
});

test("endpoints-unbound: the node a page cites binds nothing at the chart's ends", () => {
  const stripped = corpus.nodes.map((node) =>
    pageSources.has(node.id) ? { ...node, figures: [] } : node,
  );
  const found = kindsOfPages(stripped, manifest);
  expect(found.length, "a stripped source node reports every endpoint").toBeGreaterThan(0);
  expect(new Set(found)).toEqual(new Set(["endpoints-unbound"]));
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
