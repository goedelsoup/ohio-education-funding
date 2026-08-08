/**
 * Corpus prose, rendered for the web.
 *
 * Node descriptions are markdown embedded in YAML, written to be read in an editor: they link by
 * relative file path and they carry the corpus's epistemic tags inline. Two transformations turn
 * that into a page, and both matter more than they look.
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
 * `[verified]`, `[inference]` and `[open]` mark every substantive claim in this corpus, often
 * with a justification attached: `[verified — computed; see …]`. This is the discipline that makes
 * the corpus worth reading, and it is also the thing most easily lost in translation — markdown
 * leaves `[verified]` as literal brackets, which reads as a typo rather than as an assertion
 * about evidence. They become badges instead.
 *
 * The substitution runs on the rendered HTML and is deliberately conservative: a tag whose
 * justification contains markup is left as text rather than risk splitting an element across a
 * badge boundary. Losing a badge is a cosmetic failure; emitting broken HTML from a document
 * whose whole purpose is to be trusted is not.
 */

import { createMarkdownProcessor } from "@astrojs/markdown-remark";

import { resolveTarget } from "./corpus.ts";
import { escapeHtml } from "./format.ts";

/** The three epistemic states the corpus tags claims with. */
const TAGS = ["verified", "inference", "open"] as const;

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

/**
 * Turn the corpus's inline claim tags into badges.
 *
 * Two forms: a bare `[verified]`, and a justified `[verified — computed; see [`crates/dispersion`
 * ](…)]`. The justification is frequently a citation, so by the time this runs it is usually an
 * `<a>` element — which is why the detail pattern admits markup but never a square bracket. The
 * markdown link inside it has already become an anchor, so the first unmatched `]` is reliably the
 * tag's own closing bracket.
 *
 * The detail is interpolated unescaped because it *is* rendered HTML at this point. Escaping it
 * would print the anchor's tags to the reader.
 */
export function badgeClaims(html: string): string {
  const alternatives = TAGS.join("|");
  return html
    .replace(
      new RegExp(`\\[(${alternatives})\\s+—\\s+([^\\[\\]]+)\\]`, "g"),
      (_whole, tag: string, detail: string) =>
        `<span class="claim ${tag}">${tag}</span><span class="claim-detail"> ${detail.trim()}</span>`,
    )
    .replace(
      new RegExp(`\\[(${alternatives})\\]`, "g"),
      (_whole, tag: string) => `<span class="claim ${tag}">${tag}</span>`,
    );
}

/**
 * Render a corpus markdown string to HTML.
 *
 * Async because the markdown processor is. Astro components can await in their frontmatter, so
 * this costs nothing at the call site.
 */
export async function renderProse(markdown: string, fromClass: string): Promise<string> {
  processor ??= await createMarkdownProcessor({
    // The corpus is authored in this repository by people who can be trusted with it, but the
    // rendering is still not given a licence to emit script: `gfm` for tables and strikethrough,
    // and nothing that would let a node inject markup into a page.
    gfm: true,
    smartypants: true,
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
 */
export function renderPropertyValue(value: string, fromClass: string): string {
  const escaped = escapeHtml(value)
    // Inline links, which several properties carry.
    .replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_whole, label: string, target: string) => {
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
