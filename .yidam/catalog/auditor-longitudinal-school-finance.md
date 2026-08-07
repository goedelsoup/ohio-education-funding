# Longitudinal School Finance Study — Ohio Auditor of State

**Source.** Ohio Auditor of State (Keith Faber), *Longitudinal School Finance Study: A Special
Report*, November 2024. Accompanied by a per-district dashboard on the Auditor's website.
**Type.** Secondary compilation of primary federal data — the Auditor's analysis, built on
NCES series.
**Location.** `ohioauditor.gov/performance/LSFS_2024/Longitudinal_School_Finance_Study_Special_Report.pdf`;
dashboard linked from the same site.

**What it contains.** Ohio public school operating expenditures and expenditures per pupil for
every fiscal year from FY2000 through FY2022, in nominal and inflation-adjusted terms, split by
function (instruction, support, other) and by object (salaries, benefits, purchased services,
supplies, other). The dashboard carries the same breakdown for each individual district.

This is the source that closed the corpus's largest named metric gap. Per-pupil operating
expenditure is the outcome side of every adequacy and equity claim, and until this entry the
corpus could define the measure but not populate it.

**Why the Auditor used NCES rather than Ohio data.** Stated explicitly in the report: NCES
collected school financial data in a standardized manner across the whole period, and *"a
comparable state source of public school financial data was not available across the desired
time period."* [verified] That is a finding about Ohio's own record-keeping, not merely a
methodological footnote — the state's auditor could not assemble a twenty-two-year comparable
series from state sources.

**Access constraints.** Freely available. The PDF does not extract through text-only
converters; read it as a rendered document or run `pdftotext -layout`.

**Caveats.** Three, each capable of producing a wrong comparison:

- **Scope is wider than "school districts."** Figures include city, local, exempted village,
  and joint vocational districts, educational service centers, community schools, STEM schools,
  and state- and federal-run schools, plus tuition payments those entities make to private and
  out-of-state schools. This is not the same population as the 611 districts receiving
  foundation funding.
- **Operating only.** Capital outlay, buses, and non-elementary-secondary programs (adult
  education, community services) are excluded. The capital channel is invisible here, as it is
  in every operating per-pupil figure.
- **COVID relief distorts the endpoint.** FY2022 operating EPP is $15,314 including federal
  relief funds and $14,493 excluding them — a difference of $821 per pupil, larger than most
  policy changes this corpus models. Any series ending in FY2020-FY2024 must state which
  version it uses.

Inflation adjustment uses national CPI for all items, June of each year, aligned to the fiscal
year end. CPI grew 71.9% from June 2000 to June 2022. [verified]

## Used by

- [`metric/per-pupil-operating-expenditure`](../corpus/metric/per-pupil-operating-expenditure.yml)
- [`doctrine/adequacy`](../corpus/doctrine/adequacy.yml)

## Feeds connectors

[`census-f33`](../../crates/connect/sources/census-f33.md) and
[`nces-ccd`](../../crates/connect/sources/nces-ccd.md) — the underlying series — and
[`bls-cpi`](../../crates/connect/sources/bls-cpi.md) for the deflator.
