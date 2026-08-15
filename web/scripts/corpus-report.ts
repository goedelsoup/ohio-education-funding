/**
 * What the corpus validator found, in full.
 *
 * The build prints a count and stops on errors; this prints the list. Separated because the
 * warnings are a standing property of the corpus rather than something to fix before the next
 * deploy — 93 of them, all of the same kind — and a build log that reprinted 93 lines every time
 * would train everyone to stop reading it.
 *
 * Run with `pnpm corpus:report`.
 */

import { loadCorpus } from "../src/lib/corpus.ts";

const { diagnostics, nodes, classes, sources } = loadCorpus();

const errors = diagnostics.filter((d) => d.severity === "error");
const warnings = diagnostics.filter((d) => d.severity === "warning");

console.log(
  `${nodes.length} nodes, ${classes.length} classes, ${sources.length} sources — ` +
    `${errors.length} errors, ${warnings.length} warnings\n`,
);

if (errors.length > 0) {
  console.log("ERRORS");
  for (const d of errors) console.log(`  ${d.file}\n    ${d.message}`);
  console.log();
}

/*
 * Grouped by message rather than by file, because that is the shape of the answer.
 *
 * These warnings are a handful of questions asked many times — "is this relationship part of the
 * vocabulary or not?", "does this paragraph belong in `revisions:`?" — and listing them per file
 * buries that under a hundred near-identical lines. Grouped, the distinct questions are visible at
 * once, which is what someone deciding whether to widen an ontology or move a paragraph needs.
 *
 * # Why the two kinds are printed apart
 *
 * They go to different people and imply different work. Vocabulary drift is a decision about an
 * ontology; prose drift is an authoring task on one node. Interleaved and counted together they
 * produced a single headline — "155 corpus nodes use vocabulary their ontology does not declare" —
 * that was false of 108 of them.
 */
function report(title: string, of: typeof warnings, note: string): void {
  if (of.length === 0) return;
  const grouped = new Map<string, string[]>();
  for (const d of of) {
    // The word-count warning carries its count, which would otherwise make every node its own
    // group of one and hide the fact that it is a single question asked twenty-odd times.
    const key = d.message.replace(/ is not declared by .*$/, "").replace(/: \d+ words\./, ": too long.");
    grouped.set(key, [...(grouped.get(key) ?? []), d.file]);
  }
  console.log(title);
  for (const [message, files] of [...grouped].sort((a, b) => b[1].length - a[1].length)) {
    console.log(`  ${String(files.length).padStart(3)}  ${message}`);
    if (files.length <= 3) for (const f of files) console.log(`       ${f}`);
  }
  console.log(`\n  ${grouped.size} distinct. ${note}\n`);
}

report(
  "WARNINGS — prose in the wrong field",
  warnings.filter((d) => d.kind === "prose"),
  "A node's `description:` is what Ohio does. What this repository computed belongs in\n" +
    "  `findings:`, and what the node used to say belongs in `revisions:`. See\n" +
    "  .yidam/decisions/the-four-genres-of-a-description.yml.",
);

report(
  "WARNINGS — vocabulary a class's ontology does not declare",
  warnings.filter((d) => d.kind !== "prose"),
  "These are reported and rendered, not rejected: the corpus's relationship vocabulary is\n" +
    "  demonstrably open — 90-odd relationships, most used once — while its property vocabulary\n" +
    "  is 94% declared and effectively closed.",
);

process.exit(errors.length > 0 ? 1 : 0);
