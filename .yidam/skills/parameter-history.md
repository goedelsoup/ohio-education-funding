---
name: parameter-history
description: Assemble a parameter's value series across fiscal periods, naming the enactment behind each value and the kind of thing that sets it
---

# Procedure: parameter-history

**Purpose.** Produce the series for one [`parameter`](../corpus/parameter/) — value, fiscal
period, and the authority that set it — so that a counterfactual can bind to it and a reader can
tell what would have to happen for it to change.

**Composes.** the `ohio-laws` and `lsc-budget` fixtures; [`real-dollar`](real-dollar.md) for any
dollar-denominated parameter.

## Steps

1. **Establish the `kind` before the values.** Legislated, delegated, measured, or uncodified —
   because it decides where to look. A legislated value is in the section; a measured one is not
   anywhere, it is computed from reported data; an uncodified one is in a budget act and nowhere
   else.
2. **One row per change, with its authority.** Session law citation where the Revised Code does
   not carry the history. The charge-off millage's progression — 20 mills, 20.5, 23 — is recited
   with its session-law citations at *DeRolph I* ¶97 and nowhere more conveniently.
3. **Record the base as well as the rate.** A rate applied to a changed base is a changed
   parameter even when the number is identical.
4. **Mark gaps as gaps.** A missing year is `[open]` if nobody knows and `[unentered]` if the
   source is open and nobody has typed it in.

## Two failures worth expecting

**The base changes while the rate stands still.** The charge-off was 23 mills for two decades and
the valuation it applied to went from total taxable value to *recognized* valuation — a three-year
phase-in of reappraisal growth, not an H.B. 920 adjustment, which this corpus had wrong for the
whole life of the node and which named the wrong districts when it was wrong. A history of the
rate alone would have shown a flat line through a substantive change.

**A flat line can be an injunction.** The same rate stopped moving in 1997 not because nobody
revisited it but because [*DeRolph III*](../corpus/litigation/derolph-iii-2001.yml) ¶62 forbade
changing it through FY2007. Before writing "unchanged", check whether something was preventing the
change — the taxonomy asks what would have to happen for a value to move, and the answer is not
always an act.

## Refusals

- **Do not infer a value from a calculator's output.** A number in a spreadsheet cell is what the
  department computed, not necessarily what statute set; those coincide until they do not, and the
  difference is the whole point of the `kind` field.
- **Do not write "not in law" from "not in the Revised Code."** Read the enacted act. This corpus
  has made that exact error twice and both times the provision was on the face of a budget bill.
