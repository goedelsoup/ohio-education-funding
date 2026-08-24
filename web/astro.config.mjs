// @ts-check
import { appendFileSync, existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";

import sitemap from "@astrojs/sitemap";
import { defineConfig } from "astro/config";

/**
 * Put the CSV download's response headers back, where the static host will actually apply them.
 *
 * `src/pages/data/districts.csv.ts` returns a `Response` carrying `Content-Type: text/csv` and a
 * `Content-Disposition` naming the file — and in a static build a `Response`'s headers are
 * discarded. Astro takes the body, writes `dist/data/districts.csv`, and throws the rest away;
 * there is no server left to send them. So the comment in that route saying "the provenance
 * travels in the filename" described something that had stopped happening: a browser saved the
 * file as `districts.csv`, with the fiscal year the figures are on nowhere on it, and a reader
 * with three of them in a downloads folder could not tell which model each came from.
 *
 * Cloudflare Pages reads `_headers` from the deploy root, so that is where they have to go. It
 * cannot be written into `public/_headers` by hand, because the filename carries the fiscal year
 * and that moves with the feed — a literal there would be right until the next calculator and
 * then quietly wrong, which is the class of defect `src/lib/year.ts` exists to end. This appends
 * the block after the build, reading the year out of the same feed the pages were built from.
 *
 * Not visible in `dist/`'s HTML, in `vite preview`, or in any test that opens a page — the same
 * blind spot the rest of `public/_headers` documents at length. `tests/e2e/app.spec.ts` reads the
 * built `_headers` instead, which is the artefact that gets deployed.
 */
function csvDownloadHeaders() {
  return {
    name: "csv-download-headers",
    hooks: {
      /** @type {(context: { dir: URL }) => void} */
      "astro:build:done": ({ dir }) => {
        const feed = JSON.parse(
          readFileSync(new URL("./public/data/bundle.json", import.meta.url), "utf8"),
        );
        const headers = new URL("_headers", dir);
        // The rest of the file is the point; appending to a file that is not there would produce
        // a `_headers` holding only this block and silently drop the CSP.
        if (!existsSync(headers)) {
          throw new Error(
            "public/_headers did not reach dist/ — appending the CSV block would replace it",
          );
        }
        appendFileSync(
          headers,
          "\n# Written by the `csv-download-headers` integration in `astro.config.mjs`, not by hand.\n" +
            "# A static build discards the headers the route returns; these are the ones it meant.\n" +
            "# The filename carries the fiscal year, so it is read from the feed rather than typed.\n" +
            "/data/districts.csv\n" +
            "  Content-Type: text/csv; charset=utf-8\n" +
            `  Content-Disposition: attachment; filename="ohio-school-funding-fy${feed.fiscal_year}.csv"\n`,
        );
      },
    },
  };
}

/**
 * Astro, in static mode, over Vite.
 *
 * The output is a directory of plain files with no server behind it — the property static hosting
 * wants, and the one this platform has had since it was a hand-rolled `tsc` build. What changed
 * with the move to real routes is how much of it there is: roughly 2,500 documents rather than
 * one, because a district's figures are now written into that district's page at build time
 * instead of fetched and rendered in the browser.
 *
 * That is the trade this configuration encodes. The site can no longer be updated by regenerating
 * `public/data/bundle.json` alone — the numbers are in the HTML, so a feed change is a rebuild.
 * In exchange every page is complete before any script runs, carries its figures to a search
 * engine, and costs one document instead of a 1.1 MB feed.
 *
 * The feed is still copied verbatim into `dist/` and still served. It is what the scenario routes
 * fetch — they compute in the browser, so they need the whole 609-district panel and the
 * checkpoints that gate it — and it is what `/data` offers for download.
 */
/**
 * Keep the sourcemaps that explain the arithmetic; drop the library source riding along with them.
 *
 * `vite.build.sourcemap` is on for a stated reason — the scenario routes re-derive Ohio's funding
 * formula in the browser and somebody checking that arithmetic should be able to read it. What
 * ships alongside it is not that: `scenario.js.map` is 1.37 MB of which **82% is d3 and Plot**,
 * 385 sources, none of which is the formula. The 155 KB that is `src/` is the whole of what the
 * option was turned on for.
 *
 * So the map keeps its mappings and every `src/` file keeps its text; a `node_modules` entry keeps
 * its name and loses its `sourcesContent`. A debugger stepping into Plot then shows a frame it
 * cannot source rather than one it cannot name, which is what a library frame is anyway.
 *
 * Done here rather than through a Vite option because Vite has none that fits:
 * `sourcemapExcludeSources` drops every source including the formula, and `sourcemapIgnoreList`
 * only marks frames as third-party without removing the text they carry.
 */
function trimVendorSources() {
  return {
    name: "trim-vendor-sources",
    hooks: {
      /** @type {(context: { dir: URL }) => void} */
      "astro:build:done": ({ dir }) => {
        const assets = new URL("_astro/", dir);
        let saved = 0;
        for (const name of readdirSync(assets)) {
          if (!name.endsWith(".map")) continue;
          const path = new URL(name, assets);
          const before = readFileSync(path, "utf8");
          /** @type {{ sources?: string[], sourcesContent?: (string | null)[] }} */
          const map = JSON.parse(before);
          const sources = map.sources ?? [];
          if (!Array.isArray(map.sourcesContent)) continue;
          map.sourcesContent = map.sourcesContent.map((content, i) =>
            String(sources[i] ?? "").includes("node_modules") ? null : content,
          );
          const after = JSON.stringify(map);
          if (after.length >= before.length) continue;
          writeFileSync(path, after);
          saved += before.length - after.length;
        }
        if (saved > 0) {
          console.log(
            `[trim-vendor-sources] dropped ${(saved / 1e6).toFixed(2)} MB of library source text`,
          );
        }
      },
    },
  };
}

/**
 * Tell the browser about the second file before it has finished reading the first.
 *
 * Every page loads one module — `<script type="module" src="…Base…js">` — and that module's first
 * line imports another. So the chain is: parse the HTML, fetch the stub, parse it, discover the
 * import, fetch that. **Two serial round trips before any of it runs**, for about 2 KB, on all
 * 3,487 pages; the scenario routes go one deeper and take three.
 *
 * A `modulepreload` link in the head starts the second fetch at the same moment as the first, so
 * the round trips overlap instead of queueing. Nothing about the bundle changes — same files, same
 * hashes, same order of execution — only when the browser learns they exist.
 *
 * # Why this is a post-build pass over the HTML
 *
 * Astro emits these links for hydrated components; a plain `<script>` in a static build gets none,
 * and the import graph is not known at render time anyway — it is a property of the bundle Vite
 * produces afterwards. So the graph is read back off the emitted chunks, which is the same
 * argument `semantics.ts` makes about reading the document: whatever is in the artefact is what
 * gets preloaded, and a chunk that stops existing stops being named.
 *
 * Same-origin and governed by `script-src 'self'`, which the CSP already grants — see
 * `public/_headers`.
 */
function preloadModules() {
  return {
    name: "preload-modules",
    hooks: {
      /** @type {(context: { dir: URL }) => void} */
      "astro:build:done": ({ dir }) => {
        const assetDir = new URL("_astro/", dir);
        /** What each chunk imports, by filename. */
        const imports = new Map();
        for (const name of readdirSync(assetDir)) {
          if (!name.endsWith(".js")) continue;
          const code = readFileSync(new URL(name, assetDir), "utf8");
          imports.set(name, [...code.matchAll(/["']\.\/([^"']+\.js)["']/g)].map((m) => m[1]));
        }

        /** @type {(entry: string) => string[]} Everything a chunk pulls in, transitively. */
        const closure = (entry) => {
          const found = new Set();
          const queue = [...(imports.get(entry) ?? [])];
          while (queue.length > 0) {
            const next = queue.pop();
            if (next == null || found.has(next)) continue;
            found.add(next);
            queue.push(...(imports.get(next) ?? []));
          }
          return [...found];
        };

        /** @type {URL[]} */
        const pages = [];
        /** @type {(at: URL) => void} */
        const walk = (at) => {
          for (const entry of readdirSync(at, { withFileTypes: true })) {
            const path = new URL(`${entry.name}${entry.isDirectory() ? "/" : ""}`, at);
            if (entry.isDirectory()) walk(path);
            else if (entry.name.endsWith(".html")) pages.push(path);
          }
        };
        walk(dir);

        let linked = 0;
        for (const page of pages) {
          const html = readFileSync(page, "utf8");
          /*
           * Every module script on the page, not the first one.
           *
           * A scenario route carries two — its own and the layout's — and reading only the first
           * left the layout's chunk undeclared on exactly the pages that already take the most
           * round trips. The two closures overlap, so they are unioned rather than concatenated.
           */
          const entries = [
            ...html.matchAll(/<script type="module" src="\/_astro\/([^"]+\.js)"/g),
          ].map((m) => m[1] ?? "");
          if (entries.length === 0) continue;
          const deps = [...new Set(entries.flatMap((entry) => closure(entry)))].filter(
            (name) => !entries.includes(name),
          );
          if (deps.length === 0) continue;
          const links = deps
            .map((name) => `<link rel="modulepreload" href="/_astro/${name}">`)
            .join("");
          // Into the head, where the browser has already begun looking for what to fetch.
          const updated = html.replace("</head>", `${links}</head>`);
          if (updated === html) continue;
          writeFileSync(page, updated);
          linked += deps.length;
        }
        console.log(`[preload-modules] ${linked} preload links across ${pages.length} pages`);
      },
    },
  };
}

export default defineConfig({
  // Required by `@astrojs/sitemap`, and used for the canonical link in the layout. This is the
  // production hostname; a preview deploy will emit canonicals pointing at production, which is
  // the correct behaviour for a preview.
  //
  // The Pages project's own subdomain, because that is what actually serves. A custom domain was
  // named here for long enough to reach every canonical link, the sitemap and a thousand preview
  // cards while never resolving — so if one is added later, this line moves in the same commit as
  // the DNS record, not after it.
  site: "https://ohio-education-funding.pages.dev",
  output: "static",
  integrations: [
    sitemap({
      // The 404 is reachable only by failing to reach something else, and the search index is a
      // data file that happens to live in `pages/`. Neither belongs in a sitemap.
      //
      // Nor do the preview cards. `/og/*` is roughly a thousand PNGs that exist to be fetched by
      // an unfurler when someone shares the page they belong to — they are an attribute of a
      // document rather than documents, and listing them would treble the sitemap with images no
      // reader can navigate to. The two icon routes are excluded for the same reason.
      filter: (page) =>
        !page.includes("/404") &&
        !page.includes("search-index") &&
        !page.includes("/og/") &&
        !page.endsWith("apple-touch-icon.png") &&
        !page.endsWith("icon-32.png"),
    }),
    csvDownloadHeaders(),
    trimVendorSources(),
    preloadModules(),
  ],
  build: {
    // `dist/district/043786.html` rather than `dist/district/043786/index.html`. Keeps the deploy
    // a flat readable tree, and means every route in `src/lib/routes.ts` has no trailing slash.
    format: "file",
    /*
     * Never inline a script, however small.
     *
     * Astro's default is to inline a bundled `<script>` when the output is small enough that a
     * request costs more than the bytes. That is the right trade almost everywhere and the wrong
     * one here: this site is served under `script-src 'self'`, which permits no inline script at
     * all, so an inlined bundle is a script the browser refuses to run.
     *
     * It is invisible in development and in `vite preview`, both of which apply no headers, and it
     * only bites the three pages whose scripts happened to be under the threshold — the 404
     * suggestions, the district filter, and search. All three shipped dead. See the built-output
     * check in `tests/e2e/`, which now fails on any inline script rather than waiting for someone
     * to open the deployed site.
     */
    inlineStylesheets: "never",
  },
  vite: {
    build: {
      // The formula lives in this bundle and gets read by people checking the arithmetic.
      sourcemap: true,
      // Emit every script as a file. Pairs with the CSP note above.
      assetsInlineLimit: 0,
    },
  },
});
