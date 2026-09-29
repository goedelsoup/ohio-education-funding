/**
 * Every value a module exports is imported by some other file.
 *
 * # Why this is a test and not a lint rule
 *
 * `oxlint`'s `no-unused-vars` sees a binding nobody reads inside one file, and stops at the file's
 * edge: an `export` is a use as far as it can tell. #506 listed twenty-six exports in `src/` that
 * nothing imported, and this census found sixty-seven — seven of them formula constants and
 * functions in `policy.ts`. The rule would have reported none of them. So the two work together. This test refuses an export with no
 * importer; once the `export` comes off, the lint rule is what sees whether the file itself still
 * reads it.
 *
 * # What counts as an importer
 *
 * A file other than the module's own that names the module in an import — `from`, a bare
 * `import`, or a dynamic `import()` — *and* names the symbol as a word. Both halves: a namespace
 * import (`routes.districtPath`) and the dynamic `import("./scenario.ts")` behind the scenario
 * runner carry no named specifier to match, and a word match on its own would accept a docstring
 * in an unrelated file.
 *
 * # What it does not check
 *
 * Types and interfaces. They cost nothing at runtime and no test can reach one, so an unimported
 * one is untidiness rather than untested code. And `src/pages`, where `GET` and `getStaticPaths`
 * are read by Astro and imported by nobody.
 */

import { expect, test } from "vitest";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

const WEB = resolve(import.meta.dirname, "../..");

/** The modules whose exports must each have an importer. */
const CHECKED = ["src/lib", "src/scripts"];

/** Everywhere an importer may live. */
const CONSUMERS = ["src", "tests", "scripts"];

const SOURCE = /\.(ts|astro|mjs)$/;

function filesUnder(dir: string): string[] {
  return readdirSync(join(WEB, dir), { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && SOURCE.test(entry.name))
    .map((entry) => join(entry.parentPath, entry.name));
}

/** The absolute paths of every relative module a file imports, by any of the three forms. */
function importsOf(file: string, text: string): Set<string> {
  const out = new Set<string>();
  for (const match of text.matchAll(/(?:\bfrom|\bimport)\s*\(?\s*["'](\.{1,2}\/[^"']+)["']/g)) {
    out.add(resolve(dirname(file), match[1]!));
  }
  return out;
}

function valueExports(text: string): string[] {
  return [...text.matchAll(/^export (?:async )?(?:const|let|function\*?|class) +([A-Za-z_$][\w$]*)/gm)].map(
    (match) => match[1]!,
  );
}

const consumers = CONSUMERS.flatMap(filesUnder).map((file) => {
  const text = readFileSync(file, "utf8");
  return { file, text, imports: importsOf(file, text) };
});

const checked = CHECKED.flatMap(filesUnder).filter((file) => file.endsWith(".ts"));

test("every exported value in src/lib and src/scripts has an importer", () => {
  const unimported: string[] = [];
  for (const module of checked) {
    const importers = consumers.filter((c) => c.file !== module && c.imports.has(module));
    for (const name of valueExports(readFileSync(module, "utf8"))) {
      const word = new RegExp(`(?<![\\w$])${name.replace(/\$/g, "\\$")}(?![\\w$])`);
      if (!importers.some((c) => word.test(c.text))) {
        unimported.push(`${module.slice(WEB.length + 1)}: ${name}`);
      }
    }
  }
  expect(unimported).toEqual([]);
});

test("the census reads the modules it claims to", () => {
  // A walk that found nothing would pass the test above on nothing. `policy.ts` exports the formula
  // the checkpoints hold to the crate, and `scenario.ts` reaches its runner only by `import()`.
  const policy = join(WEB, "src/lib/policy.ts");
  expect(checked).toContain(policy);
  expect(valueExports(readFileSync(policy, "utf8"))).toContain("apply");
  expect(consumers.some((c) => c.imports.has(join(WEB, "src/scripts/scenario.ts")))).toBe(true);
  expect(checked.length).toBeGreaterThan(50);
});
