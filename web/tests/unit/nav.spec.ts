/**
 * The section bar, which is part typed and part read out of the corpus.
 *
 * # Why this file exists
 *
 * A bar that is typed out is wrong only when somebody types it wrong, and it is wrong visibly.
 * This one reads `.yidam/corpus/` — which classes sit under `Library`, and how many nodes each
 * holds — so it can go wrong without anybody touching a line of it. Rename a class and every
 * page's header points at a route the build does not emit; add one and, unless something places
 * it, the bar never mentions it.
 *
 * The end-to-end suite already walks every href in the rendered header and opens it, which is the
 * check that a link goes somewhere. What it cannot ask is whether the *right* things were
 * selected, because a menu that has silently emptied out still renders and every one of its zero
 * links still resolves. That question belongs here, against the module rather than the markup.
 */

import { expect, test } from "vitest";

import { loadCorpus } from "../../src/lib/corpus.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import {
  DISTRICT_VIEWS,
  LIBRARY,
  NAMES,
  UNPLACED,
  districtTitle,
  nav,
  navLinks,
  sectionForClass,
  type NavGroup,
} from "../../src/lib/nav.ts";

const corpus = loadCorpus();
const { bundle } = loadFeed();
const entries = nav(bundle, corpus);
const links = navLinks(entries);
const groups = entries.filter((e): e is NavGroup => e.kind === "group");

test("the bar is six entries by task, two of them flat links", () => {
  /*
   * #548's target, written out. The two flat entries are the two things a reader most often came
   * to do — find one district, change the formula — and a menu in front of either is a click that
   * decides nothing.
   */
  expect(entries.map((e) => [e.kind, e.label])).toEqual([
    ["place", "Find a district"],
    ["group", "Places"],
    ["group", "Analysis"],
    ["place", "Change the formula"],
    ["group", "Library"],
    ["group", "About"],
  ]);
  const flat = entries.filter((e) => e.kind === "place").map((e) => e.front);
  expect(flat).toEqual([NAMES.districts.href, NAMES.scenario.href]);
});

test("each menu holds the places #548 put in it, in its order", () => {
  const labels = (label: string) =>
    groups
      .find((g) => g.label === label)!
      .sections.flatMap((s) => s.links.map((l) => l.label));
  expect(labels("Places")).toEqual([
    "Statewide",
    "What changed",
    "Counties",
    "House districts",
    "Senate districts",
    "Compare",
  ]);
  expect(labels("Analysis")).toEqual(["Outcomes", "History", "Bounds"]);
  expect(labels("About")).toEqual(["Method", "Data"]);
});

test("every corpus class is two clicks from the bar: one to open Library, one to arrive", () => {
  /*
   * The point of the redesign, as an assertion. Before it, seven of eighteen classes were lifted
   * into `Law` and `Formula` and the other eleven — `school`, `program`, `actor` and the rest —
   * were reachable only by going to `/wiki` and reading down a list.
   *
   * Every class, read off the corpus rather than listed here, so a class added to `.yidam/` is
   * checked the day it lands.
   */
  const library = groups.find((g) => g.key === "library")!;
  const reachable = new Set(
    library.sections.flatMap((run) =>
      run.links.flatMap((link) => {
        const wiki = link.href.match(/^\/wiki\/([^/]+)$/);
        return wiki ? [wiki[1]!] : [];
      }),
    ),
  );
  const classes = corpus.classes.map((c) => c.className);
  expect(classes.length).toBeGreaterThan(15);
  for (const className of classes) {
    expect(reachable.has(className), `${className} is not in Library`).toBe(true);
  }
});

test("every class is placed by name, and none has fallen into the catch-all run", () => {
  /*
   * The catch-all is what keeps a new class reachable before anybody has decided where it goes.
   * It is a notification, not a place: while it exists this fails, and the fix is a line in
   * `LIBRARY`. And a name in `LIBRARY` the corpus does not hold is a typo, or a class retired
   * without the bar being told.
   */
  const library = groups.find((g) => g.key === "library")!;
  expect(library.sections.map((s) => s.heading)).not.toContain(UNPLACED);
  const placed = LIBRARY.flatMap((run) => run.classes);
  expect(new Set(placed).size, "a class is placed twice").toBe(placed.length);
  for (const className of placed) {
    expect(corpus.byClass.has(className), `LIBRARY names ${className}, which is not a class`).toBe(
      true,
    );
  }
});

test("the catch-all run appears when a class is unplaced, and carries it", () => {
  /*
   * The guard above passes on every real corpus, so it is paired with a doctored one: a corpus
   * holding a class `LIBRARY` does not name. Without this, a catch-all that silently dropped the
   * class would pass the test above and leave it unreachable.
   */
  const extra = { ...corpus.byClass.get("school")!, className: "made-up-class", label: "Made Up" };
  const doctored = {
    ...corpus,
    classes: [...corpus.classes, extra],
    byClass: new Map([...corpus.byClass, ["made-up-class", extra]]),
  };
  const library = nav(bundle, doctored).find((e) => e.key === "library") as NavGroup;
  const run = library.sections.find((s) => s.heading === UNPLACED);
  expect(run?.links.map((l) => l.href)).toEqual(["/wiki/made-up-class"]);
});

test("every corpus page reports itself as Library", () => {
  // A class page claiming a group that cannot reach it would tell a reader on H.B. 110 they are
  // somewhere the bar does not show them.
  for (const c of corpus.classes) expect(sectionForClass(c.className), c.className).toBe("library");
});

test("every corpus link in the bar lands on a class that exists", () => {
  const missing: string[] = [];
  for (const link of links) {
    const wiki = link.href.match(/^\/wiki\/([^/]+)$/);
    if (!wiki || wiki[1] === "source" || wiki[1] === "decision") continue;
    if (!corpus.byClass.has(wiki[1]!)) missing.push(`${link.label} → class ${wiki[1]}`);
  }
  expect(missing).toEqual([]);
});

test("every entry has a front door and a line saying what it answers", () => {
  for (const entry of entries) {
    expect(entry.blurb.length, `${entry.label} has no blurb`).toBeGreaterThan(20);
    expect(entry.front, `${entry.label} has no front door`).toMatch(/^\//);
  }
  for (const group of groups) {
    expect(group.sections.length, `${group.label} has no sections`).toBeGreaterThan(0);
    for (const run of group.sections) {
      expect(run.links.length, `${group.label} has an empty run`).toBeGreaterThan(0);
    }
  }
});

test("no column of a panel has grown past what it can hold", () => {
  /*
   * The bound is not tidiness. The panel is a `<details>` with no scrolling of its own above
   * 640px, so a column past roughly a dozen rows leaves its tail below the fold of a laptop
   * viewport, under a sticky header, reachable only by scrolling the page beneath a floating box.
   *
   * A one-column panel is one column. `Library` is two (`.menu-wide`), which `columns` balances by
   * height, so what it has to hold is half its rows, each heading counted as one — held under the
   * same bound, and a run may not be larger than a column on its own.
   */
  for (const group of groups) {
    const rows = group.sections.reduce((n, run) => n + run.links.length + (run.heading ? 1 : 0), 0);
    const perColumn = group.wide ? Math.ceil(rows / 2) : rows;
    expect(perColumn, `${group.label} has ${rows} rows and will not fit its panel`).toBeLessThan(
      17,
    );
    for (const run of group.sections) {
      expect(run.links.length, `${group.label} › ${run.heading ?? "a run"}`).toBeLessThan(13);
    }
  }
});

test("the bar does not point at the homepage, which has no entry by design", () => {
  // The homepage is what the brand mark opens. A tab as well would put one destination in the bar
  // twice, and this is the assertion that keeps it out — `/` is an easy href to add back.
  expect(links.filter((link) => link.href === "/")).toEqual([]);
});

test("a note never repeats its own label", () => {
  /*
   * Notes are the second line under a link, and they are load-bearing. A note that restates its
   * heading — `Components` over `16 components` — teaches a reader that the second line carries
   * nothing, and then they stop reading the ones that do. That is why the class counts sit inside
   * their labels instead.
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

test("a district tab's address is its name, and its title says it after the district", () => {
  /*
   * #559. The fourth tab was `Property tax` over `/district/[irn]/taxes`. Every view of the
   * district's own ends in its name, lower-cased; the dashboard is the district itself and ends in
   * nothing; the runner keeps its own address and the bar's name for it. The built pages' titles
   * are held to the same names in `tests/dist/names.spec.ts`.
   */
  const irn = "043786";
  for (const view of DISTRICT_VIEWS) {
    const href = view.href(irn);
    if (view.key === "scenario") {
      expect(view.name).toBe(NAMES.scenario.name);
      expect(new URL(href, "https://host.invalid").pathname).toBe(NAMES.scenario.href);
    } else if (view.key === "dashboard") {
      expect(href).toBe(`/district/${irn}`);
      expect(districtTitle("Cleveland Municipal", view.key)).toBe("Cleveland Municipal");
    } else {
      expect(href, view.name).toBe(`/district/${irn}/${view.name.toLowerCase().replace(/ /g, "-")}`);
      expect(districtTitle("Cleveland Municipal", view.key)).toBe(`Cleveland Municipal — ${view.name}`);
    }
  }
});
