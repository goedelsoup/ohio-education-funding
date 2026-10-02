/**
 * Where each corpus node appears on the rest of the site, read off the built pages.
 *
 * # Why this exists
 *
 * The site links from data to the corpus in about two thousand places and from the corpus back to
 * the data in almost none: when #551 was opened, 5 of 274 wiki pages linked a district page, and
 * those five were the education-agency nodes that carry an IRN. A reader who arrived at
 * `metric/performance-index` from a search had no way to learn that every district's outcome page
 * charts it. The link existed; it pointed one way.
 *
 * So this inverts the map. Every page outside `/wiki` is scanned for the corpus nodes its `<main>`
 * links, and each node's page lists the places that cite it — by the cited page's own title, and
 * the section of it the link sits in.
 *
 * # Why it is read off the build and not declared
 *
 * The same reason `contents.ts` reads a page's sections off the page. A list of "where this is
 * used", written beside the components that use it, is a claim about what renders rather than a
 * record of it, and the links on a district page are conditional: the transition supplement is
 * linked on the 144 districts that receive one and on no others. Reading the output is the only
 * way to be right about those without enumerating them.
 *
 * # Why it is committed rather than computed during the build
 *
 * A page cannot read another page in the same build — Astro renders them in no order a template
 * can rely on — so the inversion is computed from one build and rendered by the next. The
 * committed `appearances.json` is that intermediate, regenerated with `pnpm appearances` after a
 * build, and `tests/dist/appearances.spec.ts` fails when it no longer matches the build it would be
 * regenerated from. A branch that changes which corpus nodes a data page links, or what those pages
 * or sections are called, regenerates it; the test says so.
 *
 * Build-time only, like `contents.ts`: read by a page's frontmatter, never shipped.
 */

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";

/** One place a node is cited from: one page, or one route every district or county has. */
export interface Place {
  /** The route, with a placeholder where a district, county or legislative district goes. */
  route: string;
  /** How many pages of this route link the node. */
  pages: number;
  /** How many pages the route has at all, so "144 of 609" can be said and not just "144". */
  of: number;
  /** One of those pages, so a reader can see the link in place. */
  example: { href: string; title: string };
  /** The sections of the example page that carry the link, in page order. */
  sections: { id: string; label: string }[];
}

/** Every node that some page outside the corpus links, by node id (`<class>/<name>`). */
export type Appearances = Record<string, Place[]>;

/** One built page: its path under `dist/`, and its bytes. */
export interface BuiltPage {
  file: string;
  html: string;
}

/** What `Base.astro` appends to every `<title>`; the place is named by what precedes it. */
const SITE_SUFFIX = " — Ohio school funding";

/** Corpus pages that are not nodes: the class indexes' siblings under `/wiki`. */
const NOT_NODES = new Set(["source", "decision"]);

/**
 * The route a built file serves, with its variable segment named.
 *
 * Only the segments that multiply a route by hundreds are folded — a district, a county, a
 * legislative district. Everything else is one page and is its own route.
 */
export function routeOf(file: string): { path: string; route: string } {
  const stem = file.replace(/\\/g, "/").replace(/\.html$/, "");
  const path = stem === "index" ? "/" : `/${stem.replace(/\/index$/, "")}`;
  const route = path
    .replace(/^\/district\/\d{6}/, "/district/[irn]")
    .replace(/^\/county\/[^/]+/, "/county/[slug]")
    .replace(/^\/(house|senate)\/\d+/, "/$1/[n]");
  return { path, route };
}

/** Whether a built file is a page this inverts — anything a reader reaches that is not the corpus. */
function isCited(file: string): boolean {
  const f = file.replace(/\\/g, "/");
  return !(f === "wiki.html" || f.startsWith("wiki/") || f.startsWith("og/") || f === "404.html");
}

/**
 * A heading as a reader reads it aloud: without its section anchor's `#`, without the year chip
 * beside it (whose button and hidden panel would both be read into the text), and without a
 * glossary definition's body.
 */
const textOf = (html: string): string =>
  html
    .replace(/<a class="section-anchor"[\s\S]*?<\/a>/g, "")
    .replace(/<span class="year-chip-wrap">[\s\S]*?<\/span><\/span>/g, "")
    .replace(/<span class="term-def"[\s\S]*?<\/span>/g, "")
    .replace(/<[^>]+>/g, "")
    .replace(/&amp;/g, "&")
    .replace(/&#39;|&apos;/g, "'")
    .replace(/&quot;/g, '"')
    .replace(/\s+/g, " ")
    .trim();

/**
 * The corpus nodes one page links, each with the sections that link it.
 *
 * A section is the card the link sits in — the same `id` the page's contents list and section
 * anchors use, so a link to it lands where the citation is. Its name is the page's own contents
 * entry for that id where the page has a contents list, which is the name `contents.ts` already
 * settled on; otherwise the card's first `h2`, as text.
 */
export function citationsOf(html: string): Map<string, { id: string; label: string }[]> {
  const out = new Map<string, { id: string; label: string }[]>();
  const main = html.match(/<main[\s\S]*?<\/main>/)?.[0] ?? "";
  if (!main.includes('href="/wiki/')) return out;

  const listed = new Map<string, string>();
  const contents = main.match(/<nav class="contents"[\s\S]*?<\/nav>/)?.[0] ?? "";
  for (const entry of contents.matchAll(/<a href="#([^"]+)">([\s\S]*?)<\/a>/g)) {
    listed.set(entry[1]!, textOf(entry[2]!));
  }

  const cards = [...main.matchAll(/<div class="card[^"]*"[^>]*?\sid="([^"]+)"/g)].map((card) => ({
    at: card.index,
    id: card[1]!,
  }));
  const labelOf = (card: { at: number; id: string }): string => {
    const known = listed.get(card.id);
    if (known) return known;
    const heading = main.slice(card.at).match(/<h2[^>]*>([\s\S]*?)<\/h2>/)?.[1];
    return heading ? textOf(heading) : "";
  };

  for (const link of main.matchAll(/href="\/wiki\/([a-z0-9-]+)\/([a-z0-9-]+)(?:\.html)?(?:#[^"]*)?"/g)) {
    if (NOT_NODES.has(link[1]!)) continue;
    const id = `${link[1]}/${link[2]}`;
    const card = cards.filter((c) => c.at < link.index).at(-1);
    const sections = out.get(id) ?? [];
    out.set(id, sections);
    if (!card || sections.some((s) => s.id === card.id)) continue;
    const label = labelOf(card);
    if (label !== "") sections.push({ id: card.id, label });
  }
  return out;
}

/**
 * The inversion: for every node some page links, the routes that link it.
 *
 * `prefer` names the districts to show as a route's example, in order — the corpus's own exemplar
 * districts, so the page a reader is sent to is one the corpus has a node for. A route none of them
 * links falls back to its first page in path order, which is arbitrary and stable.
 */
export function appearancesOf(pages: BuiltPage[], prefer: string[] = []): Appearances {
  const perRoute = new Map<string, number>();
  const found = new Map<
    string,
    Map<string, { paths: string[]; byPath: Map<string, { title: string; sections: Place["sections"] }> }>
  >();

  for (const page of [...pages].sort((a, b) => (a.file < b.file ? -1 : a.file > b.file ? 1 : 0))) {
    if (!isCited(page.file)) continue;
    const { path, route } = routeOf(page.file);
    perRoute.set(route, (perRoute.get(route) ?? 0) + 1);
    const cited = citationsOf(page.html);
    if (cited.size === 0) continue;
    const raw = page.html.match(/<title>([\s\S]*?)<\/title>/)?.[1] ?? path;
    const title = textOf(raw.endsWith(SITE_SUFFIX) ? raw.slice(0, -SITE_SUFFIX.length) : raw);
    for (const [id, sections] of cited) {
      const routes = found.get(id) ?? new Map();
      found.set(id, routes);
      const entry = routes.get(route) ?? { paths: [], byPath: new Map() };
      routes.set(route, entry);
      entry.paths.push(path);
      entry.byPath.set(path, { title, sections });
    }
  }

  const rank = (path: string): number => {
    const at = prefer.findIndex((irn) => path.includes(`/${irn}`));
    return at === -1 ? prefer.length : at;
  };

  const out: Appearances = {};
  for (const id of [...found.keys()].sort()) {
    const routes = found.get(id)!;
    out[id] = [...routes.keys()].sort().map((route) => {
      const { paths, byPath } = routes.get(route)!;
      const href = [...paths].sort((a, b) => rank(a) - rank(b) || (a < b ? -1 : 1))[0]!;
      const { title, sections } = byPath.get(href)!;
      return { route, pages: paths.length, of: perRoute.get(route)!, example: { href, title }, sections };
    });
  }
  return out;
}

/** Every built page under `dist`, as {@link BuiltPage}s. */
export function builtPages(dist: string): BuiltPage[] {
  const walk = (dir: string): string[] =>
    readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
      entry.isDirectory()
        ? walk(join(dir, entry.name))
        : entry.name.endsWith(".html")
          ? [join(dir, entry.name)]
          : [],
    );
  return walk(dist).map((file) => ({ file: relative(dist, file), html: readFileSync(file, "utf8") }));
}

/** Where the committed inversion lives, relative to whichever directory we run in. */
const CANDIDATES = ["src/lib/appearances.json", "web/src/lib/appearances.json"];

/** The file {@link loadAppearances} reads and `pnpm appearances` writes. */
export const APPEARANCES_PATH =
  CANDIDATES.map((candidate) => resolve(process.cwd(), candidate)).find((candidate) =>
    existsSync(candidate),
  ) ?? resolve(process.cwd(), CANDIDATES[0]!);

let cached: Appearances | null = null;

/** The committed inversion. Memoized; an absent file is an empty map, which renders no block. */
export function loadAppearances(): Appearances {
  if (cached) return cached;
  cached = existsSync(APPEARANCES_PATH)
    ? (JSON.parse(readFileSync(APPEARANCES_PATH, "utf8")) as Appearances)
    : {};
  return cached;
}
