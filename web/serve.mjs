/**
 * A static file server for this directory, in the Node standard library.
 *
 * Browsers block `fetch` from `file://`, so the page needs to be served. This exists so that
 * the instruction for doing it does not reach for a runtime the repository otherwise does not
 * use — the extraction pipeline was moved off Python for the same reason.
 *
 *     node web/serve.mjs [port]
 */

import { createServer } from "node:http";
import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import { extname, join, normalize, resolve } from "node:path";

const ROOT = resolve(import.meta.dirname);
const PORT = Number(process.argv[2] ?? 8000);

const TYPES = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
};

createServer(async (request, response) => {
  const url = new URL(request.url ?? "/", "http://localhost");
  // normalize() collapses `..`, and the prefix check rejects anything that still escapes the
  // served directory. A static server is a small thing to get wrong in a large way.
  const path = join(ROOT, normalize(decodeURIComponent(url.pathname)));
  if (!path.startsWith(ROOT)) {
    response.writeHead(403).end("forbidden");
    return;
  }

  let target = path;
  try {
    if ((await stat(target)).isDirectory()) target = join(target, "index.html");
  } catch {
    response.writeHead(404).end("not found");
    return;
  }

  try {
    await stat(target);
  } catch {
    response.writeHead(404).end("not found");
    return;
  }

  response.writeHead(200, {
    "content-type": TYPES[extname(target)] ?? "application/octet-stream",
    "cache-control": "no-cache",
  });
  createReadStream(target).pipe(response);
}).listen(PORT, () => {
  console.log(`serving ${ROOT} at http://localhost:${PORT}`);
});
