/**
 * The ontology's labels, held from the declaration end. The build end is `tests/dist/wikiKeys.spec.ts`.
 *
 * The rule is `.yidam/decisions/a-key-is-not-a-label.yml`: every property and edge declaration
 * says what a reader is shown in place of its key, one relationship slug reads the same wherever it
 * is declared, and a key nothing declares is read as words — never printed as itself (#551).
 */

import { expect, test } from "vitest";

import { humanizeKey, loadCorpus, propertyLabel, relationshipLabel } from "../../src/lib/corpus.ts";
import { shownProperties } from "../../src/lib/prose.ts";

/** A key as a reader would never write it: lowercase words joined by underscores. */
const SNAKE = /^[a-z]+(_[a-z]+)+$/;

test("every property and edge declaration carries a label that is not its key", () => {
  const unlabelled: string[] = [];
  for (const entry of loadCorpus().classes) {
    for (const property of entry.properties) {
      const { name, label } = property;
      if (label === "" || label === name || SNAKE.test(label)) {
        unlabelled.push(`${entry.className}.${name}: ${JSON.stringify(label)}`);
      }
    }
    for (const edge of entry.edges) {
      const { relationship, label } = edge;
      if (label === "" || label === relationship || /^[a-z]+(-[a-z]+)+$/.test(label)) {
        unlabelled.push(`${entry.className} ${relationship}: ${JSON.stringify(label)}`);
      }
    }
  }
  expect(unlabelled).toEqual([]);
});

test("one relationship slug has one label, wherever it is declared", () => {
  /*
   * An inbound edge is named by its slug alone — the page it is rendered on is the target's, and
   * the declaration is the source class's — so two labels for one slug would make the same edge
   * read two ways depending on which end the reader is standing at.
   */
  const labels = new Map<string, Set<string>>();
  let declared = 0;
  for (const entry of loadCorpus().classes) {
    for (const edge of entry.edges) {
      declared += 1;
      labels.set(edge.relationship, (labels.get(edge.relationship) ?? new Set()).add(edge.label));
    }
  }
  expect(declared).toBeGreaterThan(labels.size); // a slug declared twice exists, or this is vacuous
  const split = [...labels].filter(([, set]) => set.size > 1).map(([slug, set]) => `${slug}: ${[...set].join(" | ")}`);
  expect(split).toEqual([]);
});

test("every key a node page shows reads as words, declared or not", () => {
  /*
   * The fallback is what reaches the page for the undeclared relationships, the two universal
   * ones, and the observation properties matched by pattern. Asked of every key the corpus
   * actually uses, not of a list someone remembered to write.
   */
  const corpus = loadCorpus();
  const printed: string[] = [];
  const slugs = new Set<string>();
  for (const node of corpus.nodes) {
    for (const property of shownProperties(node.properties)) {
      const label = propertyLabel(node.className, property.shownAs);
      if (label === property.shownAs || SNAKE.test(label)) printed.push(`${node.id} ${property.shownAs}`);
    }
    for (const edge of node.out) if (edge.relationship) slugs.add(edge.relationship);
    for (const edge of node.in) if (edge.relationship) slugs.add(edge.relationship);
  }
  expect(slugs.size).toBeGreaterThan(100);
  for (const slug of slugs) {
    const label = relationshipLabel(slug);
    if (label === slug) printed.push(`relationship ${slug}`);
  }
  expect(printed).toEqual([]);
});

test("the fallback keeps a leading fiscal year in the key's own form", () => {
  expect(humanizeKey("fy2027_scale")).toBe("FY2027 scale");
  expect(humanizeKey("fy2026_27_appropriation")).toBe("FY2026-27 appropriation");
  expect(humanizeKey("recovered-funds-from")).toBe("Recovered funds from");
  expect(humanizeKey("instance-of")).toBe("Instance of");
});

test("a declared label wins over the fallback, and an undeclared key falls back", () => {
  expect(propertyLabel("legislation", "effective_note")).toBe("When each part took effect");
  expect(relationshipLabel("simulated-by")).toBe("Priced by");
  // Undeclared on every class: the universal convention, in words.
  expect(relationshipLabel("instance-of")).toBe("Instance of");
});
