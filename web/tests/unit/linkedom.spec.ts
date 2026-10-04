/**
 * The patch on linkedom's `EventTarget`, in `patches/linkedom@0.18.13.patch` (#648).
 *
 * Upstream gives every node a listener map in its constructor. The build makes hundreds of
 * thousands of nodes and gives none of them a listener, and the map's WeakMap entry alone was over
 * a quarter of the build's time in the collector. The patch makes the map on the first
 * `addEventListener`. These hold the two things that change: a listener still fires, and a node
 * that never had one has no map, which `removeEventListener` must not trip over.
 */

import { parseHTML } from "linkedom";
import { expect, test } from "vitest";

test("a listener added to a node still hears its event", () => {
  const { document, Event } = parseHTML("<!doctype html><html><body><p>x</p></body></html>");
  const p = document.querySelector("p")!;
  const heard: string[] = [];
  p.addEventListener("ping", (event) => heard.push(event.type));
  p.dispatchEvent(new Event("ping"));
  expect(heard).toEqual(["ping"]);
});

test("a node that never had a listener can be asked to drop one, and dispatched on", () => {
  const { document, Event } = parseHTML("<!doctype html><html><body><p>x</p></body></html>");
  const p = document.querySelector("p")!;
  expect(() => p.removeEventListener("ping", () => {})).not.toThrow();
  expect(p.dispatchEvent(new Event("ping"))).toBe(true);
});
