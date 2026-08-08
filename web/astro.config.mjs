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
  // `dist/district/043786.html` rather than `dist/district/043786/index.html`. Keeps the deploy a
  // flat readable tree, and means every route in `src/lib/routes.ts` has no trailing slash.
  build: { format: "file" },
  integrations: [
    sitemap({
      // The 404 is reachable only by failing to reach something else, and the search index is a
      // data file that happens to live in `pages/`. Neither belongs in a sitemap.
      filter: (page) => !page.includes("/404") && !page.includes("search-index"),
    }),
  ],
  vite: {
    build: {
      // The formula lives in this bundle and gets read by people checking the arithmetic.
      sourcemap: true,
    },
  },
});
