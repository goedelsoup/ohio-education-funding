/**
 * The build carries Explained exactly when it was built to (#718).
 *
 * Read off the same variable the build read, so this suite has to be run with the shell the build
 * ran in: `pnpm build && pnpm test:dist` for the build that deploys, and both under
 * `PUBLIC_EXPLAINED=1` for the `web explained` job. A build and a suite that disagree fail here
 * rather than letting a hidden section pass for an absent one.
 *
 * Off, nothing reaches the section: no page, no card, no sitemap entry and no link anywhere in any
 * document — the bar included, which is why this reads the whole file and not only `main`.
 */

import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

import { expect, test } from "vitest";

import { explainedOn } from "../../src/lib/explained/flag.ts";
import { TOPICS } from "../../src/lib/explained/registry.ts";
import { DIST, pages } from "./artefact.ts";

const on = explainedOn();

test.runIf(!on)("an off build has no Explained page, card or sitemap entry", () => {
  expect(existsSync(join(DIST, "explained.html"))).toBe(false);
  expect(existsSync(join(DIST, "explained"))).toBe(false);
  expect(existsSync(join(DIST, "og/explained"))).toBe(false);
  expect(existsSync(join(DIST, "og/page/explained.png"))).toBe(false);
  const sitemaps = readdirSync(DIST).filter((f) => f.startsWith("sitemap") && f.endsWith(".xml"));
  expect(sitemaps.length).toBeGreaterThan(0);
  // The route, not the word: the corpus has a decision whose slug starts `explained-`.
  const route = /<loc>https?:\/\/[^/<]+\/explained(?:[/.]|<)/;
  expect(sitemaps.filter((f) => route.test(readFileSync(join(DIST, f), "utf8")))).toEqual([]);
});

test.runIf(!on)("an off build links Explained from no document", () => {
  const linking = pages().filter((file) => /(?:href|src|content)="[^"]*\/explained[/".#]/.test(readFileSync(file, "utf8")));
  expect(linking).toEqual([]);
});

test.runIf(on)("an on build has the index and every topic", () => {
  const missing = ["explained.html", ...TOPICS.map((t) => `explained/${t.slug}.html`)].filter(
    (file) => !existsSync(join(DIST, file)),
  );
  expect(missing).toEqual([]);
});
