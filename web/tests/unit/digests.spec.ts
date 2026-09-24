/**
 * The count `/method` states for its own provenance guarantee.
 *
 * That sentence said "Thirteen of the retrieved files are pinned by SHA-256 in the repository"
 * while the manifest held several hundred. It was true when it was typed, and then understated
 * the repository's strongest checkable claim by thirty-fold, on the page whose subject is exactly
 * that claim.
 *
 * Two gates could have caught it and neither can see it: `figures.spec.ts` matches only dollar
 * amounts and percentages, so a bare count is not a figure to it, and a count spelled as a word
 * is not even a digit. `yearLiterals.spec.ts` sees years. So the count is read at build, and this
 * is the test that the reading is a reading.
 */

import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import { expect, test } from "vitest";

import { pinnedSources } from "../../src/lib/digests.ts";

/** The manifest, from the test's own resolution rather than the module's. */
const MANIFEST = ["../crates/connect/source-digests.txt", "crates/connect/source-digests.txt"]
  .map((candidate) => resolve(process.cwd(), candidate))
  .find((candidate) => existsSync(candidate));

test("the count is the manifest's, and the manifest is reachable from the build", () => {
  /*
   * Reachability is half the assertion. `pinnedSources` returns `null` rather than throwing when
   * the manifest is absent, so a wrong relative path would render the sentence without its figure
   * and pass every other check on the site silently.
   */
  expect(MANIFEST, "the digest manifest is not where the site expects it").toBeDefined();
  expect(pinnedSources()).toBeGreaterThan(0);
});

test("the header is not counted as a pinned file", () => {
  /*
   * The manifest opens with three comment lines telling a reader how to regenerate it. Counting
   * lines would have made the figure right in the same way the typed one was: by coincidence,
   * until the header gained a sentence.
   */
  const lines = readFileSync(MANIFEST!, "utf8").split("\n");
  const comments = lines.filter((line) => line.startsWith("#"));
  expect(comments.length, "the header this test is about has gone").toBeGreaterThan(0);
  expect(pinnedSources()).toBe(lines.length - comments.length - lines.filter((l) => l === "").length);
});

test("every line the count includes is a digest and a byte length", () => {
  // What the sentence on the page claims about them: SHA-256 and a size, per published file. A
  // line that matched the count but not this would make the sentence false while the figure moved.
  const records = readFileSync(MANIFEST!, "utf8")
    .split("\n")
    .filter((line) => line !== "" && !line.startsWith("#"));
  expect(records).toHaveLength(pinnedSources()!);
  for (const record of records) {
    const [digest, bytes, path] = record.split(/\s+/);
    expect(digest, record).toMatch(/^[0-9a-f]{64}$/);
    expect(Number(bytes), record).toBeGreaterThan(0);
    expect(path, record).toBeTruthy();
  }
});
