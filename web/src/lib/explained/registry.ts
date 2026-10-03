/**
 * Every Explained topic, in the order the bar's panel and the index list them.
 *
 * A topic module adds itself here and nowhere else: `nav()` builds the panel's runs from this list,
 * `/explained` its index, `[slug].astro` its routes and `og/pages.ts` its cards. A topic added by
 * hand-editing `nav.ts` would be a link the index does not list, which is the disagreement the bar
 * and the homepage were built to rule out.
 *
 * Empty until the pilot (#715). The section exists before its first topic so the chrome change it
 * needs lands alone.
 */

import { PHASE_IN } from "./phase-in.ts";
import type { TopicModule } from "./topic.ts";

export const TOPICS: readonly TopicModule[] = [PHASE_IN];

/** One topic by slug, or a thrown error naming the slug no module registered. */
export function topicModule(slug: string): TopicModule {
  const found = TOPICS.find((t) => t.slug === slug);
  if (!found) throw new Error(`No Explained topic has the slug \`${slug}\``);
  return found;
}
