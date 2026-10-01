/**
 * A section's `#` ends its heading and never wraps onto a line of its own (#549).
 *
 * Two writers produce the shape: `heading()` in `section.ts` writes it directly, and
 * `moveAnchors` in `semantics.ts` rebuilds it from the older `anchor()`-first markup. Both have to
 * land on the same tree, because the stylesheet hides and reveals the mark by that tree and a
 * heading in the other shape would show its `#` permanently and let it wrap.
 */

import { parseHTML } from "linkedom";
import { describe, expect, test } from "vitest";

import { contentsOf, renderContents } from "../../src/lib/contents.ts";
import { anchor, heading } from "../../src/lib/section.ts";
import { applySemantics } from "../../src/lib/semantics.ts";

function h2(html: string): Element {
  const { document } = parseHTML(`<!doctype html><html><body>${html}</body></html>`);
  return document.querySelector("h2")!;
}

/** What the tree says, in the terms the stylesheet reads it by. */
function shape(el: Element) {
  const tail = el.querySelector(".heading-tail");
  return {
    textFirst: el.firstElementChild?.className,
    tailInText: tail?.parentElement?.className,
    tailStartsWithSpace: /^\s/.test(tail?.firstChild?.textContent ?? ""),
    anchorInTail: el.querySelector("a.section-anchor")?.parentElement?.className,
    anchors: el.querySelectorAll("a.section-anchor").length,
  };
}

const EXPECTED = {
  textFirst: "heading-text",
  tailInText: "heading-text",
  tailStartsWithSpace: true,
  anchorInTail: "heading-tail",
  anchors: 1,
};

describe("the section anchor", () => {
  test("heading() writes the title, then a tail holding the space and the mark", () => {
    expect(shape(h2(`<h2>${heading("aid-source", "Where the state aid comes from")}</h2>`))).toEqual(EXPECTED);
  });

  test("moveAnchors rebuilds anchor()-first markup into the same tree", () => {
    const { html, anchored } = applySemantics(`<h2>${anchor("limits")}What these figures are not</h2>`);
    expect(anchored).toBe(1);
    const el = h2(html);
    expect(shape(el)).toEqual(EXPECTED);
    // The title's last word and the mark share the no-wrap tail's line; nothing sits between them
    // but the one space inside it.
    expect(el.textContent).toBe("What these figures are not #");
  });

  test("and keeps a year chip after the text, outside the tail", () => {
    const { html } = applySemantics(
      `<h2>${anchor("tiles")}State aid <span class="year-chip-wrap"><button>FY2027</button></span></h2>`,
    );
    const el = h2(html);
    expect(shape(el)).toEqual(EXPECTED);
    expect(el.lastElementChild?.className).toBe("year-chip-wrap");
  });

  test("and leaves a heading heading() already wrote alone", () => {
    const written = `<h2>${heading("x", "Already done")}</h2>`;
    const { html, anchored } = applySemantics(written);
    expect(anchored).toBe(0);
    expect(html).toBe(written);
  });
});

/**
 * The way back to the contents list, beside the address (#594).
 *
 * `linkContents` gives one to every heading with an address, on a page that has a list for it to
 * lead back to, and to nothing on a page that has none.
 */
describe("the link back to the contents", () => {
  const card = (id: string, title: string, inner = "") =>
    `<div class="card" id="${id}"><h2>${heading(id, title)}</h2>${inner}</div>`;
  const page = (cards: string) => renderContents(contentsOf(cards)) + cards;

  test("sits in the tail of every section heading, after the address", () => {
    const cards = ["a", "b", "c", "d"].map((id) => card(id, `Section ${id}`)).join("");
    const { html, linked } = applySemantics(page(cards));
    expect(linked).toBe(4);
    const { document } = parseHTML(`<!doctype html><html><body>${html}</body></html>`);
    for (const tail of document.querySelectorAll("h2 .heading-tail")) {
      const [address, back] = [...tail.querySelectorAll("a")];
      expect(address?.className).toBe("section-anchor");
      expect(back?.className).toBe("to-contents");
      expect(back?.getAttribute("href")).toBe("#contents");
    }
    // It lands somewhere: the list carries the id the link names.
    expect(document.querySelector("nav.contents")?.id).toBe("contents");
    // Separated from the `#` by a space, never fused to it.
    expect(document.querySelector("h2")?.textContent).toBe("Section a # ↑ Contents");
  });

  test("and in a card's own h3s too, so a long card is not a stretch with no way back", () => {
    const inner = `<h3>${heading("a-row", "A row of the breakdown")}</h3>`;
    const cards = card("a", "Section a", inner) + ["b", "c", "d"].map((id) => card(id, id)).join("");
    const { html, linked } = applySemantics(page(cards));
    expect(linked).toBe(5);
    const { document } = parseHTML(`<!doctype html><html><body>${html}</body></html>`);
    expect(document.querySelector("h3 .heading-tail .to-contents")).not.toBeNull();
  });

  test("and nowhere on a page with no list", () => {
    const { html, linked } = applySemantics(card("a", "Alone"));
    expect(linked).toBe(0);
    expect(html).not.toContain("to-contents");
  });
});
