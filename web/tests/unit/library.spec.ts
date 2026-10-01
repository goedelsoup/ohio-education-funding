/**
 * The Library names things by what they are, not by how the corpus files them (#579).
 *
 * Two places showed the machinery: a class blurb cut at 180 characters, mid-way through the
 * argument for the class's foundational type ("It is a Phase rather than a Kind"), and a decision
 * record headed by its file stem in monospace. The dist end — what the built pages print — is
 * `tests/dist/wikiKeys.spec.ts`; this is the library end.
 */

import { expect, test } from "vitest";

import { decisionTitle, loadCorpus } from "../../src/lib/corpus.ts";
import { firstSentence } from "../../src/lib/prose.ts";

/** The foundational types a class description argues for after it has defined the class. */
const ONTOLOGY_TYPE = /\b(Kind|Phase|Event|Role|Quality|Relator|Situation)\b/;

test("a decision title is its slug as a sentence, with the corpus's capitals put back", () => {
  expect(decisionTitle("the-four-kinds-of-parameter")).toBe("The four kinds of parameter");
  expect(decisionTitle("the-three-streams-of-mr81")).toBe("The three streams of MR-81");
  expect(decisionTitle("the-jvsd-rewrite-that-changed-no-percentage")).toBe(
    "The JVSD rewrite that changed no percentage",
  );
  expect(decisionTitle("scenario-models-ohio")).toBe("Scenario models Ohio");
});

test("no decision slug carries a word sentence case cannot spell", () => {
  // A word with a digit in it, or with no vowel, is an initialism or a code — `mr81`, `jvsd` — and
  // lower case is wrong for it. Each needs an entry in DECISION_WORDS; this is what makes a new
  // record with one fail here rather than print "mr81" in its own heading.
  const { decisions } = loadCorpus();
  expect(decisions.length).toBeGreaterThan(50);
  const unspelled = decisions.flatMap((decision) =>
    decision.title
      .split(" ")
      .filter((word) => /\d/.test(word) || !/[aeiouy]/i.test(word))
      .filter((word) => word !== word.toUpperCase())
      .map((word) => `${decision.slug}: ${word}`),
  );
  expect(unspelled).toEqual([]);
  for (const decision of decisions) {
    expect(decision.title).not.toMatch(/[a-z]-[a-z]/);
    expect(decision.title).toMatch(/^[A-Z]/);
  }
});

test("firstSentence ends where the author ended the sentence, not at an initial", () => {
  expect(firstSentence("Run by the U.S. Department of Education. It is a Kind.", "district")).toBe(
    "Run by the U.S. Department of Education.",
  );
  expect(firstSentence("Paid as federal Title I. It is a Relator.", "district")).toBe("Paid as federal Title I.");
  expect(firstSentence("Decided in *DeRolph v. State* in 1997. An Event.", "district")).toBe(
    "Decided in DeRolph v. State in 1997.",
  );
  expect(firstSentence("Paid under [the formula](../x.md) [verified]. It is a Quality.", "district")).toBe(
    "Paid under the formula.",
  );
  expect(firstSentence("One sentence and nothing after it", "district")).toBe("One sentence and nothing after it");
});

test("every class blurb is a whole definition with no ontology vocabulary in it", () => {
  const { classes } = loadCorpus();
  expect(classes.length).toBeGreaterThan(15);
  const bad = classes
    .map((c) => ({ name: c.className, blurb: firstSentence(c.description, c.className) }))
    .filter(({ blurb }) => !/[.!?]$/.test(blurb) || ONTOLOGY_TYPE.test(blurb) || blurb.length > 400);
  expect(bad).toEqual([]);
});
