/**
 * Filtering and sorting the district index, over markup that is already complete.
 *
 * Nothing here fetches anything and nothing here renders a row. All 609 rows are in the document
 * when this loads; this hides some of them and reorders the rest. That ordering matters for what
 * a reader without JavaScript gets — the full table, alphabetical, and the browser's own find —
 * rather than an empty page and a spinner.
 */

const table = document.querySelector<HTMLTableElement>("#district-table");
const body = table?.tBodies[0];
const nameInput = document.querySelector<HTMLInputElement>("#f-name");
const statusSelect = document.querySelector<HTMLSelectElement>("#f-status");
const countOut = document.querySelector<HTMLElement>("#f-count");

if (table && body && nameInput && statusSelect && countOut) {
  const rows = Array.from(body.rows);

  /** Which `data-` flag each status option tests. Empty string means "no filter". */
  const FLAG: Record<string, string> = {
    guarantee: "guarantee",
    floor: "floor",
    minshare: "minshare",
    shrinking: "shrinking",
  };

  function applyFilters(): void {
    const needle = nameInput!.value.trim().toLowerCase();
    const status = statusSelect!.value;
    let shown = 0;
    for (const row of rows) {
      const matchesName = needle === "" || (row.dataset.name ?? "").includes(needle);
      // "On formula" is the complement of the guarantee flag rather than a flag of its own:
      // every district is one or the other and storing both would let them disagree.
      const matchesStatus =
        status === ""
          ? true
          : status === "formula"
            ? row.dataset.guarantee !== "1"
            : row.dataset[FLAG[status] ?? ""] === "1";
      const show = matchesName && matchesStatus;
      row.hidden = !show;
      if (show) shown++;
    }
    countOut!.textContent =
      shown === rows.length
        ? `${rows.length.toLocaleString("en-US")} districts`
        : `${shown.toLocaleString("en-US")} of ${rows.length.toLocaleString("en-US")} districts`;
  }

  nameInput.addEventListener("input", applyFilters);
  statusSelect.addEventListener("change", applyFilters);

  /**
   * Sort by a column, addressed by its index.
   *
   * The sort value is carried on the cell rather than parsed out of its text, because the text is
   * formatted for reading — `$14,245`, `−6.1%`, an em dash for a missing figure — and parsing it
   * back would be a second, worse implementation of the formatter. Missing values sort last in
   * both directions: a district with no reported valuation is not the poorest one.
   *
   * By index and not by a key repeated on all 4,263 cells. The column a cell belongs to is
   * already in the DOM's structure, and naming it again on every cell cost 90 KB of markup on a
   * page whose whole argument is that the document arrives complete.
   */
  function sortBy(column: number, descending: boolean): void {
    const value = (row: HTMLTableRowElement): string =>
      (row.cells[column] as HTMLElement | undefined)?.dataset.sortValue ?? "";

    const decorated = rows.map((row) => {
      const raw = value(row);
      const numeric = raw === "" ? null : Number(raw);
      return {
        row,
        missing: raw === "",
        numeric: numeric != null && Number.isFinite(numeric) ? numeric : null,
        text: raw,
      };
    });

    decorated.sort((a, b) => {
      if (a.missing !== b.missing) return a.missing ? 1 : -1;
      const order =
        a.numeric != null && b.numeric != null
          ? a.numeric - b.numeric
          : a.text.localeCompare(b.text);
      return descending ? -order : order;
    });

    for (const entry of decorated) body!.appendChild(entry.row);
  }

  for (const button of table.querySelectorAll<HTMLButtonElement>("thead button[data-sort]")) {
    button.addEventListener("click", () => {
      const column = (button.closest("th") as HTMLTableCellElement).cellIndex;
      const first = button.getAttribute("aria-sort") == null;
      const wasAscending = button.getAttribute("aria-sort") === "ascending";
      for (const other of table.querySelectorAll("thead button[data-sort]")) {
        other.removeAttribute("aria-sort");
      }
      // Names read best ascending on first click; every other column is a quantity, and the
      // question a reader has about a quantity is almost always "who is at the top".
      const descending = first ? button.dataset.sort !== "name" : wasAscending;
      button.setAttribute("aria-sort", descending ? "descending" : "ascending");
      sortBy(column, descending);
    });
  }
}

/*
 * Keep Enter from reloading the page.
 *
 * This was `onsubmit="return false"` on the form itself, which `script-src 'self'` blocks — an
 * inline event handler is inline script. The violation only appears where the CSP is actually
 * applied, which is the deployed site and never `vite preview`, so it shipped. See the built-output
 * check in `tests/e2e/`.
 */
document
  .querySelector<HTMLFormElement>("#district-filters")
  ?.addEventListener("submit", (event) => event.preventDefault());
