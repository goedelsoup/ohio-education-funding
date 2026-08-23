---
name: millage
description: Compute effective operating millage under HB 920 reduction factors and determine 20-mill floor status for an agency and tax year
---

# Calculator: millage (stub)

**Computes.** Effective operating millage for an agency in a tax year after
[H.B. 920](../corpus/legislation/hb-920-1976.yml) reduction factors, and whether the agency is
at the [20-mill floor](../corpus/parameter/twenty-mill-floor.yml) — 2 mills for a joint
vocational school district.

**Reads.** [`revenue-stream/local-property-tax`](../corpus/revenue-stream/local-property-tax.yml),
the floor parameter, per-agency voted millage and valuation series from `crates/`.

**Returns.** Voted millage, effective millage, reduction factor, floor status, and the local
yield implied by each.

## Why it is load-bearing

Floor status is a regime switch, not a continuous variable. Above the floor, valuation growth
does not reach revenue; at the floor, it passes through directly. The same policy change moves
the two groups in opposite directions, so any statewide claim about a property tax change that
has not partitioned agencies by floor status is unreliable — frequently it is not merely
imprecise but wrong in sign for a large share of districts.

This calculator is also a precondition for
[`charge-off-local-share`](../corpus/formula-component/charge-off-local-share.yml): the phantom
revenue gap is the difference between the millage the charge-off assumes and the effective
millage this calculator returns.

## Status

**Implemented** in [`crates/millage/`](../../crates/millage/src/lib.rs). 19 tests.

The tests encode the mechanism rather than any particular district's numbers, because no
per-district valuation series exists yet: doubling valuation halves effective millage and holds
the dollar yield exactly flat; yields stay flat across four successive reappraisals; the floor
binds at 20 mills for districts and 2 for JVSDs; and — the one that matters for policy — the
same reappraisal raises revenue for a floored district while leaving an unfloored one
unchanged.

Two guards worth naming, both of which would otherwise produce plausible nonsense: a district
that voted 12 mills is never pushed up to 20 by the floor, and falling valuation never raises
an effective rate above what voters approved.

**Blocked on `tax-abstract`** for real inputs. The logic is ready; the data is not.
