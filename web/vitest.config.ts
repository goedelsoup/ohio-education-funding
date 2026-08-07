/// <reference types="vitest/config" />
import { getViteConfig } from "astro/config";

/**
 * The formula suite.
 *
 * Through `getViteConfig` rather than a bare Vitest config, so these modules are resolved and
 * transformed exactly as the shipped bundle resolves and transforms them. A test that passes
 * under different resolution rules than the browser gets is testing something adjacent to the
 * thing that ships.
 *
 * No environment is requested: this is arithmetic over the committed feed, it touches no DOM,
 * and it should not pay for one. The page itself is covered by Playwright, against a real build.
 */
export default getViteConfig({
  test: {
    // Narrow on purpose. Vitest's default glob would also collect `tests/e2e/*.spec.ts`, which
    // are Playwright's and fail in confusing ways when run by anything else.
    include: ["tests/unit/**/*.spec.ts"],
    environment: "node",
  },
});
