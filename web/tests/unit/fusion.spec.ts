/**
 * No expression is fused to the text above it.
 *
 * # The defect
 *
 * Astro trims a newline between text and an adjacent expression, exactly as JSX does. So a
 * paragraph reflowed across lines silently loses a space:
 *
 * ```
 *     ... in the department's FY{bundle.fiscal_year} model.
 *     {count(s.on_guarantee)} are held above what the formula ...
 * ```
 *
 * renders as `model.294 are held`. It is invisible in the source, which reads correctly, and
 * visible only in the output — which is why it keeps shipping. `7a78b40` found eleven of these
 * live and fixed them; six more were live when this file was written, on `/districts` and
 * `/scenario`, including `the$812.5M` and `moves$113.0M`.
 *
 * # Why the existing scan cannot see it
 *
 * `app.spec.ts` scans the built site for a letter against an inline tag boundary —
 * `</strong>219` and `computed by<code>`. That catches the half of this defect that leaves a tag
 * behind. **This half leaves none**: `model.294` is a single text node, indistinguishable from
 * prose that meant to say that. Scanning the rendered text for a letter beside a digit finds
 * 13,441 matches across 400 pages, essentially all of them `textContent` running across a table
 * cell or a heading. There is no signal in the output.
 *
 * So this scans the source, where the defect actually is: an authoring pattern, not a rendering
 * one.
 *
 * # Why the rule is absolute
 *
 * Every expression opening a line below a line of text needs an explicit `{" "}`, whether or not
 * its own value happens to begin with a separator. That is stricter than the defect requires and
 * the strictness is the point, for two reasons.
 *
 * The first is that the exception cannot be checked. `{chamber === "senate" ? " Senate seats …"}`
 * is safe and `{millions(812_500_000).replace("+", "")}` is not, and telling them apart means
 * knowing which literal a conditional will emit — a judgment this test would get wrong in both
 * directions.
 *
 * The second is that the redundancy costs nothing. **HTML collapses whitespace**, so a `{" "}`
 * before an expression that already begins with a space renders identically. A rule with no
 * false-positive cost can afford to be blunt, and this defect has shipped often enough to deserve
 * a blunt one.
 *
 * Two things are exempt, because neither can fuse to prose: a comment, which emits nothing, and an
 * expression emitting markup, where the separation is between elements rather than inside a
 * sentence.
 *
 * # The mirror shape, which this missed for as long as it existed
 *
 * Everything above is about an expression *opening* a line. The same trimming happens when one
 * *closes* a line and prose opens the next:
 *
 * ```
 *     ... — {spans.length} formulas across {covered}
 *     fiscal years, from a duty written in 1851 ...
 * ```
 *
 * renders `across 50fiscal years`. That shipped to a built page and was found by looking at a
 * screenshot, with this file green beside it — the scan was reading one end of the seam.
 *
 * Attributes are why the mirror rule needs a qualifier rather than being the same rule reversed.
 * `image={routes.og.page("legislation")}` ends a line with `}` and the next line opens with
 * `imageAlt=`, which is a letter, and nothing is fusing: those are two attributes inside one tag.
 * The character before the expression's `{` separates the two cases exactly — an `=` means an
 * attribute value, anything else means it was sitting in a run of prose.
 */

import { mkdirSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";

import { expect, test } from "vitest";

const ROOT = new URL("../../src/", import.meta.url).pathname;

function* astroFiles(dir: string): Generator<string> {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) yield* astroFiles(path);
    else if (entry.name.endsWith(".astro")) yield path;
  }
}

/** The brace-balanced expression beginning at `from`. */
function expressionAt(body: string, from: number): string {
  let depth = 0;
  for (let i = from; i < body.length; i += 1) {
    if (body[i] === "{") depth += 1;
    else if (body[i] === "}") {
      depth -= 1;
      if (depth === 0) return body.slice(from, i + 1);
    }
  }
  return body.slice(from);
}

/** Everything after the frontmatter fence. The script half has no text runs to fuse to. */
function template(source: string): string {
  const opening = source.indexOf("---");
  if (opening === -1) return source;
  const closing = source.indexOf("\n---", opening + 3);
  return closing === -1 ? source : source.slice(closing + 4);
}

/** A line of prose ends in one of these. A line ending in `>` or `{` is markup or structure. */
const ENDS_IN_TEXT = /[A-Za-z0-9),.:;—–%]$/;

export function fusedExpressions(dir: string): string[] {
  const found: string[] = [];
  for (const file of astroFiles(dir)) {
    const body = template(readFileSync(file, "utf8"));
    const lines = body.split("\n");
    let offset = 0;
    for (let i = 0; i < lines.length; i += 1) {
      const line = lines[i]!;
      const start = offset + line.indexOf("{");
      offset += line.length + 1;

      const current = line.trim();
      if (i === 0 || !current.startsWith("{")) continue;
      if (current.startsWith("{/*") || current.startsWith('{" "}')) continue;

      const previous = lines[i - 1]!.trimEnd();
      if (!previous || !ENDS_IN_TEXT.test(previous)) continue;
      // An expression emitting elements separates blocks, not words.
      if (expressionAt(body, start).includes("<")) continue;

      found.push(
        `${relative(dir, file)}:${i + 1}\n    …${previous.slice(-52)}\n    ${current.slice(0, 52)}…`,
      );
    }
  }
  return found;
}

/**
 * The `{` that opens the balanced expression this line ends with, or `-1`.
 *
 * Deliberately gives up on an expression that opened on an earlier line: the shape it is looking
 * for is a short interpolation inside a sentence, and a multi-line expression closing a line is
 * markup or a ternary, neither of which fuses to the prose below.
 */
function trailingExpression(line: string): number {
  if (!line.endsWith("}")) return -1;
  let depth = 0;
  for (let i = line.length - 1; i >= 0; i -= 1) {
    if (line[i] === "}") depth += 1;
    else if (line[i] === "{") {
      depth -= 1;
      if (depth === 0) return i;
    }
  }
  return -1;
}

/** The other end of the same seam: an expression closing a line, prose opening the next. */
export function fusedText(dir: string): string[] {
  const found: string[] = [];
  for (const file of astroFiles(dir)) {
    const lines = template(readFileSync(file, "utf8")).split("\n");
    for (let i = 0; i < lines.length - 1; i += 1) {
      const line = lines[i]!.trimEnd();
      const open = trailingExpression(line);
      if (open === -1) continue;
      // `image={…}` and `imageAlt={…}` are two attributes in one tag, not a sentence.
      if (line[open - 1] === "=") continue;
      const expression = line.slice(open);
      // `{" "}` at the end of a line is the fix for this defect, not the defect.
      if (expression.includes("<") || /^\{\s*"\s*"\s*\}$/.test(expression)) continue;
      // The expression must be sitting in prose rather than alone on its line.
      if (!ENDS_IN_TEXT.test(line.slice(0, open).trimEnd())) continue;

      const next = lines[i + 1]!.trim();
      if (!/^[A-Za-z0-9]/.test(next)) continue;

      found.push(
        `${relative(dir, file)}:${i + 1}\n    …${line.slice(-52)}\n    ${next.slice(0, 52)}…`,
      );
    }
  }
  return found;
}

test("no expression is fused to the text above it", () => {
  const fused = fusedExpressions(ROOT);
  expect(
    fused,
    `${fused.length} expression(s) open a line directly below prose with no separator between ` +
      `them. Astro trims the newline, so the two render as one word — "model.294", "the$812.5M". ` +
      `Put an explicit {" "} at the start of the line: it costs nothing where the separation was ` +
      `already there, because HTML collapses whitespace.`,
  ).toEqual([]);
});

test("no text is fused to the expression above it", () => {
  const fused = fusedText(ROOT);
  expect(
    fused,
    `${fused.length} line(s) open with prose directly below an expression that closes the line ` +
      `above. Astro trims the newline the same way in this direction, so the two render as one ` +
      `word — "across 50fiscal years". Move the expression down onto the prose line, or end the ` +
      `line with an explicit {" "}.`,
  ).toEqual([]);
});

test("the scan sees the shapes that fuse and ignores the shapes that cannot", () => {
  /*
   * Pinned against fixtures rather than against the tree, so the rule is legible without reading
   * every template — and so a future loosening of it fails here rather than going quiet.
   */
  const dir = mkdtempSync(join(tmpdir(), "fusion-"));
  mkdirSync(join(dir, "pages"), { recursive: true });

  const write = (name: string, template: string) =>
    writeFileSync(join(dir, "pages", name), `---\nconst x = 1;\n---\n${template}\n`);

  // Fuses: prose, then an expression with nothing between.
  write("bad.astro", `<p>\n  the department's model.\n  {count(x)} are held above it.\n</p>`);
  // Does not: the separator is explicit.
  write("spaced.astro", `<p>\n  the department's model.\n  {" "}{count(x)} are held above it.\n</p>`);
  // Does not: a comment emits nothing.
  write("comment.astro", `<p>\n  the department's model.\n  {/* why */}\n</p>`);
  // Does not: an expression emitting elements separates blocks, not words.
  write("markup.astro", `<div>\n  the department's model.\n  {x > 0 && (<p>more</p>)}\n</div>`);
  // Does not: the line above ends in markup rather than prose.
  write("after-tag.astro", `<div>\n  <span>text</span>\n  {count(x)}\n</div>`);

  const found = fusedExpressions(dir).map((f) => f.split(":")[0]);
  expect(found).toEqual(["pages/bad.astro"]);
});

test("and the mirror scan tells a sentence from a tag full of attributes", () => {
  const dir = mkdtempSync(join(tmpdir(), "fusion-mirror-"));
  mkdirSync(join(dir, "pages"), { recursive: true });
  const write = (name: string, template: string) =>
    writeFileSync(join(dir, "pages", name), `---\nconst x = 1;\n---\n${template}\n`);

  // Fuses: an expression closes a line of prose and a word opens the next.
  write("bad.astro", `<p>\n  across {count(x)}\n  fiscal years, from 1851.\n</p>`);
  // Does not: the separator is explicit.
  write("spaced.astro", `<p>\n  across {count(x)}{" "}\n  fiscal years, from 1851.\n</p>`);
  // Does not: two attributes inside one tag, which is what `=` before the brace distinguishes.
  write("attrs.astro", `<Base\n  image={og(x)}\n  imageAlt={alt(x)}\n>text</Base>`);
  // Does not: the expression is alone on its line, so there is no prose to fuse to.
  write("alone.astro", `<div>\n  {count(x)}\n  districts\n</div>`);
  // Does not: an expression emitting elements separates blocks, not words.
  write("markup.astro", `<div>\n  the model {x > 0 && (<p>more</p>)}\n  districts\n</div>`);
  // Does not: the next line opens with a tag rather than a word.
  write("tag-below.astro", `<p>\n  across {count(x)}\n  <em>years</em>\n</p>`);

  const found = fusedText(dir).map((f) => f.split(":")[0]);
  expect(found).toEqual(["pages/bad.astro"]);
});
