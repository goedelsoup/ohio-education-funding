/**
 * What a corpus file is allowed to be.
 *
 * # Why this exists
 *
 * Nothing had ever parsed all 62 nodes until the wiki needed to. Doing it found four separate
 * authoring defects, and the instructive thing is that **not one of them was catchable by
 * reading the file**:
 *
 * - Two nodes were invalid YAML (`irn: "044909" [verified — …]`, a quoted scalar followed by a
 *   bracket). Only a parser complains.
 * - Four had a plain scalar containing `: `, which YAML silently reads as a nested mapping.
 * - One wrote its whole `links:` block as a prose paragraph. Valid YAML, so nothing complained,
 *   and its fifteen edges were invisible to every consumer.
 * - Two wrote `target:` as a bare class name rather than a path, which rendered as a relative
 *   `href` that 404s while looking correct in the markup.
 *
 * Each was found by a page coming out wrong, days after it was written. A schema turns that class
 * of mistake into an error at the point of authoring, which is the only place it is cheap.
 *
 * # Why zod, and where the JSON Schema comes from
 *
 * `astro/zod` is already in the dependency tree, so this costs nothing new, and zod 4 emits JSON
 * Schema — which means `pnpm schemas` can write files the editor's YAML language server reads
 * while someone types. One definition, two consumers: the build validates against it, and the
 * editor warns before the build ever runs.
 *
 * # What is an error, and what is only a warning
 *
 * The line is drawn on evidence rather than on taste. Measured across the corpus as it stands:
 * **94% of properties** and only **68% of relationships** are declared in the class's own
 * ontology. The property vocabulary is effectively closed and the relationship vocabulary plainly
 * is not — 90-odd distinct relationships, most used once, describing genuinely different
 * connections. Rejecting an undeclared relationship would reject the corpus.
 *
 * So structure is an error and vocabulary is a warning. A node that cannot be read, cannot be
 * linked, or cannot be placed in a class stops the build. A node using a relationship its
 * ontology has not declared is reported and rendered.
 */

import { z } from "astro/zod";

/**
 * One edge.
 *
 * `target` is the shape that varies most and is not constrained here beyond being a non-empty
 * string: which shapes are legal is `resolveTarget`'s business, and whether the thing at the far
 * end exists needs the whole corpus. Both are checked by the validator.
 *
 * `note` is optional prose hanging off an edge, which a handful of nodes use to say why the link
 * is there.
 */
export const LinkSchema = z
  .object({
    target: z.string().min(1, "a link needs a target"),
    relationship: z.string().min(1, "a link needs a relationship"),
    note: z.string().optional(),
  })
  .strict();

/**
 * One corpus node.
 *
 * `links` is `array` and deliberately not `array | string`. Accepting the prose form would make
 * the schema agree with the defect it exists to catch.
 *
 * Property *values* are strings because all 380 of them are: the corpus writes numbers, dates and
 * lists as prose carrying a claim tag, and a schema that admitted numbers would let
 * `irn: 044933` parse as an integer and lose its leading zero.
 */
export const NodeSchema = z
  .object({
    class: z.string().min(1),
    label: z.string().min(1),
    description: z.string().min(1, "a node with no description is a filename"),
    properties: z.record(z.string(), z.string()).default({}),
    links: z.array(LinkSchema).min(1, "every node must have at least one outgoing link"),
    findings: z.string().optional(),
  })
  .strict();

/** One declared property on a class. */
export const OntologyPropertySchema = z
  .object({
    name: z.string().min(1),
    type: z.string().min(1),
    description: z.string().min(1),
  })
  .strict();

/** One declared edge on a class. */
export const OntologyEdgeSchema = z
  .object({
    relationship: z.string().min(1),
    target: z.string().min(1),
    direction: z.enum(["in", "out"]),
    description: z.string().min(1),
  })
  .strict();

/** One ontology class — the schema the corpus writes for itself. */
export const OntologyClassSchema = z
  .object({
    class: z.string().min(1),
    label: z.string().min(1),
    description: z.string().min(1),
    foundational_type: z
      .object({ ontology: z.string(), type: z.string() })
      .strict()
      .optional(),
    properties: z.array(OntologyPropertySchema).default([]),
    edges: z.array(OntologyEdgeSchema).default([]),
  })
  .strict();

export type NodeFile = z.infer<typeof NodeSchema>;
export type OntologyFile = z.infer<typeof OntologyClassSchema>;

/** Where a problem was found, and whether it stops the build. */
export interface Diagnostic {
  /** Repository-relative path. */
  file: string;
  severity: "error" | "warning";
  message: string;
}

/**
 * Turn a zod failure into one diagnostic per problem, addressed by field.
 *
 * A raw `ZodError` dump is unreadable in a build log and names the offending field only by index.
 */
export function diagnose(file: string, error: z.ZodError): Diagnostic[] {
  return error.issues.map((issue) => ({
    file,
    severity: "error" as const,
    message: `${issue.path.length > 0 ? issue.path.join(".") + ": " : ""}${issue.message}`,
  }));
}
