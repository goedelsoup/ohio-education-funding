//! Category 3: what the state pays chartered nonpublic schools outside the scholarships.
//!
//! Three appropriation lines, about a quarter of a billion dollars a year, and none of them is a
//! scholarship. The department's own budget structure groups them — LSC's redbook and greenbook
//! both head the section **Category 3: Nonpublic School Support** — and the money moves under
//! R.C. 3317.024, 3317.06, 3317.062, 3317.063 and 3317.064 rather than under any of the six
//! funding units of R.C. 3317.022.
//!
//! | ALI | Fund | Line | Section |
//! |---|---|---|---|
//! | [`AUXILIARY_SERVICES`] | GRF | Auxiliary Services | R.C. 3317.06 and 3317.062 |
//! | [`ADMINISTRATIVE_COST_REIMBURSEMENT`] | GRF | Nonpublic Administrative Cost Reimbursement | R.C. 3317.063 |
//! | [`AUXILIARY_SERVICES_REIMBURSEMENT`] | 5980 | Auxiliary Services Reimbursement | R.C. 3317.064 |
//!
//! # What the sections say, and three things the line titles do not
//!
//! **R.C. 3317.06 does not allocate the money it is named for.** Its divisions (A) to (Q) are a
//! list of *permitted uses* — secular textbooks, health services, remedial services, mobile
//! units, security, language support — of "moneys paid to school districts under division (E)(1)
//! of section 3317.024 of the Revised Code". The allocating rule is
//! **R.C. 3317.024(E)(2)(d)**: the per-pupil amount "shall equal the total amount appropriated
//! for the implementation of sections 3317.06 and 3317.062 of the Revised Code" divided by the
//! average daily membership in grades kindergarten through twelve in chartered nonpublic schools
//! "as determined as of the last day of October of each school year". A reader who cites 3317.06
//! for the allocation is citing the wrong section, which is why [`series`] is documented against
//! the appropriation and [`per_pupil`] is documented against its denominator.
//!
//! **There are two payment routes and the line title implies one.** R.C. 3317.024(E)(1) pays the
//! school district the nonpublic school sits in; (E)(2) pays the chartered nonpublic school
//! directly where it elects to be paid that way, and the school may designate an organization to
//! receive and manage the funds, which may charge up to 4% of the school's total. Both LSC
//! editions state the same thing in prose. So "auxiliary services" is not a district programme
//! that happens to serve nonpublic pupils; it is a nonpublic entitlement with a district route.
//!
//! **R.C. 3317.064 is not a third programme.** It creates the auxiliary services reimbursement
//! fund, and the fund's revenue is a transfer of whatever the R.C. 4141.47 auxiliary services
//! personnel unemployment compensation fund holds *in excess* of its claims — and 4141.47(A)
//! says that fund "shall consist of moneys paid into the fund pursuant to section 3317.06". So
//! ALI `200659` is a two-step reflux of [`AUXILIARY_SERVICES`]'s own unspent money rather than
//! new money, which is why this module treats it as a component of Category 3 and the corpus
//! folds it into `program/auxiliary-services` as a property. Both LSC editions add the fact that
//! settles it: **no transfers have occurred since FY 2013** — see [`LAST_TRANSFER_YEAR`] — and
//! the line has been spending down a balance ever since.
//!
//! # The two mechanisms really are different
//!
//! [`AUXILIARY_SERVICES`] is a **direct appropriation** divided by a pupil count: what a school
//! is worth is the quotient, whatever its costs were. [`ADMINISTRATIVE_COST_REIMBURSEMENT`] is a
//! **reimbursement**: R.C. 3317.063 pays a school for the *actual* costs it incurred in the prior
//! year performing state-mandated clerical work, subject to a per-pupil ceiling the General
//! Assembly sets, with separate accounts, audit and repayment of anything unspent or misspent.
//!
//! The ceiling and the appropriation are two different limits and in FY2025 the appropriation was
//! the binding one: the General Assembly authorised [`AUTHORIZED_ADMINISTRATIVE_RATE`] per
//! student for FY2024 through FY2027, and LSC records that "the amount appropriated permitted DEW
//! to pay up to" [`PUBLISHED_ADMINISTRATIVE_RATE_FY2025`] — see [`rationing_fy2025`].
//!
//! # Why every year has up to four answers
//!
//! The catalog restates six fiscal years per edition, and a year's figure changes as it closes:
//! `appropriation` when the biennium is enacted, `adjusted` if it is reopened mid-biennium,
//! `actual` once the books are shut. [`restatements`] is the edition-to-edition walk, and the
//! invariant worth holding is that a *kind* never disagrees with itself — two editions publishing
//! the same year on the same basis publish the same number. See
//! [`same_kind_disagreements`], which asserts it over the whole fixture rather than these three
//! lines, and `the_lines_that_pay_the_nonpublic_schools` which pins the count.

use std::collections::{BTreeMap, BTreeSet};

use deflator::CpiSeries;
use edfund_core::FiscalYear;

/// GRF ALI `200511`, Auxiliary Services.
///
/// Spent under R.C. 3317.06 and 3317.062, allocated under R.C. 3317.024(E)(2)(d), and paid either
/// through the district or direct to the school. $166.8 million in FY2025.
pub const AUXILIARY_SERVICES: &str = "200511";

/// GRF ALI `200532`, Nonpublic Administrative Cost Reimbursement.
///
/// R.C. 3317.063. Established by Am. Sub. H.B. 694 of the 114th General Assembly. $75.3 million
/// in FY2025.
pub const ADMINISTRATIVE_COST_REIMBURSEMENT: &str = "200532";

/// Fund 5980 ALI `200659`, Auxiliary Services Reimbursement.
///
/// R.C. 3317.064. The mobile-unit line, and the reflux described in the module note: its revenue
/// is excess cash transferred out of the R.C. 4141.47 fund, which is itself fed from `200511`.
/// $534,753 in FY2025 — three thousandths of the category.
pub const AUXILIARY_SERVICES_REIMBURSEMENT: &str = "200659";

/// The three lines LSC groups as Category 3, in the order both editions print them.
pub const CATEGORY_THREE: [&str; 3] = [
    AUXILIARY_SERVICES,
    ADMINISTRATIVE_COST_REIMBURSEMENT,
    AUXILIARY_SERVICES_REIMBURSEMENT,
];

/// The last fiscal year the R.C. 4141.47 fund transferred anything to
/// [`AUXILIARY_SERVICES_REIMBURSEMENT`].
///
/// "No transfers have occurred since FY 2013", in both the redbook and the greenbook. Every
/// dollar `200659` has spent in the twelve years since is drawdown of a balance, which is the
/// reason the line is a property of the auxiliary services programme rather than a programme.
pub const LAST_TRANSFER_YEAR: u16 = 2013;

/// What LSC publishes as the FY2025 auxiliary services rate: **$913 per pupil**.
///
/// Stated identically by the redbook ("is $913 per pupil") and the greenbook ("was $913 per
/// pupil"), so it is not a forecast that was later corrected. This module does not reproduce it
/// — see [`per_pupil`] and [`implied_membership`] for why, which is a question about the
/// denominator rather than about the rate.
pub const PUBLISHED_AUXILIARY_RATE_FY2025: f64 = 913.0;

/// What the FY2025 appropriation for [`ADMINISTRATIVE_COST_REIMBURSEMENT`] permitted: **$440 per
/// student**.
pub const PUBLISHED_ADMINISTRATIVE_RATE_FY2025: f64 = 440.0;

/// What R.C. 3317.063's ceiling was set at for FY2024 through FY2027: **$475 per student**.
///
/// The authorised maximum, not the paid rate. The gap between this and
/// [`PUBLISHED_ADMINISTRATIVE_RATE_FY2025`] is [`rationing_fy2025`].
pub const AUTHORIZED_ADMINISTRATIVE_RATE: f64 = 475.0;

/// The FY2025 College Credit Plus earmark inside [`AUXILIARY_SERVICES`], as actually spent.
///
/// The greenbook's figure. The redbook, written against the executive proposal, carries
/// $2,600,000 — see [`Restatement`] for the general shape of that difference, which this is an
/// instance of inside a line rather than of the line.
pub const COLLEGE_CREDIT_PLUS_FY2025: f64 = 2_739_015.0;

/// The committed catalog extract: one row per line item per fiscal year per edition.
const CATALOG: &str = include_str!("../../fixtures/catalog-line-items.csv");

/// The header [`claims`] indexes against.
const CATALOG_HEADER: &str = "edition,fund,ali,name,fiscal_year,kind,amount";

/// The columns of [`CATALOG_HEADER`] this module reads.
mod catalog_column {
    pub const EDITION: usize = 0;
    pub const FUND: usize = 1;
    pub const ALI: usize = 2;
    pub const NAME: usize = 3;
    pub const FISCAL_YEAR: usize = 4;
    pub const KIND: usize = 5;
    pub const AMOUNT: usize = 6;
}

/// The department's own count of what is enrolled where, 2023-24.
const CHANNELS: &str = include_str!("../../fixtures/education-landscape-channels.csv");

/// The header [`chartered_nonpublic_enrolment`] indexes against.
const CHANNELS_HEADER: &str = "channel,enrollment,parent";

/// The row of [`CHANNELS`] that counts the pupils Category 3 is paid for.
const CHARTERED_PRIVATE: &str = "Chartered Private Schools";

/// One edition's statement about one line in one fiscal year.
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    /// The catalog edition making the claim — the vintage of the number.
    pub edition: u16,
    /// The fund the line sits in: `GRF` for two of the three, `5980` for the reflux.
    pub fund: String,
    /// The six-digit appropriation line item.
    pub ali: String,
    /// Its title as the edition prints it.
    pub name: String,
    /// The fiscal year claimed for.
    pub fiscal_year: u16,
    /// `appropriation`, `adjusted` or `actual`.
    pub kind: String,
    /// The amount, in the dollars of `fiscal_year`. `None` where the edition prints no figure.
    pub amount: Option<f64>,
}

/// Every claim any edition makes about `ali`, oldest edition first.
///
/// # Panics
///
/// If the catalog fixture's header is not the one this was written against, which means the
/// extractor changed and this reader did not.
#[must_use]
pub fn claims(ali: &str) -> Vec<Claim> {
    let mut out: Vec<Claim> = edfund_core::csv::rows(CATALOG, CATALOG_HEADER)
        .filter(|row| row.str(catalog_column::ALI) == ali)
        .filter_map(|row| {
            Some(Claim {
                edition: row.str(catalog_column::EDITION).parse().ok()?,
                fund: row.str(catalog_column::FUND).to_string(),
                ali: row.str(catalog_column::ALI).to_string(),
                name: row.str(catalog_column::NAME).to_string(),
                fiscal_year: row.str(catalog_column::FISCAL_YEAR).parse().ok()?,
                kind: row.str(catalog_column::KIND).to_string(),
                amount: row.num(catalog_column::AMOUNT),
            })
        })
        .collect();
    out.sort_by_key(|claim| (claim.fiscal_year, claim.edition));
    out
}

/// What the newest edition that speaks for `fiscal_year` says about `ali`.
///
/// The rule the whole module is built on: a year is answered by exactly one edition, the latest
/// one that carries it. That is what makes [`series`] a single vintage rather than a mixture —
/// the defect `ledger::appropriations` records having made once, where a series' vintage changed
/// halfway along without saying so.
#[must_use]
pub fn latest(ali: &str, fiscal_year: u16) -> Option<Claim> {
    claims(ali)
        .into_iter()
        .rfind(|claim| claim.fiscal_year == fiscal_year && claim.amount.is_some())
}

/// One fiscal year of one line, nominal and real, with the vintage that answers for it.
#[derive(Debug, Clone, PartialEq)]
pub struct Year {
    /// The fiscal year.
    pub fiscal_year: u16,
    /// The edition the figure is taken from — always the latest that carries the year.
    pub edition: u16,
    /// Whether that edition calls it an `appropriation`, an `adjusted` appropriation or an
    /// `actual`. The last two years of any series are appropriations; the rest are actuals.
    pub kind: String,
    /// The amount in the dollars of `fiscal_year`.
    pub nominal: f64,
    /// The same in `base` dollars. `None` where the index cannot reach the year.
    pub real: Option<f64>,
}

/// One line's series, latest vintage per year, restated into `base` dollars.
#[must_use]
pub fn series(ali: &str, base: FiscalYear) -> Vec<Year> {
    let cpi = CpiSeries::cpi_u_june();
    let mut answered: BTreeMap<u16, Claim> = BTreeMap::new();
    for claim in claims(ali) {
        if claim.amount.is_none() {
            continue;
        }
        answered
            .entry(claim.fiscal_year)
            .and_modify(|held| {
                if claim.edition > held.edition {
                    *held = claim.clone();
                }
            })
            .or_insert(claim);
    }
    answered
        .into_values()
        .map(|claim| {
            let nominal = claim.amount.unwrap_or_default();
            Year {
                fiscal_year: claim.fiscal_year,
                edition: claim.edition,
                kind: claim.kind,
                nominal,
                real: cpi
                    .convert(nominal, FiscalYear(claim.fiscal_year), base)
                    .ok()
                    .map(|deflated| deflated.value),
            }
        })
        .collect()
}

/// [`CATEGORY_THREE`] summed, latest vintage per line per year, restated into `base` dollars.
///
/// A year appears only where all three lines carry a figure for it, which over this fixture is
/// every year from FY2002 to FY2027. `kind` is the kind the three agree on, or `mixed` where they
/// do not — which has not happened and would mean an edition closed two of the three.
#[must_use]
pub fn category_three(base: FiscalYear) -> Vec<Year> {
    let cpi = CpiSeries::cpi_u_june();
    let mut parts: BTreeMap<u16, Vec<Year>> = BTreeMap::new();
    for ali in CATEGORY_THREE {
        for year in series(ali, base) {
            parts.entry(year.fiscal_year).or_default().push(year);
        }
    }
    parts
        .into_iter()
        .filter(|(_, years)| years.len() == CATEGORY_THREE.len())
        .map(|(fiscal_year, years)| {
            let nominal: f64 = years.iter().map(|year| year.nominal).sum();
            let kinds: BTreeSet<&str> = years.iter().map(|year| year.kind.as_str()).collect();
            Year {
                fiscal_year,
                edition: years.iter().map(|year| year.edition).max().unwrap_or(0),
                kind: if kinds.len() == 1 {
                    years[0].kind.clone()
                } else {
                    "mixed".to_string()
                },
                nominal,
                real: cpi
                    .convert(nominal, FiscalYear(fiscal_year), base)
                    .ok()
                    .map(|deflated| deflated.value),
            }
        })
        .collect()
}

/// One edition changing what an earlier edition said about a year.
#[derive(Debug, Clone, PartialEq)]
pub struct Restatement {
    /// The line.
    pub ali: String,
    /// The year restated.
    pub fiscal_year: u16,
    /// The edition that spoke first, and what it said.
    pub from: (u16, String, f64),
    /// The edition that changed it, and what it said instead.
    pub to: (u16, String, f64),
}

impl Restatement {
    /// Whether the change is a step along `appropriation` → `adjusted` → `actual`.
    ///
    /// The only kind of restatement this fixture contains. A restatement that is *not* a kind
    /// transition would be one edition contradicting another on the same basis, which is the
    /// failure [`same_kind_disagreements`] exists to catch.
    #[must_use]
    pub fn is_kind_transition(&self) -> bool {
        self.from.1 != self.to.1
    }

    /// What the later edition added to the earlier one, in the dollars of the year.
    #[must_use]
    pub fn shift(&self) -> f64 {
        self.to.2 - self.from.2
    }
}

/// Every consecutive pair of editions that changed a figure for `ali`.
///
/// Pairs where the two editions agree are not returned: an edition merely repeating a closed
/// year's actual is not a restatement of it.
#[must_use]
pub fn restatements(ali: &str) -> Vec<Restatement> {
    let mut by_year: BTreeMap<u16, Vec<Claim>> = BTreeMap::new();
    for claim in claims(ali) {
        if claim.amount.is_some() {
            by_year.entry(claim.fiscal_year).or_default().push(claim);
        }
    }
    let mut out = Vec::new();
    for (fiscal_year, spoken) in by_year {
        for pair in spoken.windows(2) {
            let (earlier, later) = (&pair[0], &pair[1]);
            let (before, after) = (
                earlier.amount.unwrap_or_default(),
                later.amount.unwrap_or_default(),
            );
            if (before - after).abs() < f64::EPSILON && earlier.kind == later.kind {
                continue;
            }
            out.push(Restatement {
                ali: ali.to_string(),
                fiscal_year,
                from: (earlier.edition, earlier.kind.clone(), before),
                to: (later.edition, later.kind.clone(), after),
            });
        }
    }
    out
}

/// Two editions publishing the same line, the same year and the same kind, and disagreeing.
///
/// Over the **whole** catalog fixture rather than [`CATEGORY_THREE`], because the invariant is a
/// property of the extraction and a three-line version of it would pass on a fixture that had
/// gone wrong everywhere else. Returns `(ali, fiscal_year, kind, the distinct amounts)`.
///
/// This is empty, and that is the point: every edition-to-edition difference in the fixture is a
/// kind transition. It is the evidence that [`series`] taking the newest edition is a choice of
/// *vintage* and not a choice between two claims about one quantity.
#[must_use]
pub fn same_kind_disagreements() -> Vec<(String, u16, String, Vec<f64>)> {
    let mut seen: BTreeMap<(String, u16, String), Vec<f64>> = BTreeMap::new();
    for row in edfund_core::csv::rows(CATALOG, CATALOG_HEADER) {
        let Some(amount) = row.num(catalog_column::AMOUNT) else {
            continue;
        };
        let Ok(fiscal_year) = row.str(catalog_column::FISCAL_YEAR).parse::<u16>() else {
            continue;
        };
        let key = (
            row.str(catalog_column::ALI).to_string(),
            fiscal_year,
            row.str(catalog_column::KIND).to_string(),
        );
        let held = seen.entry(key).or_default();
        if !held.iter().any(|v| (v - amount).abs() < f64::EPSILON) {
            held.push(amount);
        }
    }
    seen.into_iter()
        .filter(|(_, amounts)| amounts.len() > 1)
        .map(|((ali, fiscal_year, kind), amounts)| (ali, fiscal_year, kind, amounts))
        .collect()
}

/// How many `(ali, fiscal_year)` cells the fixture carries a figure for, and over how many lines.
///
/// The denominator [`same_kind_disagreements`]' emptiness is a result about. A count of zero is
/// only worth stating beside the number of chances it had.
#[must_use]
pub fn restatable_cells() -> (usize, usize) {
    let mut cells: BTreeSet<(String, u16)> = BTreeSet::new();
    for row in edfund_core::csv::rows(CATALOG, CATALOG_HEADER) {
        if row.num(catalog_column::AMOUNT).is_none() {
            continue;
        }
        let Ok(fiscal_year) = row.str(catalog_column::FISCAL_YEAR).parse::<u16>() else {
            continue;
        };
        cells.insert((row.str(catalog_column::ALI).to_string(), fiscal_year));
    }
    let lines: BTreeSet<&String> = cells.iter().map(|(ali, _)| ali).collect();
    (cells.len(), lines.len())
}

/// Chartered nonpublic enrolment as the department's own landscape sheet counts it.
///
/// **173,156**, and the vintage is the whole caveat: the sheet is *Ohio's Education Landscape
/// 2023-2024*, so this is the 2023-24 school year over the 711 chartered nonpublic schools it
/// counts. The FY2025 appropriation was divided by a FY2025 October membership over a larger
/// population — LSC's redbook counts 747 chartered nonpublic schools and its greenbook 725 — so
/// every quotient [`per_pupil`] returns is an **upper bound** on the rate the department paid.
///
/// # Panics
///
/// If the landscape fixture's header changed, or if it no longer carries the chartered private
/// school row this is the whole point of reading.
#[must_use]
pub fn chartered_nonpublic_enrolment() -> f64 {
    edfund_core::csv::rows(CHANNELS, CHANNELS_HEADER)
        .find(|row| row.str(0) == CHARTERED_PRIVATE)
        .and_then(|row| row.num(1))
        .expect("the landscape fixture no longer counts chartered private school enrolment")
}

/// The published vintage of [`chartered_nonpublic_enrolment`], for prose that has to say so.
pub const ENROLMENT_VINTAGE: &str = "2023-2024";

/// What a line is worth per chartered nonpublic pupil in `fiscal_year`, on
/// [`chartered_nonpublic_enrolment`].
///
/// R.C. 3317.024(E)(2)(d) really does divide, so this is the right *shape* of computation and the
/// wrong denominator — see [`chartered_nonpublic_enrolment`] and [`implied_membership`]. It is
/// published as a bound rather than as a rate for that reason, and the corpus records the October
/// average daily membership the statute names as unheld.
#[must_use]
pub fn per_pupil(ali: &str, fiscal_year: u16) -> Option<f64> {
    let claim = latest(ali, fiscal_year)?;
    Some(claim.amount? / chartered_nonpublic_enrolment())
}

/// The membership a published per-pupil rate implies, given what was appropriated.
///
/// The inverse of [`per_pupil`], and the instrument that says how far the fixture's denominator
/// is from the statutory one. On FY2025 it answers two different numbers from the two published
/// rates — see `the_two_rates_do_not_share_a_denominator` — which is a fact about the two
/// mechanisms rather than about either figure.
#[must_use]
pub fn implied_membership(appropriated: f64, rate: f64) -> Option<f64> {
    (rate > 0.0).then_some(appropriated / rate)
}

/// What the FY2025 appropriation cost a school against the rate R.C. 3317.063 authorised.
///
/// The General Assembly authorised [`AUTHORIZED_ADMINISTRATIVE_RATE`]; the appropriation
/// permitted [`PUBLISHED_ADMINISTRATIVE_RATE_FY2025`]. Returns `(shortfall per student, share of
/// the authorised rate not paid)`.
///
/// This is the difference between the two mechanisms made arithmetic. A reimbursement rationed by
/// its appropriation pays less than the statute allows and the statute is unchanged; a direct
/// appropriation divided by a pupil count has no rate to fall short of.
#[must_use]
pub fn rationing_fy2025() -> (f64, f64) {
    let shortfall = AUTHORIZED_ADMINISTRATIVE_RATE - PUBLISHED_ADMINISTRATIVE_RATE_FY2025;
    (shortfall, shortfall / AUTHORIZED_ADMINISTRATIVE_RATE)
}

/// Growth across a series, nominal and real, as fractions.
///
/// `None` where either endpoint has no real figure — a growth rate computed from one deflated
/// endpoint and one nominal one is not a number anybody should be handed. The same rule, and the
/// same reason, as `ledger::appropriations::growth`.
#[must_use]
pub fn growth(history: &[Year]) -> Option<(f64, f64)> {
    let first = history.first()?;
    let last = history.last()?;
    if first.nominal <= 0.0 {
        return None;
    }
    let (first_real, last_real) = (first.real?, last.real?);
    if first_real <= 0.0 {
        return None;
    }
    Some((
        last.nominal / first.nominal - 1.0,
        last_real / first_real - 1.0,
    ))
}

/// One year of a series, found by fiscal year.
#[must_use]
pub fn at(history: &[Year], fiscal_year: u16) -> Option<Year> {
    history
        .iter()
        .find(|year| year.fiscal_year == fiscal_year)
        .cloned()
}
