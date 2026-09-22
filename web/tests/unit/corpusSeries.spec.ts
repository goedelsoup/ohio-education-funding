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
  crossCheckSeries,
  endpoints,
  formatRow,
  loadSeriesManifest,
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

test("the two committed manifests agree on every row that names a figure", () => {
  expect(rowsAgainstFigures(manifest, figures)).toEqual([]);
});

/**
 * The coverage floor, at the value and not under it, on the figure ratchet's standing rule:
 * recount with the expressions below rather than incrementing the last stated number.
 *
 * One series and one carrier: `dispersion/fy2016-step-by-business-class` on
 * `education-agency/toledo-city`, which is #416's finding drawn as the four bars it was found by.
 */
test("the corpus draws no fewer series than it did", () => {
  const drawn = corpus.nodes.flatMap((node) => node.series);
  expect(drawn.length, "1 series binding; raise this when you add one").toBeGreaterThanOrEqual(1);
  expect(
    corpus.nodes.filter((node) => node.series.length > 0).length,
    "1 node draws a series; raise this when a second does",
  ).toBeGreaterThanOrEqual(1);
  expect(
    new Set(drawn.map((binding) => binding.key)).size,
    "every series the manifest exports is drawn by some node",
  ).toBe(manifest.series.length);
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
