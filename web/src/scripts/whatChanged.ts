/**
 * The county filter on `/what-changed`, over a table that is already complete.
 *
 * All 609 rows are in the document, ordered by county; this hides the ones outside the county
 * chosen. Without script the select does nothing and the table is every district, which the
 * browser's own find still searches.
 */
import { count } from "../lib/format.ts";
import { SECTIONS } from "../lib/routes.ts";

const id = SECTIONS.changed.byCounty;
const select = document.querySelector<HTMLSelectElement>(`#${id}-county`);
const out = document.querySelector<HTMLElement>(`#${id}-count`);
const rows = Array.from(document.querySelectorAll<HTMLTableRowElement>(`#${id} tbody tr[data-county]`));

if (select && out) {
  const apply = () => {
    const county = select.value;
    let shown = 0;
    for (const row of rows) {
      row.hidden = county !== "" && row.dataset.county !== county;
      if (!row.hidden) shown++;
    }
    out.textContent =
      shown === rows.length ? `${count(rows.length)} districts` : `${count(shown)} of ${count(rows.length)} districts`;
  };
  select.addEventListener("change", apply);
  // A select the browser restored on a back navigation is the reader's last choice.
  if (select.value !== "") apply();
}

/* Enter in the filter form would reload the page; `script-src 'self'` blocks an inline handler. */
document.querySelector<HTMLFormElement>(`#${id}-filter`)?.addEventListener("submit", (event) => event.preventDefault());
