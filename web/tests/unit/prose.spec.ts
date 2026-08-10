/**
 * Rendering corpus prose: the claim tags, the summaries, and the sanitizer.
 *
 * # Why these three
 *
 * Each pins a failure that shipped and was invisible in review, because each produced output that
 * is wrong without looking wrong. A claim tag that does not match its pattern prints as literal
 * brackets, which reads as a typo. A summary that deletes a link along with its label produces a
 * grammatical sentence with the subject missing — "seeded as the contrast case against ." — that
 * only a reader who knows the source can spot. And a comment claiming the renderer cannot emit
 * script is not a thing anyone re-checks.
 *
 * The cases below are taken from the corpus rather than invented; the counts in the comments are
 * occurrences at the time of writing.
 */

import { expect, test } from "vitest";

import { loadCorpus } from "../../src/lib/corpus.ts";
import { badgeClaims, renderProse, renderPropertyValue, summarize } from "../../src/lib/prose.ts";

const corpus = loadCorpus();

/** Badges collapsed to `‹tag›`, details to `{…}`, so the assertions read as prose. */
const shape = (html: string) =>
  html
    .replace(/<span class="claim ([a-z]+)">\1<\/span>/g, "‹$1›")
    .replace(/<span class="claim-detail"> /g, "{")
    .replace(/<\/span>/g, "}");

test("every form the corpus writes a claim tag in becomes a badge", () => {
  // Bare, and the three qualifier separators. The pattern this replaced admitted only the spaced
  // em dash, so `[verified as proposed]` — 26 occurrences, the commonest qualified form — and
  // every comma-separated one printed as brackets.
  expect(shape(badgeClaims("[verified]"))).toBe("‹verified›");
  expect(shape(badgeClaims("[unentered]"))).toBe("‹unentered›");
  expect(shape(badgeClaims("[verified — LSC]"))).toBe("‹verified›{LSC}");
  expect(shape(badgeClaims("[inference, Fordham]"))).toBe("‹inference›{Fordham}");
  expect(shape(badgeClaims("[verified as proposed]"))).toBe("‹verified›{as proposed}");
  expect(shape(badgeClaims("[verified for FY2022; inference for FY2023]"))).toBe(
    "‹verified›{for FY2022; inference for FY2023}",
  );
});

test("a claim tag inside a claim tag badges both, outermost first", () => {
  // The old pattern found the *inner* tag and badged it in the middle of a sentence, leaving the
  // outer brackets as literal punctuation around it.
  expect(shape(badgeClaims("[verified, and its intent is [open]]"))).toBe(
    "‹verified›{and its intent is ‹open›}",
  );
  expect(shape(badgeClaims("[inference on the mechanism, [verified] on the measurement]"))).toBe(
    "‹inference›{on the mechanism, ‹verified› on the measurement}",
  );
});

test("a bracket that is not a claim is left alone", () => {
  // The corpus brackets link labels, statute names, and report-card line letters. Only the four
  // marks are tags, and only when the bracket opens with one.
  const text = "Reported as lines [L] and [M]; see [H.B. 920] and [DeRolph I].";
  expect(badgeClaims(text)).toBe(text);
  expect(badgeClaims("[opening the door]")).toBe("[opening the door]");
});

test("a claim tag whose justification is a link keeps both the badge and the link", () => {
  // `.yidam/corpus/legislation/hb-33-2023.yml`. The label pattern used to run across the inner
  // `[`, so the anchor text captured "verified — [the department's commission page" and a bare
  // `]` was left dangling after the element.
  const html = renderPropertyValue(
    "[verified — [the department's page](../../catalog/dew-academic-distress-commission.md)]",
    "legislation",
  );
  expect(html).toContain('<a href="/wiki/source/dew-academic-distress-commission">');
  expect(html).toContain('class="claim verified"');
  expect(html).not.toMatch(/<\/a>\]/);
  expect(html).not.toContain("verified — [");
});

test("no claim tag survives as literal brackets anywhere in the corpus", () => {
  // The property values are the ones that regressed: they do not go through the markdown
  // processor, so nothing else would have noticed.
  const literal = /\[(verified|inference|open|unentered)\b/;
  const leaked: string[] = [];
  for (const node of corpus.nodes) {
    for (const property of node.properties) {
      const html = renderPropertyValue(property.value, node.className);
      if (literal.test(html)) leaked.push(`${node.id}#${property.name}`);
    }
  }
  expect(leaked, "property values still printing a claim tag as brackets").toEqual([]);
});

test("summarize keeps a link's label and drops only its target", () => {
  // `/wiki/education-agency` shipped "seeded as the contrast case against ." and "the suburban
  // counterpart to  eleven miles away across the Maumee" — the label was deleted with the target.
  expect(summarize("the counterpart to [Toledo](toledo-city.yml) eleven miles away", 200)).toBe(
    "the counterpart to Toledo eleven miles away",
  );
});

test("summarize strips the markup a meta description cannot show", () => {
  expect(summarize("*DeRolph v. State* found the **system** unconstitutional. [verified]", 200)).toBe(
    "DeRolph v. State found the system unconstitutional.",
  );
  expect(summarize("computed in `crates/bundle` from the report card", 200)).toBe(
    "computed in crates/bundle from the report card",
  );
  // A tag whose justification is itself a link: the link is flattened to its label first, so the
  // tag is then a plain bracket pair and goes whole.
  expect(summarize("Base cost. [verified — [the page](../../catalog/x.md)]", 200)).toBe("Base cost.");
});

test("summarize cuts on a word boundary and only claims an elision when there was one", () => {
  expect(summarize("a short one", 200)).toBe("a short one");
  expect(summarize("a short one", 200).endsWith("…")).toBe(false);

  const long = "The Fair School Funding Plan sets a base cost per pupil computed from inputs.";
  const cut = summarize(long, 40);
  expect(cut.endsWith("…")).toBe(true);
  expect(long.startsWith(cut.slice(0, -1))).toBe(true);
  // The last kept token is a whole word, not a fragment of the next one.
  expect(long[cut.length - 1]).toMatch(/[\s.]/);
});

test("every corpus summary is clean markdown-free prose", () => {
  const dirty: string[] = [];
  for (const node of corpus.nodes) {
    const summary = summarize(node.description, 200);
    if (/[*`]|\[(verified|inference|open|unentered)\b|\]\(|\s\s/.test(summary)) {
      dirty.push(`${node.id}: ${summary}`);
    }
  }
  expect(dirty, "meta descriptions still carrying markup").toEqual([]);
});

test("the renderer cannot emit markup a corpus node wrote", async () => {
  // The comment that used to sit in prose.ts said the configuration allowed "nothing that would
  // let a node inject markup into a page", while `createMarkdownProcessor` was setting
  // `allowDangerousHtml` and installing rehypeRaw with no sanitize step. It does now.
  for (const attack of [
    "<script>alert(1)</script>",
    '<img src=x onerror="alert(1)">',
    '<div onclick="steal()">hi</div>',
    '<iframe src="//evil"></iframe>',
  ]) {
    const html = await renderProse(attack, "metric");
    expect(html, attack).not.toMatch(/<(script|img|iframe|div)\b/);
    expect(html, attack).not.toMatch(/on(error|click)=/);
  }
  // `javascript:` is dropped from an href while the visible text survives.
  const link = await renderProse('<a href="javascript:alert(1)">click</a>', "metric");
  expect(link).not.toContain("javascript:");
  expect(link).toContain("click");
});

test("sanitizing costs the corpus nothing it actually renders", async () => {
  // The reason the schema needs no widening: the corpus writes no raw HTML and no fenced code
  // block, so the default GitHub schema is a superset of what it uses. If that stops being true
  // this fails rather than silently dropping an element from a page.
  const marked = await renderProse(
    "A **bold** claim, *emphasis*, `code`, a [link](../metric/enrolled-adm.yml), and:\n\n" +
      "| a | b |\n| --- | --- |\n| 1 | 2 |\n\n    an indented block\n",
    "metric",
  );
  for (const fragment of ["<strong>", "<em>", "<code>", "<table>", "<td>", "<pre>", "<a href="]) {
    expect(marked, fragment).toContain(fragment);
  }
});
