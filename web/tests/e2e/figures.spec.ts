/**
 * Every figure on every built page can reach the year it is measured in.
 *
 * # What this closes, and why the count in the issue was the wrong count
 *
 * `design/README.md` recorded `.fig` as "available and not yet adopted" and priced adoption at 750
 * call sites. #191 recounted and got 450 formatter calls against three `fig()` ones, and proposed
 * deciding with the number in hand. Both numbers answer "how many places format a number", which
 * is not the question the rule exists for. The question is how many rendered figures a reader
 * cannot date, and it is answerable only from the built pages, because a figure is dated by where
 * it lands — a card's chip, a column head, the sentence around it — and not by the call that
 * formatted it.
 *
 * Measured over all 3,493 pages: **4,637 figures with no year within reach**, in eighteen shapes.
 * Not 450 edits. Six site-authored templates, and 2,724 of the 4,637 were one of them — the
 * `/districts` table, where six columns drawn from three series sat under heads that named none,
 * with the footnote that named two of the three years 609 rows below the first figure it is about.
 * A wide table repeats an omission once per cell, which is why one page held more of these than
 * the rest of the build put together.
 *
 * What the eighteen shapes did NOT include is as much of the answer: no chipped card, no table
 * whose head carries its year, and nothing in the 44% of money strings that sit in a dated column.
 * The mechanism was working everywhere it had been applied. It had not been applied outside cards.
 *
 * # Reach, and the five ways a figure has it
 *
 * A figure is dated if any one of these holds. They are not ranked; a figure needs one.
 *
 * 1. **It is inside a `.fig`** — the compound carries the year as a child, so the year cannot be
 *    dropped without the markup losing an element.
 * 2. **It is in a table cell whose column head, row header or caption names a year.** The design
 *    system's rule: the annotation goes on the head rather than down 609 rows. A cell holding a
 *    *sentence* may also take the year from elsewhere in its own row — a record's year is a
 *    property of the record. That relaxation is confined to prose cells on purpose: in a numeric
 *    table each column is a different series, and letting one cell's year license another's is
 *    exactly the confusion the chips exist to prevent.
 * 3. **Its card or section heading carries a `.year-chip`.** This is the mechanism most of the
 *    site already uses, and `app.spec.ts` separately requires every card with figures to have one.
 * 4. **The sentence it sits in names a year.**
 * 5. **It is in a tile or a distribution strip whose own text names a year, or which contains a
 *    `.fig`.** Both are compact composed units — key, value, note; note, chart, scale — a few
 *    dozen characters end to end. There is no "elsewhere on the page" inside one.
 *
 * # Exemptions carry a reason, and are enforced in both directions
 *
 * The shape `yearLiterals.spec.ts` uses, for the reason its header gives: an exemption whose
 * reason reads "it is what the page says" is not an exemption, it is an unfixed defect. So an
 * entry that stops matching fails too — a licence nobody is using is one waiting to cover the next
 * figure somebody adds.
 */

import { readFileSync, readdirSync } from "node:fs";
import { join, relative } from "node:path";

import { expect, test } from "@playwright/test";
import { parseHTML } from "linkedom";

const DIST = join(import.meta.dirname, "../../dist");

/** A money amount or a percentage, as this site's formatters render them. */
const FIGURE = /[−-]?\$[\d,]+(\.\d+)?(B|M|m|k)?|[−-]?\d[\d,]*(\.\d+)?%/;

/**
 * A year in any of Ohio's three reckonings.
 *
 * Lenient on the right and strict on the left, and both sides were found by being wrong. Strict on
 * the left because a district IRN is six digits and `043786` must not read as the year 3786.
 * Lenient on the right because `textContent` concatenates adjacent elements with nothing between
 * them: `<div class="n">…FY2024</div>` followed by a definition renders as two lines and reads as
 * `FY2024A headcount…`, so a trailing `\b` reported 44 cells that name their year in the cell.
 */
const YEAR = /(?<![A-Za-z0-9])(FY|TY)?(19|20)\d{2}(?![0-9])/;

/** Above this many words a cell holds a sentence rather than a value — the count `measure.ts` uses. */
const PROSE_CELL_MIN_WORDS = 6;

/** A licence for figures that are not measurements of Ohio in a year. */
interface Allowance {
  /** Why a year would be wrong here rather than merely absent. */
  reason: string;
  /** Exactly the figures the reason argues for, as the page renders them. */
  figures: string[];
}

/** A container whose figures are dated by something other than this page. */
interface Exempt {
  /** Why the figures inside it are covered, or why a year would be wrong. */
  reason: string;
  /** The containers, as selectors. A figure inside any of them is licensed. */
  containers: string[];
  /** Where the containers count, when the same class means different things elsewhere. */
  route?: RegExp;
}

const CONTAINERS: Exempt[] = [
  {
    /*
     * The corpus's own prose dates its figures by a different and stronger mechanism.
     *
     * A node quotes a number from a source and binds it with a `figures:` entry naming the crate
     * that computes it — see `.yidam/corpus/README.md`. Stronger, because it survives the number
     * being wrong: a correction's blast radius becomes the binding rather than whichever files the
     * author happened to open. Wrapping corpus sentences in site markup would also be this site
     * editorialising the corpus, which `/wiki` states it does not do.
     *
     * Sixty-six of the seventy-three figures left after this phase are these.
     */
    reason:
      "Corpus prose, dated by the node's own `figures:` bindings rather than by this site's markup. #158 is the issue that finishes binding them.",
    route: /^wiki[/.]/,
    containers: [
      ".prose-body",
      ".record",
      ".revision-body",
      ".claim-detail",
      ".row-note",
      "p.lead",
      "table.prose td",
    ],
  },
  {
    /*
     * A contents entry is an address, not an assertion, and the omission is deliberate.
     *
     * `words()` in `lib/contents.ts` strips the year chip out of the label it copies from the
     * heading, because "# Why base cost is $8,120 per pupil FY2027" in a list of eight reads as a
     * distinction between the entries rather than as an annotation on each — every one of them
     * carries the same chip. The heading itself keeps it, three lines down the page.
     */
    reason:
      "A contents entry repeats a heading as a link. `contents.ts` strips the year chip from the label on purpose; the heading it addresses carries it.",
    containers: ["nav.contents"],
  },
];

/**
 * Figures that are not measurements, keyed by the route that renders them.
 *
 * Every one is on `/method`, and that is not a coincidence: it is the page about how this
 * repository reproduces the department's figures, so most of its numbers are about the work rather
 * than about Ohio in a year.
 */
const NOT_A_MEASUREMENT: Record<string, Allowance> = {
  "method.html": {
    figures: ["60%", "2.5%", "$4.7", "4.4%", "$7.13B", "3.4%"],
    reason:
      "Three kinds, none of which a fiscal year would qualify. 60% and 2.5% are rates written into R.C. 3317.017 — a statutory rate is not measured in a year, and dating it would assert that it changes annually when the whole point of the `parameter` class is that its changes are legislated events. $4.7 million and 4.4% are the magnitudes of two defects in this repository's own Rust, quoted in the account of how they were found; they are facts about the code's history, and the sentences say so. $7.13B and 3.4% appear inside quotation marks as an example of a rendering the page argues against — 'A forecast rendered as \"$7.13B (±3.4%)\" is read as $7.13B' — so they are typography being discussed, not figures being asserted.",
  },
};

function walk(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
    entry.isDirectory()
      ? walk(join(dir, entry.name))
      : entry.name.endsWith(".html")
        ? [join(dir, entry.name)]
        : [],
  );
}

test("every figure on every page can reach the year it is measured in", () => {
  const undated: string[] = [];
  /* Both directions: a licence that no longer covers anything is reported as loudly as a figure
     that no licence covers. */
  const used = new Set<string>();

  for (const file of walk(DIST)) {
    const html = readFileSync(file, "utf8");
    if (!FIGURE.test(html)) continue;
    const route = relative(DIST, file);
    const allowed = NOT_A_MEASUREMENT[route];

    const { document } = parseHTML(html);
    /* `<annotation>` is MathML's copy of the LaTeX source. It is inside `<semantics>` and is never
       painted, so a `\$90` in it is markup rather than a figure a reader meets. */
    for (const el of document.querySelectorAll("script,style,svg,head,template,annotation")) {
      el.remove();
    }

    const texts: Text[] = [];
    const collect = (node: Node): void => {
      for (const child of node.childNodes) {
        if (child.nodeType === 3) texts.push(child as Text);
        else if (child.nodeType === 1) collect(child);
      }
    };
    collect(document.body ?? document);

    for (const text of texts) {
      const value = text.textContent ?? "";
      if (!FIGURE.test(value)) continue;
      const parent = text.parentElement;
      if (!parent) continue;

      // 1. The compound.
      if (parent.closest(".fig")) continue;

      // 5. A tile or a strip, which is one composed unit end to end.
      const unit = parent.closest(".tile, .measure");
      if (unit && (unit.querySelector(".fig") || YEAR.test(unit.textContent ?? ""))) continue;

      // 3. A chipped card or section, at any depth.
      let chipped = false;
      let box = parent.closest(".card, .tile, section, .panel");
      while (box && !chipped) {
        const heading = box.querySelector("h1,h2,h3,h4,caption");
        if (heading?.querySelector(".year-chip") || box.querySelector(":scope > .year-chip-wrap")) {
          chipped = true;
        }
        box = box.parentElement?.closest(".card, .tile, section, .panel") ?? null;
      }
      if (chipped) continue;

      // 2. The column head, the row header, the caption — and the record, for a prose cell.
      const cell = parent.closest("td,th");
      if (cell) {
        const table = cell.closest("table");
        const row = cell.closest("tr");
        const index = row ? [...row.children].indexOf(cell) : -1;
        const head = table?.querySelectorAll("thead tr")[0]?.children?.[index];
        const scope = [
          head?.textContent,
          row?.querySelector("th")?.textContent,
          table?.querySelector("caption")?.textContent,
        ];
        const words = (cell.textContent ?? "").trim().split(/\s+/).length;
        if (words >= PROSE_CELL_MIN_WORDS) scope.push(row?.textContent);
        if (scope.some((t) => YEAR.test(t ?? ""))) continue;
      }

      // 4. The sentence.
      const block = parent.closest("p,li,figcaption,dd,dt,td,th,h1,h2,h3,h4,div") ?? parent;
      if (YEAR.test(block.textContent ?? "")) continue;

      const container = CONTAINERS.find(
        (e) =>
          (e.route?.test(route) ?? true) && e.containers.some((selector) => parent.closest(selector)),
      );
      if (container) {
        used.add(container.reason);
        continue;
      }

      const figures = value.match(new RegExp(FIGURE.source, "g")) ?? [];
      const unlicensed = figures.filter((f) => !allowed?.figures.includes(f));
      for (const f of figures) if (allowed?.figures.includes(f)) used.add(`${route}: ${f}`);
      if (unlicensed.length === 0) continue;

      undated.push(
        `${route}: ${unlicensed.join(", ")} — in "${value.trim().replace(/\s+/g, " ").slice(0, 90)}"`,
      );
    }
  }

  expect(
    undated,
    "a figure with no year in reach: put it in a `.fig`, chip its card, date its column head, or say the year in the sentence",
  ).toEqual([]);

  const unused = [
    ...Object.entries(NOT_A_MEASUREMENT).flatMap(([route, allowance]) =>
      allowance.figures.filter((f) => !used.has(`${route}: ${f}`)).map((f) => `${route}: ${f}`),
    ),
    ...CONTAINERS.filter((e) => !used.has(e.reason)).map((e) => e.containers.join(", ")),
  ];
  expect(unused, "an exemption covering nothing — delete it rather than leave it to cover the next figure").toEqual([]);
});
