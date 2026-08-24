// @ts-check
import { appendFileSync, existsSync, readFileSync } from "node:fs";

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
