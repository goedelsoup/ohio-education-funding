/** Wiring: load the feed, check the contract, check our own arithmetic, render the tabs. */
import { attachHover } from "./chart.js";
import { renderDistrict } from "./district.js";
import { escapeHtml, pct } from "./format.js";
import { defaultLevers, renderControls, renderScenario, } from "./scenario.js";
import { renderStatewide } from "./statewide.js";
import { REQUIRED_CONTRACT } from "./types.js";
import { isVerified, verify } from "./verify.js";
const $ = (selector) => document.querySelector(selector);
let state = null;
/**
 * Read the tab and district out of the URL fragment.
 *
 * `#statewide`, `#scenario`, `#district/043786`. A view worth showing someone is worth being
 * able to link to, and a school board arguing about one district should not have to say "now
 * pick us from the dropdown".
 */
function fromHash() {
    const raw = location.hash.replace(/^#/, "");
    const [route, query] = raw.split("?");
    const [tab, irn] = (route ?? "").split("/");
    if (tab === "statewide")
        return { tab: "statewide" };
    if (tab === "scenario") {
        const params = new URLSearchParams(query ?? "");
        const number = (key) => params.has(key) ? Number(params.get(key)) : undefined;
        const levers = {};
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
        ]) {
            const value = number(key);
            if (value != null && Number.isFinite(value))
                levers[field] = value;
        }
        return { tab: "scenario", levers };
    }
    return irn ? { tab: "district", irn } : { tab: "district" };
}
/**
 * Put the current view in the URL fragment.
 *
 * A scenario carries its levers, so the whole point of the tab — "here is what this proposal
 * does" — is something one person can send another.
 */
function toHash() {
    if (!state)
        return;
    let next;
    if (state.tab === "district") {
        next = `#district/${state.selected}`;
    }
    else if (state.tab === "statewide") {
        next = "#statewide";
    }
    else {
        const l = state.levers;
        const params = new URLSearchParams({
            g: l.guarantee,
            arg: String(l.guaranteeArgument),
            base: String(l.baseCostScale),
            min: String(l.minimumStateShare),
            pb: String(l.phaseInBaseCost),
            pc: String(l.phaseInCategorical),
        });
        next = `#scenario?${params.toString()}`;
    }
    if (location.hash !== next)
        history.replaceState(null, "", next);
}
function showTab(tab) {
    if (!state)
        return;
    state.tab = tab;
    for (const button of document.querySelectorAll("[data-tab]")) {
        const active = button.dataset.tab === tab;
        button.classList.toggle("active", active);
        button.setAttribute("aria-selected", String(active));
    }
    for (const panel of document.querySelectorAll("[data-panel]")) {
        panel.hidden = panel.dataset.panel !== tab;
    }
    toHash();
    render();
}
function render() {
    if (!state)
        return;
    const { bundle, tab } = state;
    if (tab === "district") {
        const district = state.byIrn.get(state.selected);
        if (district)
            $("#district-out").innerHTML = renderDistrict(bundle, district);
    }
    else if (tab === "statewide") {
        $("#statewide-out").innerHTML = renderStatewide(bundle);
    }
    else {
        $("#scenario-out").innerHTML = renderScenario(bundle, state.levers);
    }
}
function readLevers() {
    const number = (id) => Number($(id).value);
    return {
        guarantee: $("#lv-guarantee").value,
        guaranteeArgument: number("#lv-arg"),
        baseCostScale: number("#lv-base"),
        minimumStateShare: number("#lv-min"),
        phaseInBaseCost: number("#lv-phase"),
        phaseInCategorical: number("#lv-phase-cat"),
    };
}
function syncLeverLabels(levers) {
    const set = (id, text) => {
        $(id).textContent = text;
    };
    set("#lv-arg-out", pct(levers.guaranteeArgument, 0));
    set("#lv-base-out", `${levers.baseCostScale >= 1 ? "+" : "−"}${pct(Math.abs(levers.baseCostScale - 1), 0)}`);
    set("#lv-min-out", pct(levers.minimumStateShare, 0));
    set("#lv-phase-out", pct(levers.phaseInBaseCost, 0));
    set("#lv-phase-cat-out", pct(levers.phaseInCategorical, 0));
    // The retained-share slider only means anything for the two rules that take an argument.
    const argument = $("#lv-arg").closest(".lever");
    argument.hidden = levers.guarantee === "as-enacted" || levers.guarantee === "removed";
}
function wireScenario(bundle, initial) {
    $("#scenario-controls").innerHTML = renderControls(bundle.statewide.minimum_state_share);
    // A shared link's levers, applied before the first read of the controls.
    if (initial.guarantee)
        $("#lv-guarantee").value = initial.guarantee;
    const put = (id, value) => {
        if (value != null)
            $(id).value = String(value);
    };
    put("#lv-arg", initial.guaranteeArgument);
    put("#lv-base", initial.baseCostScale);
    put("#lv-min", initial.minimumStateShare);
    put("#lv-phase", initial.phaseInBaseCost);
    put("#lv-phase-cat", initial.phaseInCategorical);
    const update = () => {
        if (!state)
            return;
        state.levers = readLevers();
        syncLeverLabels(state.levers);
        toHash();
        render();
    };
    for (const control of document.querySelectorAll("#scenario-controls input, #scenario-controls select")) {
        control.addEventListener("input", update);
    }
    $("#scenario-reset").addEventListener("click", () => {
        if (!state)
            return;
        const defaults = defaultLevers(bundle.statewide.minimum_state_share);
        $("#lv-guarantee").value = defaults.guarantee;
        $("#lv-arg").value = String(defaults.guaranteeArgument);
        $("#lv-base").value = String(defaults.baseCostScale);
        $("#lv-min").value = String(defaults.minimumStateShare);
        $("#lv-phase").value = String(defaults.phaseInBaseCost);
        $("#lv-phase-cat").value = String(defaults.phaseInCategorical);
        update();
    });
    update();
}
/**
 * Report a checkpoint failure rather than rendering a scenario.
 *
 * The page's formula and the Rust that is authoritative for it have diverged. Showing a number
 * anyway would be worse than showing nothing, because it would look exactly like a right one.
 */
function reportVerificationFailure(verification) {
    const failures = verification.comparisons.filter((c) => !c.agrees);
    const detail = verification.comparisons.length === 0
        ? "<li>The feed carries no checkpoints, so nothing could be checked.</li>"
        : failures
            .map((c) => `<li><strong>${escapeHtml(c.label)}</strong>: ${c.differences
            .map(escapeHtml)
            .join("; ")}</li>`)
            .join("");
    $("#scenario-controls").innerHTML = "";
    $("#scenario-out").innerHTML = `<div class="card err">
    <h2>The scenario builder is disabled</h2>
    <p>This page re-derives Ohio's funding formula in the browser so a slider does not need a
      round trip, and it checks that derivation against results computed by
      <code>crates/project</code> before using it. Those checks did not pass:</p>
    <ul>${detail}</ul>
    <p class="note">The Rust is authoritative. Either the two implementations have drifted apart
      or the feed is from a different build. The district and statewide views read published
      figures and are unaffected.</p>
  </div>`;
}
function boot(bundle) {
    if (bundle.contract_version !== REQUIRED_CONTRACT) {
        $("#district-out").innerHTML = `<div class="card err">This page reads bundle contract
      ${REQUIRED_CONTRACT}; the feed declares ${escapeHtml(bundle.contract_version)}. Refusing
      to render rather than guess at field meanings.</div>`;
        return;
    }
    const verification = verify(bundle);
    state = {
        bundle,
        verification,
        byIrn: new Map(bundle.districts.map((d) => [d.irn, d])),
        selected: bundle.districts.some((d) => d.irn === "049056")
            ? "049056"
            : (bundle.districts[0]?.irn ?? ""),
        tab: fromHash().tab,
        levers: defaultLevers(bundle.statewide.minimum_state_share),
    };
    const picker = $("#pick");
    picker.innerHTML = "";
    for (const district of [...bundle.districts].sort((a, b) => a.name.localeCompare(b.name))) {
        const option = document.createElement("option");
        option.value = district.irn;
        option.textContent = district.name;
        picker.appendChild(option);
    }
    picker.value = state.selected;
    picker.addEventListener("change", () => {
        if (!state)
            return;
        state.selected = picker.value;
        toHash();
        render();
    });
    for (const button of document.querySelectorAll("[data-tab]")) {
        button.addEventListener("click", () => showTab(button.dataset.tab));
    }
    const requested = fromHash();
    if (isVerified(verification)) {
        wireScenario(bundle, requested.levers ?? {});
        $("#verified").textContent = `Formula verified against ${verification.comparisons.length} reference scenarios`;
    }
    else {
        reportVerificationFailure(verification);
        $("#verified").textContent = "Formula check FAILED — scenario builder disabled";
        $("#verified").classList.add("err");
    }
    $("#prov").textContent = bundle.provenance;
    attachHover(document.body, $("#tip"));
    if (requested.irn && state.byIrn.has(requested.irn)) {
        state.selected = requested.irn;
        picker.value = requested.irn;
    }
    showTab(requested.tab);
}
fetch("data/bundle.json")
    .then((response) => {
    if (!response.ok)
        throw new Error(`HTTP ${response.status}`);
    return response.json();
})
    .then(boot)
    .catch((error) => {
    const message = error instanceof Error ? error.message : String(error);
    $("#district-out").innerHTML = `<div class="card">
      <p class="err">Could not load <code>data/bundle.json</code> (${escapeHtml(message)}).</p>
      <p class="note">Browsers block <code>fetch</code> from <code>file://</code>. Serve the
        directory over HTTP:</p>
      <p class="note"><code>node web/serve.mjs</code>, then open
        <code>http://localhost:8000</code>.</p></div>`;
});
