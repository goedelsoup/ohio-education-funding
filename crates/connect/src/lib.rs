//! Retrieval and extraction for the Ohio education funding corpus.
//!
//! The side-effecting half of the domain computer. It fetches the department's publications,
//! reads them with [`spreadsheet`], and writes the CSV fixtures the calculator crates compute
//! against.
//!
//! # The connector/calculator split
//!
//! [PHASES.md](../../../.yidam/.vendor/prelude/PHASES.md) draws the line: connectors are
//! side-effecting and may fail, calculators are pure and deterministic. Everything here can
//! fail — a URL moves, a workbook changes shape, the network is not there. Nothing here
//! computes a funding figure. That is why an extraction commit and an assessment commit have
//! different meanings in the log, and why they are not mixed.
//!
//! # Refresh and verification check each other
//!
//! Extraction can be re-run freely *because* the findings are pinned downstream. A source
//! revision that moves the guarantee count, the millage floor count, or a scenario result fails
//! a test in [`foundation`](../../foundation/) or [`dispersion`](../../dispersion/) rather than
//! passing silently into the corpus. The digest manifest in [`cache`] closes the other
//! direction: it records exactly which published file a fixture was built from, so a changed
//! answer can always be traced to a changed source.
//!
//! ```text
//! edfund-connect list                 what is retrievable, and how far each connector got
//! edfund-connect fetch <source>       download one source into .cache/sources
//! edfund-connect rebuild              regenerate the committed fixtures
//! edfund-connect verify               check cached sources against the committed digests
//! edfund-connect cpi                  check the deflator series against the Bureau's file
//! ```

#![forbid(unsafe_code)]

pub mod cache;
pub mod conventions;
pub mod cpi;
pub mod fixtures;
pub mod forecast;
pub mod html;
pub mod index;
pub mod registry;
pub mod sha256;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use cache::FetchError;
use registry::Source;
use spreadsheet::AnyWorkbook;

pub use fixtures::{CPI_FIXTURE, FY27_FIXTURE, GRADE_BANDS_FIXTURE, PROFILE_FIXTURE};

pub use registry::{connector, source, Connector, Format, Status, CONNECTORS};

/// What happened to one fixture during a rebuild.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rebuilt {
    /// Regenerated from a cached source.
    Written {
        /// Path relative to the repository root.
        path: String,
        /// How many data rows were written.
        rows: usize,
    },
    /// Left alone, because its source is not available.
    ///
    /// Reported rather than silently omitted: a rebuild that quietly regenerates two of three
    /// fixtures reads as a full refresh in a commit message, and the third then carries an
    /// older vintage than the others without saying so.
    Skipped {
        /// Path relative to the repository root.
        path: String,
        /// Why it was not rebuilt.
        reason: String,
    },
}

/// Something went wrong rebuilding a fixture.
#[derive(Debug)]
pub enum RebuildError {
    /// A source could not be read.
    Source(FetchError),
    /// A workbook could not be parsed.
    Workbook(spreadsheet::OpenError),
    /// A five-year forecast filing could not be parsed.
    Forecast(forecast::ForecastError),
    /// A fixture could not be written.
    Io(std::io::Error),
    /// A published layout moved and a named column is gone.
    Layout(String),
}

impl core::fmt::Display for RebuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Source(cause) => write!(f, "{cause}"),
            Self::Workbook(cause) => write!(f, "{cause}"),
            Self::Forecast(cause) => write!(f, "{cause}"),
            Self::Io(cause) => write!(f, "{cause}"),
            Self::Layout(cause) => f.write_str(cause),
        }
    }
}

impl std::error::Error for RebuildError {}

impl From<FetchError> for RebuildError {
    fn from(cause: FetchError) -> Self {
        Self::Source(cause)
    }
}

impl From<forecast::ForecastError> for RebuildError {
    fn from(cause: forecast::ForecastError) -> Self {
        Self::Forecast(cause)
    }
}

impl From<spreadsheet::OpenError> for RebuildError {
    fn from(cause: spreadsheet::OpenError) -> Self {
        Self::Workbook(cause)
    }
}

impl From<std::io::Error> for RebuildError {
    fn from(cause: std::io::Error) -> Self {
        Self::Io(cause)
    }
}

/// Open a cached source as a workbook, converting a legacy `.xls` first if need be.
///
/// # Errors
///
/// Returns [`RebuildError`] if the source is not cached, cannot be converted, or is not a
/// readable workbook.
pub fn open_workbook(root: &Path, source: &Source) -> Result<AnyWorkbook, RebuildError> {
    let path = cache::cached_path(root, source);
    if !path.exists() {
        return Err(RebuildError::Source(FetchError::NotCached {
            key: source.key.to_string(),
            path,
        }));
    }
    // The format is decided by the file's leading bytes, not by `source.format` or the
    // extension. The department has published `.xls` files that were XLSX and `.xls` files that
    // were HTML tables, so the declared format is a hint and the magic number is the fact.
    Ok(spreadsheet::open(std::fs::read(path)?)?)
}

/// Rebuild every committed fixture from the cached sources.
///
/// Reads only what is already in the cache — refreshing is a separate, explicit act, so that
/// regenerating a fixture never silently pulls a revised publication.
///
/// # Errors
///
/// Returns [`RebuildError`] if a source is missing from the cache or will not parse.
pub fn rebuild(root: &Path) -> Result<Vec<Rebuilt>, RebuildError> {
    let fy27 = source("fy27-calculator").expect("registered").1;
    let profile = source("cupp-fy24").expect("registered").1;

    let fy27_book = open_workbook(root, fy27)?;
    let profile_book = open_workbook(root, profile)?;
    let profile_rows = profile_book.rows("District Data")?;

    let model = fixtures::build_fy27_model(&fixtures::Fy27Sheets {
        base_cost_rows: &fy27_book.rows("Base_Cost")?,
        summary_rows: &fy27_book.rows("Summary_SFPR")?,
        adm_rows: &fy27_book.rows("ADM Data")?,
        profile_rows: &profile_rows,
        detail_rows: &fy27_book.rows("Detail_SFPR")?,
        capacity_rows: &fy27_book.rows("Local_Capacity")?,
        special_education_rows: &fy27_book.rows("Special Edu")?,
        dpia_rows: &fy27_book.rows("DPIA")?,
        cte_rows: &fy27_book.rows("CTE")?,
        el_rows: &fy27_book.rows("EL")?,
        gifted_rows: &fy27_book.rows("Gifted")?,
        targeted_assistance_rows: &fy27_book.rows("Targeted_Assistance")?,
        performance_rows: &fy27_book.rows("Performance Supplement")?,
        growth_rows: &fy27_book.rows("Base_Enrollment Growth")?,
        transportation_rows: &fy27_book.rows("Transportation")?,
        prek_sped_rows: &fy27_book.rows("PSS_pec_Ed")?,
    });
    let extract = fixtures::build_profile_extract(&profile_rows);

    let mut out = vec![
        Rebuilt::Written {
            path: fixtures::FY27_FIXTURE.to_string(),
            rows: fixtures::write_csv(
                &root.join(fixtures::FY27_FIXTURE),
                fixtures::FY27_HEADER,
                &model,
            )?,
        },
        Rebuilt::Written {
            path: fixtures::PROFILE_FIXTURE.to_string(),
            rows: fixtures::write_csv(
                &root.join(fixtures::PROFILE_FIXTURE),
                fixtures::PROFILE_HEADER,
                &extract,
            )?,
        },
    ];

    let achievement = source("achievement-district-2425").expect("registered").1;
    let spending = source("spend-per-pupil-2425").expect("registered").1;
    let expanded = source("expanded-list-fy25").expect("registered").1;
    let value_added = source("va-district-details-2425").expect("registered").1;
    let details = source("district-details-2425").expect("registered").1;

    let achievement_book = open_workbook(root, achievement)?;
    let spending_book = open_workbook(root, spending)?;
    let expanded_book = open_workbook(root, expanded)?;
    let value_added_book = open_workbook(root, value_added)?;
    let details_book = open_workbook(root, details)?;

    // Excel truncates a sheet name at 31 characters, which is why the weighted sheet is
    // "Expenditure per Equivalent Pup" and not "...Pupil". Spelling it out is not an option.
    let report_card = fixtures::build_report_card_extract(
        &achievement_book.rows("Performance_Index")?,
        &spending_book.rows("DISTRICT_SPENDING_PER_PUPIL")?,
        &expanded_book.rows("Expenditure per Equivalent Pup")?,
        &expanded_book.rows("Expenditure per Pupil")?,
        &value_added_book.rows("OVERALL_VALUE_ADDED_OVERVIEW")?,
        &value_added_book.rows("OVERALL_VA_1_YEAR_GAINS")?,
        &details_book.rows("District_Details")?,
    );
    let unweighted_rows = expanded_book.rows("Expenditure per Pupil")?;
    out.push(Rebuilt::Written {
        path: fixtures::FUNCTIONS_FIXTURE.to_string(),
        rows: fixtures::write_csv(
            &root.join(fixtures::FUNCTIONS_FIXTURE),
            fixtures::FUNCTIONS_HEADER,
            &fixtures::build_function_extract(&unweighted_rows),
        )?,
    });
    out.push(Rebuilt::Written {
        path: fixtures::REPORT_CARD_FIXTURE.to_string(),
        rows: fixtures::write_csv(
            &root.join(fixtures::REPORT_CARD_FIXTURE),
            fixtures::REPORT_CARD_HEADER,
            &report_card,
        )?,
    });

    // The same report card at building grain. A separate fixture rather than more columns,
    // because it is a different unit: the funding formula pays agencies and the accountability
    // system identifies schools, and one file cannot be keyed on both.
    let building_achievement = source("achievement-building-2425").expect("registered").1;
    let building_details = source("building-details-2425").expect("registered").1;
    out.push(
        match (|| -> Result<Vec<Vec<String>>, String> {
            let achievement =
                open_workbook(root, building_achievement).map_err(|e| e.to_string())?;
            let details = open_workbook(root, building_details).map_err(|e| e.to_string())?;
            fixtures::build_buildings(
                &achievement
                    .rows("Performance_Index")
                    .map_err(|e| e.to_string())?,
                &details
                    .rows("Building_Details")
                    .map_err(|e| e.to_string())?,
            )
        })() {
            Ok(rows) => Rebuilt::Written {
                path: fixtures::BUILDING_FIXTURE.to_string(),
                rows: fixtures::write_csv(
                    &root.join(fixtures::BUILDING_FIXTURE),
                    fixtures::BUILDING_HEADER,
                    &rows,
                )?,
            },
            Err(reason) => Rebuilt::Skipped {
                path: fixtures::BUILDING_FIXTURE.to_string(),
                reason,
            },
        },
    );

    // Who is actually in the accountability system. Three lists, one fixture, because a
    // building's status is the union of what it appears on and the interesting rows are the ones
    // on more than one.
    out.push(
        match (|| -> Result<Vec<Vec<String>>, String> {
            let mut books = Vec::new();
            for status in ["csi", "tsi", "atsi"] {
                let entry = source(&format!("{status}-identified-2026"))
                    .expect("registered")
                    .1;
                books.push((
                    status,
                    open_workbook(root, entry)
                        .map_err(|e| e.to_string())?
                        .rows("Sheet1")
                        .map_err(|e| e.to_string())?,
                ));
            }
            let lists: Vec<(&str, &[Vec<String>])> =
                books.iter().map(|(s, r)| (*s, r.as_slice())).collect();
            fixtures::build_identified(&lists)
        })() {
            Ok(rows) => Rebuilt::Written {
                path: fixtures::IDENTIFIED_FIXTURE.to_string(),
                rows: fixtures::write_csv(
                    &root.join(fixtures::IDENTIFIED_FIXTURE),
                    fixtures::IDENTIFIED_HEADER,
                    &rows,
                )?,
            },
            Err(reason) => Rebuilt::Skipped {
                path: fixtures::IDENTIFIED_FIXTURE.to_string(),
                reason,
            },
        },
    );

    // The fifth fixture, and the one that until now needed LibreOffice. The department still
    // publishes October headcount in the pre-2007 format; `spreadsheet` reads it natively.
    let enrollment = source("enrollment-fy24").expect("registered").1;
    out.push(match open_workbook(root, enrollment) {
        Ok(book) => {
            let headcount = book.rows("fy24_hdcnt_dist")?;
            Rebuilt::Written {
                path: fixtures::GRADE_BANDS_FIXTURE.to_string(),
                rows: fixtures::write_csv(
                    &root.join(fixtures::GRADE_BANDS_FIXTURE),
                    fixtures::GRADE_BANDS_HEADER,
                    &fixtures::build_grade_bands(&headcount, &profile_rows),
                )?,
            }
        }
        Err(cause) => Rebuilt::Skipped {
            path: fixtures::GRADE_BANDS_FIXTURE.to_string(),
            reason: cause.to_string(),
        },
    });

    let cpi_source = source("cpi-u-all-items").expect("registered").1;
    out.push(match cache::read_cached(root, cpi_source) {
        Ok(bytes) => {
            let extract = fixtures::build_cpi_extract(
                &String::from_utf8_lossy(&bytes),
                cpi::ALL_ITEMS_NSA,
                cpi::JUNE,
            );
            let path = root.join(fixtures::CPI_FIXTURE);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let rows = extract.lines().count().saturating_sub(1);
            std::fs::write(&path, extract)?;
            Rebuilt::Written {
                path: fixtures::CPI_FIXTURE.to_string(),
                rows,
            }
        }
        Err(cause) => Rebuilt::Skipped {
            path: fixtures::CPI_FIXTURE.to_string(),
            reason: cause.to_string(),
        },
    });

    // The financial panel: two filings, three years of actuals each, tiling FY2020 to FY2025.
    // Both must be present — half the panel is worse than none, because a series with a hole in
    // it silently becomes a series about the years that happened to be retrievable.
    let filings: Result<Vec<_>, RebuildError> =
        ["five-year-forecast-fy23", "five-year-forecast-fy26"]
            .into_iter()
            .map(|key| {
                let filing = source(key).expect("registered").1;
                let bytes = cache::read_cached(root, filing)?;
                Ok(forecast::parse(&String::from_utf8_lossy(&bytes))?)
            })
            .collect();
    out.push(match filings {
        Ok(filings) => Rebuilt::Written {
            path: fixtures::FINANCE_FIXTURE.to_string(),
            rows: fixtures::write_csv(
                &root.join(fixtures::FINANCE_FIXTURE),
                fixtures::FINANCE_HEADER,
                &fixtures::build_finance_extract(&filings),
            )?,
        },
        Err(cause) => Rebuilt::Skipped {
            path: fixtures::FINANCE_FIXTURE.to_string(),
            reason: cause.to_string(),
        },
    });

    // The local side: taxable value by class and real property taxes charged, four tax years.
    //
    // Two would give a level and a change, which is what the tax page reads. Four are here for a
    // different reason: Ohio's 88 counties reappraise or update on a staggered three-year cycle,
    // so TY2022 through TY2024 contains exactly one valuation event per county — and TY2021 makes
    // the earliest of those a change rather than a level. That window is what lets each district's
    // reappraisal be measured against its own quiet years instead of a statewide average. See
    // `regime_diff::recognized_valuation`.
    let abstracts: Result<Vec<_>, RebuildError> = [
        (2021, "sd1-ty2021"),
        (2022, "sd1-ty2022"),
        (2023, "sd1-ty2023"),
        (2024, "sd1-ty2024"),
    ]
    .into_iter()
    .map(|(tax_year, key)| {
        let book = open_workbook(root, source(key).expect("registered").1)?;
        let excluding = sheet_by_prefix(&book, "ExJVS")?;
        let including = sheet_by_prefix(&book, "SD1DAT")?;
        Ok((tax_year, excluding, including))
    })
    .collect();
    out.push(match abstracts {
        Ok(abstracts) => {
            let years: Vec<fixtures::Sd1Year<'_>> = abstracts
                .iter()
                .map(|(tax_year, excluding, including)| fixtures::Sd1Year {
                    tax_year: *tax_year,
                    excluding_jvsd: excluding,
                    including_jvsd: including,
                })
                .collect();
            Rebuilt::Written {
                path: fixtures::SD1_FIXTURE.to_string(),
                rows: fixtures::write_csv(
                    &root.join(fixtures::SD1_FIXTURE),
                    fixtures::SD1_HEADER,
                    &fixtures::build_sd1_extract(&years),
                )?,
            }
        }
        Err(cause) => Rebuilt::Skipped {
            path: fixtures::SD1_FIXTURE.to_string(),
            reason: cause.to_string(),
        },
    });

    // The Revised Code. Skipped rather than fatal when a section is not cached, for the same
    // reason as F-33 below: it feeds prose and verification, and an absent copy should cost those
    // rather than the whole rebuild.
    //
    // Skipped as a whole, though — all fifteen sections or none, for the reason the opinions are
    // all four or none. "Skip the fixture when a section is missing" and "write the sections that
    // happened to be there" are different behaviours, and this collected into a `Vec` and guarded
    // on `is_empty`, which is the second one: fourteen cached sections rewrote the committed
    // extract as a fourteen-section file and reported it as a success. A statute the corpus cites
    // that is silently absent from the extract is worse than an extract that is not there, because
    // the first looks like an answer.
    let statutes: Result<Vec<fixtures::Record>, String> = registry::OHIO_LAWS_SECTIONS
        .iter()
        .map(|source| {
            let page = cache::read_cached(root, source).map_err(|cause| cause.to_string())?;
            let page = String::from_utf8_lossy(&page);
            // The key is `rc-3317-013`; the section it cites is `3317.013`.
            let section = source.key.trim_start_matches("rc-").replacen('-', ".", 1);
            // `None` means neither landmark was found, which says the page is no longer the page
            // the parser was written against — a different failure from an absent file, and one
            // worth naming rather than counting.
            fixtures::parse_statute(&section, &page).ok_or_else(|| {
                format!(
                    "{section} did not parse; {} is no longer the page it was",
                    source.url
                )
            })
        })
        .collect();
    out.push(match statutes {
        Ok(statutes) => Rebuilt::Written {
            path: fixtures::STATUTE_FIXTURE.to_string(),
            rows: fixtures::write_text(
                &root.join(fixtures::STATUTE_FIXTURE),
                &fixtures::build_records(
                    "Ohio Revised Code, the sections this corpus cites. One record per section.",
                    &statutes,
                ),
            )?,
        },
        Err(reason) => Rebuilt::Skipped {
            path: fixtures::STATUTE_FIXTURE.to_string(),
            reason,
        },
    });

    // The enacted budget act. Skipped rather than fatal on two counts: the PDF may not be cached,
    // and `pdftotext` may not be installed — see `cache::pdf_text` for why that is a weaker
    // guarantee than the one `curl` gets. Either way the committed extract stands and the rebuild
    // says what it did not do.
    let enacted = source("hb96-final-analysis").expect("registered").1;
    out.push(
        match cache::pdf_text(root, enacted)
            .map_err(RebuildError::from)
            .and_then(|text| {
                fixtures::extract_school_funding(&text).ok_or_else(|| {
                    RebuildError::Io(std::io::Error::other(
                        "the final analysis no longer contains its school funding heading",
                    ))
                })
            }) {
            Ok(text) => Rebuilt::Written {
                path: fixtures::ENACTED_FIXTURE.to_string(),
                rows: fixtures::write_text(&root.join(fixtures::ENACTED_FIXTURE), &text)?,
            },
            Err(cause) => Rebuilt::Skipped {
                path: fixtures::ENACTED_FIXTURE.to_string(),
                reason: cause.to_string(),
            },
        },
    );

    // The department's redbook. Committed whole rather than sliced: unlike the final analysis,
    // which legislates on everything from Medicaid to liquor permits, a redbook is about one
    // agency and there is nothing in it to cut away.
    let redbook = source("hb96-edu-redbook").expect("registered").1;
    out.push(match cache::pdf_text(root, redbook) {
        Ok(text) => Rebuilt::Written {
            path: fixtures::REDBOOK_FIXTURE.to_string(),
            rows: fixtures::write_text(&root.join(fixtures::REDBOOK_FIXTURE), text.trim())?,
        },
        Err(cause) => Rebuilt::Skipped {
            path: fixtures::REDBOOK_FIXTURE.to_string(),
            reason: cause.to_string(),
        },
    });

    // The DeRolph opinions. One record per case, same shape as the statute extract, so the two
    // sources a legal claim can rest on read alike.
    //
    // All four or none, which is why this collects into a `Result` rather than filtering the
    // failures out. A cache holding three of the opinions would otherwise write a fixture about
    // the cases that happened to be retrievable and report it as a success — the hazard the
    // finance panel refuses above, and no better for a litigation record: a reader who finds
    // DeRolph II absent cannot tell whether the case did not exist or the PDF did not download.
    // Skipping the whole fixture leaves the last good one committed and says why.
    let opinions: Result<Vec<fixtures::Record>, cache::FetchError> =
        registry::connector("ohio-courts")
            .expect("registered")
            .sources
            .iter()
            .map(|src| {
                let text = cache::pdf_text(root, src)?;
                let body = text.trim().to_string();
                Ok(fixtures::Record {
                    id: src.key.to_string(),
                    title: src.title.unwrap_or(src.key).to_string(),
                    date: fixtures::decided_on(&body),
                    source: src.url.to_string(),
                    body,
                })
            })
            .collect();
    out.push(match opinions {
        Ok(opinions) => Rebuilt::Written {
            path: fixtures::OPINIONS_FIXTURE.to_string(),
            rows: fixtures::write_text(
                &root.join(fixtures::OPINIONS_FIXTURE),
                &fixtures::build_records(
                    "DeRolph v. State, the four opinions this corpus cites. One record per case.",
                    &opinions,
                ),
            )?,
        },
        Err(cause) => Rebuilt::Skipped {
            path: fixtures::OPINIONS_FIXTURE.to_string(),
            reason: cause.to_string(),
        },
    });

    // Census F-33. Skipped rather than fatal when the workbook is not cached: it is the one
    // source in the registry that nothing else depends on, so an absent copy should cost the
    // interstate comparison and nothing else.
    let f33 = source("f33-fy2022").expect("registered").1;
    out.push(match open_workbook(root, f33) {
        Ok(book) => match book.rows("elsec22t") {
            Ok(rows) => Rebuilt::Written {
                path: fixtures::F33_FIXTURE.to_string(),
                rows: fixtures::write_csv(
                    &root.join(fixtures::F33_FIXTURE),
                    fixtures::F33_HEADER,
                    &fixtures::build_f33_states(&rows, "the FY2022 F-33 state table")
                        .map_err(RebuildError::Layout)?,
                )?,
            },
            Err(cause) => Rebuilt::Skipped {
                path: fixtures::F33_FIXTURE.to_string(),
                reason: cause.to_string(),
            },
        },
        Err(cause) => Rebuilt::Skipped {
            path: fixtures::F33_FIXTURE.to_string(),
            reason: cause.to_string(),
        },
    });

    // The same table two years later, which is the year White Paper 015 quotes. Kept as its own
    // fixture: FY2022 is what the corpus's published interstate figures were computed from.
    let f33_2024 = source("f33-fy2024").expect("registered").1;
    out.push(match open_workbook(root, f33_2024) {
        Ok(book) => match book.rows("elsec24t") {
            Ok(rows) => Rebuilt::Written {
                path: fixtures::F33_FY2024_FIXTURE.to_string(),
                rows: fixtures::write_csv(
                    &root.join(fixtures::F33_FY2024_FIXTURE),
                    fixtures::F33_HEADER,
                    &fixtures::build_f33_states(&rows, "the FY2024 F-33 state table")
                        .map_err(RebuildError::Layout)?,
                )?,
            },
            Err(cause) => Rebuilt::Skipped {
                path: fixtures::F33_FY2024_FIXTURE.to_string(),
                reason: cause.to_string(),
            },
        },
        Err(cause) => Rebuilt::Skipped {
            path: fixtures::F33_FY2024_FIXTURE.to_string(),
            reason: cause.to_string(),
        },
    });

    // The same survey per district, which needs two archives: NCES's keying of the F-33, and the
    // CCD directory that carries the `LEAID`-to-IRN join. Committed since the national comparison
    // was built and produced by nothing until now — the registry declared the fixture, the digest
    // manifest pinned its sources, and the derivation between them lived only in prose.
    out.push(match f33_districts(root) {
        Ok(rows) => Rebuilt::Written {
            path: fixtures::F33_DISTRICTS_FIXTURE.to_string(),
            rows: fixtures::write_csv(
                &root.join(fixtures::F33_DISTRICTS_FIXTURE),
                fixtures::F33_DISTRICTS_HEADER,
                &rows,
            )?,
        },
        Err(reason) => Rebuilt::Skipped {
            path: fixtures::F33_DISTRICTS_FIXTURE.to_string(),
            reason,
        },
    });

    // The same survey again, Ohio only, across every year the archive publishes. The national
    // file answers whether Ohio is unusual; this one answers how Ohio changed, and the two need
    // different populations rather than different filters over one fixture.
    out.push(match f33_ohio_panel(root) {
        Ok(rows) => Rebuilt::Written {
            path: fixtures::F33_OHIO_PANEL_FIXTURE.to_string(),
            rows: fixtures::write_csv(
                &root.join(fixtures::F33_OHIO_PANEL_FIXTURE),
                fixtures::F33_OHIO_PANEL_HEADER,
                &rows,
            )?,
        },
        Err(reason) => Rebuilt::Skipped {
            path: fixtures::F33_OHIO_PANEL_FIXTURE.to_string(),
            reason,
        },
    });

    // Eleven Octobers of the child nutrition report, aggregated from school sites to sponsors.
    out.push(match mr81_panel(root) {
        Ok(rows) => Rebuilt::Written {
            path: fixtures::MR81_FIXTURE.to_string(),
            rows: fixtures::write_csv(
                &root.join(fixtures::MR81_FIXTURE),
                fixtures::MR81_HEADER,
                &rows,
            )?,
        },
        Err(reason) => Rebuilt::Skipped {
            path: fixtures::MR81_FIXTURE.to_string(),
            reason,
        },
    });

    // School districts across legislative seats. The last extraction because it is the only one
    // that depends on another fixture's contents rather than only on a cached publication: the
    // panel it apportions is the FY2027 model's 609 districts.
    let panel: BTreeSet<String> = model
        .iter()
        .filter_map(|row| row.first().cloned())
        .collect();
    out.push(match legislative_crosswalk(root, &panel) {
        Ok(rows) => Rebuilt::Written {
            path: fixtures::CROSSWALK_FIXTURE.to_string(),
            rows: fixtures::write_csv(
                &root.join(fixtures::CROSSWALK_FIXTURE),
                fixtures::CROSSWALK_HEADER,
                &rows,
            )?,
        },
        Err(reason) => Rebuilt::Skipped {
            path: fixtures::CROSSWALK_FIXTURE.to_string(),
            reason,
        },
    });

    Ok(out)
}

/// Read the sole member of a cached zip whose name ends in `suffix`.
///
/// By suffix rather than by name: the CCD archive holds the same directory as both `.csv` and
/// `.sas7bdat`, and its filename carries a release date (`ccd_lea_029_2223_w_1a_083023.csv`) that
/// changes when the file is reposted.
fn zip_member(root: &Path, source: &Source, suffix: &str) -> Result<String, String> {
    let bytes = cache::read_cached(root, source).map_err(|cause| cause.to_string())?;
    let archive = spreadsheet::zip::Archive::open(bytes)
        .map_err(|cause| format!("{}: {cause}", source.key))?;
    let named: Vec<&str> = archive
        .entries()
        .iter()
        .map(|e| e.name.as_str())
        .filter(|name| name.ends_with(suffix))
        .collect();
    let [name] = named[..] else {
        return Err(format!(
            "{} holds {} members ending in {suffix}, expected exactly one",
            source.key,
            named.len()
        ));
    };
    archive
        .read_to_string(name)
        .map_err(|cause| format!("{}: {cause}", source.key))
}

/// Block populations from the PL 94-171 release: block code to total and under-18 count.
///
/// The three members are read and dropped one at a time. Uncompressed they are 174, 154 and 157
/// megabytes, and holding all three as strings to join them would cost half a gigabyte for a
/// result that is two integers per block.
fn block_population(root: &Path, source: &Source) -> Result<BTreeMap<String, (i64, i64)>, String> {
    let blocks = fixtures::pl_blocks(&zip_member(root, source, "geo2020.pl")?);
    let people = fixtures::pl_counts(&zip_member(root, source, "000012020.pl")?);
    let adults = fixtures::pl_counts(&zip_member(root, source, "000022020.pl")?);
    Ok(blocks
        .into_iter()
        .map(|(record, block)| {
            let total = people.get(&record).copied().unwrap_or(0);
            // Under 18 is the residual: the release publishes the total and the 18-and-over
            // count, and never the children directly.
            let children = total - adults.get(&record).copied().unwrap_or(0);
            (block, (total, children))
        })
        .collect())
}

/// The legislative-district crosswalk, from four census archives and the CCD directory.
///
/// `panel` is the funding model's IRNs, which is why this runs after the FY2027 model rather than
/// beside the other archive extractions: the crosswalk apportions that model across seats, and a
/// school district the model does not carry has nothing to apportion.
fn legislative_crosswalk(
    root: &Path,
    panel: &BTreeSet<String>,
) -> Result<Vec<Vec<String>>, String> {
    let blocks = source("baf-2020-oh").expect("registered").1;
    let house = source("sldl24-bef").expect("registered").1;
    let senate = source("sldu24-bef").expect("registered").1;
    let census = source("pl94-171-2020-oh").expect("registered").1;
    let directory = source("ccd-lea-directory-2223").expect("registered").1;

    let population = block_population(root, census)?;
    fixtures::build_legislative_crosswalk(&fixtures::Crosswalk {
        population: &population,
        school_districts: &zip_member(root, blocks, "_SDUNI.txt")?,
        house: &zip_member(root, house, "39_OH_SLDL24.txt")?,
        senate: &zip_member(root, senate, "39_OH_SLDU24.txt")?,
        directory: &zip_member(root, directory, ".csv")?,
        panel,
    })
}

/// The per-district F-33 panel, from the survey archive and the directory that keys it to IRN.
fn f33_districts(root: &Path) -> Result<Vec<Vec<String>>, String> {
    let survey = source("sdf22-districts").expect("registered").1;
    let directory = source("ccd-lea-directory-2223").expect("registered").1;
    fixtures::build_f33_districts(
        &zip_member(root, survey, ".txt")?,
        &zip_member(root, directory, ".csv")?,
    )
}

/// Every year of the survey this repository holds, oldest first.
///
/// FY2014 is absent. It is not a naming problem: `sdf14_1a.zip`, `sdf141a.zip`, `sdf14_2a.zip`
/// and `sdf14_1a_rev.zip` all 404 while FY2013 and FY2015 answer under two of those patterns.
/// The gap is recorded here rather than papered over by interpolation, because a panel with a
/// silently missing year reads as a series and is not one.
const F33_PANEL_YEARS: &[(u16, &str)] = &[
    (2009, "sdf09-districts"),
    (2010, "sdf10-districts"),
    (2011, "sdf11-districts"),
    (2012, "sdf12-districts"),
    (2013, "sdf13-districts"),
    (2015, "sdf15-districts"),
    (2016, "sdf16-districts"),
    (2017, "sdf17-districts"),
    (2018, "sdf18-districts"),
    (2019, "sdf19-districts"),
    (2020, "sdf20-districts"),
    (2021, "sdf21-districts"),
    (2022, "sdf22-districts"),
];

/// Ohio across every year of the survey, from ten archives and one directory.
fn f33_ohio_panel(root: &Path) -> Result<Vec<Vec<String>>, String> {
    let directory = source("ccd-lea-directory-2223").expect("registered").1;
    let directory = zip_member(root, directory, ".csv")?;

    // Read every archive first: the member text has to outlive the borrow the builder takes, and
    // 30 MB of survey held at once is cheaper than a builder that takes a closure.
    let mut surveys = Vec::new();
    for (fiscal_year, key) in F33_PANEL_YEARS {
        let survey = source(key).expect("registered").1;
        surveys.push((*fiscal_year, zip_member(root, survey, ".txt")?));
    }
    let years: Vec<fixtures::PanelYear<'_>> = surveys
        .iter()
        .map(|(fiscal_year, survey)| fixtures::PanelYear {
            fiscal_year: *fiscal_year,
            survey,
        })
        .collect();
    fixtures::build_f33_ohio_panel(&years, &directory)
}

/// Every October of MR-81 this repository reads: the sponsor-centric era, unbroken.
const MR81_YEARS: &[u16] = &[
    2001, 2002, 2003, 2004, 2005, 2006, 2007, 2008, 2009, 2010, 2011,
];

/// The MR-81 sponsor panel, from eleven delimited reports.
fn mr81_panel(root: &Path) -> Result<Vec<Vec<String>>, String> {
    let mut reports = Vec::new();
    for year in MR81_YEARS {
        let source = source(&format!("mr81-{year}")).expect("registered").1;
        let bytes = cache::read_cached(root, source).map_err(|e| e.to_string())?;
        // Latin-1: the reports carry the occasional non-UTF-8 byte in a sponsor name, and a
        // lossy UTF-8 read would put a replacement character into committed data.
        reports.push((*year, bytes.iter().map(|b| *b as char).collect::<String>()));
    }
    let years: Vec<fixtures::Mr81Year<'_>> = reports
        .iter()
        .map(|(year, report)| fixtures::Mr81Year {
            year: *year,
            report,
        })
        .collect();
    fixtures::build_mr81(&years)
}

/// Read the one worksheet whose name begins with `prefix`.
///
/// The Department of Taxation renames SD-1's worksheets between tax years: `ExJVS` and
/// `SD1DATWK23` in the TY2023 workbook against `ExJVS24` and `SD1DAT24` in TY2024. The stem is
/// stable and the year suffix is not, which is most of what
/// [`tax-abstract`](../sources/tax-abstract.md) was blocked on for twelve phases. Matching the
/// stem is the whole fix, and it is here rather than in [`fixtures`] because it is a property of
/// how the file is published, not of what the numbers mean.
///
/// With no match, the prefix is passed through to the reader so the error names it and lists the
/// sheets that are actually there — better than a bare `None` when the department renames again.
fn sheet_by_prefix(
    book: &spreadsheet::AnyWorkbook,
    prefix: &str,
) -> Result<Vec<Vec<String>>, RebuildError> {
    let name = book
        .sheet_names()
        .into_iter()
        .find(|name| name.starts_with(prefix))
        .map_or_else(|| prefix.to_string(), str::to_string);
    Ok(book.rows(&name)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fixtures_rebuild_names_point_at_files_that_exist() {
        let root = cache::repository_root();
        for path in [
            fixtures::FY27_FIXTURE,
            fixtures::PROFILE_FIXTURE,
            fixtures::GRADE_BANDS_FIXTURE,
        ] {
            assert!(root.join(path).exists(), "{path} is not committed");
        }
    }

    #[test]
    fn every_wired_connector_has_a_committed_digest() {
        // A source the fixtures were built from but that nothing pins is a provenance hole.
        let root = cache::repository_root();
        let manifest = std::fs::read_to_string(root.join(cache::MANIFEST)).unwrap_or_default();
        let pinned = cache::parse_manifest(&manifest);
        for connector in CONNECTORS {
            if !connector.status.is_wired() {
                continue;
            }
            for source in connector.sources {
                assert!(
                    pinned.iter().any(|d| d.key == source.key),
                    "{} is wired but not pinned in {}",
                    source.key,
                    cache::MANIFEST
                );
            }
        }
    }
}
