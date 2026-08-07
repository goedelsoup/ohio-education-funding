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
| [per-pupil-operating-expenditure](per-pupil-operating-expenditure.yml) | dollars per pupil | Adequacy and equity — the outcome side |

The third of these was named as this class's largest gap at genesis and is now populated with a
statewide series from FY2000 to FY2022. It carries the corpus's clearest demonstration of why
`deflate` is a prerequisite rather than a refinement: over that span per-pupil operating
expenditure rose **116.8% nominally and 26.1% in real terms** — and 19.4% if federal COVID
relief is excluded. Three correct numbers, three different arguments.

| [effective-operating-millage](effective-operating-millage.yml) | mills | Equity — the local effort side |

The dispersion statistics themselves are computed by
[`crates/dispersion`](../../../crates/dispersion/) and recorded on
[`doctrine/equity`](../doctrine/equity.yml) rather than as a node of their own, since they
describe a distribution rather than an agency.

## Known gaps

No metric covers the scholarship and community school diversion, so a district's net position
is still not measurable — the `deduction` calculator remains a stub. Per-district expenditure
series over time exist on the Auditor of State dashboard and are not extracted, so every metric
here is a single cross-section rather than a series. [open]
