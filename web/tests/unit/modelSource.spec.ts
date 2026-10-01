import { describe, expect, test } from "vitest";

import { datedOf, modelSource } from "../../src/lib/modelSource.ts";

describe("the model's date is read from its catalog entry", () => {
  test("the FY2027 calculator is dated 16 December 2025", () => {
    expect(modelSource(2027)).toEqual({
      slug: "dew-fy27-funding-calculator",
      href: "/wiki/source/dew-fy27-funding-calculator",
      date: "16 December 2025",
      iso: "2025-12-16",
    });
  });

  test("a single-digit day is padded in the machine form only", () => {
    expect(datedOf({ slug: "x", body: "The file, dated 3 March 2024. A model." })).toEqual({
      date: "3 March 2024",
      iso: "2024-03-03",
    });
  });

  test("an entry with no date, or two, refuses rather than guessing", () => {
    expect(() => datedOf({ slug: "x", body: "Published in December." })).toThrow(/found 0/);
    expect(() => datedOf({ slug: "x", body: "dated 1 May 2024, replacing one dated 2 April 2024" })).toThrow(/found 2/);
  });

  test("a model year the corpus has not catalogued fails the build", () => {
    expect(() => modelSource(2031)).toThrow(/dew-fy31-funding-calculator/);
  });
});
