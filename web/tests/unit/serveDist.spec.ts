/**
 * The server the measure instrument loads the build through answers `_redirects` as the host does.
 *
 * `scripts/serve-dist.ts` once served files only, so an address the host redirects was a 404 there
 * and a route measured at it would have measured the error page (#561). This serves a scratch
 * `dist/` holding the real `public/_redirects` — the file that deploys, not a fixture that could
 * agree with the server and disagree with the site — and asks it for each kind of address.
 */

import { copyFileSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import type { Server } from "node:http";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

import { afterAll, beforeAll, describe, expect, test } from "vitest";

import { serveDist } from "../../scripts/serve-dist.ts";

let dist: string;
let server: Server;
let origin: string;

beforeAll(async () => {
  dist = mkdtempSync(join(tmpdir(), "serve-dist-"));
  copyFileSync(resolve(import.meta.dirname, "../../public/_redirects"), join(dist, "_redirects"));
  mkdirSync(join(dist, "scenario"));
  writeFileSync(join(dist, "scenario/index.html"), "<p>runner</p>");
  ({ server, origin } = await serveDist(dist));
});

afterAll(() => {
  server.close();
  rmSync(dist, { recursive: true, force: true });
});

/** Ask without following, so the answer is the server's and not the destination's. */
const ask = (path: string) => fetch(`${origin}${path}`, { redirect: "manual" });

describe("serveDist", () => {
  test("a moved address answers the rule's status and destination", async () => {
    const response = await ask("/reach");
    expect(response.status).toBe(301);
    expect(response.headers.get("location")).toBe("/scenario/reach");
  });

  test("a placeholder is spliced and the incoming query survives", async () => {
    const response = await ask("/district/043786/scenario?base=6020");
    expect(response.status).toBe(301);
    expect(response.headers.get("location")).toBe("/scenario?base=6020#d=043786");
  });

  // The control: an address no rule names still reaches the file, and a missing one is still a
  // 404, so the two cases above are the rules answering and not every request redirecting.
  test("an address no rule names is served from the build", async () => {
    const found = await ask("/scenario/");
    expect(found.status).toBe(200);
    expect(await found.text()).toBe("<p>runner</p>");
    expect((await ask("/nowhere")).status).toBe(404);
  });
});
