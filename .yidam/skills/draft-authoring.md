---
name: draft-authoring
description: Write a bill that is not law as a draft-legislation node, bind each provision to a parameter the calculators can run, and refuse the bindings that are category errors
---

# Procedure: draft-authoring

**Purpose.** Turn a proposal — introduced, counterfactual, or invented — into a
[`draft-legislation`](../corpus/draft-legislation/) node whose runnable provisions can be priced
and whose unrunnable ones are recorded rather than dropped.

**Composes.** [`parameter-history`](parameter-history.md) before any binding,
[`scenario-run`](scenario-run.md) for each run the provisions imply,
[`scenario-delta`](scenario-delta.md) for reach and incidence,
[`provenance-trace`](provenance-trace.md) where the draft has an external text.

## Steps

1. **Establish the provenance first**, because it decides what every other field means.
   `introduced` — a real bill, whose text exists and moves. `counterfactual` — an alternative
   version of an act that passed. `hypothetical` — nobody proposed this. A hypothetical draft
   takes no bill designation, not even a vacant one.

2. **Pin the text, or state that there is none.** For an introduced bill this is the step most
   likely to be skipped and the one most likely to go wrong later: the text changes as the bill
   is amended, at the same URL. Record the version and its digest. For a counterfactual there is
   no draft text — pin the enactment being rewritten instead, and say which stage of it, because
   figures circulating for a budget act belong variously to the executive proposal, a
   chamber-passed version, and the act.

3. **Enumerate the provisions before binding any of them.** One entry per change the draft would
   make, each naming the Revised Code section it would amend or `uncodified`. Enumerating first
   is what keeps the unrunnable ones on the list: a pass that binds as it reads will quietly
   produce a draft consisting of exactly the provisions the levers happen to reach.

4. **Bind each provision to a `simulation_key`.** A provision that cannot name the parameter it
   moves cannot be run, which is a finding about the provision and not a reason to omit it.

5. **Check the parameter's `kind` before accepting the binding.** This is the refusal that earns
   the step. Ohio's funding parameters are four different kinds of thing —
   [legislated, delegated, measured, uncodified](../decisions/the-four-kinds-of-parameter.yml) —
   and a spreadsheet renders all four as a number in a cell.

   - **legislated** — bind directly. An act changes it by saying so.
   - **uncodified** — bind directly, and record that it lapses if not re-enacted. Most of what a
     budget act actually moves is here.
   - **delegated** — the act can constrain the department's discretion but does not set the
     value. A provision claiming to set it is describing a different bill from the one written.
   - **measured** — **refuse the binding.** Nobody sets a measured value; statute specifies a
     computation over reported data and the number moves when districts' behaviour moves. What a
     policy can change is the *measurement* — which reference year is read, which population is
     counted — one level up. Re-bind there. The cost input refresh is exactly this shape, and it
     is why the counterfactual in this class perturbs the reference year rather than a salary.

6. **Run the combined policy, then each provision alone.** The combined run is the draft's cost.
   The per-provision runs attribute it. Report the residual between their sum and the combined
   figure; do not absorb it.

7. **Fill `unpriced` before writing any total**, including when the answer is none. Each entry
   names the provision, why it does not run, and what would have to exist before it could —
   which is the difference between a question somebody can work and one nobody has scoped.

8. **Commit the node with the `establish:` verb** — a node authored is new understanding —
   carrying its provisions, its `simulated-by` edges, and the enacted counterpart it redrafts
   or amends. Naming the verb at the step is the whole mechanism: `yidam lint --commits` is
   Warn severity because history cannot be rewritten, so it reports drift only once the drift
   is permanent. See [`GRAPH.md`](../.vendor/prelude/GRAPH.md).

## The three refusals

**A cost may not be stated without the count of provisions it fails to price.** The transposition
of [`scenario-delta`](scenario-delta.md)'s rule that a total cannot be constructed without the
count of districts it fails to reach, one level up. A draft reporting the priced subset as the
bill's cost is the failure this whole class exists to prevent, and it is the easy failure: the
priced subset is the part that produces a number.

**Provisions do not sum.** They interact through the guarantee, and the interaction is large and
measured. Raising base cost costs the state $47.7M per point at +2% and $72.5M per point at +20%,
because each increment lifts more districts off the guarantee onto a formula that then has to pay
them — see [guarantee-phase-out](../corpus/scenario/guarantee-phase-out.yml). Two provisions
priced separately and added will understate a bill that raises base cost and touches the
guarantee, and the error grows with the size of the change.

**A draft that passes does not become an enactment.** Write a new
[`legislation`](../corpus/legislation/) node and add an `enacted-as` edge. What was introduced
and what was enacted differ — often in the provisions that mattered — and a class that overwrote
one with the other would destroy the only record of the difference.

## What bounds this more tightly than it bounds a scenario

A scenario is authored around a perturbation the levers can express, because whoever writes it
chooses the question. A draft is authored around provisions somebody else chose. So the runnable
surface — the five levers in
[`crates/project/src/policy.rs`](../../crates/project/src/policy.rs) — is a real constraint here
rather than a framing convenience, and the share of a real bill that can be priced is an
empirical question. Do not answer it from one draft.

**Adding a lever costs three things, not one.** A new field on `Policy` needs a matching entry in
`bundle`'s `checkpoint_policies` and a mirrored implementation in `web/src/lib/policy.ts`, or
`verify.ts` refuses to render the scenario page at all. That refusal is the mechanism keeping the
browser's second implementation of the formula honest, so it is not something to work around.
Price the lever before promising it to a draft.

## On the channels outside the model

A draft says what the formula would pay. It does not say what a district would receive: the
scholarship and community-school deduction is not modelled, and the capital channel is invisible
in every operating per-pupil figure here. A bill whose provisions land in either channel is not
partially priced — it is unpriced, and the node should say so in those words rather than
reporting a formula-side figure that reads like an answer.
