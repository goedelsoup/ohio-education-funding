# metric — actions

## Queries

- **Compute.** Given a metric, an agency set, and a fiscal period, return the value with its
  input provenance. A metric value without the source series it was computed from is not
  usable in this corpus.
- **Distribution.** Compute across all agencies for one period and return the full
  distribution, not the mean. In a domain about disparity, a statewide average is the least
  informative summary available.
- **Wealth neutrality.** Regress a resource metric against
  [assessed valuation per pupil](assessed-valuation-per-pupil.yml). The strength of that
  association is the empirical content of the equity claim.
- **Real terms.** Any dollar-denominated metric compared across periods must pass through
  `deflator` first.
- **What has this node been wrong about?** Read `revisions:`. Each entry carries the claim as
  it stood, what replaced it, and the test or source that settled it — so a withdrawn figure can be
  recognised rather than re-derived, and `found_by` gives the check to re-run.

## Transitions

- **Definition revision.** The computation changes. This breaks the series, and the break must
  be recorded in `caveats` rather than smoothed — a redefined metric with a continuous-looking
  series is worse than a gap.
- **Implementation.** The metric gains a `calculator` binding and becomes computable rather
  than merely defined.

## Calculators

- `dispersion`, `deflator`, `local-capacity`, `millage`.

## Connectors

- `dew-foundation`, `tax-abstract`, `census-f33`, `bls-cpi`.
