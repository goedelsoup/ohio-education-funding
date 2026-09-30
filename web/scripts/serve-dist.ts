/**
 * The built site, served on an ephemeral port on this machine, for a browser to load.
 *
 * `measure.ts` and `baseline.ts` both open `dist/` in Chromium and both need it over HTTP rather
 * than `file://`, because the pages use root-relative URLs. They each carried a copy of this, and
 * the copies had already drifted: one knew `.xml` and `.txt` and the other did not.
 *
 * It answers `dist/_redirects` before it looks for a file, with the same reader the Playwright
 * preview uses (`src/lib/redirects.ts`), so there is one reading of that file and not three.
 * Without it an address the host redirects — `/reach` — was a 404 here, and a route added to the
 * measure list at an old address would have measured the error page and reported nothing wrong.
 */

import { createServer, type Server } from "node:http";
import { readFile } from "node:fs/promises";
import { extname, join, normalize } from "node:path";

import { parseRedirects, resolveRedirect } from "../src/lib/redirects.ts";

const TYPES: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".csv": "text/csv; charset=utf-8",
  ".xml": "application/xml; charset=utf-8",
  ".txt": "text/plain; charset=utf-8",
};

/** Serve `dist` on 127.0.0.1 at a port the OS picks. Close the server when done with it. */
export async function serveDist(dist: string): Promise<{ server: Server; origin: string }> {
  /* Read once, and required: the build copies `public/_redirects` into every `dist/`, so a build
     without it is not the artefact that deploys. */
  const rules = parseRedirects(await readFile(join(dist, "_redirects"), "utf8"));

  const server = createServer(async (request, response) => {
    const url = new URL(request.url ?? "/", "http://serve-dist.invalid");
    const hit = resolveRedirect(rules, url.pathname, url.search);
    if (hit) {
      response.writeHead(hit.status, { location: hit.location }).end();
      return;
    }
    const path = decodeURIComponent(url.pathname);
    /* `normalize` before joining, so a `..` in the request cannot reach outside the build. This
       serves a directory to a browser on this machine and is not exposed, but a traversal here
       would silently read a file that is not part of the artefact. */
    const relative = normalize(path).replace(/^(\.\.[/\\])+/, "");
    let file = join(dist, relative);
    if (file.endsWith("/") || !extname(file)) file = join(file, "index.html");
    try {
      const body = await readFile(file);
      response.writeHead(200, { "content-type": TYPES[extname(file)] ?? "application/octet-stream" });
      response.end(body);
    } catch {
      response.writeHead(404).end("not found");
    }
  });

  const port = await new Promise<number>((ok) => {
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      ok(typeof address === "object" && address != null ? address.port : 0);
    });
  });
  return { server, origin: `http://127.0.0.1:${port}` };
}
