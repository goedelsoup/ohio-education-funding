/**
 * No node page says its first thing twice (#551). The rendered half is `tests/dist/wikiLead.spec.ts`,
 * which asks the same question of the built lead and the paragraph under it.
 */

import { expect, test } from "vitest";

import { loadCorpus } from "../../src/lib/corpus.ts";
import { leadAndBody, overlap, paragraphs, readerText, restates, RESTATES_AT } from "../../src/lib/lead.ts";

/** A description as `renderProse` shapes it, closely enough for the split: one `<p>` per paragraph. */
const asHtml = (markdown: string): string => paragraphs(markdown).map((p) => `<p>${p}</p>`).join("\n");

/** The lead a page shows and the first paragraph of the body under it, as reader text. */
function pageOf(summary: string, description: string): { lead: string; next: string } {
  const { lead, body } = leadAndBody(summary, description, asHtml(description));
  return {
    lead: readerText("html" in lead ? lead.html : lead.text),
    next: readerText(body.match(/<p>([\s\S]*?)<\/p>/)?.[1] ?? ""),
  };
}

test("no node's lead restates the paragraph under it", () => {
  const repeated: string[] = [];
  for (const node of loadCorpus().nodes) {
    const { lead, next } = pageOf(node.summary, node.description);
    const score = overlap(lead, next);
    if (score >= RESTATES_AT) repeated.push(`${node.id} ${score.toFixed(2)}`);
  }
  expect(repeated).toEqual([]);
});

test("the check bites on a real node rendered the old way: its summary above its own opening", () => {
  // H.B. 110, the case #551 named. Its summary and the description's first paragraph share an
  // opening sentence and differ after it — a paraphrase no prefix comparison sees.
  const node = loadCorpus().byId.get("legislation/hb-110-2021")!;
  const opening = paragraphs(node.description)[0]!;
  expect(node.description.trim().startsWith(node.summary)).toBe(false);
  expect(restates(node.summary, opening)).toBe(true);
  expect(overlap(readerText(node.summary), readerText(opening))).toBeGreaterThanOrEqual(RESTATES_AT);

  // And the page no longer renders it that way: the opening is the lead, the body starts after it.
  const { lead, body } = leadAndBody(node.summary, node.description, asHtml(node.description));
  expect("html" in lead && lead.html).toBe(opening);
  expect(body).not.toContain(opening);
});

test("a summary that does not restate the opening stays the lead, and the body stays whole", () => {
  const summary = "A lead about one thing, written for a reader arriving from a search result page.";
  const description = "An opening about a different matter altogether, the statute's arithmetic.\n\nMore.";
  const html = asHtml(description);
  expect(leadAndBody(summary, description, html)).toEqual({ lead: { text: summary }, body: html });
});

test("the threshold sits above what adjacent body paragraphs share", () => {
  /*
   * The calibration `lib/lead.ts` states, measured rather than trusted. Two adjacent paragraphs of
   * one description are distinct by their author's intent; if many of them reached the threshold
   * it would be flagging a shared subject, not a repeat.
   */
  let pairs = 0;
  let over = 0;
  for (const node of loadCorpus().nodes) {
    const ps = paragraphs(node.description).map(readerText);
    for (let i = 0; i + 1 < ps.length; i++) {
      pairs += 1;
      if (overlap(ps[i]!, ps[i + 1]!) >= RESTATES_AT) over += 1;
    }
  }
  expect(pairs).toBeGreaterThan(300);
  expect(over / pairs).toBeLessThan(0.01);
});
