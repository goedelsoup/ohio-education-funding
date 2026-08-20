/**
 * Every act the budget's own Catalog names as the origin of a line, against the acts the corpus
 * can actually describe.
 *
 * # Why this is a test and not a list in a README
 *
 * Because the README said the gap was "before 2005" and it was not. `catalog-line-item-basis.tsv`
 * records, for each budget line, the act that established it — and crossing that against
 * `.yidam/corpus/legislation/` found that the four most-cited acts of the last two decades had no
 * node at all. Every appropriation figure the site shows for FY2014 through FY2021 traced back to
 * an act the corpus could not name, and nothing noticed, because the two facts lived in different
 * files and no check compared them.
 *
 * The site renders these citations: `lineOrigins.ts` prints "the act that created each line" on
 * the statewide budget card. A reader following one had nowhere to go.
 *
 * # Why a ratchet and not a zero
 *
 * 45 acts are named and 32 have no node. Most are one- or two-line curiosities from the 1970s and
 * 1980s — the act that created a line for a programme that has since been folded into another —
 * and a zero here would either be a lie or a reason never to land the check. So: a coverage floor
 * that can only rise, plus a named allowlist for the acts big enough that leaving them unwritten
 * is a decision rather than an oversight. A new act crossing that line fails until somebody
 * decides which it is, which is the moment the question is cheap to answer.
 */

import { readFileSync } from "node:fs";
import { readdirSync } from "node:fs";
import { join } from "node:path";

import { expect, test } from "vitest";

const ROOT = join(import.meta.dirname, "../../..");
const BASIS = join(ROOT, "crates/project/fixtures/catalog-line-item-basis.tsv");
const NODES = join(ROOT, ".yidam/corpus/legislation");

/** `Am. Sub. H.B. 64 of the 131st G.A.` and every abbreviation of it the Catalog uses. */
const CITATION = /(?:Am\.\s*)?(?:Sub\.\s*)?(H\.B\.|S\.B\.)\s*(\d+)\s*of the (\d+)(?:st|nd|rd|th) G\.A\./g;

/**
 * Acts the Catalog leans on that the corpus still cannot describe, and why each is still waiting.
 *
 * Keyed `<bill> of the <n>th`, valued by what it would take. An entry reading "not got to yet" is
 * not a reason; it is this list being used as a place to put things.
 */
const UNWRITTEN: Record<string, string> = {
  "H.B. 152 of the 120th":
    "FY1994-95. Below the floor `ohio-session-laws` describes: the legislature's own version index stops at the 122nd G.A., so neither the act nor an LSC analysis of it is served in any form. Needs a records request or a library.",
  "H.B. 282 of the 123rd":
    "FY2000-01. The enrolled act is retrieved and pinned (`hb282-123-enrolled`) for its appropriation table. Writing the node means reading the act rather than its table, which is the judgement step `ohio-bills` deliberately does not automate.",
  "H.B. 650 of the 122nd":
    "The mid-biennium act that itemised FY1999 after H.B. 215 deferred it. Reachable, and it prints every amended row twice — struck and inserted — which is the reader `ohio-session-laws` records as still missing.",
  "H.B. 191 of the 112th": "FY1978-79. Below the publisher's floor. Same obstacle as H.B. 152.",
  "H.B. 111 of the 118th":
    "FY1990-91, and it shares a bill number with H.B. 111 of the 117th, which the Catalog also names. Below the publisher's floor, and the pair is the reason the slug convention here carries a year rather than a General Assembly.",
  "H.B. 204 of the 113th": "FY1980-81. Below the publisher's floor.",
  "H.B. 238 of the 116th": "FY1986-87. Below the publisher's floor.",
};

/** Citations below this are one-line curiosities; above it, a missing node is a decision. */
const MATERIAL = 30;

/**
 * How much of the Catalog's origin story the corpus can tell. Raise it; never lower it.
 *
 * 33.7% before the Bridge decade was written, 65.3% after, 71.5% once H.B. 95 and H.B. 119 joined
 * them. The next six points are H.B. 152 of the 120th and H.B. 282 of the 123rd, and only one of
 * those is reachable.
 */
const COVERAGE_FLOOR = 0.71;

interface Named {
  label: string;
  number: string;
  lines: number;
}

function cited(): Named[] {
  const rows = readFileSync(BASIS, "utf8").split("\n").slice(1);
  const counts = new Map<string, Named>();
  for (const row of rows) {
    const basis = row.split("\t")[4];
    if (basis == null) continue;
    for (const m of basis.matchAll(CITATION)) {
      // Keyed by General Assembly as well as number, because Ohio reuses bill numbers: H.B. 111
      // is named here for both the 117th and the 118th and they are different acts.
      const label = `${m[1]} ${m[2]} of the ${ordinal(Number(m[3]))}`;
      const held = counts.get(label) ?? { label, number: m[2]!, lines: 0 };
      held.lines += 1;
      counts.set(label, held);
    }
  }
  return [...counts.values()].sort((a, b) => b.lines - a.lines);
}

function ordinal(n: number): string {
  const rest = n % 100;
  if (rest >= 11 && rest <= 13) return `${n}th`;
  return `${n}${["th", "st", "nd", "rd"][n % 10] ?? "th"}`;
}

/**
 * Bill numbers with a node.
 *
 * Matched on the number alone, because a node's slug carries the calendar year rather than the
 * General Assembly — `hb-96-2025`, not `hb-96-136`. Two acts sharing a number in different
 * assemblies would both match, which is why {@link UNWRITTEN} says so for the H.B. 111 pair
 * rather than leaving it to be discovered.
 */
function noded(): Set<string> {
  return new Set(
    readdirSync(NODES)
      .filter((f) => f.endsWith(".yml"))
      .map((f) => /^[hs]b-(\d+)-\d{4}\.yml$/.exec(f)?.[1])
      .filter((n): n is string => n != null),
  );
}

test("every act the Catalog leans on either has a node or a stated reason it does not", () => {
  const have = noded();
  const unexplained = cited()
    .filter((act) => act.lines >= MATERIAL && !have.has(act.number))
    .filter((act) => UNWRITTEN[act.label] == null)
    .map((act) => `${act.label} — ${act.lines} lines`);

  expect(
    unexplained,
    "an act this many budget lines trace back to needs a node or an entry in UNWRITTEN",
  ).toEqual([]);
});

test("the allowlist holds no act that has since been written", () => {
  // Otherwise the list decays into a record of what used to be missing, which is the failure mode
  // every stale README in this repository has had.
  const have = noded();
  const stale = Object.keys(UNWRITTEN).filter((label) => {
    const number = /[HS]\.B\.\s*(\d+)/.exec(label)?.[1];
    return number != null && have.has(number);
  });
  expect(stale, "remove these from UNWRITTEN; they have nodes now").toEqual([]);
});

test("the share of budget lines whose origin reaches a node only rises", () => {
  /*
   * The measure that matters is lines, not acts. 32 of 45 named acts have no node and together
   * they account for a third of the citations; the four written in this phase account for 761 on
   * their own. Counting acts would report this phase as a 9% improvement and counting lines
   * reports it as what it was: 33.7% to 65.3%.
   */
  const acts = cited();
  const have = noded();
  const total = acts.reduce((n, a) => n + a.lines, 0);
  const covered = acts.filter((a) => have.has(a.number)).reduce((n, a) => n + a.lines, 0);
  const share = covered / total;

  expect(total).toBeGreaterThan(2000);
  expect(
    share,
    `budget-line origins reaching a legislation node: ${(share * 100).toFixed(1)}%`,
  ).toBeGreaterThanOrEqual(COVERAGE_FLOOR);
});

test("the four acts of the Bridge decade are all present", () => {
  // Named individually rather than counted, because the finding they rest on is the sequence:
  // each act re-anchored the guarantee on the year before its biennium, and a chain missing a
  // link is not a chain.
  const have = noded();
  for (const bill of ["59", "64", "49", "166"]) {
    expect(have.has(bill), `H.B. ${bill} has no legislation node`).toBe(true);
  }
});

test("no budget act between DeRolph's end and the Bridge formula is missing", () => {
  /*
   * H.B. 95, H.B. 66, H.B. 119, H.B. 1, H.B. 153 — every biennium from FY2004 to FY2013, with no
   * hole. The run matters because the arguments the corpus makes across it are sequential: the
   * base cost method survives from H.B. 95 to H.B. 119 while its adjustments are removed, and the
   * regime boundary in `foundation-base-cost-formula` is drawn on exactly that contrast.
   */
  const have = noded();
  for (const bill of ["95", "66", "119", "1", "153"]) {
    expect(have.has(bill), `H.B. ${bill} has no legislation node`).toBe(true);
  }
});
