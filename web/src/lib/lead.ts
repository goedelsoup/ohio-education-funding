/**
 * Whether a node page's lead says what its description is about to say, and what to render if so.
 *
 * # Why this exists
 *
 * `summary:` was split out of `description:` (decision `corpus-four-prose-fields`), and most
 * summaries were written by rereading the description's first paragraph and saying it shorter. So
 * a reader of H.B. 110 met "The FY2022-23 operating budget, which enacted the Fair School Funding
 * Plan" in the lead and then again, word for word, as the first sentence of the body. When #551 was
 * opened, 97 of the 135 node pages opened by saying their first thing twice.
 *
 * The loader already refuses a summary that *is* the description's opening, cut (the exact-prefix
 * test in `schema.spec.ts`). Nothing caught the paraphrase, which is the common case: the lead
 * drops a clause, the body adds a claim badge and a link, and the two strings share no prefix
 * longer than a few words.
 *
 * # What "restates" means here
 *
 * Content words — lowercased, stopwords dropped, a plural `s` folded — compared sentence by
 * sentence. One text restates another when some sentence of it, of four content words or more,
 * has at least {@link RESTATES_AT} of them in the other. Taken in both directions: a lead that
 * repeats the opening's substance, or an opening whose words the lead already said.
 *
 * The threshold is set against a baseline rather than by eye. Two adjacent paragraphs of one
 * description are text its author wrote as *different* things about the same subject, so they
 * share vocabulary without restating each other; across the corpus's 566 such pairs, 2 reach 0.7
 * and 11 reach 0.6. Summary against opening, 91 of 135 reach 0.7. So 0.7 is the point above which
 * a pair is more alike than one paragraph of this corpus is to the next — and it is also why the
 * dist test can ask the same question of the rendered lead and the paragraph under it. A lower
 * threshold would catch more paraphrase (the pairs between 0.5 and 0.7 read as near-repeats too)
 * and would also start calling a description's second paragraph a repeat of its first.
 *
 * # What renders instead
 *
 * The opening, once, in the lead's place. Not the summary with the opening cut from the body: the
 * opening is the text that carries the claim badges and the links, and the summary is the one
 * derived from it — rendering the derivation and dropping its source would take verified
 * statements off the page to keep an unverified restatement of them. The summary still serves
 * every place it was written for (the `<meta>` description, the preview cards, the class index);
 * only the node page, which has the description beside it, has no need of it.
 */

/** Words that carry no content, so two sentences do not match on "the" and "of". */
const STOPWORDS = new Set(
  (
    "a all an and any are as at be been but by can did do does each every for from had has have " +
    "here in into is it its may must no not of on one or over per so than that the their them " +
    "then there they this three to two under was were what when where which who whom will with would"
  ).split(" "),
);

/** Markdown or rendered HTML, as the words a reader sees — claim badges and link targets gone. */
export function readerText(source: string): string {
  return source
    .replace(/<span class="claim[^"]*">[\s\S]*?<\/span>/g, " ")
    .replace(/<[^>]+>/g, " ")
    .replace(/&[a-z]+;|&#\d+;/g, " ")
    .replace(/\[(?:verified|unverified|inferred|contested|sourced)[^\]]*\]/gi, " ")
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/[`*_>#]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

/**
 * Sentences, split where a period follows a lowercase letter, digit or closing mark and a capital
 * follows the space — so "H.B. 110" and "R.C. 3317.022" stay whole.
 */
function sentences(text: string): string[] {
  return text.split(/(?<=[a-z0-9)\]"'’”%][.!?])\s+(?=["“]?[A-Z$])/);
}

function contentWords(text: string): Set<string> {
  return new Set(
    text
      .toLowerCase()
      .replace(/[^a-z0-9$%]+/g, " ")
      .split(" ")
      .filter((word) => word.length > 1 && !STOPWORDS.has(word))
      .map((word) => word.replace(/(?:ies|es|s)$/, "")),
  );
}

/** The largest share of any one sentence of `text`'s content words that `other` also has. */
function covered(text: string, other: Set<string>): number {
  let best = 0;
  for (const sentence of sentences(text)) {
    const words = contentWords(sentence);
    if (words.size < 4) continue;
    const shared = [...words].filter((word) => other.has(word)).length;
    best = Math.max(best, shared / words.size);
  }
  return best;
}

/** At or above this, one of the two texts is saying the other again. See the module doc. */
export const RESTATES_AT = 0.7;

/** How far either text restates the other, 0 to 1. Both are reader text, not markup. */
export function overlap(a: string, b: string): number {
  return Math.max(covered(a, contentWords(b)), covered(b, contentWords(a)));
}

/** Whether a lead and the paragraph after it say the same thing. Either may be markdown or HTML. */
export function restates(lead: string, opening: string): boolean {
  return overlap(readerText(lead), readerText(opening)) >= RESTATES_AT;
}

/** The markdown's paragraphs, as the renderer will split them. */
export function paragraphs(markdown: string): string[] {
  return markdown
    .trim()
    .split(/\n\s*\n/)
    .map((paragraph) => paragraph.trim())
    .filter((paragraph) => paragraph !== "");
}

/**
 * The lead and the body a node page renders.
 *
 * `html` is the description as `renderProse` returned it. When the summary restates the first
 * paragraph and that paragraph rendered as a `<p>`, the lead is that paragraph's inner HTML and the
 * body is everything after it; otherwise the lead is the summary, as text, and the body is whole.
 */
export function leadAndBody(
  summary: string,
  markdown: string,
  html: string,
): { lead: { html: string } | { text: string }; body: string } {
  const opening = paragraphs(markdown)[0] ?? "";
  const first = html.match(/^\s*<p>([\s\S]*?)<\/p>/);
  if (!first || !restates(summary, opening)) return { lead: { text: summary }, body: html };
  return { lead: { html: first[1]!.trim() }, body: html.slice(first[0].length).trim() };
}
