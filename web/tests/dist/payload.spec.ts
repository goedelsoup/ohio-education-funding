/**
 * What the build asks a reader to download, and what it has already sent them.
 *
 * # Why these are file sizes and not stopwatches
 *
 * A wall-clock threshold in CI is a claim about the runner. Each of these is instead a claim about
 * the artefact, exact and machine-checkable: the number of preload links against the import graph
 * of the emitted chunks, the comparison table present in `compare.html` rather than fetched, the
 * bytes of one district's JSON against the 641,042 B panel the route used to download.
 *
 * All three were round trips before they were assertions, and none of them is visible from a page:
 * a missing preload is a slower load, not an error, and a `/compare` that shipped an empty `<div>`
 * looked correct three seconds later.
 *
 * The browser half — that a bare `/compare` fetches nothing at all and a pair costs two district
 * files — is in `directory.spec.ts`, where a real network log is the only evidence available.
 */

import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, test } from "vitest";

import { DIST, pages } from "./artefact.ts";

/** Northern Local (Perry County). The corpus's property-poor exemplar, and `/compare`'s default. */
const NORTHERN = "049056";

/*
 * What a page tells the browser to fetch before it has finished reading the first file.
 *
 * Every page loads one module and that module's first line imports another, so the chain was:
 * parse the HTML, fetch the stub, parse it, discover the import, fetch that — two serial round
 * trips before anything ran, for about 2 KB, on all 3,487 pages. The scenario routes went a level
 * deeper. `preloadModules` in `astro.config.mjs` reads the import graph off the emitted chunks and
 * declares it.
 */
describe("module preloading", () => {


  test("every page declares the chunks its script will import, and no others", () => {
    /*
     * Three properties at once, because they fail in different directions. A preload naming a file
     * that is not there is a wasted request and a console error; a preload naming a script the page
     * already loads directly is a duplicate fetch; and a page whose module imports something it
     * never declares is the round trip this exists to remove, back again.
     */
    const dangling: string[] = [];
    const duplicated: string[] = [];
    const undeclared: string[] = [];
    let declared = 0;

    // What each emitted chunk imports, read off the bundle exactly as the integration does.
    const chunks = new Map<string, string[]>();
    for (const name of readdirSync(join(DIST, "_astro"))) {
      if (!name.endsWith(".js")) continue;
      const code = readFileSync(join(DIST, "_astro", name), "utf8");
      chunks.set(name, [...code.matchAll(/["']\.\/([^"']+\.js)["']/g)].map((m) => m[1] ?? ""));
    }
    const closure = (entry: string): string[] => {
      const found = new Set<string>();
      const queue = [...(chunks.get(entry) ?? [])];
      while (queue.length > 0) {
        const next = queue.pop();
        if (next == null || found.has(next)) continue;
        found.add(next);
        queue.push(...(chunks.get(next) ?? []));
      }
      return [...found];
    };

    for (const file of pages()) {
      const html = readFileSync(file, "utf8");
      const where = file.slice(DIST.length + 1);
      const scripts = [...html.matchAll(/<script type="module" src="\/_astro\/([^"]+\.js)"/g)].map(
        (m) => m[1] ?? "",
      );
      const preloads = [...html.matchAll(/<link rel="modulepreload" href="\/_astro\/([^"]+)"/g)].map(
        (m) => m[1] ?? "",
      );
      declared += preloads.length;

      for (const name of preloads) {
        if (!existsSync(join(DIST, "_astro", name))) dangling.push(`${where}: ${name}`);
        if (scripts.includes(name)) duplicated.push(`${where}: ${name}`);
      }
      const wanted = new Set(scripts.flatMap((s) => closure(s)));
      for (const name of wanted) {
        if (!scripts.includes(name) && !preloads.includes(name)) undeclared.push(`${where}: ${name}`);
      }
    }

    expect(declared, "the build declares preloads at all").toBeGreaterThan(3_000);
    expect(dangling.slice(0, 5), "a preload naming a file that is not there").toEqual([]);
    expect(duplicated.slice(0, 5), "a preload for a script the page already loads").toEqual([]);
    expect(undeclared.slice(0, 5), "an import the page never declared").toEqual([]);
  });

  test("a scenario route asks for the panel before it has downloaded Plot", () => {
    /*
     * #504. The runner draws with Observable Plot in the browser, and Plot and d3 are about 300 KB
     * of it. Imported statically, all of that arrived and ran before the script could request the
     * panel — two downloads in series where neither needs the other. `scenario-load.ts` imports
     * the runner dynamically, so it is fetched beside the panel.
     *
     * Held as the bytes a page names in its HTML — its module scripts and its preloads, which the
     * test above holds to the static import graph — because that is what arrives before the fetch
     * can start. That included the 337,895 B runner on all three routes; it is under 8 KB now, and
     * the bound is set where a return of the library could not hide under it.
     */
    for (const route of ["scenario.html", "reach.html", `district/${NORTHERN}/scenario.html`]) {
      const html = readFileSync(join(DIST, route), "utf8");
      const named = [
        ...html.matchAll(/<script type="module" src="\/_astro\/([^"]+\.js)"/g),
        ...html.matchAll(/<link rel="modulepreload" href="\/_astro\/([^"]+)"/g),
      ].map((m) => m[1] ?? "");
      expect(named.join(" "), `${route} loads the scenario entry`).toMatch(/scenario-load\./);
      const bytes = named.reduce((sum, name) => sum + statSync(join(DIST, "_astro", name)).size, 0);
      expect(bytes, `${route}: script it must download before asking for the panel`).toBeLessThan(
        32_768,
      );
    }
  });
});

/**
 * `/compare` ships its comparison, and fetches a pair rather than a panel.
 *
 * # The measurement this holds
 *
 * The route used to download the whole 609-district panel — 641,042 B, 127,961 gzipped — to render
 * seventeen rows about two districts, and shipped an empty `#compare-out` until it landed. At
 * 400 Kbps / 400 ms RTT: first paint 1,284 ms, panel finishing **4,131 ms**, table 4,340 ms. Three
 * seconds of an empty box. That was the last open half of #111.
 *
 * Measured again after: a bare `/compare` has its table in the HTML and fetches **nothing**, and
 * `?a=&b=` fetched two files totalling **1,181 B**, finishing at 1,678 ms against a first paint of
 * 1,264 ms.
 *
 * Those two files are **4,006 B** today, against a panel that is now 1,237,912 B. Both figures
 * move with the feed — a column added to a district is added 609 times — so both are dated here
 * rather than maintained. What the assertions below hold is the shape, which does not move.
 */
describe("the comparison arrives with the page", () => {


  test("the built page carries a whole comparison table, not an empty div", () => {
    const page = readFileSync(join(DIST, "compare.html"), "utf8");
    const card = /<div class="card" id="comparison"[^>]*data-a="(\d+)"[^>]*data-b="(\d+)"/.exec(page);
    expect(card, "#compare-out holds a comparison card naming its own pair").not.toBeNull();
    expect(card![1], "and the pair is the corpus's own property-poor half").toBe(NORTHERN);

    // Seventeen rows: thirteen quantities and four flags, each addressable by its own key.
    expect((page.match(/data-row="/g) ?? []).length).toBe(17);
    // Both column heads name a district and link to it, which is the half a swap rewrites.
    // `[^>]*` because a post-build pass adds `scope="col"` between the tag and the attribute.
    expect((page.match(/<th[^>]*data-head="[ab]"[^>]*><a href="\/district\//g) ?? []).length).toBe(2);
    // And no cell is waiting to be filled in: the figures are there, not placeholders.
    expect(page).not.toContain("<td class=\"tnum\"></td>");
  });



  test("one district is a couple of kilobytes, and there are 609 of them", () => {
    const dir = join(DIST, "data", "district");
    const files = readdirSync(dir).filter((name) => name.endsWith(".json"));
    expect(files.length, "one per district in the feed").toBe(609);
    const sizes = files.map((name) => readFileSync(join(dir, name), "utf8").length);
    const largest = Math.max(...sizes);
    /*
     * A ceiling on the largest file, and #490 moved it.
     *
     * The bound was `2_048` — a byte under 2 KiB — and the largest district measured 2,039, nine
     * bytes inside it. `guarantee_in_fy21_base` costs 27 bytes a district, so the third reading of
     * Section 265.225 broke a bound it had been clearing by nine. Measured now: 1,757 smallest,
     * 1,991 median, 2,066 largest, five files over 2 KiB.
     *
     * Raised rather than deleted, and raised past the measurement rather than to it. What this
     * guards is a shape — a swap costs a pair of small files and not the panel — and a ceiling
     * that has to move every time a column is added is a ceiling that stops being read. The
     * headroom is about four more columns; the failure it exists to catch is this route going back
     * to shipping the whole feed, which is two orders of magnitude away from either number.
     */
    expect(
      largest,
      "a district's formula inputs, against the 641,042 B panel this route used to download",
    ).toBeLessThan(2_304);
  });
});
