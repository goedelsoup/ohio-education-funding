/**
 * Explained (#712): the template every topic renders through, checked on a fixture topic.
 *
 * The registry is empty until the pilot (#715), so these tests render a topic written here. What
 * they hold is the template's contract — every section present, in order, addressable, and the
 * claim badges in "How we know" and nowhere else — so a topic module can rely on it rather than
 * re-check it.
 */

import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";

import { parseHTML } from "linkedom";
import { describe, expect, test } from "vitest";

import { loadCorpus } from "../../src/lib/corpus.ts";
import {
  renderHowWeKnow,
  renderNumbers,
  renderPicture,
  renderReadNext,
  renderRule,
  renderShortAnswer,
  renderTopic,
} from "../../src/lib/explained/render.ts";
import { TOPICS, topicModule } from "../../src/lib/explained/registry.ts";
import {
  DICTIONARY,
  GRADE_CEILING,
  LIMITS,
  LOOSENINGS,
  forbidden,
  grade,
  lint,
  nounClusters,
  overlap,
  passives,
  plainSections,
  sentences,
  syllables,
  typedNumbers,
  words,
  type Violation,
} from "../../src/lib/explained/style.ts";
import {
  GROUPS,
  QUESTION_LIMIT,
  description,
  plain,
  reconciles,
  topic,
  type Identity,
  type Topic,
  type TopicModule,
} from "../../src/lib/explained/topic.ts";
import { SECTIONS } from "../../src/lib/routes.ts";

const FIXTURE: TopicModule = {
  slug: "fixture",
  group: GROUPS[0],
  question: "What does a fixture do?",
  build: () => ({
    shortAnswer: {
      lead: "A fixture stands in for a real page.",
      rest: "It fills every section so the template can be read without one.",
    },
    picture: "It is like a <em>model home</em>: every room is there, and nobody lives in it.",
    breaks: ["A model home is built to sell. A fixture is built to be checked."],
    numbers: {
      chip: "formula",
      intro: ["The example is the fixture itself."],
      steps: ["Start with the lead.", "Add the rest."],
      after: ["Together they are the description."],
    },
    rule: {
      notation: "<var>paid</var> = <var>base</var> + <var>p</var>",
      statute: "R.C. 3317.022",
      href: "https://codes.ohio.gov/ohio-revised-code/section-3317.022",
      symbols: ["<var>p</var> is the share phased in."],
    },
    howWeKnow: [
      {
        claim: "[verified] The phase-in percentage is set in the law each year.",
        nodes: ["parameter/fsfp-phase-in-percentage"],
        figures: ["example.key"],
      },
    ],
    readNext: {
      topics: [],
      data: { href: "/districts", label: "Find a district", note: "the same term, for one district" },
    },
  }),
};

const t = topic(FIXTURE);
const page = renderTopic(t);
const { document } = parseHTML(`<!doctype html><html><body>${page}</body></html>`);

describe("the template", () => {
  test("every section is present, addressable and in order", () => {
    const ids = [...document.querySelectorAll("[id]")].map((el) => el.id);
    const sections = Object.values(SECTIONS.explained);
    expect(ids.filter((id) => sections.includes(id as never))).toEqual(sections);
  });

  test("the page is its sections, joined, and nothing between them", () => {
    const parts = [
      renderShortAnswer,
      renderPicture,
      renderNumbers,
      renderRule,
      renderHowWeKnow,
      renderReadNext,
    ];
    expect(parts.map((part) => part(t)).join("")).toBe(page);
  });

  test("the short answer is the lead, before the first card", () => {
    const first = document.body.firstElementChild!;
    expect(first.id).toBe(SECTIONS.explained.shortAnswer);
    expect(first.className).toBe("lead");
    expect(renderShortAnswer(t)).toContain("<strong>A fixture stands in for a real page.</strong>");
  });

  test("the claim badge is drawn in How we know and nowhere else", () => {
    const howWeKnow = document.getElementById(SECTIONS.explained.howWeKnow)!;
    expect(howWeKnow.querySelector("span.claim.verified")?.textContent).toBe("verified");
    expect(document.querySelectorAll("span.claim")).toHaveLength(1);
    // A bracket in plain prose stays a bracket: the plain sections are not the corpus's register.
    const bracketed = { ...t, picture: "[verified] is a tag, not a badge, here." };
    expect(renderPicture(bracketed)).toContain("[verified] is a tag");
    expect(renderPicture(bracketed)).not.toContain('class="claim');
  });

  test("a cited node links to its wiki page by label", () => {
    const corpus = loadCorpus();
    const node = corpus.byId.get("parameter/fsfp-phase-in-percentage")!;
    const link = document.querySelector(`#${SECTIONS.explained.howWeKnow} li a`)!;
    expect(link.getAttribute("href")).toBe("/wiki/parameter/fsfp-phase-in-percentage");
    expect(link.textContent).toBe(node.label);
  });

  test("a topic citing a node the corpus lacks does not render", () => {
    const broken = { ...t, howWeKnow: [{ claim: "[verified] x", nodes: ["parameter/nope"] }] };
    expect(() => renderHowWeKnow(broken)).toThrow(/parameter\/nope/);
  });

  test("the steps are numbered and the year chip dates the example", () => {
    const numbers = document.getElementById(SECTIONS.explained.numbers)!;
    expect(numbers.querySelectorAll("ol > li")).toHaveLength(2);
    expect(numbers.querySelector("h2")!.innerHTML).toContain("year-chip");
  });
});

describe("the topic", () => {
  test("its description is the short answer as text", () => {
    expect(description(t)).toBe(
      "A fixture stands in for a real page. It fills every section so the template can be read without one.",
    );
    expect(plain("<em>a</em> , b &amp; c")).toBe("a, b & c");
  });

  test("every registered question is a question that fits a phone's line", () => {
    for (const m of [...TOPICS, FIXTURE]) {
      expect(m.question, m.slug).toMatch(/\?$/);
      expect(m.question.length, m.slug).toBeLessThanOrEqual(QUESTION_LIMIT);
    }
  });

  test("an unknown slug is an error, not a missing link", () => {
    expect(() => topicModule("no-such-topic")).toThrow(/no-such-topic/);
  });
});

/*
 * The style guide (#714). Every check is broken on purpose before it is trusted: the clean fixture
 * passes all of them, and each doctored copy below breaks one rule and must fail that rule and no
 * other. A check no doctored topic can fail is one every topic passes by construction.
 */

const rules = (violations: Violation[]) => [...new Set(violations.map((v) => v.rule))];

/** A sentence of `n` words, none of them a noun cluster. */
const sentenceOf = (n: number) =>
  `${Array.from({ length: n }, (_, i) => (i % 2 === 0 ? "the" : "district")).join(" ")}.`.replace(
    /^t/,
    "T",
  );

const DOCTORED: { rule: Violation["rule"]; why: string; doctor: (t: Topic) => Topic }[] = [
  {
    rule: "sentence",
    why: "a step over its word limit",
    doctor: (t) => ({ ...t, numbers: { ...t.numbers, steps: [sentenceOf(LIMITS.stepWords + 1)] } }),
  },
  {
    rule: "sentence",
    why: "a long sentence with no dated figure to license it",
    doctor: (t) => ({ ...t, picture: sentenceOf(LIMITS.sentenceWords + 4) }),
  },
  {
    rule: "paragraph",
    why: "a paragraph over its sentence limit",
    doctor: (t) => ({
      ...t,
      numbers: {
        ...t.numbers,
        after: [Array.from({ length: LIMITS.paragraphSentences + 1 }, () => "It adds up.").join(" ")],
      },
    }),
  },
  {
    rule: "dictionary",
    why: "a forbidden synonym",
    doctor: (t) => ({ ...t, breaks: ["A hold harmless amount is not a model home."] }),
  },
  {
    rule: "dictionary",
    why: '"funding" alone',
    doctor: (t) => ({ ...t, numbers: { ...t.numbers, after: ["The funding goes up."] } }),
  },
  {
    rule: "cluster",
    why: "five nouns in a row",
    doctor: (t) => ({ ...t, numbers: { ...t.numbers, intro: ["Read the base cost aid phase-in amount."] } }),
  },
  {
    rule: "empty",
    why: "a required section with nothing in it",
    doctor: (t) => ({ ...t, breaks: [] }),
  },
  {
    rule: "empty",
    why: "a section whose prose is only markup",
    doctor: (t) => ({ ...t, picture: "<em> </em>" }),
  },
  {
    rule: "empty",
    why: "no claim in How we know",
    doctor: (t) => ({ ...t, howWeKnow: [] }),
  },
  {
    rule: "restates",
    why: "where it breaks, restating the picture",
    doctor: (t) => ({ ...t, breaks: [t.picture] }),
  },
];

describe("the style guide", () => {
  test("the clean fixture passes every check", () => {
    expect(lint(t)).toEqual([]);
  });

  test("every registered topic passes every check", () => {
    for (const m of TOPICS) expect(lint(topic(m)), m.slug).toEqual([]);
  });

  test.each(DOCTORED)("a doctored topic fails $rule: $why", ({ rule, doctor }) => {
    expect(rules(lint(doctor(t)))).toEqual([rule]);
  });

  test("the grade ceiling bites once set, and the clean fixture is under any honest one", () => {
    expect(rules(lint(t, 0))).toEqual(["grade"]);
    expect(lint(t, 12)).toEqual([]);
    // The pilot sets it (#715). Until then it is unset, not guessed.
    if (GRADE_CEILING !== undefined) expect(GRADE_CEILING).toBeGreaterThan(0);
  });

  test("one long sentence per paragraph is allowed if it carries a figure with its unit and year", () => {
    const long = `In FY2024 the state paid this district $4,120,000, which is ${sentenceOf(20).toLowerCase()}`;
    expect(words(long).length).toBeGreaterThan(LIMITS.sentenceWords);
    expect(words(long).length).toBeLessThanOrEqual(LIMITS.longSentenceWords);
    expect(rules(lint({ ...t, picture: long }))).toEqual([]);
    expect(rules(lint({ ...t, picture: `${long} ${long}` }))).toContain("sentence");
  });

  test("school funding and the funding formula name the system and pass", () => {
    expect(forbidden("How school funding works, and the funding formula behind it.", "x")).toEqual([]);
    expect(forbidden("Their funding fell.", "x")).toHaveLength(1);
    expect(forbidden("A school system and an LEA.", "x")).toHaveLength(2);
    expect(forbidden("A phase-in, not a ramp or a transition.", "x")).toHaveLength(2);
    expect(forbidden("Transitional aid is a different word.", "x")).toEqual([]);
  });

  test("a term allowed in one topic is allowed there and nowhere else", () => {
    const term = DICTIONARY.find((d) => d.write === "local share")!;
    const allowed = { ...term, allowedIn: ["history"] };
    expect(allowed.not.some((p) => p.test("the charge-off"))).toBe(true);
    expect(DICTIONARY.every((d) => d.why.length > 0 && d.not.length > 0)).toBe(true);
    expect(LOOSENINGS.length).toBeGreaterThan(0);
  });

  test("sentences split at a full stop, and not at the site's abbreviations", () => {
    expect(sentences("R.C. 3317.022 sets it. H.B. 110 passed it in FY2022. Then it moved.")).toEqual([
      "R.C. 3317.022 sets it.",
      "H.B. 110 passed it in FY2022.",
      "Then it moved.",
    ]);
    expect(sentences("Is it? It is.")).toEqual(["Is it?", "It is."]);
    expect(words("The $1,234.50 aid, at 52.2%, isn't late.")).toEqual([
      "The",
      "$1,234.50",
      "aid",
      "at",
      "52.2%",
      "isn't",
      "late",
    ]);
  });

  test("syllables and grade read plain words as plain and long ones as long", () => {
    expect(["cat", "money", "district", "guarantee", "interpolation"].map(syllables)).toEqual([
      1, 2, 2, 3, 5,
    ]);
    expect(grade("The cat sat on the mat.")).toBeLessThan(3);
    expect(grade("Notwithstanding the aforementioned interpolation, apportionment remains indeterminate.")).toBeGreaterThan(14);
  });

  test("a noun cluster is four nouns with nothing between them", () => {
    expect(nounClusters("Base cost aid is paid.")).toEqual([]);
    expect(nounClusters("The base cost aid amount is paid.")).toEqual(["base cost aid amount"]);
    expect(nounClusters("Base, cost, aid and amount.")).toEqual([]);
  });

  test("the passive is reported, as a warning", () => {
    expect(passives("The share is set in the budget. The district pays.")).toEqual(["is set"]);
    for (const m of TOPICS) {
      const found = plainSections(topic(m)).flatMap(({ html }) => passives(plain(html)));
      if (found.length > 0) console.warn(`${m.slug} uses the passive: ${found.join("; ")}`);
    }
  });

  test("overlap counts the words two runs share, not their function words", () => {
    expect(overlap("the dial turns", "a dial turns")).toBe(1);
    expect(overlap("the dial turns", "a district moves")).toBe(0);
  });
});

describe("no number is typed into topic prose", () => {
  const dir = resolve(import.meta.dirname, "../../src/lib/explained");
  /** The section's machinery, which renders and checks prose and writes none. */
  const MACHINERY = new Set(["topic.ts", "registry.ts", "render.ts", "style.ts"]);
  const formatters = [
    ...readFileSync(resolve(import.meta.dirname, "../../src/lib/format.ts"), "utf8").matchAll(
      /^export function (\w+)/gm,
    ),
  ].map((m) => m[1]!);

  test("format.ts is read, so the formatter check has names to look for", () => {
    expect(formatters).toEqual(expect.arrayContaining(["money", "pct", "count", "millions"]));
  });

  test("no topic module types a digit into its prose", () => {
    for (const file of readdirSync(dir).filter((f) => f.endsWith(".ts") && !MACHINERY.has(f))) {
      expect(typedNumbers(readFileSync(resolve(dir, file), "utf8"), formatters), file).toEqual([]);
    }
  });

  test("the scan catches a typed figure, in text and in a formatter, and passes the allowlist", () => {
    const doctored = [
      "const a = `the district gets ${money(500)} more`;",
      "const b = `it moved 5 percent`;",
      'const c = "in FY2024 it moved";',
    ].join("\n");
    expect(typedNumbers(doctored, formatters)).toEqual([
      "money(5",
      "it moved 5 percent",
      "in FY2024 it moved",
    ]);
    const clean = [
      "const a = `the district gets ${money(row.aid)} under R.C. 3317.022(A)(1)`;",
      "const b = `H.B. 110 and Am. Sub. H.B. 33 set column [H2]`;",
      'const c = ["parameter/fsfp-phase-in-percentage", "phase_in.fy2024"];',
      "const d = rows[0]; // 500 in a comment is not prose",
      "/* nor 500 here */ const e = `${pct(share[1])}`;",
    ].join("\n");
    expect(typedNumbers(clean, formatters)).toEqual([]);
  });
});

describe("a topic's identity", () => {
  type Row = { irn: string; paid: number; base: number; computed: number; p: number };
  const PHASE_IN: Identity<Row> = {
    left: (r) => r.paid,
    right: (r) => r.base + r.p * (r.computed - r.base),
    tolerance: 0.5,
  };
  const rows: Row[] = [
    { irn: "000001", paid: 150, base: 100, computed: 200, p: 0.5 },
    { irn: "000002", paid: 175.4, base: 200, computed: 150, p: 0.5 },
  ];

  test("holds on every row that satisfies it", () => {
    expect(reconciles(PHASE_IN, rows)).toEqual([]);
  });

  test("names, by IRN, a doctored row that does not", () => {
    // The multiplier bug the FY2027 panel cannot see: a share of the new amount, not a weight.
    const doctored = { irn: "000003", paid: 100, base: 100, computed: 200, p: 0.5 };
    expect(reconciles(PHASE_IN, [...rows, doctored])).toEqual(["000003"]);
    expect(reconciles(PHASE_IN, [{ ...rows[0]!, paid: Number.NaN }])).toEqual(["000001"]);
  });
});
