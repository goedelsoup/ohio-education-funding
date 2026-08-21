/**
 * The section bar, which is now two thirds generated.
 *
 * # Why this file exists
 *
 * A bar that is typed out is wrong only when somebody types it wrong, and it is wrong visibly.
 * This one reads `.yidam/corpus/` — which acts appear under `Law`, which regimes under `Formula`,
 * and how many nodes sit behind each class link — so it can go wrong without anybody touching a
 * line of it. Rename a node and thirty pages' worth of header points at a route the build does
 * not emit. Change an edge and the menu quietly stops naming the act that established the current
 * formula, with no error and nothing on screen to say a rule stopped selecting.
 *
 * The end-to-end suite already walks every href in the rendered header and opens it, which is the
 * check that a link goes somewhere. What it cannot ask is whether the *right* things were
 * selected, because a menu that has silently emptied out still renders and every one of its zero
 * links still resolves. That question belongs here, against the module rather than the markup.
 *
 * # Why some of these assert exact names
 *
 * Because the alternative is a shape test that a broken derivation passes. "Between four and
 * eight acts" is satisfied by four wrong ones. The seven below are what three stated rules select
 * from today's corpus, and a change to them is not a failure so much as a notification: something
 * about the shape of Ohio's statute history moved, and somebody should look at the bar before
 * shipping it.
 */

import { expect, test } from "vitest";

import { loadCorpus } from "../../src/lib/corpus.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import {
  actLabel,
  landmarkActs,
  nav,
  navLinks,
  sectionForClass,
  signedYear,
} from "../../src/lib/nav.ts";

const corpus = loadCorpus();
const { bundle } = loadFeed();
const groups = nav(bundle, corpus);
const links = navLinks(groups);

/** The classes the bar lifts out of the wiki, and the group each is lifted into. */
const LIFTED: Record<string, string> = {
  legislation: "law",
  litigation: "law",
  doctrine: "law",
  "funding-regime": "formula",
  "formula-component": "formula",
  parameter: "formula",
  metric: "formula",
};

test("the three rules select these seven acts, and each for a stated reason", () => {
  /*
   * The derivation, written out. If this fails, read the diff before changing the expectation:
   * a corpus edit that adds or removes an act here has changed what the bar tells a reader is
   * load-bearing in Ohio school funding, which is a decision and not a fixture update.
   */
  expect(landmarkActs(corpus).map((a) => [actLabel(a.node), a.rule])).toEqual([
    ["Am. Sub. H.B. 96 (2025)", "in-force"],
    ["Sub. H.B. 583 (2022)", "outside-the-budget"],
    ["Am. Sub. H.B. 110 (2021)", "establishes"],
    ["Am. Sub. H.B. 153 (2011)", "establishes"],
    ["Am. Sub. H.B. 1 (2009)", "establishes"],
    ["Am. Sub. H.B. 920 (1976)", "outside-the-budget"],
    ["Ohio Const. art. VI, sec. 2 (1851)", "outside-the-budget"],
  ]);
});

test("every rule still selects something, so none of them has quietly stopped matching", () => {
  /*
   * The failure the exact list above would report but not explain. An edge renamed from
   * `establishes` to `establishes-regime` empties rule 1 and leaves a menu of four acts that
   * still renders, still resolves, and no longer names the act behind the formula in force.
   */
  const rules = new Set(landmarkActs(corpus).map((a) => a.rule));
  expect([...rules].sort()).toEqual(["establishes", "in-force", "outside-the-budget"]);
});

test("the act in force is the newest one that appropriates, and it is newer than every other", () => {
  // Rule 3 is the one that maintains itself across a budget cycle. What it must never do is pick
  // a superseded act because a newer one sorted oddly — the acts arrive in directory order, and
  // `hb-96-2025` sorts before `hb-99-2027` but after `hb-110-2021`.
  const acts = landmarkActs(corpus);
  const inForce = acts.find((a) => a.rule === "in-force");
  expect(inForce, "no act is selected as the budget in force").toBeDefined();

  const appropriating = (corpus.byClass.get("legislation")?.nodes ?? []).filter((n) =>
    n.out.some((e) => e.relationship === "appropriates-for"),
  );
  const newest = Math.max(...appropriating.map(signedYear));
  expect(signedYear(inForce!.node)).toBe(newest);
});

test("no act is selected twice, however many rules would admit it", () => {
  // H.B. 110 both establishes the Fair School Funding Plan and appropriates for FY2022-23; a
  // future act could establish a regime *and* be the one in force. Two entries for one act in a
  // menu of seven is a quarter of the panel saying the same thing.
  const ids = landmarkActs(corpus).map((a) => a.node.id);
  expect(new Set(ids).size).toBe(ids.length);
});

test("every act the menu names is a legislation node the corpus actually holds", () => {
  for (const act of landmarkActs(corpus)) {
    expect(act.node.className, act.node.id).toBe("legislation");
    expect(corpus.byId.has(act.node.id), `${act.node.id} is not in the corpus`).toBe(true);
  }
});

test("every corpus link in the bar lands on a node or a class that exists", () => {
  /*
   * The end-to-end suite opens all thirty hrefs against a built site, which catches this too — a
   * page later than this one, after a fifty-second build, reported as a missing heading rather
   * than as a missing node. Here it is a name and a reason.
   */
  const missing: string[] = [];
  for (const link of links) {
    const wiki = link.href.match(/^\/wiki\/([^/]+)(?:\/([^/]+))?$/);
    if (!wiki) continue;
    const [, className, node] = wiki;
    if (node == null) {
      if (!corpus.byClass.has(className!)) missing.push(`${link.label} → class ${className}`);
    } else if (!corpus.byId.has(`${className}/${node}`)) {
      missing.push(`${link.label} → node ${className}/${node}`);
    }
  }
  expect(missing).toEqual([]);
});

test("every lifted class is two clicks from anywhere: one to open a menu, one to arrive", () => {
  /*
   * The point of the redesign, as an assertion. Before it, statute and formula were reachable
   * only by going to `/wiki` and reading down a list of eighteen classes — three clicks and a
   * scan, from a section labelled `Reference`, which is where a reader looks last.
   *
   * `sectionForClass` is checked against the same table, because the two are halves of one claim:
   * a class is lifted if the bar reaches it *and* its pages say which group they are in. A class
   * in the menu whose nodes still report `wiki` leaves a reader on H.B. 110 with the bar telling
   * them they are in `Reference`.
   */
  const reachable = new Set(
    links.flatMap((link) => {
      const wiki = link.href.match(/^\/wiki\/([^/]+)/);
      return wiki ? [wiki[1]!] : [];
    }),
  );
  for (const [className, group] of Object.entries(LIFTED)) {
    expect(reachable.has(className), `${className} is not reachable from the bar`).toBe(true);
    expect(sectionForClass(className), className).toBe(group);
  }
});

test("a class the bar does not lift still reports itself as the wiki", () => {
  // The default matters as much as the table. `scenario`, `school`, `actor` and the rest are not
  // in the bar, and a page of one of them claiming to be in `Law` would mark a group that cannot
  // reach it.
  const unlifted = corpus.classes
    .map((c) => c.className)
    .filter((className) => !(className in LIFTED));
  expect(unlifted.length).toBeGreaterThan(0);
  for (const className of unlifted) expect(sectionForClass(className), className).toBe("wiki");
});

test("the bar has five groups, each with a front door and a line saying what it answers", () => {
  expect(groups.map((g) => g.label)).toEqual([
    "Places",
    "Law",
    "Formula",
    "Research",
    "Reference",
  ]);
  for (const group of groups) {
    expect(group.sections.length, `${group.label} has no sections`).toBeGreaterThan(0);
    expect(group.blurb.length, `${group.label} has no blurb`).toBeGreaterThan(20);
    expect(group.front, `${group.label} has no front door`).toMatch(/^\//);
    for (const run of group.sections) {
      expect(run.links.length, `${group.label} has an empty run`).toBeGreaterThan(0);
    }
  }
});

test("no group is empty, and none has grown past what a panel can hold", () => {
  /*
   * The upper bound is not tidiness. The panel is a `<details>` with no scrolling of its own, so
   * a run of items past roughly a dozen leaves its tail below the fold of a laptop viewport,
   * under a sticky header, reachable only by scrolling the page beneath a floating box. `Law` is
   * the one at risk: it is seven acts today and every act added to the corpus is a candidate.
   */
  for (const group of groups) {
    const count = group.sections.reduce((n, run) => n + run.links.length, 0);
    expect(count, `${group.label} is empty`).toBeGreaterThan(0);
    expect(count, `${group.label} has ${count} links and will not fit its panel`).toBeLessThan(13);
  }
});

test("the bar does not point at the homepage, which has no entry by design", () => {
  // The homepage is what the brand mark opens. A tab as well would put one destination in the bar
  // twice, and this is the assertion that keeps it out — `/` is an easy href to add back.
  expect(links.filter((link) => link.href === "/")).toEqual([]);
});

test("a note never repeats its own label", () => {
  /*
   * Notes are the second line under a link, and they are load-bearing: the rule that admitted an
   * act, the year a regime was established. A note that restates its heading — `Components` over
   * `16 components` — teaches a reader that the second line carries nothing, and then they stop
   * reading the ones that do. That is why the class counts sit inside their labels instead.
   */
  for (const link of links) {
    if (!link.note) continue;
    expect(
      link.note.toLowerCase().includes(link.label.toLowerCase()),
      `"${link.label}" has a note that says it again: "${link.note}"`,
    ).toBe(false);
  }
});

test("every link has somewhere to go and something to be called", () => {
  for (const link of links) {
    expect(link.href, JSON.stringify(link)).toMatch(/^\/[^\s]*$/);
    expect(link.label.trim().length, JSON.stringify(link)).toBeGreaterThan(0);
  }
  // Two links to one place in one bar is a reader clicking to where they already were.
  const hrefs = links.map((l) => l.href);
  expect(new Set(hrefs).size, `duplicate hrefs: ${hrefs.join(", ")}`).toBe(hrefs.length);
});
