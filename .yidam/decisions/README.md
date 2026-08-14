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
    rationale     used by the four connector records in place of the last two
    amendment     a later revision of the record, where one was made

Every field is markdown in a YAML block scalar. Links are relative file paths so they work in an
editor: `../corpus/class/name.yml`, `../catalog/name.md`, and a bare `name.yml` for a sibling
decision. The site rewrites all three.

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
