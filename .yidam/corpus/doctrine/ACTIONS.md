# doctrine — actions

## Queries

- **Assessment.** Given a `funding-regime` and a doctrine, evaluate the regime against the
  doctrine's `formulation` using the metric that operationalizes it. Returns a claim with
  evidence, not a verdict.
- **Tension trace.** Follow `tensions-with` to surface what a proposed reform trades away.
  A change that improves one doctrine's metric while worsening another's is the normal case,
  not an anomaly.
- **Judicial reasoning.** Follow `invoked-by` to the cases that reasoned from a doctrine and
  compare how the same principle was applied to opposite results — *Walter* and *DeRolph I*
  on the same constitutional text.

## Transitions

- **Reformulation.** A decision or a body of scholarship restates the principle. Recorded as a
  revision to `formulation` with the source; the prior statement stays in git history.
- **Operationalization.** A metric is defined that makes the doctrine testable. Recorded as an
  `operationalized-by` edge, and the doctrine stops being purely rhetorical.

## Calculators

- `dispersion` — operationalizes equity across agencies.
- `foundation` and `deflate` — together supply the costed-requirement side of an adequacy test.

## Skills

- No dedicated skill yet. An `adequacy-assessment` skill that evaluates a regime-year against
  a doctrine's formulation is the natural first addition to this class. [open]
