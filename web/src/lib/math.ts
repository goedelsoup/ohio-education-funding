/**
 * LaTeX to MathML, at build time, with nothing shipped to the reader but the markup.
 *
 * # Why MathML and not KaTeX's HTML
 *
 * Measured in chromium rather than assumed. MathML Core lays out with no font shipped — fraction
 * bars draw, subscripts position, `≥ ≤ ×` render. KaTeX's HTML output positions glyphs with
 * absolute spacing computed for its own fonts, about twenty woff2 files; without them it does not
 * degrade, it breaks. The site's CSP is `default-src 'self'` with no external anything, so a CDN
 * was never available either.
 *
 * Temml is LaTeX to MathML only. As a dev dependency it ships nothing: the build emits markup and
 * there is no client runtime, no hydration, and no script for the CSP to admit.
 *
 * # Why a fence and never a dollar sign
 *
 * `remark-math` — the usual way to get LaTeX into markdown — delimits inline maths with `$…$`.
 * This corpus carries **1,233 dollar signs across 84 of its 117 nodes**, because it is about money:
 * `$8,241.61`, `$7,281,227,593.71`, `$0.96`. Half of them would open a maths span and the text
 * between two dollar amounts would be typeset as algebra. So maths is a fenced block and the `$`
 * character keeps meaning what it has always meant here.
 *
 * # Why this rejects rather than strips
 *
 * Everything here runs at build time over sixteen corpus nodes, and the producer is one library
 * called with `trust: false`. In that setting an unexpected element is a bug to look at rather
 * than something to quietly clean: stripping it would render a formula that is silently missing a
 * term, which on this site is worse than not building. So the allowlist is a three-way policy —
 * kept, dropped by name with the reason recorded, or the build stops and says which node did it.
 *
 * # Where the sanitize boundary is, and why it is not the markdown schema
 *
 * `prose.ts` records that syntax highlighting is off so `rehype-sanitize` can run the default
 * schema "with nothing widened", after finding that an earlier claim about that boundary was false
 * while the configuration said otherwise. Widening it for MathML would put every corpus node's
 * markdown inside a larger allowlist to serve sixteen formulas.
 *
 * It does not need to be. Maths is rendered *after* sanitize, from a fenced block that sanitize
 * has already escaped to text, and the markup that comes out passes through the enumerated
 * allowlist below instead. The markdown schema is untouched, and MathML gets a boundary of its own
 * that only has to describe MathML.
 */

import { DOMParser } from "linkedom";
import temml from "temml";

/**
 * The MathML Core elements this site emits, and nothing else.
 *
 * On the list because the corpus needs them, not because a spec table lists them. `menclose`,
 * `mmultiscripts`, `mstyle` and the deprecated presentation elements are deliberately absent: the
 * formulas here are named-quantity arithmetic and piecewise rules, and if one of them starts
 * needing a new element the build says so and the list gets an entry with a reason.
 */
export const ELEMENTS = new Set([
  "math",
  // `annotate: true` keeps the LaTeX source in the page, which is what makes a formula
  // copy-pasteable and recoverable. A site that shows its working should not render the working
  // into a form the reader cannot get back out.
  "semantics",
  "annotation",
  "mrow",
  "mi",
  "mn",
  "mo",
  "mtext",
  "mspace",
  "msub",
  "msup",
  "msubsup",
  /*
   * Limits above and below an operator, which is what `\sum_{k}` is in display style.
   *
   * Added deliberately, and the allowlist is why it had to be: the first indexed formula in the
   * corpus — the special-education weights, summed over six categories — failed the build with
   * "`<munder>` is not on the MathML allowlist" rather than rendering something wrong. That is the
   * second time this list has stopped a conversion at the boundary instead of after it; `mstyle`
   * was the first.
   *
   * These are MathML Core, they carry no scripting and no styling surface, and a summation with
   * its index under the sign is exactly the notation LaTeX was brought in for.
   */
  "munder",
  "mover",
  "munderover",
  "mfrac",
  "msqrt",
  "mroot",
  "mtable",
  "mtr",
  "mtd",
  "mpadded",
  "mphantom",
  // Added when the first real formula needed it, which is how every entry on this list should
  // arrive. `\dfrac` and the `aligned` environment emit it to carry `displaystyle` and
  // `scriptlevel` — the two attributes below and no others.
  "mstyle",
  // A link from a named quantity to the parameter node that defines it. `href` on a MathML element
  // is dead in MathML Core — measured, the cursor stays `auto` — so the anchor goes inside an
  // `<mtext>`, which is the one arrangement that renders.
  "a",
]);

/**
 * Attributes, per element. `href` is here and `style` is not.
 *
 * `style` would be the one attribute that turns this allowlist back into a general-purpose hole,
 * and it buys nothing: temml writes `style="display:block math;"` on every `<math>`, and the
 * `display="block"` attribute beside it already produces exactly that from the UA stylesheet —
 * measured, `getComputedStyle` reports `block math` with the inline style removed.
 */
export const ATTRIBUTES: Record<string, Set<string>> = {
  math: new Set(["xmlns", "display"]),
  annotation: new Set(["encoding"]),
  mo: new Set(["fence", "separator", "stretchy", "lspace", "rspace", "symmetric", "movablelimits", "minsize", "maxsize", "form", "largeop"]),
  mtable: new Set(["columnalign", "rowspacing", "columnspacing", "displaystyle"]),
  mtd: new Set(["columnalign", "rowspan", "columnspan"]),
  mtr: new Set(["columnalign"]),
  mspace: new Set(["width", "height", "depth"]),
  mpadded: new Set(["width", "height", "depth", "lspace", "voffset"]),
  mi: new Set(["mathvariant"]),
  mn: new Set(["mathvariant"]),
  mtext: new Set(["mathvariant"]),
  mfrac: new Set(["linethickness"]),
  // Whether a limit is set as an accent — tighter, and without the operator's own spacing.
  munder: new Set(["accentunder"]),
  mover: new Set(["accent"]),
  munderover: new Set(["accent", "accentunder"]),
  msqrt: new Set([]),
  // `mathcolor` and `mathbackground` are deliberately absent. A formula that can set its own
  // colours can hide a term inside itself, and on this site colour is solved rather than picked
  // — see `tokens/colors.css`. `mathsize` is absent for the same reason `--text-*` is a closed
  // ramp: a size nobody chose is a size nobody can hold.
  mstyle: new Set(["displaystyle", "scriptlevel"]),
  a: new Set(["href"]),
};

/**
 * Dropped by name rather than rejected, because temml always writes them and neither carries
 * meaning: `style` is redundant against the `display` attribute, and `class` is temml's own
 * `tml-*` naming which no stylesheet here reads.
 */
export const DROPPED = new Set(["style", "class"]);

/** Anything an `<a>` inside a formula may point at: this site, and nothing off it. */
const INTERNAL_HREF = /^\/[^/\\]/;

export class MathError extends Error {}

/**
 * Walk the rendered markup and hold it to the lists above.
 *
 * Exported so a test can drive it with markup temml would never produce — the boundary is only
 * worth having if something checks it in the direction that matters.
 */
export function checkAndClean(element: Element, where: string): void {
  const tag = element.tagName.toLowerCase();
  if (!ELEMENTS.has(tag)) {
    throw new MathError(`${where}: <${tag}> is not on the MathML allowlist in lib/math.ts`);
  }
  const permitted = ATTRIBUTES[tag] ?? new Set<string>();
  for (const attribute of [...element.attributes]) {
    const name = attribute.name.toLowerCase();
    if (DROPPED.has(name)) {
      element.removeAttribute(attribute.name);
      continue;
    }
    if (!permitted.has(name)) {
      throw new MathError(`${where}: <${tag} ${name}> is not on the MathML allowlist in lib/math.ts`);
    }
    if (name === "href" && !INTERNAL_HREF.test(attribute.value)) {
      throw new MathError(`${where}: a formula may only link inside this site, not "${attribute.value}"`);
    }
  }
  for (const child of [...element.children]) checkAndClean(child as Element, where);
}

const spaces = (text: string): string => text.replace(/\s+/gu, " ").trim();

/**
 * Turn the named quantities in a formula into links to the parameter nodes that define them.
 *
 * # Why the phrases are declared and not matched
 *
 * #204 proposed deriving the link from a parameter's `name` property, so that nothing is
 * hand-authored. Measured against the corpus: the fourteen `function_tex` fields write **152
 * distinct `\text{}` phrases**, and exactly **one** of them equals a parameter's `name`. A
 * formula does not write "Special education category multiples"; it writes `w_k` and, where it
 * names a quantity in words, it writes the words the section uses — "statewide average base cost
 * per pupil", "the school-age weights", "general proration".
 *
 * So the phrases are declared, on the parameter, as `written_as:`. That keeps the property #204
 * actually cares about — **zero hand-written hrefs in any `function_tex`** — while putting the
 * vocabulary beside the thing being named rather than beside each use of it. A parameter written
 * six ways declares six phrases once; `formulaLinks.spec.ts` fails if a phrase stops being used.
 *
 * # Scoped by the edge, which is what keeps it honest
 *
 * A formula only links to a parameter its component declares a `governed-by` edge to. That is why
 * `supplement` can be an alias of the performance supplement rate without linking every other
 * component's supplements: the map handed in here is built per node from that node's own edges.
 * A phrase with no edge behind it links to nothing, silently and correctly.
 *
 * Only a whole `<mtext>` is linked, never a substring. A partial match inside a named quantity
 * would put an underline through half a phrase, and the phrase is the unit of meaning.
 *
 * # The spaces are not spaces
 *
 * temml writes the gaps inside `\text{}` as U+00A0, so the words of `\text{statewide average
 * base cost per pupil}` arrive joined by non-breaking spaces rather than by U+0020 — written
 * out here rather than pasted, because the character is invisible in source and the next
 * reformat would delete the demonstration without changing a visible byte. A lookup on
 * the raw `textContent` therefore matched only the single-word phrases: five of twenty-eight,
 * and every one of the five was a word with no gap in it. Both sides are normalised here rather
 * than the corpus being asked to type a non-breaking space it cannot see.
 */
export function linkTerms(root: Element, terms: Map<string, string>, document: Document): number {
  if (terms.size === 0) return 0;
  let linked = 0;
  const walk = (element: Element): void => {
    if (element.tagName.toLowerCase() === "mtext") {
      const raw = element.textContent ?? "";
      const href = terms.get(spaces(raw).toLowerCase());
      // `element.children.length` guards against wrapping something already wrapped, which is what
      // a second pass over the same tree would otherwise do.
      if (href !== undefined && element.children.length === 0 && raw.trim() !== "") {
        const anchor = document.createElement("a");
        anchor.setAttribute("href", href);
        /* The ORIGINAL text, not the normalised one. Those gaps are U+00A0 and temml means them:
           a named quantity that starts breaking across lines inside a formula is a phrase the
           reader has to reassemble. Only the lookup is normalised. */
        anchor.textContent = raw;
        element.textContent = "";
        element.appendChild(anchor);
        linked += 1;
      }
      return;
    }
    for (const child of [...element.children]) walk(child as Element);
  };
  walk(root);
  return linked;
}

/**
 * One LaTeX expression, as MathML.
 *
 * `throwOnError` is the setting that matters and it defaults to **false**: left alone, temml
 * renders a malformed expression as a red error string, which on this site would mean shipping
 * "\\undefined" in a formula across every page that carries it rather than failing a build.
 * `trust: false` is temml's own default and is set anyway, because a default is not a decision.
 *
 * @param where names the node and field, so a failure says which formula to go and fix.
 */
export function renderMath(
  latex: string,
  options: { display: boolean; where: string; terms?: Map<string, string> },
): string {
  let markup: string;
  try {
    markup = temml.renderToString(latex, {
      displayMode: options.display,
      throwOnError: true,
      trust: false,
      annotate: true,
      xml: true,
    });
  } catch (cause) {
    throw new MathError(`${options.where}: ${(cause as Error).message}`);
  }

  const document = new DOMParser().parseFromString(markup, "text/xml");
  const root = document.querySelector("math");
  if (!root) throw new MathError(`${options.where}: temml produced no <math> element`);
  /* Before the boundary, not after: an anchor this function inserts is held to the same allowlist
     as anything temml produced, so a malformed href fails the build rather than shipping. */
  if (options.terms) {
    linkTerms(root as unknown as Element, options.terms, document as unknown as Document);
  }
  checkAndClean(root as unknown as Element, options.where);

  /*
   * Serialised with every tag closed explicitly, and that is not a style preference.
   *
   * The XML serialiser writes `<mspace width="0.1667em" />`. In HTML that is correct: inside
   * `<math>` the parser is in foreign content and honours the solidus. But every page on this site
   * is then re-serialised by `applySemantics`, which parses with linkedom — and linkedom drops the
   * solidus, so what ships is `<mspace width="0.1667em">`, an unclosed tag. `mspace` is an empty
   * element, so the browser makes the entire rest of the row its children and paints none of it.
   *
   * The formula still LOOKS like a formula. `\bigl( 0.6\,C_1 + 0.2\,C_2 + 0.2\,C_3 \bigr)
   * \times C_6` shipped as "= ( 0.6" — a plausible-looking equation missing four of its five
   * terms, and the last row of the first formula this site ever rendered. Geometry does not catch
   * it either: the row still measures 552px and reports itself inside its box.
   *
   * `<mspace …></mspace>` survives both parsers, so the pair is written out. `math.spec.ts`
   * asserts that no empty MathML element ever ends up with a child.
   */
  const element = document
    .toString()
    .replace(/^<\?xml[^?]*\?>/, "")
    .replace(/<([a-z]+)((?:\s[^<>]*?)?)\s*\/>/g, "<$1$2></$1>");

  /*
   * Wrapped, because the scroll cannot go on the `<math>` element itself.
   *
   * Measured: with `overflow-x: auto` on a `display: block math` box, Chromium reports
   * `scrollWidth === clientWidth` while the last row of an `aligned` environment is visibly cut in
   * half — the browser lays the table out against a width it then does not honour, so there is
   * nothing to scroll to and the content is simply gone. A formula that silently loses its last
   * term is the worst failure available here, because it still looks like a formula.
   *
   * The wrapper is an ordinary block, which scrolls the way every other wide thing on this site
   * does. `.scroll` is not reused: that class is swept by `semantics.ts` for boxes holding a
   * `<table>`, and a formula is not one.
   */
  return `<div class="formula">${element}</div>`;
}
