# litigation

The cases in which Ohio's funding system has been tested against the state constitution. Four
nodes, spanning 1979 to 2025, and together they carry a strange result: the system was upheld
once, held unconstitutional twice, and then left unadjudicated when the Supreme Court of Ohio
ended the remand in 2003 without the system having been found compliant. [verified]

That ending is why this class matters more than a case list usually would. The *DeRolph*
declaration was never withdrawn and never satisfied. Every funding regime since 2003 has
operated in the space that opened when a constitutional judgment lost its enforcement
mechanism, and any account of the current system that omits this is describing policy
choices as though they were made against a clean slate.

Litigation is an Event class: cases unfold over intervals and have temporal parts. *DeRolph*
in particular is four decisions over five years, and the corpus models them as separate
nodes joined by `cites` rather than as one case with a date range, because the court's
position changed materially between them.

See the class definition: [litigation.ont.yml](../litigation.ont.yml).

## Instances

| Node | Decided | Result |
|------|---------|--------|
| [cincinnati-v-walter-1979](cincinnati-v-walter-1979.yml) | 1979 | System upheld, 4-3 |
| [derolph-i-1997](derolph-i-1997.yml) | 1997 | System unconstitutional, 4-3 |
| [derolph-ii-2000](derolph-ii-2000.yml) | 2000 | Response still non-compliant; the thorough-and-efficient test defined |
| [derolph-iii-2001](derolph-iii-2001.yml) | 2001 | H.B. 94 constitutional as modified — vacated in 2002 |
| [derolph-iv-2002](derolph-iv-2002.yml) | 2002 | DeRolph III vacated, I and II reinstated; remand later ended |
| [vouchers-hurt-ohio-2025](vouchers-hurt-ohio-2025.yml) | 2025 | EdChoice held unconstitutional at trial level |

**Read the four in order or not at all.** The sequence is not cumulative: III found the enacted
plan constitutional subject to four ordered fixes, and IV vacated III outright. The standing
judgment is I and II, which declare the system unconstitutional and define the test, without any
of III's specifics about how to satisfy it.

## Known gaps

The 2003 prohibition action that ended the remand is described inside `derolph-iv-2002` rather
than modeled separately. *Zelman v. Simmons-Harris* (U.S. 2002), which upheld the Cleveland
voucher program against a federal Establishment Clause challenge, belongs here too — the Ohio
constitutional question the 2025 case raises is distinct from the federal one Zelman settled.
[open]
