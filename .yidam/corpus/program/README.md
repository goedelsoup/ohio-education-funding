# program

The separately-authorized initiatives that move education money outside the base formula. Each
node binds the state, a recipient, and — in most cases — a district whose funding is affected,
which is why programs are modeled as Relators rather than as edges. A voucher is not a channel
of money to a district; it is an arrangement with its own eligibility rules, appropriation
line, per-pupil amount, and in two cases its own litigation.

This class holds the funding channels that are easiest to argue about and hardest to compare.
Scholarships move money toward families and away from resident districts;
[facilities assistance](classroom-facilities-assistance.yml) moves money toward districts for
capital purposes and was itself ordered as part of the *DeRolph* remedy. Both sit outside the
formula, and neither shows up in a per-pupil operating figure — so a district's foundation
payment can rise while its total position worsens, or fall while its buildings are rebuilt.

And a third direction, added last and older than either. [Auxiliary
services](auxiliary-services.yml) and the [nonpublic administrative cost
reimbursement](nonpublic-administrative-cost-reimbursement.yml) move state money to chartered
nonpublic schools with no family applying and no district deducted — a quarter of a billion
dollars a year, about a quarter of the scholarship channel, on appropriation lines committed to
this repository since the 2006 catalog edition and read by nothing until now. The pair is also
the corpus's clearest worked contrast between the two mechanisms this class names: one divides an
appropriation by a membership, the other repays what a school can show it spent.

See the class definition: [program.ont.yml](../program.ont.yml).

## Instances

| Node | Mechanism | Direction |
|------|-----------|-----------|
| [auxiliary-services](auxiliary-services.yml) | direct appropriation | Toward chartered nonpublic school |
| [nonpublic-administrative-cost-reimbursement](nonpublic-administrative-cost-reimbursement.yml) | reimbursement | Toward chartered nonpublic school |
| [cleveland-scholarship](cleveland-scholarship.yml) | scholarship | Away from resident district |
| [edchoice-expansion](edchoice-expansion.yml) | scholarship | Away from resident district |
| [classroom-facilities-assistance](classroom-facilities-assistance.yml) | capital assistance | Toward district |
| [edchoice-scholarship](edchoice-scholarship.yml) | scholarship | Away from resident district |
| [autism-scholarship](autism-scholarship.yml) | scholarship | Away from resident district |
| [jon-peterson-special-needs](jon-peterson-special-needs.yml) | scholarship | Away from resident district |

## Known gaps

Every scholarship programme the department runs has a node, the channel is totalled from the
2025 annual report, and the two largest non-scholarship nonpublic lines now have nodes of their
own. The October membership R.C. 3317.024(E)(2)(d) divides by, which this list used to call
missing, is held: forty-nine Octobers of it, and the auxiliary services rate reproduces on the
one the statute names. What the class does not hold:

- **No district of location for a chartered nonpublic school.** R.C. 3317.024(E)(1) pays the
  district a nonpublic school sits in, so the resident-side incidence of auxiliary services is a
  district question. The department's October enrolment files carry an IRN, and before 2007 a
  county and a school type, and no district column in any of the six layouts they have used; a
  join on name is not available either, since October 2023's 711 schools carry 601 distinct names
  between them. [open]
- **No split between the district route and the direct route.** R.C. 3317.024(E)(2) makes it a
  per-school election and neither LSC edition reports the result, so how much of auxiliary
  services passes through districts is not held. [open]
- **No per-district participation.** Every scholarship figure held is statewide. The annual
  report cites a per-district breakdown on the department's reports portal, and this project has
  not reached it — the route's 404 is a property of the portal, not evidence about the file
  ([#10](https://github.com/goedelsoup/ohio-education-funding/issues/10)). [open]
- **FY2014 through FY2023 has no participation series.** The archive stops at FY2013 and the
  annual report starts at 2024-25; inside the hole the greenbooks quote rounded counts for FY2014,
  FY2018 and FY2019 and nothing else. [open]
