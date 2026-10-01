/**
 * The runner and the words it shares, read from the built pages.
 *
 * # One baseline, two views (#562)
 *
 * Both views carried a card headed "What the levers move away from" under one anchor, and they
 * gave two accounts of it: one named the act's terms and its earlier stages, the other its size. A
 * reader crossing the tabs could not tell whether the two cards described one baseline. They are
 * one component now, `Baseline.astro`, and this holds the bytes to it — the card is the same text
 * on both views except for one paragraph, which is marked `data-view-specific` and opens with the
 * view's own tab label so the reader can see that it is the view talking.
 *
 * # One word, two things (#560)
 *
 * The runner lives at `/scenario` and the corpus had a class labelled `Scenario` in Library. The
 * class is `Worked Scenario` now, and the two point at each other: the class index and each of its
 * nodes link the runner, and the runner's baseline card links the class. A label alone would make
 * the collision smaller; the links are what let a reader tell from the page which one they are on.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { parseHTML } from "linkedom";
import { describe, expect, test } from "vitest";

import { RUNNER_VIEWS, type RunnerView } from "../../src/lib/nav.ts";
import { DIST } from "./artefact.ts";

const VIEWS = { changes: "scenario.html", reach: "scenario/reach.html" } as const;
const TABS = Object.fromEntries(RUNNER_VIEWS.map((v) => [v.key, v.label])) as Record<RunnerView, string>;

const doc = (file: string): Document => parseHTML(readFileSync(join(DIST, file), "utf8")).document;
const text = (el: Element): string => (el.textContent ?? "").replace(/\s+/g, " ").trim();

/** The baseline card, and its text with the view-specific paragraph taken out. */
function baseline(file: string): { card: Element; specific: Element[]; shared: string } {
  const card = doc(file).querySelector('[data-part="baseline"]');
  if (!card) throw new Error(`${file} has no baseline card`);
  const specific = [...card.querySelectorAll("[data-view-specific]")];
  const copy = card.cloneNode(true) as Element;
  for (const p of copy.querySelectorAll("[data-view-specific]")) p.remove();
  return { card, specific, shared: text(copy) };
}

describe("the runner's baseline card (#562)", () => {
  test("both views give the same account of the baseline", () => {
    const changes = baseline(VIEWS.changes);
    const reach = baseline(VIEWS.reach);
    expect(reach.shared).toBe(changes.shared);
    // The parts the two cards used to split between them, now both on each.
    expect(changes.shared).toContain("as enacted");
    expect(changes.shared).toContain("guarantee money");
    expect(changes.shared).toContain("earlier stages are not lever positions");
  });

  test("the one paragraph that differs is labelled with its view", () => {
    for (const view of ["changes", "reach"] as const) {
      const { specific } = baseline(VIEWS[view]);
      expect(specific, `${VIEWS[view]} has one view-specific paragraph`).toHaveLength(1);
      expect(text(specific[0]!)).toMatch(new RegExp(`^In ${TABS[view]},`));
    }
    expect(text(baseline(VIEWS.changes).specific[0]!)).not.toBe(
      text(baseline(VIEWS.reach).specific[0]!),
    );
  });

  test("the guard sees a drifted copy", () => {
    // A test that compares two copies of one component passes on anything; this is the doctored
    // case it has to fail on — a word changed in the shared text of one view.
    const { card } = baseline(VIEWS.reach);
    const doctored = card.cloneNode(true) as Element;
    doctored.querySelector("p:not([data-view-specific])")!.append(" and something else");
    for (const p of doctored.querySelectorAll("[data-view-specific]")) p.remove();
    expect(text(doctored)).not.toBe(baseline(VIEWS.changes).shared);
  });
});

describe("the runner and the worked scenarios (#560)", () => {
  test("the class is not labelled with the runner's word", () => {
    const page = doc("wiki/scenario.html");
    expect(text(page.querySelector("h1")!)).toBe("Worked Scenario");
    // Nor in Library, where the bar lists the classes on every page.
    const library = [...doc("index.html").querySelectorAll("header.site nav a")].map(text);
    expect(library.some((label) => /^Worked Scenario \(\d+\)$/.test(label))).toBe(true);
    expect(library.some((label) => /^Scenarios? \(\d+\)$/.test(label))).toBe(false);
  });

  test("the class index and every worked scenario point at the runner", () => {
    const index = doc("wiki/scenario.html");
    const nodes = [...index.querySelectorAll('[data-part="nodes"] th a')].map((a) =>
      a.getAttribute("href")!,
    );
    expect(nodes.length).toBeGreaterThan(0);
    expect(index.querySelector('main a[href="/scenario"]'), "the class index").not.toBeNull();
    for (const href of nodes) {
      const node = doc(`${href.replace(/^\//, "")}.html`);
      expect(node.querySelector('[data-part="runner"] a[href="/scenario"]'), href).not.toBeNull();
    }
  });

  test("both views of the runner point at the worked scenarios", () => {
    for (const file of Object.values(VIEWS)) {
      const card = baseline(file).card;
      expect(card.querySelector('a[href="/wiki/scenario"]'), file).not.toBeNull();
    }
  });
});
