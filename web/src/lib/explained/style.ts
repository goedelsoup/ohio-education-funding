/**
 * The Explained style guide, as data, and the checks that read a topic against it (#714).
 *
 * # Why the guide is code
 *
 * A style guide in prose is followed by the writer who read it last. These pages are written one
 * at a time across a dozen PRs, and the words they share — `guarantee`, `local share`, `foundation
 * aid` — mean one thing each only if every page uses them the same way. So the dictionary is a
 * table that `tests/unit/explained.spec.ts` reads every registered topic against, and the limits
 * are numbers it compares against.
 *
 * # What the checks read
 *
 * The plain sections: the short answer, the picture, where it breaks, and the prose of the worked
 * example. Not the rule, which is notation; not "How we know", which is in the corpus's register
 * and carries its claim tags (`.yidam/decisions/explained-is-a-reading-not-a-record.yml`); and not
 * the question, which `QUESTION_LIMIT` already bounds.
 *
 * # The heuristics are heuristics
 *
 * Sentences are split on terminal punctuation with the site's abbreviations protected, syllables
 * are counted by vowel groups, and a noun cluster is a run of words with no function word or verb
 * form between them. Each can be fooled. Each is tested against a sentence that should pass and one
 * that should fail, and a false positive is fixed by rewording the sentence — the plain reading the
 * check was standing in for — rather than by loosening the check.
 */

import { plain, type Prose, type Topic } from "./topic.ts";

/** One approved term, and the words a page must not use in its place. */
export interface Term {
  /** The word to write. */
  write: string;
  /** The words not to write, as patterns matched whole-word and case-blind against plain text. */
  not: readonly RegExp[];
  /** Why, so a reviewer can tell a rule from a taste. */
  why: string;
  /** Topics, by slug, where a forbidden word is the subject and may be named. */
  allowedIn?: readonly string[];
}

/**
 * The house dictionary: each approved term has one meaning, and each forbidden synonym maps to it.
 *
 * `funding` is forbidden alone and allowed in the two phrases that name the system rather than a
 * measure of it — "school funding" and "funding formula" — because the site's own name is one of
 * them and neither says how much anyone is paid.
 */
export const DICTIONARY: readonly Term[] = [
  {
    write: "guarantee",
    not: [/\bhold[- ]harmless\b/i],
    why: "one word for column [H2]'s mechanism",
  },
  {
    write: "local share",
    not: [/\bcharge[- ]off\b/i],
    why: "the charge-off was repealed; a page about the old formula names it, and no other does",
    // The history topic (#718's "Why is 2011 still inside the formula?") adds its slug here.
    allowedIn: [],
  },
  {
    write: "foundation aid, or total state support",
    not: [/(?<!\bschool )\bfunding\b(?! formula)/i],
    why: 'the two measures disagree in sign statewide, so "funding" alone is ambiguous, not loose',
  },
  {
    write: "district",
    not: [/\bschool systems?\b/i, /\bLEAs?\b/],
    why: "the feed, the department and every other page call it a district",
  },
  {
    write: "phase-in",
    not: [/\bramp(?:s|ed)?\b/i, /\btransition\b/i],
    why: "`transition` is also a feed term, and a ramp is a picture, not a name",
  },
];

/** The limits, in words and sentences. */
export const LIMITS = {
  /** A step of the worked example: one sentence a reader can check against the one before it. */
  stepWords: 20,
  /** Every other plain sentence. */
  sentenceWords: 25,
  /** The one sentence per paragraph that may run long, if it carries a number with its unit and year. */
  longSentenceWords: 35,
  /** A paragraph: each `Prose` item is one. */
  paragraphSentences: 6,
  /** Nouns in a row with nothing between them: "base cost aid" is three, and the most allowed. */
  nounCluster: 3,
} as const;

/**
 * Where the guide is looser than a plain-language standard, recorded so a reviewer can check a
 * page against what was actually allowed rather than against a stricter rule nobody adopted.
 */
export const LOOSENINGS: readonly string[] = [
  "One comparison per page: the picture, and no second metaphor in the steps.",
  'The reader may be called "you".',
  "A legal name — R.C. 3317.022, H.B. 110 — may be used once it has been said what it is.",
  'The passive is allowed where the law is the actor: "the share is set in the budget".',
];

/**
 * The Flesch–Kincaid grade the short answer, the picture and where it breaks must not exceed.
 *
 * Measured on the pilot (#715), "What does the phase-in do?": its short answer reads at grade 4.6,
 * its picture at 2.9 and where the picture breaks at 3.7. The ceiling is #712's 8 and not the
 * pilot's 4.6, because the pilot is the smallest identity in the plan and the outcome and
 * projection topics have more to name. A dense picture in `tests/unit/explained.spec.ts` is
 * what shows the check bites at this ceiling and not only at zero.
 */
export const GRADE_CEILING: number | undefined = 8;

/** Abbreviations the site writes that end in a full stop without ending a sentence. */
const ABBREVIATIONS = /\b(?:R\.C|H\.B|S\.B|Am|Sub|U\.S|e\.g|i\.e|vs|No|Inc|St)\.$/;

/** A run of text, split into sentences. */
export function sentences(text: string): string[] {
  const out: string[] = [];
  let start = 0;
  for (const match of text.matchAll(/[.?!]["”’)]*(?=\s+["“(]?[A-Z]|\s*$)/g)) {
    const end = match.index + match[0].length;
    const candidate = text.slice(start, end).trim();
    if (ABBREVIATIONS.test(text.slice(start, match.index + 1))) continue;
    if (candidate) out.push(candidate);
    start = end;
  }
  const tail = text.slice(start).trim();
  if (tail) out.push(tail);
  return out;
}

/** The words of a run of text: letters, digits and the joins inside a word. */
export function words(text: string): string[] {
  return text.match(/\$?[\p{L}\p{N}][\p{L}\p{N}'’.,%-]*[\p{L}\p{N}%]|\$?[\p{L}\p{N}]/gu) ?? [];
}

/** Syllables in a word, by vowel groups: a heuristic, and the one Flesch–Kincaid was fitted with. */
export function syllables(word: string): number {
  const w = word.toLowerCase().replace(/[^a-z]/g, "");
  if (w.length === 0) return 1;
  if (w.length <= 3) return 1;
  const groups = w.replace(/(?:[^laeiouy]es|ed|[^laeiouy]e)$/, "").replace(/^y/, "").match(/[aeiouy]+/g);
  return Math.max(1, groups?.length ?? 1);
}

/** The Flesch–Kincaid grade of a run of text. */
export function grade(text: string): number {
  const s = sentences(text);
  const w = s.flatMap(words);
  if (w.length === 0) return 0;
  const syl = w.reduce((n, word) => n + syllables(word), 0);
  return 0.39 * (w.length / s.length) + 11.8 * (syl / w.length) - 15.59;
}

/**
 * Words that end a noun cluster: articles, prepositions, conjunctions, pronouns, auxiliaries and
 * the verbs these pages use most. A cluster is what is left between them.
 */
const BREAKS = new Set(
  (
    "a an the this that these those each every any some no all both either neither its their his her " +
    "our your my of in on at by for to from with without into onto over under between among than as " +
    "and or but nor so yet if then when while because though although where which who whom whose what " +
    "it they them he she we you i me us is are was were be been being am has have had having do does " +
    "did not can could will would shall should may might must pays paid pay gets get got moves move " +
    "moved sets set sits sit stops stop rises rise falls fall goes go takes take gives give makes make " +
    "says say reads read shows show counts count adds add"
  ).split(" "),
);

/** Runs of more than {@link LIMITS.nounCluster} words with nothing between them to break them. */
export function nounClusters(text: string): string[] {
  const found: string[] = [];
  for (const sentence of sentences(text)) {
    // Punctuation breaks a run as a function word does: "base, computed and paid" is a list.
    for (const phrase of sentence.split(/[,;:()—–"“”]/)) {
      let run: string[] = [];
      const flush = () => {
        if (run.length > LIMITS.nounCluster) found.push(run.join(" "));
        run = [];
      };
      for (const raw of phrase.split(/\s+/)) {
        const word = raw.replace(/[.?!]+$/, "");
        const lower = word.toLowerCase();
        const verbish = /(?:ly|ing)$/.test(lower) && lower.length > 4;
        if (!word || BREAKS.has(lower) || verbish || /^[\d$]/.test(word)) flush();
        else run.push(word);
      }
      flush();
    }
  }
  return found;
}

/** The irregular participles these pages are likeliest to use: "is set", "was paid". */
const IRREGULAR = "set|paid|made|held|given|taken|known|cut|built|shown|found|kept|met|put|spent|left";

/** Passive constructions, as warnings: the law may be the actor (see {@link LOOSENINGS}). */
export function passives(text: string): string[] {
  const passive = new RegExp(
    `\\b(?:is|are|was|were|be|been|being)\\s+(?:\\w+ly\\s+)?(?:\\w+ed|${IRREGULAR})\\b`,
    "gi",
  );
  return [...text.matchAll(passive)].map((m) => m[0]);
}

/** A number with its unit beside it, and a year: what licenses a long sentence. */
function carriesDatedFigure(html: string, sentence: string): boolean {
  const figure = /\$[\d,.]+|[\d.]+%|\b\d[\d,]*\s+(?:pupils|students|districts|mills|dollars)\b/.test(
    sentence,
  );
  const year = /\bFY\d{4}\b|\b(?:19|20)\d{2}\b/.test(sentence) || html.includes("year-chip");
  return figure && year;
}

/** A forbidden word, the term to write instead, and where it was found. */
export interface Violation {
  rule: "sentence" | "paragraph" | "dictionary" | "cluster" | "grade" | "empty" | "restates";
  where: string;
  detail: string;
}

/** One paragraph's sentence and paragraph lengths, against its limit. */
function lengths(html: Prose, where: string, limit: number): Violation[] {
  const out: Violation[] = [];
  const text = plain(html);
  const s = sentences(text);
  if (s.length > LIMITS.paragraphSentences) {
    out.push({ rule: "paragraph", where, detail: `${s.length} sentences` });
  }
  let longUsed = false;
  for (const sentence of s) {
    const n = words(sentence).length;
    if (n <= limit) continue;
    const licensed =
      !longUsed && n <= LIMITS.longSentenceWords && carriesDatedFigure(html, sentence);
    if (licensed) longUsed = true;
    else out.push({ rule: "sentence", where, detail: `${n} words: ${sentence}` });
  }
  return out;
}

/** Forbidden synonyms in a run of plain text. */
export function forbidden(text: string, slug: string): Violation[] {
  const out: Violation[] = [];
  for (const term of DICTIONARY) {
    if (term.allowedIn?.includes(slug)) continue;
    for (const pattern of term.not) {
      const hit = text.match(pattern);
      if (hit) out.push({ rule: "dictionary", where: "", detail: `"${hit[0]}": write "${term.write}"` });
    }
  }
  return out;
}

/** The share of `a`'s distinct words that `b` also uses, lower-cased, ignoring the function words. */
export function overlap(a: string, b: string): number {
  const bag = (text: string) =>
    new Set(words(text.toLowerCase()).filter((w) => !BREAKS.has(w) && w.length > 2));
  const left = bag(a);
  const right = bag(b);
  if (left.size === 0) return 0;
  return [...left].filter((w) => right.has(w)).length / left.size;
}

/** The plain sections of a topic, each paragraph named by where it sits. */
export function plainSections(topic: Topic): { where: string; html: Prose; step: boolean }[] {
  const n = topic.numbers;
  return [
    { where: "shortAnswer", html: `${topic.shortAnswer.lead} ${topic.shortAnswer.rest}`, step: false },
    { where: "picture", html: topic.picture, step: false },
    ...topic.breaks.map((html, i) => ({ where: `breaks[${i}]`, html, step: false })),
    { where: "numbers.finding", html: n.finding, step: false },
    ...n.intro.map((html, i) => ({ where: `numbers.intro[${i}]`, html, step: false })),
    ...n.steps.map((html, i) => ({ where: `numbers.steps[${i}]`, html, step: true })),
    ...(n.chart ? [{ where: "numbers.chart", html: n.chart.caption, step: false }] : []),
    ...n.after.map((html, i) => ({ where: `numbers.after[${i}]`, html, step: false })),
  ];
}

/**
 * Every violation of the guide in one topic: empty sections, lengths, the dictionary, noun
 * clusters, a picture restated as its own limits, and the grade ceiling once the pilot sets one.
 */
export function lint(topic: Topic, ceiling: number | undefined = GRADE_CEILING): Violation[] {
  const out: Violation[] = [];
  const empty = (where: string, value: string | readonly string[]) => {
    const all = typeof value === "string" ? [value] : value;
    if (all.length === 0 || all.some((p) => plain(p) === "")) {
      out.push({ rule: "empty", where, detail: "a required section is empty" });
    }
  };
  empty("shortAnswer.lead", topic.shortAnswer.lead);
  empty("shortAnswer.rest", topic.shortAnswer.rest);
  empty("picture", topic.picture);
  empty("breaks", topic.breaks);
  empty("numbers.finding", topic.numbers.finding);
  empty("numbers.intro", topic.numbers.intro);
  empty("numbers.steps", topic.numbers.steps);
  empty("numbers.after", topic.numbers.after);
  empty("rule.notation", topic.rule.notation);
  empty("rule.symbols", topic.rule.symbols);
  if (topic.howWeKnow.length === 0) out.push({ rule: "empty", where: "howWeKnow", detail: "no claim" });

  for (const { where, html, step } of plainSections(topic)) {
    const text = plain(html);
    out.push(...lengths(html, where, step ? LIMITS.stepWords : LIMITS.sentenceWords));
    out.push(...forbidden(text, topic.slug).map((v) => ({ ...v, where })));
    for (const run of nounClusters(text)) out.push({ rule: "cluster", where, detail: run });
  }

  const breaks = plain(topic.breaks.join(" "));
  const share = overlap(breaks, plain(topic.picture));
  if (share >= 0.5) {
    out.push({ rule: "restates", where: "breaks", detail: `${Math.round(share * 100)}% of its words are the picture's` });
  }

  if (ceiling !== undefined) {
    for (const [where, html] of [
      ["shortAnswer", `${topic.shortAnswer.lead} ${topic.shortAnswer.rest}`],
      ["picture", topic.picture],
      ["breaks", topic.breaks.join(" ")],
    ] as const) {
      const g = grade(plain(html));
      if (g > ceiling) out.push({ rule: "grade", where, detail: `grade ${g.toFixed(1)}` });
    }
  }
  return out;
}

/**
 * Digits typed into prose, in a topic module's source: what #702's `money(500)` was.
 *
 * Two shapes. A digit in the literal text of a string or template — interpolations removed, tags
 * removed, the statute and bill numbers on the allowlist removed. And a numeric literal passed
 * straight to a `format.ts` call, which is the same defect wearing a formatter: `money(500)` prints
 * a figure no feed computed. `formatters` is the list of `format.ts`'s exported names.
 *
 * Strings that are whole identifiers — a `class/slug` corpus reference, a figure key, an import
 * path — are not prose and are skipped.
 */
export function typedNumbers(source: string, formatters: readonly string[]): string[] {
  const found: string[] = [];
  const code = source.replace(/\/\*[\s\S]*?\*\//g, "").replace(/(^|[^:])\/\/.*$/gm, "$1");
  const call = new RegExp(`\\b(?:${formatters.join("|")})\\(\\s*-?\\d`, "g");
  for (const m of code.matchAll(call)) found.push(m[0]);
  for (const m of code.matchAll(/`(?:\\.|[^`\\])*`|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/g)) {
    const literal = m[0].slice(1, -1);
    if (/^[\w./@:-]*$/.test(literal)) continue;
    const text = stripInterpolations(literal)
      .replace(/<[^>]+>/g, " ")
      .replace(/\b(?:R\.C\.|section)\s*\d+(?:\.\d+)*[A-Za-z]?(?:\([A-Za-z0-9]+\))*/g, "")
      .replace(/\b(?:Am\.\s+)?(?:Sub\.\s+)?[HS]\.\s?B\.\s*\d+/g, "")
      .replace(/\[[A-Z]\d*\]/g, "");
    if (/\d/.test(text)) found.push(literal.length > 60 ? `${literal.slice(0, 60)}…` : literal);
  }
  return found;
}

/** A template literal's text with its `${…}` interpolations removed, braces balanced. */
function stripInterpolations(literal: string): string {
  let out = "";
  let depth = 0;
  for (let i = 0; i < literal.length; i++) {
    if (depth === 0 && literal[i] === "$" && literal[i + 1] === "{") {
      depth = 1;
      i++;
    } else if (depth > 0) {
      if (literal[i] === "{") depth++;
      else if (literal[i] === "}") depth--;
    } else out += literal[i];
  }
  return out;
}
