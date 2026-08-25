/**
 * The corpus-to-crate cross-check, and the gate broken on purpose in each position it can fail in.
 *
 * # Why the mutations are half this file
 *
 * #131 asks for it, and #125 is why. Sixteen checks in this repository were green against the
 * defect they were written for — including two that made `//:generated` unfalsifiable — and **every
 * one was found by mutation rather than by reading**. A check that has only ever seen correct input
 * has not been shown to reject anything.
 *
 * So each `DiscrepancyKind` below is produced deliberately, from a node that is correct except in
 * that one position, and asserted to be the *only* thing reported. `crossCheck` is pure over its
 * two arguments so that this is possible without touching the corpus on disk.
 *
 * # And why the real corpus is checked too
 *
 * Because a mutation suite passing over synthetic input proves the checker works and says nothing
 * about whether it is pointed at anything. The first assertion runs it over all 135 nodes and the
 * committed manifest, and the two coverage floors below stop the answer from being "nothing is
 * bound, so nothing disagrees".
 */

import { expect, test } from "vitest";

import { loadCorpus, type Node } from "../../src/lib/corpus.ts";
import {
  claimTags,
  crossCheck,
  flatten,
  loadFigureManifest,
  numerals,
  READS_CONTRACT,
  type Discrepancy,
  type DiscrepancyKind,
  type Manifest,
} from "../../src/lib/corpusFigures.ts";

const corpus = loadCorpus();
const manifest = loadFigureManifest();
const bindings = corpus.nodes.flatMap((node) => node.figures);

test("every bound figure in the corpus agrees with the crate it cites", () => {
  const found = crossCheck(corpus.nodes, manifest);
  expect(
    found.map((entry) => `[${entry.kind}] ${entry.node} — ${entry.message}`),
    "the corpus and crates/figures.json have come apart",
  ).toEqual([]);
});

/**
 * The coverage floor, set at the value rather than below it.
 *
 * A ratchet whose floor sits under the current count cannot ratchet: it permits exactly as much
 * regression as the slack it was given. These are the counts today, and the only edit they admit is
 * upward. Raise them in the same commit that adds a binding.
 *
 * The figure count is not the interesting one — `uncited-figure` already makes an unbound export a
 * failure, so the manifest cannot grow without the corpus. The binding count is: it is what stops a
 * node from dropping a `figures:` entry to make a red gate green.
 */
test("the corpus binds no fewer figures than it did", () => {
  expect(bindings.length, "154 bindings; raise this when you add one").toBeGreaterThanOrEqual(154);
  expect(
    corpus.nodes.filter((node) => node.figures.length > 0).length,
    "24 nodes carry bindings; raise this when a twenty-fifth does",
  ).toBeGreaterThanOrEqual(24);
  expect(
    new Set(bindings.map((binding) => binding.key)).size,
    "every figure the manifest exports is bound by some node",
  ).toBe(manifest.figures.length);
});

/**
 * The figures that matter most are bound in **every** node that states them.
 *
 * This is the finding the whole mechanism comes from. `316`/`290` — the headline distributional
 * claim — is written in three nodes, and the `recognized-valuation` correction reached three of six
 * carriers, leaving two of them publishing the reversed sign under `[verified]`. One carrier bound
 * and two loose would reproduce exactly that: a green gate over a corrected node beside two stale
 * ones.
 */
test("the charge-off comparison is bound in all three nodes that state it", () => {
  const carriers = (key: string) =>
    corpus.nodes.filter((node) => node.figures.some((figure) => figure.key === key)).map((n) => n.id);

  expect(carriers("regime-diff/districts-the-charge-off-pays-more").sort()).toEqual([
    "formula-component/charge-off-local-share",
    "formula-component/fsfp-local-capacity-measure",
  ]);
  expect(carriers("regime-diff/charge-off-zeroes-base-cost-aid").sort()).toEqual([
    "formula-component/charge-off-local-share",
    "parameter/local-share-charge-off-millage",
  ]);
  expect(carriers("regime-diff/median-shortfall-under-the-plan").sort()).toEqual([
    "formula-component/charge-off-local-share",
    "formula-component/fsfp-local-capacity-measure",
  ]);
});

test("the manifest declares the contract this check reads", () => {
  expect(manifest.contract).toBe(READS_CONTRACT);
  expect(manifest.figures.length).toBeGreaterThan(0);
  for (const figure of manifest.figures) {
    expect(figure.owner, `${figure.key} is owned by a crate`).toMatch(/^crates\/[a-z-]+$/);
    expect(Number.isFinite(figure.value), `${figure.key} has a value`).toBe(true);
  }
});

// --- Reading a number out of a sentence -------------------------------------------------------

test("a numeral is read at the precision it is written to", () => {
  // The precision rule, which is the only thing here that needs no per-figure judgement: `$5.1
  // million` claims a figure to the nearest hundred thousand, `$674,561` claims it to the dollar,
  // and each is held to what it claims.
  expect(numerals("$5.1 million withheld", "dollars")).toEqual([
    { value: 5_100_000, precision: 50_000 },
  ]);
  expect(numerals("had $674,561 taken off", "dollars")).toEqual([{ value: 674_561, precision: 0.5 }]);
  expect(numerals("8.20% below total taxable value", "share")).toEqual([
    { value: 0.082, precision: 0.00005 },
  ]);
  expect(numerals("$34.5bn", "dollars")).toEqual([{ value: 34.5e9, precision: 50_000_000 }]);
  expect(numerals("65 of 606 districts", "count")).toEqual([
    { value: 65, precision: 0.5 },
    { value: 606, precision: 0.5 },
  ]);
});

test("a numeral in the wrong unit is not read at all", () => {
  // Rejected rather than coerced. `43 districts, $5.1 million withheld` binds twice — once as a
  // count and once as dollars — and each binding must see only its own number, or the phrase would
  // satisfy a count binding with the dollar figure beside it.
  expect(numerals("43 districts, $5.1 million withheld", "count")).toEqual([
    { value: 43, precision: 0.5 },
  ]);
  expect(numerals("43 districts, $5.1 million withheld", "dollars")).toEqual([
    { value: 43, precision: 0.5 },
    { value: 5_100_000, precision: 50_000 },
  ]);
  // A share is a fraction of one, so a dollar amount in the same sentence is not a candidate for it.
  expect(numerals("$4,448 of it — 46%", "share")).toEqual([{ value: 0.46, precision: 0.005 }]);
  expect(numerals("46%", "dollars")).toEqual([]);
});

test("a scale word is a scale word and not the start of the next one", () => {
  // `m` is millions; `metres` is not. The token boundary is what separates them, and without it
  // "65 more districts" reads as sixty-five million.
  expect(numerals("65 more districts", "count")).toEqual([{ value: 65, precision: 0.5 }]);
  expect(numerals("$793m of charge-off", "dollars")).toEqual([
    { value: 793e6, precision: 500_000 },
  ]);
});

/**
 * A rank is not readable, in either form the corpus writes one.
 *
 * Recorded as a test rather than only as prose in `.yidam/corpus/README.md`, because it is the
 * reason four of `litigation/derolph-i-1997`'s figures are stated and not bound, and a future
 * reader is otherwise entitled to assume nobody tried.
 *
 * A spelled-out ordinal has no digits at all. A digit ordinal has digits and still yields
 * nothing: the `th` follows `25` with no token boundary between them, so the optional scale
 * group cannot close and the numeral is dropped. That is the same `\b` that stops `65 more
 * districts` from reading as sixty-five million — the rule is right and this is its cost.
 */
test("a rank is not a numeral this check can read, spelled or in digits", () => {
  expect(numerals("seventh highest of fifty-one", "count")).toEqual([]);
  expect(numerals("ranks Ohio 25th of 51", "count")).toEqual([{ value: 51, precision: 0.5 }]);
});

/**
 * A count spelled as a word can bind to the wrong numeral and pass.
 *
 * The hazard that made `parameter/twenty-mill-floor` state its counts in digits. `Twenty
 * districts report an effective Class 1 rate below 20 mills` bound cleanly to a figure of 20 —
 * on the `20` that meant *mills*. One numeral matched, so `phrase-ambiguous` had nothing to
 * report, and the binding was satisfied by a quantity in the wrong dimension.
 *
 * There is no check that catches this: the reader cannot know which of two identical numbers a
 * sentence means. What the corpus does instead is write a computed count in digits, so the
 * numeral that matches is the one the binding is about.
 */
test("a phrase can satisfy a binding with a number that means something else", () => {
  expect(numerals("Twenty districts report a rate below 20 mills", "count")).toEqual([
    { value: 20, precision: 0.5 },
  ]);
  expect(numerals("20 of the 606 report a rate below the floor", "count")).toEqual([
    { value: 20, precision: 0.5 },
    { value: 606, precision: 0.5 },
  ]);
});

test("prose re-wrapped across lines is the same prose", () => {
  // YAML block scalars wrap at whatever column the author left them at, and re-wrapping a
  // paragraph is not a change to what it says.
  expect(flatten("Quartiling the 611\ncomparable districts")).toBe(
    "Quartiling the 611 comparable districts",
  );
});

test("a claim tag is read out of markdown, in every form the corpus writes one", () => {
  expect(claimTags("[verified]")).toEqual([{ tag: "verified", detail: "" }]);
  expect(claimTags("[verified — `crates/regime-diff`]")).toEqual([
    { tag: "verified", detail: " — `crates/regime-diff`" },
  ]);
  // A justification that is itself a markdown link, which is how half the crate citations are
  // written. The inner brackets must not close the tag.
  expect(
    claimTags("[verified — [`crates/dispersion`](../../../crates/dispersion/tests/cupp_fy24.rs)]"),
  ).toHaveLength(1);
  expect(claimTags("Reported as lines [L] and [M]")).toEqual([]);
});

// --- The gate, broken on purpose in each position ---------------------------------------------

/** A node that is correct in every position, and the manifest it is correct against. */
function fixture(): { node: Node; manifest: Manifest } {
  const node: Node = {
    id: "formula-component/example",
    className: "formula-component",
    name: "example",
    label: "Example",
    summary: "65 of 606 districts, and nothing else.",
    description: "65 of 606 districts would receive nothing. [verified — `crates/regime-diff`]",
    linkText: "65 of 606 districts would receive nothing. [verified — `crates/regime-diff`]",
    properties: [],
    findings: null,
    revisions: [],
    unfilled: [],
    figures: [
      {
        key: "regime-diff/zeroed",
        value: 65,
        field: "description",
        as_written: "65 of 606 districts",
      },
    ],
    out: [],
    in: [],
  };
  return {
    node,
    manifest: {
      contract: READS_CONTRACT,
      figures: [
        {
          key: "regime-diff/zeroed",
          owner: "crates/regime-diff",
          unit: "count",
          value: 65,
          label: "Districts the charge-off would zero",
        },
      ],
    },
  };
}

/** Run the check over one node and report only the kinds it found. */
function kinds(node: Node, manifest: Manifest): DiscrepancyKind[] {
  return crossCheck([node], manifest).map((entry: Discrepancy) => entry.kind);
}

test("the fixture the mutations are cut from is itself clean", () => {
  // Without this, a mutation test proves nothing: every one of them would also pass against a
  // fixture that was already failing for an unrelated reason.
  const { node, manifest: mine } = fixture();
  expect(crossCheck([node], mine)).toEqual([]);
});

test("a binding on a key the manifest does not carry is caught", () => {
  const { node, manifest: mine } = fixture();
  node.figures[0]!.key = "regime-diff/renamed";
  expect(kinds(node, mine)).toEqual(["unknown-key", "uncited-figure"]);
});

test("a value the crate does not compute is caught", () => {
  // The leg #131 asks for. The corpus says 65, the crate now says 70, and the node is stale.
  const { node, manifest: mine } = fixture();
  mine.figures[0]!.value = 70;
  expect(kinds(node, mine)).toEqual(["value-disagrees"]);
});

test("a count is compared exactly, so a value off by one is caught", () => {
  const { node, manifest: mine } = fixture();
  node.figures[0]!.value = 66;
  expect(kinds(node, mine)).toEqual(["value-disagrees"]);
});

test("a binding pointed at a field the node does not have is caught", () => {
  const { node, manifest: mine } = fixture();
  node.figures[0]!.field = "findings";
  expect(kinds(node, mine)).toEqual(["field-missing"]);
});

test("a phrase edited out of the prose is caught", () => {
  // The correction that reached the binding and not the paragraph. Reader-facing text still says
  // the old thing; without this leg the gate is green over it.
  const { node, manifest: mine } = fixture();
  node.description = node.description.replace("65 of 606", "70 of 606");
  expect(kinds(node, mine)).toEqual(["phrase-missing"]);
});

test("a phrase that quotes a different number from the one it is bound to is caught", () => {
  const { node, manifest: mine } = fixture();
  node.description += " Against 606 on the other base. [verified]";
  node.figures[0]!.as_written = "Against 606 on the other base";
  expect(kinds(node, mine)).toEqual(["phrase-disagrees"]);
});

test("a phrase where two numbers are the bound value is caught as ambiguous", () => {
  // Not pedantry. A phrase carrying the value twice does not say which occurrence it binds, so an
  // edit to one of them passes on the strength of the other.
  const { node, manifest: mine } = fixture();
  node.description = "65 districts, of which 65 receive nothing. [verified — `crates/regime-diff`]";
  node.figures[0]!.as_written = "65 districts, of which 65 receive nothing";
  expect(kinds(node, mine)).toEqual(["phrase-ambiguous"]);
});

test("a figure bound into a field that claims nothing is caught", () => {
  const { node, manifest: mine } = fixture();
  node.description = "65 of 606 districts would receive nothing.";
  node.linkText = node.description;
  // Both, and in this order: the tag carried the crate citation as well as the confidence.
  expect(kinds(node, mine)).toEqual(["unattributed", "field-unverified"]);
});

test("a summary is exempt from the claim tag and only a summary is", () => {
  const { node, manifest: mine } = fixture();
  node.figures[0]!.field = "summary";
  node.figures[0]!.as_written = "65 of 606 districts";
  expect(kinds(node, mine)).toEqual([]);
});

test("a node that binds a crate's figure and never cites that crate is caught", () => {
  const { node, manifest: mine } = fixture();
  const loose = "65 of 606 districts would receive nothing. [verified — the department's sheet]";
  node.description = loose;
  node.linkText = loose;
  expect(kinds(node, mine)).toEqual(["unattributed"]);
});

test("a figure no node binds is caught, so the manifest cannot grow past the corpus", () => {
  const { node, manifest: mine } = fixture();
  mine.figures.push({
    key: "regime-diff/nobody-quotes-this",
    owner: "crates/regime-diff",
    unit: "count",
    value: 1,
    label: "A figure exported and never cited",
  });
  expect(kinds(node, mine)).toEqual(["uncited-figure"]);
});

test("the same phrase in two nodes is two bindings, and one going stale fails alone", () => {
  /*
   * The blast-radius property, stated as a test rather than as a hope.
   *
   * Two nodes state the same figure. The crate moves. Correcting one of them must leave the gate
   * red — that is the whole difference between this mechanism and the `[verified — crates/X]`
   * string it replaces, which went green after three of six carriers were fixed.
   */
  const { node, manifest: mine } = fixture();
  const second: Node = { ...node, id: "parameter/example-two", figures: [{ ...node.figures[0]! }] };

  mine.figures[0]!.value = 70;
  expect(crossCheck([node, second], mine).map((entry) => entry.node)).toEqual([
    "formula-component/example",
    "parameter/example-two",
  ]);

  // One carrier corrected, in both its value and its prose. The other still fails.
  node.figures[0]!.value = 70;
  node.description = node.description.replace("65 of 606", "70 of 606");
  node.figures[0]!.as_written = "70 of 606 districts";
  expect(crossCheck([node, second], mine).map((entry) => entry.node)).toEqual([
    "parameter/example-two",
  ]);
});
