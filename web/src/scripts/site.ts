/**
 * The chrome, on every page.
 *
 * Deliberately small, and deliberately not load-bearing. Every figure is in the HTML before this
 * runs; what this adds is the hover layer and the theme switch. A reader with JavaScript off
 * loses two tooltips and keeps every number.
 *
 * The nominal/constant-dollar toggle is *not* here. It is two radio inputs and a sibling
 * selector in `app.css`, so it works with nothing running — see `BasisToggle.astro`.
 */

import { attachHover } from "../lib/chart.ts";

const tip = document.querySelector<HTMLElement>("#tip");
if (tip) attachHover(document.body, tip);

/**
 * Honour the links the old single-page site handed out.
 *
 * For its whole life this platform was one document that routed on the URL fragment:
 * `#district/043786`, `#statewide`, `#scenario?g=phase-out&arg=0.5`. Those links were the point —
 * "a scenario worth arguing about is worth being able to send someone" — and some of them are in
 * emails and board packets now. The routes moved out of the fragment and into the path; a link
 * that used to work should not now land on the front page with no explanation.
 *
 * Only ever a redirect from the root, and only for fragments that were real routes. A fragment on
 * any other page is an anchor and is left alone.
 */
if (location.pathname === "/" && location.hash.length > 1) {
  const [route, query] = location.hash.slice(1).split("?");
  const [view, irn] = (route ?? "").split("/");
  const search = query ? `?${query}` : "";
  const moved: Record<string, string> = {
    statewide: "/",
    outcomes: "/outcomes",
    scenario: `/scenario${search}`,
  };
  const target =
    view === "district" && /^\d{6}$/.test(irn ?? "") ? `/district/${irn}` : moved[view ?? ""];
  // `/` is in the table so a `#statewide` link is recognised rather than falling through, but
  // redirecting the front page to itself would be a loop.
  if (target && target !== location.pathname) location.replace(target);
}

/**
 * The theme switch.
 *
 * `data-theme` on the root beats the OS setting in both directions, which the stylesheet already
 * arranges. Applied from a deferred module rather than an inline script in `<head>`, because the
 * page's CSP has no `script-src 'unsafe-inline'` and keeping it that way is worth more than the
 * flash a reader who has explicitly chosen the non-OS theme will see. A reader who has never
 * touched this sees no flash at all: with nothing stored, the OS setting is already correct.
 */
const KEY = "theme";
const root = document.documentElement;

const stored = localStorage.getItem(KEY);
if (stored === "light" || stored === "dark") root.dataset.theme = stored;

document.querySelector("#theme")?.addEventListener("click", () => {
  const dark = matchMedia("(prefers-color-scheme: dark)").matches;
  const current = root.dataset.theme ?? (dark ? "dark" : "light");
  const next = current === "dark" ? "light" : "dark";
  root.dataset.theme = next;
  // Store the choice even when it agrees with the OS: the reader picked it, and an OS that later
  // changes should not silently overrule them.
  localStorage.setItem(KEY, next);
});
