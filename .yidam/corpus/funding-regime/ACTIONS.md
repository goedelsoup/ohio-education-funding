# funding-regime — actions

## Queries

- **Which regime governed?** Given an agency and a fiscal period, resolve the regime in force
  and the components that computed its payment.
- **Regime succession.** Walk `supersedes` to reconstruct the full sequence and the enactment
  behind each transition.
- **Component-level comparison.** For one agency-year, run `regime-diff` between two regimes
  to isolate which component accounts for a change, rather than reporting a single net delta.
- **Was it ever funded as designed?** Compare the computed formula amount against the
  appropriated amount for each fiscal period the regime was in force. For three of the five
  regimes here the answer is no, and the gap is the number that matters.

## Transitions

- **Supersession.** A new regime takes effect. The prior node's `effective_to` closes and
  `status` becomes `superseded`; the new node gains a `supersedes` edge. Neither node's
  content is rewritten.
- **Phase-in advance.** The regime is unchanged but its phase-in parameter moves. Recorded on
  the `parameter`, not here — this separation is deliberate.

## Calculators

- `foundation` — re-runs a named regime for a fiscal period against a parameter set.
- `regime-diff` — component-level difference between two regimes for one agency-year.
- `dispersion` — equity statistics across agencies under a regime, for
  `assessed-against` claims.

## Skills

- `formula-walk` — decomposes one agency-year figure into the components that produced it.
