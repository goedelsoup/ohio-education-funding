# revenue-stream — actions

## Queries

- **Revenue composition.** For an agency and fiscal period, decompose total revenue by stream
  and level. The share from local sources is the headline equity number in this domain.
- **Substitution chain.** Follow `substitutes-for` to explain a discontinuity — a stream that
  drops to zero and another that appears is a policy event, not a data error.
- **Real growth.** Apply `deflate` to a stream's series. A local levy stream that is nominally
  flat under H.B. 920 is declining, and only the constant-dollar series shows it.
- **Restriction test.** Determine what share of an agency's revenue is unrestricted and
  therefore available for the base operating cost the formula computes.
- **What has this node been wrong about?** Read `revisions:`. Each entry carries the claim as
  it stood, what replaced it, and the test or source that settled it — so a withdrawn figure can be
  recognised rather than re-derived, and `found_by` gives the check to re-run.

## Transitions

- **Substitution.** A stream is abolished and another created to replace it. Recorded as a
  `substitutes-for` edge on the successor; neither series is edited.
- **Phase-down.** A replacement stream's schedule is revised. Recorded in `growth_behavior`
  with the enactment.

## Calculators

- `millage` — effective millage and 20-mill floor status, which determine local yield.
- `deflate` — constant-dollar normalization.
- `dispersion` — cross-agency distribution of local versus state share.

## Connectors

- `tax-abstract` — valuation, effective millage, reduction factors.
- `dew-foundation` — state aid by agency and payment period.
- `census-f33` — revenue by source, comparable across states.
