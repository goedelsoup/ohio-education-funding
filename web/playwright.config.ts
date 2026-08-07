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
    command: `pnpm build && pnpm exec vite preview --outDir dist --port ${PORT} --strictPort`,
    url: `http://localhost:${PORT}`,
    // Never inherit a server this run did not start. A stale one serving a stale `dist/` would
    // make the suite pass against the previous build.
    reuseExistingServer: false,
    timeout: 120_000,
  },
});
