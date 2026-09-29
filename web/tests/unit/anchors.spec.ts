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
