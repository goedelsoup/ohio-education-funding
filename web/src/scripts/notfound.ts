/**
 * Turn a 404 into a near miss.
 *
 * The matching itself is in `src/lib/nearest.ts` and is unit-tested there; this is the wiring —
 * read the failed path, fetch the index, put the suggestions on the page.
 */

import { nearest } from "../lib/nearest.ts";
import type { SearchEntry } from "../pages/search-index.json.ts";

const card = document.querySelector<HTMLElement>("#suggest");
const list = document.querySelector<HTMLElement>("#suggest-list");

async function suggest(): Promise<void> {
  if (!card || !list) return;
  const path = decodeURIComponent(location.pathname);

  const response = await fetch(`${import.meta.env.BASE_URL}search-index.json`);
  if (!response.ok) return;
  const entries = (await response.json()) as SearchEntry[];
  const matches = nearest(
    path,
    entries.filter((e) => e.k === "district"),
  );
  if (matches.length === 0) return;

  for (const match of matches) {
    const link = document.createElement("a");
    link.className = "flag";
    link.href = match.u;
    link.textContent = match.a ? `${match.t} · ${match.a}` : match.t;
    list.appendChild(link);
  }
  card.hidden = false;
}

void suggest();
