/**
 * `vite preview`, taught the host's redirects.
 *
 * Playwright serves `dist/` through `vite preview`, which applies nothing in `_redirects` — it
 * serves the file as a file. So every address `public/_redirects` keeps alive was a 404 in the
 * browser suite, and a test of those addresses could only have been written against a server that
 * behaves differently from the one that deploys.
 *
 * This reads the built `dist/_redirects` — the file that ships — and answers from it with the
 * host's rules (`src/lib/redirects.ts`) before the static handler sees the request. Nothing else
 * changes: the preview still applies no `_headers`, which `public/_headers` documents.
 *
 * Used only by `playwright.config.ts`. The site's build does not read it.
 */

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { defineConfig } from "vite";

import { parseRedirects, resolveRedirect } from "../src/lib/redirects.ts";

export default defineConfig({
  plugins: [
    {
      name: "cloudflare-redirects",
      configurePreviewServer(server) {
        const rules = parseRedirects(
          readFileSync(resolve(server.config.root, "dist/_redirects"), "utf8"),
        );
        server.middlewares.use((request, response, next) => {
          const url = new URL(request.url ?? "/", "http://preview.invalid");
          const hit = resolveRedirect(rules, url.pathname, url.search);
          if (!hit) return next();
          response.statusCode = hit.status;
          response.setHeader("Location", hit.location);
          response.end();
        });
      },
    },
  ],
});
