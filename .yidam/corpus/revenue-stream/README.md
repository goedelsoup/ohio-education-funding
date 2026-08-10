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

**The state lottery has no node, and it is the sharper version of the question the casino node
raises.** Ohio Constitution Article XV Section 6(A) — three paragraphs above the casino provision,
in the section this class now cites — requires the entire net proceeds of the lottery to be "used
solely for the support of elementary, secondary, vocational, and special education programs **as
determined in appropriations made by the General Assembly**." That final clause is the hinge of
every earmark-substitution argument in Ohio school finance, and the lottery is the instance voters
are most likely to believe is additive. [open]

**Neither earmark can be shown additive or substitutive from statute.** No section of law reduces
foundation aid by what a district receives from either channel; whether the General Assembly set
aid lower because the channels exist is a counterfactual about appropriations, answerable only
from the appropriation history. That is `lsc-budget` work and is the same blocker as the pre-2000
record. [open]

None of the three federal nodes carries a per-district series. Title I and ESSER allocations are
not retrieved by any connector; IDEA Part B is
[catalogued](../../catalog/ode-idea-part-b-allocations.md) with a known reader and no connector,
which makes it extraction work rather than a blocker. The aggregate federal channel per district
is available for FY2022 only, from the Census F-33. [unentered]
