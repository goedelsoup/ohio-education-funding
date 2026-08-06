# actor

The institutions that act on Ohio's school funding system. Four kinds of action matter here
and the class exists to keep them distinguishable: enacting law, deciding cases,
administering programs, and publishing the data everything else in this corpus is grounded
in. The last one is easy to overlook and is why this class is not merely decorative — nearly
every number in the corpus traces back to a publication by one of these bodies, and knowing
which body published it is often the difference between two figures that disagree.

Institutional identity here is rigid but successions are real. When the Department of
Education and Workforce replaced the Department of Education in 2023, that was a change of
entity — different governance, different reporting line — not a rename, and the corpus models
it as two nodes joined by a `succeeds` edge rather than one node with a changed name.

See the class definition: [actor.ont.yml](../actor.ont.yml).

## Instances

| Node | Type |
|------|------|
| [ohio-general-assembly](ohio-general-assembly.yml) | legislature |
| [supreme-court-of-ohio](supreme-court-of-ohio.yml) | court |
| [department-of-education-and-workforce](department-of-education-and-workforce.yml) | executive agency |
| [ohio-department-of-education](ohio-department-of-education.yml) | executive agency (superseded) |
