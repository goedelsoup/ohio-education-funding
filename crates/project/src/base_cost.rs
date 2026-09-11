//! Ohio's statewide base cost per pupil, every year it has had one, in nominal and constant
//! dollars.
//!
//! # The question this answers
//!
//! `parameter/base-cost-per-pupil` calls this "the most consequential single number in Ohio
//! school funding" and publishes a series for it — and closed with three things it did not
//! hold: "the FY1992-FY2009 statutory series, the Evidence-Based Model's computed adequacy
//! amount for FY2010-11, and the FY2024-FY2027 statewide averages. All figures need a
//! constant-dollar companion series before any cross-era comparison."
//!
//! Every part of that was already in this workspace. [`crate::greenbook`] holds the Legislative
//! Service Commission's analysis of all twelve enacted budget acts from the 124th General
//! Assembly through the 135th, plus the 136th's at [`crate::ledger::budget_analysis::GREENBOOK`];
//! LSC states the amount, in prose, biennium by biennium. [`deflator`] converts it.
//!
//! # What the series is not
//!
//! **It is not annual.** Four fiscal years have no statewide base cost per pupil at all — not an
//! unrecorded one, none. The Evidence-Based Model replaced the per-pupil base with a
//! per-teacher compensation amount, and LSC says so outright: "there is no specific formula
//! amount per pupil." The Bridge formula that replaced *it* distributed on each district's own
//! FY2011 aid. So FY2010 through FY2013 are [`Basis::Absent`], and the node's request for "the
//! Evidence-Based Model's computed adequacy amount" is a request for a number that was never
//! computed. See [`gap`].
//!
//! **It is not a new figure each year either.** Since FY2019 the amount has been recomputed once
//! a biennium and frozen for the second year, and H.B. 96 froze the FY2024 amount for two more.
//! [`Year::frozen`] carries the act's own words for each freeze, and [`freezes`] reads them back.
//!
//! # What deflation shows
//!
//! The nominal series rises 71.2% from FY2002 to FY2026. In FY2026 dollars it **falls 7.8%**,
//! and its high is FY2003. Within each regime, where the comparison is between figures of the
//! same kind:
//!
//! | era | years | real change |
//! |---|---|---|
//! | base cost formula amount | FY2002-FY2009 | −0.7% |
//! | opportunity grant formula amount | FY2014-FY2021 | −8.1% |
//! | Fair School Funding Plan average | FY2022-FY2026 | −0.5% |
//!
//! The deflator settles the price half of the objection `parameter/base-cost-per-pupil` records
//! against cross-era comparison. It does not settle the other half: a pre-2022 figure is a
//! legislated amount and a post-2022 one is the quotient of a district build-up, and no index
//! makes those the same quantity. [`Basis`] is carried per year so a caller can decline the
//! comparison the node warns about while still making the ones inside an era.
//!
//! # FY2027 cannot be deflated
//!
//! The series' last year ends 30 June 2027 and the June 2027 CPI does not exist yet, so
//! [`real_series`] returns `None` for it rather than an extrapolation. That is the honest
//! answer and it is [`deflator::DeflatorError::MissingIndex`] doing its job.

use std::collections::BTreeMap;

use deflator::{CpiSeries, Deflated};
use edfund_core::FiscalYear;

use crate::greenbook;
use crate::ledger::budget_analysis;

/// The year every real figure here is stated in unless a caller names another.
///
/// FY2026 because it is the last year the Bureau has published a June observation for, so it is
/// the newest year the whole series can be expressed in without extrapolating.
pub const BASE_YEAR: FiscalYear = FiscalYear(2026);

/// The department's own multiplicand in the FY2027 model, to the cent.
///
/// Secondary sources print it as $8,240, LSC as $8,241 in one greenbook and $8,242 in the next.
/// All three are this number, and `the_frozen_years_are_one_number` in
/// `tests/the_series_that_needed_a_deflator.rs` is the check.
pub const EXACT_FY2024_AVERAGE: f64 = crate::panel::AVERAGE_BASE_COST_PER_PUPIL;

/// What kind of quantity the year's figure is.
///
/// The node's `kind:` property says this parameter "changed kind at the regime boundary" and
/// names two kinds. There are three.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Basis {
    /// A statewide dollar figure set by act — one a legislator could amend and a reader look up.
    Legislated,
    /// No statewide per-pupil amount exists this year, because the formula has no place for one.
    Absent,
    /// The quotient of a district-by-district build-up. Nobody chose it; the cost inputs'
    /// reference year did.
    Computed,
}

/// One statement of an amount, with the analysis it comes from.
///
/// The quote is checked against the committed greenbook on every test run, so a row cannot drift
/// from its source without failing. It is the same discipline the corpus's `as_written` bindings
/// apply to prose, applied to a table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stated {
    /// The amount, or `None` where what LSC states is that there is no such amount.
    pub dollars: Option<f64>,
    /// The bill whose analysis says it, as [`greenbook::Greenbook::bill`] keys it.
    pub act: &'static str,
    /// LSC's own words, as they read with the PDF's line breaks collapsed.
    pub quote: &'static str,
    /// Whether LSC states it as an estimate made at enactment rather than as a settled amount.
    pub estimate: bool,
}

/// An act carrying a prior year's amount forward instead of setting one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Freeze {
    /// The year whose amount this year is held at.
    pub anchor: u16,
    /// The bill that freezes it.
    pub act: &'static str,
    /// LSC's own words.
    pub quote: &'static str,
}

/// One fiscal year of the statewide series.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Year {
    /// The fiscal year.
    pub fiscal_year: u16,
    /// What kind of quantity the year's figure is.
    pub basis: Basis,
    /// Where the act holds the year at an earlier year's amount.
    pub frozen: Option<Freeze>,
    /// LSC on this year, in the analysis of the act that appropriates for it.
    pub stated: Stated,
    /// A later source's figure, where what LSC published at enactment was an estimate.
    ///
    /// Under the Fair School Funding Plan the amount is not knowable when the act passes — it is
    /// the quotient of a build-up over data the department has not yet collected — so the
    /// biennium's own greenbook can only estimate it and the next biennium's states what it was.
    /// FY2022 was estimated at $7,202 and turned out to be $7,352, a 2.1% miss.
    pub realized: Option<Stated>,
}

impl Year {
    /// The amount this year's own sources state, before any freeze is resolved.
    #[must_use]
    pub fn own_dollars(&self) -> Option<f64> {
        self.realized
            .and_then(|r| r.dollars)
            .or(self.stated.dollars)
    }
}

/// The statewide series, oldest first.
///
/// FY1999 is here because it is the only year in the record whose amount was *measured* — the
/// average adjusted base expenditure of the 127 districts that met at least 20 of 27 performance
/// standards — rather than indexed forward from a measurement. Everything through FY2009 is that
/// figure plus compounding, which `the_indexation_reproduces_the_series` checks.
pub const SERIES: &[Year] = &[
    Year {
        fiscal_year: 1999,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(4_420.0),
            act: "hb94",
            quote: "This calculation resulted in a statewide base cost formula amount of $4,420 \
                    in FY 1999",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2002,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(4_814.0),
            act: "hb94",
            quote: "the updated statewide base cost formula amount is determined at $4,814 in \
                    FY 2002",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2003,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(4_949.0),
            act: "hb94",
            quote: "The base cost formula amount is $4,949 in FY 2003 by applying the same \
                    inflationary factor of 2.8 percent",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2004,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(5_058.0),
            act: "hb95",
            quote: "base cost formula amount of $5,058 in FY 2004 and $5,169 in FY 2005",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2005,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(5_169.0),
            act: "hb95",
            quote: "base cost formula amount of $5,058 in FY 2004 and $5,169 in FY 2005",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2006,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(5_283.0),
            act: "hb66",
            quote: "This results in a formula amount of $5,283 in FY 2006",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2007,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(5_403.0),
            act: "hb66",
            quote: "The resulting formula amount for FY 2007 is $5,403",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2008,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(5_565.0),
            act: "hb119",
            quote: "formula amount of $5,565 in FY 2008 and $5,732 in FY 2009",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2009,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(5_732.0),
            act: "hb119",
            quote: "formula amount of $5,565 in FY 2008 and $5,732 in FY 2009",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2010,
        basis: Basis::Absent,
        frozen: None,
        stated: Stated {
            dollars: None,
            act: "hb1",
            quote: "The new model is also largely based on the number of students in a district, \
                    but there is no specific formula amount per pupil",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2011,
        basis: Basis::Absent,
        frozen: None,
        stated: Stated {
            dollars: None,
            act: "hb1",
            quote: "The new model is also largely based on the number of students in a district, \
                    but there is no specific formula amount per pupil",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2012,
        basis: Basis::Absent,
        frozen: None,
        stated: Stated {
            dollars: None,
            act: "hb153",
            quote: "the method of allocating foundation aid in FY 2012 and FY 2013 starts with \
                    the per pupil foundation aid each district received in FY 2011",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2013,
        basis: Basis::Absent,
        frozen: None,
        stated: Stated {
            dollars: None,
            act: "hb153",
            quote: "the method of allocating foundation aid in FY 2012 and FY 2013 starts with \
                    the per pupil foundation aid each district received in FY 2011",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2014,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(5_745.0),
            act: "hb59",
            quote: "based on a per pupil formula amount of $5,745 in FY 2014 and $5,800 in \
                    FY 2015",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2015,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(5_800.0),
            act: "hb59",
            quote: "based on a per pupil formula amount of $5,745 in FY 2014 and $5,800 in \
                    FY 2015",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2016,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(5_900.0),
            act: "hb64",
            quote: "The budget increases the per-pupil formula amount from $5,800 in FY 2015 to \
                    $5,900 in FY 2016 and $6,000 in FY 2017",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2017,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(6_000.0),
            act: "hb64",
            quote: "The budget increases the per-pupil formula amount from $5,800 in FY 2015 to \
                    $5,900 in FY 2016 and $6,000 in FY 2017",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2018,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(6_010.0),
            act: "hb49",
            quote: "the per-pupil formula amount from $6,000 in FY 2017 to $6,010 in FY 2018 and \
                    $6,020 in FY 2019",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2019,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(6_020.0),
            act: "hb49",
            quote: "the per-pupil formula amount from $6,000 in FY 2017 to $6,010 in FY 2018 and \
                    $6,020 in FY 2019",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2020,
        basis: Basis::Legislated,
        frozen: Some(Freeze {
            anchor: 2019,
            act: "hb166",
            quote: "equals the formula amount for FY 2019 ($6,020)",
        }),
        stated: Stated {
            dollars: None,
            act: "hb166",
            quote: "For FY 2020 and FY 2021, the budget specifies that",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2021,
        basis: Basis::Legislated,
        frozen: Some(Freeze {
            anchor: 2019,
            act: "hb166",
            quote: "equals the formula amount for FY 2019 ($6,020)",
        }),
        stated: Stated {
            dollars: None,
            act: "hb166",
            quote: "For FY 2020 and FY 2021, the budget specifies that",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2022,
        basis: Basis::Computed,
        frozen: None,
        stated: Stated {
            dollars: Some(7_202.0),
            act: "hb110",
            quote: "is estimated to be $7,202 in FY 2022",
            estimate: true,
        },
        realized: Some(Stated {
            dollars: Some(7_352.0),
            act: "hb33",
            quote: "the statewide average base cost per pupil increases from $7,352 in FY 2023 \
                    to an estimated $8,241 in FY 2024",
            estimate: false,
        }),
    },
    Year {
        fiscal_year: 2023,
        basis: Basis::Computed,
        frozen: Some(Freeze {
            anchor: 2022,
            act: "hb110",
            quote: "The budget specifies that the statewide average base cost per pupil for \
                    FY 2023 will remain fixed at the FY 2022 amount",
        }),
        stated: Stated {
            dollars: Some(7_352.0),
            act: "hb33",
            quote: "the statewide average base cost per pupil increases from $7,352 in FY 2023 \
                    to an estimated $8,241 in FY 2024",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2024,
        basis: Basis::Computed,
        frozen: None,
        stated: Stated {
            dollars: Some(8_241.0),
            act: "hb33",
            quote: "the statewide average base cost per pupil increases from $7,352 in FY 2023 \
                    to an estimated $8,241 in FY 2024",
            estimate: true,
        },
        realized: Some(Stated {
            dollars: Some(8_242.0),
            act: "hb96",
            quote: "maintains the use of the FY 2024 statewide average amounts ($8,242 and \
                    $9,856, respectively) in FY 2026 and FY 2027",
            estimate: false,
        }),
    },
    Year {
        fiscal_year: 2025,
        basis: Basis::Computed,
        frozen: Some(Freeze {
            anchor: 2024,
            act: "hb33",
            quote: "The budget requires the two statewide average base cost per-pupil amounts \
                    for FY 2025 to remain fixed at the FY 2024 levels",
        }),
        stated: Stated {
            dollars: None,
            act: "hb33",
            quote: "The statewide average base cost per pupil and, therefore, the maximum \
                    scholarship amounts do not change in FY 2025",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2026,
        basis: Basis::Computed,
        frozen: Some(Freeze {
            anchor: 2024,
            act: "hb96",
            quote: "maintains the use of the FY 2024 statewide average amounts ($8,242 and \
                    $9,856, respectively) in FY 2026 and FY 2027",
        }),
        stated: Stated {
            dollars: None,
            act: "hb96",
            quote: "The budget retains the use of FY 2022 data for the various statewide average \
                    salary and",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2027,
        basis: Basis::Computed,
        frozen: Some(Freeze {
            anchor: 2024,
            act: "hb96",
            quote: "maintains the use of the FY 2024 statewide average amounts ($8,242 and \
                    $9,856, respectively) in FY 2026 and FY 2027",
        }),
        stated: Stated {
            dollars: None,
            act: "hb96",
            quote: "The budget retains the use of FY 2022 data for the various statewide average \
                    salary and",
            estimate: false,
        },
        realized: None,
    },
];

/// The per-pupil amount deducted from a district for a pupil it does not teach, across the four
/// years the district base cost did not exist.
///
/// **Ohio had no statewide per-pupil base cost from FY2010 to FY2013 and went on setting a
/// statewide per-pupil deduction the whole time.** The amount a district was charged for a pupil
/// who left for a community school, a STEM school, open enrolment or the Post-Secondary
/// Enrolment Options programme was legislated in each of those acts, and it fell in each one:
/// $5,732 in FY2009, when it was the base cost, then $5,718, $5,703 and $5,653.
///
/// The series is here because it is the closest thing to a continuous per-pupil figure across the
/// gap, and because what continued is worth naming: the formula stopped saying what a pupil costs
/// and did not stop saying what one is worth taking away.
pub const DEDUCTION: &[Year] = &[
    Year {
        fiscal_year: 2010,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(5_718.0),
            act: "hb1",
            quote: "The budget establishes the formula amount as $5,718 for FY 2010 and $5,703 \
                    for FY 2011",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2011,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(5_703.0),
            act: "hb1",
            quote: "The budget establishes the formula amount as $5,718 for FY 2010 and $5,703 \
                    for FY 2011",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2012,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(5_653.0),
            act: "hb153",
            quote: "students from the FY 2011 amount of $5,703 to $5,653",
            estimate: false,
        },
        realized: None,
    },
    Year {
        fiscal_year: 2013,
        basis: Basis::Legislated,
        frozen: None,
        stated: Stated {
            dollars: Some(5_653.0),
            act: "hb153",
            quote: "students from the FY 2011 amount of $5,703 to $5,653",
            estimate: false,
        },
        realized: None,
    },
];

/// What the per-pupil deduction lost in real terms across the four years the base cost did not
/// exist, as a fraction.
///
/// From $5,732 in FY2009, when the deduction *was* the base cost, to $5,653 in FY2013. The
/// nominal fall is $79 and small; the real fall is not, because the four years the formula had no
/// price for a pupil were four years of prices rising.
///
/// # Panics
///
/// If [`DEDUCTION`] is empty or its last year is outside the index, which the committed table and
/// [`CpiSeries::cpi_u_june`] rule out.
#[must_use]
pub fn deduction_real_change() -> f64 {
    let last = DEDUCTION.last().expect("the deduction series is not empty");
    let from = nominal(2009).expect("FY2009 carries the amount the deduction began at");
    let to = last
        .stated
        .dollars
        .expect("every deduction year states an amount");
    CpiSeries::cpi_u_june()
        .real_growth(from, FiscalYear(2009), to, FiscalYear(last.fiscal_year))
        .expect("both years are in the index")
        .value
}

/// The row for one fiscal year.
#[must_use]
pub fn year(fiscal_year: u16) -> Option<&'static Year> {
    SERIES.iter().find(|row| row.fiscal_year == fiscal_year)
}

/// The amount in force in a fiscal year, with any freeze resolved to the year it anchors on.
///
/// `None` where the year has no statewide amount at all, which is FY2010 through FY2013.
#[must_use]
pub fn nominal(fiscal_year: u16) -> Option<f64> {
    let mut at = year(fiscal_year)?;
    // Bounded rather than recursive: a freeze cycle would be a data error, and hanging is a worse
    // way to report one than returning nothing.
    for _ in 0..SERIES.len() {
        if let Some(dollars) = at.own_dollars() {
            return Some(dollars);
        }
        at = year(at.frozen?.anchor)?;
    }
    None
}

/// Every year the series covers, with its nominal amount and its amount in `base` dollars.
///
/// The real figure is `None` where the year has no amount, and also where the index has no
/// observation for it — which is FY2027, whose June has not happened.
#[must_use]
pub fn real_series(base: FiscalYear) -> Vec<(&'static Year, Option<f64>, Option<Deflated>)> {
    let cpi = CpiSeries::cpi_u_june();
    SERIES
        .iter()
        .map(|row| {
            let nominal = nominal(row.fiscal_year);
            let real = nominal
                .and_then(|dollars| cpi.convert(dollars, FiscalYear(row.fiscal_year), base).ok());
            (row, nominal, real)
        })
        .collect()
}

/// One year's amount in [`BASE_YEAR`] dollars.
#[must_use]
pub fn real(fiscal_year: u16) -> Option<f64> {
    real_series(BASE_YEAR)
        .into_iter()
        .find(|(row, _, _)| row.fiscal_year == fiscal_year)
        .and_then(|(_, _, deflated)| deflated)
        .map(|deflated| deflated.value)
}

/// Growth between two years as `(nominal, real)` fractions.
///
/// The pair is the point. Reporting either alone is the thing this module exists to stop.
#[must_use]
pub fn growth(from: u16, to: u16) -> Option<(f64, f64)> {
    let (from_nominal, to_nominal) = (nominal(from)?, nominal(to)?);
    let (from_real, to_real) = (real(from)?, real(to)?);
    Some((to_nominal / from_nominal - 1.0, to_real / from_real - 1.0))
}

/// The fiscal years with no statewide base cost per pupil, in order.
#[must_use]
pub fn gap() -> Vec<u16> {
    SERIES
        .iter()
        .filter(|row| row.basis == Basis::Absent)
        .map(|row| row.fiscal_year)
        .collect()
}

/// A run of consecutive years held at one earlier year's amount.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrozenRun {
    /// The year the run is held at.
    pub anchor: u16,
    /// The years held there, in order.
    pub years: Vec<u16>,
}

/// Every freeze in the series, oldest anchor first.
///
/// Three of them, and they lengthen: FY2019's amount held for 2 years, FY2022's for 1, FY2024's
/// for 3 and counting. A freeze is a nominal decision with a real effect, which [`erosion`]
/// measures.
#[must_use]
pub fn freezes() -> Vec<FrozenRun> {
    let mut by_anchor: BTreeMap<u16, Vec<u16>> = BTreeMap::new();
    for row in SERIES {
        if let Some(freeze) = row.frozen {
            by_anchor
                .entry(freeze.anchor)
                .or_default()
                .push(row.fiscal_year);
        }
    }
    by_anchor
        .into_iter()
        .map(|(anchor, years)| FrozenRun { anchor, years })
        .collect()
}

/// What a freeze has cost per pupil, in the dollars of the last year it can be measured in.
///
/// Returned as `(the anchor amount restated in the later year's dollars, the amount actually
/// paid, the fraction lost)`. A frozen amount is a falling one: nothing has to be cut for a
/// district to receive less.
#[must_use]
pub fn erosion(run: &FrozenRun) -> Option<(f64, f64, f64)> {
    let cpi = CpiSeries::cpi_u_june();
    let paid = nominal(run.anchor)?;
    // The last frozen year the index reaches. FY2027 is in the run and not in the index.
    let measurable = *run
        .years
        .iter()
        .rev()
        .find(|fy| cpi.point(FiscalYear(**fy)).is_some())?;
    let restated = cpi
        .convert(paid, FiscalYear(run.anchor), FiscalYear(measurable))
        .ok()?
        .value;
    Some((restated, paid, paid / restated - 1.0))
}

/// A stretch of the series over which the figures are the same kind of quantity.
#[derive(Debug, Clone, PartialEq)]
pub struct Era {
    /// What the era's figures are.
    pub basis: Basis,
    /// What the acts of the era call the amount.
    pub name: &'static str,
    /// First and last fiscal year.
    pub span: (u16, u16),
    /// Nominal and real change across the era.
    pub change: (f64, f64),
}

/// The three eras in which a cross-year comparison compares like with like.
///
/// `parameter/base-cost-per-pupil` warns that "comparing a pre-2022 base cost to a post-2022 one
/// compares a policy decision to a computed output". Deflation removes the price objection to
/// such a comparison and leaves that one standing, so the eras are reported separately.
///
/// # Panics
///
/// If a named endpoint has no amount, which would mean [`SERIES`] and this list disagree.
#[must_use]
pub fn eras() -> Vec<Era> {
    [
        (Basis::Legislated, "base cost formula amount", 2002, 2009),
        (
            Basis::Legislated,
            "opportunity grant formula amount",
            2014,
            2021,
        ),
        (
            Basis::Computed,
            "statewide average base cost per pupil",
            2022,
            2026,
        ),
    ]
    .into_iter()
    .map(|(basis, name, first, last)| Era {
        basis,
        name,
        span: (first, last),
        change: growth(first, last).expect("both endpoints of a named era carry an amount"),
    })
    .collect()
}

/// The year the series is worth most in constant dollars, as `(fiscal year, amount)`.
///
/// # Panics
///
/// If no year in [`SERIES`] can be deflated, which the committed index rules out.
#[must_use]
pub fn peak() -> (u16, f64) {
    real_series(BASE_YEAR)
        .into_iter()
        .filter_map(|(row, _, deflated)| deflated.map(|d| (row.fiscal_year, d.value)))
        .max_by(|left, right| left.1.total_cmp(&right.1))
        .expect("the series has at least one deflatable year")
}

/// The flattened text of the analysis an act's greenbook is, for checking a [`Stated::quote`].
///
/// The 136th General Assembly's is committed on its own rather than in the twelve-act extract —
/// see [`crate::ledger::budget_analysis::GREENBOOK`] — so it is reached separately here. A caller
/// that forgot would find `hb96` absent and conclude the quote was wrong.
///
/// # Panics
///
/// If `act` names no committed greenbook.
#[must_use]
pub fn analysis(act: &str) -> String {
    if act == "hb96" {
        return budget_analysis::GREENBOOK
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
    }
    greenbook::greenbook(act).flat()
}
