/**
 * Zero and missing, read from the built pages (#597).
 *
 * "—" meant $0 under one column of /districts and "the source has no figure" two cells along, on
 * the same row. A zero is now written `$0` and a missing value "not reported", and the unit half in
 * `tests/unit/format.spec.ts` holds the formatters and the source to that. This holds the cells a
 * reader actually sees, on the row the issue was filed against.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { parseHTML } from "linkedom";
import { describe, expect, test } from "vitest";

import { NOT_REPORTED } from "../../src/lib/format.ts";
import { DIST } from "./artefact.ts";

const page = parseHTML(readFileSync(join(DIST, "districts.html"), "utf8")).document;
const text = (el: Element | null | undefined): string => (el?.textContent ?? "").replace(/\s+/g, " ").trim();

/** A row's cells by the column label they carry. */
function row(irn: string): Map<string, string> {
  const tr = [...page.querySelectorAll("tbody tr")].find((r) => r.querySelector(`th .n`)?.textContent?.trim() === irn);
  if (!tr) throw new Error(`/districts has no row for ${irn}`);
  return new Map([...tr.querySelectorAll("td")].map((td) => [td.getAttribute("data-label") ?? "", text(td)]));
}

describe("/districts", () => {
  // College Corner Local: not on the guarantee, and no valuation or poverty figure in the source.
  const college = row("064964");
  const label = (prefix: string): string => {
    const key = [...college.keys()].find((k) => k.startsWith(prefix));
    if (!key) throw new Error(`no column labelled ${prefix}`);
    return key;
  };

  test("a district off the guarantee reads $0 under it", () => {
    expect(college.get(label("From guarantee"))).toBe("$0");
  });

  test("a missing valuation reads as not reported", () => {
    expect(college.get(label("Valuation"))).toBe(NOT_REPORTED);
    expect(college.get(label("Econ. disadv."))).toBe(NOT_REPORTED);
  });

  test("no cell in the table is a bare dash", () => {
    const dashes = [...page.querySelectorAll("tbody td")].filter((td) => text(td) === "—");
    expect(dashes.length).toBe(0);
  });

  test("the table carries a key for the two", () => {
    const notes = [...page.querySelectorAll("p.note")].map(text);
    expect(notes.some((n) => n.includes("A zero means nothing was paid") && n.includes(NOT_REPORTED))).toBe(true);
  });
});
