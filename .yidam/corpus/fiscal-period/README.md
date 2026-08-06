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
| [fy2026-27](fy2026-27.yml) | biennium | Am. Sub. H.B. 96 (2025) |
| [fy2026](fy2026.yml) | fiscal year | Am. Sub. H.B. 96 (2025) |

## Known gap

Only the current biennium is seeded, so the `follows` edge has no target and no historical
period is addressable yet. Backfilling fiscal years to at least FY2022, and biennia to at
least FY2010-11, is required before any cross-period query works. This is the cheapest
high-value expansion in the corpus — the nodes are small and mechanical. [open]
