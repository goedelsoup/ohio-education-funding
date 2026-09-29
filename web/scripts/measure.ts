/**
 * The measure report: what the built pages look like, as numbers.
 *
 * ```
 * node scripts/measure.ts [--dist dist] [--json measure.json] [--font "DejaVu Sans"] [--fail]
 * ```
 *
 * Runs after `build`, over `dist/`, for the same reason `check-dist-links.ts` does: the question
 * is about the artefact that gets uploaded, and scanning a stale `dist/` reports on the previous
 * commit's output. See `src/lib/measure.ts` for what each metric is and which of them survive a
 * change of platform.
 *
 * # Why it serves the directory rather than opening a file
 *
 * Every asset on this site is absolute from the root — `/_astro/…`, `/data/panel.json` — so a page
 * opened over `file://` loads none of its stylesheet, and a report on an unstyled page is a report
 * on nothing. The server here is deliberately the smallest thing that answers a GET, rather than
 * `vite preview` as the e2e suite uses: this asks only about computed style and layout, neither of
 * which a response header can reach, and a child process that has to be waited for and killed is
 * two failure modes bought for nothing.
 *
 * # `--font`
 *
 * The reading measure and the chrome height depend on how wide the platform draws a glyph, and
 * this repository has already shipped a chart defect that reproduced under DejaVu Sans and not
 * under SF Pro. Passing `--font` forces one family for the run so two machines can be compared;
 * without it the report measures what a reader on THIS machine sees, which is the honest default
 * and the useless one for a diff. Either way the family and the advance width land in the output.
 *
 * # `--fail`
 *
 * Exits non-zero on a threshold breach. There are no thresholds yet — #183 builds the instrument
 * and grades nothing, because a check that starts red teaches whoever added it to skip it. Each
 * later redesign phase fills in one entry of `THRESHOLDS` and this flag starts biting.
 */

import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { chromium } from "@playwright/test";
import { serveDist } from "./serve-dist.ts";
import {
  BOX_MIN_RADIUS,
  PROSE_CELL_MIN_WORDS,
  PROSE_MIN_CHARS,
  ROUTES,
  THRESHOLDS,
  WIDTHS,
  collect,
  formatReport,
  violations,
  type Measured,
  type Report,
} from "../src/lib/measure.ts";

const argv = process.argv.slice(2);
const flag = (name: string, fallback?: string): string | undefined => {
  const index = argv.indexOf(`--${name}`);
  return index === -1 ? fallback : (argv[index + 1] ?? fallback);
};

const DIST = resolve(flag("dist", "dist")!);
const JSON_OUT = flag("json");
const FONT = flag("font");
const FAIL = argv.includes("--fail");

if (!existsSync(DIST)) {
  console.error(`No build at ${DIST}. Run \`pnpm build\` first — this reports on the artefact.`);
  process.exit(1);
}

const { server, origin } = await serveDist(DIST);

const browser = await chromium.launch();
const rows: Measured[] = [];
const missing: string[] = [];

try {
  for (const width of WIDTHS) {
    const page = await browser.newPage({ viewport: { width, height: 900 } });
    if (FONT != null) {
      /* Applied per page rather than once, and with `!important`, because the site sets `font` on
         `body` as a shorthand and several rules set a family below it. Charts are excluded: their
         layout constants are calibrated to `system-ui` and forcing a face on them would report a
         defect this run invented. */
      await page.addInitScript((family: string) => {
        addEventListener("DOMContentLoaded", () => {
          const style = document.createElement("style");
          style.textContent = `*:not(svg):not(svg *) { font-family: ${family} !important }`;
          document.head.append(style);
        });
      }, FONT);
    }

    for (const route of ROUTES) {
      const response = await page.goto(`${origin}${route}`, { waitUntil: "load" });
      if (response == null || !response.ok()) {
        if (width === WIDTHS[0]) missing.push(route);
        continue;
      }
      const measured = await page.evaluate(collect, {
        proseMinChars: PROSE_MIN_CHARS,
        proseCellMinWords: PROSE_CELL_MIN_WORDS,
        boxMinRadius: BOX_MIN_RADIUS,
      });
      rows.push({ route, width, ...measured });
    }
    await page.close();
  }
} finally {
  await browser.close();
  server.close();
}

/*
 * The clock is read once, here, rather than inside the collector.
 *
 * A report is a thing two runs get diffed against each other, and a timestamp taken per row would
 * make every row differ. This is also the only non-deterministic value in the output, which is
 * worth knowing when the JSON is committed or compared.
 */
const report: Report = { measuredAt: new Date().toISOString(), rows };

console.log(formatReport(report));

if (missing.length > 0) {
  console.log("");
  console.log(`  ${missing.length} route(s) in ROUTES are not in this build, and were skipped:`);
  for (const route of missing) console.log(`    ${route}`);
  console.log("  A route that has been renamed should be renamed in `src/lib/measure.ts` too —");
  console.log("  silently reporting on seven of eight genres is how a genre stops being watched.");
}

if (JSON_OUT != null) {
  const { writeFile } = await import("node:fs/promises");
  await writeFile(resolve(JSON_OUT), `${JSON.stringify(report, null, 2)}\n`);
  console.log(`\n  Written to ${resolve(JSON_OUT)}`);
}

const breaches = violations(report);
if (breaches.length > 0) {
  console.log("");
  console.log(`  ${breaches.length} threshold breach(es):`);
  for (const breach of breaches) {
    console.log(`    ${breach.route} @${breach.width}px — ${breach.message}`);
  }
} else {
  /* Say which thresholds held, not merely that nothing failed. "No thresholds are set" printed
     while one was set and passing, which is a report contradicting itself — the same shape as the
     `_headers` comment that claimed an override the platform does not do. */
  const set = Object.entries(THRESHOLDS);
  console.log("");
  if (set.length === 0) {
    console.log("  No thresholds are set yet. Each redesign phase fills one in; see #183.");
  } else {
    /* `headingRatio` is a FLOOR and every other threshold is a ceiling, so one comparator for all
       of them would have printed the one that matters most here backwards. */
    const shown = set.map(([k, v]) => `${k} ${k === "headingRatio" ? ">=" : "<="} ${v}`);
    console.log(`  ${set.length} threshold(s) set and green: ${shown.join(", ")}`);
    console.log("  The rest of the report is printed and not graded; see #183.");
  }
}

process.exit(FAIL && breaches.length > 0 ? 1 : 0);
