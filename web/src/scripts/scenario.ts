import { modelOf } from "../lib/policy.ts";
/**
 * The scenario builder, in the browser.
 *
 * # Why these three routes still compute client-side when nothing else does
 *
 * Every other page on this site is baked: its figures are written into the HTML at build time and
 * no formula runs in the browser. These three cannot be. A lever has a continuum of positions, and
 * a static page per position is not a page — so the formula runs here, over the whole
 * 609-district panel, on every slider tick.
 *
 * The three are `/scenario`, `/district/[irn]/scenario` and `/reach`, and which one this is comes
 * off the page rather than out of the URL — see `irn` and `view`.
 *
 * # The verification gate, which is why that is allowed
 *
 * `policy.ts` is a second implementation of `crates/project/src/policy.rs`. Two implementations of
 * one formula is normally a bad trade — they drift, and the one nobody runs is the one that is
 * wrong. What makes it acceptable is that the panel carries Rust-computed checkpoints and this
 * refuses to render a scenario until it reproduces every one of them, against the real panel, to
 * within a dollar across seven billion.
 *
 * The build runs the same check and fails if it does not pass, so a drifted formula never ships.
 * This one runs anyway, on the served panel, because the two artefacts can diverge after the
 * build: the panel is a separate file and can be replaced without one. A gate that only ever ran
 * where the answer was already known would be decorative.
 *
 * When it fails, the tab is not merely left empty — an explanation replaces it, naming which
 * scenario disagreed and by how much. The failure mode this guards is a plausible wrong number,
 * which looks exactly like a right one.
 */

import { escapeHtml } from "../lib/format.ts";
import {
  clampLevers,
  defaultLevers,
  draftLevers,
  renderDistrictScenario,
  renderDraft,
  renderProjection,
  renderScenario,
  type HorizonBound,
  type Levers,
} from "../lib/scenario.ts";
import { pct } from "../lib/format.ts";
import type { Panel } from "../lib/types.ts";
import { REQUIRED_CONTRACT } from "../lib/types.ts";
import {
  DEFAULT_VIEW,
  PRESET_FIELDS,
  renderReach,
  viewFromQuery,
  type View,
} from "../lib/reach.ts";
import { isForecastVerified, isVerified, verify, type Verification } from "../lib/verify.ts";
import { heading } from "../lib/section.ts";
import { saying, tileSummary } from "../lib/status.ts";

const $ = <T extends HTMLElement>(selector: string): T | null =>
  document.querySelector<T>(selector);

/*
 * What the page says once the levers stop moving.
 *
 * Every render below replaces the whole result block, and until this there was nothing telling a
 * reader who cannot see it that anything had happened. Debounced, because a slider drag is fifty
 * `input` events — see `saying`.
 */
const changed = $("#changed");
const say = changed ? saying(changed) : () => {};

const root = $("#scenario-root");
/** Present on the district route, absent on the other two. */
const irn = root?.dataset.irn;

/**
 * Which of the three views this script is driving, from the page that carries it.
 *
 * `/scenario` is the default and names nothing. `/reach` sets `data-view="reach"` and renders the
 * cloud alone: it asks which districts a lever reaches rather than by how much, which is a
 * different question on a formula that pays the larger of two numbers, and it is the one view
 * that still says something at rest — the guarantee wall is there before any lever moves.
 *
 * A page attribute rather than a `location.pathname` test, for the reason `irn` is one: this file
 * is the runner for whatever page imports it, and a runner that reads the URL to find out which
 * page it is on cannot be put on a third without being edited.
 */
const view = root?.dataset.view ?? "";

/**
 * The draft this page was opened from, if any.
 *
 * Read once at load and never cleared. It is not lever state: a reader who moves a slider has
 * departed from the bill rather than stopped reading it, and `renderDraft` says which — so
 * forgetting the slug on the first `input` event would silently turn "this is no longer the
 * draft" into no message at all.
 */
const draftSlug = new URLSearchParams(location.search).get("draft") ?? "";

/**
 * The year chip, rendered into a `<template>` by the page that carries this script.
 *
 * `yearChip` reads the feed through `loadFeed`, which touches the filesystem, so nothing rendered
 * in the browser could build one — which is why every card on these routes carried figures
 * under no year at all. The chip rule never caught it: `/scenario` is not in the sweep's route
 * list, and the district route passes it in a state where the only card rendered has no figures in
 * it.
 *
 * Read once. It is a property of the feed, not of the levers.
 */
const chip = $<HTMLTemplateElement>("#scenario-chip")?.innerHTML ?? "";

interface State {
  panel: Panel;
  verification: Verification;
  levers: Levers;
  /**
   * How the reach view draws, which is not lever state.
   *
   * Choosing an axis changes what is plotted, not what is computed — the formula runs the same
   * either way — so these travel with the levers in the query string but never reach `toPolicy`.
   * The distinction is worth keeping in the type: a dimension that could reach the model would be
   * a chooser that changes the answer.
   */
  view: View;
}

let state: State | null = null;

/**
 * Read the levers out of the query string.
 *
 * `?g=phase-out&arg=0.5&base=1.05`. A scenario worth arguing about is worth being able to send
 * someone, and on a static host the query string is the only part of a URL that can carry state
 * without minting a page for every combination.
 */
function fromQuery(horizon: HorizonBound): Partial<Levers> {
  const params = new URLSearchParams(location.search);
  const levers: Partial<Levers> = {};
  const rule = params.get("g");
  if (rule === "removed" || rule === "rebase" || rule === "phase-out" || rule === "as-enacted") {
    levers.guarantee = rule;
  }
  for (const [key, field] of [
    ["arg", "guaranteeArgument"],
    ["base", "baseCostScale"],
    ["min", "minimumStateShare"],
    ["pb", "phaseInGeneral"],
    ["pc", "phaseInDpia"],
    ["dc", "dpiaBlend"],
    ["sta", "supplementalTopRate"],
    ["tf", "transportationFloor"],
    ["h", "horizon"],
  ] as const) {
    const raw = params.get(key);
    if (raw == null) continue;
    levers[field] = Number(raw);
  }
  /*
   * Held to the lever bounds here rather than by the controls downstream.
   *
   * This used to return whatever parsed, and the range inputs did the clamping on the way back out
   * through `readLevers`. That covered every path but one: `?draft=` renders before the first read
   * of a control, on purpose, so `?draft=x&h=999999` reached `forecastPath` with a million-year
   * horizon and locked the tab, and `?draft=x&base=100` rendered $928B under a slider reading 1.3.
   *
   * Clamping at the boundary makes the two paths agree instead of making one of them defend the
   * other. See `clampLevers` for why the step is not enforced with the ends.
   */
  return clampLevers(levers, horizon);
}

/** Put the current levers in the query string, without adding a history entry per tick. */
function toQuery(): void {
  if (!state) return;
  const l = state.levers;
  const params = new URLSearchParams({
    g: l.guarantee,
    arg: String(l.guaranteeArgument),
    base: String(l.baseCostScale),
    min: String(l.minimumStateShare),
    pb: String(l.phaseInGeneral),
    pc: String(l.phaseInDpia),
    dc: String(l.dpiaBlend),
    sta: String(l.supplementalTopRate),
    tf: String(l.transportationFloor),
  });
  // Only where a horizon can be set. The district route has no such control, draws no band, and
  // reads nothing from `h` — so every URL it minted carried `h=2032` for a reader to copy and send.
  if ($("#lv-horizon")) params.set("h", String(l.horizon));
  /*
   * The view, only where there is one to read.
   *
   * These name what is plotted rather than what is computed, so they are not lever state — but a
   * cloud is worth sending someone in the pair of axes it was read in, and on a static host the
   * query string is the only place that can live. Written only on the page that has the controls,
   * for the same reason `h` is.
   */
  if (view === "reach") {
    const v = state.view;
    params.set("vx", v.x);
    params.set("vy", v.y);
    params.set("c", v.shading);
    /*
     * The scope, only when there is one. A bare `/reach` is the whole state and says so by carrying
     * nothing, the way the trails do.
     *
     * Keys of their own rather than a share of an existing one — see the account just below of what
     * happened the last time two meanings were written to one parameter on this page.
     */
    if (v.counties.length > 0) params.set("co", v.counties.join(","));
    if (v.districts.length > 0) params.set("d", v.districts.join(","));
    /*
     * Only where it means something. A `ty=` on a cloud coloured by regime is a parameter the page
     * is not using, and a link carrying it would suggest otherwise.
     *
     * `ty` and not `g`. This was `params.set("g", …)`, and `g` is the guarantee rule's key two
     * dozen lines above — `set` overwrites, so colouring the cloud by district type replaced the
     * lever in the URL with a typology code, and `fromQuery` then read `g=1` as no rule at all and
     * fell back to `as-enacted`. Every link this page minted under that colouring silently dropped
     * the reader's guarantee setting. See `viewFromQuery` for the whole account.
     */
    if (v.shading === "type") params.set("ty", String(v.highlight));
    if (!v.trails) params.set("t", "0");
  }
  // `draft` survives every lever move. It is not lever state — see `draftSlug` — and dropping it
  // here would make the URL in the bar stop being the one that opened the bill, while the page
  // was still explaining that these figures came from one.
  if (draftSlug) params.set("draft", draftSlug);
  const next = `${location.pathname}?${params.toString()}`;
  if (location.pathname + location.search !== next) history.replaceState(null, "", next);
}

/**
 * Hand the reader's lever positions to the other view.
 *
 * `/scenario` and `/reach` run the same levers and ask different questions of them, so a link
 * between the two that dropped the settings would be offering the reader the other question about
 * a scenario they are no longer looking at. `toQuery` has just written the levers into the address
 * bar, so the current search string is exactly the state to hand over.
 *
 * Marked on the anchor rather than composed where the anchor is written: both links are inside
 * containers this script replaces wholesale on every render, so the href has to be re-applied
 * afterwards anyway, and an attribute is what survives being re-rendered.
 *
 * `link.pathname` rather than the stored href, so this is idempotent — a pathname carries no
 * query, and re-running it replaces the search rather than appending a second one.
 *
 * The horizon does not cross. `/reach` draws no band and carries no horizon control, so `toQuery`
 * omits `h` there exactly as it does on the district route; a reader who had set a forecast year
 * gets it back by setting it again. The alternative is carrying a lever the page cannot show.
 */
function carryLevers(): void {
  for (const link of document.querySelectorAll<HTMLAnchorElement>("a[data-carry-levers]")) {
    link.href = `${link.pathname}${location.search}`;
  }
}

/*
 * `backstop` has no control and is carried rather than read.
 *
 * It is a draft-only lever: Section 265.225 either stands or is repealed, which is a clause of a
 * bill rather than a dial a reader would sweep. Reading it from the DOM would reset a draft's
 * position the moment any other control was touched, and `renderDraft` would report a departure
 * the reader did not make — the failure the comment on `update` describes. So it falls back the
 * way `horizon` does when its control is absent.
 */
function readLevers(fallbackHorizon: number, fallbackBackstop: Levers["backstop"]): Levers {
  const number = (id: string) => Number($<HTMLInputElement>(id)!.value);
  return {
    guarantee: $<HTMLSelectElement>("#lv-guarantee")!.value as Levers["guarantee"],
    guaranteeArgument: number("#lv-arg"),
    baseCostScale: number("#lv-base"),
    minimumStateShare: number("#lv-min"),
    phaseInGeneral: number("#lv-phase"),
    phaseInDpia: number("#lv-phase-dpia"),
    dpiaBlend: number("#lv-dpia-blend"),
    supplementalTopRate: number("#lv-supplemental"),
    transportationFloor: number("#lv-transport"),
    horizon: $("#lv-horizon") ? number("#lv-horizon") : fallbackHorizon,
    backstop: fallbackBackstop,
  };
}

/**
 * Mark the preset whose settings the levers are currently sitting on.
 *
 * Derived from the lever values on every render rather than remembered from the click, so a
 * slider dragged by hand onto a preset's position reads as that preset, and one nudged a step off
 * it stops reading as any — which is the honest report. A preset names only the fields it sets, so
 * the comparison is over those fields alone: "Retire half the floor" is still in force when the
 * reader has also moved base cost, because it never said anything about base cost.
 */
function syncPresets(levers: Levers): void {
  for (const button of document.querySelectorAll<HTMLButtonElement>("[data-preset]")) {
    const wanted = JSON.parse(button.dataset.preset ?? "{}") as Levers;
    const on = PRESET_FIELDS.every((field) => {
      const a = levers[field];
      const b = wanted[field];
      /*
       * Within half a step, not equal.
       *
       * The sliders are quantized — base cost steps by 0.01 from 0.8 — and a preset's value need
       * not land on a step: the refresh the feed prices is 1.0395, which the control rounds to
       * 1.04. An equality here would leave "Fund the plan" reading unpressed the instant it was
       * pressed, which is the same rounding the draft path already documents at `fromControls`.
       */
      return typeof a === "number" && typeof b === "number" ? Math.abs(a - b) < 0.005 : a === b;
    });
    button.setAttribute("aria-pressed", String(on));
  }
}

/** The reach view, off its own controls. Absent controls mean the defaults, as on every route. */
function readView(): View {
  const pick = <T extends string>(id: string, fallback: T): T => {
    const control = $<HTMLSelectElement>(id);
    return control ? (control.value as T) : fallback;
  };
  const lit = Number($<HTMLSelectElement>("#rv-group")?.value);
  return {
    x: pick("#rv-x", DEFAULT_VIEW.x),
    y: pick("#rv-y", DEFAULT_VIEW.y),
    shading: pick("#rv-shading", DEFAULT_VIEW.shading),
    highlight: Number.isFinite(lit) ? lit : DEFAULT_VIEW.highlight,
    trails: $<HTMLInputElement>("#rv-trails")?.checked ?? DEFAULT_VIEW.trails,
    /*
     * The scope, from the DOM on every tick like everything else in this function.
     *
     * The counties are the checked boxes. The named districts are a hidden field rather than a set
     * held in this module, and that is not a shortcut: `update` re-reads the controls on every
     * `input`, so a selection kept only in `state` would be overwritten by whatever the DOM said the
     * first time a reader touched a slider — the failure the note on the view's restore describes.
     */
    /*
     * Both are interpolated into attribute selectors below, which is safe because both are held to a
     * shape at every entrance: a slug is `[a-z0-9-]+` because `slugify` built it, an IRN is six
     * digits because that is the only key a district has, and `viewFromQuery` holds a query string
     * to the same two. Nothing here needs escaping and nothing here may stop being shaped.
     */
    counties: [...document.querySelectorAll<HTMLInputElement>('#reach-scope input[name="co"]:checked')].map(
      (box) => box.value,
    ),
    districts: named(),
  };
}

/** The individually named districts, as the hidden field carries them. */
function named(): string[] {
  const raw = $<HTMLInputElement>("#rv-districts")?.value ?? "";
  return raw.split(",").filter((irn) => irn !== "");
}

/** Write the named districts back, and hand authority to the same field `readView` reads. */
function setNamed(irns: string[]): void {
  const field = $<HTMLInputElement>("#rv-districts");
  if (field) field.value = [...new Set(irns)].join(",");
}

/**
 * Show the group picker only when the colour is the department's grouping.
 *
 * Hidden rather than disabled: a select offering nine categories that change nothing is worse than
 * no select, because it looks like a control whose effect the reader has failed to notice. This is
 * the same rule `syncLabels` applies to the retained-share slider, which means nothing for two of
 * the four guarantee rules.
 */
function syncView(view: View): void {
  const pick = $("#rv-group-pick");
  if (pick) pick.hidden = view.shading !== "type";
  syncScope(view);
}

/**
 * The scope, as the reader sees it: a chip per selection and a count on the disclosure.
 *
 * Rendered from the view rather than remembered from the click, the way `syncPresets` is and for the
 * same reason — a scope that arrived in the query string has to look exactly like one a reader built
 * by hand, and there is only one code path if the state is derived.
 *
 * The chip row is hidden when there is no scope rather than showing a row that says "Every
 * district". A chip is a thing you can take off; one that cannot be is a label pretending to be a
 * control.
 *
 * A county's label comes off its own checkbox and a district's off the picker's option, so neither
 * is spelled twice. Both were written at build time from the feed, which is what that `<label>` text
 * and that `<option>` text are: the department's own names, with `qualifiedName`'s county qualifier
 * on the 28 districts whose name is shared.
 *
 * # Why it does not rebuild on every tick
 *
 * `update` runs on every `input`, and a slider drag is fifty of them. The chip row is an
 * `aria-live` region, and `replaceChildren` is a change to it whether or not anything differs — so
 * rebuilding unconditionally would announce the whole selection fifty times to a screen reader while
 * the reader dragged a lever that has nothing to do with it. The same reason `saying` is debounced.
 *
 * Compared as a string because that is what the row is a function of. Two selections that print the
 * same print the same.
 */
let drawnScope = "";

function syncScope(view: View): void {
  const chips = $("#rv-chips");
  const summary = $("#rv-county-count");
  if (summary) {
    summary.textContent =
      view.counties.length === 0
        ? `all ${document.querySelectorAll('#reach-scope input[name="co"]').length}`
        : `${view.counties.length} selected`;
  }
  if (!chips) return;
  const key = `${view.counties.join(",")}|${view.districts.join(",")}`;
  if (key === drawnScope) return;
  drawnScope = key;
  const empty = view.counties.length === 0 && view.districts.length === 0;
  chips.hidden = empty;
  if (empty) {
    chips.replaceChildren();
    return;
  }

  const chip = (label: string, kind: "county" | "district", value: string): HTMLButtonElement => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "chip";
    button.dataset[kind] = value;
    button.append(label);
    /* A mark that says it comes off, and out of the accessibility tree — the whole action is on
       `aria-label`, because "Allen County" alone says nothing about what pressing it does. */
    const off = document.createElement("span");
    off.className = "chip-off";
    off.setAttribute("aria-hidden", "true");
    off.textContent = "\u00d7";
    button.append(off);
    button.setAttribute("aria-label", `Remove ${label} from the selection`);
    return button;
  };

  const label = (selector: string, fallback: string): string =>
    $(selector)?.textContent?.trim() ?? fallback;

  chips.replaceChildren(
    ...view.counties.map((slug) =>
      chip(
        `${label(`#reach-scope input[name="co"][value="${slug}"] + span`, slug)} County`,
        "county",
        slug,
      ),
    ),
    ...view.districts.map((irn) =>
      chip(label(`#rv-add option[value="${irn}"]`, irn), "district", irn),
    ),
  );

  const clear = document.createElement("button");
  clear.type = "button";
  clear.className = "chip chip-clear";
  clear.id = "rv-clear";
  clear.textContent = "Every district";
  clear.setAttribute("aria-label", "Clear the selection and light every district");
  chips.append(clear);
}

function syncLabels(levers: Levers, baseYear: number): void {
  /*
   * The `<output>` beside a lever's name, and the same string as the slider's spoken value.
   *
   * A range input announces `value`, and these carry formula multipliers: the base-cost slider
   * runs 0.8 to 1.3 and the page shows what the reader moved it to as `+4%`. So a screen reader
   * read out "1.04" where the label said "+4%", and the minimum-state-share slider read "0.05"
   * against "5%" — the raw argument to the formula rather than the quantity the page is about.
   *
   * `aria-valuetext` replaces the announced value with the text, which is what it exists for, and
   * writing both here rather than in two places is what stops them disagreeing.
   */
  const set = (id: string, text: string) => {
    const output = $<HTMLOutputElement>(id);
    if (output) output.textContent = text;
    // `#lv-arg-out` names the output; the control it belongs to is `#lv-arg`.
    $(id.replace(/-out$/, ""))?.setAttribute("aria-valuetext", text);
  };
  set("#lv-arg-out", pct(levers.guaranteeArgument, 0));
  set(
    "#lv-base-out",
    `${levers.baseCostScale >= 1 ? "+" : "−"}${pct(Math.abs(levers.baseCostScale - 1), 0)}`,
  );
  set("#lv-min-out", pct(levers.minimumStateShare, 0));
  set("#lv-phase-out", pct(levers.phaseInGeneral, 0));
  set("#lv-phase-dpia-out", pct(levers.phaseInDpia, 0));
  set("#lv-horizon-out", levers.horizon <= baseYear ? "not projected" : `FY${levers.horizon}`);
  // The retained-share slider only means anything for the two rules that take an argument.
  const argument = $<HTMLInputElement>("#lv-arg")?.closest(".lever") as HTMLElement | null;
  if (argument) {
    argument.hidden = levers.guarantee === "as-enacted" || levers.guarantee === "removed";
  }
}

function render(): void {
  if (!state) return;
  // An unverified panel never reaches here: `reportFailure` has already written that panel, and
  // rendering over it would put the page back exactly as if nothing were wrong — on the one view
  // the check exists to hold shut.
  if (!isVerified(state.verification)) return;
  const out = $("#scenario-out");
  const detail = $("#scenario-detail");
  /*
   * Prepended rather than appended: the provisions this page cannot price belong above the total,
   * not under it. See `renderDraft`.
   *
   * Written on **both** routes, and the district one is the route that most needs it. A district
   * page is the one a school board sends, and it answers "what would this bill do to us" with a
   * dollar figure and a per-pupil figure — so a draft's levers applied there with no card naming
   * the clauses the model cannot reach is the invariant's worst failure, not its mildest. The
   * first version of this wrote the banner only in the statewide branch.
   */
  const banner = draftSlug ? renderDraft(state.panel, state.levers, draftSlug) : "";
  if (view === "reach") {
    // No projection container on this page and no detail half: the forecast is a different claim
    // and the cloud is the whole of this one.
    if (out) out.innerHTML = banner + renderReach(state.panel, state.levers, state.view, chip);
    if (detail) detail.innerHTML = "";
  } else if (irn) {
    // One district, one container. There is no forecast on that route — the band is drawn once,
    // statewide, where it is the subject — so there is nothing for a detail half to sit below.
    if (out) out.innerHTML = banner + renderDistrictScenario(state.panel, state.levers, irn, chip);
    if (detail) detail.innerHTML = "";
  } else {
    const rendered = renderScenario(state.panel, state.levers, chip);
    if (out) out.innerHTML = banner + rendered.summary;
    // Written on every render, including when it is empty. Under current law there is nothing to
    // distribute or rank, and a detail half left standing from the last lever position would be
    // describing a scenario the controls no longer hold — below the fan chart, where the reader
    // would have to scroll past a forecast to find out.
    if (detail) detail.innerHTML = rendered.detail;
  }
  const projection = $("#projection-out");
  // The band is gated on its own checks. A forecast that failed them costs the reader the band,
  // not the scenario builder: they are different claims and one can be wrong alone.
  if (projection && isForecastVerified(state.verification)) {
    projection.innerHTML = renderProjection(state.panel, state.levers, chip);
  }
  /*
   * And say so. Read back off the tiles that were just written rather than composed from the
   * levers, so the sentence a reader hears is the figure a reader sees — see `tileSummary`.
   *
   * The forecast tiles are included where they exist: on the statewide route the band *is* the
   * headline, and a summary naming only the simulation would leave out the half of the page the
   * horizon slider moves.
   */
  const said = [out, projection]
    .filter((node): node is HTMLElement => node != null)
    .map((node) => tileSummary(node))
    .filter((part) => part !== "")
    .join(". ");
  if (said !== "") say(`Scenario updated. ${said}.`);
}

function reportFailure(verification: Verification): void {
  const failures = verification.comparisons.filter((c) => !c.agrees);
  const detail =
    verification.comparisons.length === 0
      ? "<li>The panel carries no checkpoints, so nothing could be checked.</li>"
      : failures
          .map(
            (c) =>
              `<li><strong>${escapeHtml(c.label)}</strong>: ${c.differences
                .map(escapeHtml)
                .join("; ")}</li>`,
          )
          .join("");
  /*
   * The whole card, not the form inside it.
   *
   * Emptying `#scenario-controls` left `#levers` standing with nothing in it but its own heading —
   * a card reading `Levers #` above an explanation that the builder is disabled. There are no
   * levers; the heading was describing an empty box.
   */
  $("#levers")?.remove();
  // Every container the builder writes, not just the first. The disabled notice replaces the
  // simulation, and a detail half left below the fan chart would be figures from the last render
  // sitting under a card explaining that the derivation producing them cannot be trusted.
  const below = $("#scenario-detail");
  if (below) below.innerHTML = "";
  const out = $("#scenario-out");
  if (out) {
    out.innerHTML = `<div class="card err" id="disabled" data-part="disabled">
      <h2>${heading("disabled", "The scenario builder is disabled")}</h2>
      <p>This page re-derives Ohio's funding formula in the browser so a slider does not need a
        round trip, and it checks that derivation against results computed by
        <code>crates/project</code> before using it. Those checks did not pass:</p>
      <ul>${detail}</ul>
      <p class="note">The Rust is authoritative. Either the two implementations have drifted apart
        or this panel is from a different build. Every other page on this site reads figures
        computed at build time and is unaffected.</p>
    </div>`;
  }
}

function reportForecastFailure(panel: Panel, verification: Verification): void {
  const out = $("#projection-out");
  if (!out) return;
  if (!panel.projection) {
    out.innerHTML = `<div class="card" id="projection" data-part="projection">
      <h2>${heading("projection", "At projected enrollment")}</h2>
      <p class="note">This panel carries no projection block, so enrollment cannot be carried
        forward. The simulation above is unaffected — it runs at published enrollment.</p>
    </div>`;
    return;
  }
  const failures = verification.forecasts.filter((c) => !c.agrees);
  const detail =
    verification.forecasts.length === 0
      ? "<li>The panel declares a projection but carries no forecasts to check it against.</li>"
      : failures
          .map(
            (c) =>
              `<li><strong>${escapeHtml(c.label)}</strong>: ${c.differences
                .map(escapeHtml)
                .join("; ")}</li>`,
          )
          .join("");
  out.innerHTML = `<div class="card err" id="projection-disabled" data-part="projection-disabled">
    <h2>${heading("projection-disabled", "The projection is disabled")}</h2>
    <p>This page carries its own copy of the enrollment projection so a slider does not need a
      round trip, and checks it against forecasts computed by <code>crates/project</code> before
      drawing a band. Those checks did not pass:</p>
    <ul>${detail}</ul>
    <p class="note">The Rust is authoritative. The simulation above runs at published enrollment
      and does not depend on this.</p>
  </div>`;
}

function boot(panel: Panel): void {
  if (panel.contract_version !== REQUIRED_CONTRACT) {
    reportFailure({
      ok: false,
      comparisons: [
        {
          label: "bundle contract",
          agrees: false,
          differences: [
            `this page reads ${REQUIRED_CONTRACT}, the panel declares ${panel.contract_version}`,
          ],
        },
      ],
      forecasts: [],
    });
    return;
  }

  const verification = verify(panel);
  const baseYear = panel.projection?.base_year ?? 0;
  /*
   * The horizon's two ends, from the feed rather than from the control.
   *
   * `max` collapses onto `base` when there is no projection block, which is the honest bound: a
   * panel that cannot carry enrollment forward cannot be asked to. The district route has no
   * horizon control at all and still needs this, because its query string can carry `h=`.
   */
  const horizonBound: HorizonBound = {
    base: baseYear,
    max: panel.projection?.horizon ?? baseYear,
  };
  state = {
    panel,
    verification,
    levers: defaultLevers(modelOf(panel.statewide), baseYear),
    view: viewFromQuery(new URLSearchParams(location.search)),
  };

  const status = $("#scenario-status");
  if (!isVerified(verification)) {
    reportFailure(verification);
    if (status) {
      status.textContent = "Formula check FAILED — the scenario builder is disabled";
      status.classList.add("err");
    }
    return;
  }
  if (status) {
    const forecasts = isForecastVerified(verification)
      ? ` and ${verification.forecasts.length} reference forecasts`
      : "";
    status.textContent = `Formula reproduced against ${verification.comparisons.length} reference scenarios${forecasts} from crates/project.`;
  }
  if (!isForecastVerified(verification)) reportForecastFailure(panel, verification);

  /*
   * A shared link's levers, applied before the first read of the controls.
   *
   * A `?draft=` sets them first and an explicit lever in the same URL overrides it, so the two
   * compose: `?draft=x` opens the bill, and `?draft=x&base=1.06` opens the bill with one clause
   * moved — which `renderDraft` then reports as a departure rather than as the bill.
   */
  const opened = panel.drafts.find((d) => d.slug === draftSlug);
  const fromDraft = opened
    ? draftLevers(opened, modelOf(panel.statewide), baseYear)
    : null;
  const initial: Partial<Levers> = fromDraft
    ? { ...fromDraft, ...fromQuery(horizonBound) }
    : fromQuery(horizonBound);
  if (initial.guarantee) $<HTMLSelectElement>("#lv-guarantee")!.value = initial.guarantee;
  const put = (id: string, value: number | undefined) => {
    const control = $<HTMLInputElement>(id);
    if (control && value != null) control.value = String(value);
  };
  put("#lv-arg", initial.guaranteeArgument);
  put("#lv-base", initial.baseCostScale);
  put("#lv-min", initial.minimumStateShare);
  put("#lv-phase", initial.phaseInGeneral);
  put("#lv-phase-dpia", initial.phaseInDpia);
  put("#lv-horizon", initial.horizon);

  /*
   * And the view's controls, from the same query string, before the first render.
   *
   * Set on the DOM rather than kept only in `state` because `update()` reads the controls back on
   * every tick: a view applied to `state` alone would be overwritten by the selects' defaults the
   * first time a reader touched a slider, silently returning them to formula-against-realized.
   */
  const select = (id: string, value: string) => {
    const control = $<HTMLSelectElement>(id);
    if (control) control.value = value;
  };
  select("#rv-x", state.view.x);
  select("#rv-y", state.view.y);
  select("#rv-shading", state.view.shading);
  select("#rv-group", String(state.view.highlight));
  const trails = $<HTMLInputElement>("#rv-trails");
  if (trails) trails.checked = state.view.trails;
  /*
   * And the scope, into the same two places `readView` reads it back out of.
   *
   * The counties are checked here rather than held in `state` for exactly the reason above: the
   * first `input` event would otherwise read the unchecked boxes and quietly discard a selection a
   * reader had been sent. A slug the panel does not carry is dropped by `inScope` and so cannot be
   * checked here either — `?co=nowhere` leaves every box clear, which is the whole state, which is
   * what that link actually asks for.
   */
  for (const slug of state.view.counties) {
    const box = $<HTMLInputElement>(`#reach-scope input[name="co"][value="${slug}"]`);
    if (box) box.checked = true;
  }
  setNamed(state.view.districts);

  const fallback = defaultLevers(modelOf(panel.statewide), baseYear).horizon;

  /*
   * `fromControls: false` is for the first render of a draft, and it is not a convenience.
   *
   * The sliders are quantized — `#lv-base` steps by 0.01 from 0.8 — and a draft's lever value is
   * whatever the provision says. The refresh provision is `1.0395`, which the control rounds to
   * `1.04`: a $3.2M difference on a $220.5M figure. Reading the controls for the first render
   * would put a number that is not the draft's under a banner saying it is, which is the one
   * thing this path exists to prevent.
   *
   * So the draft is rendered from the draft. The first control the reader touches hands authority
   * to the DOM and it keeps it — and `renderDraft` reports that as a departure, correctly,
   * because a scenario the controls can express is a different scenario from the bill.
   */
  const update = (fromControls = true) => {
    if (!state) return;
    if (fromControls) state.levers = readLevers(fallback, state.levers.backstop);
    // Always from the controls. Unlike the levers there is no path that renders a view the reader
    // cannot see — a draft sets lever positions and says nothing about axes.
    state.view = readView();
    syncView(state.view);
    syncLabels(state.levers, baseYear);
    syncPresets(state.levers);
    toQuery();
    render();
    // After `render`, which replaces the containers both cross-links live in, and after `toQuery`,
    // which is what put the levers in the search string being handed over.
    carryLevers();
  };

  for (const control of document.querySelectorAll(
    "#scenario-controls input, #scenario-controls select, #reach-view input, #reach-view select, " +
      '#reach-scope input[name="co"]',
  )) {
    control.addEventListener("input", () => update());
  }

  /*
   * The scope's two controls that are actions rather than values.
   *
   * `#rv-add` is a picker, not a selection: it names one district, that district joins the set, and
   * the control returns to its placeholder so it can name another. Left showing the last choice it
   * would read as *the* selection while the chips said otherwise.
   *
   * Registered outside the loop above because neither of these is a field `readView` reads. The
   * chips are delegated rather than bound per chip, since `syncScope` rebuilds the row on every
   * tick and a listener attached to a chip would be thrown away with it.
   */
  const add = $<HTMLSelectElement>("#rv-add");
  add?.addEventListener("change", () => {
    const irn = add.value;
    if (irn === "") return;
    setNamed([...named(), irn]);
    add.value = "";
    update();
  });

  $("#rv-chips")?.addEventListener("click", (event) => {
    const chip = (event.target as HTMLElement).closest<HTMLButtonElement>("button.chip");
    if (!chip) return;
    if (chip.classList.contains("chip-clear")) {
      for (const box of document.querySelectorAll<HTMLInputElement>(
        '#reach-scope input[name="co"]',
      )) {
        box.checked = false;
      }
      setNamed([]);
    } else if (chip.dataset.county != null) {
      const box = $<HTMLInputElement>(
        `#reach-scope input[name="co"][value="${chip.dataset.county}"]`,
      );
      if (box) box.checked = false;
    } else if (chip.dataset.district != null) {
      setNamed(named().filter((irn) => irn !== chip.dataset.district));
    }
    update();
  });

  /* Enter in the scope form must not navigate. Same rule as the district index's filters, and the
     same reason it cannot be an inline handler: `script-src 'self'` blocks one. */
  $<HTMLFormElement>("#reach-scope")?.addEventListener("submit", (event) => event.preventDefault());

  /*
   * The presets, which are lever positions with names.
   *
   * Each button carries its whole lever state as JSON, written by the page from `presets()` — so
   * the scale behind "Fund the plan" is the one the feed's own draft prices rather than a number
   * typed into a script.
   *
   * A preset is a position and not a patch, which is why it sets every field. The partial version
   * composed, and the row then reported three buttons pressed at once under a heading reading
   * "Start from" — see `Preset.levers`. A reader who wants two of these at once has the sliders.
   */
  for (const button of document.querySelectorAll<HTMLButtonElement>("[data-preset]")) {
    button.addEventListener("click", () => {
      const settings = JSON.parse(button.dataset.preset ?? "{}") as Levers;
      $<HTMLSelectElement>("#lv-guarantee")!.value = settings.guarantee;
      put("#lv-arg", settings.guaranteeArgument);
      put("#lv-base", settings.baseCostScale);
      put("#lv-min", settings.minimumStateShare);
      put("#lv-phase", settings.phaseInGeneral);
      put("#lv-phase-dpia", settings.phaseInDpia);
      update();
    });
  }
  $("#scenario-reset")?.addEventListener("click", () => {
    const defaults = defaultLevers(modelOf(panel.statewide), baseYear);
    $<HTMLSelectElement>("#lv-guarantee")!.value = defaults.guarantee;
    put("#lv-arg", defaults.guaranteeArgument);
    put("#lv-base", defaults.baseCostScale);
    put("#lv-min", defaults.minimumStateShare);
    put("#lv-phase", defaults.phaseInGeneral);
    put("#lv-phase-dpia", defaults.phaseInDpia);
    put("#lv-horizon", defaults.horizon);
    update();
  });

  if (fromDraft) state.levers = { ...fromDraft, ...fromQuery(horizonBound) };
  update(fromDraft == null);
}

// The slim panel, not the full feed: this page needs every district's formula inputs and none of
// their audited finances or report cards. See `src/pages/data/panel.json.ts`.
const PANEL = `${import.meta.env.BASE_URL}data/panel.json`;

fetch(PANEL)
  .then((response) => {
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json() as Promise<Panel>;
  })
  .then(boot)
  .catch((error: unknown) => {
    const message = error instanceof Error ? error.message : String(error);
    const out = $("#scenario-out");
    if (out) {
      out.innerHTML = `<div class="card err" id="panel-unreachable" data-part="panel-unreachable">
        <p>Could not load <code>${escapeHtml(PANEL)}</code> (${escapeHtml(message)}).</p>
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
 */
document
  .querySelector<HTMLFormElement>("#scenario-controls")
  ?.addEventListener("submit", (event) => event.preventDefault());
