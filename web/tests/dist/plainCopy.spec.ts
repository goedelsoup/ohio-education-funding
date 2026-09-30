/**
 * Public pages do not read like build output.
 *
 * # What this caught
 *
 * A journalist or legislator arriving from a shared link met repository paths, internal
 * identifiers and a contract version in the copy (#571). The footer, on every route, said a
 * forecast was "computed by crates/project", pointed at `.yidam/catalog/` and printed "Bundle
 * contract 48.0.0". The Taxes tab named a column, `effective_class1_millage_ty23`, and the crate
 * that names it; `/legislation` put two field names in code font; the Dashboard named
 * `denominators.ts`; the scenario card linked a draft by its file stem. Each was accurate, and
 * none of it was addressed to the person reading.
 *
 * # The rule
 *
 * Outside `/method`, `/data` and `/wiki/**` — where a technical reader goes looking for exactly
 * these, and where the corpus's own prose carries hundreds of crate paths a figure binding needs —
 * the visible text of a built page carries none of:
 *
 * - a repository path: `crates/`, `.yidam/`;
 * - a source file name: `*.ts`, `*.rs`, `*.yml`;
 * - a bare snake_case identifier, which is a field or column name;
 * - a file-stem slug — four or more hyphenated parts, one of them a number, like
 *   `hb-96-with-refreshed-inputs` — which is a node's file name used as text. The number is what
 *   tells a slug from English: "cost-of-doing-business" is a phrase.
 *
 * Page copy names the thing and links to the code instead — `routes.source`.
 *
 * # What it cannot see
 *
 * Text a script writes after load. The scenario runner's status line and its two failure cards
 * are client-rendered and were fixed by hand; a scan of `dist/` is blind to them.
 */

import { readFileSync } from "node:fs";
import { relative } from "node:path";

import { parseHTML } from "linkedom";
import { describe, expect, test } from "vitest";

import { DIST, pages } from "./artefact.ts";

/** The routes that may keep technical references, relative to `dist/`. */
const EXEMPT = /^(method\.html|data\.html|data\/|wiki\.html|wiki\/)/;

const PATTERNS: [string, RegExp][] = [
  ["a repository path", /(?:crates|\.yidam)\/\S*/g],
  ["a source file name", /\b[\w-]+\.(?:ts|rs|yml)\b/g],
  ["a snake_case identifier", /\b[a-z][a-z0-9]*(?:_[a-z0-9]+)+\b/g],
  ["a file-stem slug", /\b[a-z]+(?=(?:-[a-z0-9]+){3,}\b)(?:-[a-z0-9]+)*?-\d+(?:-[a-z0-9]+)+\b/g],
];

/** The page's visible text: the title and the body, without scripts, styles or templates. */
function visibleText(file: string): string {
  const { document } = parseHTML(readFileSync(file, "utf8"));
  for (const node of document.querySelectorAll("script, style, template")) node.remove();
  const title = document.querySelector("title")?.textContent ?? "";
  return `${title}\n${document.body?.textContent ?? ""}`.replace(/\s+/g, " ");
}

describe("public copy is addressed to a reader", () => {
  const scanned = pages().filter((file) => !EXEMPT.test(relative(DIST, file)));

  test("the scan reaches the routes it is about", () => {
    // Guard against the exemption swallowing everything: every district tab is in scope.
    expect(scanned.length).toBeGreaterThan(2000);
    const names = new Set(scanned.map((file) => relative(DIST, file)));
    for (const route of ["index.html", "404.html", "scenario.html", "legislation.html", "bounds.html"]) {
      expect(names.has(route), route).toBe(true);
    }
  });

  test("the guard fires on the defect it exists for", () => {
    // A doctored line from the footer this replaced: a scan that passes on every page measures
    // nothing unless it can be shown to fail on one.
    const old = "reference forecasts computed by crates/project. Bundle contract 48.0.0; see .yidam/catalog/";
    expect(PATTERNS.some(([, re]) => new RegExp(re.source).test(old))).toBe(true);
    expect(PATTERNS.some(([, re]) => new RegExp(re.source).test("column effective_class1_millage_ty23"))).toBe(true);
    expect(PATTERNS.some(([, re]) => new RegExp(re.source).test("the refresh hb-96-with-refreshed-inputs prices"))).toBe(true);
  });

  test("no page outside /method, /data and /wiki prints build output", () => {
    // Collected by match, so a string repeated on 609 district pages is reported once.
    const found = new Map<string, { kind: string; page: string; pages: number }>();
    for (const file of scanned) {
      const text = visibleText(file);
      for (const [kind, re] of PATTERNS) {
        for (const match of text.matchAll(re)) {
          const at = match.index ?? 0;
          const key = `${kind}: "${match[0]}" in "…${text.slice(Math.max(0, at - 50), at + 50)}…"`;
          const seen = found.get(key);
          if (seen) seen.pages += 1;
          else found.set(key, { kind, page: relative(DIST, file), pages: 1 });
        }
      }
    }
    const report = [...found].map(([key, { page, pages }]) => `${key} — ${page}, ${pages} page(s)`);
    expect(report).toEqual([]);
  });
});
