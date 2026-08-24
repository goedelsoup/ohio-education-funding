/**
 * The corpus, read from `.yidam/` at build time.
 *
 * # Why the web layer reads the YAML directly
 *
 * Every number on this site comes through `crates/bundle`, because numbers are computed and Rust
 * is authoritative for the computation. The corpus is not computed. It is a document graph —
 * 96 nodes, 16 ontology classes and 29 catalog sources — and putting a Rust export crate between
 * the YAML and the page would add a second serialization to keep in sync without anything
 * deciding what is true along the way. So this reads the source of truth, and there is no
 * committed intermediate that can go stale.
 *
 * Structurally this is the twin of `feed.ts`: read once at build, memoize, expose the indexes the
 * pages would otherwise rebuild. Like that module it must never be imported by client code.
 *
 * # Two things the corpus does that a naive reader would get wrong
 *
 * **Links are not all in `links:`.** Node descriptions are dense with inline markdown links that
 * no structured field records, three nodes carry a `findings:` block, and property values contain
 * them too. A graph built from `links:` alone loses most of the corpus's cross-referencing, so
 * this reads all four places and {@link Edge.stated} says which kind each edge is.
 *
 * Those four places are concatenated into {@link Node.linkText}, which exists *only* to be
 * scanned. It is deliberately not {@link Node.description}: the two were once the same field, and
 * the result was that every node page printed its description twice and then rendered its raw
 * property values — `partially-implemented`, `FY2022` — as body copy above the table that already
 * showed them. What a page displays and what a link scanner reads are different questions.
 *
 * One node used to write its entire `links:` block as a prose paragraph — valid YAML, so nothing
 * complained, and its fifteen edges were invisible to every consumer including its own
 * `instance-of`. It is structured now and `schema/corpus.ts` rejects the shape, but the parsing
 * here stays tolerant of it: the reader's job is to get far enough to report what is wrong, and
 * the schema's job is to stop the build.
 *
 * **Claims carry their epistemic status inline.** `[verified]`, `[inference]`, `[open]` and
 * `[unentered]` are a corpus-wide convention, usually with a qualifier attached — `[verified —
 * computed; see …]`, `[inference, Fordham]`, `[verified as proposed]`. They are the corpus's
 * central discipline and rendering them as stray brackets would throw away the thing that makes
 * it trustworthy, so they become badges. See `prose.ts` for the forms they are written in.
 */

import { readFileSync, readdirSync } from "node:fs";
import { existsSync } from "node:fs";
import { join, resolve } from "node:path";

import YAML from "yaml";

import * as routes from "./routes.ts";
import {
  diagnose,
  NodeSchema,
  OBSERVATION_PROPERTY,
  OntologyClassSchema,
  UNIVERSAL_PROPERTIES,
  type Diagnostic,
} from "./schema/corpus.ts";

export type { Diagnostic } from "./schema/corpus.ts";

/** Where the repository puts its knowledge base, relative to whichever directory we run in. */
const CANDIDATES = ["../.yidam", ".yidam"];
const YIDAM =
  CANDIDATES.map((path) => resolve(process.cwd(), path)).find((path) => existsSync(path)) ??
  resolve(process.cwd(), CANDIDATES[0]!);

const CORPUS = join(YIDAM, "corpus");
const CATALOG = join(YIDAM, "catalog");
const DECISIONS = join(YIDAM, "decisions");

/** Where a link that leaves the corpus points. The repository, on GitHub. */
const REPO = "https://github.com/goedelsoup/ohio-education-funding/blob/main";

/**
 * The `.yidam/` subtrees this site does not publish.
 *
 * `corpus/`, `catalog/` and `decisions/` become pages here; these three are working documents for
 * whoever is authoring the corpus — skill definitions, protocol notes, JSON schemas. They are real
 * documents, so the honest resolution is GitHub rather than a refusal.
 *
 * # Why `decisions` is no longer in this list
 *
 * It was, and the reason given was that a decision record is a working document for the corpus's
 * author. That was accurate when it was written and stopped being so. Decision records became the
 * place this repository keeps the two things a reader cannot reconstruct from anywhere else:
 * corrections to claims *this site published*, and rejections a later phase overturned. A reader
 * following the site's account of Ohio's district consolidations had no on-site path to the
 * correction withdrawing it.
 *
 * The specific failure the old comment warns about is still a failure and is still prevented.
 * `/wiki/decisions/report-card-connector` — plural, from reading `decisions` as an ontology class
 * — is not a route. The route is `/wiki/decision/<slug>`, singular, exactly as `source` is, and it
 * is produced by a branch of {@link resolveTarget} that names the subtree rather than by the
 * `class/name.yml` branch falling through.
 */
const UNPUBLISHED = ["skills", "sangha", "schemas"];

/** One property of a node: a name, and prose that usually carries a claim tag. */
export interface Property {
  name: string;
  value: string;
}

/** A directed edge in the corpus graph. */
export interface Edge {
  /** Slug of the other node, `<class>/<name>`, or null if it leaves the corpus. */
  id: string | null;
  /** Where it points on this site, or off it. */
  href: string;
  /** Display text. */
  label: string;
  /** The named relationship, for edges the node declares. */
  relationship?: string;
  /**
   * Whether the node declared this edge in `links:` or merely mentions it in prose.
   *
   * Both are real and both belong in a backlink index, but they are not the same claim: a stated
   * edge is the author asserting a relationship, a mentioned one is the author citing something.
   */
  stated: boolean;
}

/**
 * One thing a node used to say and no longer does.
 *
 * The corpus is not rewritten to have always been right — the same rule
 * `.yidam/decisions/README.md` states for decision records, for the same reason: the wrong turn is
 * the most useful thing on the page and editing it out leaves a document that teaches nothing.
 *
 * Structured rather than found in prose because the failure mode is asymmetric. See
 * `RevisionSchema` in `schema/corpus.ts`.
 */
export interface Revision {
  /** The claim as it stood. Markdown. */
  was: string;
  /** What replaced it. Markdown. */
  now: string;
  /** The test, source or record that settled it. Markdown, usually a link. */
  found_by: string;
  /** What else the mistake touched, where it touched anything. Markdown. */
  reach?: string;
}

/** One corpus node. */
export interface Node {
  /** `<class>/<name>` — the identity used throughout, and the URL. */
  id: string;
  /** Directory it lives in, which is also its ontology class. */
  className: string;
  /** File stem. */
  name: string;
  label: string;
  /**
   * The lead: what this thing is, in at most 50 words and with no markdown link.
   *
   * The one string that stands in for the whole node wherever there is no room for the node —
   * the `h1`'s subtitle, `<meta name="description">`, both OG cards, the class index cell and the
   * wiki front door. Every one of those used to call `summarize(description, N)` and get a
   * different mechanical truncation of the same paragraph.
   */
  summary: string;
  /** Markdown, and *only* the description. Rendered by {@link renderProse}. */
  description: string;
  /**
   * Every place this node writes a link, concatenated — for scanning, never for display.
   *
   * The description, the `links:` block when a node wrote it as a paragraph, `findings:`, and
   * each property value. Consumers that want to know what a node references read this; consumers
   * that want to show a reader something read {@link description}.
   */
  linkText: string;
  properties: Property[];
  /** What this repository computed about the node's subject, and how to read it. Markdown. */
  findings: string | null;
  /** What the node used to say, oldest first. Empty on a node that has never been corrected. */
  revisions: Revision[];
  /**
   * What this node does not hold, named.
   *
   * Structure rather than a fourth claim mark: an empty field is the absence of a claim to grade,
   * not a weak one, so it renders as a block where the content would be instead of as a badge in
   * somebody's sentence. See `UnfilledSchema` in `schema/corpus.ts`.
   */
  unfilled: Unfilled[];
  /**
   * Numbers in this node's prose bound to the crate that computes them.
   *
   * The corpus's answer to a `[verified — crates/X]` that nothing relates to `crates/X`. Checked
   * against `crates/figures.json` by `tests/unit/corpusFigures.spec.ts`; see `FigureSchema` in
   * `schema/corpus.ts` for why an entry carries both a value and a phrase.
   */
  figures: BoundFigure[];
  /** Edges this node declares plus the ones it only mentions. */
  out: Edge[];
  /** Populated after every node is read. */
  in: { id: string; label: string; href: string; relationship?: string; stated: boolean }[];
}

/** One ontology class: what a kind of node is, and what may be said about it. */
export interface OntologyClass {
  className: string;
  label: string;
  description: string;
  foundationalType: string | null;
  /**
   * Whether {@link edges} is the permitted set or an illustrative one. Declared per class in the
   * ontology file; see `EdgePolicySchema`.
   */
  edgePolicy: "characteristic" | "exhaustive";
  properties: { name: string; type: string; description: string }[];
  edges: { relationship: string; target: string; direction: string; description: string }[];
  nodes: Node[];
}

/** One catalog entry: a source the corpus cites. */
export interface Source {
  slug: string;
  title: string;
  /** The whole markdown body, minus the leading `# ` heading. */
  body: string;
  /** Nodes that cite this source. */
  citedBy: { id: string; label: string; href: string }[];
}

/** One section of a decision record: the field name, and the markdown under it. */
export interface Section {
  /** The YAML key, verbatim: `context`, `decision`, `consequences`, `alternatives`. */
  name: string;
  /** What to print above it. */
  label: string;
  body: string;
}

/**
 * One decision record: why the repository is shaped the way it is, including where it was wrong.
 *
 * # Why there is no title field
 *
 * Because the corpus does not write one, and inventing one here would be worse than using what it
 * does write. A node carries `label:` and a catalog entry carries a `# ` heading; a decision
 * record carries only `id`, and every reference to one in corpus prose is the slug in code font —
 * ``[`the-order-was-never-the-states`](…)``. So the slug is the title, shown the way the corpus
 * already shows it, and {@link summary} is the sentence underneath. Sentence-casing the slug would
 * turn `the-three-streams-of-mr81` into "The three streams of mr81".
 */
export interface Decision {
  /** File stem, which is also `id:` in the record and the URL. */
  slug: string;
  /** The record's own one-paragraph statement of what it decided. Markdown. */
  summary: string;
  /** Every prose field the record carries, in the order they are meant to be read. */
  sections: Section[];
  /** Every section's markdown concatenated — for scanning links, never for display. */
  linkText: string;
  /** How many correction blocks it carries. See {@link markCorrections} in `prose.ts`. */
  corrections: number;
  /** Nodes, catalog entries and other decisions that link here. */
  citedBy: { id: string; label: string; href: string }[];
}

export interface Corpus {
  /**
   * Everything the validator found. Errors have already stopped the build by the time anyone can
   * read this, so in practice it holds the ontology-vocabulary warnings.
   */
  diagnostics: Diagnostic[];
  nodes: Node[];
  byId: Map<string, Node>;
  classes: OntologyClass[];
  byClass: Map<string, OntologyClass>;
  sources: Source[];
  bySlug: Map<string, Source>;
  decisions: Decision[];
  byDecision: Map<string, Decision>;
}

let cached: Corpus | null = null;

/**
 * Turn a corpus-relative link target into a URL on this site, or off it.
 *
 * The corpus links by relative file path — `../parameter/twenty-mill-floor.yml` — because it is
 * authored as files and those links have to work in an editor. On the web they have to become
 * routes. Ten shapes occur, and this table is the closed enumeration of them:
 *
 * | Shape | Written by | Becomes |
 * |---|---|---|
 * | `../class/name.yml` | nodes | the node's page |
 * | `../corpus/class/name.yml` | catalog entries | the node's page |
 * | `../class.ont.yml` | nodes | the class page |
 * | `name.yml` | nodes | a sibling node in the same class |
 * | `../../catalog/name.md` | nodes | the source's page |
 * | `name.md` | catalog entries | a sibling catalog entry |
 * | `../corpus/class/` | catalog entries | the class page |
 * | `class` (bare) | nodes, in the ontology's own style | the class page |
 * | `../../../crates/…` | nodes | the repository, on GitHub |
 * | `../decisions/name.yml` | nodes, catalog entries, decisions | the decision's page |
 * | `name.yml` | decisions | a sibling decision |
 * | `../../skills/name.md` | nodes, catalog entries | the repository |
 * | `ACTIONS.md` | nodes | the repository, on GitHub |
 *
 * The last two rows are the ones worth explaining. {@link UNPUBLISHED} and the per-class
 * `ACTIONS.md` files are in-repository documents that this site deliberately never publishes —
 * `loadCorpus` reads only `*.yml` under `corpus/` — so they cannot become a page here. Sending
 * them to GitHub is the honest resolution: the document exists, it is just not part of the wiki.
 *
 * # Why the table has to stay complete
 *
 * `resolved` is false for anything this cannot place, and {@link checkTargets} turns that into a
 * build error. That gate was written before the table was, and for a long time nothing read the
 * flag — so four of the shapes above went unhandled and 40 raw relative `href`s shipped, each one
 * looking in the markup exactly like a working link and each one a 404. A reader who trusts this
 * table to be exhaustive is making the same assumption the gate makes; both have to be true.
 */

/**
 * Passed as `fromClass` when the prose being rendered is a catalog entry rather than a node.
 *
 * Catalog entries live in `.yidam/catalog/`, beside `corpus/` rather than inside a class
 * directory, so "the class this link is relative to" has no answer for them. This used to be the
 * empty string, which silently built `/wiki//twenty-mill-floor` — a double slash that resolves to
 * nothing on a static host — the moment a catalog entry wrote a bare sibling reference. Naming it
 * lets the sibling branches tell the two contexts apart and refuse the one that makes no sense.
 *
 * Safe as a sentinel because no ontology class is called `catalog`, and none can be: the loader
 * takes class names from `*.ont.yml` files under `corpus/`.
 */
export const FROM_CATALOG = "catalog";

/**
 * Passed as `fromClass` when the prose being rendered is a decision record.
 *
 * Decision records live in `.yidam/decisions/`, a third sibling of `corpus/` and `catalog/`, and
 * they need a sentinel of their own for exactly the reason {@link FROM_CATALOG} does: a bare
 * `name.yml` written in one of them means **another decision record**, not a node in some class.
 * Nine of the twenty-four cite a sibling that way. Without this they would resolve through the
 * sibling-node branch to `/wiki/<some class>/the-three-streams-of-mr81`, which is `resolved: true`
 * and a 404 — the same failure `FROM_CATALOG` was introduced to stop, one directory over.
 *
 * Safe as a sentinel for the same reason: no ontology class is called `decision`, and none can be,
 * because the loader takes class names from `*.ont.yml` files under `corpus/`.
 */
export const FROM_DECISION = "decision";

export interface ResolvedTarget {
  href: string;
  /** `<class>/<name>` when the target is a corpus node; null for classes, sources and code. */
  id: string | null;
  /** False when the shape was not recognised and `href` is the raw target. */
  resolved: boolean;
}

export function resolveTarget(target: string, fromClass: string): ResolvedTarget {
  const clean = target.replace(/^\.\//, "");

  // `../../../crates/dispersion/tests/…` — out of the corpus and into the code that proves it.
  const escaped = clean.match(/^(?:\.\.\/)+((?:crates|docs|agents)\/.*)$/);
  if (escaped) return { href: `${REPO}/${escaped[1]}`, id: null, resolved: true };

  // `../../skills/deduction.md` — a document in a `.yidam/` subtree this site does not publish.
  // Must precede the `class/name.yml` branch, which would otherwise read the subtree name as an
  // ontology class and emit `/wiki/skills/…`: resolved, site-absolute, and a 404.
  const unpublished = clean.match(new RegExp(`(?:^|/)(${UNPUBLISHED.join("|")})/(.+)$`));
  if (unpublished) {
    return { href: `${REPO}/.yidam/${unpublished[1]}/${unpublished[2]}`, id: null, resolved: true };
  }

  // `../decisions/the-order-was-never-the-states.yml` — a decision record, which is a page here
  // now. Named explicitly and placed above the `class/name.yml` branch for the same reason the
  // subtree above is: left to fall through, `decisions` reads as an ontology class and builds
  // `/wiki/decisions/…`, plural, which is the shape the old comment here warned about.
  const decision = clean.match(/(?:^|\/)decisions\/([^/]+)\.yml$/);
  if (decision) return { href: routes.wikiDecision(decision[1]!), id: null, resolved: true };

  // `../../catalog/ocg-white-paper-013.md`
  const source = clean.match(/(?:^|\/)catalog\/([^/]+)\.md$/);
  if (source) return { href: routes.wikiSource(source[1]!), id: null, resolved: true };

  // `../corpus/metric/` — a catalog entry naming a whole class by its directory.
  const directory = clean.match(/(?:^|\/)corpus\/([a-z0-9-]+)\/$/);
  if (directory) return { href: routes.wikiClass(directory[1]!), id: null, resolved: true };

  // `../metric.ont.yml` — the class, whose page is also its index.
  const ontology = clean.match(/([^/]+)\.ont\.yml$/);
  if (ontology) return { href: routes.wikiClass(ontology[1]!), id: null, resolved: true };

  // `../parameter/twenty-mill-floor.yml`
  const other = clean.match(/(?:^|\/)([a-z0-9-]+)\/([^/]+)\.yml$/);
  if (other) {
    const id = `${other[1]}/${other[2]}`;
    return { href: routes.wikiNode(other[1]!, other[2]!), id, resolved: true };
  }

  // `northern-local-perry.yml` — a sibling node, so the same class. Refused from a catalog entry,
  // which has no class for the sibling to be in; see {@link FROM_CATALOG}. From a decision record
  // it is a sibling *decision*, which is how nine of the twenty-four cite each other.
  const sibling = clean.match(/^([^/]+)\.yml$/);
  if (sibling && fromClass === FROM_DECISION) {
    return { href: routes.wikiDecision(sibling[1]!), id: null, resolved: true };
  }
  if (sibling && fromClass !== FROM_CATALOG && fromClass !== "") {
    const id = `${fromClass}/${sibling[1]}`;
    return { href: routes.wikiNode(fromClass, sibling[1]!), id, resolved: true };
  }

  // A bare `*.md`, which means one of two unrelated things depending on where it was written.
  // From a catalog entry it is another catalog entry, and 19 of them cite each other this way.
  // From a node it is a file sitting in that node's own class directory, which is `ACTIONS.md` or
  // `README.md` — repository documents, never pages here.
  const markdown = clean.match(/^([^/]+)\.md$/);
  if (markdown) {
    if (fromClass === FROM_CATALOG) {
      return { href: routes.wikiSource(markdown[1]!), id: null, resolved: true };
    }
    if (fromClass !== "") {
      return { href: `${REPO}/.yidam/corpus/${fromClass}/${markdown[1]}.md`, id: null, resolved: true };
    }
  }

  // `education-agency` — a bare class name, in the ontology's own style. Whether that class
  // exists is checked by the validator, which has the class list; this only decides the shape.
  if (/^[a-z][a-z0-9-]*$/.test(clean)) {
    return { href: routes.wikiClass(clean), id: null, resolved: true };
  }

  return { href: target, id: null, resolved: false };
}

/** Whether a resolved href points at a catalog entry rather than at a node or off the site. */
const isSource = (href: string) => href.startsWith("/wiki/source/");

/**
 * Inline markdown link targets in a body of prose.
 *
 * Keeps two kinds: other corpus nodes, and catalog sources. Sources are kept because provenance is
 * written both ways in this corpus — six nodes cite the payment report through a structured
 * `sourced-from` edge and others cite theirs inline in the middle of a sentence — and a "cited by"
 * list built from only one of those is wrong for every entry that uses the other.
 *
 * Links out of the repository are dropped: they are references, not edges in this graph.
 */
function mentioned(text: string, fromClass: string): Edge[] {
  const edges: Edge[] = [];
  const seen = new Set<string>();
  for (const match of text.matchAll(/\[([^\]]+)\]\(([^)\s]+)\)/g)) {
    const [, label, target] = match;
    const { href, id } = resolveTarget(target!, fromClass);
    if (!id && !isSource(href)) continue;
    const key = id ?? href;
    if (seen.has(key)) continue;
    seen.add(key);
    edges.push({ id, href, label: label!, stated: false });
  }
  return edges;
}

function readNode(className: string, file: string, report: Diagnostic[]): Node {
  const relative = `.yidam/corpus/${className}/${file}`;
  const raw = readFileSync(join(CORPUS, className, file), "utf8");
  let parsed: Record<string, unknown>;
  try {
    parsed = YAML.parse(raw) as Record<string, unknown>;
  } catch (error) {
    report.push({
      file: relative,
      severity: "error",
      message: `not valid YAML: ${error instanceof Error ? error.message.split("\n")[0] : String(error)}`,
    });
    parsed = {};
  }

  /*
   * Validated against the schema, and the *result is discarded*: the reader below is tolerant on
   * purpose, so a corpus with one bad node still renders the other 61 while the diagnostics say
   * what is wrong with it. The schema decides whether the build is allowed to finish; the reader
   * decides what a page shows in the meantime. Conflating the two would mean a single stray
   * character took the whole wiki down with no page left to explain why.
   */
  const validation = NodeSchema.safeParse(parsed);
  if (!validation.success) report.push(...diagnose(relative, validation.error));

  // The directory is the class. A `class:` field disagreeing with it means one of the two is a
  // typo, and every consumer that trusts the wrong one is silently mis-filed.
  if (typeof parsed.class === "string" && parsed.class !== className) {
    report.push({
      file: relative,
      severity: "error",
      message: `declares class "${parsed.class}" but lives in ${className}/`,
    });
  }

  const name = file.replace(/\.yml$/, "");
  const id = `${className}/${name}`;
  const summary = String(parsed.summary ?? "").trim();
  const description = String(parsed.description ?? "");
  const revisions = readRevisions(parsed.revisions);
  const unfilled = readUnfilled(parsed.unfilled);
  const figures = readFigures(parsed.figures);
  const rawLinks = parsed.links;

  report.push(...lintProse(relative, summary, description));

  const stated: Edge[] = Array.isArray(rawLinks)
    ? rawLinks.flatMap((link) => {
        const entry = link as { target?: string; relationship?: string };
        if (!entry?.target) return [];
        const { href, id: to } = resolveTarget(entry.target, className);
        return [
          {
            id: to,
            href,
            // A relative file path is not a label. Nodes get theirs looked up once the whole
            // corpus is read; a class link — the `instance-of` edge every node carries — is named
            // here, since it resolves to a class page rather than to a node.
            label: to ?? entry.target.replace(/^.*\/|\.ont\.yml$|\.yml$|\.md$/g, ""),
            ...(entry.relationship ? { relationship: entry.relationship } : {}),
            stated: true,
          },
        ];
      })
    : [];

  // The prose-links node, plus every inline citation everywhere else. Deduplicated against what
  // the node already declares, so a relationship the author named is never demoted to a mention.
  const properties = Object.entries(
    (parsed.properties ?? {}) as Record<string, unknown>,
  ).map(([key, value]) => ({ name: key, value: String(value).trim() }));

  /*
   * Every place this corpus writes a link, which is four places and not one.
   *
   * `links:` is the structured one. The rest are prose: the description, the `findings` block two
   * nodes carry, individual property values, and — for the one node that writes its whole link
   * list as a paragraph — that paragraph. Each of those was found the hard way, by a source page
   * or a backlink list coming up empty for a node that plainly referenced something.
   */
  const declared = new Set(stated.map((edge) => edge.id ?? edge.href));
  const linkText = [
    description,
    typeof rawLinks === "string" ? rawLinks : "",
    parsed.findings == null ? "" : String(parsed.findings),
    // Revisions cite heavily — `found_by` is a path or a catalog record in nearly every entry —
    // and a citation that only appears in a withdrawal is still a citation. Leaving them out
    // would drop a source's backlink the moment its only mention became a correction.
    ...revisions.flatMap((revision) => [
      revision.was,
      revision.now,
      revision.found_by,
      revision.reach ?? "",
    ]),
    ...properties.map((property) => property.value),
  ].join("\n\n");
  const seen = new Set(declared);
  const inline = mentioned(linkText, className).filter((edge) => {
    const key = edge.id ?? edge.href;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });

  return {
    id,
    className,
    name,
    label: String(parsed.label ?? name),
    summary,
    description,
    linkText,
    properties,
    findings: parsed.findings == null ? null : String(parsed.findings),
    revisions,
    unfilled,
    figures,
    out: [...stated, ...inline],
    in: [],
  };
}

/** One number in this node's prose, bound to the crate that computes it. */
export interface BoundFigure {
  /** The `crates/figures.json` key. */
  key: string;
  /** What this node says the crate computes. */
  value: number;
  /** Which prose field carries it — `description`, `findings`, or a property name. */
  field: string;
  /** The phrase that field writes, verbatim. */
  as_written: string;
}

/**
 * The `figures:` block, read the same defensive way `unfilled:` and `revisions:` are.
 *
 * With one difference that matters: an entry missing `value` is **dropped**, not defaulted. A
 * default of `0` would parse, bind, and then be compared against the manifest — where it would
 * either fail with a message about the wrong thing or, for a figure that really is zero, pass. The
 * schema is what rejects a malformed entry at build time; this decides only what a consumer sees,
 * and what it must not see is an entry the author never wrote.
 */
function readFigures(raw: unknown): BoundFigure[] {
  if (!Array.isArray(raw)) return [];
  return raw.flatMap((entry) => {
    if (typeof entry !== "object" || entry === null) return [];
    const record = entry as Record<string, unknown>;
    const text = (key: string): string =>
      typeof record[key] === "string" ? (record[key] as string).trim() : "";
    const key = text("key");
    const field = text("field");
    const as_written = text("as_written");
    const value = record.value;
    if (key === "" || field === "" || as_written === "" || typeof value !== "number") return [];
    return [{ key, value, field, as_written }];
  });
}

/** One thing a node does not hold. */
export interface Unfilled {
  /** The missing fact, as a short noun phrase — `established`, `the department's typology code`. */
  field: string;
  /** Where the value lives and what it would take, when that is known. */
  why: string | null;
}

/**
 * The `unfilled:` block, read the same defensive way `revisions:` is.
 *
 * A node with none is the ordinary case, so absence of the key is not an error and an entry
 * missing its `field` is dropped rather than thrown on — the schema is what rejects a malformed
 * one at build time, and this runs on data the schema has already accepted.
 */
function readUnfilled(raw: unknown): Unfilled[] {
  if (!Array.isArray(raw)) return [];
  return raw.flatMap((entry) => {
    if (typeof entry !== "object" || entry === null) return [];
    const record = entry as Record<string, unknown>;
    const field = typeof record.field === "string" ? record.field.trim() : "";
    if (field === "") return [];
    const why = typeof record.why === "string" ? record.why.trim() : "";
    return [{ field, why: why === "" ? null : why }];
  });
}

/**
 * The `revisions:` block, read defensively.
 *
 * Same posture as everything else in this reader: the schema decides whether the build finishes,
 * and this decides what a page can show while the diagnostics say what is wrong. An entry missing
 * `found_by` is a schema error and still renders its `was` and `now`, because a half-recorded
 * withdrawal is more useful on the page than no withdrawal at all.
 */
function readRevisions(raw: unknown): Revision[] {
  if (!Array.isArray(raw)) return [];
  return raw.flatMap((entry) => {
    if (entry == null || typeof entry !== "object") return [];
    const record = entry as Record<string, unknown>;
    const text = (key: string): string => String(record[key] ?? "").trim();
    const was = text("was");
    const now = text("now");
    if (was === "" && now === "") return [];
    const reach = text("reach");
    return [{ was, now, found_by: text("found_by"), ...(reach === "" ? {} : { reach }) }];
  });
}

/**
 * The phrases that say a paragraph is about this repository rather than about Ohio.
 *
 * Not a stylistic preference. `.yidam/.vendor/prelude/guidelines/directories.md` says outright
 * that "anything that describes how the repo operates rather than what it knows" does not belong
 * in the corpus, and 213 of 905 paragraphs did. They are real provenance and they are kept —
 * `revisions:` is where they go.
 */
const APPARATUS = /\bthis corpus\b|\bthis node\b|\bthis repository\b/i;

/**
 * A run of bold capitals, which is how a retraction used to compete for attention.
 *
 * 130 of these across 33 nodes. They were shouting because nothing else in a 2,000-word field
 * could carry emphasis; once a withdrawal has its own block the capitals are redundant with the
 * structure and read on the page as exactly what they are.
 *
 * # The two exclusions, both found by running it
 *
 * No lower-case letter is not enough on its own. A first version matched `**CSI**`, `**ATSI**`,
 * `**FY2025**` and `**FY 2020**` — acronyms and year labels the corpus bolds constantly and which
 * are not shouting at anyone. So: **at least one space**, which drops the acronyms, and **ten
 * characters**, which drops `**FY 2020**` while keeping `**STILL OPEN.**` at eleven.
 *
 * Both thresholds are set by real strings rather than chosen, and they are the reason this is a
 * warning and not an error — the next legitimate bolded initialism with a space in it will trip
 * it, and a build that stopped for that would be worse than a line in a report.
 */
const CAPS_LEAD = /\*\*(?=[^a-z*]{10,}\*\*)[A-Z][^a-z*]*\s[^a-z*]*\*\*/;

/**
 * What a node's prose may not be, checked at read time.
 *
 * # Why these are warnings and the schema's are errors
 *
 * The schema's two summary rules — a word cap and no markdown links — are mechanical facts about a
 * string, and a violation is always a defect. These three are judgements about prose with a real
 * false-positive rate: a node may legitimately say "this node" while quoting a source, and a
 * long description may be long because Ohio is complicated rather than because the genres are
 * mixed. Reported and rendered, on the same principle the ontology-vocabulary warnings use.
 *
 * The prefix check is the one that matters most and looks least important. The cheap way to write
 * 101 summaries is to paste the first sentence of each description, which produces a corpus where
 * every node page says the same thing twice and no reader is better off.
 */
function lintProse(file: string, summary: string, description: string): Diagnostic[] {
  const found: Diagnostic[] = [];
  const warn = (message: string): void => {
    found.push({ file, severity: "warning", kind: "prose", message });
  };

  const opening = description.trim().slice(0, Math.max(summary.length, 1));
  if (summary !== "" && opening.toLowerCase() === summary.toLowerCase()) {
    warn("summary: is the opening of description: verbatim — the lead has to be written, not cut");
  }

  const length = description.split(/\s+/).filter((word) => word !== "").length;
  if (length > DESCRIPTION_WARN_WORDS) {
    warn(
      `description: ${length} words. Over ${DESCRIPTION_WARN_WORDS} is usually two subjects in ` +
        "one node or a genre that belongs in findings: or revisions:",
    );
  }

  if (CAPS_LEAD.test(description)) {
    warn("description: carries a shouted lead — a withdrawn claim belongs in revisions:");
  }
  if (APPARATUS.test(description)) {
    warn(
      "description: refers to the corpus rather than to Ohio — that is apparatus, and it belongs " +
        "in revisions: or findings:",
    );
  }

  return found;
}

/**
 * Where a description stops being one subject.
 *
 * The vendored guideline says "2–10 sentences is often right", which is roughly 250 words, and the
 * median node was 381. Setting the warning at the guideline would have flagged two thirds of the
 * corpus and been ignored; 400 flags the nodes where length is actually the symptom.
 */
const DESCRIPTION_WARN_WORDS = 400;

function readClass(file: string, report: Diagnostic[]): Omit<OntologyClass, "nodes"> {
  const relative = `.yidam/corpus/${file}`;
  let parsed: Record<string, unknown> = {};
  try {
    parsed = YAML.parse(readFileSync(join(CORPUS, file), "utf8")) as Record<string, unknown>;
  } catch (error) {
    report.push({
      file: relative,
      severity: "error",
      message: `not valid YAML: ${error instanceof Error ? error.message.split("\n")[0] : String(error)}`,
    });
  }

  const validation = OntologyClassSchema.safeParse(parsed);
  if (!validation.success) report.push(...diagnose(relative, validation.error));

  const foundational = parsed.foundational_type as { ontology?: string; type?: string } | undefined;
  return {
    className: String(parsed.class ?? file.replace(/\.ont\.yml$/, "")),
    label: String(parsed.label ?? ""),
    description: String(parsed.description ?? ""),
    edgePolicy: parsed.edge_policy === "exhaustive" ? "exhaustive" : "characteristic",
    foundationalType: foundational?.type
      ? `${foundational.type}${foundational.ontology ? ` (${foundational.ontology})` : ""}`
      : null,
    properties: ((parsed.properties ?? []) as Record<string, string>[]).map((p) => ({
      name: String(p.name ?? ""),
      type: String(p.type ?? ""),
      description: String(p.description ?? ""),
    })),
    edges: ((parsed.edges ?? []) as Record<string, string>[]).map((e) => ({
      relationship: String(e.relationship ?? ""),
      target: String(e.target ?? ""),
      direction: String(e.direction ?? ""),
      description: String(e.description ?? ""),
    })),
  };
}

function readSource(file: string): Source {
  const body = readFileSync(join(CATALOG, file), "utf8");
  const slug = file.replace(/\.md$/, "");
  const heading = body.match(/^#\s+(.+)$/m);
  return {
    slug,
    title: heading?.[1]?.trim() ?? slug,
    body: heading ? body.replace(heading[0], "").trim() : body.trim(),
    citedBy: [],
  };
}

/**
 * The prose fields a decision record can carry, in the order they are meant to be read.
 *
 * Not every record carries every one, and the variation is not sloppiness: the four connector
 * records state a `rationale` where the rest state `consequences` and `alternatives`, two carry an
 * `amendment` recording a later revision, and `ontology` carries a `corpus_depth` integer that is
 * not prose at all. So this is the order and the labels, and a record renders the intersection —
 * an absent field is absent rather than an empty heading.
 *
 * `summary` is deliberately not here. It leads the page rather than sitting in the sequence.
 */
const DECISION_SECTIONS: { name: string; label: string }[] = [
  { name: "context", label: "Context" },
  { name: "decision", label: "The decision" },
  { name: "rationale", label: "Rationale" },
  { name: "consequences", label: "Consequences" },
  { name: "amendment", label: "Amendment" },
  { name: "alternatives", label: "Alternatives considered" },
];

/**
 * One decision record, read straight off the YAML.
 *
 * Parsed with the same loader the nodes use, and then read defensively: these files are hand-
 * written prose in block scalars and the field set genuinely varies between them, so a missing
 * field is a shape this reader expects rather than a fault.
 */
function readDecision(file: string): Decision {
  const raw = readFileSync(join(DECISIONS, file), "utf8");
  const parsed = (YAML.parse(raw) ?? {}) as Record<string, unknown>;
  const text = (key: string): string => {
    const value = parsed[key];
    return typeof value === "string" ? value.trim() : "";
  };

  const sections = DECISION_SECTIONS.map(({ name, label }) => ({
    name,
    label,
    body: text(name),
  })).filter((section) => section.body !== "");

  const summary = text("summary");
  return {
    slug: file.replace(/\.yml$/, ""),
    summary,
    sections,
    linkText: [summary, ...sections.map((s) => s.body)].join("\n\n"),
    corrections: countCorrections([summary, ...sections.map((s) => s.body)].join("\n\n")),
    citedBy: [],
  };
}

/**
 * How many blockquotes in a markdown source **open** with strong emphasis.
 *
 * The same rule `markCorrections` applies to rendered HTML, applied to the source — because
 * `loadCorpus` is synchronous and the markdown processor is not, so a decision's correction count
 * has to be available before anything is rendered. `prose.spec.ts` holds the two in agreement over
 * every record in the corpus, which is the only thing that keeps one rule from becoming two.
 *
 * **Opens** is doing the work. A first attempt counted every line matching `> **` and got five
 * corrections out of a record that has four, because one continuation line inside a correction
 * begins with a bolded word that happened to fall at a line break:
 *
 * ```
 * > repository's own F-33 panel agrees: West Geauga gains 208 pupils in FY2021 while Chardon
 * > **loses** 110. The claim came from a judge's failure-mode analysis …
 * ```
 *
 * A markdown blockquote is a run of consecutive `>` lines, so a line only opens one if the line
 * before it is not itself part of the quote. That is the whole difference, and without it the
 * count is wrong in exactly the records that carry the most corrections.
 */
export function countCorrections(markdown: string): number {
  const lines = markdown.split("\n");
  let corrections = 0;
  for (const [index, line] of lines.entries()) {
    if (!/^\s*>\s*\*\*/.test(line)) continue;
    const previous = lines[index - 1];
    if (previous !== undefined && /^\s*>/.test(previous)) continue;
    corrections += 1;
  }
  return corrections;
}

/**
 * The checks a per-file schema cannot make, because they need the rest of the corpus.
 *
 * # Errors, and why these four
 *
 * A dangling target, a node in no class, a missing `instance-of`, and an unrecognised target shape
 * all produce the same visible symptom: a link that looks entirely normal and goes nowhere. That
 * symptom is invisible to the author, survives review, and is only found when someone clicks. All
 * four stop the build.
 *
 * # Warnings, and why not errors
 *
 * Whether a relationship or a property is declared in the class's own ontology is measured, not
 * assumed: **94% of properties** are declared and only **68% of relationships** are. The property
 * vocabulary is effectively closed; the relationship vocabulary plainly is not — ninety-odd
 * distinct relationships, most used once, describing genuinely different connections. Enforcing
 * the ontology's list would reject a third of the corpus's edges, so this reports and moves on.
 *
 * That asymmetry is itself the finding: if the relationship vocabulary is meant to be open, the
 * ontology's `edges` are documentation rather than a constraint, and it would be worth saying so
 * in the ontology files.
 */
function validate(
  nodes: Node[],
  byId: Map<string, Node>,
  classes: Map<string, Omit<OntologyClass, "nodes">>,
  report: Diagnostic[],
): void {
  for (const node of nodes) {
    const file = `.yidam/corpus/${node.id}.yml`;
    const ontology = classes.get(node.className);
    if (!ontology) {
      report.push({
        file,
        severity: "error",
        message: `no ontology class "${node.className}" — every node must be an instance of one`,
      });
      continue;
    }

    const stated = node.out.filter((edge) => edge.stated);
    if (!stated.some((edge) => edge.relationship === "instance-of")) {
      report.push({
        file,
        severity: "error",
        message: `no instance-of edge; add "target: ../${node.className}.ont.yml"`,
      });
    }

    const declaredEdges = new Set(ontology.edges.map((edge) => edge.relationship));
    const declaredProps = new Set(ontology.properties.map((property) => property.name));

    for (const edge of stated) {
      if (edge.id && !byId.get(edge.id)) {
        report.push({ file, severity: "error", message: `link to "${edge.id}", which does not exist` });
      }
      if (!edge.href.startsWith("/") && !edge.href.startsWith("http")) {
        report.push({
          file,
          severity: "error",
          message: `link target "${edge.label}" is not a shape this site can turn into a URL`,
        });
      }
      // `instance-of` and `sourced-from` are corpus-wide conventions rather than per-class edges,
      // so no ontology declares them and complaining about them would be noise on every node.
      const universal = edge.relationship === "instance-of" || edge.relationship === "sourced-from";
      if (
        edge.relationship &&
        !universal &&
        !declaredEdges.has(edge.relationship) &&
        // Only for classes that have declared their vocabulary closed. The rest say outright that
        // `edges:` documents what they are defined by rather than bounding what may be said.
        ontology.edgePolicy === "exhaustive"
      ) {
        report.push({
          file,
          severity: "error",
          message:
            `relationship "${edge.relationship}" is not declared by ${node.className}.ont.yml, ` +
            `which sets edge_policy: exhaustive`,
        });
      }
    }

    /*
     * Properties are the other way round from relationships, and the numbers say so: 94% were
     * declared, and the handful that were not turned out to be four genuine omissions in the
     * ontologies plus five year-stamped observation snapshots. The omissions are declared now, so
     * this check is expected to be silent — which is what makes it worth keeping. An undeclared
     * property name is a typo or a decision, not background noise.
     */
    for (const property of node.properties) {
      if (
        declaredProps.has(property.name) ||
        OBSERVATION_PROPERTY.test(property.name) ||
        UNIVERSAL_PROPERTIES.has(property.name)
      ) {
        continue;
      }
      report.push({
        file,
        severity: "warning",
        kind: "vocabulary",
        message: `property "${property.name}" is not declared by ${node.className}.ont.yml`,
      });
    }
  }
}

/**
 * Every inline link target in the corpus is a shape {@link resolveTarget} recognises.
 *
 * # Why this is an error rather than a warning
 *
 * `ResolvedTarget.resolved` was documented from the start as the guard that stops an unplaceable
 * link from shipping, and for a long time nothing read it. `prose.ts` used `.href` unconditionally,
 * so an unrecognised shape became a raw relative `href` — `<a href="ocg-white-paper-013.md">` on a
 * page at `/wiki/source/…`, which resolves to `/wiki/source/ocg-white-paper-013.md` and 404s while
 * the target page sits one character away. 40 of those were live, and the unit test that exists to
 * catch exactly this skipped them, because a raw relative href is not site-absolute and the test
 * read "not site-absolute" as "external, therefore none of our business".
 *
 * A link that looks right and goes nowhere is invisible to the author, survives review, and is
 * found by a reader clicking it. That is the same argument the four checks in {@link validate}
 * make, so this is the same severity.
 *
 * # Why it scans prose rather than edges
 *
 * {@link mentioned} drops any target it cannot place — it keeps nodes and sources and nothing
 * else — so by the time a link is an {@link Edge} the unresolvable ones are already gone. The
 * shapes have to be checked where they are written.
 */
function checkTargets(
  nodes: Node[],
  classes: Omit<OntologyClass, "nodes">[],
  sources: Source[],
  decisions: Decision[],
  report: Diagnostic[],
): void {
  const scan = (text: string, fromClass: string, file: string) => {
    for (const match of text.matchAll(/\]\(([^)\s]+)\)/g)) {
      const target = match[1]!;
      if (/^(https?:|mailto:|#|\/)/.test(target)) continue;
      if (resolveTarget(target, fromClass).resolved) continue;
      report.push({
        file,
        severity: "error",
        message:
          `link target "${target}" is not a shape this site can turn into a URL. ` +
          `The shapes that are are listed above resolveTarget in src/lib/corpus.ts.`,
      });
    }
  };

  for (const node of nodes) scan(node.linkText, node.className, `.yidam/corpus/${node.id}.yml`);
  for (const entry of classes) {
    scan(entry.description, entry.className, `.yidam/corpus/${entry.className}.ont.yml`);
  }
  for (const source of sources) scan(source.body, FROM_CATALOG, `.yidam/catalog/${source.slug}.md`);
  for (const decision of decisions) {
    scan(decision.linkText, FROM_DECISION, `.yidam/decisions/${decision.slug}.yml`);
  }
}

/** Read, link, and index the corpus. Memoized — the wiki pages number in the hundreds. */
export function loadCorpus(): Corpus {
  if (cached) return cached;

  const report: Diagnostic[] = [];
  const entries = readdirSync(CORPUS, { withFileTypes: true });
  const classes = entries
    .filter((e) => e.isFile() && e.name.endsWith(".ont.yml"))
    .map((e) => readClass(e.name, report));

  const nodes: Node[] = [];
  for (const dir of entries.filter((e) => e.isDirectory())) {
    for (const file of readdirSync(join(CORPUS, dir.name))) {
      if (file.endsWith(".yml")) nodes.push(readNode(dir.name, file, report));
    }
  }

  const byId = new Map(nodes.map((n) => [n.id, n]));
  const classLabels = new Map(classes.map((c) => [c.className, c.label]));
  const byClassName = new Map(classes.map((c) => [c.className, c]));

  validate(nodes, byId, byClassName, report);

  // Give every edge a name a reader recognises. Until now they carry a slug or a file stem,
  // because at read time a node knows only where it points and not what is there.
  for (const node of nodes) {
    for (const edge of node.out) {
      const target = edge.id ? byId.get(edge.id) : undefined;
      if (target) {
        edge.label = target.label;
        continue;
      }
      const asClass = classLabels.get(edge.label);
      if (asClass) edge.label = asClass;
    }
  }

  // Backlinks. Every edge that lands on a node this corpus actually holds becomes an inbound
  // entry on it; edges pointing at nothing are dropped here and reported by the link check rather
  // than rendered as a link to a page that does not exist.
  for (const node of nodes) {
    for (const edge of node.out) {
      if (!edge.id) continue;
      const target = byId.get(edge.id);
      if (!target) continue;
      target.in.push({
        id: node.id,
        label: node.label,
        href: routes.wikiNode(node.className, node.name),
        ...(edge.relationship ? { relationship: edge.relationship } : {}),
        stated: edge.stated,
      });
    }
  }

  const sources = existsSync(CATALOG)
    ? readdirSync(CATALOG)
        .filter((f) => f.endsWith(".md") && f !== "README.md")
        .map(readSource)
    : [];
  const bySlug = new Map(sources.map((s) => [s.slug, s]));

  const decisions = existsSync(DECISIONS)
    ? readdirSync(DECISIONS)
        .filter((f) => f.endsWith(".yml"))
        .map(readDecision)
        .sort((a, b) => a.slug.localeCompare(b.slug))
    : [];
  const byDecision = new Map(decisions.map((d) => [d.slug, d]));

  // After the sources are read, because catalog entries are the largest single writer of link
  // targets and half the shapes only they use.
  checkTargets(nodes, classes, sources, decisions, report);

  /*
   * Which node, catalog entry or other decision links to each decision record.
   *
   * Scanned here rather than taken from the edge lists, because a link to a decision is not an
   * edge in the corpus graph and should not become one: it is a citation of a working document,
   * like a footnote, and putting it in `node.out` would show it in the relationship lists on node
   * pages beside genuine ontology edges. The inbound list is what the reader of a decision wants —
   * "who relies on this" — and it costs one pass.
   */
  const cite = (target: string, entry: { id: string; label: string; href: string }) => {
    const decision = byDecision.get(target);
    if (!decision) return;
    if (decision.citedBy.some((c) => c.id === entry.id)) return;
    decision.citedBy.push(entry);
  };
  const DECISION_HREF = "/wiki/decision/";
  const scanCitations = (text: string, fromClass: string, entry: { id: string; label: string; href: string }) => {
    for (const match of text.matchAll(/\]\(([^)\s]+)\)/g)) {
      const raw = match[1]!;
      if (/^(https?:|mailto:|#|\/)/.test(raw)) continue;
      const { href, resolved } = resolveTarget(raw, fromClass);
      if (!resolved || !href.startsWith(DECISION_HREF)) continue;
      cite(href.slice(DECISION_HREF.length), entry);
    }
  };
  for (const node of nodes) {
    scanCitations(node.linkText, node.className, {
      id: node.id,
      label: node.label,
      href: routes.wikiNode(node.className, node.name),
    });
  }
  for (const source of sources) {
    scanCitations(source.body, FROM_CATALOG, {
      id: `catalog/${source.slug}`,
      label: source.title,
      href: routes.wikiSource(source.slug),
    });
  }
  for (const decision of decisions) {
    scanCitations(decision.linkText, FROM_DECISION, {
      id: `decision/${decision.slug}`,
      label: decision.slug,
      href: routes.wikiDecision(decision.slug),
    });
  }
  for (const decision of decisions) {
    decision.citedBy.sort((a, b) => a.label.localeCompare(b.label));
  }

  // Which nodes cite which source. Taken from the edge list, which by now holds both the
  // structured `sourced-from` links and the citations written inline in prose — this corpus uses
  // each form for different sources, and a list built from only one of them is empty for every
  // entry that happens to use the other.
  for (const node of nodes) {
    for (const edge of node.out) {
      if (!isSource(edge.href)) continue;
      bySlug.get(edge.href.slice("/wiki/source/".length))?.citedBy.push({
        id: node.id,
        label: node.label,
        href: routes.wikiNode(node.className, node.name),
      });
    }
  }

  const withNodes: OntologyClass[] = classes.map((c) => ({
    ...c,
    nodes: nodes
      .filter((n) => n.className === c.className)
      .sort((a, b) => a.label.localeCompare(b.label)),
  }));

  /*
   * The gate, and it is the same shape as the formula's: errors stop the build.
   *
   * Four authoring defects reached this repository because nothing read the corpus end to end, and
   * every one of them was found days later by a page rendering wrong. There is no reason for that
   * to be the discovery mechanism when a parse can say so in a second.
   */
  const errors = report.filter((d) => d.severity === "error");
  if (errors.length > 0) {
    throw new Error(
      `The corpus does not validate, and the build is stopping.\n\n` +
        errors.map((d) => `  ${d.file}\n    ${d.message}`).join("\n") +
        `\n\nThe shapes are declared in src/lib/schema/corpus.ts. Run \`pnpm schemas\` to write\n` +
        `JSON Schema files an editor can check these against while they are being written.`,
    );
  }

  /*
   * Not fatal, and deliberately not silent — and counted by kind, because two unrelated questions
   * arrive here.
   *
   * Vocabulary drift is a question for whoever owns an ontology. Prose drift is a question for
   * whoever is authoring a node: a genre sitting in the wrong field. Summing them under either
   * name gives a number that is true of nothing, which is the exact defect `count_tag` and the
   * connector table were each fixed for.
   */
  const warnings = report.filter((d) => d.severity === "warning");
  if (warnings.length > 0) {
    const prose = warnings.filter((d) => d.kind === "prose").length;
    const parts = [
      prose > 0 ? `${prose} node${prose === 1 ? "" : "s"} carry prose in the wrong field` : "",
      warnings.length - prose > 0
        ? `${warnings.length - prose} use vocabulary their ontology does not declare`
        : "",
    ].filter((part) => part !== "");
    console.warn(`\n  ${parts.join(", ")}.\n  Run \`pnpm corpus:report\` for the list.\n`);
  }

  cached = {
    diagnostics: report,
    nodes: nodes.sort((a, b) => a.label.localeCompare(b.label)),
    byId,
    classes: withNodes.sort((a, b) => a.label.localeCompare(b.label)),
    byClass: new Map(withNodes.map((c) => [c.className, c])),
    sources: sources.sort((a, b) => a.title.localeCompare(b.title)),
    bySlug,
    decisions,
    byDecision,
  };
  return cached;
}
