/**
 * The section bar, as data rather than as markup.
 *
 * # Why this is a module and not a `const` in the layout
 *
 * It was a `const` in `Base.astro` for as long as the bar was five flat-ish entries that never
 * changed. Two things ended that. The first is that part of the bar is *derived* from
 * `.yidam/corpus/` — every class the corpus holds is a link under `Library`, with a count read off
 * the directory — and a derivation with nothing checking it is a list that goes quietly wrong the
 * first time somebody adds a class. The second is that the checks worth having are about the
 * structure and not about the pixels: every href resolves, every class is two clicks deep, no run
 * outgrows its panel. All three are unit tests against this file, and none of them wants to parse
 * a rendered `<header>` to ask its question.
 *
 * `Base.astro` renders what `nav()` returns and decides nothing.
 *
 * # What is not here
 *
 * Routes. `routes.ts` holds those and says why it must stay a table of strings — it is imported by
 * the card generator, which cannot read `.yidam/` off disk. This module reads the corpus, so the
 * dependency runs this way and not the other.
 */

import type { Bundle } from "./types.ts";
import { loadCorpus, type Corpus } from "./corpus.ts";
import * as routes from "./routes.ts";
import type { NavIcon } from "./icons.ts";
import { GROUPS } from "./explained/topic.ts";
import { TOPICS } from "./explained/registry.ts";

/**
 * What a page passes as `section`, naming where it sits in the bar.
 *
 * A group key marks the group as containing the current page without marking any child as being
 * it — which is the case for every corpus node, of which the bar names none: it names their
 * classes.
 */
export type Section =
  // The groups.
  | "explained"
  | "places"
  | "analysis"
  | "library"
  | "about"
  // The children, where a child is a page in its own right. `districts` and `scenario` are also
  // entries of the bar on their own — see `nav()`.
  | "statewide"
  | "changed"
  | "districts"
  | "counties"
  | "house"
  | "senate"
  | "compare"
  | "legislation"
  | "outcomes"
  | "history"
  | "scenario"
  | "bounds"
  | "wiki"
  | "sources"
  | "decisions"
  | "method"
  | "data";

/**
 * The name table: one row per page the bar links to, and the one name that page goes by.
 *
 * # Why a place has one name
 *
 * A page carried up to four — its label in the bar, its `<title>`, its `h1` and its URL — and
 * they disagreed. `/legislation` was "The statute timeline" in the bar, "Statute" in the tab and
 * "Ohio school funding in statute" on the page; `/history` was "History" everywhere but its own
 * heading, which said "The long view". A reader who clicks a word expects to land on a page that
 * says the same word back, and a reader who bookmarks the tab should recognise it in the bar.
 *
 * So the bar and the page both read the name from here, and `tests/dist/names.spec.ts` holds them
 * to it on the built site: for every link in the bar, the label is the `<title>` stem and the `h1`
 * begins with it. An `h1` may add a clause after the name; it may not say something else first.
 *
 * The corpus classes are not rows. Their name is the ontology's own `label:`, which the bar and the
 * class index both read, so there is nothing here to restate.
 *
 * The URL is the fourth name and is not required to match. Changing an address costs a redirect
 * and every link already sent; changing a label costs nothing, and `/legislation` is a better
 * address than any spelling of its title would be.
 */
export const NAMES = {
  statewide: { href: "/statewide", name: "Statewide" },
  changed: { href: routes.WHAT_CHANGED, name: "What changed" },
  districts: { href: "/districts", name: "Find a district" },
  explained: { href: routes.EXPLAINED, name: "Explained" },
  counties: { href: "/counties", name: "Counties" },
  house: { href: "/house", name: "House districts" },
  senate: { href: "/senate", name: "Senate districts" },
  compare: { href: "/compare", name: "Compare" },
  legislation: { href: "/legislation", name: "The statute timeline" },
  outcomes: { href: "/outcomes", name: "Outcomes" },
  history: { href: "/history", name: "History" },
  scenario: { href: "/scenario", name: "Change the formula" },
  bounds: { href: routes.BOUNDS, name: "Bounds" },
  wiki: { href: "/wiki", name: "The corpus" },
  sources: { href: "/wiki/source", name: "Sources" },
  decisions: { href: "/wiki/decision", name: "Decisions" },
  method: { href: routes.METHOD, name: "Method" },
  data: { href: "/data", name: "Data" },
} as const satisfies Partial<Record<Section, { href: string; name: string }>>;

/**
 * The runner's two views: one place, one name, asked two questions.
 *
 * `/scenario` and `/reach` were two entries in the bar, `Scenario` and `Reach`, for one set of
 * levers over one panel — the second a question about the first rather than a place of its own.
 * They are one entry now, `Change the formula`, and these are the tabs inside it. Both pages carry the
 * name in their `<title>` and `h1`; the tab says which question is open.
 *
 * `data-carry-levers` on each tab hands the reader's lever positions to the other view — see
 * `carryLevers` in `scripts/scenario.ts`.
 */
export const RUNNER_VIEWS = [
  { key: "changes", href: NAMES.scenario.href, label: "What changes" },
  { key: "reach", href: routes.REACH, label: "Who it reaches" },
] as const;

/** Which of {@link RUNNER_VIEWS} a runner page is. */
export type RunnerView = (typeof RUNNER_VIEWS)[number]["key"];

/**
 * A district's tabs: the name table for the pages under `/district/[irn]`.
 *
 * # Why they needed one
 *
 * #547 gave every place in the bar one name and left these alone (#559). The fourth tab said
 * `Property tax`, its `<title>` said `property tax`, and its address said `taxes` — three names for
 * one page, and a hand-typed copy of the strip on every exemplar's wiki page had drifted further,
 * still calling the runner `Scenario` after the bar had renamed it.
 *
 * So the strip, the `<title>` and that copy read the name from here. The rule is the bar's with
 * the district in front: a tab's `<title>` is the district, then the name — see
 * {@link districtTitle} — and `tests/dist/names.spec.ts` holds the built pages to it.
 *
 * # The URL follows the name
 *
 * Unlike a place in the bar, where the address is not required to match. A tab's last segment is
 * its name in lower case, and `tests/unit/nav.spec.ts` fails where it is not. The name was chosen to
 * fit the address rather than the reverse, because a district page is sent as a link and every
 * moved one costs a redirect across all of them.
 *
 * The dashboard is the district's own page: its address has no segment and its `<title>` no name,
 * because the district is what the page is called.
 *
 * The last tab is not a view of this district alone — it opens the runner with the district chosen
 * (#548) — so it carries the runner's name and the runner's address.
 */
export const DISTRICT_VIEWS = [
  { key: "dashboard", name: "Dashboard", href: routes.district },
  { key: "outcome", name: "Outcome", href: routes.districtOutcome },
  { key: "finances", name: "Finances", href: routes.districtFinances },
  { key: "taxes", name: "Taxes", href: routes.districtTaxes },
  { key: "change", name: "Change", href: routes.districtChange },
  { key: "scenario", name: NAMES.scenario.name, href: routes.districtScenario },
] as const;

/** Which of {@link DISTRICT_VIEWS} a district page is. The runner is not one of them. */
export type DistrictView = Exclude<(typeof DISTRICT_VIEWS)[number]["key"], "scenario">;

/** A district page's `<title>`, before the site name: the district, then the tab it is on. */
export function districtTitle(qualifiedName: string, view: DistrictView): string {
  if (view === "dashboard") return qualifiedName;
  const { name } = DISTRICT_VIEWS.find((v) => v.key === view)!;
  return `${qualifiedName} — ${name}`;
}

/** A row of the name table as a link in the bar. */
function place(key: keyof typeof NAMES, note?: string, icon?: NavIcon): NavLink {
  const { href, name } = NAMES[key];
  return {
    key,
    href,
    label: name,
    ...(note == null ? {} : { note }),
    ...(icon == null ? {} : { icon }),
  };
}

/** One link in a menu panel. */
export interface NavLink {
  /** Set where this link is a page a reader can be *on*. Matched against `Base`'s `section`. */
  key?: Section;
  href: string;
  label: string;
  /**
   * Why this link is in the menu, on its own line under the label.
   *
   * Never decoration: a note that says nothing should be omitted, not invented. The class links
   * under `Library` carry none — their count sits in the label, see `classLink`.
   */
  note?: string;
  /**
   * A glyph drawn beside the label, from `icons.ts`. Only a group's {@link NavGroup.lead} sets one.
   *
   * Never decoration either, and never alone: the label is still the link's name, and the glyph is
   * `aria-hidden`. What it carries is a kind — these three are pages *about* the corpus, set apart
   * from the classes that are its contents — and an icon on every link would carry nothing.
   */
  icon?: NavIcon;
}

/** A run of links under one rule, and optionally under a heading. */
export interface NavSection {
  /**
   * What the run is, where a panel holds several of different kinds.
   *
   * Only `Library` sets one. Its twenty-odd links are the corpus's classes, and a flat column of
   * ontology labels reads as an index nobody sorted; five short headings say what each run is for.
   * Rendered as text, not as a heading element — see `Base.astro` — so the bar adds nothing to a
   * page's outline.
   */
  heading?: string;
  links: NavLink[];
}

/** What every entry in the bar has, whether it opens a panel or goes somewhere. */
interface NavEntryBase {
  /** Marks the entry current when a page names it. */
  key: Section;
  label: string;
  /**
   * What this entry answers, in one line, for the homepage.
   *
   * The homepage maps over these rather than carrying a list of its own. A bar and a front door
   * that can disagree about what the site contains is the navigability defect, not the fix — and
   * a hand-written list is exactly how they come to disagree, because a section added to one is
   * added to the other only if somebody remembers.
   */
  blurb: string;
  /**
   * The one page this entry opens onto. For a flat entry that is its href; for a group, which has
   * no href of its own — a `<summary>` is a disclosure, not a link — it is where the homepage
   * points its heading.
   */
  front: string;
}

/** A top-level entry that opens a panel. */
export interface NavGroup extends NavEntryBase {
  kind: "group";
  /**
   * The panel, as runs of links separated by a rule.
   *
   * One column except in `wide` panels, which are two above the nav's breakpoint — see
   * `.menu-panel[data-width="wide"]` in `app.css`.
   */
  sections: NavSection[];
  /** Lay the runs out in two columns where there is room. `Library` only. */
  wide?: boolean;
  /**
   * Links set above the runs, across the whole panel, each with its glyph. `Library` only.
   *
   * The record — the class index, the sources and the decisions — sat as a fifth headed run at the
   * foot of `Library`'s second column, drawn exactly like `Doctrine (3)`. It is not a fifth kind of
   * class: it is what the classes are indexed, cited and justified by, and the first of the three is
   * the panel's own front door. So it heads the panel as three tiles over both columns instead of
   * sitting at the bottom of one, and the columns below balance better for losing it.
   */
  lead?: NavLink[];
}

/** A top-level entry that is a link, one click from anywhere. */
export interface NavPlace extends NavEntryBase {
  kind: "place";
}

export type NavEntry = NavGroup | NavPlace;

/**
 * A whole class behind one link, with how much is behind it.
 *
 * The count sits in the label rather than in a note because a note under `Formula Component`
 * reading "18 components" says the label twice.
 *
 * The name is the ontology's `label:`, which is also the class index's `h1` and `<title>` stem.
 * It was a second, shorter name typed here — `Components`, `Parameters`, `All 16 acts` — so the
 * link said one thing and the page it opened said another.
 */
function classLink(corpus: Corpus, className: string): NavLink {
  const entry = corpus.byClass.get(className);
  if (!entry) throw new Error(`The bar links the class \`${className}\`, and the corpus has none`);
  return {
    href: routes.wikiClass(className),
    label: `${entry.label} (${entry.nodes.length})`,
  };
}

/**
 * Where each corpus class sits in `Library`, by what a reader would come to it for.
 *
 * # Every class, and why that is the rule now
 *
 * The bar lifted seven of eighteen classes — statute into `Law`, the formula's four into `Formula`
 * — and the other eleven, `school`, `program`, `actor` and the rest, were reachable only by opening
 * `/wiki` and reading down a list (#548). And `Law` did not name its class so much as seven acts
 * out of sixteen, chosen by three rules over the corpus's edges: an editorial judgement that was
 * at least derived, and still a judgement a reader could not see. The class index lists all of
 * them, and is one click from the bar.
 *
 * # A class this table does not name
 *
 * Still reaches the bar: `nav()` puts it in a last run headed `Also in the corpus`, so a class
 * added to `.yidam/` is two clicks deep on the day it lands rather than never. That run is a
 * notification and not a place — `tests/unit/nav.spec.ts` fails while it exists, and the fix is a
 * line here.
 */
export const LIBRARY: readonly { heading: string; classes: readonly string[] }[] = [
  { heading: "Law", classes: ["legislation", "litigation", "doctrine"] },
  {
    heading: "Formula",
    classes: [
      "funding-regime",
      "formula-component",
      "parameter",
      "metric",
      "revenue-stream",
      "fiscal-period",
    ],
  },
  {
    heading: "Institutions",
    classes: [
      "education-agency",
      "actor",
      "school",
      "program",
      "accountability-regime",
      "intervention",
    ],
  },
  { heading: "Proposals", classes: ["draft-legislation", "model-policy", "scenario"] },
];

/**
 * `Explained`'s panel: the index, then each topic under the run its module names.
 *
 * Built from the registry, so a topic module adds its own link and nobody edits this file to list
 * it. A run with no topic yet is left out rather than drawn empty, and while there is no topic at
 * all the panel holds the index alone.
 *
 * The question is the link's label because it is the page's name: its `<title>` and the start of
 * its `h1` (`tests/dist/names.spec.ts`). A reader choosing what to read sees the questions before
 * choosing, which is why this is a panel and not a flat link (#712).
 */
function explainedRuns(): NavSection[] {
  const index: NavSection = { links: [place("explained", "every question, in plain words")] };
  const runs = GROUPS.map((heading) => ({
    heading,
    links: TOPICS.filter((t) => t.group === heading).map((t) => ({
      href: routes.explained(t.slug),
      label: t.question,
    })),
  })).filter((run) => run.links.length > 0);
  return [index, ...runs];
}

/** The heading of the run that holds any class {@link LIBRARY} does not place. */
export const UNPLACED = "Also in the corpus";

/**
 * Where a wiki page sits in the bar: under `Library`, whatever its class.
 *
 * The point of lifting a class was that a reader standing on one of its nodes can see where they
 * are. When seven classes were lifted into `Law` and `Formula` this chose between those and
 * `wiki`; every class is under `Library` now, so the group is the answer for all of them and the
 * class index — a link in the panel — marks itself by path. Kept as a function of the class
 * because the wiki routes call it that way, and a class that one day earns a place of its own
 * changes one line here rather than every route. It reads no argument today, so it names none.
 */
export const sectionForClass: (className: string) => Section = () => "library";

/**
 * The bar.
 *
 * # Seven entries, by what a reader came to do
 *
 * It was organised by the corpus's own taxonomy — Places, Law, Formula, Research, Reference — which
 * is how the repository is built and not how it is read. It is organised by task now (#548):
 * find one district; understand the formula; look at a place; read an analysis; change the formula;
 * look something up; learn how the figures are made.
 *
 * `Explained` is second (#712). Understanding the formula comes before analysing it, and a reader who
 * has found their district and does not yet know what a "state share" is wants it next.
 *
 * # Two of them are links, not menus
 *
 * `Find a district` and `Change the formula`. A bar of nothing but disclosures put every destination two
 * clicks away, and the one most readers want — this site is read one district at a time — was the
 * second link inside `Places`. The note that was here said so, and named the fix: "a flat entry in
 * the bar, not a second index to maintain". The runner is the other: it is one page with two
 * views, and a menu holding one link is a click that decides nothing.
 *
 * Search is not coming back through this door. It was removed for guessing wrong with confidence
 * and for an 88 KB index; `/districts` filters by name or IRN, and the homepage's field is a plain
 * GET to it.
 *
 * The homepage takes no entry at all. It is what the brand mark points at, which is a convention a
 * reader already has, and giving it a tab as well would put one destination in the bar twice.
 */
export function nav(bundle: Bundle, corpus: Corpus = loadCorpus()): NavEntry[] {
  const placed = new Set(LIBRARY.flatMap((run) => run.classes));
  const unplaced = corpus.classes.map((c) => c.className).filter((name) => !placed.has(name));
  const classRuns: NavSection[] = LIBRARY.map((run) => ({
    heading: run.heading,
    links: run.classes
      .filter((name) => corpus.byClass.has(name))
      .map((name) => classLink(corpus, name)),
  }));
  // The statute timeline is the chronological view of the `Law` classes, and sits at the head of
  // their run: it is the only link there that is not an index, and a reader who wants "how did
  // this get here" wants it before any single act.
  classRuns[0]!.links.unshift(place("legislation"));
  if (unplaced.length > 0) {
    classRuns.push({ heading: UNPLACED, links: unplaced.map((name) => classLink(corpus, name)) });
  }

  return [
    {
      kind: "place",
      key: "districts",
      front: NAMES.districts.href,
      // The count is read from the feed, not typed in, for the reason `og/pages.ts` gives at
      // length: a count that a regenerated panel makes wrong is worse than no count.
      blurb: `All ${bundle.statewide.districts}, by name or IRN, each with the formula's answer beside the one it receives.`,
      label: NAMES.districts.name,
    },
    {
      kind: "group",
      key: "explained",
      front: NAMES.explained.href,
      blurb: "The formula's largest effects, one question at a time, in plain words with the math behind them.",
      label: NAMES.explained.name,
      sections: explainedRuns(),
    },
    {
      kind: "group",
      key: "places",
      front: "/statewide",
      blurb: "All of Ohio at once, every county, both legislative chambers — and any two districts side by side.",
      label: "Places",
      sections: [
        {
          links: [
            place("statewide", "all of Ohio at once"),
            place("changed", "the biennium, provision by provision"),
            place("counties"),
            place("house"),
            place("senate"),
            place("compare", "any two districts, side by side"),
          ],
        },
      ],
    },
    {
      kind: "group",
      key: "analysis",
      front: "/outcomes",
      blurb: "What this site worked out from all of it — and, as loudly, what none of it claims.",
      label: "Analysis",
      sections: [
        {
          links: [
            place("outcomes", "association, not effect"),
            place("history", "the Census long view"),
            /* Last in the run because it is the widest: the other two ask what the formula did
               to somebody, and this one asks what shape the formula is. */
            place("bounds", "every floor and ceiling, and who is on it"),
          ],
        },
      ],
    },
    {
      kind: "place",
      key: "scenario",
      front: NAMES.scenario.href,
      blurb: "Move a lever and re-run the formula for every district: what changes, and who it reaches.",
      label: NAMES.scenario.name,
    },
    {
      kind: "group",
      key: "library",
      front: "/wiki",
      blurb: "The statute, the formula and every other class of the corpus these pages are generated from, with its sources and decisions.",
      label: "Library",
      wide: true,
      lead: [
        place("wiki", "every class, and what it holds", "corpus"),
        place("sources", "what every claim cites", "sources"),
        place("decisions", "why it is built this way", "decisions"),
      ],
      sections: classRuns,
    },
    {
      kind: "group",
      key: "about",
      front: routes.METHOD,
      blurb: "How every figure here is made and checked, and the data itself as files.",
      label: "About",
      sections: [
        {
          links: [place("method", "how the figures are made"), place("data", "every figure, as files")],
        },
      ],
    },
  ];
}

/** Every link a group's panel holds, its lead first, in the order a reader meets them. */
export function groupLinks(group: NavGroup): NavLink[] {
  return [...(group.lead ?? []), ...group.sections.flatMap((s) => s.links)];
}

/** Every link the bar carries, flat entries included, in the order a reader meets them. */
export function navLinks(entries: NavEntry[]): NavLink[] {
  return entries.flatMap((entry) =>
    entry.kind === "place"
      ? [{ key: entry.key, href: entry.front, label: entry.label }]
      : groupLinks(entry),
  );
}
