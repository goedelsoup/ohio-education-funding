# School Finance Payment Report — Line by Line Explanation

**Source.** Ohio Department of Education (now Department of Education and Workforce), Office
of Budget and School Funding.
**Type.** Primary source — official methodology documentation, published per fiscal year.
**Location.** `education.ohio.gov`, Finance and Funding → School Payment Reports → State
Funding for Schools → Traditional School Districts. One PDF per fiscal year; the FY2022
edition is titled *FY 2022 School Finance Payment Report Line by Line Explanation, Based on
Provisions of Am. Sub. H.B. 110 and H.B. 583 of the 134th General Assembly*.

**What it contains.** The complete computational specification of the Fair School Funding
Plan, component by component, with the Revised Code section for each, the exact formula, the
statewide factor values in force that year, and worked screenshots from the actual payment
reports. This is the document that turns the formula from a description into something
reproducible.

The FY2022 edition supplied nearly every verified figure now in this corpus: the statewide
average base cost per pupil of $7,349.22, the FY2018 salary inputs the base cost is priced
from, the funded-teacher staffing ratios, the 60/20/20 weighting inside the local capacity
measure, the 5% minimum state share, and the phase-in percentages actually applied by line —
including that Disadvantaged Pupil Impact Aid was phased in at 0% in FY2022 while every other
phased component was at 16.67%.

**Access constraints.** Freely available. Published as PDF; the FY2022 file is ~1.1 MB and
does not extract cleanly through text-only converters — read it as a rendered document. The
publisher name changes across the series at the 2023 transition from ODE to DEW, and the
`publisher` field must be carried on every record drawn from it.

**Caveat.** Describes the formula *as enacted for that fiscal year*. Figures are not
comparable across editions without checking whether the cost-input reference year changed —
it moved from FY2018 to FY2022 between the FY2023 and FY2024 editions.

## Used by

- [`formula-component/fsfp-base-cost-calculation`](../corpus/formula-component/fsfp-base-cost-calculation.yml)
- [`formula-component/fsfp-local-capacity-measure`](../corpus/formula-component/fsfp-local-capacity-measure.yml)
- [`parameter/base-cost-per-pupil`](../corpus/parameter/base-cost-per-pupil.yml)
- [`parameter/fsfp-phase-in-percentage`](../corpus/parameter/fsfp-phase-in-percentage.yml)
- [`metric/state-share-percentage`](../corpus/metric/state-share-percentage.yml)
- [`legislation/hb-110-2021`](../corpus/legislation/hb-110-2021.yml)

## Feeds connector

[`dew-foundation`](../../crates/connect/sources/dew-foundation.md)
