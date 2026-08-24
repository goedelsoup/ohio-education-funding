/**
 * The chrome, on every page.
 *
 * Deliberately small, and deliberately not load-bearing. Every figure is in the HTML before this
 * runs; what this adds is the hover layer, the theme switch, and the closing half of the section
 * menus. A reader with JavaScript off loses two tooltips, keeps every number, and keeps a
 * navigation that opens.
 *
 * The nominal/constant-dollar toggle is *not* here. It is two radio inputs and a sibling
 * selector in `app.css`, so it works with nothing running — see `BasisToggle.astro`.
 */

import { attachValues, openToKeyboard } from "../lib/chart.ts";

/*
 * The value layer: a tooltip for a pointer, a tap for a touch screen, and an arrow-key cursor for
 * a keyboard. Every chart's marks already carry their value as `data-hover`; until this, only a
 * mouse could get at it.
 *
 * The tab stops are set here rather than baked into the build because the cursor they advertise
 * is this script. The scenario charts set their own — they are drawn in the browser, so the
 * question does not arise there.
 */
const tip = document.querySelector<HTMLElement>("#tip");
const said = document.querySelector<HTMLElement>("#said");
if (tip && said) {
  attachValues(document.body, tip, said);
  for (const chart of document.querySelectorAll("svg.plot")) openToKeyboard(chart);
}

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
 *
 * # Why the table is a `Map` and not an object literal
 *
 * Because the lookup key comes out of the URL, and an object literal answers for every name on
 * `Object.prototype`. `/#toString` read `moved["toString"]` and got `Function.prototype.toString`
 * — truthy, and not equal to `location.pathname`, so both guards passed and `location.replace`
 * was handed a function. It stringifies to `function toString() { [native code] }`, which
 * resolves as a relative URL: a reader following `/#toString` or `/#constructor` was bounced off
 * the homepage into a 404 on a path made of source code.
 *
 * A `Map` has no prototype keys, so `has`/`get` answer for what was put in and nothing else.
 * `Object.create(null)` would work too; a `Map` says the intent in the type rather than in a
 * comment nobody re-reads.
 */
if (location.pathname === "/" && location.hash.length > 1) {
  const [route, query] = location.hash.slice(1).split("?");
  const [view, irn] = (route ?? "").split("/");
  const search = query ? `?${query}` : "";
  const moved = new Map<string, string>([
    ["statewide", "/"],
    ["outcomes", "/outcomes"],
    ["scenario", `/scenario${search}`],
  ]);
  const target =
    view === "district" && /^\d{6}$/.test(irn ?? "")
      ? `/district/${irn}`
      : moved.get(view ?? "");
  // `/` is in the table so a `#statewide` link is recognised rather than falling through, but
  // redirecting the front page to itself would be a loop.
  if (target && target !== location.pathname) location.replace(target);
}

/**
 * The section menus, which already work.
 *
 * They are `<details>` elements, so they open and close on their own with nothing running. What
 * a browser will not do unaided is close one when another opens, when Escape is pressed, or when
 * the reader clicks elsewhere — three behaviours a menu is expected to have and none of which it
 * is broken without. A reader with this script off gets two menus open at once.
 */
const menus = [...document.querySelectorAll<HTMLDetailsElement>("header.site nav details.menu")];

const closeMenus = (except?: HTMLDetailsElement) => {
  for (const menu of menus) if (menu !== except) menu.open = false;
};

for (const menu of menus) {
  // `toggle` rather than `click`: it fires for the keyboard and for a programmatic change too,
  // and `click` on a summary fires before the element has actually opened.
  menu.addEventListener("toggle", () => {
    if (menu.open) closeMenus(menu);
  });
}

if (menus.length > 0) {
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    const open = menus.find((menu) => menu.open);
    if (!open) return;
    open.open = false;
    // Focus goes back to the control that opened it, or the reader is left at the top of the
    // document having done nothing but press a key.
    open.querySelector("summary")?.focus();
  });

  document.addEventListener("click", (event) => {
    const target = event.target;
    if (target instanceof Node && menus.some((menu) => menu.contains(target))) return;
    closeMenus();
  });

  // Tabbing out of a menu closes it, which is what a sighted keyboard reader expects to see.
  document.addEventListener("focusin", (event) => {
    const target = event.target;
    if (target instanceof Node && menus.some((menu) => menu.contains(target))) return;
    closeMenus();
  });
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

/**
 * Which theme is actually in force — the stored choice, or the operating system's.
 *
 * The same expression the click handler needed anyway, named because the pressed state below has
 * to agree with it at load. It cannot be baked: a static build knows neither half.
 */
const isDark = (): boolean =>
  (root.dataset.theme ?? (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light")) ===
  "dark";

const toggle = document.querySelector<HTMLButtonElement>("#theme");
/*
 * The button carried no state at all: it announced "Switch colour theme, button" whether the page
 * was light or dark, and activating it announced nothing, so a screen reader reader had no way to
 * know either what the theme was or that pressing it had done anything. `aria-pressed` answers
 * both — a toggle button's state change is announced by the screen reader on its own.
 */
toggle?.setAttribute("aria-pressed", String(isDark()));

toggle?.addEventListener("click", () => {
  const next = isDark() ? "light" : "dark";
  root.dataset.theme = next;
  toggle.setAttribute("aria-pressed", String(next === "dark"));
  // Store the choice even when it agrees with the OS: the reader picked it, and an OS that later
  // changes should not silently overrule them.
  localStorage.setItem(KEY, next);
});
