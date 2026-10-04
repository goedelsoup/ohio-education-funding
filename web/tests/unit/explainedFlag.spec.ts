/**
 * Explained is dev only until it ships (#718): a build without `PUBLIC_EXPLAINED=1` drops it whole.
 *
 * The unit suite runs with the flag on (`vitest.config.ts`), as `pnpm dev` does, so every other
 * spec reads the section. These stub it back off and ask each place Explained surfaces — routes,
 * bar, cards, the "What is this?" notes, the glossary and the wiki's "In plain terms" — for nothing.
 * `tests/dist/explainedFlag.spec.ts` asks the same of a built site.
 */

import { afterEach, expect, test, vi } from "vitest";

import { loadCorpus } from "../../src/lib/corpus.ts";
import { explainedOn } from "../../src/lib/explained/flag.ts";
import { TOPICS, published, topicsCiting, whatIsThis } from "../../src/lib/explained/registry.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import { nav } from "../../src/lib/nav.ts";
import { explainedCards, pageCards } from "../../src/lib/og/pages.ts";

afterEach(() => {
  vi.unstubAllEnvs();
});

const off = () => vi.stubEnv("PUBLIC_EXPLAINED", "0");

test("the flag is on only for exactly 1", () => {
  expect(explainedOn()).toBe(true);
  for (const value of ["0", "", "true", "yes"]) {
    vi.stubEnv("PUBLIC_EXPLAINED", value);
    expect(explainedOn(), JSON.stringify(value)).toBe(false);
  }
  vi.stubEnv("PUBLIC_EXPLAINED", undefined);
  expect(explainedOn()).toBe(false);
});

test("off, no topic is published, and every module is still registered and checked", () => {
  expect(published()).toEqual(TOPICS);
  off();
  expect(published()).toEqual([]);
  expect(TOPICS.length).toBeGreaterThan(0);
});

test("off, the bar has no Explained entry", () => {
  const { bundle } = loadFeed();
  const corpus = loadCorpus();
  expect(nav(bundle, corpus).map((e) => e.key)).toContain("explained");
  off();
  expect(nav(bundle, corpus).map((e) => e.key)).not.toContain("explained");
});

test("off, there is no index card and no topic card", () => {
  expect(Object.keys(explainedCards())).toHaveLength(TOPICS.length);
  off();
  expect(pageCards()).not.toHaveProperty("explained");
  expect(explainedCards()).toEqual({});
});

test("off, a data page's note is empty and a wiki node cites no topic, and a bad slug still throws", () => {
  const cited = "funding-regime/fair-school-funding-plan";
  expect(whatIsThis("guarantee")).toContain('href="/explained/guarantee"');
  expect(topicsCiting(cited).length).toBeGreaterThan(0);
  off();
  expect(whatIsThis("guarantee")).toBe("");
  expect(topicsCiting(cited)).toEqual([]);
  expect(() => whatIsThis("no-such-topic")).toThrow(/no-such-topic/);
});

/** The glossary's links, read fresh: it is a module constant, so the flag acts when it loads. */
async function glossaryLinks(): Promise<string[]> {
  vi.resetModules();
  const { GLOSSARY } = await import("../../src/lib/glossary.ts");
  return Object.values(GLOSSARY).flatMap((t) => (t.href ? [t.href] : []));
}

test("off, no glossary entry links a topic", async () => {
  const on = await glossaryLinks();
  expect(on.filter((h) => h.startsWith("/explained")).length).toBeGreaterThan(0);
  off();
  const hrefs = await glossaryLinks();
  expect(hrefs.length).toBeGreaterThan(0);
  expect(hrefs.filter((h) => h.startsWith("/explained"))).toEqual([]);
});
