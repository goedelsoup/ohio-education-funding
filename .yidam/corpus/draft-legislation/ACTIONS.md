# draft-legislation — actions

## Queries

- **What would this draft cost?** The combined run over every runnable provision, reported with
  the count of provisions it does not price. Never the priced subset alone.
- **Which provision does the work?** One run per provision against the same baseline, plus the
  combined run, and the residual between their sum and the combined figure. The residual is a
  finding rather than an error: the levers interact through the guarantee.
- **Who does it reach?** The `scenario-delta` table for the combined run — both orderings, with
  incidence across wealth and state share, and the off-formula count.
- **What can this repository not say about it?** Read `unpriced`. Each entry names what would
  have to exist before the provision could be run, which is the difference between a question
  somebody can work and one nobody has scoped.
- **What is the enacted counterpart?** Follow `redrafts` for a counterfactual, `amends` for a
  bill that would change a statute in force, and `enacted-as` where the draft passed.
- **What has this node been wrong about?** Read `revisions:`. Each entry carries the claim as it
  stood, what replaced it, and the test or source that settled it — so a withdrawn figure can be
  recognised rather than re-derived, and `found_by` gives the check to re-run.

## Transitions

- **Specification → priced.** A draft gains `simulated-by` edges and a cost in `findings`. An
  epistemic commit: it must record which fixture year and which calculator version produced the
  figure, because a draft re-run against updated inputs is a different claim.
- **Enactment.** The bill passes. A **new** node is written in
  [`legislation/`](../legislation/) and this one gains an `enacted-as` edge to it. This node is
  not edited and does not move class — what was introduced and what was enacted differ, and both
  are worth reading.
- **Amendment.** The bill's text changes in committee or on the floor. `text_read` gains the new
  version and its digest; the prior reading is kept, on the same ground `session_laws::OPERATIVE`
  keeps H.B. 215's superseded columns beside H.B. 770's.
- **Death.** The session ends without passage. `status` records it. The node stays: a proposal
  the General Assembly declined is evidence about the General Assembly.
- **Invalidation.** The baseline regime changes or an input series is corrected. The linked
  scenarios are superseded by the rule [`scenario/ACTIONS.md`](../scenario/ACTIONS.md) already
  states; the draft's provisions are unchanged, because what a bill says does not depend on what
  it would now cost.

## Skills

- `draft-authoring` — writes a draft, binds each provision to a parameter, and refuses the
  bindings that are category errors.
- `scenario-run` — perturbs, invokes, and commits each run this draft's provisions imply.
- `parameter-history` — establishes what kind of thing sets a parameter before a provision claims
  to change it.

## Calculators

- `project` — the five levers, and the only runnable surface a provision can bind to.
- `scenario-delta` — the reach and incidence of the combined run.
- `foundation` — where a provision changes the base cost build-up rather than scaling it.
- `deflator` — required whenever a draft's horizon spans more than one fiscal period.

## Connectors

- `ohio-bills` — the bill itself, as introduced. The only connector that declares
  `feeds: draft-legislation`.
- `ohio-laws` — the Revised Code section a provision would amend, as it currently stands.
- `lsc-budget` — the analysis and the appropriation levels that decide whether an enacted
  formula would actually be funded.

## Gaps

- **A pending bill's text is retrievable now, and deliberately not parsed.**
  [`ohio-bills`](../../catalog/ohio-bills.md) was added on the precedent
  [`the-acts-themselves`](../../decisions/the-acts-themselves.yml) set when it declined to fold the
  session laws into `ohio-laws`: same publisher, third artefact. The as-introduced version code is
  always `00_IN`, because introduction is always a bill's first version — simpler than the
  enrolled case, where the code is positional and a guess returns a nine-byte 404.

  It stays at `retrievable`. Turning a bill into provisions is judgement, not extraction, and a
  parser over section headings would emit something that looked authoritative and was not.

- **The listing endpoint reports a bill's first version, not its current one.** So it cannot tell a
  pending bill from an enacted one: H.B. 186 of the 136th appears there as `As Introduced` and was
  enrolled effective 20 March 2026. Only the per-bill version index distinguishes them. And no
  endpoint distinguishes a live bill from one that quietly stopped — the General Assembly
  publishes no marker for a bill that was never reported out. [open]

- **The levers do not span what bills do, and the gap is now measured rather than asserted.**
  Five levers — the guarantee rule, aggregate base cost, the minimum state share, and the two
  phase-in dials — against a budget act that also moves special-education weights, transportation
  reimbursement, the scholarship deduction and the capital channel.

  On the one multi-provision draft here, **two of five provisions price**, and the three that do
  not fail for three different reasons: one is unlevered, and two are outside the model entirely.
  That distinction is the reason `unpriced` is prose rather than a count — "add a lever" and "find
  a source that does not exist" are not the same amount of work, and a ratio would report them as
  though they were.

  One constructed bill is not a sample, and the one introduced bill here prices at 0 of 1. What the
  ratio looks like for a budget act — where the funding provisions are a few dozen sections out of
  several thousand pages — is still open. [open]

- **The minimum state share is a lever with no parameter node.** `Policy::minimum_state_share` is
  real, the feed carries it, and 138 districts sit exactly on it — but there is no
  [`parameter`](../parameter/) node for it, so a provision moving it cannot bind one and the
  `a_priced_provision_binds_a_parameter_node_that_exists` test would reject it. Found by trying to
  write such a provision. [open]

  Adding a lever is not free and the price should be known before one is promised: every lever
  needs a matching entry in `bundle`'s `checkpoint_policies` and a mirrored TypeScript
  implementation, or `web/src/lib/verify.ts` refuses to render the scenario page at all. That
  refusal is the feature — it is what keeps the browser's formula honest — and it means a lever
  costs Rust, a checkpoint, and TypeScript, not just a field on `Policy`.
