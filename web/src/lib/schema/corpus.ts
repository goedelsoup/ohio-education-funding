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

/**
 * Whether a class's declared `edges:` are the permitted set or an illustrative one.
 *
 * The corpus behaves as though they are illustrative and always has: 46 of the 90 relationships in
 * use are undeclared, almost all of them single-use precise verbs — `recovered-funds-from`,
 * `retreats-from`, `reframes` — that say something a generic edge would lose. That was previously
 * unstated, so a validator had no way to tell a deliberate coinage from a typo and every one of
 * them produced a warning.
 *
 * Stating it turns the check into something useful in both directions. `characteristic` means the
 * list documents what the class is defined by, and an undeclared relationship is fine.
 * `exhaustive` means the vocabulary is closed and anything outside it is an error.
 */
export const EdgePolicySchema = z.enum(["characteristic", "exhaustive"]);

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
    /**
     * Required, and with no default on purpose. A class that has not said which it means is a
     * class nobody has decided about, and defaulting would let that go unnoticed forever.
     */
    edge_policy: EdgePolicySchema,
    edges: z.array(OntologyEdgeSchema).default([]),
  })
  .strict();

/**
 * A per-node observation snapshot rather than a schema property.
 *
 * `fy2024_profile`, `fy2021_idea_part_b`, `fy2027_scale` — a year's figures pasted onto the node
 * they describe. They recur, so they are not typos, but declaring them would mean editing an
 * ontology every fiscal year to permit next year's. The year in the name is what makes them
 * self-describing, and it is what this matches on.
 */
export const OBSERVATION_PROPERTY = /^fy\d{4}(_\d{2})?_[a-z0-9_]+$/;

/**
 * Properties any class may carry, whatever its own ontology declares.
 *
 * `seeded_because` says why a node is in the corpus at all — "the anchor case for the
 * property-poor end of Ohio's distribution", "JVSDs are funded on rules that differ from
 * traditional districts". That is a fact about the corpus rather than about Ohio, and it used to
 * be written into the middle of the first sentence of a description, so a reader arriving to find
 * out what a district *is* met an editorial note about why it had been entered. It reads as
 * apparatus because it is apparatus, and the node page renders it apart from the body copy.
 *
 * Universal rather than declared per class for the same reason `instance-of` and `sourced-from`
 * are: it applies to every class, so declaring it in all sixteen ontologies would be sixteen
 * copies of one decision, and a seventeenth class would silently not have it.
 */
export const UNIVERSAL_PROPERTIES = new Set(["seeded_because"]);

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
