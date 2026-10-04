/**
 * Explained (#712): the template every topic renders through, checked on a fixture topic.
 *
 * The registry is empty until the pilot (#715), so these tests render a topic written here. What
 * they hold is the template's contract — every section present, in order, addressable, and the
 * claim badges in "How we know" and nowhere else — so a topic module can rely on it rather than
 * re-check it.
 */

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
  GROUPS,
  QUESTION_LIMIT,
  description,
  plain,
  topic,
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
