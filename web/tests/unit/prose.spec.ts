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

import { readdirSync, readFileSync } from "node:fs";
import { join, relative, sep } from "node:path";

import { describe, expect, test } from "vitest";

import { countCorrections, FROM_DECISION, loadCorpus, resolveTarget } from "../../src/lib/corpus.ts";
import { escapeHtml } from "../../src/lib/format.ts";
import {
  badgeClaims,
  markCorrections,
  renderProse,
  isBlockProperty,
  isListProperty,
  renderListProperty,
  renderPropertyValue,
  summarize,
} from "../../src/lib/prose.ts";

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
  // `unentered` is deliberately NOT a badge any more. It was a fourth inline mark on a three-mark
  // axis; it is now `unfilled:` structure on the node, rendered as a block where the content would
  // be. If this ever badges again, the migration has been undone by a one-word edit to TAGS.
  expect(shape(badgeClaims("[unentered]"))).toBe("[unentered]");
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

/**
 * The fourth mark is gone from the corpus, and this is what keeps it gone.
 *
 * `[unentered]` was an inline badge on a three-mark support axis for a long time, while the corpus
 * README argued in its own prose that it belongs on a different axis. It is now `unfilled:`
 * structure — a block naming the missing fact, in the position the content would occupy.
 *
 * Two ways it could come back and both fail here: a node writing the old mark, or the migration
 * being undone by re-adding the name to `TAGS`. The first is caught below; the second is caught by
 * the badge assertion above, which now expects literal brackets.
 */
test("no node writes the retired fourth mark, and the structure replaced it", () => {
  const writers = corpus.nodes
    .filter((node) =>
      [node.description, node.findings ?? "", ...node.properties.map((p) => p.value)]
        .join("\n")
        .includes("[unentered"),
    )
    .map((node) => node.id);
  expect(writers).toEqual([]);

  const carrying = corpus.nodes.filter((node) => node.unfilled.length > 0);
  expect(carrying.length, "the migration moved 29 marks onto 18 nodes").toBeGreaterThan(15);
  for (const node of carrying) {
    for (const entry of node.unfilled) {
      expect(entry.field.trim(), `${node.id} has an unfilled entry with no field`).not.toBe("");
      // A bare field name is a shrug. Every one of these came from prose that said where the
      // value lives, and that is the half worth keeping.
      expect(entry.why, `${node.id}: "${entry.field}" says nothing about where the value is`)
        .not.toBeNull();
    }
  }
});

/**
 * The same rule, over the prose *beside* the nodes — and the distinction that makes it checkable.
 *
 * The guard above reads nodes. It was written when the migration was about nodes, and five marks
 * survived outside its reach: three in class READMEs, one in an ontology `description:` that
 * renders on `/wiki/draft-legislation`, and one in `skills/parameter-history.md` telling whoever
 * writes the next parameter node to use the mark. The last is the one that mattered — a retired
 * convention in an instruction regenerates itself.
 *
 * # Record or instruction, not code or prose
 *
 * The obvious rule is the one {@link badgeClaims} uses: a tag inside backticks is the author
 * *naming* a mark rather than asserting one. It is the wrong rule here, and the counts say so.
 * Of the thirteen surviving mentions, eleven sat inside backticks — and two of those eleven were
 * the ontology description and the skill, which are as live as any node.
 *
 * What separates them is what the file is FOR. A decision record and the corpus README's own
 * account of the migration are history: they exist to say that this mark was added, argued with
 * and withdrawn, and rewriting them to have always been right is the one thing
 * `.yidam/corpus/README.md` says the corpus never does. Everything else — a class README, an
 * ontology, a skill, a node — answers "how do I write this today", and there the answer is that
 * the mark does not exist.
 *
 * So records may name it, inside code, where `badgeClaims` will not assert it. Instructions may
 * not name it at all.
 */
test("the retired mark survives only where the corpus is recording that it retired it", () => {
  const YIDAM = join(import.meta.dirname, "../../../.yidam");
  const files = (dir: string): string[] =>
    readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
      entry.isDirectory()
        ? files(join(dir, entry.name))
        : /\.(md|yml)$/.test(entry.name)
          ? [join(dir, entry.name)]
          : [],
    );

  /** History. It says the mark existed, which is true, and it is not rewritten. */
  const isRecord = (path: string) =>
    path.includes(`${sep}decisions${sep}`) || path.endsWith(join("corpus", "README.md"));
  const withoutCode = (text: string) =>
    text.replace(/```[\s\S]*?```/g, "").replace(/`[^`\n]*`/g, "");

  const asserted: string[] = [];
  const taught: string[] = [];
  for (const path of files(YIDAM)) {
    const text = readFileSync(path, "utf8");
    if (!text.includes("[unentered")) continue;
    const where = relative(YIDAM, path);
    for (const [n, line] of text.split("\n").entries()) {
      if (!line.includes("[unentered")) continue;
      if (!isRecord(path)) taught.push(`${where}:${n + 1}`);
      else if (withoutCode(line).includes("[unentered")) asserted.push(`${where}:${n + 1}`);
    }
  }

  expect(taught, "a file that says how to write a node today must not name the retired mark")
    .toEqual([]);
  expect(asserted, "a record may name the mark, inside code, where nothing will badge it")
    .toEqual([]);
});

test("no claim tag survives as literal brackets anywhere in the corpus", () => {
  // The property values are the ones that regressed: they do not go through the markdown
  // processor, so nothing else would have noticed.
  //
  // Code is stripped before the check rather than counted as a leak. A tag inside backticks is
  // the author *naming* a mark rather than making a claim under it — `edchoice-expansion`'s
  // `mechanism` says "So the earlier `[open]` here … is closed", referring back to a state this
  // field used to be in — and badging that would assert the field is open when the sentence's
  // whole point is that it no longer is. `badgeClaims` leaves code alone for that reason, so the
  // brackets are supposed to be there and this is the one place they may be.
  const literal = /\[(verified|inference|open|unentered)\b/;
  const withoutCode = (html: string) =>
    html.replace(/<pre\b[^>]*>[\s\S]*?<\/pre>|<code\b[^>]*>[\s\S]*?<\/code>/gi, "");
  const leaked: string[] = [];
  for (const node of corpus.nodes) {
    for (const property of node.properties) {
      const html = renderPropertyValue(property.value, node.className);
      if (literal.test(withoutCode(html))) leaked.push(`${node.id}#${property.name}`);
    }
  }
  expect(leaked, "property values still printing a claim tag as brackets").toEqual([]);
});

test("a claim tag inside code is quoted rather than asserted", async () => {
  /*
   * The defect: `badgeClaims` had no code exclusion, so any `[verified]` inside a `<pre><code>`
   * or a `<code>` was replaced by a styled `<span>`.
   *
   * Two things go wrong with that, and they are different failures. In a fenced block — every one
   * in this corpus is an unlabelled ASCII table — the badge is not five characters wide, so the
   * columns stop lining up and the table's only content is its alignment. Inline, the badge makes
   * an assertion the sentence did not: `draft-legislation.ont.yml` describes a property as
   * "`[unentered]` where it was not introduced", which is documentation of the vocabulary, and it
   * rendered on `/wiki/draft-legislation` as a live claim badge.
   */
  const table = await renderProse(
    "Prose with [verified] in it.\n\n```\n| field | mark        |\n| adm   | [verified]  |\n```\n",
    "metric",
  );
  expect(table).toContain('<span class="claim verified">verified</span>');
  expect(table).toContain("| adm   | [verified]  |");

  const inline = await renderProse("`[unentered]` where it was not introduced. [open]", "metric");
  expect(inline).toContain("<code>[unentered]</code>");
  expect(shape(inline)).toContain("‹open›");

  /*
   * And the half that makes this an exclusion rather than a split: a claim's justification may
   * contain code, so the `]` closing a tag opened in prose is often on the far side of a `<code>`
   * element. A scanner that treated code as a boundary rather than as opaque would emit that `[`
   * literally and swallow the rest — which is the shape the module docstring already lists as
   * real, `[verified — [`ode-idea-part-b-allocations`](…)]`.
   */
  expect(shape(badgeClaims("[verified — <a href='/x'><code>ode-idea</code></a>]"))).toBe(
    "‹verified›{<a href='/x'><code>ode-idea</code></a>}",
  );
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

/*
 * Corrections: the blockquotes a decision record uses to withdraw something above them.
 *
 * There are two implementations of one rule and there have to be. `countCorrections` reads the
 * markdown, because `loadCorpus` is synchronous and needs the count before anything is rendered;
 * `markCorrections` reads the HTML, because that is where the class has to be applied. Nothing
 * makes them agree except the test below, and they have already disagreed once — the source-side
 * rule counted five corrections in a record that has four, because a continuation line inside one
 * of them begins with a bolded word.
 */

test("a correction is a blockquote that opens with strong emphasis, and a quotation is not", () => {
  // Opens. The bolded word mid-quote is the case that broke the first version of this rule.
  expect(countCorrections("> **CORRECTED by x.** Two things above are wrong.")).toBe(1);
  expect(
    countCorrections("> West Geauga gains 208 pupils while Chardon\n> **loses** 110. The claim"),
  ).toBe(0);
  // A quotation, which is what most blockquotes in these records are.
  expect(
    countCorrections("> the count going from 124 in FY2012 to 0 in FY2022 *is* the history"),
  ).toBe(0);
  // Including one that emphasises something, as long as it does not open with it.
  expect(countCorrections("> a quotation that ends **emphatically**")).toBe(0);
  // Two separate blockquotes, one of each.
  expect(countCorrections("> a quotation\n\n> **This rejection has expired.** Because")).toBe(1);
});

/*
 * Twenty seconds rather than Vitest's five. This is the only test in the suite whose cost grows
 * with the corpus — it renders every section of every decision record through the full markdown
 * pipeline, twice over, and there are 34 records now against the 20 there were when it was
 * written. It takes about three seconds on an idle machine and has twice exceeded five on a loaded
 * one, which reads in CI as a corpus defect and is not one.
 *
 * The budget is generous on purpose: a timeout tuned to just above the current cost would have to
 * be raised again on the next decision record, and a test that needs re-tuning to keep passing
 * teaches people to raise the number rather than to ask why it moved.
 */
test("both readings of a correction agree, on every record the corpus holds", { timeout: 20_000 }, async () => {
  const disagree: string[] = [];
  for (const decision of corpus.decisions) {
    let rendered = 0;
    for (const section of [{ body: decision.summary }, ...decision.sections]) {
      if (section.body === "") continue;
      const html = await renderProse(section.body, FROM_DECISION);
      rendered += markCorrections(html).corrections;
    }
    if (rendered !== decision.corrections) {
      disagree.push(`${decision.slug}: source says ${decision.corrections}, HTML says ${rendered}`);
    }
  }
  expect(disagree, "the two readings of the correction rule have drifted apart").toEqual([]);
});

test("marking a correction leaves an anchor and does not touch a quotation", async () => {
  const html = await renderProse(
    "> a quotation of something superseded\n\n> **CORRECTED by x.** The claim above is wrong.\n",
    FROM_DECISION,
  );
  const { html: marked, corrections } = markCorrections(html);
  expect(corrections).toBe(1);
  expect(marked).toContain('<blockquote class="correction" id="correction-1">');
  // The quotation keeps the bare tag, so the stylesheet can treat the two differently.
  expect(marked).toContain("<blockquote>\n<p>a quotation");
});

/**
 * The alignment that was thrown away.
 *
 * `renderPropertyValue` joins single newlines with a space, and the comment above it is right
 * about why: a newline in a YAML block scalar is usually where the author wrapped at column 95 and
 * means nothing, and rendering those as `<br>` put line breaks mid-sentence on every multi-line
 * property on the site.
 *
 * It is right for 544 of the corpus's 601 multi-line paragraphs and it destroyed the other 57.
 * `fsfp-local-capacity-measure`'s `function` — the site's statement of how local capacity computes
 * — shipped as "…if C4 >= C5 then 0.025 if C4 > 1 and C4 < C5 then ((C4-1) x 0.0025)/(C5-1) +
 * 0.0225 if C4 <= 1 then C4 x 0.0225": three piecewise branches with nothing between them.
 */
describe("a property value whose columns carry meaning", () => {
  const BLOCK = /<pre class="aligned">/;

  test("is recognised by a column run, or by a continuation indent, and by nothing else", () => {
    // Padding somebody typed to line a column up. Prose does not contain this on purpose.
    expect(isBlockProperty("C1 Assessed Valuation  = V1 / ADM\nC2 Gross Income        = I1 / ADM")).toBe(true);
    // A hanging indent under a numbered item, a bullet, or an expression broken across lines.
    expect(isBlockProperty("[K] = max( [L1] FY21 Funding Base\n           - ( [H] Foundation Funding ),\n           0 )")).toBe(true);
    // Wrapped prose, which is the overwhelming majority and must not move.
    expect(isBlockProperty("Three consecutive years of an overall grade of \"F\" under R.C.\n3302.03(C)(3), or an overall grade of D.")).toBe(false);
    // One line is never a block: there is no alignment to preserve in a single line.
    expect(isBlockProperty("R.C. 3317.017;   state share at 3317.017(B).")).toBe(false);
  });

  test("keeps its line breaks and its columns, where the old rule ran them together", () => {
    const html = renderPropertyValue(
      "C6 Local Capacity Percentage = if C4 >= C5              then 0.025\n" +
        "                               if C4 <= 1               then C4 x 0.0225",
      "formula-component",
    );
    expect(html).toMatch(BLOCK);
    expect(html).toContain("then 0.025\n");
    // The interior padding is what lines the `then` column up, and is the whole point.
    expect(html).toMatch(/C5 {2,}then/);
  });

  test("strips trailing whitespace, which is invisible and widens the scroll box", () => {
    const html = renderPropertyValue("a  = 1   \nbb = 2", "metric");
    expect(html).toContain("a  = 1\nbb = 2");
  });

  test("still badges a claim tag inside a block", () => {
    // Safe only because the tags sit at the ends of lines — measured, 43 tagged lines inside
    // blocks and every one carries its tag at or past the last column, so nothing a reader is
    // comparing moves. A badge in the middle of a block would break the alignment under it.
    const html = renderPropertyValue(
      "FY2026  83.33%  H.B. 96  [verified]\nFY2027  100%    H.B. 96  [verified]",
      "parameter",
    );
    expect(html).toMatch(BLOCK);
    expect(html).toContain('class="claim verified"');
  });

  /**
   * The corpus drives this rather than a fixture, so a property written tomorrow is covered today.
   *
   * The count is asserted loosely on purpose: pinning it exactly would fail on every corpus commit
   * that adds an aligned property, which is a change this test should welcome rather than block.
   * What it does pin is that the known-worst offenders are among them.
   */
  test("every aligned property in the corpus renders as a block", () => {
    const blocks = corpus.nodes.flatMap((node) =>
      node.properties
        .filter((property) => BLOCK.test(renderPropertyValue(property.value, node.className)))
        .map((property) => `${node.id}#${property.name}`),
    );
    expect(blocks.length, "aligned property values in the corpus").toBeGreaterThan(30);
    for (const known of [
      "formula-component/fsfp-local-capacity-measure#function",
      "formula-component/fsfp-transportation#function",
      "formula-component/fsfp-targeted-assistance#function",
      "litigation/derolph-ii-2000#procedural_history",
      "parameter/fsfp-phase-in-percentage#series",
    ]) {
      expect(blocks, `${known} is an aligned block and must render as one`).toContain(known);
    }
  });

  /**
   * And the 544 that are prose are byte-identical to what shipped, **emphasis excepted**.
   *
   * This is the assertion that makes a change to this renderer safe to ship: it touches every
   * property on 3,506 pages, and "I only changed the aligned ones" is exactly the sort of claim
   * that is true when written and false three commits later. The old rule is reproduced here as
   * the oracle rather than described, so the comparison is against behaviour rather than against
   * a summary of it.
   *
   * # Why it is no longer *exactly* as before, and how it keeps its teeth
   *
   * #216 added `**bold**` and `*italic*`, which by construction moves 209 and 53 cells. Adding
   * those to the oracle would make the test compare a reimplementation against itself and assert
   * nothing at all.
   *
   * So the emphasis is **undone** before the comparison — `<strong>x</strong>` back to `**x**` —
   * and the result must still be byte-identical to the old rule. That keeps the whole of the
   * original guarantee: every property whose rendering moved for any reason other than emphasis
   * is still named, and the count of properties that did gain emphasis is asserted separately so
   * the change cannot silently reach nothing.
   */
  test("and every property that is not a block renders exactly as it did before, emphasis aside", () => {
    const before = (value: string, fromClass: string): string =>
      badgeClaims(
        escapeHtml(value)
          .replace(/\[([^[\]]+)\]\(([^)\s]+)\)/g, (_whole, label: string, target: string) => {
            const href = /^(https?:|\/)/.test(target) ? target : resolveTarget(target, fromClass).href;
            return `<a href="${href}">${label}</a>`;
          })
          .replace(/`([^`]+)`/g, "<code>$1</code>"),
      )
        .split(/\n\s*\n/)
        .map((paragraph) => paragraph.replace(/\s*\n\s*/g, " ").trim())
        .filter((paragraph) => paragraph !== "")
        .join("<br><br>");

    /**
     * The one difference this test licenses, put back so everything else must match.
     *
     * `[\s\S]*?` and not `[^<]*`: the corpus writes `**\`Am. Sub.\` is not confirmed**` and
     * `**[the plan](…)**`, so emphasis routinely wraps a tag the earlier passes already emitted.
     * A body that stopped at `<` failed to undo exactly those, and the test reported them as
     * drift — which is the right failure for the wrong reason, and would have been read as the
     * renderer breaking links.
     *
     * Strong is undone before em so a nested pair unwinds outside-in. Neither can nest in itself:
     * both patterns run on `[^*]`, which cannot cross the delimiter that would open a second one.
     */
    const unemphasised = (html: string): string =>
      html
        .replace(/<strong>([\s\S]*?)<\/strong>/g, "**$1**")
        .replace(/<em>([\s\S]*?)<\/em>/g, "*$1*");

    const drifted: string[] = [];
    let unchanged = 0;
    let emphasised = 0;
    for (const node of corpus.nodes) {
      for (const property of node.properties) {
        if (isBlockProperty(property.value)) continue;
        const now = renderPropertyValue(property.value, node.className);
        if (/<(strong|em)>/.test(now)) emphasised += 1;
        if (unemphasised(now) === before(property.value, node.className)) unchanged += 1;
        else drifted.push(`${node.id}#${property.name}`);
      }
    }
    expect(drifted, "non-block properties whose rendering moved for a reason other than emphasis").toEqual([]);
    expect(unchanged, "properties checked against the old rule").toBeGreaterThan(600);
    // The other direction: a renderer that emphasised nothing would satisfy everything above.
    expect(emphasised, "properties that gained emphasis").toBeGreaterThan(40);
  });
});

/**
 * The list that was run together.
 *
 * The join above is right for 544 paragraphs and the aligned-block rescue catches the 57 that are
 * tables — it looks for columns. A list of phrases has no columns, so it fell through to the join:
 * `written_as` on `base-cost-per-pupil` shipped as "statewide average base cost per pupil average
 * base cost per pupil district base cost per pupil district aggregate base cost". Four phrases and
 * no way to tell where any of them ends, on the one field whose entire subject is which exact
 * phrases name a quantity.
 */
describe("a property value that is a list", () => {
  test("is declared by name, because a one-item list is still a list", () => {
    expect(isListProperty("written_as")).toBe(true);
    // A property whose lines happen to be short is still prose, and must keep the join.
    expect(isListProperty("sensitivity")).toBe(false);
    expect(isListProperty("series")).toBe(false);
  });

  test("renders one item per line rather than one run-on sentence", () => {
    const html = renderListProperty(
      "statewide average base cost per pupil\naverage base cost per pupil\ndistrict aggregate base cost",
      "parameter",
    );
    expect(html.match(/<li>/g), "one item per phrase").toHaveLength(3);
    expect(html).toContain("<li>statewide average base cost per pupil</li>");
    // The failure this exists to prevent: two phrases with nothing between them.
    expect(html).not.toContain("per pupil average base cost");
  });

  test("gives a phrase the badges, links and code spans every other property gets", () => {
    const html = renderListProperty("the `school-age` weights [verified]\nthe weights", "parameter");
    expect(html).toContain("<code>school-age</code>");
    expect(html).toContain('class="claim verified"');
  });

  test("carries every phrase the corpus declares, and drops the blank lines", () => {
    const declared = corpus.nodes.flatMap((node) =>
      node.properties.filter((property) => isListProperty(property.name)).map((property) => ({
        id: node.id,
        phrases: property.value.split("\n").map((line) => line.trim()).filter((line) => line !== ""),
        html: renderListProperty(property.value, node.className),
      })),
    );
    expect(declared.length, "parameters declaring the phrases a formula writes them as").toBeGreaterThan(10);
    for (const { id, phrases, html } of declared) {
      expect(html.match(/<li>/g) ?? [], `${id} renders one item per declared phrase`).toHaveLength(
        phrases.length,
      );
    }
  });
});

/**
 * Emphasis in a property cell.
 *
 * `renderPropertyValue` handled inline links and backticks and not emphasis, so the asterisks
 * shipped. Measured over the built site before the fix: **209 bold pairs across 67 of the 226 wiki
 * pages, and 53 italic pairs across 33** — including case names, which is where it read worst:
 * `*DeRolph II*` and `*Columbus City School Dist. v. State*` in the middle of a sentence about
 * them.
 *
 * The description above each table has always rendered both, because that path goes through the
 * markdown processor. So the corpus was writing one convention and the page was honouring it in
 * one of the two places it appears.
 */
describe("emphasis in a property value", () => {
  test("renders bold and italic, and nests a link or a code span inside either", () => {
    expect(renderPropertyValue("the **operative** mechanism", "parameter")).toBe(
      "the <strong>operative</strong> mechanism",
    );
    expect(renderPropertyValue("in *DeRolph II* the court", "litigation")).toBe(
      "in <em>DeRolph II</em> the court",
    );
    // Both of these read as drift until the oracle's undo was widened to reach them.
    expect(renderPropertyValue("**`Am. Sub.` is not confirmed**", "fiscal-period")).toBe(
      "<strong><code>Am. Sub.</code> is not confirmed</strong>",
    );
    expect(
      renderPropertyValue("[*DeRolph I*](../litigation/derolph-i-1997.yml) held", "parameter"),
    ).toContain("<em>DeRolph I</em>");
  });

  test("bold wins over italic where they would both match", () => {
    // `**x**` read italic-first is an empty `<em>` wrapping `*x*`, which is why the order matters.
    expect(renderPropertyValue("**x**", "metric")).toBe("<strong>x</strong>");
    expect(renderPropertyValue("*a* and *b*", "metric")).toBe("<em>a</em> and <em>b</em>");
  });

  test("and leaves arithmetic alone, which is the only other thing an asterisk means here", () => {
    /*
     * Three occurrences in the corpus and every one is a product: `enrolled-adm`'s projection line
     * and `dispersion::of`'s `q * (n - 1)`. The guard is adjacency — an emphasis delimiter has a
     * non-space immediately inside it and a product has spaces on both sides.
     */
    const formula = "value = ADM[last] * PROD over h of (1 + rate * damping^h)";
    expect(renderPropertyValue(formula, "metric")).not.toContain("<em>");
    expect(renderPropertyValue("R type 7 on `q * (n - 1)`", "metric")).not.toContain("<em>");
    // A snake_case identifier is not italic either, which is why underscores are not emphasis.
    expect(renderPropertyValue("`exp_per_equivalent_pupil_federal`", "metric")).not.toContain(
      "<em>",
    );
  });

  test("no property in the corpus still ships an emphasis delimiter", () => {
    /*
     * The whole-corpus form, which catches a shape the four cases above do not have. An asterisk
     * surviving into the output is either emphasis this did not reach or a delimiter the author
     * left unclosed, and both are worth a failure. A spaced ` * ` is arithmetic and is exempt.
     */
    const left: string[] = [];
    for (const node of corpus.nodes) {
      for (const property of node.properties) {
        const html = renderPropertyValue(property.value, node.className).replace(/\s\*\s/g, " ");
        if (html.includes("*")) left.push(`${node.id}#${property.name}`);
      }
    }
    expect(left, "a property value still carrying an emphasis delimiter").toEqual([]);
  });
});
