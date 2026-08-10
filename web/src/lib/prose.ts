/**
 * Corpus prose, rendered for the web.
 *
 * Node descriptions are markdown embedded in YAML, written to be read in an editor: they link by
 * relative file path and they carry the corpus's epistemic tags inline. Three transformations turn
 * that into a page, and all three matter more than they look.
 *
 * # Rewriting the links
 *
 * `[H.B. 920](../legislation/hb-920-1976.yml)` has to become a route. Done before the markdown
 * processor runs, on the source text, so the processor sees ordinary links and every downstream
 * feature — reference links, titles, nested emphasis — keeps working. Rewriting the rendered HTML
 * instead would mean parsing HTML with regular expressions to find `href`s.
 *
 * # Keeping the claim tags
 *
 * `[verified]`, `[inference]`, `[open]` and `[unentered]` mark every substantive claim in this
 * corpus. This is the discipline that makes the corpus worth reading, and it is also the thing
 * most easily lost in translation — markdown leaves `[verified]` as literal brackets, which reads
 * as a typo rather than as an assertion about evidence. They become badges instead.
 *
 * The forms are more varied than a first look suggests, and a pattern that admits only the
 * commonest one is worse than no pattern: it badges some claims and prints others as punctuation,
 * so the reader cannot tell whether a bare bracket means "not a claim" or "not matched". All of
 * these occur, and {@link badgeClaims} handles all of them:
 *
 * ```
 * [verified]                                     bare
 * [verified — LSC]                               em dash
 * [inference, Fordham]                           comma
 * [verified as proposed]                         no punctuation at all — 13 occurrences
 * [verified for FY2022; inference for FY2023]    two claims in one tag
 * [verified — [`ode-idea-part-b-allocations`](…)] justification containing a link
 * [verified, and its intent is [open]]           a tag inside a tag
 * ```
 *
 * The last two are why this is a bracket scanner rather than a regular expression. A regex cannot
 * match balanced brackets, and the one that used to be here ran `[^\[\]]+` across a nested opening
 * bracket — so a justification containing a link was swallowed into the anchor's text and left a
 * stray `]` behind. Nesting is shallow and rare, but it is exactly where a naive pattern produces
 * markup that is silently wrong rather than obviously wrong.
 *
 * # Not emitting markup a node controls
 *
 * The processor is given `rehype-sanitize`, so a description containing `<script>` renders as
 * nothing rather than as a script. The corpus is authored in this repository by people who can be
 * trusted with it, and this is still not left to that: the comment that used to sit here claimed
 * the configuration allowed "nothing that would let a node inject markup into a page" while
 * `createMarkdownProcessor` was setting `allowDangerousHtml` and installing `rehypeRaw` with no
 * sanitize step. A boundary that is documented but absent is worse than one that is neither,
 * because the next author builds on it.
 *
 * Syntax highlighting is off, which is what lets the sanitize schema be the default one with
 * nothing widened. Nothing is lost: every code block in the corpus is an ASCII table with no
 * language, so Shiki contributed only a hard-coded `github-dark` background that overrode the
 * site's own `--surface-2` and ignored the reader's light or dark theme.
 */

import { createMarkdownProcessor } from "@astrojs/markdown-remark";
import rehypeSanitize from "rehype-sanitize";

import { resolveTarget } from "./corpus.ts";
import { escapeHtml } from "./format.ts";

/**
 * The four epistemic marks the corpus writes.
 *
 * `unentered` is the one that is easy to leave out, and was: `.yidam/corpus/README.md` gives it a
 * section of its own explaining that it is *not* a fourth confidence level — it says nothing has
 * been entered for a field yet, where `open` says a question has been asked and not answered. It
 * is used 76 times, and every one of them printed as literal brackets.
 */
const TAGS = ["verified", "inference", "open", "unentered"] as const;

/**
 * What may follow the tag name inside the brackets and still leave it a tag.
 *
 * End of the bracket, or a separator. The separator set is drawn from what the corpus writes
 * rather than chosen: em and en dash, comma, semicolon, colon, or nothing but a space.
 */
const SEPARATOR = /^[\s,;:—–]/;

let processor: Awaited<ReturnType<typeof createMarkdownProcessor>> | null = null;

/**
 * Rewrite every relative link target in a markdown source to a URL on this site.
 *
 * Skips anything already absolute — the corpus occasionally cites a statute or a publisher
 * directly, and those are not ours to rewrite.
 */
export function rewriteLinks(markdown: string, fromClass: string): string {
  return markdown.replace(/\]\(([^)\s]+)\)/g, (whole, target: string) => {
    if (/^(https?:|mailto:|#|\/)/.test(target)) return whole;
    return `](${resolveTarget(target, fromClass).href})`;
  });
}

/** The index of the `]` closing the `[` at `open`, or -1. Counts nesting. */
function closingBracket(text: string, open: number): number {
  let depth = 0;
  for (let index = open; index < text.length; index += 1) {
    const char = text[index];
    if (char === "[") depth += 1;
    else if (char === "]") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return -1;
}

/** The claim tag this bracket's contents open with, if they open with one. */
function leadingTag(inner: string): (typeof TAGS)[number] | null {
  for (const tag of TAGS) {
    if (!inner.startsWith(tag)) continue;
    const rest = inner.slice(tag.length);
    if (rest === "" || SEPARATOR.test(rest)) return tag;
  }
  return null;
}

/**
 * Turn the corpus's inline claim tags into badges.
 *
 * Runs on rendered HTML, so by the time it sees a justification the markdown links inside it are
 * already `<a>` elements. The detail is interpolated unescaped because it *is* rendered HTML at
 * this point; escaping it would print the anchor's tags to the reader.
 *
 * Outermost bracket first, and the detail is then run through this again — so
 * `[verified, and its intent is [open]]` badges the outer tag and the inner one, in that order,
 * rather than badging the inner one inside a pair of literal brackets. A bracket that does not
 * open with a tag is left alone and scanned into, because the corpus brackets plenty of things
 * that are not claims and some of them contain claims.
 */
export function badgeClaims(html: string): string {
  let out = "";
  let index = 0;

  while (index < html.length) {
    const open = html.indexOf("[", index);
    if (open === -1) {
      out += html.slice(index);
      return out;
    }
    out += html.slice(index, open);

    const close = closingBracket(html, open);
    const inner = close === -1 ? "" : html.slice(open + 1, close);
    const tag = close === -1 ? null : leadingTag(inner);
    if (!tag) {
      // Not a claim. Emit the bracket and carry on from just after it, so anything nested inside
      // still gets its turn.
      out += "[";
      index = open + 1;
      continue;
    }

    const detail = inner.slice(tag.length).replace(/^\s*[—–,;:]?\s*/, "");
    out += `<span class="claim ${tag}">${tag}</span>`;
    if (detail !== "") out += `<span class="claim-detail"> ${badgeClaims(detail)}</span>`;
    index = close + 1;
  }

  return out;
}

/**
 * Render a corpus markdown string to HTML.
 *
 * Async because the markdown processor is. Astro components can await in their frontmatter, so
 * this costs nothing at the call site.
 */
export async function renderProse(markdown: string, fromClass: string): Promise<string> {
  processor ??= await createMarkdownProcessor({
    gfm: true,
    smartypants: true,
    // Off so the sanitize schema below can be the default one with nothing widened; see the
    // module docstring. Every code block in this corpus is an unlabelled ASCII table.
    syntaxHighlight: false,
    rehypePlugins: [rehypeSanitize],
  });
  const { code } = await processor.render(rewriteLinks(markdown, fromClass));
  return badgeClaims(code);
}

/**
 * Render one property value: a short string that is usually a claim with a tag on the end.
 *
 * Not put through the markdown processor — these are values in a table, and wrapping each in a
 * `<p>` would fight the layout. Escaped, then given the same badges and inline links the prose
 * gets, so a property reads consistently with the description above it.
 *
 * Because it escapes rather than sanitizes, this never renders markup a node wrote; the two paths
 * reach the same guarantee by different means.
 */
export function renderPropertyValue(value: string, fromClass: string): string {
  const escaped = escapeHtml(value)
    // Inline links, which several properties carry. The label may not contain a bracket: a
    // property that writes `[verified — [the department's page](…)]` nests one link inside one
    // claim tag, and a label pattern that ran across the inner `[` captured "verified — [the
    // department's page" as the anchor text and left the `]` dangling after it.
    .replace(/\[([^[\]]+)\]\(([^)\s]+)\)/g, (_whole, label: string, target: string) => {
      const href = /^(https?:|\/)/.test(target) ? target : resolveTarget(target, fromClass).href;
      return `<a href="${href}">${label}</a>`;
    })
    // Backticks, which they carry more often.
    .replace(/`([^`]+)`/g, "<code>$1</code>");
  return (
    badgeClaims(escaped)
      // A blank line is a deliberate break; a single newline is where the author wrapped the YAML
      // block scalar at column 95 and means nothing. Preserving both as `<br>` put line breaks
      // mid-sentence in every multi-line property on the site.
      .split(/\n\s*\n/)
      .map((paragraph) => paragraph.replace(/\s*\n\s*/g, " ").trim())
      .filter((paragraph) => paragraph !== "")
      .join("<br><br>")
  );
}

/**
 * Reduce corpus markdown to a plain sentence or two, for a `<meta>` description or a table cell.
 *
 * # Why this is one function
 *
 * There were four of these, one per wiki page, each hand-rolled and each fixing a different
 * subset of the same problem — and the union of what they missed shipped. `/wiki/education-agency`
 * printed "seeded as the contrast case against ." because a link was deleted along with its
 * visible label; `*DeRolph v. State*` kept its asterisks; 110 of 143 `<meta name="description">`
 * values were cut mid-word ("It is se", "R"), 25 carried `**`, and both truncating call sites
 * appended an ellipsis whether or not anything had been elided. These are the strings a search
 * result and a link-preview card show for every corpus page, which is to say they are the first
 * prose most readers see.
 *
 * The order below matters: links lose their targets before claim tags are removed, so a tag whose
 * justification *is* a link — `[verified — [the department's page](…)]` — has become
 * `[verified — the department's page]` and is then removed whole.
 */
export function summarize(markdown: string, max: number): string {
  let text = markdown
    // A link keeps its label and loses its target. Deleting the label with it is what produced
    // "the suburban counterpart to  eleven miles away across the Maumee".
    .replace(/\[([^[\]]*)\]\([^)\s]*\)/g, "$1")
    // Block markers, which say nothing once the line breaks are gone.
    .replace(/^\s{0,3}#{1,6}\s+/gm, "")
    .replace(/^\s{0,3}>\s?/gm, "")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/\*\*([^*]+)\*\*/g, "$1")
    .replace(/\*([^*]+)\*/g, "$1");

  // Claim tags, innermost first, until none is left. A summary has no room to explain what
  // "[verified]" asserts, and an unexplained bracket reads as a typo.
  const claim = new RegExp(`\\[(?:${TAGS.join("|")})(?:[\\s,;:—–][^[\\]]*)?\\]`, "g");
  for (let previous = ""; previous !== text; ) {
    previous = text;
    text = text.replace(claim, "");
  }

  text = text.replace(/\s+/g, " ").trim();
  if (text.length <= max) return text;

  // Cut on a word boundary, and only claim an elision when there was one.
  const cut = text.slice(0, max);
  const space = cut.lastIndexOf(" ");
  const kept = space > max * 0.6 ? cut.slice(0, space) : cut;
  return `${kept.replace(/[\s,;:.—–-]+$/, "")}…`;
}
