//! The fifth chart registry: [`SPREADS`], a whole population on two signed axes that sum to a
//! third quantity, where the finding is which regions of the plane are full and which are empty.
//!
//! [`Regions`] carries both halves of the census rule: every region a reader could count names a
//! figure, and the subjects the computation could not place are named on the picture rather than
//! dropped out of the denominator. A spread of 607 captioned 609 is a small lie that is easy to
//! ship.

use super::*;

/// One spread the wiki draws: a **whole population** on two signed axes, cut into regions whose
/// counts are the claim.
///
/// The fifth drawing, and the fourth shape that needed its own binding rule. A [`Series`] is one
/// measure of several things; a [`Cloud`] is one population under one predictor with a line
/// through it; a [`Plane`] is a handful of named alternatives each quoted by both coordinates; a
/// [`Curve`] is named lines over an ordered index. A spread is none of those. It is six hundred
/// anonymous subjects on two axes that **sum to a third quantity**, where the finding is which
/// regions of the picture are full and which are empty.
///
/// An empty region is the thing a spread exists to show, and it is exactly what a list cannot.
/// "Nobody is off the floor because the formula pays less per child" is a statement that one
/// wedge of this plane holds four districts out of three hundred and fourteen — and a reader will
/// take a full region on trust from prose long before they take an empty one.
///
/// # Why not a cloud
///
/// [`Cloud::at_one_scale`] asserts a log predictor above zero and gives every panel the x axis's
/// own log span, because a cloud's claim is a *slope*. Both axes here are already logarithms and
/// both are signed, so neither is a quantity a log axis can place; and there is no fitted line to
/// draw, because nothing is fitted — the two terms are an identity. A cloud's whole binding rule
/// is its fit, and a shape with no fit would have no rule at all.
///
/// # What stands in for the endpoint rule: the census
///
/// A column is quoted by its extremes, a cloud by its fitted line, a plane by every coordinate,
/// a curve by each line's two ends and its worst departure. A spread is quoted by **how many
/// subjects are in each region**, so that is what is pinned: every [`Tally`] names an ordinary
/// [`Figure`] in [`FIGURES`], and the node that draws the picture binds all of them.
///
/// The second half of the rule is [`Regions::unreached`]. A census is a count of a population,
/// and a population is the thing that silently changes when a fixture is extended — so a subject
/// the computation could not place is **named on the picture** rather than dropped out of the
/// denominator. A spread of 607 captioned 609 is a small lie that is easy to ship.
///
/// # Boundaries are definitions, and so name no figure
///
/// A [`Boundary`] is where a classification stops being defined, not a measurement of anything:
/// on two terms that sum to a log multiple, "the multiple is one" is the line `y = -x` and "the
/// formula pays per pupil what the prior regime did" is the line `y = 0`. Both are arithmetic.
/// Nothing would go stale if a district moved, and a pin duplicating a definition checks nothing
/// — which is the argument [`CURVES`] already makes for its reference line, restated because a
/// reader meeting an unpinned number on a chart is owed it.
pub struct Spread {
    /// `<crate-directory>/<what-it-is>`, in the one key namespace the other registries share.
    pub key: &'static str,
    /// The crate that owns the computation, as the corpus cites it.
    pub owner: &'static str,
    /// What the whole drawing is, in words; its title.
    pub label: &'static str,
    /// What one point is, singular and lower case — "district". Written into the caption.
    pub subject: &'static str,
    /// The computation, over the same inputs the figures use.
    pub compute: fn(&Inputs) -> Regions,
}

/// One subject of a [`Spread`], placed and classed.
pub struct Mark {
    /// What the tooltip calls it.
    pub label: String,
    /// Its place on the horizontal axis, signed.
    pub x: f64,
    /// Its place on the vertical axis, signed.
    pub y: f64,
    /// Which of [`Regions::classes`] it belongs to, by index.
    pub class: usize,
}

/// One panel of a [`Spread`]: a sub-population, drawn on the frame all of them share.
///
/// The small-multiple half of the form. A second categorical cut of the same points cannot be a
/// second hue here — `web/src/lib/plot/tokens.ts` carries exactly two hues and a neutral, and its
/// ordinal ramp is a measured three steps — so a five-way cut is drawn as five panels, and the
/// gradient reads as the cloud climbing across a boundary panel by panel. Every panel is framed
/// identically, which is the only thing that makes "climbing" a statement about the data.
pub struct Slice {
    /// What this sub-population is, in words; the panel's own title.
    pub label: String,
    /// Its subjects.
    pub marks: Vec<Mark>,
}

/// A line on a [`Spread`] where a definition changes, given in the axes' own units.
///
/// A slope and an intercept rather than two endpoints, so that the segment cannot be anything but
/// a line across the whole frame: a caller able to supply two points could supply a line through
/// part of the cloud, and an arbitrary line through a cloud is a claim rather than a definition.
/// See [`Spread`] for why a definition names no figure.
pub struct Boundary {
    /// What the line means, in words.
    pub label: &'static str,
    /// Its slope in the drawn units.
    pub slope: f64,
    /// Its value where the horizontal axis is zero.
    pub intercept: f64,
}

impl Boundary {
    /// The segment this line makes across a frame, from the horizontal axis' low end to its high.
    #[must_use]
    pub fn across(&self, x: &Axis) -> ((f64, f64), (f64, f64)) {
        let at = |v: f64| (v, self.slope.mul_add(v, self.intercept));
        (at(x.min), at(x.max))
    }
}

/// One row of a [`Spread`]'s census: a region, how many subjects fall in it, and its pin.
pub struct Tally {
    /// What the region is, in words — the sentence the count is the answer to.
    pub label: String,
    /// How many subjects are in it.
    pub subjects: usize,
    /// The [`Figure`] that pins [`Self::subjects`].
    pub figure: &'static str,
}

/// The subjects of a [`Spread`], the frame they are drawn in, and the census that pins them.
pub struct Regions {
    /// The horizontal axis.
    pub x: Axis,
    /// The vertical axis.
    pub y: Axis,
    /// What each hue means, in the order the marks index.
    ///
    /// At most `CLASSES`, because the palette carries two hues and a neutral and generates no
    /// others. **The first class is drawn neutral**, so a spread whose classification has a
    /// "neither" state puts it first: `web/src/lib/plot/tokens.ts` calls the neutral slot "any
    /// mark with no polarity, never a hue", which is what a subject outside a classification is.
    pub classes: Vec<&'static str>,
    /// The lines where a definition changes.
    pub boundaries: Vec<Boundary>,
    /// The panels, all on the one frame.
    pub panels: Vec<Slice>,
    /// Every region of the picture a reader could count, each pinned.
    pub census: Vec<Tally>,
    /// The subjects the computation could not place.
    pub unreached: Vec<Missing>,
}

/// How many hues a spread may class its marks by.
///
/// Two series and a neutral, which is the whole of `web/src/lib/plot/tokens.ts`' categorical
/// palette and is not a limit this crate may spend its way past: a fourth class would arrive in
/// the manifest, find no colour waiting for it, and be drawn as one of the other three.
const CLASSES: usize = 3;

impl Regions {
    /// Frame a population around its own extremes, on axes neither of which is matched to the
    /// other.
    ///
    /// Unlike [`Cloud::at_one_scale`] the frame is **not** squared to the data, because no angle
    /// on this picture is a claim: the boundaries are definitions whose positions are exact
    /// wherever they are drawn, and a spread has no slope to read. What a square frame would buy
    /// is paid for by the axes — one term of this identity ranges five log units and the other
    /// one, so a shared scale would draw the second as a hairline and lose the sign of every
    /// district on it, which is the one thing the census counts.
    ///
    /// Every panel shares the frame, computed across all of them together. A panel framed on its
    /// own points would make five small multiples five different pictures.
    ///
    /// # Panics
    ///
    /// If there are no panels, a panel is empty, there are no classes or more than `CLASSES`,
    /// a mark names a class that does not exist, or a coordinate is not finite.
    #[must_use]
    pub fn framed(
        x: &'static str,
        y: &'static str,
        unit: Unit,
        classes: Vec<&'static str>,
        boundaries: Vec<Boundary>,
        panels: Vec<Slice>,
    ) -> Self {
        assert!(!panels.is_empty(), "a spread with no panels");
        assert!(
            panels.iter().all(|p| !p.marks.is_empty()),
            "a spread with an empty panel"
        );
        assert!(!classes.is_empty(), "a spread with no classes");
        assert!(
            classes.len() <= CLASSES,
            "a spread wants {} hues and the palette carries {CLASSES}",
            classes.len(),
        );
        let marks = || panels.iter().flat_map(|p| p.marks.iter());
        assert!(
            marks().all(|m| m.class < classes.len()),
            "a mark names a class this spread does not have"
        );
        let frame = |of: fn(&Mark) -> f64, label: &'static str| {
            let (low, high) = marks().map(of).fold((f64::MAX, f64::MIN), |(lo, hi), v| {
                assert!(v.is_finite(), "{label}: {v} cannot be drawn");
                (lo.min(v), hi.max(v))
            });
            let pad = (high - low) * CLOUD_PAD;
            Axis {
                label,
                unit,
                min: low - pad,
                max: high + pad,
                log: false,
            }
        };
        Self {
            x: frame(|m| m.x, x),
            y: frame(|m| m.y, y),
            classes,
            boundaries,
            panels,
            census: Vec::new(),
            unreached: Vec::new(),
        }
    }

    /// Apply the binding rule: what each region holds, and who could not be counted.
    ///
    /// One step rather than two because they are one rule. A census with nothing unreached beside
    /// it is a set of counts whose denominator the reader has to take on trust, and the denominator
    /// is what moves when a fixture is extended.
    ///
    /// # Panics
    ///
    /// If the census is empty, or a region's count exceeds the subjects drawn.
    #[must_use]
    pub fn counted(mut self, census: Vec<Tally>, unreached: Vec<Missing>) -> Self {
        assert!(!census.is_empty(), "a spread with no census");
        let drawn: usize = self.panels.iter().map(|p| p.marks.len()).sum();
        for tally in &census {
            assert!(
                tally.subjects <= drawn,
                "{:?} counts {} of {drawn} drawn",
                tally.label,
                tally.subjects,
            );
        }
        self.census = census;
        self.unreached = unreached;
        self
    }
}

/// The two terms of one identity, as the two axes they are.
///
/// Shared by both spreads because they are two readings of one picture, and a boundary or an axis
/// label that drifted between them would make them two.
const ENROLLMENT_TERM: &str = "The enrollment term: fewer pupils than in FY2020, in logs";

const PER_PUPIL_TERM: &str = "The per-pupil term: the FY2027 formula against the FY2020 regime, \
    per pupil, in logs";

/// The three states of `guarantee_origin::Decomposition::origin`, in the order the marks index
/// them and with the `None` case first — see [`Regions::classes`] for why first.
const ORIGINS: [&str; 3] = [
    "No majority to take: the floor is at or below the formula",
    "The enrollment term carries the majority",
    "The per-pupil term carries the majority",
];

/// Where the two classifications on this plane stop being defined.
///
/// `y = -x` is the multiple of one, which is exactly the boundary of the first class: a district
/// is below it when its floor is at or under its formula amount, which is the same predicate the
/// guarantee's own membership test uses, so the line does not approximate the hue change — it is
/// it. `y = 0` is the per-pupil term's sign, which is the line the "nobody is off the floor
/// because the formula pays less per child" claim is a count on one side of.
fn origin_boundaries() -> Vec<Boundary> {
    vec![
        Boundary {
            label: "A multiple of one: the floor equals the formula",
            slope: -1.0,
            intercept: 0.0,
        },
        Boundary {
            label: "The FY2027 formula pays per pupil what the FY2020 regime did",
            slope: 0.0,
            intercept: 0.0,
        },
    ]
}

/// One district as a mark on the origin plane.
fn origin_mark(s: &project::lost_pupils::Standing) -> Mark {
    use project::guarantee_origin::Origin;
    Mark {
        label: s.name.clone(),
        x: s.terms.enrollment_term,
        y: s.terms.per_pupil_term,
        class: match s.terms.origin() {
            None => 0,
            Some(Origin::EnrollmentLoss) => 1,
            Some(Origin::CapacityGrowth) => 2,
        },
    }
}

/// The two districts the identity does not reach, named the way the picture has to name them.
///
/// **By IRN as well as by name.** District names are not unique — there are three Buckeye Locals,
/// and the one the chained index misses is 047787 — so a note under the drawing saying "Buckeye
/// Local is absent" beside a cloud containing two districts of that name is worse than no note at
/// all: the reader can find the name, and what they find is not it. The absence is the whole
/// reason the subject is written down, so it is written down with the key the fixtures are on.
fn origin_unreached(i: &Inputs) -> Vec<Missing> {
    i.lost_pupils
        .unreached
        .iter()
        .map(|u| Missing {
            label: format!("{} (IRN {})", u.name, u.irn),
            why: u.why.to_string(),
        })
        .collect()
}

/// What pins each of the five wealth bands: how many districts are in it, and how many the
/// guarantee holds.
///
/// Aligned to `project::lost_pupils::FIFTHS`, which is where the bands are named and where the
/// cut is made. The held keys are the ones `crates/figures.json` already carries; the district
/// counts are new, and they are here because a panel drawing 102 marks under a caption saying
/// 103 is exactly the failure the census rule exists to prevent.
const LOST_PUPIL_FIFTHS: [(&str, &str); 5] = [
    (
        "project/lost-pupils-least-wealthy-fifth-districts",
        "project/lost-pupils-least-wealthy-fifth-held",
    ),
    (
        "project/lost-pupils-second-fifth-districts",
        "project/lost-pupils-second-fifth-held",
    ),
    (
        "project/lost-pupils-third-fifth-districts",
        "project/lost-pupils-third-fifth-held",
    ),
    (
        "project/lost-pupils-fourth-fifth-districts",
        "project/lost-pupils-fourth-fifth-held",
    ),
    (
        "project/lost-pupils-wealthiest-fifth-districts",
        "project/lost-pupils-wealthiest-fifth-held",
    ),
];

/// The spreads the wiki draws. See [`Spread`] for what stands in for the endpoint rule here.
///
/// #446's two drawings, and two readings of one plane rather than two charts. The first is every
/// district the identity reaches, coloured by which term carries the majority; the second is the
/// same axes, the same frame and the same hues over the districts that lost pupils, cut into
/// fifths by wealth. They are separate entries because they are separate populations — 607
/// against 514 — and a single entry mixing them would be a picture whose denominator changes
/// between its panels.
pub static SPREADS: &[Spread] = &[
    Spread {
        key: "project/the-two-terms-of-every-districts-multiple",
        owner: "crates/project",
        label: "What puts a district above its FY2020 floor, on the two terms that reconstruct \
                the multiple exactly",
        subject: "district",
        compute: |i| {
            let rows = &i.lost_pupils.standings;
            let formula: Vec<&project::lost_pupils::Standing> = rows
                .iter()
                .filter(|s| s.population == project::lost_pupils::Population::Formula)
                .collect();
            let by_class = |class: usize| {
                rows.iter()
                    .filter(|s| origin_mark(s).class == class)
                    .count()
            };
            Regions::framed(
                ENROLLMENT_TERM,
                PER_PUPIL_TERM,
                Unit::Ratio,
                ORIGINS.to_vec(),
                origin_boundaries(),
                vec![Slice {
                    label: "Every district the identity reaches".to_string(),
                    marks: rows.iter().map(origin_mark).collect(),
                }],
            )
            .counted(
                vec![
                    Tally {
                        label: "Districts the identity places".to_string(),
                        subjects: rows.len(),
                        figure: "project/districts-the-identity-reaches",
                    },
                    Tally {
                        label: "The formula pays them: at or below a multiple of one".to_string(),
                        subjects: formula.len(),
                        figure: "project/districts-off-the-guarantee-with-a-positive-funding-base",
                    },
                    Tally {
                        label: "Of those, with fewer pupils than in FY2020".to_string(),
                        subjects: i.lost_pupils.formula_that_lost_pupils,
                        figure: "project/formula-districts-that-lost-pupils",
                    },
                    Tally {
                        label: "Of those, paid more per pupil than the FY2020 regime".to_string(),
                        subjects: i.lost_pupils.formula_paid_more_per_pupil,
                        figure: "project/formula-districts-paid-more-per-pupil-than-fy2020",
                    },
                    Tally {
                        label: "Of those, paid less per pupil \u{2014} every one of which grew"
                            .to_string(),
                        subjects: formula.len() - i.lost_pupils.formula_paid_more_per_pupil,
                        figure: "project/formula-districts-paid-less-per-pupil-than-fy2020",
                    },
                    Tally {
                        label: "The guarantee pays them, the enrollment term in the majority"
                            .to_string(),
                        subjects: by_class(1),
                        figure: "project/districts-the-enrollment-term-carries",
                    },
                    Tally {
                        label: "The guarantee pays them, the per-pupil term in the majority"
                            .to_string(),
                        subjects: by_class(2),
                        figure: "project/districts-the-per-pupil-term-carries",
                    },
                ],
                origin_unreached(i),
            )
        },
    },
    Spread {
        key: "project/the-two-terms-by-wealth-among-those-that-lost-pupils",
        owner: "crates/project",
        label: "The same plane, over the districts that lost pupils since FY2020, in fifths by \
                published local capacity per pupil",
        subject: "district",
        compute: |i| {
            let rows = &i.lost_pupils.standings;
            let lost = project::lost_pupils::who(rows, true);
            let bands = project::lost_pupils::fifths(&lost);
            let mut census = vec![Tally {
                label: "Districts with fewer pupils than in FY2020".to_string(),
                subjects: lost.len(),
                figure: "project/districts-that-lost-pupils-since-fy2020",
            }];
            for ((band, name), (districts, held)) in bands
                .iter()
                .zip(project::lost_pupils::FIFTHS)
                .zip(LOST_PUPIL_FIFTHS)
            {
                census.push(Tally {
                    label: format!("The {name} fifth"),
                    subjects: band.len(),
                    figure: districts,
                });
                census.push(Tally {
                    label: format!("The {name} fifth, held by the guarantee"),
                    subjects: band.iter().filter(|s| s.population.held()).count(),
                    figure: held,
                });
            }
            Regions::framed(
                ENROLLMENT_TERM,
                PER_PUPIL_TERM,
                Unit::Ratio,
                ORIGINS.to_vec(),
                origin_boundaries(),
                bands
                    .iter()
                    .zip(project::lost_pupils::FIFTHS)
                    .map(|(band, name)| Slice {
                        label: format!("The {name} fifth"),
                        marks: band.iter().map(|s| origin_mark(s)).collect(),
                    })
                    .collect(),
            )
            .counted(census, origin_unreached(i))
        },
    },
];

/// A spread and the regions it came out with on this run.
pub struct ComputedSpread {
    /// The registry entry.
    pub spread: &'static Spread,
    /// What [`Spread::compute`] returned.
    pub regions: Regions,
}

/// Run every spread in [`SPREADS`], in registry order.
#[must_use]
pub fn compute_all_spreads() -> Vec<ComputedSpread> {
    let inputs = Inputs::shared();
    SPREADS
        .iter()
        .map(|spread| ComputedSpread {
            spread,
            regions: (spread.compute)(inputs),
        })
        .collect()
}
