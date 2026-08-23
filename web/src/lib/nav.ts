/**
 * The section bar, as data rather than as markup.
 *
 * # Why this is a module and not a `const` in the layout
 *
 * It was a `const` in `Base.astro` for as long as the bar was five flat-ish entries that never
 * changed. Two things ended that. The first is that the bar now lifts corpus classes into it, so
 * part of it is *derived* — which acts appear under `Law` is computed from edges in
 * `.yidam/corpus/`, and a derivation with nothing checking it is a list that goes quietly wrong the
 * first time somebody adds a node. The second is that the checks worth having are about the
 * structure and not about the pixels: every href resolves, every lifted class is two clicks deep,
 * the landmark rules select what they claim to. All three are unit tests against this file, and
 * none of them wants to parse a rendered `<header>` to ask its question.
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
import { loadCorpus, type Corpus, type Node } from "./corpus.ts";
import * as routes from "./routes.ts";

/**
 * What a page passes as `section`, naming where it sits in the bar.
 *
 * A group key marks the group as containing the current page without marking any child as being
 * it — which is the case for all sixteen acts, of which the menu names seven.
 */
export type Section =
  // The groups.
  | "places"
  | "law"
  | "formula"
  | "research"
  | "reference"
  // The children, where a child is a page in its own right.
  | "statewide"
  | "districts"
  | "counties"
  | "house"
  | "senate"
  | "compare"
  | "legislation"
  | "outcomes"
  | "history"
  | "scenario"
  | "wiki"
  | "method"
  | "data";

/** One link in a menu panel. */
export interface NavLink {
  /** Set where this link is a page a reader can be *on*. Matched against `Base`'s `section`. */
  key?: Section;
  href: string;
  label: string;
  /**
   * Why this link is in the menu, on its own line under the label.
   *
   * Never decoration. Under `Law` it is the rule that admitted the act; under `Formula` it is the
   * count of what is behind the link. A note that says nothing should be omitted, not invented.
   */
  note?: string;
}

/** A run of links under one rule. */
export interface NavSection {
  links: NavLink[];
}

/** A top-level entry, and the panel it opens. */
export interface NavGroup {
  /** Marks the whole group current when a page names it. */
  key: Section;
  label: string;
  /**
   * What this section answers, in one line, for the homepage.
   *
   * The homepage maps over these rather than carrying a list of its own. A bar and a front door
   * that can disagree about what the site contains is the navigability defect, not the fix — and
   * a hand-written list is exactly how they come to disagree, because a section added to one is
   * added to the other only if somebody remembers.
   */
  blurb: string;
  /**
   * The one page this section opens onto, for a reader who wants the section rather than a link
   * inside it. A group has no href of its own — a `<summary>` is a disclosure, not a link — so
   * this is what the homepage points its heading at.
   */
  front: string;
  /**
   * The panel, as runs of links separated by a rule.
   *
   * One column, deliberately. The first shape of the `Law` panel was two, because naming four
   * cases beside seven acts made a fourteen-row column that runs off a laptop viewport under a
   * sticky header. The cases collapsed to a single link for a different reason — see
   * `litigation` — and the second column went with them rather than staying as machinery for a
   * panel nothing builds.
   */
  sections: NavSection[];
}

/** Which act the corpus can point at as the one now in force, and which rule found each act. */
export type Rule = "establishes" | "outside-the-budget" | "in-force";

export interface Landmark {
  node: Node;
  rule: Rule;
  /** The note the menu prints. Derived from the rule, so it is true of every member of it. */
  note: string;
}

const REGIME = "funding-regime/";

/** The year an act was signed, from the ISO date every legislation node carries. */
export function signedYear(node: Node): number {
  const signed = node.properties.find((p) => p.name === "signed")?.value.trim() ?? "";
  const year = Number.parseInt(signed.slice(0, 4), 10);
  if (!Number.isFinite(year)) {
    throw new Error(`${node.id} has no parseable \`signed\` date; the bar reads its year from it`);
  }
  return year;
}

/** `Am. Sub. H.B. 110 (2021)`. The designation verbatim — `Am. Sub.` is part of the bill's name. */
export function actLabel(node: Node): string {
  const designation = node.properties.find((p) => p.name === "designation")?.value.trim();
  if (!designation) throw new Error(`${node.id} has no \`designation\`; the bar reads its label`);
  return `${designation} (${signedYear(node)})`;
}

/**
 * The acts the `Law` menu names, and why each one is there.
 *
 * # Three rules, and no list
 *
 * A menu of seven items chosen out of sixteen is an editorial judgement, and this repository's
 * standing objection to those is that nothing records them: the judgement lives in whoever typed
 * the list, the corpus never learns it, and the list is wrong the day an act is added. So the
 * seven are *selected* rather than named —
 *
 * 1. **It establishes a funding regime.** H.B. 1, H.B. 153, H.B. 110 — the three acts that put a
 *    formula in place rather than continuing one.
 * 2. **It does not appropriate.** Every other act here is a biennial budget; these are not. The
 *    Constitution's education clause, H.B. 920's tax reduction factors, and H.B. 583, which
 *    corrected the Fair School Funding Plan between budgets.
 * 3. **It is the most recently signed act that does appropriate.** The budget in force.
 *
 * Rule 3 is why this is worth deriving at all: the menu re-points itself at the next budget with
 * no edit anywhere, and an act added to the corpus enters or does not enter the bar on its own
 * merits. `tests/unit/nav.spec.ts` asserts what the rules currently select, so a corpus change
 * that reshapes the bar fails a check rather than passing silently.
 *
 * # Rule 2's odd member
 *
 * H.B. 583 is a corrective act, not standing law, and the note it gets says only that it sits
 * outside the budget cycle — which is what the rule actually detects and is true of all three.
 * It earns the slot on its own account:
 * `crates/project/tests/the_act_that_corrected_the_plan.rs` exists because a reader who reaches
 * H.B. 110 and not H.B. 583 is reading a formula that was never run.
 */
export function landmarkActs(corpus: Corpus): Landmark[] {
  const acts = corpus.byClass.get("legislation")?.nodes ?? [];
  const found = new Map<string, Landmark>();

  const appropriates = (node: Node): boolean =>
    node.out.some((e) => e.relationship === "appropriates-for");

  for (const node of acts) {
    const regime = node.out.find(
      (e) => e.relationship === "establishes" && e.id?.startsWith(REGIME),
    );
    if (regime) {
      found.set(node.id, { node, rule: "establishes", note: regime.label });
      continue;
    }
    if (!appropriates(node)) {
      found.set(node.id, { node, rule: "outside-the-budget", note: "not a budget act" });
    }
  }

  /*
   * The budget in force: latest `signed` among the acts that appropriate.
   *
   * Not "the act whose biennium contains the bundle's fiscal year", which was the first shape and
   * is the wrong one — `fiscal-period` node names are inconsistent about how they abbreviate a
   * biennium (`fy2004-2005` beside `fy2026-27`), so matching on them means parsing two formats and
   * being silently wrong about one of them. `signed` is an ISO date on every node in the class.
   */
  const budgets = acts.filter(appropriates);
  const inForce = budgets.reduce<Node | null>(
    (latest, node) => (latest == null || signedYear(node) > signedYear(latest) ? node : latest),
    null,
  );
  if (inForce && !found.has(inForce.id)) {
    const period = inForce.out.find((e) => e.relationship === "appropriates-for");
    found.set(inForce.id, {
      node: inForce,
      rule: "in-force",
      note: period ? `appropriates for ${period.label}` : "the budget in force",
    });
  }

  // Newest first. A reader looking for the act behind this year's figures starts at the top; one
  // looking for where the duty comes from reads to the bottom and finds 1851 there.
  return [...found.values()].sort((a, b) => signedYear(b.node) - signedYear(a.node));
}

/**
 * Why no case is named in the menu, and what it would take to name one.
 *
 * DeRolph is the most recognisable thing in Ohio school funding and the menu does not carry it.
 * That is not a judgement about its importance — it is that **the corpus holds no short name for
 * a case**, and the menu needs one. Litigation labels are full citations:
 * `Cincinnati City School District Board of Education v. Walter (1979)` is sixty-two characters
 * and wraps to three lines in a panel.
 *
 * The obvious derivation is the parenthetical, and it is wrong. It reads `DeRolph I (1997)` off
 * `DeRolph v. State (DeRolph I, 1997)` correctly and then reads **`Franklin County (2025)`** off
 * `EdChoice Constitutional Challenge (Franklin County, 2025)` — a venue offered to a reader as the
 * name of a case, in the bar, on every page. A rule that is right four times out of six and gives
 * no sign which two it missed is worse here than no rule.
 *
 * So `Litigation` is one link with its count, like `Doctrine`. The fix is a corpus change and not
 * a web one: declare a short name on `litigation.ont.yml` and fill it on the six nodes, and then
 * the menu, the statute timeline and a case's own breadcrumb all have something true to print.
 * That is a content decision, and it is not this phase's.
 */
function litigation(corpus: Corpus): NavLink[] {
  return [classLink(corpus, "litigation", "Litigation"), classLink(corpus, "doctrine", "Doctrine")];
}

/**
 * A whole class behind one link, with how much is behind it.
 *
 * The count sits in the label rather than in a note because a note under `Components` reading
 * "16 components" says the label twice. Notes elsewhere in the bar carry something the label does
 * not — the rule that admitted an act, the year a regime was established — and one that restates
 * its own heading teaches a reader to stop reading them.
 */
function classLink(corpus: Corpus, className: string, label: string): NavLink {
  return {
    href: routes.wikiClass(className),
    label: `${label} (${corpus.byClass.get(className)?.nodes.length ?? 0})`,
  };
}

/**
 * The regimes, newest first, by the act that established each.
 *
 * `byClass` returns them in the order the directory is read, which is alphabetical, which puts
 * `Bridge Formula` above `Fair School Funding Plan` and tells a reader nothing. Ordered by the
 * signed year of the establishing act, the panel reads as the succession it is — and the two with
 * no establishing act in the corpus (`Equal Yield`, `Foundation Base Cost`, both older than
 * anything the legislation class reaches) fall to the bottom rather than being given a date they
 * do not have.
 */
function regimes(corpus: Corpus): NavLink[] {
  const acts = corpus.byClass.get("legislation")?.nodes ?? [];
  const established = new Map<string, number>();
  for (const act of acts) {
    for (const edge of act.out) {
      if (edge.relationship === "establishes" && edge.id?.startsWith(REGIME)) {
        established.set(edge.id, signedYear(act));
      }
    }
  }
  const nodes = [...(corpus.byClass.get("funding-regime")?.nodes ?? [])];
  nodes.sort((a, b) => {
    const ya = established.get(a.id) ?? 0;
    const yb = established.get(b.id) ?? 0;
    return yb - ya || a.label.localeCompare(b.label);
  });
  return nodes.map((node) => {
    const link: NavLink = { href: routes.wikiNode(node.className, node.name), label: node.label };
    const year = established.get(node.id);
    // Spread rather than `note: year ?? undefined`. Under `exactOptionalPropertyTypes`, a key
    // present and holding `undefined` is not the same as an absent key, and the two regimes older
    // than anything the legislation class reaches have no year to print.
    return year == null ? link : { ...link, note: `established ${year}` };
  });
}

/**
 * Where a wiki page sits in the bar.
 *
 * The point of lifting a class is that a reader standing on one of its nodes can see where they
 * are. Before this every one of the wiki's pages reported itself as `Reference › Wiki`, so a
 * reader who arrived at H.B. 110 from a search had nothing in the bar telling them the site had a
 * section about statute at all.
 */
export function sectionForClass(className: string): Section {
  if (["legislation", "litigation", "doctrine"].includes(className)) return "law";
  if (["funding-regime", "formula-component", "parameter", "metric"].includes(className)) {
    return "formula";
  }
  return "wiki";
}

/**
 * The bar.
 *
 * # Why there is no flat entry left
 *
 * Every top-level entry opens a panel, where `Statewide` and `Scenario` used to be links. That is
 * a real cost — nothing in the bar is one click away any more — and it buys two things. The bar
 * holds five axes now rather than three, and a bar that mixes links with disclosures teaches a
 * reader nothing about which is which until they have clicked both.
 *
 * That cost used to be offset by a search box sitting beside the bar, which was the one-click path
 * to the thing most readers want: this site is read one district at a time. The box is gone, and
 * nothing replaced it, so the shortest path to a named district is now two clicks — the panel,
 * then `/districts` — and a name typed into that page's filter. If the depth is ever measured and
 * found to cost readers, the fix is a flat entry in the bar, not a second index to maintain.
 *
 * The homepage takes no entry at all. It is what the brand mark points at, which is a convention a
 * reader already has, and giving it a tab as well would put one destination in the bar twice.
 */
export function nav(bundle: Bundle, corpus: Corpus = loadCorpus()): NavGroup[] {
  const acts = landmarkActs(corpus);
  const legislation = corpus.byClass.get("legislation");

  return [
    {
      key: "places",
      front: "/statewide",
      blurb: "Every district, every county, both legislative chambers — and any two of them side by side.",
      label: "Places",
      sections: [
        {
          links: [
            {
              key: "statewide",
              href: "/statewide",
              label: "Statewide",
              note: "all of Ohio at once",
            },
            // Read from the feed, not typed in, for the reason `og/pages.ts` gives at length: a
            // count that a regenerated panel makes wrong is worse than no count. This one is on
            // every page.
            {
              key: "districts",
              href: "/districts",
              label: `All ${bundle.statewide.districts} districts`,
            },
            { key: "counties", href: "/counties", label: "Counties" },
            { key: "house", href: "/house", label: "House" },
            { key: "senate", href: "/senate", label: "Senate" },
            { key: "compare", href: "/compare", label: "Compare two" },
          ],
        },
      ],
    },
    {
      key: "law",
      front: routes.wikiClass("legislation"),
      blurb: "The acts, the cases and the doctrines the money is arranged under, from an 1851 duty to this biennium's budget.",
      label: "Law",
      sections: [
        {
          // The chronological view, above the acts rather than among them: it is the only entry
          // here that is not a node, and a reader who wants "how did this get here" wants it
          // before they want any single act.
          links: [
            {
              key: "legislation",
              href: "/legislation",
              label: "The statute timeline",
              note: "every act, in order",
            },
          ],
        },
        {
          links: acts.map((act) => ({
            href: routes.wikiNode(act.node.className, act.node.name),
            label: actLabel(act.node),
            note: act.note,
          })),
        },
        {
          links: [
            {
              href: routes.wikiClass("legislation"),
              label: `All ${legislation?.nodes.length ?? 0} acts`,
            },
            ...litigation(corpus),
          ],
        },
      ],
    },
    {
      key: "formula",
      front: routes.wikiClass("funding-regime"),
      blurb: "What the state computes, regime by regime, and the parameters a budget sets when it computes it.",
      label: "Formula",
      sections: [
        { links: regimes(corpus) },
        {
          links: [
            classLink(corpus, "formula-component", "Components"),
            classLink(corpus, "parameter", "Parameters"),
            classLink(corpus, "metric", "Metrics"),
          ],
        },
      ],
    },
    {
      key: "research",
      front: "/outcomes",
      blurb: "What this repository worked out from all of it — and, as loudly, what none of it claims.",
      label: "Research",
      sections: [
        {
          links: [
            {
              key: "outcomes",
              href: "/outcomes",
              label: "Outcomes",
              note: "association, not effect",
            },
            { key: "history", href: "/history", label: "History", note: "the Census long view" },
            { key: "scenario", href: "/scenario", label: "Scenario", note: "re-run the formula" },
          ],
        },
      ],
    },
    {
      key: "reference",
      front: "/wiki",
      blurb: "The corpus these pages are generated from, how the figures are made, and the data itself.",
      label: "Reference",
      sections: [
        {
          links: [
            { key: "wiki", href: "/wiki", label: "Wiki", note: "the corpus itself" },
            { key: "method", href: "/method", label: "Method" },
            { key: "data", href: "/data", label: "Downloads" },
          ],
        },
      ],
    },
  ];
}

export function navLinks(groups: NavGroup[]): NavLink[] {
  return groups.flatMap((g) => g.sections.flatMap((s) => s.links));
}
