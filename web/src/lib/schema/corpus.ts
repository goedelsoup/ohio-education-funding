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
 * One thing this node used to say and no longer does.
 *
 * # Why this is structured and the corrections in a decision record are not
 *
 * `markCorrections` finds a withdrawal in a decision record by looking for a blockquote that opens
 * with strong emphasis. It works, it is measured across all twenty-four records, and it is pinned
 * by spec. The same trick was available for nodes and is not safe for nodes: in a decision record
 * a correction misfiled as a quotation is set in the wrong type, and in a corpus node a retraction
 * misfiled as body copy **publishes a withdrawn claim as current**. The signals are equally
 * strong in both places; only one of them can afford the residue.
 *
 * # The fields
 *
 * `was` and `now` are the two halves nobody can reconstruct later — the corpus is not rewritten to
 * have always been right, on the same ground `.yidam/decisions/README.md` gives for never
 * rewriting a decision record.
 *
 * `found_by` is the field that earns the structure. It names the test, source, or catalog record
 * that settled it, which turns "this was wrong once" into something an agent can re-run. Not an
 * enum: the things that have settled a claim here are a Rust test, a catalog source, a department
 * workbook, a court opinion, and in one case the absence of a source, and five kinds from the
 * first pass is a sample rather than a vocabulary.
 *
 * `reach` is optional and says what else the mistake touched — or states that nothing downstream
 * moved, which is the more common answer and the one a reader most needs to see stated rather
 * than inferred from silence.
 */
export const RevisionSchema = z
  .object({
    was: z.string().min(1, "a revision has to say what the node used to claim"),
    now: z.string().min(1, "a revision has to say what replaced the claim"),
    found_by: z.string().min(1, "a revision names the test, source or record that settled it"),
    reach: z.string().optional(),
  })
  .strict();

/**
 * One thing a node does not hold, named.
 *
 * # Why this is structure and not a fourth claim tag
 *
 * The three claim tags grade how well a claim is supported. A field nobody has filled in is not a
 * weaker grade of support — it is the absence of a claim to grade — so it does not belong on that
 * axis, and putting it there makes it read as "worse than open" whatever it wears. The corpus
 * README said as much for a long time while the parser went on emitting it as a fourth inline
 * badge.
 *
 * So the absence moves out of the sentence and becomes data. Two things follow that prose could
 * not give: the loader reads a list instead of matching a pattern, and a build check can assert
 * that no `[unentered]` string survives anywhere in a node.
 *
 * # `field` names the missing FACT, not necessarily a property key
 *
 * Only a third of these are whole-field absences. The rest sit inside a property that carries real
 * content — an agency's `typology` holds "Urban, very high poverty" and lacks only the department's
 * own code. Naming the property there would claim the whole field is empty, which is worse than the
 * badge it replaced. So `field` is a short noun phrase: `established`, or `the department's
 * typology code`.
 *
 * # Why an object rather than a string
 *
 * `why` is worth keeping — most of these already explained where the value lives and what it would
 * take to get it, and that explanation is the difference between a to-do and a shrug. Carrying it
 * in one string would mean `field: why`, and a plain scalar containing `: ` is one of the four
 * authoring defects this corpus has already been bitten by: YAML reads it as a nested mapping.
 */
export const UnfilledSchema = z
  .object({
    field: z.string().min(1, "an unfilled entry names what is missing"),
    why: z.string().optional(),
  })
  .strict();

/**
 * One number in this node's prose, bound to the crate that computes it.
 *
 * # What this is for
 *
 * `[verified — crates/regime-diff]` is hand-typed text. Nothing relates it to
 * `crates/regime-diff`, so the blast radius of a correction is whichever files the author happened
 * to open — which is how the `recognized-valuation` correction reached three of its six carriers
 * while two nodes went on publishing a reversed sign under `[verified]`. See #120 and #131.
 *
 * A `figures:` entry is the missing edge. `crates/figures.json` carries each figure computed from
 * the crate that owns it, and `tests/unit/corpusFigures.spec.ts` asserts a three-way agreement:
 * the manifest, this entry, and the sentence a reader sees. Change the calculator and every node
 * bound to that key goes red at once.
 *
 * # Why `value` as well as `as_written`
 *
 * They are two different claims and both have been wrong here before. `value` is what this node
 * says the crate computes; `as_written` is what the page prints. #128 found seventeen numeric
 * claims disagreeing with their source, and #118 found a sentence whose two halves were computed
 * over different populations — the first is `value` against the manifest, the second is
 * `as_written` against `value`. Collapsing them into one field would drop whichever check the
 * remaining field did not make.
 *
 * # Why `as_written` is a phrase and not a number
 *
 * Because a bare numeral binds nothing. `316` appears in six places across three nodes, and the
 * one thing the audit found there was two of them stating the *opposite* distributional claim with
 * the same two numerals in it. A phrase — `316 districts would have done better under the
 * charge-off` — has to survive verbatim, so swapping which regime the count belongs to breaks it
 * where a numeral match would not.
 */
export const FigureSchema = z
  .object({
    /** The manifest key, `<crate-directory>/<what-it-is>`. */
    key: z
      .string()
      .regex(
        /^[a-z0-9-]+\/[a-z0-9-]+$/,
        "a figure key is `<crate-directory>/<what-it-is>`, lower-case kebab",
      ),
    /** What this node says the crate computes. Compared against `crates/figures.json`. */
    value: z.number(),
    /** Which prose field carries it — `description`, `findings`, or a property name. */
    field: z.string().min(1, "a figure names the field whose prose carries it"),
    /** The phrase, verbatim, as that field writes it. */
    as_written: z.string().min(1, "a figure quotes the phrase its field writes"),
  })
  .strict();

/** What a `summary` may not be longer than, in words. See {@link NodeSchema}. */
export const SUMMARY_MAX_WORDS = 50;

/** Count words the way the summary limit means them: runs of non-whitespace. */
export function words(text: string): number {
  return text.split(/\s+/).filter((word) => word !== "").length;
}

/**
 * One corpus node.
 *
 * `links` is `array` and deliberately not `array | string`. Accepting the prose form would make
 * the schema agree with the defect it exists to catch.
 *
 * Property *values* are strings because all 380 of them are: the corpus writes numbers, dates and
 * lists as prose carrying a claim tag, and a schema that admitted numbers would let
 * `irn: 044933` parse as an integer and lose its leading zero.
 *
 * # The four fields a node writes prose into, and which reader each is for
 *
 * A description was measured carrying four different documents at once — 53,196 words across 101
 * nodes, a quarter of the paragraphs about this repository rather than about Ohio. The split is
 * `.yidam/decisions/the-four-genres-of-a-description.yml`:
 *
 * | Field | Holds | Where it renders |
 * |---|---|---|
 * | `summary` | the lead — what the thing is | under the `h1`, and in every `<meta>` and OG card |
 * | `description` | the subject: mechanism, statute, history | the body |
 * | `findings` | what this repository computed, and how to read it | its own section |
 * | `revisions` | what this node used to say | a collapsed disclosure |
 *
 * `summary` is capped rather than merely encouraged because it is the string five call sites
 * substitute for the whole node — the class index cell, the node's `<meta name="description">`,
 * two OG cards and the wiki front door. Each of those used to truncate the description with
 * `slice()`, which is how 110 of 143 meta descriptions were once found cut mid-word. A cap that
 * is checked cannot drift back into a paragraph.
 *
 * It also may not contain a markdown link, for the same reason: none of those five destinations
 * can render one, and `summarize` stripping the target while keeping the label is what produced
 * "the suburban counterpart to  eleven miles away across the Maumee".
 */
export const NodeSchema = z
  .object({
    class: z.string().min(1),
    label: z.string().min(1),
    summary: z
      .string()
      .min(1, "a node needs a lead")
      /*
       * Both caps are here on purpose and they are not redundant.
       *
       * The word count is the rule and carries the message worth reading. `maxLength` and
       * `pattern` are the halves of it that survive `z.toJSONSchema` — a refinement is not
       * representable in JSON Schema and is silently dropped, so a rule expressed only as a
       * refinement stops the build and never reaches the editor, which is the loop this schema
       * exists to shorten. The character cap is the word cap at a generous average.
       */
      .max(SUMMARY_MAX_WORDS * 9, "a summary is the lead, not the description")
      .regex(/^(?!.*\]\()/s, "a summary may not carry a markdown link")
      .refine((text) => words(text) <= SUMMARY_MAX_WORDS, {
        message: `a summary is the lead, not the description — keep it to ${SUMMARY_MAX_WORDS} words`,
      })
      .refine((text) => !text.includes("]("), {
        message:
          "a summary may not carry a markdown link; it is rendered into <meta> and OG cards that cannot show one",
      }),
    description: z.string().min(1, "a node with no description is a filename"),
    properties: z.record(z.string(), z.string()).default({}),
    links: z.array(LinkSchema).min(1, "every node must have at least one outgoing link"),
    findings: z.string().optional(),
    revisions: z.array(RevisionSchema).optional(),
    unfilled: z.array(UnfilledSchema).optional(),
    figures: z.array(FigureSchema).optional(),
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
 * The corpus behaves as though they are illustrative and always has: about half its edges use an
 * undeclared relationship, almost all of them single-use precise verbs — `recovered-funds-from`,
 * `retreats-from`, `reframes` — that say something a generic edge would lose. That was previously
 * unstated, so a validator had no way to tell a deliberate coinage from a typo and every one of
 * them produced a warning.
 *
 * The exact counts are generated into `.yidam/corpus/README.md` under "relationship vocabulary".
 * The figures that stood here were hand-written, shared with fifteen ontology files, and stale in
 * all seventeen copies before anyone re-measured them.
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

/**
 * What kind of problem a warning is, which decides how it is summarised rather than whether it is
 * reported.
 *
 * Two unrelated questions arrive on the same channel and a single count conflates them. Vocabulary
 * drift asks whoever owns an ontology whether to widen it; prose drift asks whoever is authoring a
 * node to move a paragraph into the field it belongs in. Reporting "155 nodes use vocabulary their
 * ontology does not declare" when 108 of those were shouted retraction leads is the kind of
 * counter this repository has already had to fix three times — a name standing for something other
 * than what it counts.
 */
export type DiagnosticKind = "shape" | "vocabulary" | "prose";

/** Where a problem was found, and whether it stops the build. */
export interface Diagnostic {
  /** Repository-relative path. */
  file: string;
  severity: "error" | "warning";
  /** Which question the reader has to answer. Defaults to `shape` — the schema's own failures. */
  kind?: DiagnosticKind;
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
