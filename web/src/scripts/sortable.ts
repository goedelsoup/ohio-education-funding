/**
 * Sorting any `table[data-sortable]`, over rows that are already in the document.
 *
 * The county page's who-gained tables, two to a card and one per measure. Each sorts on its own:
 * the measure toggle hides one, and a sort applied to a hidden table is one the reader did not ask
 * for. The terms are the districts index's (`districts.ts`): the sort key is on the cell as
 * `data-sort-value`, a blank one sorts last in both directions, and the state is `aria-sort` on
 * the `th`. That table keeps its own script because it also filters and drives a strip of charts.
 */
import { compare } from "../lib/order.ts";

for (const table of document.querySelectorAll<HTMLTableElement>("table[data-sortable]")) {
  const body = table.tBodies[0];
  if (!body) continue;
  const rows = Array.from(body.rows);

  const sortBy = (column: number, descending: boolean) => {
    const decorated = rows.map((row) => {
      const raw = (row.cells[column] as HTMLElement | undefined)?.dataset.sortValue ?? "";
      const numeric = raw === "" ? NaN : Number(raw);
      return { row, missing: raw === "", numeric: Number.isFinite(numeric) ? numeric : null, raw };
    });
    decorated.sort((a, b) => {
      if (a.missing !== b.missing) return a.missing ? 1 : -1;
      const order = a.numeric != null && b.numeric != null ? a.numeric - b.numeric : compare(a.raw, b.raw);
      return descending ? -order : order;
    });
    for (const entry of decorated) body.appendChild(entry.row);
  };

  for (const button of table.querySelectorAll<HTMLButtonElement>("thead button[data-sort]")) {
    button.addEventListener("click", () => {
      const cell = button.closest("th") as HTMLTableCellElement;
      const was = cell.getAttribute("aria-sort");
      for (const other of table.querySelectorAll("thead th[aria-sort]")) other.setAttribute("aria-sort", "none");
      // Names ascending first, quantities descending: "who is at the top" is the usual question.
      const descending = was == null || was === "none" ? button.dataset.sort !== "name" && button.dataset.sort !== "line" : was === "ascending";
      cell.setAttribute("aria-sort", descending ? "descending" : "ascending");
      sortBy(cell.cellIndex, descending);
    });
  }
}
