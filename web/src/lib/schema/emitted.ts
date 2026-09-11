/**
 * Which JSON Schema files `.yidam/schemas/` is supposed to contain, and what goes in them.
 *
 * # Why this is a module and not a table inside the script
 *
 * `.yidam/schemas/` has two generators writing into it. `pnpm schemas` (this table, via
 * `scripts/schemas.ts`) emits from the zod definitions in this directory. `mise run schema` runs
 * `yidam schema`, which emits a generic schema compiled from the ontology into the same
 * directory: it overwrites both committed files and adds 21 more that nothing refers to.
 *
 * What the overwrite costs is most of what the corpus-node schema is for. Measured against
 * `yidam schema` output at 0.12.0, `corpus-node.json` loses `figures` (the whole figure-binding
 * shape), `findings`, `revisions` and `unfilled`; loses `summary` entirely, both its `maxLength`
 * and its no-markdown-link `pattern` and its place in `required`; drops `description` from
 * `required` too; loosens `properties` from a string map to anything; and flips
 * `additionalProperties` to `true`, so a misspelled field is no longer underlined at all. The
 * title goes generic. Three of the corpus's four prose fields simply stop existing as far as the
 * editor is concerned, and `required` falls from five fields to three.
 *
 * The inherited task cannot be removed — `[schema]` lives in `mise.yidam.toml`, which
 * `mise run yidam-vendor-update` replaces wholesale — and a `[tasks.schema]` in `mise.toml`
 * does not shadow it. So the damage cannot be prevented, only noticed, and noticing it needs
 * the emitter's intent in a form a test can read: the file list catches the 21 strays, and the
 * prose fields catch the overwrite of the two committed files.
 */

import { z } from "astro/zod";

import { NodeSchema, OntologyClassSchema } from "./corpus.ts";

export const EMITTED_SCHEMAS = [
  {
    file: "corpus-node.json",
    title: "Ohio education funding — corpus node",
    description:
      "One node in .yidam/corpus/<class>/. Every node is an instance of its directory's " +
      "ontology class, carries prose with inline [verified]/[inference]/[open] claim tags, and " +
      "declares at least one outgoing link as a list — never as a paragraph.",
    schema: NodeSchema,
  },
  {
    file: "corpus-ontology.json",
    title: "Ohio education funding — corpus ontology class",
    description:
      "One class definition, .yidam/corpus/<class>.ont.yml. Declares what a kind of node is, " +
      "the properties it may carry, and the relationships it may enter into.",
    schema: OntologyClassSchema,
  },
] as const satisfies readonly {
  file: string;
  title: string;
  description: string;
  schema: z.ZodType;
}[];

/** The exact document `pnpm schemas` writes for one entry, prose fields included. */
export function emitDocument(entry: (typeof EMITTED_SCHEMAS)[number]): Record<string, unknown> {
  return {
    $schema: "https://json-schema.org/draft/2020-12/schema",
    title: entry.title,
    description: entry.description,
    ...(z.toJSONSchema(entry.schema, { io: "input" }) as Record<string, unknown>),
  };
}
