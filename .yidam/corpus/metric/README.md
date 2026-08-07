# metric

The measures. Each node states a computation precisely enough that two people working from the
same sources get the same number, and names the caveats that make the number misleading if
quoted without them.

Metrics face the opposite direction from [`parameter/`](../parameter/): a parameter is an input
policy chooses, a metric is an output policy produces. Ohio arguments routinely conflate the
two — a base cost per pupil is a legislative decision, a spending per pupil is a consequence of
that decision plus local wealth plus levy history plus enrollment, and quoting them as
comparable quantities is how a defensible claim gets built on a category error.

Metrics are also how [`doctrine/`](../doctrine/) becomes testable. A doctrine with no metric
that operationalizes it is a rhetorical position; with one, it is a hypothesis about the data.
The `operationalizes` edge is the load-bearing one in this class.

See the class definition: [metric.ont.yml](../metric.ont.yml).

## Instances

| Node | Unit | Operationalizes |
|------|------|-----------------|
| [assessed-valuation-per-pupil](assessed-valuation-per-pupil.yml) | dollars per pupil | Equity — the wealth side |
| [state-share-percentage](state-share-percentage.yml) | percent | Equity — the compensation side |
| [per-pupil-operating-expenditure](per-pupil-operating-expenditure.yml) | dollars per pupil | Adequacy and equity — the resource side |

The third of these was named as this class's largest gap at genesis and is now populated with a
statewide series from FY2000 to FY2022. It carries the corpus's clearest demonstration of why
`deflate` is a prerequisite rather than a refinement: over that span per-pupil operating
expenditure rose **116.8% nominally and 26.1% in real terms** — and 19.4% if federal COVID
relief is excluded. Three correct numbers, three different arguments.

| [effective-operating-millage](effective-operating-millage.yml) | mills | Equity — the local effort side |
| [performance-index](performance-index.yml) | index | Adequacy — the outcome side, as level |
| [progress-value-added](progress-value-added.yml) | index | Adequacy — the outcome side, as gain |
| [expenditure-per-equivalent-pupil](expenditure-per-equivalent-pupil.yml) | dollars per pupil | Adequacy and equity — the report card's spending measure |

Two of those rows are new and the third row's label changed to make room for them. Per-pupil operating
expenditure had been carrying the phrase "the outcome side," which was accurate only because
nothing in this class measured an outcome. It is a resource. The
[Performance Index](performance-index.yml) is the outcome, and its arrival is the first time
this corpus can put a result next to an input.

**The two outcome measures disagree, and the disagreement is the finding.** Holding the
economically disadvantaged share constant, per-pupil spending correlates −0.125 with the
Performance Index and **+0.146** with the Progress effect size. The Index is 71.5% poverty and
almost static across three years; Progress is 10.6% poverty and centred on zero by construction.
Quoting either alone answers a different question than the reader will assume, and neither is
causal. The class rule that follows: **an outcome claim names whether its measure is a level or
a gain**, exactly as a spending claim names its denominator.

Read the level measure and spending together with care. Spending disperses across districts about 60% more widely than
performance does — coefficient of variation 0.202 against roughly 0.124 — so a near-zero
correlation between them is a weaker statement than it sounds, and the distributions should be
quoted before the coefficient is.

**Two spending metrics now sit in this class, and the difference between them is load-bearing.**
[per-pupil-operating-expenditure](per-pupil-operating-expenditure.yml) divides by a headcount.
[expenditure-per-equivalent-pupil](expenditure-per-equivalent-pupil.yml) divides by an ADM count
weighted upward for disadvantage, English learners, and disability, and runs 21% lower at the
median within a single file. The second is what DEW prints on the report card beside the Performance Index, which
makes it the measure any Ohio spending-versus-results argument will reach for — and a
need-weighted denominator is a need adjustment, so an analysis using it is not unadjusted no
matter how few covariates it has. The class rule this establishes: **name the denominator before
quoting the number.** Metrics that share a numerator and differ in their divisor are different
metrics, not different presentations of one.

The dispersion statistics themselves are computed by
[`crates/dispersion`](../../../crates/dispersion/) and recorded on
[`doctrine/equity`](../doctrine/equity.yml) rather than as a node of their own, since they
describe a distribution rather than an agency.

## Known gaps

No metric covers the scholarship and community school diversion, so a district's net position
is still not measurable — the `deduction` calculator remains a stub. Per-district expenditure
series over time exist on the Auditor of State dashboard and are not extracted, so every metric
here is a single cross-section rather than a series. [open]

That extraction has been done. The `dew-report-card` connector retrieves four 2024-25
publications and
[`crates/dispersion/fixtures/report-card-2425-district-data.csv`](../../../crates/dispersion/fixtures/report-card-2425-district-data.csv)
holds 607 districts with three years of Performance Index, the Progress composite and effect
size, both ADM columns, and the expenditure numerator;
[`crates/dispersion/tests/report_card_2425.rs`](../../../crates/dispersion/tests/report_card_2425.rs)
pins what it shows in 20 tests. Every figure on all three report-card metrics is computed rather
than quoted.

**The need-adjusted model exists.** `dispersion::least_squares` fits it and
[`crates/dispersion/tests/report_card_2425.rs`](../../../crates/dispersion/tests/report_card_2425.rs)
pins it across 27 tests: controlling for economic disadvantage, English-learner and disability
shares, district size, and property wealth, the standardised spending coefficient is −0.073 on
attainment level and +0.209 on growth. Missing from the specification: district typology,
regional cost, and what the money was spent on. The last of those is not a column anywhere in
this workspace and is the largest omission by far.

What is still missing is time, and less of it than before. Three years of Performance Index are
read: the measure is almost static (adjacent years +0.988, median within-district three-year
range 2.1 points), which settles that one year stands in for a district's Index and also means
the measure cannot detect anything annual. The published Progress figure turns out to be a
three-year average, and its one-, two- and three-year windows rank districts nearly alike
(+0.904 to +0.968), so it is more stable than a single-year gain would be. Whether growth is
stable across *separate* years remains unanswerable from the 2024-25 file alone. [open]

Per-district expenditure series over time exist on the Auditor of State dashboard and are not
extracted. The DEW pupil-weight schedule that produces the weighted ADM is not held, so the
corpus can measure what the weighting does without being able to say what it is. Nothing here
controls for English-learner share, disability share, district typology, or regional cost —
which is what would be needed before the +0.146 on growth means much. [open]
