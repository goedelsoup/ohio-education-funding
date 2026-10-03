/**
 * Every Explained topic, in the order the bar's panel and the index list them.
 *
 * A topic module adds itself here and nowhere else: `nav()` builds the panel's runs from this list,
 * `/explained` its index, `[slug].astro` its routes and `og/pages.ts` its cards. A topic added by
 * hand-editing `nav.ts` would be a link the index does not list, which is the disagreement the bar
 * and the homepage were built to rule out.
 */

import { GUARANTEE } from "./guarantee.ts";
import { PHASE_IN } from "./phase-in.ts";
import { SPENDING } from "./spending.ts";
import { STATE_SHARE } from "./state-share.ts";
import type { TopicModule } from "./topic.ts";
import { WHAT_A_DISTRICT_GETS } from "./what-a-district-gets.ts";

export const TOPICS: readonly TopicModule[] = [WHAT_A_DISTRICT_GETS, STATE_SHARE, PHASE_IN, GUARANTEE, SPENDING];

/** One topic by slug, or a thrown error naming the slug no module registered. */
export function topicModule(slug: string): TopicModule {
  const found = TOPICS.find((t) => t.slug === slug);
  if (!found) throw new Error(`No Explained topic has the slug \`${slug}\``);
  return found;
}
