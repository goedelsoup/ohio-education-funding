/// <reference types="vitest/config" />
import { getViteConfig } from "astro/config";

/**
 * Two suites that need no browser, in two projects, because one of them needs a build first.
 *
 * Through `getViteConfig` rather than a bare Vitest config, so these modules are resolved and
 * transformed exactly as the shipped bundle resolves and transforms them. A test that passes
 * under different resolution rules than the browser gets is testing something adjacent to the
 * thing that ships.
 *
 * No environment is requested by either: this is arithmetic over the committed feed and text
 * scans over the built files, neither touches a DOM, and neither should pay for one. The page
 * itself is covered by Playwright, against a real build.
 *
 * # Why `dist` is a project and not more files under `tests/unit`
 *
 * `unit` may run at any time; `dist` reports on `dist/` and so is only meaningful **after** a
 * build. Keeping them in one `include` would mean `pnpm test:unit` — which runs before `build` in
 * the gate, deliberately, so a formula defect reads as a formula failure — scanning the previous
 * commit's output. That is the hazard `scripts/check-dist-links.ts` documents at length, and the
 * two projects are how it is made impossible here rather than merely warned about.
 *
 * So there is no script that runs both: `test:unit` is `--project unit` and `test:dist` is
 * `--project dist`, and `web/mise.toml` orders the second after `build`. A bare `vitest run` runs
 * both and will fail loudly with no build — see the guard in `tests/dist/artefact.ts`.
 */
export default getViteConfig({
  test: {
    projects: [
      {
        extends: true,
        test: {
          name: "unit",
          // Narrow on purpose. Vitest's default glob would also collect `tests/e2e/*.spec.ts`,
          // which are Playwright's and fail in confusing ways when run by anything else.
          include: ["tests/unit/**/*.spec.ts"],
          environment: "node",
        },
      },
      {
        extends: true,
        test: {
          name: "dist",
          include: ["tests/dist/**/*.spec.ts"],
          environment: "node",
          /*
           * Vitest's default is 5s, and one of these needs eighteen.
           *
           * Every assertion in this project sweeps the whole build — 3,500 documents — and the
           * expensive one parses each of them with linkedom rather than scanning for a pattern,
           * because what it asks is about the DOM the parser produces. Under Playwright, whose
           * default is 30s, it had about twelve seconds of headroom and nobody had to know.
           *
           * The number is not a target. It is enough that a contended machine does not turn a
           * green sweep red; the work is bounded by the size of the build, which is reported by
           * `pnpm measure`.
           */
          testTimeout: 120_000,
        },
      },
    ],
  },
});
