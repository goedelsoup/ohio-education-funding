# funding-regime

The named methods by which Ohio has distributed state aid to education agencies, each holding
for a bounded span of fiscal periods before being replaced. Reading these five nodes in order
is the fastest way to understand the argument this corpus exists to support, because the
sequence has a shape: two regimes that set a per-pupil amount by dividing available money by
pupils, two that tried to derive the amount from what an education costs, and one in between
that did neither and was explicitly a placeholder.

A regime is a Phase, not a Kind — the funding system persists across the transitions while
being in a materially different condition in each. That is why `supersedes` chains rather
than branches: at any fiscal period exactly one regime governs distribution.

The recurring pattern worth naming: every regime that attempted to cost out an adequate
education was enacted with a multi-year phase-in, and the General Assembly that enacted it
could not bind its successors to complete one. The Evidence-Based Model was designed for ten
years and operated for two. Whether the Fair School Funding Plan escapes that pattern is the
live question in this class. [open]

See the class definition: [funding-regime.ont.yml](../funding-regime.ont.yml).

## Instances

| Node | Span | Method |
|------|------|--------|
| [equal-yield-formula](equal-yield-formula.yml) | FY1976–FY1991 | Guaranteed yield per mill of local effort |
| [foundation-base-cost-formula](foundation-base-cost-formula.yml) | FY1992–FY2009 | Statewide per-pupil base cost less a charge-off |
| [evidence-based-model](evidence-based-model.yml) | FY2010–FY2011 | Priced resource inputs summed per district |
| [bridge-formula](bridge-formula.yml) | FY2012–FY2021 | Prior-year amounts adjusted by caps and guarantees |
| [fair-school-funding-plan](fair-school-funding-plan.yml) | FY2022–present | District-specific base cost, local capacity share |
