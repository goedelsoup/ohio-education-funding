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
 * Not the whole log. This repository's history predates the vocabulary and roughly half of it
 * sits outside — `corpus:`, `web:`, `connect:`, `gate:` are scopes, not verbs, and
 * `vendor(yidam):` is the exact shape the prelude warns is read as a verb nobody declared. None
 * of that can be fixed and none of it should be hidden; it is a fact about when this repository
 * was written.
 *
 * # What it does check, since the practice converged
 *
 * The tail of the log, which is a different claim from the whole of it. `7d46e59` — `corpus: the
 * district pair, and the hoist that had already happened`, 2026-08-27 — is the last commit here
 * written outside the vocabulary. Every commit after it conforms, and there are enough of them
 * now that the convergence is a practice rather than a run of luck.
 *
 * So the ratchet below holds the tail at zero. It rewrites nothing and forgives everything
 * before the boundary, which is the part that cannot be fixed; it only refuses to let the share
 * grow again. That matters because the measure the kuten reports is a *whole-history* ratio —
 * 211 of 443 non-merge commits, 47.6% against a declared 0%–2% — and a ratio over all of
 * history can never re-enter that band no matter how well anyone behaves. It would take on the
 * order of ten thousand further clean commits. The band is therefore not a target, and the only
 * honest thing left to hold is the derivative: the count of non-conforming commits after the
 * boundary, which is zero and must stay zero.
 *
 * This does not replace the rule above, it backs it. A step naming its verb acts before the
 * commit exists; this acts before the branch merges. The prelude's complaint was that
 * `yidam lint --commits` reports drift only once it is permanent — on a branch it is not yet.
 */

import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { expect, test } from "vitest";

const ROOT = join(import.meta.dirname, "../../..");
const GRAPH = join(ROOT, ".yidam/.vendor/prelude/GRAPH.md");

/** The closed vocabulary, read from the prelude rather than restated here. */
function vocabulary(): Set<string> {
  const graph = readFileSync(GRAPH, "utf8");
  const start = graph.indexOf("## Commit vocabulary");
  expect(
    start,
    "GRAPH.md no longer declares a commit vocabulary",
  ).toBeGreaterThan(-1);
  const section = graph.slice(start, graph.indexOf("\n## ", start + 1));
  return new Set(
    [...section.matchAll(/^\| `([a-z]+)` \|/gm)].map(([, verb]) => verb!),
  );
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
      const named = [...line.matchAll(/`([a-z]+):`/g)].some(([, verb]) =>
        verbs.has(verb!),
      );
      if (!named)
        unnamed.push(`${path}:${index + 1} — ${line.trim().slice(0, 90)}`);
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
        if (!verbs.has(verb!))
          wrong.push(
            `${path}:${index + 1} names \`${verb}:\`, which is not in the vocabulary`,
          );
      }
    });
  }
  expect(wrong).toEqual([]);
});

/**
 * The last commit written outside the vocabulary, and the boundary the ratchet counts from.
 *
 * A commit rather than a date. Dates move under rebase and disagree across timezones; this is the
 * object the claim is actually about. If it ever stops resolving — a rewritten history, a shallow
 * clone — the test says so rather than passing on an empty log, because a ratchet that silently
 * measures nothing is worse than no ratchet.
 */
const CONVERGED_AT = "7d46e59da889a481f3e398527bd7eb27476eed54";

/** `git`, run at the repository root, trimmed. */
function git(...args: string[]): string {
  return execFileSync("git", args, {
    cwd: ROOT,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  }).trim();
}

test("no commit since the practice converged carries a verb outside the vocabulary", () => {
  // A shallow clone cannot see the boundary, and would pass this test by knowing nothing. CI
  // checks out with `fetch-depth: 0` for exactly this test; if that is ever dropped, fail loudly.
  expect(
    git("rev-parse", "--is-shallow-repository"),
    "shallow clone: set `fetch-depth: 0`",
  ).toBe("false");
  expect(
    git("cat-file", "-t", CONVERGED_AT),
    `${CONVERGED_AT} no longer resolves; history was rewritten and the boundary needs re-establishing`,
  ).toBe("commit");

  const verbs = vocabulary();
  expect(verbs.size, "the vocabulary parsed as empty").toBeGreaterThan(20);

  const log = git(
    "log",
    "--no-merges",
    "--format=%h%x09%s",
    `${CONVERGED_AT}..HEAD`,
  );
  const commits = log === "" ? [] : log.split("\n");
  expect(
    commits.length,
    "no commits after the boundary — the range is wrong",
  ).toBeGreaterThan(0);

  const offVocabulary: string[] = [];
  for (const line of commits) {
    const [sha, subject = ""] = line.split("\t");
    // GRAPH.md reads everything before the first `: ` as the verb, so a conventional-commits
    // scope makes a declared verb unreadable. Both failures are the same finding here.
    const match = /^([a-z]+)(\([^)]*\))?:/.exec(subject);
    if (!match) {
      offVocabulary.push(
        `${sha} has no \`verb:\` prefix — ${subject.slice(0, 72)}`,
      );
    } else if (match[2]) {
      offVocabulary.push(
        `${sha} writes \`${match[1]}${match[2]}:\`; the scope hides the verb`,
      );
    } else if (!verbs.has(match[1]!)) {
      offVocabulary.push(
        `${sha} writes \`${match[1]}:\`, which the vocabulary does not declare`,
      );
    }
  }
  expect(offVocabulary).toEqual([]);
});
