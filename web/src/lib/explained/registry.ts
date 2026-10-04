/**
 * Every Explained topic, in the order the bar's panel and the index list them.
 *
 * A topic module adds itself here and nowhere else: `nav()` builds the panel's runs from this list,
 * `/explained` its index, `[slug].astro` its routes and `og/pages.ts` its cards. A topic added by
 * hand-editing `nav.ts` would be a link the index does not list, which is the disagreement the bar
 * and the homepage were built to rule out.
 */

import { ANCHOR_CHAIN } from "./anchor-chain.ts";
import { DEDUCTION } from "./deduction.ts";
import { GUARANTEE } from "./guarantee.ts";
import { PHASE_IN } from "./phase-in.ts";
import { POVERTY_COUNT } from "./poverty-count.ts";
import { PROJECTION } from "./projection.ts";
import { REDUCTION_FACTORS } from "./reduction-factors.ts";
import { SOURCES } from "./sources.ts";
import { SPENDING } from "./spending.ts";
import { STATE_SHARE } from "./state-share.ts";
import { escapeHtml } from "../format.ts";
import * as routes from "../routes.ts";
import { topic, type Topic, type TopicModule } from "./topic.ts";
import { WHAT_A_DISTRICT_GETS } from "./what-a-district-gets.ts";

export const TOPICS: readonly TopicModule[] = [WHAT_A_DISTRICT_GETS, STATE_SHARE, PHASE_IN, GUARANTEE, SPENDING, SOURCES, REDUCTION_FACTORS, POVERTY_COUNT, ANCHOR_CHAIN, DEDUCTION, PROJECTION];

/** One topic by slug, or a thrown error naming the slug no module registered. */
export function topicModule(slug: string): TopicModule {
  const found = TOPICS.find((t) => t.slug === slug);
  if (!found) throw new Error(`No Explained topic has the slug \`${slug}\``);
  return found;
}

let built: readonly Topic[] | undefined;

/** Every topic built, once per process, in registry order. */
export function builtTopics(): readonly Topic[] {
  return (built ??= TOPICS.map(topic));
}

/**
 * The topics whose "How we know" cites a node, in registry order (#717).
 *
 * The wiki's link back to a topic is this inversion, read from what each topic cites, so a topic
 * that drops a citation drops the link back with it. Typed by hand, the two would drift apart.
 */
export function topicsCiting(nodeId: string): readonly Topic[] {
  return builtTopics().filter((t) => t.howWeKnow.some((proof) => proof.nodes.includes(nodeId)));
}

/**
 * The link a data page sets under a card, from the term it shows to the topic that explains it.
 *
 * Under the card and not inside it (#717), so the card keeps the height the measure baseline holds.
 */
export function whatIsThis(slug: string): string {
  const t = topicModule(slug);
  return `<p class="note" data-explained="${slug}">What is this? <a href="${routes.explained(slug)}">${escapeHtml(t.question)}</a></p>`;
}
