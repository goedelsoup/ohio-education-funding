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
 * These warnings are almost entirely one question asked many times — "is this relationship part of
 * the vocabulary or not?" — and listing them per file buries that under 93 near-identical lines.
 * Grouped, the two dozen distinct relationships are visible at once, which is what someone
 * deciding whether to widen the ontology or narrow the corpus actually needs.
 */
if (warnings.length > 0) {
  const grouped = new Map<string, string[]>();
  for (const d of warnings) {
    const key = d.message.replace(/ is not declared by .*$/, "");
    grouped.set(key, [...(grouped.get(key) ?? []), d.file]);
  }
  console.log("WARNINGS — vocabulary a class's ontology does not declare");
  for (const [message, files] of [...grouped].sort((a, b) => b[1].length - a[1].length)) {
    console.log(`  ${String(files.length).padStart(3)}  ${message}`);
    if (files.length <= 3) for (const f of files) console.log(`       ${f}`);
  }
  console.log(
    `\n  ${grouped.size} distinct. These are reported and rendered, not rejected: the corpus's\n` +
      `  relationship vocabulary is demonstrably open — 90-odd relationships, most used once —\n` +
      `  while its property vocabulary is 94% declared and effectively closed.`,
  );
}

process.exit(errors.length > 0 ? 1 : 0);
