/**
 * The slice of Node's standard library the tests use, declared here instead of installed.
 *
 * `@types/node` is an npm dependency, and this repository takes none — the Rust workspace for
 * the reason its own README gives, and this side for the same one. What the tests actually
 * touch is four functions and one property, so declaring them is cheaper than a package.
 *
 * If a test needs more of Node than this, add it here rather than reaching for the package.
 */

declare module "node:test" {
  export function test(name: string, fn: () => void | Promise<void>): void;
}

declare module "node:assert/strict" {
  interface Assert {
    (value: unknown, message?: string): asserts value;
    ok(value: unknown, message?: string): asserts value;
    equal(actual: unknown, expected: unknown, message?: string): void;
    notEqual(actual: unknown, expected: unknown, message?: string): void;
    deepEqual(actual: unknown, expected: unknown, message?: string): void;
  }
  const assert: Assert;
  export default assert;
}

declare module "node:fs" {
  export function readFileSync(path: string, encoding: "utf8"): string;
}

declare module "node:path" {
  export function join(...parts: string[]): string;
}

interface ImportMeta {
  /** Directory of the current module. Node 20.11 and later. */
  readonly dirname: string;
}
