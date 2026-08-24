/**
 * A card, rasterized.
 *
 * Two libraries in sequence: satori lays the card out and emits SVG, resvg turns that SVG into a
 * PNG. Neither reaches a browser — both are devDependencies, and this module is imported only by
 * the endpoints under `src/pages/og/`, which run during `astro build` and emit files.
 *
 * # `loadSystemFonts: false` is not an optimisation, it is the whole cost
 *
 * resvg builds a font database when it is constructed, and by default that means scanning every
 * font installed on the machine. It does this per instance. Measured on this card at 1200×630:
 * **121 ms** with the default, **9 ms** without — and since roughly a thousand cards are rendered
 * per build, that flag is the difference between two minutes and nine seconds.
 *
 * It is also free, because satori is configured to embed glyph outlines: by the time resvg sees
 * the SVG there is no `<text>` element left in it, only `<path>`. A font database would be built,
 * scanned, and consulted zero times.
 *
 * Anyone tempted to drop the flag should render one card and look at it. If the text is still
 * there, nothing was gained; if it is gone, the fix is satori's `embedFont`, never this.
 */

import { Resvg } from "@resvg/resvg-js";
import satori from "satori";

import { HEIGHT, WIDTH, render as tree, type Card } from "./card.ts";
import { fonts } from "./font.ts";
import { indexedPng } from "./indexed.ts";

/** See the header. Not defaults worth overriding at a call site. */
const RESVG = {
  fitTo: { mode: "width", value: WIDTH },
  font: { loadSystemFonts: false },
} as const;

/**
 * Lay a card out and rasterize it. The bytes are what an endpoint returns as its body.
 *
 * The pixmap goes to {@link indexedPng} before `asPng` gets a chance at it. A card is flat fills
 * and glyph outlines — 128 colours, every pixel opaque — so writing it as truecolour with alpha
 * spent 32 bits a pixel to say one of 128 things, and the whole set came to 51.8 MB. As a palette
 * it is 45% of that and decodes to the same picture.
 *
 * `asPng()` is the fallback and not the path: where the palette encoder succeeds, resvg's own PNG
 * writer is never asked to run at all.
 */
export async function png(card: Card): Promise<Buffer> {
  const svg = await satori(tree(card) as Parameters<typeof satori>[0], {
    width: WIDTH,
    height: HEIGHT,
    fonts: fonts(),
    // Text becomes paths. Named rather than left to the default, because `loadSystemFonts: false`
    // below is only correct while this holds.
    embedFont: true,
  });
  const rendered = new Resvg(svg, RESVG).render();
  return (
    indexedPng(rendered.pixels, rendered.width, rendered.height) ??
    Buffer.from(rendered.asPng())
  );
}

/**
 * The response an image endpoint returns.
 *
 * One helper rather than the same four lines in six route files, and the place the content type is
 * spelled — a PNG served as `text/html` is a broken preview on every platform at once, and it is
 * exactly the kind of thing that is invisible until someone pastes a link into Slack.
 */
export async function respond(card: Card): Promise<Response> {
  return new Response(new Uint8Array(await png(card)), {
    headers: { "Content-Type": "image/png" },
  });
}

/**
 * Rasterize an SVG that is already written, at a given square size.
 *
 * The icons, and only the icons — they are drawn by hand in `public/favicon.svg` rather than laid
 * out by satori, so they skip the first half of the pipeline. `loadSystemFonts` stays off for the
 * same reason: the icon carries no text, and a font database built to render a rectangle and two
 * bars would cost more than the rest of the build.
 */
export function rasterize(svg: string, size: number): Buffer {
  return Buffer.from(
    new Resvg(svg, {
      fitTo: { mode: "width", value: size },
      font: { loadSystemFonts: false },
    })
      .render()
      .asPng(),
  );
}
