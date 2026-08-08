// @ts-check
import sitemap from "@astrojs/sitemap";
import { defineConfig } from "astro/config";

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
  site: "https://schools.ohio.shawneesmart.systems",
  output: "static",
  integrations: [
    sitemap({
      // The 404 is reachable only by failing to reach something else, and the search index is a
      // data file that happens to live in `pages/`. Neither belongs in a sitemap.
      filter: (page) => !page.includes("/404") && !page.includes("search-index"),
    }),
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
