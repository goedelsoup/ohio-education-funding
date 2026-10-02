/**
 * The narrowed reads in `artefact.ts` read what a whole parse reads (#646).
 *
 * `head` and `drawingsAsText` exist to make sweeps cheaper, and a cheaper sweep that sees less is
 * the thing to fear: it passes on nothing and looks the same as one that passes on everything.
 * So this holds the two claims each one rests on — the preconditions, over every drawing in the
 * build, and the equivalence, on real pages — so that a template change which breaks either one
 * fails here rather than quietly narrowing six other tests.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { parseHTML } from "linkedom";
import { describe, expect, test } from "vitest";

import { DIST, drawingsAsText, head, pages } from "./artefact.ts";

const read = (file: string) => readFileSync(join(DIST, file), "utf8");

/** The text a sweep reads: title and body, without scripts, styles or templates. */
function visibleText(html: string): string {
  const { document } = parseHTML(html);
  for (const node of document.querySelectorAll("script, style, template")) node.remove();
  return `${document.querySelector("title")?.textContent ?? ""}\n${document.body?.textContent ?? ""}`;
}

/** Pages heavy with drawings, of the families the sweeps read: a district, its tabs, the state. */
const SAMPLES = [
  "index.html",
  "statewide.html",
  "district/043786.html",
  "district/043786/outcome.html",
  "district/043786/finances.html",
];

describe("drawingsAsText", () => {
  test("no drawing in the build holds what reducing it to text would change", () => {
    let drawings = 0;
    const faults: string[] = [];
    for (const file of pages()) {
      for (const svg of readFileSync(file, "utf8").match(/<svg\b[\s\S]*?<\/svg>/g) ?? []) {
        drawings += 1;
        const at = file.slice(DIST.length + 1);
        // Nesting would end the lazy match at the inner `</svg>` and leave the outer's tail as markup.
        if (svg.slice(4).includes("<svg")) faults.push(`${at}: a drawing inside a drawing`);
        // A comment's text is not `textContent`; a style's or a script's is removed by the sweeps.
        if (/<!--|<style\b|<script\b|<!\[CDATA\[|<foreignObject\b/.test(svg)) faults.push(`${at}: ${svg.slice(0, 80)}`);
      }
    }
    expect(drawings, "the build carries drawings at all").toBeGreaterThan(10_000);
    expect(faults.slice(0, 5)).toEqual([]);
  });

  test.each(SAMPLES)("%s reads the same text with its drawings reduced", (file) => {
    const html = read(file);
    const reduced = drawingsAsText(html);
    // Something was reduced, or this compares a page with itself.
    expect(reduced.length).toBeLessThan(html.length);
    expect(visibleText(reduced)).toBe(visibleText(html));
  });

  test("a chart's text survives the reduction", () => {
    const reduced = drawingsAsText(read("statewide.html"));
    expect(reduced).not.toMatch(/<svg\b/);
    expect(reduced).not.toMatch(/<(?:rect|path|text|g)\b/);
    // Each direct label and axis end, as text where its element was.
    const labels = [...read("statewide.html").matchAll(/<text\b[^>]*>([^<]+)<\/text>/g)].map((m) => m[1]!);
    expect(labels.length).toBeGreaterThan(10);
    for (const label of labels.slice(0, 20)) expect(reduced).toContain(label);
  });
});

describe("head", () => {
  test.each(SAMPLES)("%s reads the same title and metas from its head alone", (file) => {
    const html = read(file);
    const tags = (doc: Document) => [
      doc.querySelector("title")?.textContent,
      ...[...doc.querySelectorAll("meta")].map((m) => m.outerHTML),
    ];
    const whole = tags(parseHTML(html).document as unknown as Document);
    expect(whole[0], "the page has a title").toBeTruthy();
    expect(tags(parseHTML(head(html, file)).document as unknown as Document)).toEqual(whole);
  });

  test("a page with no head is refused, not read as untitled", () => {
    expect(() => head("<html><body><title>x</title></body></html>", "doctored.html")).toThrow(/no <\/head>/);
  });
});
