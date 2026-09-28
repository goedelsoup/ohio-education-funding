/**
 * What the three scenario routes load first: the panel request, and the runner beside it.
 *
 * # Why the runner is not imported here
 *
 * The runner draws its charts with Observable Plot in the browser — see `lib/plot/client.ts` —
 * and Plot and d3 are about 300 KB of it. Imported statically, all of that had to arrive and be
 * evaluated before this script could ask for the panel, and nothing can be drawn until the panel is
 * here. So the two requests were made one after the other when neither needs the other.
 *
 * Imported dynamically, the runner is its own chunk and is fetched *alongside* the panel: the page
 * waits for the slower of the two rather than for their sum. This file carries nothing a browser
 * has to wait on — no formula, no chart — which is the whole of the reason it is separate.
 *
 * It does not keep Plot off `/reach` until a lever moves. That page draws its cloud at rest, and
 * draws it from the formula run in the browser, so the library is needed before anything is shown.
 */

import { escapeHtml } from "../lib/format.ts";
import type { Panel } from "../lib/types.ts";

// The slim panel, not the full feed: this page needs every district's formula inputs and none of
// their audited finances or report cards. See `src/pages/data/panel.json.ts`.
const PANEL = `${import.meta.env.BASE_URL}data/panel.json`;

/** Which of the two requests failed, so the card below does not blame the panel for the script. */
class Unreachable extends Error {
  constructor(
    readonly what: string,
    cause: unknown,
  ) {
    super(cause instanceof Error ? cause.message : String(cause));
  }
}

const panel = fetch(PANEL)
  .then((response) => {
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json() as Promise<Panel>;
  })
  .catch((error: unknown) => {
    throw new Unreachable(PANEL, error);
  });

const runner = import("./scenario.ts").catch((error: unknown) => {
  throw new Unreachable("scripts/scenario.ts", error);
});

Promise.all([panel, runner])
  .then(([loaded, { boot }]) => boot(loaded))
  .catch((error: unknown) => {
    // A throw inside `boot` is reported against the panel, as it always was: the runner is here,
    // so what it could not make sense of is the data.
    const what = error instanceof Unreachable ? error.what : PANEL;
    const message = error instanceof Error ? error.message : String(error);
    const out = document.querySelector("#scenario-out");
    if (out) {
      out.innerHTML = `<div class="card err" id="panel-unreachable" data-part="panel-unreachable">
        <p>Could not load <code>${escapeHtml(what)}</code> (${escapeHtml(message)}).</p>
        <p class="note">Every other page on this site carries its figures in the document and is
          unaffected. This one re-runs the formula, so it needs the panel.</p>
      </div>`;
    }
  });

/*
 * Keep Enter from reloading the page.
 *
 * This was `onsubmit="return false"` on the form itself, which `script-src 'self'` blocks — an
 * inline event handler is inline script. The violation only appears where the CSP is actually
 * applied, which is the deployed site and never `vite preview`, so it shipped. See the built-output
 * check in `tests/e2e/`.
 *
 * Here rather than in the runner so it holds from the first paint, before the runner has arrived.
 */
document
  .querySelector<HTMLFormElement>("#scenario-controls")
  ?.addEventListener("submit", (event) => event.preventDefault());
