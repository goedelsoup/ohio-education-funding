/**
 * Write `src/lib/appearances.json` from the last build: which data pages cite each corpus node.
 *
 * ```
 * pnpm build && pnpm appearances && pnpm build
 * ```
 *
 * The second build is the one that renders the result: a node page reads the file this writes,
 * and the pages this reads are the ones outside the corpus, so the inversion does not depend on
 * itself and one pass settles it. `tests/dist/appearances.spec.ts` fails while the committed file
 * differs from what this would write — see `lib/appearances.ts` for why it is committed at all.
 */

import { writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { APPEARANCES_PATH, appearancesOf, builtPages } from "../src/lib/appearances.ts";
import { exemplarIrns } from "../src/lib/corpus.ts";

const appearances = appearancesOf(builtPages(resolve("dist")), exemplarIrns());
writeFileSync(APPEARANCES_PATH, `${JSON.stringify(appearances, null, 2)}\n`);
console.log(
  `[appearances] ${Object.keys(appearances).length} corpus nodes cited from outside the corpus → ${APPEARANCES_PATH}`,
);
