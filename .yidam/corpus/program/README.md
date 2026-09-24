# program

The separately-authorized initiatives that move education money outside the base formula. Each
node binds the state, a recipient, and — in most cases — a district whose funding is affected,
which is why programs are modeled as Relators rather than as edges. A voucher is not a channel
of money to a district; it is an arrangement with its own eligibility rules, appropriation
line, per-pupil amount, and in two cases its own litigation.

This class holds the two funding channels that are easiest to argue about and hardest to
compare. Scholarships move money toward families and away from resident districts;
[facilities assistance](classroom-facilities-assistance.yml) moves money toward districts for
capital purposes and was itself ordered as part of the *DeRolph* remedy. Both sit outside the
formula, and neither shows up in a per-pupil operating figure — so a district's foundation
payment can rise while its total position worsens, or fall while its buildings are rebuilt.

See the class definition: [program.ont.yml](../program.ont.yml).

## Instances

| Node | Mechanism | Direction |
|------|-----------|-----------|
| [cleveland-scholarship](cleveland-scholarship.yml) | scholarship | Away from resident district |
| [edchoice-expansion](edchoice-expansion.yml) | scholarship | Away from resident district |
| [classroom-facilities-assistance](classroom-facilities-assistance.yml) | capital assistance | Toward district |
| [edchoice-scholarship](edchoice-scholarship.yml) | scholarship | Away from resident district |
| [autism-scholarship](autism-scholarship.yml) | scholarship | Away from resident district |
| [jon-peterson-special-needs](jon-peterson-special-needs.yml) | scholarship | Away from resident district |

## Known gaps

Every scholarship programme the department runs has a node, and the channel is totalled from
the 2025 annual report. What the class does not hold:

- **No node for any non-scholarship nonpublic aid line.** Auxiliary Services and the Nonpublic
  Administrative Cost Reimbursement move state money to chartered nonpublic schools outside the
  scholarship channel, and neither is modelled here. [open]
- **No per-district participation.** Every scholarship figure held is statewide. The annual
  report cites a per-district breakdown on the department's reports portal, and this project has
  not reached it — the route's 404 is a property of the portal, not evidence about the file
  ([#10](https://github.com/goedelsoup/ohio-education-funding/issues/10)). [open]
- **FY2014 through FY2023 has no participation series.** The archive stops at FY2013 and the
  annual report starts at 2024-25; inside the hole the greenbooks quote rounded counts for FY2014,
  FY2018 and FY2019 and nothing else. [open]
