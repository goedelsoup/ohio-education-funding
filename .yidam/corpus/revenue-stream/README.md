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

## Known gap

Federal streams — Title I, IDEA Part B, and the ESSER funds that flowed from FY2020 through
FY2024 — have no nodes yet. ESSER in particular matters: it was large, temporary, and its
expiration is a live fiscal cliff for high-poverty districts. [open]
