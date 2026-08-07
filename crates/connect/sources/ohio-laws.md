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

**Declared.** Approved in [decisions/proposals.yml](../../../.yidam/decisions/proposals.yml);
no endpoint wired.

`codes.ohio.gov` serves HTML with no bulk export, and section history is rendered rather than
published as data. This is the connector whose absence is most visible in the corpus itself:
most `statutory_basis` fields on
[`parameter`](../../../.yidam/corpus/parameter/) and
[`formula-component`](../../../.yidam/corpus/formula-component/) nodes are still `[open]`, and
they are waiting on exactly this.
