/**
 * Every Explained topic, as markup: one renderer, so every page has the same eight sections in the
 * same order.
 *
 * Strings rather than components, on the pattern of `bounds.ts` and `whatChanged.ts`: a unit test
 * can render a topic and read it with `linkedom` without a build, and the style checks of phase 1
 * read the same strings the page ships. `[slug].astro` sets the question as its `h1` and places these
 * under it.
 *
 * # Claim badges, here and nowhere else
 *
 * {@link renderHowWeKnow} runs `badgeClaims` over its claims, which is what draws `[verified]` as a
 * badge on a corpus node. No other section does, so a bracket in plain prose stays a bracket and the
 * plain sections stay plain (`.yidam/decisions/explained-is-a-reading-not-a-record.yml`).
 */

import { loadCorpus, type Corpus } from "../corpus.ts";
import { escapeHtml } from "../format.ts";
import { badgeClaims } from "../prose.ts";
import * as routes from "../routes.ts";
import { heading } from "../section.ts";
import { yearChip } from "../year.ts";
import { topicModule } from "./registry.ts";
import type { Prose, Topic } from "./topic.ts";

const ID = routes.SECTIONS.explained;

/** Paragraphs of prose, each in the card's note register. */
const paragraphs = (runs: Prose[]): string => runs.map((p) => `<p class="note">${p}</p>`).join("");

/** A card with an addressable heading. */
function card(id: string, title: string, body: string, chip = ""): string {
  return `
    <div class="card" id="${id}" data-part="${id}">
      <h2>${heading(id, title, chip)}</h2>
      ${body}
    </div>`;
}

/**
 * Section 2, as the page's lead: the sentence under the `h1`, so it sits above the fold on a phone
 * and before the contents list that `withContents` inserts at the first card.
 */
export function renderShortAnswer(topic: Topic): string {
  return `<p class="lead" id="${ID.shortAnswer}"><strong>${topic.shortAnswer.lead}</strong> ${topic.shortAnswer.rest}</p>`;
}

/** Sections 3 and 4: the comparison, and where it stops being true. */
export function renderPicture(topic: Topic): string {
  return (
    card(ID.picture, "The picture", paragraphs([topic.picture])) +
    card(ID.breaks, "Where the picture breaks", paragraphs(topic.breaks))
  );
}

/** Section 5: the worked example, its steps numbered, and the one chart if there is one. */
export function renderNumbers(topic: Topic): string {
  const n = topic.numbers;
  const chart = n.chart
    ? `<figure class="explained-chart">${n.chart.svg}<figcaption class="note">${n.chart.caption}</figcaption></figure>`
    : "";
  return card(
    ID.numbers,
    "The numbers",
    paragraphs(n.intro) +
      `<ol class="note">${n.steps.map((s) => `<li>${s}</li>`).join("")}</ol>` +
      chart +
      paragraphs(n.after),
    yearChip(n.chip),
  );
}

/** Section 6: the rule in notation, what each symbol is, and the statute that says it. */
export function renderRule(topic: Topic): string {
  const r = topic.rule;
  return card(
    ID.rule,
    "The rule, written out",
    `<p class="explained-rule">${r.notation}</p>` +
      `<ul class="note">${r.symbols.map((s) => `<li>${s}</li>`).join("")}</ul>` +
      `<p class="note">The rule is in <a href="${r.href}">${r.statute}</a>.</p>`,
  );
}

/** A corpus node's label, or a thrown error: a topic citing a node that does not exist is broken. */
function nodeLink(ref: string, corpus: Corpus): string {
  const node = corpus.byId.get(ref);
  if (!node) {
    throw new Error(`An Explained topic cites the corpus node \`${ref}\`, and the corpus has none`);
  }
  return `<a href="${routes.wikiNode(node.className, node.name)}">${escapeHtml(node.label)}</a>`;
}

/** Section 7: each claim with its badge, the nodes that hold it, and the figures that bind it. */
export function renderHowWeKnow(topic: Topic, corpus: Corpus = loadCorpus()): string {
  const items = topic.howWeKnow.map((proof) => {
    const nodes = proof.nodes.map((ref) => nodeLink(ref, corpus)).join(", ");
    const figures = (proof.figures ?? []).map((key) => `<code>${key}</code>`).join(", ");
    return (
      `<li>${badgeClaims(proof.claim)}` +
      `<span class="explained-proof"> In the corpus: ${nodes}.` +
      (figures ? ` Bound figures: ${figures}.` : "") +
      `</span></li>`
    );
  });
  return card(ID.howWeKnow, "How we know", `<ul class="note">${items.join("")}</ul>`);
}

/** Section 8: the next topics, and the data page where a reader's own district shows this term. */
export function renderReadNext(topic: Topic): string {
  const next = topic.readNext.topics.map((slug) => {
    const t = topicModule(slug);
    return `<li><a href="${routes.explained(t.slug)}">${t.question}</a></li>`;
  });
  const data = topic.readNext.data;
  return card(
    ID.readNext,
    "Read next",
    `<ul class="note">${next.join("")}` +
      `<li><a href="${data.href}">${data.label}</a> — ${data.note}</li></ul>`,
  );
}

/** Sections 2 to 8, in order: everything under the `h1`. */
export function renderTopic(topic: Topic): string {
  return [
    renderShortAnswer(topic),
    renderPicture(topic),
    renderNumbers(topic),
    renderRule(topic),
    renderHowWeKnow(topic),
    renderReadNext(topic),
  ].join("");
}
