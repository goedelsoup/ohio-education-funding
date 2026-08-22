/**
 * Search, over a prebuilt index and a static host.
 *
 * There is no server to query, so the index is a single JSON document built at compile time and
 * filtered here. That bounds what it can hold — titles, kinds and a few searchable aliases, never
 * page prose — which is the trade that keeps it at a few tens of kilobytes.
 *
 * Ranking is deliberately simple and explainable: an exact match, then a prefix match, then a
 * match anywhere. Nothing fuzzy. A reader who typed six digits wants that IRN, and a scoring
 * function clever enough to disagree with them is a scoring function that will.
 */

import type { SearchEntry } from "../pages/search-index.json.ts";

const input = document.querySelector<HTMLInputElement>("#s-q");
const out = document.querySelector<HTMLElement>("#search-out");
const countOut = document.querySelector<HTMLElement>("#s-count");

/** Enough to choose from; past this the query is the thing to refine. */
const LIMIT = 40;

const KIND_LABEL: Record<SearchEntry["k"], string> = {
  district: "District",
  county: "County",
  seat: "Seat",
  node: "Corpus",
  source: "Source",
  page: "Page",
};

function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!);
}

function score(entry: SearchEntry, needle: string): number {
  const title = entry.t.toLowerCase();
  const alias = (entry.a ?? "").toLowerCase();
  if (title === needle || alias === needle) return 0;
  if (title.startsWith(needle)) return 1;
  if (alias.startsWith(needle)) return 2;
  if (title.includes(needle)) return 3;
  if (alias.includes(needle)) return 4;
  return Infinity;
}

async function start(): Promise<void> {
  if (!input || !out) return;
  const response = await fetch(`${import.meta.env.BASE_URL}search-index.json`);
  if (!response.ok) {
    out.innerHTML = `<div class="card err" id="index-unreachable" data-part="index-unreachable"><p>Could not load the search index (HTTP ${response.status}).</p></div>`;
    return;
  }
  const entries = (await response.json()) as SearchEntry[];

  const run = (raw: string) => {
    const needle = raw.trim().toLowerCase();
    if (needle === "") {
      out.innerHTML = "";
      if (countOut) countOut.textContent = "";
      return;
    }
    const hits = entries
      .map((entry) => ({ entry, rank: score(entry, needle) }))
      .filter((hit) => hit.rank !== Infinity)
      .sort((x, y) => x.rank - y.rank || x.entry.t.localeCompare(y.entry.t));

    if (countOut) {
      countOut.textContent =
        hits.length === 0
          ? "nothing found"
          : hits.length > LIMIT
            ? `${LIMIT} of ${hits.length} matches`
            : `${hits.length} match${hits.length === 1 ? "" : "es"}`;
    }
    if (hits.length === 0) {
      out.innerHTML = `<div class="card" id="no-matches" data-part="no-matches"><p class="note">Nothing matches
        <strong>${escapeHtml(raw.trim())}</strong>. District pages are keyed by six-digit IRN, and
        the corpus is browsable by class from <a href="/wiki">the wiki</a>.</p></div>`;
      return;
    }
    const rows = hits
      .slice(0, LIMIT)
      .map(
        ({ entry }) => `<tr>
          <th><a href="${entry.u}">${escapeHtml(entry.t)}</a></th>
          <td class="n">${KIND_LABEL[entry.k]}${entry.k === "district" ? ` ${escapeHtml(entry.a ?? "")}` : ""}</td>
        </tr>`,
      )
      .join("");
    out.innerHTML = `<div class="card" id="results" data-part="results"><div class="scroll"><table class="prose">
      <tbody>${rows}</tbody></table></div></div>`;
  };

  input.addEventListener("input", () => run(input.value));

  // A linked search — `/search?q=guarantee`, and where the header's box submits to with no script.
  const initial = new URLSearchParams(location.search).get("q");
  if (initial) {
    input.value = initial;
    run(initial);
  }
  input.focus();
}

void start();

/*
 * Keep Enter from reloading the page.
 *
 * This was `onsubmit="return false"` on the form itself, which `script-src 'self'` blocks — an
 * inline event handler is inline script. The violation only appears where the CSP is actually
 * applied, which is the deployed site and never `vite preview`, so it shipped. See the built-output
 * check in `tests/e2e/`.
 */
document
  .querySelector<HTMLFormElement>("#search-form")
  ?.addEventListener("submit", (event) => event.preventDefault());
