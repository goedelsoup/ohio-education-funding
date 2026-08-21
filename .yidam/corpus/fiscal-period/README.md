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
| [fy2002-03](fy2002-03.yml) | biennium | Am. Sub. H.B. 94 (2001) |
| [fy2004-05](fy2004-05.yml) | biennium | Am. Sub. H.B. 95 (2003) |
| [fy2006-07](fy2006-07.yml) | biennium | Am. Sub. H.B. 66 (2005) |
| [fy2008-09](fy2008-09.yml) | biennium | Am. Sub. H.B. 119 (2007) |
| [fy2010-11](fy2010-11.yml) | biennium | Am. Sub. H.B. 1 (2009) |
| [fy2012-13](fy2012-13.yml) | biennium | Am. Sub. H.B. 153 (2011) |
| [fy2014-15](fy2014-15.yml) | biennium | Am. Sub. H.B. 59 (2013) |
| [fy2016-17](fy2016-17.yml) | biennium | Am. Sub. H.B. 64 (2015) |
| [fy2018-19](fy2018-19.yml) | biennium | Am. Sub. H.B. 49 (2017) |
| [fy2020-21](fy2020-21.yml) | biennium | Am. Sub. H.B. 166 (2019) |
| [fy2022-23](fy2022-23.yml) | biennium | Am. Sub. H.B. 110 (2021) |
| [fy2022](fy2022.yml) | fiscal year | Am. Sub. H.B. 110 (2021) |
| [fy2024-25](fy2024-25.yml) | biennium | Am. Sub. H.B. 33 (2023) |
| [fy2026-27](fy2026-27.yml) | biennium | Am. Sub. H.B. 96 (2025) |
| [fy2026](fy2026.yml) | fiscal year | Am. Sub. H.B. 96 (2025) |
| [fy2027](fy2027.yml) | fiscal year | Am. Sub. H.B. 96 (2025) |

**All thirteen biennia chain by `follows`**, unbroken from FY2002-03 to FY2026-27 — a quarter
century in one traversal, four funding regimes deep. The three Fair School Funding
Plan biennia carry the plan's central tension traversably rather than narrated: FY2022-23 ran on
FY2018 cost inputs, FY2024-25 refreshed them to FY2022 and moved statewide average base cost about
12%, and FY2026-27 completes the phase-in while holding the inputs where H.B. 33 left them.

[FY2022](fy2022.yml) is the best-documented period in the corpus — nearly every verified figure
here is anchored to it — and [FY2027](fy2027.yml) is entirely prospective, with no actuals at
all. Between them they mark the range within which any claim has to be located.

## Known gap

**The Bridge formula decade is now addressable.** FY2012-13 through FY2020-21 exist as biennia,
each carrying the revenue shares and quartile equalization measure the F-33 panel supplies for its
years. That was called the cheapest high-value expansion in the corpus and it was, though the
value only appeared once there was a decade of per-district finance for the periods to hold.

They are deliberately thin. Each carries what is measured and the regime in force rather than a
narrative — but the sentence that stood here, saying "only FY2012-13's act is held", stopped being
true and nobody told this file. **All five acts are held now**: H.B. 153, H.B. 59, H.B. 64,
H.B. 49 and H.B. 166 each have a node, and each period names its act by an `appropriates-for` edge
in one direction and `funded-by` in the other. What is still `lsc-budget` work is the department's
own per-district figures for these years. [open]

That drift is worth stating rather than editing away. This class exists to make the join
traversable, and for five biennia the join was written as a string in `appropriating_bill:` with
no edge under it — so a traversal from an act to the years it paid for returned nothing, while the
prose two sections down said the acts were missing. The relationship
[`legislation.ont.yml`](../legislation.ont.yml) declares for exactly this had one user in the
whole corpus.

**FY2014 is absent from the F-33 archive**, so FY2014-15 is half-measured and nothing
interpolates the missing year. [verified]

## The decade before the panel, and what it is measured with instead

**FY2002-03 through FY2010-11 exist now**, and the sentence that stood here — *"Nothing before
FY2012… the obstacle is no longer the corpus's shape but its sources"* — conflated two things. It
was true that **NCES publishes no F-33 before FY2012**, so these five biennia cannot carry the
revenue shares and quartile equalization measure their successors do. It did not follow that they
could not be measured, and the argument was doing that work: no F-33 was read as no evidence, and
two committed series covering exactly this decade went unused.

They carry a different pair, and both reach further back than the panel does:

- the **enacted appropriation**, from `appropriation-lines.csv`, which runs FY1999 to FY2027 —
  the department's total, the foundation lines inside it, and its share; and
- the **free and reduced-price meal share**, from `mr81-sponsor-panel.csv`, which runs FY1998 to
  FY2014 and is the only per-district measure of need the corpus holds for these years.

That is a statewide grain rather than a per-district one, and it is stated as such on each node.
What it is enough for is the shape of the decade: real foundation aid peaks in FY2009, falls 14.1%
by FY2011, and ends below where FY2002 started, while the meal share rises 11.4 points on one
unbroken basis.

**FY2006-07 is read from the wrong act's greenbook**, and there is no alternative: H.B. 66's own
education greenbook prints no line-item table, the single gap in the `lsc-budget` series. Those two
years are actuals taken from H.B. 119's greenbook two years later, where every other year on these
nodes is enacted. [verified]

## Known gap

**Nothing before FY2002.** The equal yield formula, the whole *DeRolph* trial record, and
everything back to 1851 still have no periods, so a query about FY1997 has nowhere to land. The
appropriation series reaches FY1999 and the meal panel FY1998, so two of the three biennia between
1997 and 2002 are already measurable; what is missing is their acts. H.B. 215 of the 122nd and
H.B. 282 of the 123rd are both retrieved and pinned, and writing them is the reading step
`ohio-session-laws` does not automate. Below the 122nd the legislature's own version index stops
and no publisher serves anything. [open]
