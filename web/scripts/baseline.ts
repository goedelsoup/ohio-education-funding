/**
 * The visual baseline: what the pages looked like before, and what moved.
 *
 * ```
 * node scripts/baseline.ts            capture, and diff against the reference if there is one
 * node scripts/baseline.ts --promote  make what was just captured the new reference
 * ```
 *
 * The companion to `measure.ts`. That one turns the page into numbers a threshold can grade; this
 * one is for the half of a redesign no number reaches — whether the result is any good. Phases
 * #185, #186 and #189 are all *supposed* to change these images, and the point is that somebody
 * looks at the change rather than that a count stayed under a limit.
 *
 * # Why the references are not committed
 *
 * The obvious shape is `expect(page).toHaveScreenshot()` with reference PNGs in the repository, and
 * it would be red on the first CI run and every one after.
 *
 * A screenshot is a rendering of text, and this site ships no font binary — every reader, and every
 * machine, gets whatever their platform calls a UI sans. SF Pro on a laptop, DejaVu Sans or
 * Liberation Sans on a runner. Those disagree about glyph widths and, as this repository has
 * already paid to learn, about vertical metrics: `axisFoot` laid two rows touching under DejaVu and
 * a pixel apart under SF Pro, and the defect was invisible locally. Committed references would
 * encode one machine's fonts as the truth and fail on every other, which trains whoever sees the
 * failure to regenerate them without looking — the exact opposite of the review this exists for.
 *
 * So the reference lives in `.baseline/`, which is ignored, and the workflow is local and explicit:
 * promote before a phase, capture after it, look at what moved. If a CI-enforced version is ever
 * wanted, the way to get it is a pinned container with pinned fonts, not committed PNGs.
 *
 * # Why the diff runs in a browser
 *
 * Comparing two PNGs means decoding two PNGs, and Node has no decoder without a dependency. A
 * browser has one, and this script has already started one. So the images go back into a page and
 * a canvas counts the pixels that differ — no new dependency for a tool that runs by hand.
 */

import { mkdir, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import { existsSync } from "node:fs";
import { extname, join, normalize, resolve } from "node:path";
import { chromium, type Browser } from "@playwright/test";
import { ROUTES, WIDTHS } from "../src/lib/measure.ts";

const argv = process.argv.slice(2);
const PROMOTE = argv.includes("--promote");
const DIST = resolve("dist");
const ROOT = resolve(".baseline");
const REF = join(ROOT, "ref");
const CURRENT = join(ROOT, "current");

/** A pixel is "different" past this, so anti-aliasing on a glyph edge is not a finding. */
const CHANNEL_TOLERANCE = 12;
/** Below this share of differing pixels, a shot is unchanged — a caret or a scrollbar is not news. */
const NOISE_FLOOR = 0.0005;

if (!existsSync(DIST)) {
  console.error(`No build at ${DIST}. Run \`pnpm build\` first.`);
  process.exit(1);
}

const TYPES: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".csv": "text/csv; charset=utf-8",
};

const server = createServer(async (request, response) => {
  const path = decodeURIComponent((request.url ?? "/").split("?")[0] ?? "/");
  let file = join(DIST, normalize(path).replace(/^(\.\.[/\\])+/, ""));
  if (file.endsWith("/") || !extname(file)) file = join(file, "index.html");
  try {
    const body = await readFile(file);
    response.writeHead(200, { "content-type": TYPES[extname(file)] ?? "application/octet-stream" });
    response.end(body);
  } catch {
    response.writeHead(404).end("not found");
  }
});
const port = await new Promise<number>((ok) =>
  server.listen(0, "127.0.0.1", () => {
    const address = server.address();
    ok(typeof address === "object" && address != null ? address.port : 0);
  }),
);

/** `/district/043786/finances.html` at 375px in dark → `district-043786-finances@375-dark.png`. */
const shotName = (route: string, width: number, theme: string): string =>
  `${route.replace(/\.html$/, "").replace(/^\//, "").replace(/\//g, "-") || "index"}@${width}-${theme}.png`;

async function capture(browser: Browser): Promise<string[]> {
  await rm(CURRENT, { recursive: true, force: true });
  await mkdir(CURRENT, { recursive: true });
  const names: string[] = [];

  for (const theme of ["light", "dark"] as const) {
    for (const width of WIDTHS) {
      const page = await browser.newPage({ viewport: { width, height: 900 }, colorScheme: theme });
      /* Both channels, agreeing: `colorScheme` drives the media query and the attribute drives the
         explicit-choice blocks, and the palette is restated separately for each. Setting only one
         would leave half of `tokens/colors.css` unphotographed. */
      await page.addInitScript((value: string) => {
        addEventListener("DOMContentLoaded", () => {
          document.documentElement.setAttribute("data-theme", value);
        });
      }, theme);

      for (const route of ROUTES) {
        const response = await page.goto(`http://127.0.0.1:${port}${route}`, { waitUntil: "load" });
        if (response == null || !response.ok()) continue;
        /* Full page rather than viewport: the thing a redesign moves is usually below the fold,
           and a fold-only shot would call a rewritten card an unchanged page. */
        const name = shotName(route, width, theme);
        await page.screenshot({ path: join(CURRENT, name), fullPage: true });
        names.push(name);
      }
      await page.close();
    }
  }
  return names;
}

/**
 * How much of `after` differs from `before`, as a share of the larger image.
 *
 * Returns 1 when the two differ in size, which is the honest answer: there is no per-pixel
 * comparison to make, and a page that got taller is exactly the kind of change worth looking at.
 */
async function difference(browser: Browser, before: Buffer, after: Buffer): Promise<number> {
  const page = await browser.newPage();
  try {
    return await page.evaluate(
      async ([a, b, tolerance]) => {
        const load = async (base64: string): Promise<ImageBitmap> =>
          createImageBitmap(await (await fetch(`data:image/png;base64,${base64}`)).blob());
        const [one, two] = await Promise.all([load(a as string), load(b as string)]);
        if (one.width !== two.width || one.height !== two.height) return 1;

        const pixels = (bitmap: ImageBitmap): Uint8ClampedArray => {
          const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
          const context = canvas.getContext("2d");
          if (context == null) throw new Error("no 2d context");
          context.drawImage(bitmap, 0, 0);
          return context.getImageData(0, 0, bitmap.width, bitmap.height).data;
        };
        const left = pixels(one);
        const right = pixels(two);

        let differing = 0;
        for (let i = 0; i < left.length; i += 4) {
          if (
            Math.abs((left[i] ?? 0) - (right[i] ?? 0)) > (tolerance as number) ||
            Math.abs((left[i + 1] ?? 0) - (right[i + 1] ?? 0)) > (tolerance as number) ||
            Math.abs((left[i + 2] ?? 0) - (right[i + 2] ?? 0)) > (tolerance as number)
          ) {
            differing += 1;
          }
        }
        return differing / (left.length / 4);
      },
      [before.toString("base64"), after.toString("base64"), CHANNEL_TOLERANCE] as const,
    );
  } finally {
    await page.close();
  }
}

const browser = await chromium.launch();

try {
  const captured = await capture(browser);
  console.log(`Captured ${captured.length} shots to ${CURRENT}`);

  if (PROMOTE) {
    await rm(REF, { recursive: true, force: true });
    await mkdir(REF, { recursive: true });
    for (const name of captured) await writeFile(join(REF, name), await readFile(join(CURRENT, name)));
    console.log(`Promoted to ${REF}. Make the change, then run this again to see what moved.`);
  } else if (!existsSync(REF)) {
    console.log("");
    console.log("  No reference to compare against. Run `--promote` to make this one, then make");
    console.log("  the change and run again. See #183.");
  } else {
    const referenced = await readdir(REF);
    const changed: Array<[string, number]> = [];
    const added = captured.filter((name) => !referenced.includes(name));
    const removed = referenced.filter((name) => !captured.includes(name));

    for (const name of captured) {
      if (!referenced.includes(name)) continue;
      const share = await difference(
        browser,
        await readFile(join(REF, name)),
        await readFile(join(CURRENT, name)),
      );
      if (share > NOISE_FLOOR) changed.push([name, share]);
    }

    console.log("");
    if (changed.length === 0 && added.length === 0 && removed.length === 0) {
      console.log(`  Nothing moved, against ${referenced.length} reference shots.`);
    } else {
      changed.sort((a, b) => b[1] - a[1]);
      console.log(`  ${changed.length} of ${referenced.length} shots moved:`);
      for (const [name, share] of changed) {
        console.log(`    ${(share * 100).toFixed(1).padStart(5)}%  ${name}`);
      }
      for (const name of added) console.log(`      new  ${name}`);
      for (const name of removed) console.log(`     gone  ${name}`);
      console.log("");
      console.log("  A share of 100% usually means the page changed height, not that every pixel");
      console.log("  differs — there is no per-pixel comparison between two different sizes.");
      console.log(`  Look at them: ${CURRENT} against ${REF}.`);
      /*
       * A moved image is not a failure. Three of the redesign phases exist to move these, so an
       * exit code here would make the tool useless for its own purpose; it reports, and a person
       * decides. `--promote` is how a reviewed change becomes the new reference.
       */
    }
  }
} finally {
  await browser.close();
  server.close();
}
