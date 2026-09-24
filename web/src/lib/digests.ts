/**
 * How many published files this repository has pinned by content hash.
 *
 * # Why this is read rather than written down
 *
 * `/method` said "Thirteen of the retrieved files are pinned by SHA-256 in the repository". The
 * manifest held 384. The sentence was true when somebody typed it and then understated the
 * repository's own provenance guarantee by thirty-fold, on the page whose subject is provenance.
 *
 * Nothing caught it, and the reason is worth stating because it generalises. `figures.spec.ts`
 * gates every figure on every built page against the year it is measured in — but its `FIGURE`
 * pattern matches dollar amounts and percentages only, so a bare count is not a figure to it, and
 * a count spelled as a word is not even a digit. The count was outside two gates at once.
 *
 * So it is counted from the manifest the connector writes, at build, like everything else on that
 * page that moves.
 */

import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

/** Where the connector writes the manifest, relative to whichever directory we run in. */
const CANDIDATES = [
  "../crates/connect/source-digests.txt",
  "crates/connect/source-digests.txt",
];

/**
 * A digest line: 64 hex characters, then the byte length and the path it belongs to.
 *
 * Anchored, so the file's comment header and any blank line between records are skipped by not
 * matching rather than by a second rule that has to be kept in step with the writer.
 */
const DIGEST = /^[0-9a-f]{64}\s/;

/**
 * The number of published files pinned by SHA-256, or `null` if the manifest is not reachable.
 *
 * `null` rather than zero, and rather than a throw: the site builds from a checkout that always
 * has the manifest, but the unit suite and a consumer vendoring `src/` need not. A caller renders
 * the sentence without the figure rather than asserting that nothing is pinned.
 */
export function pinnedSources(): number | null {
  const path = CANDIDATES.map((candidate) => resolve(process.cwd(), candidate)).find((candidate) =>
    existsSync(candidate),
  );
  if (!path) return null;
  return readFileSync(path, "utf8").split("\n").filter((line) => DIGEST.test(line)).length;
}
