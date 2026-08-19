/**
 * The table of contents, over the shapes the renderers actually emit.
 *
 * It is derived from the rendered body rather than declared beside it — `src/lib/contents.ts` says
 * at length why — which means the whole feature is one function from an HTML string to a list, and
 * that is a thing the unit suite can hold exhaustively without a browser or a build.
 */

import { expect, test } from "vitest";

import { contentsOf, renderContents, withContents } from "../../src/lib/contents.ts";
import { anchor } from "../../src/lib/section.ts";

/** A card as the renderers write one. */
const card = (id: string, heading: string, chip = "") =>
  `<div class="card" id="${id}" data-part="${id}"><h2>${anchor(id)}${heading}${chip}</h2><p>body</p></div>`;

const CHIP = '<span class="year-chip" data-kind="fiscal">FY2027</span>';

test("a section is listed by its card's address and its heading's words", () => {
  const body = `<h1>A page</h1><p class="sub">What it is.</p>${card("base-cost", "Why base cost is $8,120 per pupil", CHIP)}`;
  expect(contentsOf(body)).toEqual([{ id: "base-cost", label: "Why base cost is $8,120 per pupil" }]);
});

test("the anchor and the year chip are not part of the entry", () => {
  /*
   * Both sit inside the heading and neither is what the section is about. The chip is the more
   * misleading of the two: in a list of eight entries an "FY2027" on each reads as a distinction
   * between them rather than as an annotation on every one.
   */
  const [entry] = contentsOf(card("millage", "What voters approved", CHIP));
  expect(entry!.label).not.toContain("#");
  expect(entry!.label).not.toContain("FY2027");
  expect(entry!.label).toBe("What voters approved");
});

test("a heading carrying its own id is listed by that id, not by its card's", () => {
  // The corpus prose grows headings and the markdown processor slugs each one, so a `.prose-body`
  // card holds several sections rather than being one.
  const body =
    `<div class="card prose-body" id="description">` +
    `<p>Lede.</p>` +
    `<h2 id="used-by">${anchor("used-by")}Used by</h2>` +
    `<h3 id="two-traps">${anchor("two-traps")}Two traps in this file</h3>` +
    `</div>`;
  expect(contentsOf(body)).toEqual([
    { id: "used-by", label: "Used by" },
    { id: "two-traps", label: "Two traps in this file" },
  ]);
});

test("a card's own sub-headings are not sections of the page", () => {
  /*
   * The distinction is not depth. The district dashboard heads the six categoricals plus
   * transportation and preschool with an `h3` inside one card: those are the rows of a breakdown,
   * and listing them produces a contents list longer than the card it summarises. They keep their
   * addresses and their anchors — they are simply not a way into the page.
   */
  const body =
    `<div class="card" id="categoricals" data-part="categoricals">` +
    `<h2>${anchor("categoricals")}The categorical half</h2>` +
    `<h3 id="dpia">${anchor("dpia")}Disadvantaged Pupil Impact Aid</h3>` +
    `</div>`;
  expect(contentsOf(body).map((e) => e.id)).toEqual(["categoricals"]);
});

test("a section rendered once per dollar basis is one entry, named by what both headings agree on", () => {
  /*
   * The statewide finances card names its basis in its own heading, because both panels are in the
   * document and a reader with no script has to be able to tell which is which. Neither "…— nominal"
   * nor "…— FY2020 dollars" is the name of the section; what they share is.
   */
  const body =
    `<div class="basis-scope" id="finances" data-part="finances">` +
    `<div class="basis-panel nominal"><div class="card" data-part="finances">` +
    `<h2>${anchor("finances")}What districts actually received, spent, and hold — nominal</h2></div></div>` +
    `<div class="basis-panel real"><div class="card" data-part="finances">` +
    `<h2>${anchor("finances")}What districts actually received, spent, and hold — FY2020 dollars</h2></div></div>` +
    `</div>`;
  expect(contentsOf(body)).toEqual([
    { id: "finances", label: "What districts actually received, spent, and hold" },
  ]);
});

test("two panels that say the same thing keep the whole heading", () => {
  // Three of the four basis sections do not name their basis in the heading, and the common-prefix
  // rule has to leave those alone rather than shortening them to nothing in particular.
  const panel = `<div class="card" data-part="actuals"><h2>${anchor("actuals")}What it actually received, and what it holds</h2></div>`;
  const body = `<div class="basis-scope" id="actuals" data-part="actuals">${panel}${panel}</div>`;
  expect(contentsOf(body)).toEqual([
    { id: "actuals", label: "What it actually received, and what it holds" },
  ]);
});

test("a heading with no address anywhere above it is skipped rather than guessed at", () => {
  expect(contentsOf("<div class='card'><h2>Nowhere</h2></div>")).toEqual([]);
});

test("the list goes above the first section and below the page introducing itself", () => {
  /*
   * The boundary is found rather than marked. Everything before the first section element is the
   * page introducing itself — the `h1`, the sentence under it, and on a district route the
   * sub-navigation — and a contents list above that sits above the title of the page.
   */
  const intro = `<h1>Cleveland Municipal</h1><p class="sub">IRN 043786</p><nav class="subnav"><a href="/x">Dashboard</a></nav>`;
  const sections = card("a", "One") + card("b", "Two") + card("c", "Three") + card("d", "Four");
  const out = withContents(intro + sections, contentsOf(intro + sections));

  expect(out.indexOf("<nav class=\"contents\"")).toBeGreaterThan(out.indexOf("subnav"));
  expect(out.indexOf("<nav class=\"contents\"")).toBeLessThan(out.indexOf('<div class="card" id="a"'));
  // And the body is otherwise untouched: this is an insertion, not a rewrite.
  expect(out.replace(/<nav class="contents".*?<\/nav>/s, "")).toBe(intro + sections);
});

test("a page with too few sections gets no list at all", () => {
  // A list of three is longer than what it lists.
  const body = card("a", "One") + card("b", "Two") + card("c", "Three");
  expect(withContents(body, contentsOf(body))).toBe(body);

  const four = body + card("d", "Four");
  expect(withContents(four, contentsOf(four))).toContain('nav class="contents"');
});

test("a label is escaped, because a heading is prose and prose contains ampersands", () => {
  const rendered = renderContents([{ id: "x", label: 'Fish & chips "and" <b>' }]);
  expect(rendered).toContain("Fish &amp; chips &quot;and&quot; &lt;b&gt;");
  expect(rendered).not.toContain("<b>");
});

test("every entry is a fragment link into the page it sits on", () => {
  // Which is what makes `check-dist-links.ts` the check on this: an entry naming an id the page
  // does not carry is a dangling same-page fragment, and that fails the build.
  const rendered = renderContents([{ id: "base-cost", label: "Why base cost is what it is" }]);
  expect(rendered).toContain('href="#base-cost"');
});
