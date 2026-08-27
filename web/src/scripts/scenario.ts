/**
 * The scenario builder, in the browser.
 *
 * # Why these two routes still compute client-side when nothing else does
 *
 * Every other page on this site is baked: its figures are written into the HTML at build time and
 * no formula runs in the browser. These two cannot be. A lever has a continuum of positions, and
 * a static page per position is not a page — so the formula runs here, over the whole
 * 609-district panel, on every slider tick.
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
/** Present on the district route, absent on the statewide one. That is the only difference. */
const irn = root?.dataset.irn;

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
 * in the browser could build one — which is why every card on these two routes carried figures
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
    h: String(l.horizon),
  });
  // `draft` survives every lever move. It is not lever state — see `draftSlug` — and dropping it
  // here would make the URL in the bar stop being the one that opened the bill, while the page
  // was still explaining that these figures came from one.
  if (draftSlug) params.set("draft", draftSlug);
  const next = `${location.pathname}?${params.toString()}`;
  if (location.pathname + location.search !== next) history.replaceState(null, "", next);
}

function readLevers(fallbackHorizon: number): Levers {
  const number = (id: string) => Number($<HTMLInputElement>(id)!.value);
  return {
    guarantee: $<HTMLSelectElement>("#lv-guarantee")!.value as Levers["guarantee"],
    guaranteeArgument: number("#lv-arg"),
    baseCostScale: number("#lv-base"),
    minimumStateShare: number("#lv-min"),
    phaseInGeneral: number("#lv-phase"),
    phaseInDpia: number("#lv-phase-dpia"),
    horizon: $("#lv-horizon") ? number("#lv-horizon") : fallbackHorizon,
  };
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
  if (irn) {
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
  const controls = $("#scenario-controls");
  if (controls) controls.innerHTML = "";
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
  state = { panel, verification, levers: defaultLevers(panel.statewide.minimum_state_share, baseYear) };

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
    ? draftLevers(opened, panel.statewide.minimum_state_share, baseYear)
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

  const fallback = defaultLevers(panel.statewide.minimum_state_share, baseYear).horizon;

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
    if (fromControls) state.levers = readLevers(fallback);
    syncLabels(state.levers, baseYear);
    toQuery();
    render();
  };

  for (const control of document.querySelectorAll("#scenario-controls input, #scenario-controls select")) {
    control.addEventListener("input", () => update());
  }
  $("#scenario-reset")?.addEventListener("click", () => {
    const defaults = defaultLevers(panel.statewide.minimum_state_share, baseYear);
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
