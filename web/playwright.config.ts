import { defineConfig, devices } from "@playwright/test";

/**
 * The page, in a browser, against the built site.
 *
 * Deliberately the *built* site rather than the dev server — the property this project cares
 * about is that `dist/` is deployable as static files, and the only way to know that holds is to
 * serve it the way a host would.
 *
 * The formula is not tested here. That is Vitest's job (`vitest.config.ts`), and it needs
 * neither a browser nor a build; what this suite is for is the things that only exist once a
 * browser has run the code.
 */

// Not Astro's default 4321: that port collects other projects' dev servers, and a suite that
// silently tests whatever answered is worse than one that cannot start.
const PORT = 4329;

const PREVIEW = `pnpm exec vite preview --outDir dist --port ${PORT} --strictPort`;

export default defineConfig({
  testDir: "./tests/e2e",
  fullyParallel: true,
  forbidOnly: !!process.env["CI"],
  retries: process.env["CI"] ? 2 : 0,
  reporter: process.env["CI"] ? [["github"], ["html", { open: "never" }]] : [["list"]],

  use: {
    baseURL: `http://localhost:${PORT}`,
    trace: "on-first-retry",
  },

  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],

  webServer: {
    // `vite preview` rather than `astro preview`, which daemonizes itself and so outlives the
    // run that started it. This stays in the foreground and dies with Playwright.
    //
    // On CI the build in front of it is gone: the workflow has already built, and three steps
    // between the build and this one — `check:dist`, `test:dist` and `measure` — read `dist/`
    // directly. Building again here served Chromium a *second* build and left those three
    // reporting on the first, which is the one difference a reader of the artefacts cannot see.
    // Now there is one build per run and everything, the browser included, sees the same bytes.
    //
    // Locally the build stays, because locally there is usually no build to serve. `pnpm test:e2e`
    // on a clean checkout has to be a command that works.
    command: process.env["CI"] ? PREVIEW : `pnpm build && ${PREVIEW}`,
    url: `http://localhost:${PORT}`,
    // Never inherit a server this run did not start. A stale one serving a stale `dist/` would
    // make the suite pass against the previous build.
    reuseExistingServer: false,
    /*
     * Long enough for whatever the command in front of it is, on the slowest machine that runs it.
     *
     * This was 120s when the command was `pnpm build && vite preview`, and the window has to hold
     * a full build rather than a server start. A GitHub runner builds this site in 2m 24s now,
     * against roughly 1m 50s before charts were drawn at two widths, and the whole e2e step failed
     * with `Timed out waiting 120000ms from config.webServer` rather than with anything a reader
     * would recognise as "the build takes longer than the limit".
     *
     * On CI the build is no longer in front of it, so the window only has to hold `vite preview`
     * binding a port. The generous number stays where the build still runs. Neither is a target:
     * each is enough headroom that a contended machine does not turn a green suite red, and the
     * build's actual cost is tracked where it belongs — see #111.
     */
    timeout: process.env["CI"] ? 60_000 : 300_000,
  },
});
