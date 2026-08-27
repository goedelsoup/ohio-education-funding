//! Everything above one district: the statewide aggregates, the policy checkpoints, the
//! appropriation series, and [`Bundle`] itself.

use crate::*;

/// Statewide context, so a consumer can position any district without recomputing.
#[derive(Debug, Clone, PartialEq)]
pub struct Statewide {
    /// Number of districts in the bundle.
    pub districts: usize,
    /// Districts funded by the guarantee.
    pub on_guarantee: usize,
    /// Districts at the 20-mill floor.
    pub at_millage_floor: usize,
    /// Districts above the floor by less than a twentieth of a mill; see
    /// [`District::near_millage_floor`].
    pub near_millage_floor: usize,
    /// Median voted current operating millage — the rate voters approved.
    pub median_voted_millage: f64,
    /// Median effective Class I rate — the rate anyone pays. The gap is H.B. 920.
    pub median_effective_millage: f64,
    /// Median share of its voted rate a district has lost to reduction factors.
    ///
    /// Not `1 - median_effective / median_voted`. That is the ratio of medians, which is a
    /// different district's arithmetic in the numerator and the denominator and answers no
    /// question anyone asked. This is the median of the per-district ratio.
    pub median_millage_reduction: f64,
    /// What one mill raises per pupil, statewide median.
    ///
    /// The local half of the formula reduced to one number. A mill is the same rate everywhere
    /// and raises hundreds of times as much in one district as in another, which is why
    /// comparing two districts' millage without it compares effort to capacity.
    pub median_yield_per_mill: Dollars,
    /// The lowest yield per mill per pupil in the state.
    pub min_yield_per_mill: Dollars,
    /// The highest.
    pub max_yield_per_mill: Dollars,
    /// Median taxable value per pupil **on Table SD-1's denominator**.
    ///
    /// Separate from [`Statewide::median_valuation_per_pupil`], which is on the District Profile
    /// Report's enrolled ADM. The two numerators are identical to the dollar and the two pupil
    /// counts are not, so a district's SD-1 figure has to be positioned against this median and
    /// not against the other one. See [`PropertyTaxYear::adm`].
    pub median_sd1_value_per_pupil: Dollars,
    /// Districts whose effective Class I rate is below the charge-off rate they would be
    /// charged at — the phantom revenue the mechanism was replaced for producing.
    pub below_charge_off_rate: usize,
    /// Districts the charge-off would leave with no base cost aid at all, having no minimum
    /// state share to stop at.
    pub charge_off_exceeds_base_cost: usize,
    /// Median change in base cost aid per pupil from the charge-off to the plan.
    pub median_regime_difference: Dollars,
    /// Districts receiving nothing from targeted assistance, the largest categorical program.
    ///
    /// It is equalisation: it switches off once a district has enough valuation per pupil. That
    /// the largest single program in the state reaches only four districts in five is invisible
    /// while the six categoricals are reported as one number.
    pub districts_without_targeted_assistance: usize,
    /// Districts whose base cost aid is set by the minimum state share.
    pub at_minimum_state_share: usize,
    /// Median assessed valuation per pupil.
    pub median_valuation_per_pupil: Dollars,
    /// Median operating expenditure per pupil.
    pub median_operating_expenditure_per_pupil: Dollars,
    /// Correlation between valuation per pupil and formula aid per pupil.
    pub wealth_neutrality_formula: f64,
    /// Correlation between valuation per pupil and realized aid per pupil.
    pub wealth_neutrality_realized: f64,
    /// Total guarantee dollars.
    pub guarantee_total: Dollars,
    /// Total realized state aid.
    pub realized_aid_total: Dollars,
    /// The minimum state share this model operates under.
    pub minimum_state_share: f64,
    /// The statewide median district's weighted wealth, which targeted assistance's capacity
    /// index divides by — `project::panel::categoricals::TA_MEDIAN_WEIGHTED_WEALTH`.
    ///
    /// Carried in the feed because the district page states it, and a figure the page types is a
    /// figure that stays exactly as stale as the last person to remember it. The two tiers of the
    /// largest categorical in Ohio are unreadable without the median they index against, so the
    /// page needs it; this is where it comes from.
    pub targeted_assistance_median_weighted_wealth: Dollars,
    /// The statewide median weighted wealth per resident pupil, which the wealth index divides
    /// by — `project::panel::categoricals::TA_MEDIAN_WEALTH_PER_PUPIL`.
    pub targeted_assistance_median_wealth_per_pupil: Dollars,
    /// The appropriation limit the preschool special education sheet prints beside its factor.
    ///
    /// `project::panel::supplements::PREK_SPED_APPROPRIATION`. It is the FY2025 estimate rather
    /// than the FY2027 appropriation, and the district page says so — see that constant.
    pub preschool_appropriation: Dollars,
    /// The proration factor that sheet states.
    pub preschool_proration: f64,
    /// What the program pays at that factor, summed over the districts in this feed.
    ///
    /// Summed here rather than on the page for the reason [`Statewide::finances`] is: the page and
    /// the feed cannot then disagree about which districts are in the total. It exceeds
    /// [`Statewide::preschool_appropriation`], which is the fact the card is built around.
    pub preschool_total: Dollars,
    /// How the funding side relates to the outcome side. `None` if no district joined.
    pub outcomes: Option<OutcomeStatewide>,
    /// Closed fiscal years of actuals, summed over the districts in this feed.
    ///
    /// Summed in Rust rather than left to the page so that the two cannot disagree about which
    /// districts are in the total. The panel behind it covers 660 reporting bodies including
    /// joint vocational districts; this is the 609 traditional districts the feed carries, which
    /// is the population every other figure on the page is over.
    pub finances: Vec<FinanceYear>,
}

/// A policy, in the shape the web layer sends it back.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolicyShape {
    /// `as-enacted`, `removed`, `rebase`, or `phase-out`.
    pub guarantee: &'static str,
    /// The factor or remaining share, where the rule takes one.
    pub guarantee_argument: f64,
    /// Multiplier on aggregate base cost.
    pub base_cost_scale: f64,
    /// Minimum state share of base cost.
    pub minimum_state_share: f64,
    /// Appropriated fraction of base cost aid.
    pub phase_in_general: f64,
    /// Appropriated fraction of categorical aid.
    pub phase_in_dpia: f64,
}

/// One clause of a draft bill, as the site needs it.
///
/// The lever fields carry a provision the model can run; they are empty on one it cannot, so an
/// unpriced provision is one whose levers are absent — counted from the data rather than declared.
#[derive(Debug, Clone, PartialEq)]
pub struct DraftProvision {
    /// Position in the draft, one-based.
    pub ordinal: u16,
    /// What it does, in one line.
    pub title: String,
    /// The Revised Code section it would amend, or `uncodified`.
    pub authority: String,
    /// The corpus `parameter` node it binds, empty where none exists.
    pub parameter: String,
    /// The lever key — one of the five, or empty when nothing here can run it.
    pub lever: String,
    /// The lever's proposed value, in the string form the query string carries.
    pub proposed: String,
    /// Why it does not price, or what the run is sized against where it does.
    pub note: String,
}

/// A bill that is not law, exported so the site can open it in the scenario runner.
///
/// # Why the feed carries this and the corpus does not
///
/// The binding is `crates/project/fixtures/draft-provisions.tsv`, and a test holds the corpus node
/// to it. The site could read the node instead — it reads every other corpus document directly —
/// but the node states a provision in prose, and building a lever position by parsing prose is
/// the failure mode this repository has already paid for twice. So the machine-readable half
/// travels through the feed, where Rust is authoritative, and the prose half stays in the node,
/// where a reader is.
///
/// # The unpriced count travels with it, and that is the whole point
///
/// `project::drafts::Priced` cannot be constructed without the provisions it failed to price, so
/// the CLI cannot print a cost without them. A feed that exported the lever positions and dropped
/// the rest would hand the web layer exactly the hole the Rust refuses — the scenario page would
/// show a statewide total for two of a bill's five clauses and call it the bill.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    /// The slug, which is also the corpus node's filename.
    pub slug: String,
    /// Every provision, in the order the draft states them.
    pub provisions: Vec<DraftProvision>,
}

/// A Rust-computed result the web layer must reproduce before it is allowed to compute more.
///
/// See the crate note. This is what makes a second implementation of the formula acceptable.
#[derive(Debug, Clone, PartialEq)]
pub struct Checkpoint {
    /// Human label, shown if the check fails.
    pub label: String,
    /// The policy that produced it. Without this a consumer could verify a number while
    /// computing a different scenario from the one the number belongs to.
    pub policy: PolicyShape,
    /// Change in total state aid against current law.
    pub cost: Dollars,
    /// Total realized aid under the policy.
    pub realized_aid: Dollars,
    /// Districts whose aid rises.
    pub gainers: usize,
    /// Districts whose aid falls.
    pub losers: usize,
    /// Districts the policy does not reach.
    pub unmoved: usize,
    /// Districts on the guarantee under the policy.
    pub on_guarantee: usize,
    /// Districts the guarantee paid under **both** policies.
    ///
    /// Not the same as [`Checkpoint::unmoved`], and the gap between them is informative: a
    /// formula district can be unmoved because the lever pulled does not touch it, while a
    /// guarantee district is unmoved because nothing pulled can touch it until the formula
    /// overtakes its frozen baseline.
    pub held_throughout: usize,
    /// Districts the policy lifted off the guarantee onto the formula.
    pub lifted_off: usize,
    /// Districts the policy pushed from the formula onto the guarantee.
    pub pushed_on: usize,
}

/// A Rust-computed *forecast* the web layer must reproduce before it may draw a band.
///
/// The same discipline as [`Checkpoint`], applied to the harder half. Reproducing a simulation
/// checks one function; reproducing a forecast checks the projection, the prior, the compounding
/// of the interval with the horizon, and the decision to re-run the whole formula at each end of
/// the enrollment band rather than scale the central answer — which matters because the
/// guarantee is a `max` and the aid curve has a kink no scaling reproduces.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCheckpoint {
    /// Human label, shown if the check fails.
    pub label: String,
    /// The policy held fixed across the horizon.
    pub policy: PolicyShape,
    /// The fiscal year projected to.
    pub fiscal_year: u16,
    /// Total realized aid at the central enrollment estimate.
    pub realized_aid: Dollars,
    /// Total realized aid at the low end of the enrollment band.
    pub low: Dollars,
    /// Total realized aid at the high end.
    pub high: Dollars,
    /// Projected total ADM.
    pub adm: f64,
    /// Districts on the guarantee at projected enrollment.
    pub on_guarantee: usize,
}

/// How this feed's forecasts were made, and what their interval rests on.
///
/// The page carries its own copy of the projection so a slider does not need a round trip, as it
/// does for the formula. This block is what makes that acceptable: the method and its parameters
/// so the page runs the same one, and [`Projection::checkpoints`] so it has to prove it did.
#[derive(Debug, Clone, PartialEq)]
pub struct Projection {
    /// The last observed fiscal year. Everything past it is forecast.
    pub base_year: u16,
    /// The furthest year the checkpoints reach, and the furthest the page should offer.
    pub horizon: u16,
    /// The last fiscal year the plan's own sections apply to.
    ///
    /// Not a bound on the forecast — the horizon above is that. This is the year past which
    /// "current law" stops naming a law: three sections of R.C. 3317 apply only for FY2026 and
    /// FY2027 by their own terms, and forty divisions of five more hand their values to a General
    /// Assembly that has not met. A consumer drawing a year past this one has to say so.
    ///
    /// Read from `project::statute`, which derives it from the committed extract.
    pub statute_ends: u16,
    /// `damped`, `cagr`, `linear`, or `flat`.
    pub method: String,
    /// Per-year decay applied to the fitted growth rate. 1.0 is undamped.
    pub damping: f64,
    /// Standard deviation of annual enrolled-ADM growth **across districts**.
    ///
    /// Not this district's variability — three observations cannot give that. It is how much
    /// districts differ from one another, used as a floor on the uncertainty.
    pub sigma: f64,
    /// Standard deviations spanned on each side of the point.
    pub z: f64,
    /// What produced [`Projection::sigma`]. Printed wherever the band is.
    pub prior_source: String,
    /// Forecasts the consumer must reproduce.
    pub checkpoints: Vec<ForecastCheckpoint>,
}

/// The exported feed.
#[derive(Debug, Clone, PartialEq)]
pub struct Bundle {
    /// Schema version; see [`CONTRACT_VERSION`].
    pub contract_version: String,
    /// What the figures describe and where they came from.
    pub provenance: String,
    /// The fiscal year the model computes.
    ///
    /// **The model's year, and not the page's.** A district page shows this beside a 2024 tax
    /// year, a 2024-25 report card, an FY2022 Census survey and a five-year forecast reaching back
    /// to FY2020. It is the year of the formula and of nothing else; see [`Bundle::series_years`]
    /// for what each other block is measured in.
    pub fiscal_year: u16,
    /// The year every other series in this feed is measured in, by series key.
    ///
    /// Ordered by key so a diff of the feed is readable. See [`SeriesYear`].
    pub series_years: Vec<SeriesYear>,
    /// Statewide aggregates.
    pub statewide: Statewide,
    /// Reference results the consumer must reproduce.
    pub checkpoints: Vec<Checkpoint>,
    /// The drafts this repository holds, ordered by slug. Empty if none.
    ///
    /// Carries the unpriced provisions as well as the priced ones, so a consumer cannot show a
    /// draft's cost without what the cost leaves out. See [`Draft`].
    pub drafts: Vec<Draft>,
    /// How to project, and the forecasts that check the projection. `None` disables the band.
    pub projection: Option<Projection>,
    /// The price index. `None` means the feed can only be shown in nominal dollars.
    pub deflator: Option<Deflator>,
    /// Where Ohio sits among the states. `None` if the Census fixture is absent.
    pub national: Option<National>,
    /// The Census survey year by year, oldest first. Empty if the panel is absent.
    ///
    /// The only part of the feed that reaches before FY2020, and the only part measured on
    /// something other than the department's own formula. See [`HistoryYear`].
    pub history: Vec<HistoryYear>,
    /// The appropriation lines themselves, with the act that created each. Empty if absent.
    ///
    /// The current edition only, ordered by line item. See [`AppropriationLine`].
    pub appropriation_lines: Vec<AppropriationLine>,
    /// What the General Assembly appropriated, year by year, oldest first. Empty if absent.
    ///
    /// The only block in this feed that is an input to the funding system rather than an output
    /// of it. See [`AppropriationYear`].
    pub appropriations: Vec<AppropriationYear>,
    /// The meal-program poverty share, October by October, oldest first. Empty if absent.
    ///
    /// Reaches back further than [`Self::history`] — FY2001 against FY2009 — and on a third
    /// measurement again. See [`MealProgramYear`].
    pub meal_program: Vec<MealProgramYear>,
    /// The whole casino county student fund, fiscal year by fiscal year, oldest first.
    ///
    /// **Statewide here means every district the Department of Taxation pays**, which is about a
    /// thousand — community schools, STEM schools and joint vocational districts are inside
    /// R.C. 5753.11's definition of a public school district. Summing [`District::casino`] across
    /// this feed gives a smaller number, because this feed carries 609 districts. The two are
    /// different quantities and the difference is the point: the fund's population is not the
    /// formula's.
    pub casino: Vec<CasinoYear>,
    /// Ohio's 99 House districts, with school funding apportioned across them.
    pub house_districts: Vec<HouseDistrict>,
    /// And its 33 Senate districts, each exactly three House districts.
    ///
    /// A less approximate view than the House one: seats three times larger mean 392 of 609
    /// school districts sit wholly inside a single Senate district, against 270 for the House.
    pub senate_districts: Vec<HouseDistrict>,
    /// Per-district records.
    pub districts: Vec<District>,
}

/// One state's school finance, from the Census Bureau's Annual Survey of School System Finances.
///
/// A third source, and a federal one. Everything else in this feed comes from Ohio describing
/// itself; the corpus has been able to say what Ohio does and never whether it is unusual.
#[derive(Debug, Clone, PartialEq)]
pub struct StateFinance {
    /// Two-digit FIPS.
    pub fips: String,
    /// State name, or the District of Columbia.
    pub name: String,
    /// School systems with enrolment.
    pub systems: usize,
    /// Fall enrolment, a headcount.
    pub enrollment: f64,
    /// Total revenue, in thousands of dollars as the survey reports it.
    pub total_revenue: Dollars,
    /// Federal revenue, thousands.
    pub federal_revenue: Dollars,
    /// State revenue, thousands.
    pub state_revenue: Dollars,
    /// Local revenue, thousands. Includes parent-government appropriations.
    pub local_revenue: Dollars,
    /// Local revenue from the district's own property tax, thousands. Zero where districts are
    /// dependent; see [`StateFinance::fiscally_independent`].
    pub property_tax_revenue: Dollars,
    /// Appropriations from a parent city or county, thousands.
    pub parent_government_revenue: Dollars,
    /// Current spending, thousands.
    pub current_spending: Dollars,
}

impl StateFinance {
    /// Whether this state's school districts levy their own tax rather than being funded by a
    /// parent government.
    ///
    /// The distinction that makes a property tax comparison possible or impossible. Twelve states
    /// fund schools mostly through a city or county appropriation, so the survey attributes the
    /// tax to the parent and reports the district's own property tax as zero. Massachusetts and
    /// Virginia raise as much from property tax as anywhere and score nothing.
    #[must_use]
    pub fn fiscally_independent(&self) -> bool {
        self.parent_government_revenue < self.local_revenue * 0.10
    }

    /// Local revenue as a share of total. Comparable across both district structures.
    #[must_use]
    pub fn local_share(&self) -> f64 {
        if self.total_revenue > 0.0 {
            self.local_revenue / self.total_revenue
        } else {
            0.0
        }
    }

    /// State revenue as a share of total.
    #[must_use]
    pub fn state_share(&self) -> f64 {
        if self.total_revenue > 0.0 {
            self.state_revenue / self.total_revenue
        } else {
            0.0
        }
    }

    /// Current spending per pupil, in dollars. The survey reports thousands.
    #[must_use]
    pub fn spending_per_pupil(&self) -> f64 {
        if self.enrollment > 0.0 {
            self.current_spending * 1_000.0 / self.enrollment
        } else {
            0.0
        }
    }
}

/// Where Ohio sits among the states, and the figures that put it there.
///
/// # What this settles that nothing else in the corpus could
///
/// The *DeRolph* holding was that Ohio relied too heavily on local property tax. Every figure the
/// corpus has held until now describes Ohio alone, so the claim could be restated and never
/// tested — "too heavily" needs a comparison, and there was nothing to compare against.
///
/// There is now. Ohio raises **51.8% of school revenue locally against a national 43.4%, seventh
/// highest of fifty-one**, and takes **34.4% from the state against a national 43.4%, forty-fifth
/// of fifty-one**. It spends about the national average per pupil and is exactly average on
/// federal money. The distinctive thing about Ohio is not how much its schools cost but who pays.
///
/// # The year, and why it flatters nothing
///
/// FY2022 is the peak of federal pandemic relief, so the federal share is inflated and the local
/// and state shares are correspondingly deflated. That runs against the finding rather than for
/// it: in an ordinary year Ohio's local share would be higher, not lower.
#[derive(Debug, Clone, PartialEq)]
pub struct National {
    /// The survey year, as a fiscal year.
    pub fiscal_year: u16,
    /// Every state and the District of Columbia, alphabetically.
    pub states: Vec<StateFinance>,
    /// Ohio's rank on local share, 1 being the highest, out of all 51.
    pub ohio_local_rank: usize,
    /// Ohio's rank on state share, 1 being the highest.
    pub ohio_state_rank: usize,
    /// Ohio's rank on current spending per pupil.
    pub ohio_spending_rank: usize,
    /// Ohio's rank on property tax share, among fiscally independent states only.
    pub ohio_property_tax_rank: usize,
    /// How many states that comparison is over.
    pub independent_states: usize,
    /// National local share of school revenue.
    pub national_local_share: f64,
    /// National state share.
    pub national_state_share: f64,
    /// National current spending per pupil.
    pub national_spending_per_pupil: f64,
}

/// One year of the Census survey, as the historical view needs it.
///
/// # Why this grain and not the formula's
///
/// Everything else in this feed is the Fair School Funding Plan computing FY2027 for 609
/// traditional districts. This is the only block that reaches back, and it reaches back on a
/// different measurement entirely: the Census Bureau's survey of what school systems actually
/// took in, which covers roughly 950 Ohio agencies a year including community schools and
/// educational service centers, on the Bureau's own enrollment count rather than ADM.
///
/// The two do not reconcile and are not meant to. A funding formula figure and a revenue survey
/// figure for the same district in the same year routinely disagree, which is exactly why the
/// [catalog](../../../../.yidam/catalog/census-f33-school-system-finances.md) exists.
///
/// # Comparable only
///
/// Every share and every quartile here is computed over the agencies the survey marks comparable
/// — roughly two-thirds of the rows — because that is the population the corpus's existing
/// single-year figures were computed over. A series whose first point is not comparable to the
/// number already on the page is worse than no series.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HistoryYear {
    /// The survey year, as a fiscal year.
    pub fiscal_year: u16,
    /// Comparable agencies the year is computed over.
    pub districts: usize,
    /// Local revenue as a share of total.
    pub local_share: f64,
    /// State revenue as a share of total.
    pub state_share: f64,
    /// Federal revenue as a share of total.
    pub federal_share: f64,
    /// Mean local revenue per pupil in the poorest quartile of districts.
    pub poorest_local_per_pupil: f64,
    /// And in the richest.
    pub richest_local_per_pupil: f64,
    /// The gap between them, which is what the other levels are measured against.
    ///
    /// Named for its denominator, as every per-pupil field in the feed is. `gap` alone would
    /// have escaped the web layer's denominator guard, which reads field names — and this whole
    /// block divides by a pupil count that is not the one any other figure on the site uses.
    pub gap_per_pupil: f64,
    /// Dollars per pupil of that gap closed by state aid.
    pub state_closes_per_pupil: f64,
    /// And by federal aid.
    pub federal_closes_per_pupil: f64,
}

impl HistoryYear {
    /// What neither level closes — the part a district actually experiences.
    #[must_use]
    pub fn residual_per_pupil(&self) -> f64 {
        self.gap_per_pupil - self.state_closes_per_pupil - self.federal_closes_per_pupil
    }
}

/// One fiscal year of what the General Assembly appropriated to the department.
///
/// # An appropriation is not a payment
///
/// This is what was set aside, not what a district received. An appropriation is a ceiling, and
/// the formula's own proration factor exists because at least one line has been a residual
/// claimant. A difference between this and the payment reports is not an error in either, and the
/// two must never be differenced to produce a third figure.
///
/// # Why the source is carried
///
/// Two publications answer for this series and they are not interchangeable, even though they
/// agree. The workbooks and greenbooks cover every year but four; the Catalog of Budget Line Items
/// covers **FY2006-07 and FY2012-13**, the two bienniums the workbook route cannot reach — one
/// because the 126th's greenbook has no line-item table at all, the other because LSC serves that
/// biennium's two workbook variants as the same file.
///
/// Across the 1,712 claims where both speak, the two extractions do not differ by a cent. Carrying
/// [`Self::source`] is therefore not a hedge about accuracy; it is so a reader can see that four
/// years of this series rest on a different document from the rest, and check them separately if
/// the difference ever starts to matter.
///
/// # No dollars per pupil here, deliberately
///
/// A statewide appropriation divided by a pupil count would be a per-pupil figure on a denominator
/// no other figure in this feed uses, sitting one division away from the formula's own per-pupil
/// numbers. The block carries totals and nothing else.
#[derive(Debug, Clone, PartialEq)]
pub struct AppropriationYear {
    /// The fiscal year.
    pub fiscal_year: u16,
    /// Everything the department was appropriated that year, in that year's dollars.
    ///
    /// Excludes the property tax reimbursement lines, which are numbered as the department's and
    /// are not its budget — `200903` alone is $1.3 billion a year.
    pub enacted: f64,
    /// The two foundation funding lines together: GRF `200550` and Lottery `200612`.
    ///
    /// The formula's own appropriation, as against everything else the department is given.
    pub foundation_funding: f64,
    /// How many line items the total is over.
    pub items: usize,
    /// Which publication answers for this year: `workbook` or `catalog`.
    pub source: String,
}

/// One appropriation line the department is funded through, and the act that created it.
///
/// # What this is for
///
/// [`AppropriationYear`] says how much the department was given. This says what the giving is made
/// of. Together they carry a fact neither carries alone: the department's budget is accreted
/// rather than designed — the lines in force were created by acts spanning half a century, and
/// the oldest still-live one predates every funding regime this corpus documents.
///
/// # Half of them say nothing about their origin
///
/// [`Self::general_assembly`] is `None` for roughly half the lines, because the Catalog's legal
/// basis cites only their current authority. Those are carried as unknown rather than filled from
/// an earlier edition with the same number: a line item number is reused — `200604` names three
/// different programmes across three funds in this series — so inheriting an origin down a number
/// attributes one programme's founding act to another's.
///
/// # `discontinued` is a label, not a finding
///
/// The publisher's own mark, and it does not distinguish abolition from consolidation: a line
/// folded into another is discontinued too. Whether the department's disappearing lines were
/// abolished or folded is an open question in `state-foundation-aid` that this does not settle.
#[derive(Debug, Clone, PartialEq)]
pub struct AppropriationLine {
    /// The fund it is paid from.
    pub fund: String,
    /// The six-digit line item number.
    pub ali: String,
    /// Its name in the current edition.
    pub name: String,
    /// The act that established it, as the Catalog writes it. Empty when it names none.
    pub established_by: String,
    /// That act's General Assembly, and the year it convened. `None` when no act is named.
    pub general_assembly: Option<u16>,
    /// The year that General Assembly convened. `None` alongside `general_assembly`.
    pub convened: Option<u16>,
    /// Whether the Catalog marks the line discontinued.
    pub discontinued: bool,
}

/// One October of the free and reduced-price lunch report, as a share.
///
/// # What this measures, and what it does not
///
/// Not enrollment and not poverty. MR-81 is the Office for Child Nutrition's meal-program report:
/// a count of *applications approved* for free or reduced-price lunch, over the denominator a
/// lunch claim is filed against. It is here because R.C. 3317.03(B)(21) hands the definition of
/// "economically disadvantaged" to the department, free-lunch eligibility has been the
/// department's operative test, and this is the longest run of that test available — seventeen
/// years where the rest of the feed has six.
///
/// [Disadvantaged pupil impact aid](../../../../.yidam/corpus/formula-component/fsfp-disadvantaged-pupil-impact-aid.yml)
/// is paid on that count, so this is the closest thing the feed carries to a history of what the
/// formula's poverty weight is computed on.
///
/// # Why there are no dollars here
///
/// Deliberately. A share is dimensionless, so this block needs no deflator and cannot be shown in
/// real terms — which is the point: the underlying counts are on a denominator no other figure in
/// this feed uses, and a dollar figure computed on it would be one division away from being
/// compared to a formula-side number. See [`Self::basis`] for the second reason.
///
/// # Sponsors are not districts
///
/// [`Self::sponsors`] counts *public sponsors*, which includes county boards of developmental
/// disabilities and community schools alongside traditional districts. The count rising across
/// the window is mostly community schools opening, not districts appearing, and it is carried
/// here so that a reader watching the share move can see the population move underneath it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MealProgramYear {
    /// The October counted, as a fiscal year.
    pub fiscal_year: u16,
    /// Public sponsors the year is computed over, after excluding published corruption.
    ///
    /// The FY2005 file gives one elementary school an enrollment of 342,332. It is excluded by
    /// name upstream rather than repaired, so this count is one lower that year than the file
    /// implies.
    pub sponsors: usize,
    /// The denominator in force that year, summed over those sponsors.
    ///
    /// Carried rather than implied. Two reasons, and the second is the load-bearing one: a share
    /// on its own cannot be checked, and the web layer's denominator guard walks *field names* —
    /// so a block whose only fields are `share` and `sponsors` is invisible to it. `enrollment`
    /// is a name that guard recognises, which is what forces this block to declare what it
    /// divides by. See `web/src/lib/denominators.ts`.
    pub enrollment: f64,
    /// Free and reduced-price applications approved, summed over those sponsors.
    ///
    /// From FY2012 this is short by every child in a community-eligibility school, because those
    /// sponsors collect no applications. See [`Self::streams`].
    pub approved: f64,
    /// Directly certified children in community-eligibility schools. Zero before FY2012.
    ///
    /// Not an approval and not comparable to one. Direct certification reaches families already on
    /// SNAP, TANF, foster care or a homeless roll; an application reaches anyone under the income
    /// line who files. The programme's own reckoning of the gap is the 1.6 multiplier behind
    /// [`Self::ceiling`].
    pub identified: f64,
    /// [`Self::approved`] over [`Self::enrollment`], which is the figure worth reading — while the
    /// report is one file.
    ///
    /// `None` from FY2012, and that is the finding rather than a gap. Three publications counting
    /// three different things do not add up to a share, so those years carry
    /// [`Self::floor`] and [`Self::ceiling`] instead and nothing writes a number between them.
    pub share: Option<f64>,
    /// The lowest share the source supports: approvals plus directly certified children.
    ///
    /// Equal to [`Self::share`] while the report is one file.
    pub floor: f64,
    /// The highest: what every sponsor may claim for, which under community eligibility is the
    /// directly certified count times 1.6, capped at enrollment school by school.
    ///
    /// Equal to [`Self::share`] while the report is one file.
    pub ceiling: f64,
    /// The share of the October's enrollment under sponsors that collect no applications.
    ///
    /// Zero through FY2011 and a sixth by FY2014. This is the size of the hole in
    /// [`Self::approved`], and it grows because community eligibility is open to schools whose
    /// poverty is already high — the population leaving the applications-based count is not a
    /// random sample of it.
    pub without_applications: f64,
    /// How many files the October was published as: one through FY2011, three from FY2012.
    ///
    /// The field a consumer has to read before drawing a line. From FY2012 the report splits into
    /// Traditional, Provision 2 and Community Eligibility, and only the first still counts
    /// applications — so a series that joins across this reads the poorest sponsors leaving the
    /// form as poverty falling.
    pub streams: usize,
    /// Which denominator that is: `adm` through FY2009, `ce` from FY2010.
    ///
    /// The definition changes mid-series. `CECount` is "the highest daily number of students with
    /// access to the program", which is neither ADM nor the count that preceded it, so the share
    /// is not continuous across FY2009/FY2010 and nothing here splices it. A consumer that plots
    /// this as one line without breaking it at the basis change is making the error this field
    /// exists to prevent.
    pub basis: String,
    /// Whether this October is a reading of the state at all.
    ///
    /// False for exactly two, and a consumer drawing a series has to break on it. Under USDA's
    /// nationwide free-meal waivers a sponsor could serve every student free without collecting
    /// one application, and almost every one of them stopped: October 2020 and October 2021 carry
    /// **296 and 261 sponsors against about 850**, and a quarter of the enrolment. Their floor
    /// reads twenty points above the years either side, and every point of that is about who
    /// filed rather than about Ohio.
    ///
    /// The rows are carried rather than dropped because they are true about the sponsors in them,
    /// and because a gap would be indistinguishable from a year nobody retrieved.
    pub comparable: bool,
}
