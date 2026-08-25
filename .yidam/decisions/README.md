# Decision records

Why this repository is shaped the way it is: what was chosen, what was rejected and on what
evidence, and where the answer turned out to be wrong.

These are published. `/wiki/decision/<id>` on the site renders one file each, and 13 links from
corpus nodes and catalog entries point into them. They were unpublished until the phase that
[#23](https://github.com/goedelsoup/ohio-education-funding/issues/23) records, on the ground that a
decision record is a working document for whoever is authoring the corpus — which was true when it
was written and stopped being true once these became the only place two things live: **corrections
to claims the site published**, and **rejections a later phase overturned**.

## Shape

One file per decision, named for what it settled rather than for the phase that settled it.
`id:` matches the file stem and is the URL.

    id            the file stem, and the page's own title
    summary       one paragraph. Leads the page; not a section.
    context       what was in front of whoever decided
    decision      what was chosen
    consequences  what followed, including what it cost
    alternatives  what was rejected, and why — one paragraph each
    rationale     stated in place of the last two by four records — `ontology`, `proposals`,
                  `report-card-connector` and `five-year-forecast-connector`
    amendment     a later revision of the record, where one was made
    connectors    the registry keys this record approves. Not prose; see below.

Every prose field is markdown in a YAML block scalar. Links are relative file paths so they work
in an editor: `../corpus/class/name.yml`, `../catalog/name.md`, and a bare `name.yml` for a
sibling decision. The site rewrites all three.

A record may carry fields outside this list — `ontology` has a `corpus_depth` integer and a
`governance` string — and the site ignores them rather than rendering them as unheaded cards,
because `readDecision` in [`web/src/lib/corpus.ts`](../../web/src/lib/corpus.ts) reads an
allowlist. That is worth knowing in both directions: a data field costs the page nothing, and a
*prose* field added here will not appear on it until it is added to `DECISION_SECTIONS` and to
`SECTION_NAMES` in `routes.ts`, which `web/tests/unit/links.spec.ts` holds in agreement.

## `connectors:`

A record that approves one or more connectors lists their registry keys:

```yaml
connectors:
  - dew-foundation
  - tax-abstract
```

Thirteen records carry it and they account for all 21 connectors, each named exactly once.
`registry::tests` in [`crates/connect`](../../crates/connect/src/registry/mod.rs) derives the
approval list from these fields and checks it against the registry in both directions, so a
connector cannot enter the registry without a record here and cannot leave it without its approval
being withdrawn in the record that made it.

It is a field and not something read out of the prose because a record cites the connectors it
reasons about far more often than it approves one — `tax-abstract` is named in seven records and
approved by one. [`what-approves-a-connector`](what-approves-a-connector.yml) records that, and
records the eleven phases during which `crates/connect/README.md` claimed this check existed while
the test compared the registry against a hand-written array.

## Correcting a record

**A decision record is never rewritten to be right.** The reasoning that led somewhere wrong is the
most useful thing in it, and editing the wrong turn out leaves a document that has always been
correct and teaches nothing. Corrections are added in place, against the sentence they withdraw.

A correction is **a blockquote opening with strong emphasis**:

```
> **CORRECTED by [`the-order-was-never-the-states`](the-order-was-never-the-states.yml).** Two
> things above are wrong.
```

Five openers are in use — `CORRECTED by`, `RESOLVED by`, `SUPERSEDED by`, `This rejection has
expired.`, and one whole sentence — and the site does not match on any of them. It matches on the
strong emphasis, because that is the one property all of them share and none of the ordinary
quotations in these records has. Blockquotes are used for both jobs throughout: most of them quote
a superseded docstring or a previous record's blocker, and those open with plain prose.

The site counts the corrections, lists them at the top of the record, and marks each one where it
stands. So **opening a correction with plain prose makes it a quotation**, and it will be rendered
as one. `web/tests/unit/prose.spec.ts` pins the split against every record here, including the
agreement between the two places the rule is implemented — the count is read off the markdown at
load time and the class is applied to the rendered HTML, and nothing but that test keeps them
saying the same thing.

## What does not belong here

A decision record is not a catalog entry and not a corpus node. It records a choice this
repository made. What a *source* is and what it can be trusted for goes in
[`../catalog/`](../catalog/); what a thing in Ohio's funding system *is* goes in
[`../corpus/`](../corpus/).
