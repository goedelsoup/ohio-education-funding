/**
 * A long table cut to its first rows, with the rest one click away (#708).
 *
 * /history ran 5,400px between two charts at 1280 because a table of every row stood between
 * them. The rows a reader scans first stay open; the rest go in a closed `details` under the same
 * head, as a chart's values do. Both halves are in the document, so nothing is lost without
 * script and a find-in-page still reaches every row.
 */
export function foldedTable(head: string, rows: string[], shown: number, summary: string): string {
  const table = (body: string[]) =>
    `<div class="scroll"><table>${head}<tbody>${body.join("")}</tbody></table></div>`;
  if (rows.length <= shown) return table(rows);
  return `${table(rows.slice(0, shown))}
      <details class="table-rest"><summary>${summary}</summary>${table(rows.slice(shown))}</details>`;
}
