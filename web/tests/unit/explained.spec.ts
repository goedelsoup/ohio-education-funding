/**
 * Explained (#712): the template every topic renders through, checked on a fixture topic.
 *
 * The template tests render a topic written here rather than a registered one, so that a topic's
 * prose changing cannot move them. What they hold is the template's contract — every section present, in order, addressable, and the
 * claim badges in "How we know" and nowhere else — so a topic module can rely on it rather than
 * re-check it.
 */

import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";

import { parseHTML } from "linkedom";
import { describe, expect, test } from "vitest";

import { loadCorpus } from "../../src/lib/corpus.ts";
import { loadFigureManifest } from "../../src/lib/corpusFigures.ts";
import {
  IDENTITY as GUARANTEE_IDENTITY,
  ORIGIN_YEAR,
  computed as foundationAid,
  example as medianByGuarantee,
  floorOf,
} from "../../src/lib/explained/guarantee.ts";
import {
  BASE_YEAR,
  PHASE_IN as PHASE_IN_TOPIC,
  example,
  fy2026PhaseIn,
  identity,
  sides,
} from "../../src/lib/explained/phase-in.ts";
import {
  renderHowWeKnow,
  renderNumbers,
  renderPicture,
  renderReadNext,
  renderRule,
  renderShortAnswer,
  renderTopic,
} from "../../src/lib/explained/render.ts";
import {
  identity as shareRule,
  example as medianMover,
  reach,
  shareOf,
} from "../../src/lib/explained/state-share.ts";
import { CHAIN, EMPTYING_FLOOR, EMPTYING_LOSS, inflation } from "../../src/lib/explained/anchor-chain.ts";
import { IDENTITY as DEDUCTION_IDENTITY, example as deductionExample, terminal, transfers } from "../../src/lib/explained/deduction.ts";
import {
  IDENTITY as POVERTY_IDENTITY,
  blend,
  breaks,
  example as medianPoverty,
} from "../../src/lib/explained/poverty-count.ts";
import { TOPICS, topicModule } from "../../src/lib/explained/registry.ts";
import {
  FLOOR,
  IDENTITY as LEVY_IDENTITY,
  example as medianReduction,
  rows as levies,
} from "../../src/lib/explained/reduction-factors.ts";
import {
  BREAK_YEAR,
  IDENTITY as SHARES_IDENTITY,
  reliefPeak,
  usualFederal,
  years,
} from "../../src/lib/explained/sources.ts";
import { example as medianSpender, measure, rows } from "../../src/lib/explained/spending.ts";
import {
  CATEGORICALS,
  example as medianByAid,
  shareIdentity,
  statePerPupil,
  sumIdentity,
} from "../../src/lib/explained/what-a-district-gets.ts";
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
import { loadFeed } from "../../src/lib/feed.ts";
import { fiscalYear } from "../../src/lib/yearLabel.ts";
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
      finding: "The fixture has two steps.",
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

  // Without the grade ceiling: a 30-word sentence reads above grade 8 on its own, and these
  // isolate the rule each one doctors. The ceiling has its own test below.
  test.each(DOCTORED)("a doctored topic fails $rule: $why", ({ rule, doctor }) => {
    expect(rules(lint(doctor(t), Infinity))).toEqual([rule]);
  });

  test("the grade ceiling bites once set, and the clean fixture is under any honest one", () => {
    expect(rules(lint(t, 0))).toEqual(["grade"]);
    expect(lint(t, 12)).toEqual([]);
    // Set from the pilot (#715), and it bites at its own value, not only at zero.
    expect(GRADE_CEILING).toBe(8);
    const dense = {
      ...t,
      picture:
        "Notwithstanding considerable administrative complexity, apportionment institutionalizes " +
        "intergovernmental redistribution through categorical calculations.",
    };
    expect(grade(plain(dense.picture))).toBeGreaterThan(GRADE_CEILING!);
    expect(rules(lint(dense))).toContain("grade");
  });

  test("one long sentence per paragraph is allowed if it carries a figure with its unit and year", () => {
    const long = `In FY2024 the state paid this district $4,120,000, which is ${sentenceOf(20).toLowerCase()}`;
    expect(words(long).length).toBeGreaterThan(LIMITS.sentenceWords);
    expect(words(long).length).toBeLessThanOrEqual(LIMITS.longSentenceWords);
    expect(rules(lint({ ...t, picture: long }, Infinity))).toEqual([]);
    expect(rules(lint({ ...t, picture: `${long} ${long}` }, Infinity))).toContain("sentence");
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

describe("What does the phase-in do? (#715)", () => {
  const districts = loadFeed().bundle.districts;
  const { year, p } = fy2026PhaseIn(districts);

  test("the dial is read in the biennium's middle year, where it is short of the end", () => {
    expect(year).toBe(districts[0]!.biennium.year_middle);
    // At 100% the interpolation and a multiplier pay the same figure, and the check below proves nothing.
    expect(p).toBeGreaterThan(0);
    expect(p).toBeLessThan(1);
  });

  test("paid = base + p × (computed − base) holds on every district, to the cent", () => {
    expect(reconciles(identity(p), districts)).toEqual([]);
  });

  test("a district paid p × computed fails it, by IRN", () => {
    const real = example(districts);
    const o = real.biennium.observed[1]!;
    const doctored = {
      ...real,
      irn: "999999",
      biennium: {
        ...real.biennium,
        observed: real.biennium.observed.map((row, i) =>
          i === 1 ? { ...row, phase_in_paid: p * o.phase_in_calculated } : row,
        ) as typeof real.biennium.observed,
      },
    };
    expect(reconciles(identity(p), [...districts, doctored])).toEqual(["999999"]);
  });

  test("the base year is the corpus's", () => {
    const node = loadCorpus().byId.get("parameter/guarantee-funding-base")!;
    expect(node.summary).toContain(`FY${BASE_YEAR}`);
  });

  test("the example is the median district by step, and moves with the feed", () => {
    const ex = example(districts);
    const step = (d: (typeof districts)[number]) =>
      d.biennium.observed[1]!.phase_in_calculated - d.biennium.observed[1]!.funding_base;
    const smaller = districts.filter((d) => step(d) < step(ex)).length;
    expect(smaller).toBe(Math.floor(districts.length / 2));
  });

  test("every district is on one side of its base, and the guarantee holds those below", () => {
    const s = sides(districts);
    expect(s.above.length + s.below.length).toBe(districts.length);
    expect(s.held.length).toBeLessThanOrEqual(s.below.length);
    expect(s.restored).toBeGreaterThan(0);
  });

  test("its description fits a search result", () => {
    for (const m of [...TOPICS, PHASE_IN_TOPIC]) expect(description(topic(m)).length, m.slug).toBeLessThanOrEqual(160);
  });
});

describe("How does Ohio decide what a district gets? (#716)", () => {
  const feed = loadFeed().bundle;
  const districts = feed.districts;
  const floor = feed.statewide.minimum_state_share;

  test("the state share of base cost per pupil is max(B − L, m × B), to the cent, on every district", () => {
    expect(reconciles(shareIdentity(floor), districts)).toEqual([]);
  });

  test("a floor district paid B − L, as if there were no floor, fails it", () => {
    const real = districts.find((d) => d.at_minimum_state_share)!;
    const unfloored = Math.max(real.base_cost_per_pupil - real.regime!.local_capacity!, 0);
    const doctored = { ...real, irn: "999999", base_cost_state_share: unfloored * real.categorical_adm };
    expect(reconciles(shareIdentity(floor), [...districts, doctored])).toEqual(["999999"]);
  });

  test("the floor is the one in force, and it binds exactly where the feed says", () => {
    expect(floor).toBeGreaterThan(0);
    for (const d of districts) {
      const atFloor = statePerPupil(d, floor) === floor * d.base_cost_per_pupil;
      expect(atFloor, d.irn).toBe(d.at_minimum_state_share);
    }
  });

  test("the state share plus the six categoricals is the computed amount, on every district", () => {
    expect(reconciles(sumIdentity, districts)).toEqual([]);
  });

  test("a district missing one categorical fails the sum", () => {
    const real = medianByAid(districts);
    const doctored = {
      ...real,
      irn: "999999",
      categoricals: { ...real.categoricals, gifted: 0, targeted_assistance: 0 },
    };
    expect(reconciles(sumIdentity, [...districts, doctored])).toEqual(["999999"]);
  });

  test("the six are the feed's six, so a seventh cannot go unsummed", () => {
    expect(CATEGORICALS.map(([key]) => key).sort()).toEqual(Object.keys(districts[0]!.categoricals).sort());
  });

  test("the example is the median district by computed aid per pupil", () => {
    const ex = medianByAid(districts);
    const perPupil = (d: (typeof districts)[number]) =>
      d.biennium.observed[2]!.phase_in_calculated / d.categorical_adm;
    const smaller = districts.filter((d) => perPupil(d) < perPupil(ex)).length;
    expect(smaller).toBe(Math.floor(districts.length / 2));
  });
});

describe("Why are so many districts paid an old amount? (#716)", () => {
  const districts = loadFeed().bundle.districts;

  test("[I] = max([H2] − [I1] − [H], 0) holds on every district, to the cent", () => {
    expect(reconciles(GUARANTEE_IDENTITY, districts)).toEqual([]);
  });

  test("a guarantee computed without the open enrollment charge fails it", () => {
    const real = districts.find((d) => d.on_guarantee && d.transition.open_enrollment_adjustment > 0)!;
    const doctored = {
      ...real,
      irn: "999999",
      guarantee: Math.max(real.transition.funding_base - foundationAid(real), 0),
    };
    expect(reconciles(GUARANTEE_IDENTITY, [...districts, doctored])).toEqual(["999999"]);
  });

  test("the rebuilt floor is the feed's, and the guarantee pays exactly the districts flagged", () => {
    for (const d of districts) {
      expect(floorOf(d), d.irn).toBeCloseTo(d.guarantee_floor, 2);
      expect(d.guarantee > 0, d.irn).toBe(d.on_guarantee);
    }
  });

  test("the chain's first year is the corpus's", () => {
    const node = loadCorpus().byId.get("parameter/guarantee-funding-base")!;
    expect(node.findings).toContain(`bottoms out at FY${ORIGIN_YEAR}`);
  });

  test("the example is the median guaranteed district by the guarantee's share of what it receives", () => {
    const ex = medianByGuarantee(districts);
    const held = districts.filter((d) => d.on_guarantee);
    const share = (d: (typeof districts)[number]) => d.guarantee / (foundationAid(d) + d.guarantee);
    expect(ex.on_guarantee).toBe(true);
    // An even count averages the two middle shares, so the nearest district is either one.
    const below = held.filter((d) => share(d) < share(ex)).length;
    expect([Math.floor((held.length - 1) / 2), Math.floor(held.length / 2)]).toContain(below);
  });
});

describe("Why do so many districts sit on the state's floor? (#716)", () => {
  const bundle = loadFeed().bundle;
  const districts = bundle.districts;
  const floor = bundle.statewide.minimum_state_share;
  const r = reach(bundle);

  test("the published share is max(1 − L/B, m) on every district", () => {
    expect(reconciles(shareRule(floor), districts)).toEqual([]);
  });

  test("a floor district published at its unfloored share fails it", () => {
    const real = districts.find((d) => d.at_minimum_state_share)!;
    const unfloored = 1 - real.regime!.local_capacity! / real.base_cost_per_pupil;
    const doctored = {
      ...real,
      irn: "999999",
      biennium: {
        ...real.biennium,
        observed: real.biennium.observed.map((row, i) =>
          i === 2 ? { ...row, state_share: unfloored } : row,
        ) as typeof real.biennium.observed,
      },
    };
    expect(reconciles(shareRule(floor), [...districts, doctored])).toEqual(["999999"]);
  });

  test("the floor binds exactly where the feed says, and the bound figure counts the same districts", () => {
    for (const d of districts) expect(shareOf(d, floor) === floor, d.irn).toBe(d.at_minimum_state_share);
    const figure = loadFigureManifest().figures.find((f) => f.key === "project/districts-at-the-minimum-state-share")!;
    expect(r.onFloor.length).toBe(figure.value);
  });

  test("the lower floor reaches only districts on the floor, and every one it misses is on the guarantee", () => {
    expect(r.low).toBeLessThan(floor);
    expect(r.moved.length).toBeGreaterThan(0);
    expect(r.moved.every((o) => o.atMinimumStateShare && o.delta < 0)).toBe(true);
    expect(r.moved.length + r.heldByGuarantee.length).toBe(r.onFloor.length);
    expect(r.saved).toBeLessThan(r.naive);
  });

  test("the example is the median mover by its loss, and not on the guarantee", () => {
    const ex = medianMover(r.moved);
    expect(ex.onGuarantee).toBe(false);
    const below = r.moved.filter((o) => o.delta < ex.delta).length;
    expect([Math.floor((r.moved.length - 1) / 2), Math.floor(r.moved.length / 2)]).toContain(below);
  });
});

describe("Does spending more raise test scores? (#716)", () => {
  const bundle = loadFeed().bundle;
  const districts = bundle.districts;
  const published = bundle.statewide.outcomes!;
  const weighted = measure(districts, "weighted");
  const enrolled = measure(districts, "enrolled");

  // The feed prints each coefficient to four places, so half the last one.
  const PRINTED = 5e-5;

  test("the pooled coefficients are the feed's, on both denominators", () => {
    expect(Math.abs(weighted.pooled - published.weighted_spending_vs_performance)).toBeLessThan(PRINTED);
    expect(Math.abs(enrolled.pooled - published.enrolled_spending_vs_performance)).toBeLessThan(PRINTED);
  });

  test("the enrolled coefficient is the corpus's bound figure, which counts the same districts", () => {
    const figure = loadFigureManifest().figures.find(
      (f) => f.key === "dispersion/headcount-estimate-negative-excluding-top-spender",
    )!;
    expect(enrolled.pooled).toBeLessThan(0);
    expect(Math.abs(Math.abs(enrolled.pooled) - figure.value)).toBeLessThan(1e-9);
  });

  test("one district's spending doctored moves the pooled coefficient off the feed's", () => {
    const real = rows(districts)[0]!.d;
    const doctored = districts.map((d) =>
      d.irn === real.irn ? { ...d, outcome: { ...d.outcome!, per_enrolled_pupil: d.outcome!.per_enrolled_pupil! * 3 } } : d,
    );
    const moved = measure(doctored, "enrolled").pooled;
    expect(Math.abs(moved - published.enrolled_spending_vs_performance)).toBeGreaterThan(PRINTED);
  });

  test("the near zero is a cancellation: the least poor third rises, the poorest falls, and the pool is weaker than any third", () => {
    const [rich, middle, poor] = weighted.thirds;
    expect(rich!).toBeGreaterThan(0);
    expect(poor!).toBeLessThan(0);
    for (const t of [rich!, middle!, poor!]) expect(Math.abs(weighted.pooled)).toBeLessThan(Math.abs(t));
    expect(rows(districts).length).toBe(published.districts);
  });

  test("the example is the median district by spending per enrolled pupil", () => {
    const ex = medianSpender(districts);
    const all = rows(districts);
    const spend = (d: (typeof all)[number]["d"]) => d.outcome!.per_enrolled_pupil!;
    const below = all.filter((x) => spend(x.d) < spend(ex.d)).length;
    expect([Math.floor((all.length - 1) / 2), Math.floor(all.length / 2)]).toContain(below);
  });
});

describe("Where does school money come from? (#716)", () => {
  const history = loadFeed().bundle.history;

  test("the three shares add to the whole in every year of the panel", () => {
    expect(reconciles(SHARES_IDENTITY, years(history))).toEqual([]);
  });

  test("a year with its federal share doubled fails it", () => {
    const real = history.at(-1)!;
    const doctored = { ...real, fiscal_year: 1900, federal_share: real.federal_share * 2 };
    expect(reconciles(SHARES_IDENTITY, years([...history, doctored]))).toEqual(["1900"]);
  });

  test("local money is the largest part in every year, as the page says", () => {
    const not = history.filter((y) => !(y.local_share > y.state_share && y.local_share > y.federal_share));
    expect(not.map((y) => y.fiscal_year)).toEqual([]);
  });

  test("the relief peak is about twice the federal share's usual size", () => {
    const { times } = reliefPeak(history);
    expect(times).toBeGreaterThan(1.75);
    expect(times).toBeLessThan(2.25);
    expect(usualFederal(history)).toBeGreaterThan(0);
  });

  test("the break year is the one the survey's catalog entry names", () => {
    const catalog = readFileSync(
      resolve(import.meta.dirname, "../../../.yidam/catalog/census-f33-school-system-finances.md"),
      "utf8",
    );
    expect(catalog).toContain(`changes definition at ${fiscalYear(BREAK_YEAR)}`);
  });
});

describe("Why don't my taxes rise when my house value does? (#718)", () => {
  const districts = loadFeed().bundle.districts;
  const all = levies(districts);

  test("effective = voted × (1 − r) holds on every district carrying the three", () => {
    expect(all.length).toBeGreaterThan(600);
    expect(reconciles(LEVY_IDENTITY, all)).toEqual([]);
  });

  test("a district whose reduction is read off the following year's rate fails it", () => {
    // The reduction is measured against the year before `millage.tax_year`; one recomputed from the
    // observed year is the slip the identity exists to catch, on a district whose rate moved.
    const real = all.find((d) => Math.abs(d.millage.observed_rate - d.millage.prior_rate) > 0.5)!;
    const doctored = {
      ...real,
      irn: "999999",
      millage: { ...real.millage, cumulative_reduction: 1 - real.millage.observed_rate / real.voted_operating_millage },
    };
    expect(reconciles(LEVY_IDENTITY, [...all, doctored])).toEqual(["999999"]);
  });

  test("the floor is the corpus's", () => {
    const node = loadCorpus().byId.get("parameter/twenty-mill-floor")!;
    expect(node.description).toContain(`Set at ${FLOOR} mills for school`);
  });

  test("the example is above the floor, at the median reduction among districts that are", () => {
    const ex = medianReduction(districts);
    const above = all.filter((d) => d.millage.prior_rate > FLOOR + 0.005);
    const r = (d: (typeof all)[number]) => d.millage.cumulative_reduction;
    expect(ex.millage.prior_rate).toBeGreaterThan(FLOOR);
    const below = above.filter((d) => r(d) < r(ex)).length;
    const tied = above.filter((d) => r(d) === r(ex)).length;
    const mid = (above.length - 1) / 2;
    expect(below).toBeLessThanOrEqual(Math.ceil(mid));
    expect(below + tied - 1).toBeGreaterThanOrEqual(Math.floor(mid));
  });
});

describe("How does Ohio count poor students? (#718)", () => {
  const { districts, meal_program: meal } = loadFeed().bundle;

  test("the capped blend is the department's weighted count on every district", () => {
    expect(districts.length).toBeGreaterThan(600);
    expect(reconciles(POVERTY_IDENTITY, districts)).toEqual([]);
  });

  test("a district counted on the uncapped blend fails it", () => {
    // Edgerton Local is the corpus's identification of the cap: the one district whose blend
    // exceeds its enrollment. Without the cap its count is the blend, and the identity must see it.
    const real = districts.find((d) => blend(d) > d.current_year_adm)!;
    expect(real.name).toContain("Edgerton");
    const doctored = { ...real, irn: "999999", dpia: { ...real.dpia, weighted_adm: blend(real) } };
    expect(reconciles(POVERTY_IDENTITY, [...districts, doctored])).toEqual(["999999"]);
  });

  test("the blend's weight is the corpus's", () => {
    const node = loadCorpus().byId.get("formula-component/fsfp-disadvantaged-pupil-impact-aid")!;
    const w = Math.round(blend({ dpia: { economically_disadvantaged_adm: 0, directly_certified_adm: 100 } } as never));
    expect(node.description).toContain(`${100 - w}/${w} blend is in the act`);
  });

  test("the waiver years are the Octobers of 2020 and 2021, keyed on their fiscal years (#705)", () => {
    const { split, waived, after } = breaks(meal);
    expect(waived.map((y) => fiscalYear(y.fiscal_year))).toEqual([fiscalYear(2021), fiscalYear(2022)]);
    expect(after.fiscal_year).toBe(2023);
    expect(split.fiscal_year).toBe(2013);
  });

  test("the example is the median share", () => {
    const ex = medianPoverty(districts);
    const p = (d: (typeof districts)[number]) => d.dpia.percentage;
    const below = districts.filter((d) => p(d) < p(ex)).length;
    const tied = districts.filter((d) => p(d) === p(ex)).length;
    const mid = (districts.length - 1) / 2;
    expect(below).toBeLessThanOrEqual(Math.ceil(mid));
    expect(below + tied - 1).toBeGreaterThanOrEqual(Math.floor(mid));
  });
});

describe("Why is FY2011 still inside the formula? (#718)", () => {
  const corpus = loadCorpus();
  const chain = corpus.byId.get("funding-regime/bridge-formula")!.findings!;

  test("every link is the corpus's: the act, and the year it set the floor at", () => {
    const [first, ...rest] = CHAIN;
    expect(chain).toMatch(new RegExp(`FY${first.year}\\s+Evidence-Based Model run[\\s\\S]*?↓\\s+${first.act}:`));
    for (const link of rest) expect(chain, link.act).toMatch(new RegExp(`FY${link.year}\\s+${link.act}:`));
  });

  test("H.B. 49's exception is the corpus's", () => {
    const pc = (v: number) => `${Math.round(v * 100)}%`;
    expect(chain).toContain(`except ${pc(EMPTYING_FLOOR)} for districts whose ADM fell ${pc(EMPTYING_LOSS)} or`);
  });

  test("the chain starts where the guarantee topic says it does, and ends at the phase-in's base", () => {
    expect(CHAIN[0].year).toBe(ORIGIN_YEAR);
    expect(CHAIN.at(-1)!.year).toBe(BASE_YEAR);
  });

  test("the price rise is measured from the base year to the deflator's last", () => {
    const deflator = loadFeed().bundle.deflator!;
    const { last, rise } = inflation(deflator);
    const at = (y: number) => deflator.points.find((p) => p.fiscal_year === y)!.index;
    expect(last).toBeGreaterThan(BASE_YEAR);
    expect(rise).toBeCloseTo(at(last) / at(BASE_YEAR) - 1, 12);
    expect(rise).toBeGreaterThan(0);
  });
});

describe("What happened to the deduction for community schools and vouchers? (#718)", () => {
  const { districts } = loadFeed().bundle;
  const corpus = loadCorpus();

  test("net state funding is total state support plus the two transfer lines on every district", () => {
    expect(districts.length).toBeGreaterThan(600);
    expect(reconciles(DEDUCTION_IDENTITY, districts)).toEqual([]);
  });

  test("a payment with a third line taken off fails it", () => {
    // A deduction would be a charge between total state support and the net payment that neither
    // transfer line carries. Two cents is the smallest the three cent figures cannot absorb.
    const real = deductionExample(districts);
    const o = terminal(real);
    const doctored = {
      ...real,
      irn: "999999",
      biennium: {
        ...real.biennium,
        observed: [
          real.biennium.observed[0]!,
          real.biennium.observed[1]!,
          { ...o, net_state_funding: o.net_state_funding! - 0.02 },
        ],
      },
    };
    expect(reconciles(DEDUCTION_IDENTITY, [...districts, doctored])).toEqual(["999999"]);
  });

  test("the two lines are the payment report's own transfers, to the cent", () => {
    for (const d of districts) expect(Math.abs(transfers(d) - terminal(d).transfers!), d.irn).toBeLessThan(0.01);
  });

  test("the example is the district the state pays the most", () => {
    const ex = deductionExample(districts);
    for (const d of districts) expect(d.biennium.total_terminal).toBeLessThanOrEqual(ex.biennium.total_terminal);
  });

  test("the one negative guarantee base is the corpus's", () => {
    const below = districts.filter((d) => d.transition.funding_base < 0);
    expect(below).toHaveLength(1);
    const text = corpus.byId.get("parameter/guarantee-funding-base")!.findings!;
    const dollars = Math.abs(below[0]!.transition.funding_base).toLocaleString("en-US", { minimumFractionDigits: 2 });
    expect(text).toContain(`${below[0]!.name} at **-$${dollars}**`);
  });

  test("the corpus says community school students are paid directly, not deducted", () => {
    const plan = corpus.byId.get("funding-regime/fair-school-funding-plan")!;
    expect(plan.description).toMatch(/funded directly by the state rather than by deducting from a resident district's\s+foundation payment/);
  });
});
