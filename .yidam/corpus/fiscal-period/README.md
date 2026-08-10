# fiscal-period

The spans of state fiscal time that funding decisions attach to. An Ohio fiscal year runs
July 1 through June 30 and is named for the calendar year it ends in, so FY2026 begins
July 1, 2025. Budget acts appropriate for a biennium — two fiscal years — which is why both
grains exist here and why `part-of` matters.

This class is a join, not a topic. It exists so that "what happened in the FY2026-27 biennium"
is a traversal rather than a scan of date strings across twelve other classes. Legislation
appropriates for a period, parameters hold values in one, regimes govern during one, and
metrics are computed for one — five incoming edge types converge here.

The distinction between fiscal year and biennium is not cosmetic. Formula parameters are set
per fiscal year and can differ between the two years of a single biennium, while the
appropriation and the political decision are made at the biennium level. Collapsing the two
loses the phase-in, which advances annually inside a biennially-negotiated act.

See the class definition: [fiscal-period.ont.yml](../fiscal-period.ont.yml).

## Instances

| Node | Kind | Appropriated by |
|------|------|-----------------|
| [fy2022-23](fy2022-23.yml) | biennium | Am. Sub. H.B. 110 (2021) |
| [fy2022](fy2022.yml) | fiscal year | Am. Sub. H.B. 110 (2021) |
| [fy2024-25](fy2024-25.yml) | biennium | Am. Sub. H.B. 33 (2023) |
| [fy2026-27](fy2026-27.yml) | biennium | Am. Sub. H.B. 96 (2025) |
| [fy2026](fy2026.yml) | fiscal year | Am. Sub. H.B. 96 (2025) |
| [fy2027](fy2027.yml) | fiscal year | Am. Sub. H.B. 96 (2025) |

The three biennia now chain by `follows` and carry the whole Fair School Funding Plan era, which
makes the plan's central tension traversable rather than narrated: FY2022-23 ran on FY2018 cost
inputs, FY2024-25 refreshed them to FY2022 and moved statewide average base cost about 12%, and
FY2026-27 completes the phase-in while holding the inputs where H.B. 33 left them.

[FY2022](fy2022.yml) is the best-documented period in the corpus — nearly every verified figure
here is anchored to it — and [FY2027](fy2027.yml) is entirely prospective, with no actuals at
all. Between them they mark the range within which any claim has to be located.

## Known gap

**The Bridge formula decade is now addressable.** FY2012-13 through FY2020-21 exist as biennia,
each carrying the revenue shares and quartile equalization measure the F-33 panel supplies for its
years. That was called the cheapest high-value expansion in the corpus and it was, though the
value only appeared once there was a decade of per-district finance for the periods to hold.

They are deliberately thin. Each carries what is measured and the regime in force, and not a
narrative — the appropriating acts, the enacted changes and the department's own per-district
figures are `lsc-budget` work, and only FY2012-13's act is held. [open]

**FY2014 is absent from the F-33 archive**, so FY2014-15 is half-measured and nothing
interpolates the missing year. [verified]

## Known gap

**Nothing before FY2012.** The DeRolph era, the Evidence-Based Model biennium and everything back
to 1851 still have no periods, so a query about FY1997 has nowhere to land. The obstacle is no
longer the corpus's shape but its sources: NCES publishes no F-33 before FY2012 under any naming
its later years use, and the pre-2000 record is the same `lsc-budget` blocker. [open]
