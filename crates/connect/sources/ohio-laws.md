# ohio-laws — connector

**Source.** Ohio General Assembly and Legislative Service Commission bill archive: enrolled
bill text, session laws, effective dates, veto messages, and the Ohio Revised Code sections
each act amends.

**Feeds.** [`legislation`](../../../.yidam/corpus/legislation/), and through it the
`statutory_basis` fields on [`parameter`](../../../.yidam/corpus/parameter/) and
[`formula-component`](../../../.yidam/corpus/formula-component/) — most of which are currently
`[open]` and waiting on exactly this.

## Retrieval interface

```
fetch_bill(general_assembly, designation) -> BillDocument
fetch_sections(bill, code_chapter?)       -> Vec<CodeSection>   // sections amended
fetch_vetoes(bill)                        -> Vec<VetoItem>
code_history(section)                     -> Vec<AmendmentRecord>
```

`code_history` is the important one. Ohio's funding formula lives in R.C. 3317 and is rewritten
in whole or part by nearly every budget act, so reconstructing a parameter's series means
walking the amendment history of a section rather than reading any single bill.

## Constraints

- Veto messages must be retrieved separately from enrolled text. An enrolled bill is not what
  became law if a provision was vetoed, and education provisions have been vetoed repeatedly.
- Offline mode required.

## Status

**Wired**, for the current text of fourteen named sections. See
[`catalog/ohio-revised-code`](../../../.yidam/catalog/ohio-revised-code.md) and
[`decisions/reading-the-statute`](../../../.yidam/decisions/reading-the-statute.yml).

The recorded blocker was: "`codes.ohio.gov` serves HTML with no bulk export, and section history
is rendered rather than published as data." **Both clauses are true and the first was read as
though it meant the text was unreachable.** It is server-rendered — `curl` returns the operative
text in the response body — and `connect::html` turns it into prose. The absence of a bulk export
means the sections are named in the registry rather than crawled, which is a better fit anyway:
the list is exactly what some node's `statutory_basis` points at.

The second clause still stands, and it is the reason the interface above is only partly built.
`fetch_sections` and `code_history` need the version archive, which the site renders per version
and does not publish as data; the archive also begins at 1 July 2014, so it could not carry the
charge-off era even if it were machine-readable. What is wired is the current text and its
effective date, which is what a `statutory_basis` field needs and is not what a parameter *series*
needs.

`fetch_bill` and `fetch_vetoes` remain unbuilt. Veto messages in particular are still a real gap:
an enrolled bill is not what became law where a provision was vetoed, and education provisions
have been vetoed repeatedly.
