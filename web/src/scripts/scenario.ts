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
  defaultLevers,
  renderDistrictScenario,
  renderProjection,
  renderScenario,
  type Levers,
} from "../lib/scenario.ts";
import { pct } from "../lib/format.ts";
import type { Panel } from "../lib/types.ts";
import { REQUIRED_CONTRACT } from "../lib/types.ts";
import { isForecastVerified, isVerified, verify, type Verification } from "../lib/verify.ts";

const $ = <T extends HTMLElement>(selector: string): T | null =>
  document.querySelector<T>(selector);

const root = $("#scenario-root");
/** Present on the district route, absent on the statewide one. That is the only difference. */
const irn = root?.dataset.irn;

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
function fromQuery(): Partial<Levers> {
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
    ["pb", "phaseInBaseCost"],
    ["pc", "phaseInCategorical"],
    ["h", "horizon"],
  ] as const) {
    const raw = params.get(key);
    if (raw == null) continue;
    const value = Number(raw);
    if (Number.isFinite(value)) levers[field] = value;
  }
  return levers;
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
    pb: String(l.phaseInBaseCost),
    pc: String(l.phaseInCategorical),
    h: String(l.horizon),
  });
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
    phaseInBaseCost: number("#lv-phase"),
    phaseInCategorical: number("#lv-phase-cat"),
    horizon: $("#lv-horizon") ? number("#lv-horizon") : fallbackHorizon,
  };
}

function syncLabels(levers: Levers, baseYear: number): void {
  const set = (id: string, text: string) => {
    const output = $<HTMLOutputElement>(id);
    if (output) output.textContent = text;
  };
  set("#lv-arg-out", pct(levers.guaranteeArgument, 0));
  set(
    "#lv-base-out",
    `${levers.baseCostScale >= 1 ? "+" : "−"}${pct(Math.abs(levers.baseCostScale - 1), 0)}`,
  );
  set("#lv-min-out", pct(levers.minimumStateShare, 0));
  set("#lv-phase-out", pct(levers.phaseInBaseCost, 0));
  set("#lv-phase-cat-out", pct(levers.phaseInCategorical, 0));
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
  if (out) {
    out.innerHTML = irn
      ? renderDistrictScenario(state.panel, state.levers, irn)
      : renderScenario(state.panel, state.levers);
  }
  const projection = $("#projection-out");
  // The band is gated on its own checks. A forecast that failed them costs the reader the band,
  // not the scenario builder: they are different claims and one can be wrong alone.
  if (projection && isForecastVerified(state.verification)) {
    projection.innerHTML = renderProjection(state.panel, state.levers);
  }
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
  const out = $("#scenario-out");
  if (out) {
    out.innerHTML = `<div class="card err">
      <h2>The scenario builder is disabled</h2>
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
    out.innerHTML = `<div class="card">
      <h2>At projected enrollment</h2>
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
  out.innerHTML = `<div class="card err">
    <h2>The projection is disabled</h2>
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

  // A shared link's levers, applied before the first read of the controls.
  const initial = fromQuery();
  if (initial.guarantee) $<HTMLSelectElement>("#lv-guarantee")!.value = initial.guarantee;
  const put = (id: string, value: number | undefined) => {
    const control = $<HTMLInputElement>(id);
    if (control && value != null) control.value = String(value);
  };
  put("#lv-arg", initial.guaranteeArgument);
  put("#lv-base", initial.baseCostScale);
  put("#lv-min", initial.minimumStateShare);
  put("#lv-phase", initial.phaseInBaseCost);
  put("#lv-phase-cat", initial.phaseInCategorical);
  put("#lv-horizon", initial.horizon);

  const fallback = defaultLevers(panel.statewide.minimum_state_share, baseYear).horizon;
  const update = () => {
    if (!state) return;
    state.levers = readLevers(fallback);
    syncLabels(state.levers, baseYear);
    toQuery();
    render();
  };

  for (const control of document.querySelectorAll("#scenario-controls input, #scenario-controls select")) {
    control.addEventListener("input", update);
  }
  $("#scenario-reset")?.addEventListener("click", () => {
    const defaults = defaultLevers(panel.statewide.minimum_state_share, baseYear);
    $<HTMLSelectElement>("#lv-guarantee")!.value = defaults.guarantee;
    put("#lv-arg", defaults.guaranteeArgument);
    put("#lv-base", defaults.baseCostScale);
    put("#lv-min", defaults.minimumStateShare);
    put("#lv-phase", defaults.phaseInBaseCost);
    put("#lv-phase-cat", defaults.phaseInCategorical);
    put("#lv-horizon", defaults.horizon);
    update();
  });

  update();
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
      out.innerHTML = `<div class="card err">
        <p>Could not load <code>${escapeHtml(PANEL)}</code> (${escapeHtml(message)}).</p>
        <p class="note">Every other page on this site carries its figures in the document and is
          unaffected. This one re-runs the formula, so it needs the panel.</p>
      </div>`;
    }
  });
