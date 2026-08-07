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

Nothing before FY2022 is addressable. The Bridge formula decade and the DeRolph era have no
periods at all, so a query about FY2012 or FY1997 has nowhere to land. Backfilling biennia to at
least FY2010-11 remains the cheapest high-value expansion in the corpus. [open]
