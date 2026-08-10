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

/** Where a link that leaves the corpus points. The repository, on GitHub. */
const REPO = "https://github.com/goedelsoup/ohio-education-funding/blob/main";

/**
 * The `.yidam/` subtrees this site does not publish.
 *
 * `corpus/` and `catalog/` become pages here; these are working documents for whoever is authoring
 * the corpus — skill definitions, decision records — and 13 links point into them from prose that
 * does become a page. They are real documents, so the honest resolution is GitHub rather than a
 * refusal; what is not honest is `/wiki/decisions/report-card-connector`, which is what
 * `decisions/…` produced for as long as nothing distinguished a subtree from an ontology class.
 */
const UNPUBLISHED = ["skills", "decisions", "sangha", "schemas"];

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

/** One corpus node. */
export interface Node {
  /** `<class>/<name>` — the identity used throughout, and the URL. */
  id: string;
  /** Directory it lives in, which is also its ontology class. */
  className: string;
  /** File stem. */
  name: string;
  label: string;
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
  /** Findings, on the three nodes that carry them. Markdown, like the description. */
  findings: string | null;
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
 * | `../../skills/name.md`, `../decisions/name.yml` | nodes, catalog entries | the repository |
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

  // `../../skills/deduction.md`, `../decisions/report-card-connector.yml` — a document in a
  // `.yidam/` subtree this site does not publish. Must precede the `class/name.yml` branch, which
  // would otherwise read `decisions` as an ontology class and emit `/wiki/decisions/…`: resolved,
  // site-absolute, and a 404 on every one of the seven catalog entries that link there.
  const unpublished = clean.match(new RegExp(`(?:^|/)(${UNPUBLISHED.join("|")})/(.+)$`));
  if (unpublished) {
    return { href: `${REPO}/.yidam/${unpublished[1]}/${unpublished[2]}`, id: null, resolved: true };
  }

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
  // which has no class for the sibling to be in; see {@link FROM_CATALOG}.
  const sibling = clean.match(/^([^/]+)\.yml$/);
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
  const description = String(parsed.description ?? "");
  const rawLinks = parsed.links;

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
    description,
    linkText,
    properties,
    findings: parsed.findings == null ? null : String(parsed.findings),
    out: [...stated, ...inline],
    in: [],
  };
}

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

  // After the sources are read, because catalog entries are the largest single writer of link
  // targets and half the shapes only they use.
  checkTargets(nodes, classes, sources, report);

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

  const warnings = report.filter((d) => d.severity === "warning");
  if (warnings.length > 0) {
    // Not fatal, and deliberately not silent. These are vocabulary drift — a relationship or a
    // property a class's ontology has not declared — which is a question for whoever owns the
    // ontology rather than a reason to refuse to publish.
    console.warn(
      `\n  ${warnings.length} corpus nodes use vocabulary their ontology does not declare.\n` +
        `  Run \`pnpm corpus:report\` for the list.\n`,
    );
  }

  cached = {
    diagnostics: report,
    nodes: nodes.sort((a, b) => a.label.localeCompare(b.label)),
    byId,
    classes: withNodes.sort((a, b) => a.label.localeCompare(b.label)),
    byClass: new Map(withNodes.map((c) => [c.className, c])),
    sources: sources.sort((a, b) => a.title.localeCompare(b.title)),
    bySlug,
  };
  return cached;
}
