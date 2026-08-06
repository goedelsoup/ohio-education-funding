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

## Known gaps

The metric this corpus most needs and does not have is **per-pupil operating expenditure in
constant dollars**, which is the outcome side of every equity and adequacy claim. Also missing:
the dispersion statistics (coefficient of variation, McLoone and Verstegen indices, federal
range ratio) that the `dispersion` calculator is specified to produce, and effective operating
millage, which determines 20-mill floor status. [open]
