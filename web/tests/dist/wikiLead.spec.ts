/**
 * No node page says its first thing twice (#551), asked of the page as built.
 *
 * `tests/unit/lead.spec.ts` asks it of the corpus with the description's paragraphs standing in
 * for the rendered ones. This reads the lead a reader is shown and the first paragraph under it,
 * which is where a template that forgot to drop the paragraph it promoted would show.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { parseHTML } from "linkedom";
import { expect, test } from "vitest";

import { loadCorpus } from "../../src/lib/corpus.ts";
import { overlap, readerText, RESTATES_AT } from "../../src/lib/lead.ts";
import { DIST } from "./artefact.ts";

/** The lead and the description's first paragraph on one built page, as a reader reads them. */
function opening(html: string): { lead: string; next: string } {
  const { document } = parseHTML(html);
  const lead = document.querySelector("main > p.lead")?.innerHTML ?? "";
  const next = document.querySelector("#description > p")?.innerHTML ?? "";
  return { lead: readerText(lead), next: readerText(next) };
}

test("no node page's lead restates the paragraph under it", () => {
  const nodes = loadCorpus().nodes;
  expect(nodes.length).toBeGreaterThan(100);
  const repeated: string[] = [];
  let compared = 0;
  for (const node of nodes) {
    const { lead, next } = opening(readFileSync(join(DIST, `wiki/${node.id}.html`), "utf8"));
    if (lead === "" || next === "") continue;
    compared += 1;
    const score = overlap(lead, next);
    if (score >= RESTATES_AT) repeated.push(`${node.id} ${score.toFixed(2)}`);
  }
  expect(compared).toBeGreaterThan(100);
  expect(repeated).toEqual([]);
});

test("the check bites on H.B. 110 with its summary put back above its own opening", () => {
  // The page as it rendered before #551: the summary as the lead, and the paragraph it paraphrases
  // left in place as the first thing under it.
  const node = loadCorpus().byId.get("legislation/hb-110-2021")!;
  const html = readFileSync(join(DIST, "wiki/legislation/hb-110-2021.html"), "utf8");
  const { document } = parseHTML(html);
  const lead = document.querySelector("main > p.lead")!;
  const description = document.querySelector("#description")!;
  const promoted = lead.innerHTML;
  lead.textContent = node.summary;
  description.insertAdjacentHTML("afterbegin", `<p>${promoted}</p>`);

  const { lead: shown, next } = opening(document.toString());
  expect(overlap(shown, next)).toBeGreaterThanOrEqual(RESTATES_AT);
});
