/**
 * Every instruction that tells a reader to commit something names the verb it commits under.
 *
 * # Why this is a test and not a convention
 *
 * The vendored prelude's `GRAPH.md` carries a closed vocabulary of thirty leading verbs, and
 * `yidam lint --commits` reports anything outside it. That lint is Warn severity, correctly so —
 * history cannot be rewritten to fix a verb — which means it reports drift only *after* the drift
 * is permanent. The prelude names the consequence directly: a derived repository put four
 * consecutive commits on the wrong verb and the finding sat in a warning nobody read.
 *
 * Its own remedy is the rule this test enforces: **a step that produces a commit names its verb,
 * in the step**. That is the only mechanism in the system that acts before the commit exists.
 *
 * # What it does not check
 *
 * Not the log. This repository's history predates the vocabulary and roughly half of it sits
 * outside — `corpus:`, `web:`, `connect:`, `gate:` are scopes, not verbs, and `vendor(yidam):`
 * is the exact shape the prelude warns is read as a verb nobody declared. None of that can be
 * fixed and none of it should be hidden; it is a fact about when this repository was written.
 * What can be held is that the next instruction to write a commit says what to call it.
 */

import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { expect, test } from "vitest";

const ROOT = join(import.meta.dirname, "../../..");
const GRAPH = join(ROOT, ".yidam/.vendor/prelude/GRAPH.md");

/** The closed vocabulary, read from the prelude rather than restated here. */
function vocabulary(): Set<string> {
  const graph = readFileSync(GRAPH, "utf8");
  const start = graph.indexOf("## Commit vocabulary");
  expect(start, "GRAPH.md no longer declares a commit vocabulary").toBeGreaterThan(-1);
  const section = graph.slice(start, graph.indexOf("\n## ", start + 1));
  return new Set([...section.matchAll(/^\| `([a-z]+)` \|/gm)].map(([, verb]) => verb!));
}

/** Every document in this repository that could tell somebody to make a commit. */
function instructions(): { path: string; lines: string[] }[] {
  const files: string[] = [];
  const skills = join(ROOT, ".yidam/skills");
  for (const file of readdirSync(skills)) {
    if (file.endsWith(".md")) files.push(join(skills, file));
  }
  const corpus = join(ROOT, ".yidam/corpus");
  for (const entry of readdirSync(corpus, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    for (const file of readdirSync(join(corpus, entry.name))) {
      if (file.endsWith(".md")) files.push(join(corpus, entry.name, file));
    }
  }
  return files.map((path) => ({
    path: path.slice(ROOT.length + 1),
    lines: readFileSync(path, "utf8").split("\n"),
  }));
}

/**
 * A step that directs a commit, as against a line that mentions one.
 *
 * A **step** is a list item opening in the imperative: `6. **Commit the node** …`. That is the
 * shape the prelude's rule is about — the point where a reader is about to run `git commit` and
 * the verb can still be chosen.
 *
 * Deliberately narrower than "any line containing the word". A skill's frontmatter `description:`
 * and its own index row both say the skill "commits the result", and `foundation.md` observes
 * that scenario nodes "commit its output" — three descriptions of behaviour, none of them an
 * order to anybody. Holding those to a verb would mean writing `establish:` into an English
 * sentence about what a tool does, which is noise, and the pressure to do that is how a check
 * like this gets deleted rather than obeyed.
 */
const DIRECTS = /^\s*(?:[0-9]+\.|[-*])\s+(?:\*\*)?Commit\b/;

test("every step that directs a commit names its verb from the prelude's vocabulary", () => {
  const verbs = vocabulary();
  expect(verbs.size, "the vocabulary parsed as empty").toBeGreaterThan(20);

  const unnamed: string[] = [];
  for (const { path, lines } of instructions()) {
    lines.forEach((line, index) => {
      if (!DIRECTS.test(line)) return;
      const named = [...line.matchAll(/`([a-z]+):`/g)].some(([, verb]) => verbs.has(verb!));
      if (!named) unnamed.push(`${path}:${index + 1} — ${line.trim().slice(0, 90)}`);
    });
  }
  expect(unnamed).toEqual([]);
});

test("and the verb a step names is one the prelude actually declares", () => {
  const verbs = vocabulary();
  const wrong: string[] = [];
  for (const { path, lines } of instructions()) {
    lines.forEach((line, index) => {
      if (!DIRECTS.test(line)) return;
      for (const [, verb] of line.matchAll(/with the `([a-z]+):` verb/g)) {
        if (!verbs.has(verb!)) wrong.push(`${path}:${index + 1} names \`${verb}:\`, which is not in the vocabulary`);
      }
    });
  }
  expect(wrong).toEqual([]);
});
