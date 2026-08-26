# revenue-stream

The channels through which money reaches an Ohio education agency. Each node binds a source
of funds to a recipient under a specific legal authority, and carries the properties that
matter about the binding: what constrains its growth, whether its use is restricted, and
whether it is guaranteed or re-appropriated every biennium.

Streams are modeled as Relators rather than as bare edges because those properties are the
substance. The difference between a locally-levied tax and a state replacement payment of
identical size is not the amount — it is that one is controlled by the district's voters and
the other by the next General Assembly. Ohio has converted the first kind into the second
repeatedly, and [`substitutes-for`](tpp-replacement-payments.yml) is the edge that records it.

See the class definition: [revenue-stream.ont.yml](../revenue-stream.ont.yml).

## Instances

| Node | Level | Constraint |
|------|-------|-----------|
| [local-property-tax](local-property-tax.yml) | local | Frozen against inflation by H.B. 920 reduction factors |
| [state-foundation-aid](state-foundation-aid.yml) | state | Re-appropriated each biennium |
| [tpp-replacement-payments](tpp-replacement-payments.yml) | state | Phase-down schedule revised repeatedly |
| [casino-tax-distribution](casino-tax-distribution.yml) | state | Constitutionally earmarked; never touches the formula or the department's budget |
| [lottery-profits](lottery-profits.yml) | state | Constitutionally earmarked, and appropriated *into* foundation aid rather than onto it |
| [title-i](title-i.yml) | federal | Appropriated annually; allocated on a poverty count Ohio does not use |
| [idea-part-b](idea-part-b.yml) | federal | Base frozen at FY1999; does not respond to disability counts |
| [esser](esser.yml) | federal | Bounded, in three tranches, expired September 2024 |

**The three federal nodes exist to be subtracted.** Ohio's federal share is unremarkable — 13.87%
against a national 13.24% in FY2022, 25th of 51 — and that is the point of holding them. The
distinctive thing about Ohio is the split between local and state, and a channel that behaves
normally is a constant that cannot explain it. Naming the constant is what lets the rest of the
position be read as structure rather than as an artefact of the year.

Federal money in Ohio *is* compensatory, monotonically across the wealth distribution, and it
closes 9.5% of the local gap against state equalization's 46%. The figures are on
[`equity`](../doctrine/equity.yml) and pinned by a test in
[`dispersion`](../../../crates/dispersion/src/national_peers.rs).

## Known gaps

**The two earmarks now answer the substitution question differently, and the difference is the
mechanism.** For the [lottery](lottery-profits.yml) it is settled as far as a budget document can
settle it: LSC states that profits are "combined with the GRF to provide foundation funding to
schools", and Fund 7017 sits inside the foundation aid appropriation table at $1.44 billion of an
$11.23 billion total in FY2026. The money is one of the buckets foundation aid is paid from, not
an addition to it. [verified — the greenbook, as enacted]

Those figures read $1.34 billion and $11.15 billion until the enacted document was read; both were
the executive proposal. Between proposal and act the lottery line rose $97.6m and the rest of the
table fell $15.6m — the substitution, at the margin the legislature controls. [verified]

For the [casino distribution](casino-tax-distribution.yml) there is no equivalent evidence and
cannot be, because the money never passes through an appropriation to the department — it runs
from the Tax Commissioner to county funds to districts, and no budget table exists in which it
could be shown netting against anything. [inference] The remaining question for both is the
counterfactual the critique actually asserts: whether aid would have been set higher without the
earmark. No appropriation record answers that. [open]

None of the three federal nodes carries a per-district series. Title I and ESSER allocations are
not retrieved by any connector; IDEA Part B is
[catalogued](../../catalog/ode-idea-part-b-allocations.md) with a known reader and no connector,
which makes it extraction work rather than a blocker. The aggregate federal channel per district
is available for FY2022 only, from the Census F-33. [verified]
