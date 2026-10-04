/**
 * Whether this build carries Explained (#718): dev only until the section ships.
 *
 * On when `PUBLIC_EXPLAINED` is `1`, which `pnpm dev` sets unless the shell already set it, so
 * `PUBLIC_EXPLAINED=0 pnpm dev` previews the site as production serves it. A plain `pnpm build`
 * leaves it off, and that is the build CI tests and deploys: no Explained page, card, bar entry or
 * inbound link. The `web explained` job in `ci.yml` builds a second time with it on and runs the
 * suites that read the section, so it is not left untested while it is hidden.
 *
 * Read on every call rather than once, so a test can stub the variable. `process` is reached through
 * `globalThis` because a module that imports this may one day be bundled for the browser, where
 * there is no `process` and the section stays off.
 */
export function explainedOn(): boolean {
  return globalThis.process?.env?.["PUBLIC_EXPLAINED"] === "1";
}
