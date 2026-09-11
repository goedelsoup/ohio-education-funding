/**
 * Write the corpus schemas out as JSON Schema, for the editor.
 *
 * # Why this is worth a build step
 *
 * The zod schemas in `src/lib/schema/` stop a bad corpus file from being *published*. That is the
 * wrong end of the loop. Every one of the four authoring defects this repository has had was
 * written, committed, and only noticed when a page rendered wrong days later — by which point the
 * author has moved on and the person debugging it is reading YAML they did not write.
 *
 * JSON Schema moves the check to the keystroke. `yaml-language-server`, which ships inside the
 * usual YAML extension for VS Code and is available to Neovim and Helix through LSP, reads these
 * files and underlines `links:` the moment it is written as a paragraph.
 *
 * One definition, two consumers: zod 4 emits the JSON Schema, so the editor and the build cannot
 * disagree about what a node is.
 *
 * # Where they are written, and why not next to the code
 *
 * `.yidam/schemas/`, beside the corpus they describe. An editor mapping has to name a path, and a
 * path into `web/` would tie authoring the knowledge base to the presence of the web project.
 *
 * Run with `pnpm schemas`. CI checks the committed copies are current, the same way it checks the
 * feed — a generated file that is committed is a file that can go stale.
 */

import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { EMITTED_SCHEMAS, emitDocument } from "../src/lib/schema/emitted.ts";

const OUT = join(process.cwd(), "..", ".yidam", "schemas");

mkdirSync(OUT, { recursive: true });

for (const entry of EMITTED_SCHEMAS) {
  // Trailing newline: these are committed, and a file without one is a diff every editor makes.
  writeFileSync(join(OUT, entry.file), `${JSON.stringify(emitDocument(entry), null, 2)}\n`);
  console.log(`wrote .yidam/schemas/${entry.file}`);
}
