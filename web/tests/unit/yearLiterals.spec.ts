/**
 * A year that moves when a fixture advances must be read, not typed.
 *
 * # Why this is a test and not a convention
 *
 * Because the convention was already written down and the corpus kept breaking it. Three separate
 * labels were found stale in prose that no compiler could check:
 *
 * - `Bundle::provenance` said "millage is TY2023" while all 609 districts carried `tax_year: 2024`
 * - `denominators.ts` said the MR-81 series ran `FY2001-FY2011` against a feed carrying
 *   `FY1998-FY2014`, and that sponsors rose "from 730 to 949" against an actual 718 to 1001
 * - `taxes.astro` carried `TY2023`, `TY2024` and a hard-coded count of `219`
 *
 * Every one was correct when written. Each went wrong when a fixture advanced, and none of them
 * could disagree with the data sitting beside it in a way anything would notice.
 *
 * # Why an allowlist rather than a ban
 *
 * Because half the year literals on this site are *historical facts* and belong in prose. "FY2020,
 * a year Ohio froze funding under the Bridge formula" does not move when the report card advances;
 * neither does "H.B. 110 priced FY2022 from FY2018 salaries". Banning the digits outright would
 * force those into derivations that assert a relationship which is not there.
 *
 * So the rule is: **every year literal in rendered code is either derived or listed here with a
 * reason.** A new one fails the build until somebody decides which it is, which is the moment the
 * question is cheap to answer.
 */

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

import { expect, test } from "vitest";

const SRC = join(import.meta.dirname, "../../src");

/**
 * Years that are facts about the past rather than labels on a moving fixture.
 *
 * Keyed by the file, valued by *why* — the reason is the point. An entry whose reason reads "it
 * is what the page says" is not a historical fact, it is an un-migrated literal.
 */
const HISTORICAL: Record<string, string> = {
  "lib/district.ts":
    "FY2020 as the Bridge-formula freeze, the FY2021 `[L1]` statutory base, FY2019 poverty, and the FY2021 count freeze — all fixed events, none of which move when a fixture advances.",
  "lib/glossary.ts":
    "Definitions: the FY2020 guarantee anchor, FY2022 cost pricing, and a worked tax-year-versus-fiscal-year example whose whole point is the specific pair of numbers.",
  "lib/history.ts":
    "FY2009-FY2011 as the Census panel's own caveat window, and FY2020 as the furthest back anything else in the feed reaches.",
  "lib/outcomes.ts": "FY2020 as the guarantee baseline, in the sentence explaining who is on it and why.",
  "lib/scenario.ts":
    "FY2020 as the guarantee baseline every lever on the scenario page moves against — the anchor itself, not a label on it.",
  "lib/statewide.ts": "FY2021-FY2024 as the federal pandemic relief years — a fixed span, not a fixture.",
  "lib/tax.ts": "FY2008 gap aid, a superseded mechanism, and FY2027 as the counterfactual's stated input year.",
  "lib/denominators.ts":
    "MR-81's definitional break — `AdmCount` through FY2009, then `CECount` from FY2010 — is a fact about a closed series that ended in FY2014, not a label on a moving fixture. Every other year in this file is now read from `series_years`.",
  "lib/basecost.ts": "H.B. 96 holding cost inputs at FY2022 through FY2027 — an act's terms, not a label.",
  "components/ScenarioControls.astro":
    "Lever names: the FY2022 cost inputs and the FY2020 floor are what the levers *are*, and renaming them with a derived year would make the control describe something else.",
  "pages/method.astro":
    "The pricing history of three acts — H.B. 110 from FY2018 salaries, H.B. 33 to FY2022, H.B. 96 holding there — and FY2022 as the pandemic peak.",
};

/** Everything under `src`, minus the OG image routes, which render no prose. */
function sources(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return path.includes("/og") ? [] : sources(path);
    return /\.(ts|astro)$/.test(entry.name) && !path.endsWith(".png.ts") ? [path] : [];
  });
}

/**
 * Blank out comments, keeping offsets so a hit can be classified by position.
 *
 * A year in a docstring is documentation and always fine — most of the 145 in this tree explain
 * *why* a year is what it is, which is exactly the writing this rule wants more of.
 */
function withoutComments(source: string): string {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, " "))
    .replace(/^\s*\/\/.*$/gm, (m) => " ".repeat(m.length))
    .replace(/\{\/\*[\s\S]*?\*\/\}/g, (m) => m.replace(/[^\n]/g, " "));
}

test("a year literal is either derived or declared a historical fact", () => {
  const YEAR = /FY20\d\d|TY20\d\d|\b20[0-2]\d-[0-9]\d\b/g;
  const undeclared: string[] = [];

  for (const file of sources(SRC)) {
    const relative = file.slice(SRC.length + 1);
    if (relative in HISTORICAL) continue;

    const raw = readFileSync(file, "utf8");
    const live = withoutComments(raw);
    for (const match of raw.matchAll(YEAR)) {
      const at = match.index ?? 0;
      if (live.slice(at, at + match[0].length) !== match[0]) continue;
      const line = raw.slice(0, at).split("\n").length;
      undeclared.push(`${relative}:${line}  ${match[0]}`);
    }
  }

  expect(
    undeclared,
    "derive it from `series_years`, or add the file to HISTORICAL with the reason",
  ).toEqual([]);
});

test("every allowlisted file actually still carries a year literal", () => {
  /*
   * The other half of the ratchet. An allowlist entry that no longer matches anything is a licence
   * nobody is using, and it will silently cover the next literal somebody adds to that file.
   */
  const stale: string[] = [];
  for (const relative of Object.keys(HISTORICAL)) {
    const raw = readFileSync(join(SRC, relative), "utf8");
    if (!/FY20\d\d|TY20\d\d|\b20[0-2]\d-[0-9]\d\b/.test(withoutComments(raw))) {
      stale.push(relative);
    }
  }
  expect(stale, "this file no longer needs its HISTORICAL entry — remove it").toEqual([]);
});

test("each allowlist entry gives a reason, not a restatement", () => {
  // "It is what the page says" is not a historical fact, it is an un-migrated literal.
  for (const [file, reason] of Object.entries(HISTORICAL)) {
    expect(reason.length, file).toBeGreaterThan(60);
  }
});
