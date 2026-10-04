/**
 * No address this site has published returns a 404.
 *
 * `public/_redirects` keeps the addresses that moved alive at the edge, and
 * `tests/unit/redirects.spec.ts` pins what each rule says. What that cannot see is whether the
 * page a rule points at *works* when a reader arrives by the old address — the runner reading the
 * district out of a fragment, the levers surviving the hop — which only a browser can.
 *
 * The preview server applies the file (`scripts/preview.config.ts`), so these land where the host
 * sends them rather than on a 404 from a server that never read it.
 */

import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import { expect, test } from "@playwright/test";

import { parseRedirects } from "../../src/lib/redirects.ts";
import { explainedOn } from "../../src/lib/explained/flag.ts";
import { CLEVELAND, NORTHERN } from "./helpers.ts";

const DIST = resolve(import.meta.dirname, "../../dist");
const RULES = parseRedirects(
  // The built copy, which is the one the preview serves from — `public/_redirects` copied at build.
  readFileSync(resolve(DIST, "_redirects"), "utf8"),
);

/** A concrete address for a rule's source: its placeholders filled with a district that exists. */
const sample = (from: string): string =>
  from.replace(/:[A-Za-z]\w*/g, CLEVELAND).replace(/\*$/, "anything");

test.describe("addresses that moved", () => {
  test("every source in _redirects lands on a page that exists", async ({ request }) => {
    /*
     * Two requests a rule, the second following the hop, one after another over every rule the
     * build wrote: 762 since the unpadded addresses were generated (#596), so about 2,300 round
     * trips in one test. That took 17.0s alone and 24.6s beside three other workers on a laptop,
     * and it timed out at the default 30s mid-loop on the 4-vCPU runner, at a different request
     * each time, in both of the first two runs with four workers per shard (#647). Slow triples
     * the budget. The cost is bounded by the rule count, not by anything a request waits on.
     */
    test.slow();
    expect(RULES.length).toBeGreaterThan(0);
    for (const rule of RULES) {
      const from = sample(rule.from);
      const hop = await request.get(from, { maxRedirects: 0 });
      expect(hop.status(), `${from} is not redirected`).toBe(rule.status);
      const to = hop.headers()["location"] ?? "";
      const landed = await request.get(from);
      expect(landed.status(), `${from} → ${to}`).toBe(200);
      /*
       * And the page is one the build emitted. A status is not enough: `vite preview` answers an
       * unknown path with the root document and a 200, so a rule pointing at a page that does not
       * exist passed the line above. The host would 404 it.
       */
      const path = new URL(landed.url()).pathname;
      const target = new URL(to, landed.url()).pathname;
      expect(path, `${from} landed somewhere other than ${to}`).toBe(target);
      const file = path === "/" ? "index.html" : `${path.replace(/^\//, "")}.html`;
      const emitted = existsSync(resolve(DIST, file));
      expect(emitted, `${from} → ${to}, which the build did not emit`).toBe(true);
    }
  });

  test("the old reach address opens the runner's reach view, levers and all", async ({ page }) => {
    await page.goto("/reach?base=1.05");
    await expect(page).toHaveURL(/\/scenario\/reach\?/);
    await expect(page.locator("h1")).toHaveText("Change the formula");
    await expect(page.locator('.subnav a[aria-current="page"]')).toHaveText("Who it reaches");
    await expect(page.locator("#lv-base")).toHaveValue("1.05");
  });

  test("a district's old scenario address opens the runner on that district, levers and all", async ({
    page,
  }) => {
    await page.goto(`/district/${NORTHERN}/scenario?g=removed&base=1.05`);
    await expect(page).toHaveURL(/\/scenario\?/);
    await expect(page.locator("#sc-district")).toHaveValue(NORTHERN);
    await expect(page.locator("#lv-base")).toHaveValue("1.05");
    await expect(page.locator("#scenario-out")).toContainText("State aid under this scenario");
    await expect(page.locator("#scenario-out")).toContainText("And to everyone else");
    // And the district is written back into the query, so the address in the bar is one a reader
    // can copy and send.
    await expect(page).toHaveURL(new RegExp(`[?&]d=${NORTHERN}`));
    await expect(page).toHaveURL(/[?&]base=1\.05/);
  });
});

test.describe("the runner's two views", () => {
  test("crossing between them keeps the levers and the district", async ({ page }) => {
    await page.goto(`/scenario?d=${CLEVELAND}`);
    await expect(page.locator("#scenario-out .card")).not.toHaveCount(0);
    await page.locator("#lv-base").fill("1.05");
    await page.locator("#lv-base").dispatchEvent("input");
    await expect(page).toHaveURL(/[?&]base=1\.05/);

    await page.locator(".subnav a", { hasText: "Who it reaches" }).click();
    await expect(page).toHaveURL(/\/scenario\/reach\?/);
    await expect(page.locator("#lv-base")).toHaveValue("1.05");
    await expect(page).toHaveURL(new RegExp(`[?&]d=${CLEVELAND}`));

    await page.locator(".subnav a", { hasText: "What changes" }).click();
    await expect(page).toHaveURL(/\/scenario\?/);
    await expect(page.locator("#lv-base")).toHaveValue("1.05");
    await expect(page.locator("#sc-district")).toHaveValue(CLEVELAND);
  });
});

test.describe("the page an address that is not there lands on (#596)", () => {
  test.use({ viewport: { width: 375, height: 800 } });

  test("fits a 375px screen, nothing inside it scrolling sideways", async ({ page }) => {
    // `/404` rather than a missing path: `vite preview` answers an unknown path with the root
    // document, where the host serves this one. The overflow was a 392px table in a 295px box,
    // which scrolled inside its `.scroll` and left the page's own width alone, so every element
    // is asked, not the document.
    await page.goto("/404");
    const wide = await page.locator("main").evaluate((main) =>
      [main, ...main.querySelectorAll("*")]
        .filter((el) => el.scrollWidth > el.clientWidth + 1 && el.clientWidth > 0)
        .map((el) => `${el.tagName.toLowerCase()}.${el.className}: ${el.scrollWidth} in ${el.clientWidth}`),
    );
    expect(wide).toEqual([]);
  });

  test("offers the name-or-IRN field and every section of the bar", async ({ page }) => {
    await page.goto("/404");
    await expect(page.locator("#where-to-go li a")).toHaveText([
      "Find a district",
      // Only where the build publishes it (#718).
      ...(explainedOn() ? ["Explained"] : []),
      "Places",
      "Analysis",
      "Change the formula",
      "Library",
      "About",
    ]);
    await page.locator("#missing-q").fill(CLEVELAND);
    await page.locator("#missing-q").press("Enter");
    await expect(page).toHaveURL(new RegExp(`/districts\\?q=${CLEVELAND}$`));
  });
});
