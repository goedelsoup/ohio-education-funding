/**
 * Every node can be reached by following edges, not only by reading paragraphs.
 *
 * # The distinction this rests on
 *
 * `corpus.ts` separates a **stated** edge from a **mentioned** one, and says why: "a stated edge
 * is the author asserting a relationship, a mentioned one is the author citing something." Both
 * belong in a backlink index and they are not the same claim.
 *
 * A review found twelve nodes with no stated inbound edge at all. Eight were cited somewhere in
 * prose — the relationship had been asserted in a sentence and never written into `links:` — so
 * anything traversing the graph could not reach them, while a reader skimming the site could.
 * The corpus's own generated index reported four orphans rather than twelve, because it counts
 * prose mentions as inbound; that count is right for what it measures and wrong for this.
 *
 * # Why an allowlist rather than a zero
 *
 * `draft-legislation` is exempt, and by design rather than by concession.
 * [`drafts-are-not-legislation`](../../../.yidam/decisions/drafts-are-not-legislation.yml) states
 * it: *"A draft produces scenarios rather than being one: `simulated-by` runs from the draft to
 * one or more scenario nodes."* A draft points at the parameters it perturbs, the programme it
 * would narrow, the act it redrafts and the runs that price it. Nothing in Ohio's enacted
 * arrangement points back at a bill that has not passed, and manufacturing an edge so it would
 * be inventing a relationship to satisfy a test.
 *
 * The exemption is the class, not a list of node names, so a fourth draft inherits it. Anything
 * else that turns up here is a gap.
 */

import { expect, test } from "vitest";

import { loadCorpus } from "../../src/lib/corpus.ts";

/**
 * Classes whose nodes may have nothing pointing at them, and the document that says so.
 *
 * An entry here is a design decision with a citation, not a place to put things.
 */
const OUTWARD_FACING: Record<string, string> = {
  "draft-legislation":
    "A draft produces scenarios rather than being one — see .yidam/decisions/drafts-are-not-legislation.yml. Nothing enacted points back at a bill that has not passed.",
};

test("every node has an edge pointing at it, not just a sentence", async () => {
  const corpus = await loadCorpus();
  const unreachable = corpus.nodes
    .filter((node) => !OUTWARD_FACING[node.className])
    .filter((node) => node.in.filter((edge) => edge.stated && edge.id).length === 0)
    .map((node) => node.id);
  expect(unreachable).toEqual([]);
});

test("and the exempt classes are exempt because a decision says so, not because they are empty", async () => {
  const corpus = await loadCorpus();
  for (const className of Object.keys(OUTWARD_FACING)) {
    const held = corpus.nodes.filter((node) => node.className === className);
    expect(held.length, `${className} is allowlisted and holds no nodes`).toBeGreaterThan(0);
  }
});

/**
 * A node cited in prose by nobody and pointed at by nobody is invisible from every direction.
 *
 * Weaker than the check above and worth keeping separately: this one would still fail if the
 * exemption above were ever widened to a class that genuinely had been forgotten about.
 */
test("no node is invisible from both directions at once", async () => {
  const corpus = await loadCorpus();
  const invisible = corpus.nodes
    .filter((node) => !OUTWARD_FACING[node.className])
    .filter((node) => node.in.length === 0)
    .map((node) => node.id);
  expect(invisible).toEqual([]);
});
