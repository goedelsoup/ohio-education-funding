# dew-payment-reports — connector

**Source.** Ohio Department of Education and Workforce, foundation payment reports: the
district-by-district record of what was actually paid, as against what a formula computed.

**Feeds.** [`program`](../../../.yidam/corpus/program/),
[`education-agency`](../../../.yidam/corpus/education-agency/),
[`revenue-stream`](../../../.yidam/corpus/revenue-stream/).

This is the one source that would carry the voucher and community-school **deduction per resident
district**, for the years it existed.

The FY2027 calculator does not carry it. Its transfer channel is a named service-centre charge
plus a residual too small to hide a deduction in, and under the Fair School Funding Plan those
students are funded directly rather than deducted from the district of residence. So the deduction
is not missing from the current model — it is absent from it by design. What is missing is the era
before.

## Constraints

- Payment reports are periodic within a fiscal year. A district's annual total is the final
  settlement, not the sum of the interim runs, and adding the runs double-counts.
- Deduct-era and direct-funding-era reports are not the same measurement, and a series that spans
  the change without marking it describes a discontinuity as a trend.
- Offline mode required.

## Status

**Declared.** Approved in
[decisions/payment-reports-connector.yml](../../../.yidam/decisions/payment-reports-connector.yml);
no endpoint wired.

**The recorded blocker was wrong in both directions.**

*"No index and no stable path"* is stale. `education.ohio.gov` now lists 38 direct `.xlsx` payment
reports for FY2026 and FY2027 at fixed URLs.

*"The years before about 2015 are not on the current host"* is wrong in kind. The department
publishes *Foundation Legacy Payment Reports (1999–2021)*, covering the whole deduct era, on its
reports portal. That portal gates on `sessionStorage.claims` and needs an OH|ID account.

**And the gated collection overlaps the open one by six years.** The same
`Districts-Payment-Reports-in-Excel` index that carries FY2026 and FY2027 also carries one final
settlement per year back to **FY2016** — `FY16_SFPR_FIN_4.XLSX` through `FY21_SFPR_FIN_2.xlsx`, then
FY2022 forward. Those are deduct-era files, openly served, no credentials. FY2019 and FY2021 have
been taken; see [`dew-fy19-payment-report`](../../../.yidam/catalog/dew-fy19-payment-report.md).

They carry the deduction this connector exists for, per resident district and per channel: the
FY2019 report breaks out `COMMUNITY SCHOOL TRANSFER`, `STEM SCHOOL TRANSFER` and
`SCHOLARSHIP TRANSFER` as separate columns, and the FY2021 report splits the scholarship transfer
into its Cleveland, autism, EdChoice and Jon Peterson components.

So FY2016 through FY2021 of this connector's range needs no login. What OH|ID still gates is 1999
through 2015, and a login is a different problem from an absent index and not one to route around.
