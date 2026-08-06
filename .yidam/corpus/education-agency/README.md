# education-agency

The exemplar agencies. Ohio has roughly 610 traditional districts plus several hundred
community and STEM schools, and this corpus deliberately does not hold a node for each — the
bulk per-agency-year series live under [`crates/`](../../../crates/) and are queried by the
domain computer. What lives here is a curated set chosen to span the distinctions that matter,
so that a claim about the system can be checked against a concrete case rather than only
against a statewide average.

The five seeded agencies were chosen to differ on the axes that drive Ohio funding outcomes:

- **Property wealth per pupil** — [Northern Local](northern-local-perry.yml) at the low end,
  [Upper Arlington](upper-arlington-city.yml) at the high end
- **Agency type** — traditional district, joint vocational
  ([Eastland-Fairfield](eastland-fairfield-ctc.yml)), community school
  ([ECOT](electronic-classroom-of-tomorrow.yml))
- **Scholarship exposure** — [Cleveland](cleveland-municipal.yml), the origin of Ohio's
  voucher channel

An agency is a UFO Kind, and the labels that get attached to districts in this domain —
property-poor, 20-mill-floor, guarantee district, *DeRolph* plaintiff — are Roles it plays
contingently and often simultaneously. They belong in the `roles` property, with the fiscal
periods each applied, not in the node's identity.

See the class definition: [education-agency.ont.yml](../education-agency.ont.yml).

## Instances

| Node | Type | Why it is here |
|------|------|----------------|
| [northern-local-perry](northern-local-perry.yml) | local | *DeRolph* origin district; rural, property-poor |
| [upper-arlington-city](upper-arlington-city.yml) | city | High valuation per pupil; the contrast case |
| [cleveland-municipal](cleveland-municipal.yml) | city | Largest urban district; origin of the voucher channel |
| [eastland-fairfield-ctc](eastland-fairfield-ctc.yml) | joint-vocational | JVSDs are funded differently, including a 2-mill floor |
| [electronic-classroom-of-tomorrow](electronic-classroom-of-tomorrow.yml) | community | The enrollment-funding failure case |

## Known gaps

No exempted village district and no STEM school, so two of the six agency types are
unrepresented. No suburban district of moderate wealth — the current set is deliberately
polarized and the middle of the distribution, where most Ohio students actually are, is
missing. [open]
