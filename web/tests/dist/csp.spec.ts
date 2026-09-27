/**
 * Nothing in the build is a thing `script-src 'self'` would refuse to run.
 *
 * # Why this reads files instead of driving a browser
 *
 * `vite preview` serves `dist/` with no headers at all, so the CSP that the host applies is absent
 * for the entire e2e suite. Everything there could pass while the deployed site refuses to run its
 * own scripts — and that is not hypothetical: `onsubmit="return false"` shipped on four pages and
 * was found by opening the live site, after the tests were green.
 *
 * `script-src 'self'` permits no inline script of any kind, and an `on*` attribute is inline
 * script. So rather than test the browser, this tests the artefact: whatever the headers do, the
 * output must contain nothing the strict directive would reject.
 *
 * The directives themselves are in `public/_headers`, and that file's own presence in the build is
 * checked in `machines.spec.ts` beside the other two files no reader ever opens.
 */

import { readFileSync } from "node:fs";

import { describe, expect, test } from "vitest";

import { DIST, pages } from "./artefact.ts";

describe("the content security policy", () => {


  test("no page carries an inline event handler", () => {
    // `onsubmit`, `onclick`, and friends. Excludes SVG's `on` in other positions by requiring the
    // attribute form exactly.
    const offenders: string[] = [];
    for (const file of pages()) {
      const found = readFileSync(file, "utf8").match(/\son[a-z]+\s*=\s*["']/gi);
      if (found) offenders.push(`${file.slice(DIST.length + 1)}: ${[...new Set(found)].join(", ")}`);
    }
    expect(offenders.slice(0, 10)).toEqual([]);
  });



  test("no page carries an inline script block", () => {
    // Astro bundles every `<script>` to a hashed file under `_astro/`, so any `<script>` left with
    // a body is either a mistake or a deliberate exception that the CSP has not been told about.
    //
    // `application/ld+json` is the one exception, and it is an exception to this test rather than
    // to the directive. `script-src` governs elements the browser would *execute*; an element
    // whose type is not a JavaScript MIME type is a data block, never parsed as script and never
    // blocked. `/data` carries one — a `schema.org/Dataset` naming the three downloads, which is
    // the one page on this site where the vocabulary does work prose cannot. It has to be inline
    // because a data block has no `src`.
    //
    // Every `<` in it is written as `<`, so the JSON cannot close its own element early
    // whatever a description string turns out to contain.
    const offenders: string[] = [];
    for (const file of pages()) {
      const body = readFileSync(file, "utf8");
      for (const match of body.matchAll(/<script(?![^>]*\bsrc=)([^>]*)>([\s\S]*?)<\/script>/gi)) {
        if (/type\s*=\s*["']application\/ld\+json["']/i.test(match[1] ?? "")) continue;
        if ((match[2] ?? "").trim() !== "") offenders.push(file.slice(DIST.length + 1));
      }
    }
    expect([...new Set(offenders)].slice(0, 10)).toEqual([]);
  });



  test("nothing is fetched from another origin", () => {
    // `default-src 'self'` and `connect-src 'self'`. A CDN font or script would be invisible in
    // preview and dead in production.
    const offenders: string[] = [];
    for (const file of pages()) {
      const found = readFileSync(file, "utf8").match(
        /(?:src|href)\s*=\s*["'](?:https?:)?\/\/(?!ohio-education-funding\.pages\.dev)[^"']+/gi,
      );
      // Links in prose are fine; only fetched subresources matter, which are src= or a stylesheet.
      const fetched = (found ?? []).filter((m) => /^src/i.test(m) || /stylesheet/i.test(m));
      if (fetched.length > 0) offenders.push(`${file.slice(DIST.length + 1)}: ${fetched[0]}`);
    }
    expect(offenders.slice(0, 10)).toEqual([]);
  });
});
