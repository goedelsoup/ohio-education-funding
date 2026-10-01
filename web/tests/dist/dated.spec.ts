/**
 * Every page says when its data dates from and how to report an error, and `/data` says how to
 * cite it (#595).
 *
 * # What this caught
 *
 * A journalist citing the site needs the date of the data, a maintainer and a way to report an
 * error. The calculator's date, 16 December 2025, was on its catalog page and nowhere else; the
 * footer said "FY2027 model" and offered only the repository. `/data`'s Terms asked anyone
 * publishing a figure to name its year and basis and gave no line to copy that did.
 *
 * # What the date is checked against
 *
 * The catalog entry, read here independently of `modelSource`: the footer's link names the entry,
 * and the entry's own "dated …" sentence is what the footer's `<time>` must say. A date typed into
 * the layout would pass a test that only asked for *a* date.
 */

import { readFileSync } from "node:fs";
import { join, relative } from "node:path";

import { parseHTML } from "linkedom";
import { describe, expect, test } from "vitest";

import { DIST, pages } from "./artefact.ts";

const CATALOG = join(import.meta.dirname, "../../../.yidam/catalog");
const ISSUES = "https://github.com/goedelsoup/ohio-education-funding/issues/new";
const FULL_DATE = /^\d{1,2} (January|February|March|April|May|June|July|August|September|October|November|December) \d{4}$/;

function footerOf(file: string) {
  const html = readFileSync(file, "utf8");
  const start = html.indexOf('<footer class="site"');
  expect(start, relative(DIST, file)).toBeGreaterThan(-1);
  const { document } = parseHTML(`<!doctype html><body>${html.slice(start, html.indexOf("</footer>", start) + 9)}`);
  return document.querySelector("footer")!;
}

describe("the footer dates the data and takes error reports", () => {
  const home = footerOf(join(DIST, "index.html"));
  const time = home.querySelector("time");
  const source = time?.parentElement?.querySelector('a[href^="/wiki/source/"]');

  test("the date is the calculator's own, read from its catalog entry", () => {
    expect(time, "the footer carries no <time>").toBeTruthy();
    expect(time!.textContent).toMatch(FULL_DATE);
    const slug = source!.getAttribute("href")!.replace("/wiki/source/", "");
    const entry = readFileSync(join(CATALOG, `${slug}.md`), "utf8");
    expect(entry).toContain(`dated ${time!.textContent}`);
    const machine = new Date(`${time!.getAttribute("datetime")}T00:00:00Z`);
    const spelled = machine.toLocaleDateString("en-GB", { day: "numeric", month: "long", year: "numeric", timeZone: "UTC" });
    expect(spelled, "the datetime attribute and the text disagree").toBe(time!.textContent);
  });

  test("every page carries the same date and a report link naming that page", () => {
    const files = pages();
    expect(files.length).toBeGreaterThan(2000);
    for (const file of files) {
      const footer = footerOf(file);
      const name = relative(DIST, file);
      expect(footer.querySelector("time")?.textContent, name).toBe(time!.textContent);
      const report = [...footer.querySelectorAll("a")].find((a) => a.textContent?.trim() === "Report an error");
      expect(report, name).toBeTruthy();
      const href = new URL(report!.getAttribute("href")!);
      expect(`${href.origin}${href.pathname}`, name).toBe(ISSUES);
      expect(href.searchParams.get("title"), name).toMatch(/^Error on \//);
    }
  });
});

describe("/data says how to cite it", () => {
  const { document } = parseHTML(readFileSync(join(DIST, "data.html"), "utf8"));
  const cite = document.querySelector("#cite");

  test("a #cite section carries a line with the year, the basis, the address and the date", () => {
    expect(cite, "/data has no #cite section").toBeTruthy();
    const line = cite!.querySelector(".citation")?.textContent ?? "";
    const footerDate = footerOf(join(DIST, "data.html")).querySelector("time")!.textContent!;
    expect(line).toMatch(/\bFY\d{4} model\b/);
    expect(line).toMatch(/\bnominal dollars\b/);
    expect(line).toMatch(/https:\/\/\S+\/data\b/);
    expect(line).toContain(`dated ${footerDate}`);
  });
});
