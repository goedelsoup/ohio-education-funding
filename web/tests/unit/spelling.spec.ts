/**
 * One spelling: US forms in everything a reader sees (#572).
 *
 * The site is about Ohio, and Ohio spells it *enrollment* — the Revised Code does, the department
 * does, and so do the statute quotations the corpus carries. The copy around those quotations had
 * drifted to British forms ("enrolment" 3,459 times across 1,296 pages, "programme", "centre",
 * "-ise"), so a quoted section and the sentence explaining it spelled the same word two ways, and
 * an institution's own name ("Post-Secondary Enrollment Options", "educational service centers")
 * was respelled in the sentence that named it.
 *
 * # What is scanned
 *
 * - **Corpus and decision prose** — every string value in `.yidam/corpus/**` and
 *   `.yidam/decisions/*`, which is what the wiki renders. Keys are never read.
 * - **The catalog** — `.yidam/catalog/*.md`, which renders as `/wiki/source/*`.
 * - **`web/src` copy** — `.ts` and `.astro` with comments blanked. A comment is not rendered, and
 *   is left in whatever spelling its author wrote.
 *
 * # What is not, and why each exemption is narrow
 *
 * - **Addresses.** A node id, a figure key's segment, a path, a URL, a link target or a code span
 *   is a name a machine resolves; respelling one breaks a link rather than fixing a word. So
 *   `guarantee-open-enrolment-clawback` stays, and the node's *label* does not.
 * - **Quotations** — {@link QUOTED}. The department's workbook names its line "Open Enrolment
 *   Adjust.", and a quotation is evidence of what a source says.
 * - **Identifiers** — {@link IDENTIFIERS}, per file and per word. `colour` as a parameter name in
 *   the palette maths is never on a page, and listing it by word means a British word typed into
 *   that file's copy still fails.
 */

import { readFileSync, readdirSync } from "node:fs";
import { basename, join, relative } from "node:path";

import { expect, test } from "vitest";
import { parse } from "yaml";

const ROOT = join(import.meta.dirname, "../../..");
const SRC = join(ROOT, "web/src");

/** Forms that are British and never American. Case-insensitive, whole word. */
const UK = new RegExp(
  "\\b(?:" +
    [
      "enrol(?:s|ment|ments)?",
      "centre(?:s|d)?",
      "centring",
      "programmes?",
      "artefacts?",
      "judgements?",
      "catalogue(?:s|d)?",
      "licences?",
      "defence",
      "(?:colour|favour|behaviour|neighbour|labour|honour|flavour|rumour|humour|harbour|vapour)\\w*",
      "(?:un|re|mis)?(?:modell|labell|levell|cancell|travell|totall|diall|counsell|equall|fuell|signall|channell|credentiall)(?:ed|ing|er|ers)",
      "\\w+isations?",
      "\\w+is(?:e|es|ed|ing|able)",
      "\\w+ys(?:e|ed|ing)",
      "whilst",
      "amongst",
      "instalments?",
    ].join("|") +
    ")\\b",
  "gi",
);

/**
 * Words the `-ise` / `-yse` arms reach that are American too. The arm is broad on purpose — a
 * list of British verbs is never finished — so the exceptions are what is kept short.
 */
const AMERICAN = new Set(
  [
    "advertise", "advise", "anise", "appraise", "arise", "chastise", "circumcise", "comprise",
    "compromise", "concise", "demise", "despise", "devise", "disguise", "enterprise", "excise",
    "exercise", "expertise", "franchise", "guise", "improvise", "incise", "likewise", "merchandise",
    "noise", "otherwise", "paradise", "poise", "praise", "precise", "premise", "promise", "raise",
    "reappraise", "revise", "rise", "supervise", "surprise", "televise", "treatise", "wise",
    "clockwise", "pairwise", "piecewise", "stepwise", "elementwise", "crosswise", "lengthwise",
    "analyses", "hypotheses", "emphases", "syntheses", "parentheses", "theses", "crises", "bases",
    "cruise", "bruise", "denise", "louise", "elise", "valise", "mortise", "vise", "chemise",
    "uprise", "sunrise", "moonrise", "high-rise", "franchises", "premises", "unrevised",
  ].flatMap((w) => [w, w.replace(/e$/, "ed"), w.replace(/e$/, "es"), w.replace(/e$/, "ing")]),
);

/** Department text quoted as written. Matched as a substring of the line it appears in. */
const QUOTED = [
  // The FY2026-27 workbook's own line names for `[I1]`, reproduced in the node's formula block.
  "Open Enrolment Adjust",
];

/** Code identifiers that are never rendered, by file and by word. */
const IDENTIFIERS: Record<string, string[]> = {
  "lib/plot/palette.ts": ["colour"],
  "lib/og/indexed.ts": ["colour"],
  "lib/plot/spec.ts": ["labelled"],
  // `programmes` is the feed's key for a funding unit's programs (`crates/bundle/src/serialize.rs`).
  "lib/schema/feed.ts": ["programmes"],
  "lib/statewide.ts": ["programmes"],
  "pages/method.astro": ["programmes"],
  "lib/semantics.ts": ["relevelled"],
  // An error thrown at build time, never rendered.
  "lib/plot/ssr.ts": ["colour"],
};

/** Names a machine resolves: every node and decision id, and every figure key's segments. */
function addresses(): Set<string> {
  const ids = new Set<string>();
  const walk = (dir: string): void => {
    for (const e of readdirSync(dir, { withFileTypes: true })) {
      const path = join(dir, e.name);
      if (e.isDirectory()) walk(path);
      else if (/\.(yml|md)$/.test(e.name)) ids.add(basename(e.name).replace(/\.(ont\.)?(yml|md)$/, ""));
    }
  };
  walk(join(ROOT, ".yidam/corpus"));
  walk(join(ROOT, ".yidam/decisions"));
  walk(join(ROOT, ".yidam/catalog"));
  const manifest = JSON.parse(readFileSync(join(ROOT, "crates/figures.json"), "utf8")) as
    | { figures: { key: string }[] }
    | { key: string }[];
  const figures = Array.isArray(manifest) ? manifest : manifest.figures;
  for (const f of figures) for (const segment of f.key.split("/")) ids.add(segment);
  return ids;
}
const ADDRESSES = addresses();

/** Blank what is not a word a reader reads, keeping offsets. */
function unaddressed(text: string): string {
  const blank = (m: string): string => m.replace(/[^\n]/g, " ");
  return text
    .replace(/\]\([^)\n]*\)/g, blank)
    .replace(/\bhttps?:\/\/\S+/g, blank)
    .replace(/[\w.-]*\/[\w./-]+/g, blank)
    .replace(/[\w-]+\.(?:yml|md|csv|tsv|rs|ts|json|xlsx|pdf)\b/g, blank)
    .replace(/\b[a-z0-9]+(?:[-_][a-z0-9]+)+\b/g, (m) =>
      m.includes("_") || ADDRESSES.has(m) ? blank(m) : m,
    );
}

function britishIn(text: string, allowed: string[] = []): string[] {
  const hits: string[] = [];
  for (const line of text.split("\n")) {
    if (QUOTED.some((q) => line.includes(q))) continue;
    for (const m of unaddressed(line).matchAll(UK)) {
      const word = m[0].toLowerCase();
      if (AMERICAN.has(word) || allowed.includes(word)) continue;
      hits.push(`${m[0]} — ${line.trim().slice(0, 120)}`);
    }
  }
  return hits;
}

function files(dir: string, pattern: RegExp): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
    const path = join(dir, e.name);
    if (e.isDirectory()) return e.name === ".vendor" ? [] : files(path, pattern);
    return pattern.test(e.name) ? [path] : [];
  });
}

/** Keys whose values are addresses, not sentences: an edge's target and its relationship name. */
const ADDRESS_KEYS = new Set(["id", "key", "target", "relationship", "class"]);

function strings(value: unknown): string[] {
  if (typeof value === "string") return [value];
  if (Array.isArray(value)) return value.flatMap(strings);
  if (value && typeof value === "object")
    return Object.entries(value).flatMap(([k, v]) => (ADDRESS_KEYS.has(k) ? [] : strings(v)));
  return [];
}

test("corpus and decision prose spells American", () => {
  const found: string[] = [];
  for (const file of [
    ...files(join(ROOT, ".yidam/corpus"), /\.yml$/),
    ...files(join(ROOT, ".yidam/decisions"), /\.yml$/),
  ]) {
    const text = strings(parse(readFileSync(file, "utf8"))).join("\n");
    // A code span names something; it is not a word in the sentence.
    for (const hit of britishIn(text.replace(/`[^`\n]*`/g, (m) => " ".repeat(m.length))))
      found.push(`${relative(ROOT, file)}: ${hit}`);
  }
  expect(found).toEqual([]);
});

test("catalog entries spell American", () => {
  const found: string[] = [];
  for (const file of files(join(ROOT, ".yidam/catalog"), /\.md$/)) {
    const text = readFileSync(file, "utf8").replace(/`[^`\n]*`/g, (m) => " ".repeat(m.length));
    for (const hit of britishIn(text)) found.push(`${relative(ROOT, file)}: ${hit}`);
  }
  expect(found).toEqual([]);
});

test("web copy spells American", () => {
  const found: string[] = [];
  for (const file of files(SRC, /\.(ts|astro)$/)) {
    const rel = relative(SRC, file);
    const source = readFileSync(file, "utf8")
      .replace(/<!--[\s\S]*?-->/g, (m) => m.replace(/[^\n]/g, " "))
      .replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, " "))
      .replace(/(^|[^:"'`\\])\/\/.*$/gm, (m, lead: string) => lead + " ".repeat(m.length - lead.length));
    for (const hit of britishIn(source, IDENTIFIERS[rel])) found.push(`${rel}: ${hit}`);
  }
  expect(found).toEqual([]);
});

test("every identifier exemption is still in use", () => {
  const unused: string[] = [];
  for (const [rel, words] of Object.entries(IDENTIFIERS)) {
    const source = readFileSync(join(SRC, rel), "utf8");
    for (const w of words) if (!new RegExp(`\\b${w}\\b`).test(source)) unused.push(`${rel}: ${w}`);
  }
  expect(unused).toEqual([]);
});

test("the scan reaches a British word, a slug and a quotation the way it says", () => {
  expect(britishIn("its open enrolment fell")).toHaveLength(1);
  expect(britishIn("the programme was modelled and standardised")).toHaveLength(3);
  expect(britishIn("otherwise precise analyses of the premises")).toEqual([]);
  expect(britishIn("[the node](guarantee-open-enrolment-clawback.yml)")).toEqual([]);
  expect(britishIn("see guarantee-open-enrolment-clawback")).toEqual([]);
  expect(britishIn("[I1] Open Enrolment Adjust. = if (prior FTE - current FTE)")).toEqual([]);
});
