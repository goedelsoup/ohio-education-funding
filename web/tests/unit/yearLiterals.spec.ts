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
 *
 * # The allowlist is keyed by literal, not by file
 *
 * It used to be keyed by file: `if (relative in HISTORICAL) continue`. So an entry admitted the
 * *file* and every year literal in it — including ones added years later, and ones the entry's own
 * reason never mentioned. That is not the rule stated above, and the gap was silent by
 * construction.
 *
 * `lib/tax.ts` is how it was found. Its entry reads "FY2008 gap aid, a superseded mechanism, and
 * FY2027 as the counterfactual's stated input year"; the file also carried a rendered `TY2024` in
 * the deferral sentence, describing the moving tax-year fixture. The gate was green on it the
 * whole time, and stayed green when the literal was typed back in after being derived.
 *
 * Across the eleven allowlisted files there were 32 distinct rendered literals against reasons
 * naming about twenty. Listing the literals is what makes the reason answerable: an entry can no
 * longer license a year nobody has looked at.
 */

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

import { expect, test } from "vitest";

const SRC = join(import.meta.dirname, "../../src");

/**
 * A file's licence to carry year literals: which ones, and why they are not derivable.
 *
 * `allowed` is checked against the literals themselves, so an entry covers exactly the years its
 * reason argues for. Both directions are enforced — an undeclared literal fails, and a declared
 * year that no longer appears fails too, because a licence nobody is using is one waiting to cover
 * the next literal somebody adds.
 */
interface Allowance {
  /** Why these are facts about the past rather than labels on a moving fixture. */
  reason: string;
  /** Exactly the literals the reason argues for, as they are written in the source. */
  allowed: string[];
}

/**
 * Years that are facts about the past rather than labels on a moving fixture.
 *
 * Keyed by the file, valued by *why* — the reason is the point. An entry whose reason reads "it
 * is what the page says" is not a historical fact, it is an un-migrated literal.
 */
const HISTORICAL: Record<string, Allowance> = {
  "components/district/AidSource.astro": {
    allowed: ["FY2020"],
    reason:
      "FY2020 as the Bridge-formula freeze, a year Ohio froze funding rather than computing it — a fixed event in the condition that explains an enrollment change, and one that does not move when a fixture advances.",
  },
  "components/district/HoldHarmless.astro": {
    allowed: ["FY2021"],
    reason:
      "FY2021 as the funding base the guarantee and the formula transition supplement each compare against, and the year all three of Ohio's hold-harmless mechanisms are anchored to — a fixed event, which stays FY2021 whatever the model year becomes.",
  },
  "components/district/Transportation.astro": {
    allowed: ["FY2021"],
    reason:
      "FY2021 as the `[L1]` statutory base the transportation guarantee holds a district at — the third of the hold-harmless mechanisms anchored to that year, a fixed event rather than a label on a moving fixture.",
  },
  "components/district/TargetedAssistance.astro": {
    allowed: ["FY2019"],
    reason:
      "FY2019 poverty in the supplemental targeted assistance test: the statute asks whether a district was poor in FY2019 specifically, so the year is part of the rule rather than a vintage of the data.",
  },
  "components/district/CareerTechnical.astro": {
    allowed: ["FY2021"],
    reason:
      "FY2021 as the career-technical count freeze — the department's own column is enrolment as it stood in FY2021, a fixed event that the sentence exists to point out, and which does not move when a fixture advances.",
  },
  "components/district/EnglishLearners.astro": {
    allowed: ["FY2021"],
    reason:
      "FY2021 as the English-learner count freeze — the department's column header reads ADM-FY21, so the counts are enrolment as it stood in FY2021, a fixed event the note is there to name.",
  },
  "components/district/Dpia.astro": {
    allowed: ["FY2025", "FY2026"],
    reason:
      "FY2025 and FY2026 are DPIA's two poverty input years — properties of fixtures that will move, declared here rather than derived because the feed carries neither of them. That is the honest state of it, and precisely what a file-keyed allowlist could not say.",
  },
  "components/district/WhatThisIsNot.astro": {
    allowed: ["FY2023"],
    reason:
      "FY2023 is the profile report's valuation vintage — a property of a fixture that will move, declared here rather than derived because the feed does not carry it, beside the spending year that is read with `yearOf`.",
  },
  "lib/history.ts": {
    allowed: ["FY2020"],
    reason:
      "FY2020 as the furthest back anything else in the feed reaches. The FY2009-FY2011 caveat window it also carried is computed from the series now (#705).",
  },
  "lib/outcomes.ts": {
    allowed: ["FY2020"],
    reason:
      "FY2020 as the guarantee baseline, in the sentence explaining who is on it and why. The FY2024 that used to sit beside it named the District Profile Report and reads from the fixture now — which is exactly what an allowlist licensing a whole file had been hiding.",
  },
  "lib/scenario.ts": {
    allowed: ["FY2020"],
    reason:
      "FY2020 as the guarantee baseline every lever on the scenario page moves against — the anchor itself, not a label on it.",
  },
  "lib/statewide.ts": {
    allowed: ["FY2021", "FY2024"],
    reason:
      "FY2021-FY2024 as the federal pandemic relief years — a fixed span, not a fixture. The FY2025 that used to follow it was the report card's spending year, and is read from `series_years` now.",
  },
  "lib/tax.ts": {
    allowed: ["FY2008", "FY2027"],
    reason:
      "FY2008 gap aid, a superseded mechanism, and FY2027 as the counterfactual's stated input year. This is the entry that exposed the defect: it also covered a rendered TY2024 describing the moving tax-year fixture, which is read off the district's own panel now.",
  },
  "lib/denominators.ts": {
    allowed: ["FY2010", "FY2011"],
    reason:
      "MR-81's definitional break — `AdmCount` through FY2010, then `CECount` from FY2011, the Octobers of 2009 and 2010 (#705) — is a fact about the count's history, not a label on a moving fixture. Every other year in this file is now read from `series_years`.",
  },
  "lib/basecost.ts": {
    allowed: ["FY2022", "FY2027"],
    reason: "H.B. 96 holding cost inputs at FY2022 through FY2027 — an act's terms, not a label.",
  },
  "components/ScenarioControls.astro": {
    allowed: ["FY2019", "FY2020", "FY2021", "FY2022"],
    reason:
      "Lever names: the FY2022 cost inputs and the FY2020 floor are what the levers *are*, and renaming them with a derived year would make the control describe something else. FY2019 joins them as the DPIA phase-in's own anchor — R.C. 3317.02(N)(2) bases it on the FY2019 DPIA payment while the general term uses FY2020, and the note exists to say the two dials interpolate from different years. FY2021 is the third of the same kind and the sharpest: Section 265.225 names that year's total and no other, so the three readings of the backstop are readings of an FY2021 base — a derived year there would describe a section that does not exist.",
  },
  "components/Baseline.astro": {
    allowed: ["FY2022"],
    reason:
      "The vintage the enacted act prices base cost from, named in the card that says what the " +
      "runner's levers move away from — on both views since #562. It is a property of H.B. 96 rather than of a fixture — the " +
      "act holds the salary inputs at FY2022 through FY2027, so the year does not advance when " +
      "the feed does, and there is no series to derive it from because the feed carries the " +
      "model's output rather than the year its prices were read in.",
  },
  "pages/method.astro": {
    allowed: ["FY2018", "FY2022"],
    reason:
      "The pricing history of three acts — H.B. 110 from FY2018 salaries, H.B. 33 to FY2022, H.B. 96 holding there — and FY2022 as the pandemic peak.",
  },
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
 * Blank out what is not a year on a page, keeping offsets so a hit can be classified by position.
 *
 * A year in a docstring is documentation and always fine — most of the 145 in this tree explain
 * *why* a year is what it is, which is exactly the writing this rule wants more of.
 *
 * A URL is the other case, and it is not documentation — it is a live string that no page renders
 * as a date. `https://json-schema.org/draft/2020-12/schema` is the version of a specification; it
 * matches the `20\d\d-\d\d` arm exactly and means nothing the allowlist can argue about, since
 * the mechanism's question — is this a fact about the past, or a label on a moving fixture? — has
 * no answer for it. Blanking the URL is the honest classification; an allowance would have been a
 * licence granted on a false premise.
 */
function withoutUnrendered(source: string): string {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, " "))
    .replace(/^\s*\/\/.*$/gm, (m) => " ".repeat(m.length))
    .replace(/\{\/\*[\s\S]*?\*\/\}/g, (m) => m.replace(/[^\n]/g, " "))
    .replace(/\bhttps?:\/\/\S+/g, (m) => " ".repeat(m.length));
}

/** Every live year literal in one file, as `LITERAL` paired with the line it is on. */
function literals(file: string): { year: string; line: number }[] {
  const YEAR = /FY20\d\d|TY20\d\d|\b20[0-2]\d-[0-9]\d\b/g;
  const raw = readFileSync(file, "utf8");
  const live = withoutUnrendered(raw);
  const found: { year: string; line: number }[] = [];
  for (const match of raw.matchAll(YEAR)) {
    const at = match.index ?? 0;
    // A hit whose characters survived comment-blanking is rendered; one that did not is prose
    // about a year rather than a year on a page.
    if (live.slice(at, at + match[0].length) !== match[0]) continue;
    found.push({ year: match[0], line: raw.slice(0, at).split("\n").length });
  }
  return found;
}

test("a year literal is either derived or declared, one literal at a time", () => {
  const undeclared: string[] = [];

  for (const file of sources(SRC)) {
    const relative = file.slice(SRC.length + 1);
    // The allowance, not the file. `relative in HISTORICAL` used to end the iteration here, which
    // is the whole defect: it admitted every literal in the file rather than the declared ones.
    const allowed = HISTORICAL[relative]?.allowed ?? [];
    for (const { year, line } of literals(file)) {
      if (allowed.includes(year)) continue;
      undeclared.push(`${relative}:${line}  ${year}`);
    }
  }

  expect(
    undeclared,
    "derive it from `series_years`, or add the literal to its file's `allowed` with the reason",
  ).toEqual([]);
});

test("every declared literal is one the file still carries", () => {
  /*
   * The other half of the ratchet, and it now runs per literal rather than per file. An entry
   * matching nothing is a licence nobody is using, and under the old file-keyed check a single
   * surviving literal kept the whole entry alive — so an allowance could name four years, carry
   * one, and go on covering everything else the file grew.
   */
  const stale: string[] = [];
  for (const [relative, { allowed }] of Object.entries(HISTORICAL)) {
    const present = new Set(literals(join(SRC, relative)).map((hit) => hit.year));
    for (const year of allowed) {
      if (!present.has(year)) stale.push(`${relative}  ${year}`);
    }
  }
  expect(stale, "this year is no longer rendered in that file — drop it from `allowed`").toEqual([]);
});

test("an allowance declares at least one literal, and gives a reason rather than a restatement", () => {
  // "It is what the page says" is not a historical fact, it is an un-migrated literal.
  for (const [file, { reason, allowed }] of Object.entries(HISTORICAL)) {
    expect(reason.length, file).toBeGreaterThan(60);
    expect(allowed.length, file).toBeGreaterThan(0);
    // Duplicates would make the two directions above disagree about what the entry declares.
    expect(new Set(allowed).size, file).toBe(allowed.length);
  }
});

test("the reason names every year it licenses", () => {
  /*
   * The failure this whole change is about, one level up. An `allowed` entry nobody argued for is
   * the file-keyed allowlist again in a smaller box: the literal is listed, and the sentence
   * beside it is about a different year. Naming it is cheap and it is the only thing that makes
   * the list reviewable.
   */
  const unargued: string[] = [];
  for (const [file, { reason, allowed }] of Object.entries(HISTORICAL)) {
    for (const year of allowed) {
      if (!reason.includes(year)) unargued.push(`${file}  ${year}`);
    }
  }
  expect(unargued, "say in the reason why this year is a fact about the past").toEqual([]);
});

test("the chart module composes no year, and the series manifest writes school years short", () => {
  /*
   * #610: chart text wrote years four ways. Two of them were composed rather than typed, so the
   * literal scan above could not see them — `FY${last.year}` in the fan's foot, and the crate's
   * EdChoice editions drawn as "2024-2025" beside every other school year's "2024-25". The chart
   * module and the module that turns the manifest into charts take a year from a formatter, and the
   * manifest the crate writes is held to the site's school-year form.
   */
  const composed = /(?:FY|TY)\$\{|\$\{[^}]*\}-\$\{/g;
  const found: string[] = [];
  for (const relative of ["lib/plot/spec.ts", "lib/corpusSeries.ts"]) {
    const live = withoutUnrendered(readFileSync(join(SRC, relative), "utf8"));
    for (const match of live.matchAll(composed)) {
      found.push(`${relative}:${live.slice(0, match.index).split("\n").length}  ${match[0]}`);
    }
  }
  expect(found, "write the year through `year.ts` or `yearLabel.ts`").toEqual([]);

  const manifest = readFileSync(join(SRC, "../../crates/series.json"), "utf8");
  const long = [...manifest.matchAll(/"label": "[^"]*\b(20\d\d-20\d\d)\b[^"]*"/g)].map((m) => m[1]);
  expect(long, "a school year is `2024-25` — see `Edition::school_year`").toEqual([]);
});
