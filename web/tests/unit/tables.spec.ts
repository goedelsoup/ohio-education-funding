/**
 * Which way a table's columns and their heads read (#575).
 *
 * `alignColumns` in `semantics.ts` classes each column as figures or words, and #575 found it
 * wrong in two directions: `/statewide`'s money columns came out as words, because `millions()`
 * writes `$7.28B` and the pattern took no suffix, and because the cell's hidden year label was read
 * as part of it. And the heads over figure columns stayed left while their figures went right.
 */

import { parseHTML } from "linkedom";
import { describe, expect, test } from "vitest";

import { fig } from "../../src/lib/format.ts";
import { applySemantics, isFigure } from "../../src/lib/semantics.ts";

function table(html: string): Document {
  return parseHTML(`<!doctype html><html><body>${applySemantics(html).html}</body></html>`).document;
}

describe("a figure", () => {
  test("carries a thousands, millions or billions suffix", () => {
    expect(isFigure("$7.28B")).toBe(true);
    expect(isFigure("$412.6M")).toBe(true);
    expect(isFigure("−$3.1M")).toBe(true);
    expect(isFigure("$12K")).toBe(true);
    expect(isFigure("$1.2M – $4.8M")).toBe(true);
  });

  test("and a word is still a word", () => {
    expect(isFigure("Community schools")).toBe(false);
    expect(isFigure("Bus")).toBe(false);
    expect(isFigure("R.C. 3317.022")).toBe(false);
  });
});

describe("a column of fig() amounts", () => {
  const rows = ["Community schools", "EdChoice", "Autism"]
    .map((name, i) => `<tr><th>${name}</th><td>${fig(`$${i + 1}.28B`, "FY2027")}</td><td>Paid directly to the school that enrolled the pupil</td></tr>`)
    .join("");
  const document = table(
    `<table><thead><tr><th>Unit</th><th>Amount</th><th>How</th></tr></thead><tbody>${rows}</tbody></table>`,
  );

  test("is figures, although its text carries the year the stylesheet hides", () => {
    const amounts = [...document.querySelectorAll("tbody tr")].map((row) => row.children[1]!);
    expect(amounts[0]!.textContent).toContain("FY2027");
    expect(amounts.filter((cell) => cell.classList.contains("says"))).toEqual([]);
  });

  test("and its head reads right, over it, while a words column's head stays left", () => {
    const heads = [...document.querySelectorAll("thead th")];
    expect(heads.map((th) => th.classList.contains("heads-figures"))).toEqual([false, true, false]);
  });
});

test("a head spanning a figure column and a words column stays left", () => {
  const document = table(
    `<table><thead><tr><th>Unit</th><th colspan="2">Paid</th></tr></thead>
      <tbody><tr><th>A</th><td>$1.0M</td><td>to the district that enrolled the pupil</td></tr></tbody></table>`,
  );
  expect(document.querySelector("thead th[colspan]")!.classList.contains("heads-figures")).toBe(false);
});
