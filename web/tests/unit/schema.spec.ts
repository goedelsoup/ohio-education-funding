/**
 * The schemas, and the defects they exist to catch.
 *
 * Every case below is a mistake this repository actually made. They are written as tests rather
 * than as comments because the schema is only worth having if it fails on the real thing, and
 * "would this have been caught?" is answerable.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { z } from "astro/zod";
import { expect, test } from "vitest";
import YAML from "yaml";

import { loadCorpus, resolveTarget } from "../../src/lib/corpus.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import {
  NodeSchema,
  OBSERVATION_PROPERTY,
  OntologyClassSchema,
} from "../../src/lib/schema/corpus.ts";
import { BundleSchema } from "../../src/lib/schema/feed.ts";

const raw: unknown = JSON.parse(
  readFileSync(join(import.meta.dirname, "../../public/data/bundle.json"), "utf8"),
);

test("the committed feed matches the shape this site reads", () => {
  const parsed = BundleSchema.safeParse(raw);
  const problems = parsed.success
    ? []
    : parsed.error.issues.slice(0, 5).map((i) => `${i.path.join(".")}: ${i.message}`);
  expect(problems).toEqual([]);
});

test("a field the Rust adds and this mirror does not know about fails the build", () => {
  // The whole point of `.strict()`. `crates/bundle` is authoritative and this file is a
  // hand-maintained mirror of it; the failure that matters is the two drifting apart, and an
  // unexpected key is the earliest possible signal of exactly that.
  const feed = structuredClone(raw) as Record<string, unknown>;
  (feed.districts as Record<string, unknown>[])[0]!.new_rust_field = 1;
  expect(BundleSchema.safeParse(feed).success).toBe(false);
});

test("a field the Rust renames fails rather than rendering as an em dash", () => {
  // This is the failure a cast could not see. `realized_aid_per_pupil` going missing produced
  // `undefined`, which `money()` formats as "—" — indistinguishable from "not reported".
  const feed = structuredClone(raw) as Record<string, unknown>;
  const district = (feed.districts as Record<string, unknown>[])[0]!;
  district.realised_aid_per_pupil = district.realized_aid_per_pupil;
  delete district.realized_aid_per_pupil;
  expect(BundleSchema.safeParse(feed).success).toBe(false);
});

test("an IRN that is not six digits fails", () => {
  // Every route on this site is keyed on it, and 28 district names are not unique, so the IRN is
  // the only thing standing between two districts and the same page.
  const feed = structuredClone(raw) as Record<string, unknown>;
  (feed.districts as Record<string, unknown>[])[0]!.irn = "43786";
  expect(BundleSchema.safeParse(feed).success).toBe(false);
});

test("an enrollment history of the wrong length fails", () => {
  // `adm_history` is a tuple of three. A district with a short history cannot be projected, and a
  // page that quietly dropped it would report a statewide total over a subset of the panel.
  const feed = structuredClone(raw) as Record<string, unknown>;
  (feed.districts as Record<string, unknown>[])[0]!.adm_history = [1, 2];
  expect(BundleSchema.safeParse(feed).success).toBe(false);
});

test("the corpus validates with no errors", () => {
  const errors = loadCorpus().diagnostics.filter((d) => d.severity === "error");
  expect(errors.map((d) => `${d.file}: ${d.message}`)).toEqual([]);
});

test("links written as a paragraph are rejected", () => {
  // The defect that motivated all of this. `scenario/guarantee-phase-out` wrote its entire link
  // list as prose; it was valid YAML, so nothing complained, and its fifteen edges were invisible
  // to every consumer for as long as the node existed.
  const proseLinks = {
    class: "scenario",
    label: "Something",
    description: "A node.",
    properties: {},
    links: "Perturbs [a](../parameter/a.yml) and baselines on [b](../funding-regime/b.yml).",
  };
  const result = NodeSchema.safeParse(proseLinks);
  expect(result.success).toBe(false);
  if (!result.success) {
    expect(result.error.issues.some((i) => i.path.includes("links"))).toBe(true);
  }
});

test("a node with no outgoing links is rejected", () => {
  // The corpus's own rule: a node nothing reaches and that reaches nothing is a gap, not a fact.
  const orphan = { class: "metric", label: "X", description: "Y", properties: {}, links: [] };
  expect(NodeSchema.safeParse(orphan).success).toBe(false);
});

test("a property value that is not a string is rejected", () => {
  // `irn: 044933` unquoted would parse as a number in a schema that allowed one, and lose its
  // leading zero — silently turning a valid IRN into an invalid one.
  const node = {
    class: "education-agency",
    label: "X",
    description: "Y",
    properties: { irn: 44933 },
    links: [{ target: "../education-agency.ont.yml", relationship: "instance-of" }],
  };
  expect(NodeSchema.safeParse(node).success).toBe(false);
});

test("every corpus file on disk parses and validates individually", () => {
  // Belt and braces against `loadCorpus` becoming tolerant of something it should not be.
  const corpus = loadCorpus();
  const failures: string[] = [];
  for (const node of corpus.nodes) {
    const file = join(import.meta.dirname, "../../../.yidam/corpus", `${node.id}.yml`);
    const parsed = NodeSchema.safeParse(YAML.parse(readFileSync(file, "utf8")));
    if (!parsed.success) failures.push(`${node.id}: ${parsed.error.issues[0]?.message}`);
  }
  for (const entry of corpus.classes) {
    const file = join(
      import.meta.dirname,
      "../../../.yidam/corpus",
      `${entry.className}.ont.yml`,
    );
    const parsed = OntologyClassSchema.safeParse(YAML.parse(readFileSync(file, "utf8")));
    if (!parsed.success) failures.push(`${entry.className}.ont: ${parsed.error.issues[0]?.message}`);
  }
  expect(failures).toEqual([]);
});

test("every class states whether its edge vocabulary is open", () => {
  // Required with no default, so a class nobody has decided about cannot pass silently. All 13
  // currently say `characteristic`, which is what the corpus has always done and never said.
  const corpus = loadCorpus();
  for (const entry of corpus.classes) {
    expect(["characteristic", "exhaustive"]).toContain(entry.edgePolicy);
  }
  expect(corpus.classes).toHaveLength(13);
});

test("an ontology missing edge_policy is rejected", () => {
  const withoutPolicy = {
    class: "metric",
    label: "Metric",
    description: "A measure.",
    properties: [],
    edges: [],
  };
  expect(OntologyClassSchema.safeParse(withoutPolicy).success).toBe(false);
  expect(
    OntologyClassSchema.safeParse({ ...withoutPolicy, edge_policy: "characteristic" }).success,
  ).toBe(true);
  // And it is an enum, so a typo is not quietly treated as "open".
  expect(OntologyClassSchema.safeParse({ ...withoutPolicy, edge_policy: "open" }).success).toBe(
    false,
  );
});

test("the corpus is clean under the policy it declares", () => {
  // The point of stating the policy: what is left is signal. 46 undeclared relationships were
  // noise against an unstated assumption; the four undeclared properties were real omissions and
  // are now declared. Zero of either should remain.
  const diagnostics = loadCorpus().diagnostics;
  expect(diagnostics.map((d) => `${d.severity} ${d.file}: ${d.message}`)).toEqual([]);
});

test("a year-stamped observation is allowed where a bare property name would not be", () => {
  // `fy2024_profile` recurs across agency nodes and is a snapshot rather than a schema property.
  // Declaring it would mean editing an ontology every fiscal year to permit next year's.
  expect(OBSERVATION_PROPERTY.test("fy2024_profile")).toBe(true);
  expect(OBSERVATION_PROPERTY.test("fy2021_idea_part_b")).toBe(true);
  expect(OBSERVATION_PROPERTY.test("fy2026_27_appropriation")).toBe(true);
  // Not a licence for anything unrecognised.
  expect(OBSERVATION_PROPERTY.test("profile")).toBe(false);
  expect(OBSERVATION_PROPERTY.test("fy_profile")).toBe(false);
  expect(OBSERVATION_PROPERTY.test("notes")).toBe(false);
});

test("a bare class name resolves to that class's page", () => {
  // Two metric nodes write `target:` in the ontology's vocabulary — a class name, not a path.
  // Unhandled it emitted `<a href="education-agency">`, a relative link that 404s while looking
  // entirely normal in the markup.
  const resolved = resolveTarget("education-agency", "metric");
  expect(resolved.resolved).toBe(true);
  expect(resolved.href).toBe("/wiki/education-agency");
});

test("a target shape nothing recognises is reported rather than emitted", () => {
  const resolved = resolveTarget("../../somewhere/odd.txt", "metric");
  expect(resolved.resolved).toBe(false);
});

test("the committed JSON Schemas match the zod definitions", () => {
  // Generated and committed, so the editor can read them without a build. Which means they can go
  // stale, so they are checked like the feed is.
  for (const [file, schema] of [
    ["corpus-node.json", NodeSchema],
    ["corpus-ontology.json", OntologyClassSchema],
  ] as const) {
    // The prose fields are added by the emitter, not by zod, so they are stripped from both sides
    // rather than only from the committed copy.
    const strip = (o: Record<string, unknown>) => {
      const { $schema: _s, title: _t, description: _d, ...body } = o;
      return body;
    };
    const committed = JSON.parse(
      readFileSync(join(import.meta.dirname, "../../../.yidam/schemas", file), "utf8"),
    ) as Record<string, unknown>;
    expect(strip(committed), `${file} is stale — run \`pnpm schemas\``).toEqual(
      strip(z.toJSONSchema(schema, { io: "input" }) as Record<string, unknown>),
    );
  }
});

test("the feed the site builds from is the one the schema accepted", () => {
  // `loadFeed` parses rather than casts, so this is really asserting the wiring: what pages get is
  // schema output, not a cast over whatever was on disk.
  const { bundle } = loadFeed();
  expect(bundle.districts).toHaveLength(609);
  expect(BundleSchema.safeParse(bundle).success).toBe(true);
});
