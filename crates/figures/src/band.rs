//! The sixth chart registry: [`BANDS`], one row per step of an ordered index, each running
//! between two values whose ratio is the finding.
//!
//! Drawn on a log axis, where a row's length is that ratio and its position is its level — which
//! is what keeps a set of gaps from turning into a statement about district size. [`Ranges`]
//! holds the rows, the [`Marker`]s the plan puts on the index, the [`Aggregate`] the picture is
//! captioned with, and the floors the source cannot see.

use super::*;

/// One band chart the wiki draws: an ordered index, two values at every step, and the ratio
/// between them as the thing the picture is for.
///
/// The sixth drawing, and the one whose geometry is its argument. `web/src/lib/plot/spec.ts`'s
/// `rangeSpec` puts the measure on a **logarithmic** axis, where a row's length is its ratio and
/// its position is its level. That is not a presentation choice here, it is the finding: the
/// smallest sixth of districts employ 6.25 administrators against 5.23 funded and the largest
/// employ 39 against 20.69, so on a linear axis the second row is eighteen times the first row's
/// length and the picture says *large districts have a large gap* — which is a statement about
/// district size. On a log axis the first row is the short one, because 1.23 is a smaller ratio
/// than 1.82, and the picture says what the numbers say.
///
/// # Why not a curve, and why not a column
///
/// A [`Curve`]'s index is monotone too, and two lines over it would carry the same two values.
/// But a curve's claim is what happens *along* the index and the lines are read against a
/// reference; here the two values at one step are read against **each other**, and the reference
/// a reader wants is the ratio of one, which is not a line on a chart of position counts. A
/// [`Series`] would draw twelve bars in six pairs, which is the presentation a shape survives
/// least well — and a column is categorical, so nothing in it would say that the order is the
/// order of the thing the gap is supposed to vary with.
///
/// # What stands in for the endpoint rule
///
/// Three things, and #447 names all three.
///
/// - **The first and last row, both ends.** An ordered picture is quoted by where it starts and
///   where it finishes, which is [`Curve`]'s rule read on a shape whose steps have two values
///   rather than one. The rows between are held by `mise run //:generated` diffing the committed
///   document, exactly as a column's middle rows are.
/// - **Every [`Marker`].** A marker is a threshold in the plan drawn on the index, and a
///   threshold is a parameter the corpus quotes in prose rather than an arithmetic definition —
///   which is what separates it from a [`Curve`]'s reference or a [`Boundary`], both of which
///   name no figure. `1,500 ADM` is a number `foundation::minimums` holds and the prose repeats,
///   so binding it is what ties the sentence to the statute.
/// - **The [`Aggregate`].** The caption's one number, which on this chart is the sentence the
///   whole picture is drawn for and is on none of its rows.
pub struct Band {
    /// `<crate-directory>/<what-it-is>`, in the one key namespace the other registries share.
    pub key: &'static str,
    /// The crate that owns the computation, as the corpus cites it.
    pub owner: &'static str,
    /// What the whole drawing is, in words; its title.
    pub label: &'static str,
    /// What one row is, singular and lower case — "sextile". Written into the caption.
    pub subject: &'static str,
    /// What the two ends of a row are, low first.
    ///
    /// The legend, and it is not optional. A row is two shades of one hue rather than two hues,
    /// because the ends are one measure at two points and not two kinds of thing — which
    /// leaves nothing on the picture saying which shade is which. The rule is stated on the
    /// `ORDINAL` ramp in `web/src/lib/plot/tokens.ts`: a chart using it carries a legend.
    pub ends: [&'static str; 2],
    /// The computation, over the same inputs the figures use.
    pub compute: fn(&Inputs) -> Ranges,
}

/// One row of a [`Band`]: a step of the ordered index, and the two values whose ratio it draws.
pub struct Span {
    /// What this step is, as the index writes it. Short: the row is fourteen pixels tall and its
    /// label is drawn in a capped gutter rather than wrapped, so a long one is drawn cut.
    pub label: String,
    /// What a reader hovering the row should be told — both values, the ratio, and how many
    /// subjects the step holds, since a median of 101 districts is not a district.
    pub hover: String,
    /// The low end, which is the first of the two shades.
    pub low: f64,
    /// The high end.
    pub high: f64,
    /// The [`Figure`] that pins [`Self::low`]. Required on the first and last row.
    pub low_figure: Option<&'static str>,
    /// And the one that pins [`Self::high`].
    pub high_figure: Option<&'static str>,
}

/// A position along a [`Band`]'s ordered index where a rule in the plan changes.
///
/// [`Self::at`] is in **rows**, and fractional on purpose. R.C. 3317.011(F)(3)'s threshold is
/// 1,500 ADM, and 1,500 ADM does not fall on a sextile boundary: 319 of 606 districts are under
/// it and the sextiles hold 101 each, so it sits sixteen districts into the fourth. Rounding it
/// to the nearest boundary would move it past sixteen districts to make a cleaner picture, which
/// is the one thing a threshold drawn on a chart must not do.
///
/// Unlike a [`Boundary`] this names a figure, and the difference is what the number is. A
/// boundary is arithmetic — `y = -x` is where a multiple is one — and a pin duplicating a
/// definition checks nothing. A marker is a **parameter of the plan**, held in a crate and
/// quoted in prose, so pinning it is what makes the prose's copy of it checkable.
pub struct Marker {
    /// What the position means, in words, with the value written the way prose writes it.
    pub label: String,
    /// Where it sits on the index, in the index's own units.
    pub value: f64,
    /// Where it sits along the rows, in rows: `3.16` is sixteen per cent of the way through the
    /// fourth. An integer falls exactly between two rows.
    pub at: f64,
    /// The [`Figure`] that pins [`Self::value`].
    pub figure: &'static str,
}

/// The one number a [`Band`] is captioned with, which is on none of its rows.
///
/// A band chart's rows are medians of sub-populations and the sentence it is drawn for is usually
/// about the whole one — "districts employ about twice what the build-up funds" is a ratio of two
/// sums, and no row carries it. Stated beside the picture rather than drawn on it, because on
/// this measure the whole population's two values are three orders of magnitude above the rows'
/// and a seventh row holding them would leave the other six in a corner of the axis.
pub struct Aggregate {
    /// The sentence, with the value written into it.
    pub label: String,
    /// The value, in whatever unit the sentence states it in — a ratio here, and the [`Figure`]
    /// is what says so.
    pub value: f64,
    /// The [`Figure`] that pins [`Self::value`].
    pub figure: &'static str,
}

/// The rows of a [`Band`], the axis they are drawn on, and the three things that pin them.
pub struct Ranges {
    /// The measure both ends are on, always logarithmic. See [`Band`] for why that is the design
    /// and not a setting.
    pub measure: Axis,
    /// The rows, in index order.
    pub spans: Vec<Span>,
    /// The positions in the plan drawn across them.
    pub markers: Vec<Marker>,
    /// The caption's number.
    pub whole: Aggregate,
    /// What the picture cannot show, named under it rather than dropped.
    ///
    /// The same shape a [`Spread`] uses for the subjects its census could not place, put to the
    /// other use the rule covers: here it is the **floors the source cannot see**. A chart of one
    /// of R.C. 3317.011's seven binding staffing floors, captioned as a chart of staffing, would
    /// overstate what the District Profile Report supports — so the other six are named on it.
    pub unreached: Vec<Missing>,
}

impl Ranges {
    /// Frame a set of rows on the log axis their ratios are read off.
    ///
    /// The axis is logarithmic unconditionally and every value must therefore be above zero. A
    /// band chart whose lengths are differences rather than ratios is a different picture making
    /// a different claim, and it should be a different shape rather than this one with a flag
    /// turned off. No padding, for the reason `rangeSpec` gives: padding a log domain by a share
    /// of its span is not a thing the span means.
    ///
    /// # Panics
    ///
    /// If there are fewer than two rows, a value is not finite or not above zero, a row's low end
    /// is above its high end, or the first or last row leaves one of its two ends unpinned.
    #[must_use]
    pub fn framed(measure: &'static str, unit: Unit, spans: Vec<Span>) -> Self {
        assert!(
            spans.len() >= 2,
            "a band chart of {} row(s), which is a value rather than a shape",
            spans.len()
        );
        let (mut low, mut high) = (f64::MAX, f64::MIN);
        for span in &spans {
            assert!(
                span.low.is_finite() && span.high.is_finite(),
                "{:?}: {} to {} cannot be drawn",
                span.label,
                span.low,
                span.high
            );
            assert!(
                span.low > 0.0,
                "{:?}: {} is not a position a log axis can place",
                span.label,
                span.low
            );
            assert!(
                span.low <= span.high,
                "{:?}: the low end {} is above the high end {}",
                span.label,
                span.low,
                span.high
            );
            low = low.min(span.low);
            high = high.max(span.high);
        }
        for end in [spans.first(), spans.last()] {
            let span = end.expect("the row count is checked above");
            assert!(
                span.low_figure.is_some() && span.high_figure.is_some(),
                "{:?} is an end of the chart and leaves one of its two values unpinned",
                span.label
            );
        }
        Self {
            measure: Axis {
                label: measure,
                unit,
                min: low,
                max: high,
                log: true,
            },
            spans,
            markers: Vec::new(),
            whole: Aggregate {
                label: String::new(),
                value: f64::NAN,
                figure: "",
            },
            unreached: Vec::new(),
        }
    }

    /// Apply the rest of the binding rule: the positions drawn on the index, the caption's
    /// number, and what the picture cannot show.
    ///
    /// One step rather than three because they are one rule, the way [`Regions::counted`] takes
    /// its census and its unreached together.
    ///
    /// # Panics
    ///
    /// If a marker sits outside the rows, a marker or the aggregate is not finite, or the
    /// aggregate names no figure.
    #[must_use]
    pub fn marked(
        mut self,
        markers: Vec<Marker>,
        whole: Aggregate,
        unreached: Vec<Missing>,
    ) -> Self {
        #[allow(clippy::cast_precision_loss)]
        let rows = self.spans.len() as f64;
        for marker in &markers {
            assert!(
                marker.value.is_finite(),
                "{:?}: {} cannot be drawn",
                marker.label,
                marker.value
            );
            assert!(
                marker.at >= 0.0 && marker.at <= rows,
                "{:?} sits at row {} of {rows}, which is off the picture",
                marker.label,
                marker.at
            );
            assert!(
                !marker.figure.is_empty(),
                "{:?} is a position in the plan and names no figure",
                marker.label
            );
        }
        assert!(
            whole.value.is_finite() && !whole.figure.is_empty(),
            "{:?}: a band chart's caption is one pinned number",
            whole.label
        );
        self.markers = markers;
        self.whole = whole;
        self.unreached = unreached;
        self
    }
}

/// A rounded count with thousands separators, for a label a reader reads rather than parses.
///
/// The row labels and the marker are the manifest's first computed strings carrying a number a
/// reader is meant to recognise — `5256.4043` in a fourteen-pixel gutter is not one.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn grouped(value: f64) -> String {
    let digits = value.round().abs().min(1e18) as u64;
    let digits = digits.to_string();
    let mut out = String::new();
    if value < 0.0 {
        out.push('-');
    }
    for (at, digit) in digits.char_indices() {
        if at > 0 && (digits.len() - at).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// The staffing floors of R.C. 3317.011 the District Profile Report has no column for.
///
/// Derived from [`Minimum::ALL`](foundation::minimums::Minimum::ALL) rather than listed, so a
/// floor added to the section arrives here rather than being silently absent from a picture whose
/// whole second claim is what is absent. The count is asserted because the claim is "six": if the
/// section gains a floor, or the report gains a column, this is where the negative stops being
/// true and the assertion is what says so.
///
/// (F)(6)'s leadership support is left out on the other side of the same test: it can never bind
/// — `Minimum::can_bind` is false for exactly that one — so it is not a floor the report fails to
/// see, it is a floor there is nothing to see.
///
/// # Panics
///
/// If the section no longer has exactly six binding floors outside the administrator elements.
fn floors_the_report_cannot_see() -> Vec<Missing> {
    use foundation::minimums::Minimum;
    let unseen: Vec<Missing> = Minimum::ALL
        .into_iter()
        .filter(|m| m.can_bind() && !matches!(m, Minimum::OtherAdministrators))
        .map(|m| {
            let floor = match (m.positions(), m.threshold()) {
                (Some(n), Some(adm)) => format!(
                    "a floor of {} {} under {} ADM",
                    grouped(n),
                    if n == 1.0 { "position" } else { "positions" },
                    grouped(adm)
                ),
                (Some(n), None) => format!(
                    "a floor of {} {}, on grades 9-12 rather than on enrolled ADM",
                    grouped(n),
                    if n == 1.0 { "position" } else { "positions" }
                ),
                _ => "a floor of one per open building at any enrollment".to_string(),
            };
            Missing {
                label: format!("R.C. 3317.011{}, {}", m.division(), m.label()),
                why: format!("{floor}; no column of the report counts it"),
            }
        })
        .collect();
    assert_eq!(
        unseen.len(),
        6,
        "the chart says six floors are unseen and the section now has {} that could be",
        unseen.len()
    );
    unseen
}

/// The ordinal of each sextile, for the marker's own sentence.
const SIXTHS: [&str; 6] = ["first", "second", "third", "fourth", "fifth", "sixth"];

/// The band charts the wiki draws. See [`Band`] for what stands in for the endpoint rule here.
///
/// One, and #447's: what districts employ against what R.C. 3317.011's four administrator
/// elements fund, by sextile of FY2024 enrolled ADM. Both sides are FY2024 — the employed count
/// is the District Profile Report's `FTE Number of Administrators` and the funded count is taken
/// at the report's own enrolment — and the vintage is carried onto the chart from
/// [`EMPLOYED_VINTAGE`](project::administrator_staffing::EMPLOYED_VINTAGE) rather than written
/// here, because #439 is open on whether to adopt the department's revised FY2024 report and
/// every number on this picture moves if it is adopted.
pub static BANDS: &[Band] = &[Band {
    key: "project/administrators-employed-against-administrators-funded",
    owner: "crates/project",
    label: "What districts employ against what the four administrator elements of R.C. 3317.011 \
            fund, by sextile of enrolled ADM",
    subject: "sextile",
    ends: [
        "What the four administrator elements fund",
        "What the district employs",
    ],
    compute: |i| {
        use project::administrator_staffing as staffing;
        let rows = staffing::districts(&i.profile, &i.panel);
        let bands = staffing::sextiles(&rows);
        let last = bands.len() - 1;
        let spans: Vec<Span> = bands
            .iter()
            .enumerate()
            .map(|(at, b)| Span {
                label: format!("{} pupils", grouped(b.adm)),
                hover: format!(
                    "The {} band by enrollment: {} districts around {} pupils, funded for {:.2} \
                     administrators and employing {:.2} \u{2014} {:.2}\u{d7}, or {:.1} per \
                     thousand pupils",
                    SIXTHS[at],
                    b.n,
                    grouped(b.adm),
                    b.funded,
                    b.employed,
                    b.multiple,
                    b.per_thousand,
                ),
                low: b.funded,
                high: b.employed,
                low_figure: match at {
                    0 => Some("project/administrators-funded-in-the-smallest-sextile"),
                    _ if at == last => Some("project/administrators-funded-in-the-largest-sextile"),
                    _ => None,
                },
                high_figure: match at {
                    0 => Some("project/administrators-employed-in-the-smallest-sextile"),
                    _ if at == last => {
                        Some("project/administrators-employed-in-the-largest-sextile")
                    }
                    _ => None,
                },
            })
            .collect();

        // Where 1,500 ADM falls along the rows, counted in districts rather than interpolated in
        // pupils: the rows hold equal numbers of districts, so the honest fraction through one is
        // the share of its districts under the threshold and not the share of its pupil range.
        let below = rows.iter().filter(|d| d.floored()).count();
        let (mut passed, mut place) = (0usize, None);
        for (at, b) in bands.iter().enumerate() {
            if below < passed + b.n {
                place = Some((at, below - passed, b.n));
                break;
            }
            passed += b.n;
        }
        let (which, into, held) = place.expect("the threshold falls inside the ordered rows");
        #[allow(clippy::cast_precision_loss)]
        let at = which as f64 + into as f64 / held as f64;

        let employed: f64 = rows.iter().map(|d| d.employed).sum();
        let funded: f64 = rows.iter().map(|d| d.funded.total()).sum();
        #[allow(clippy::cast_precision_loss)]
        let districts = rows.len() as f64;

        Ranges::framed(
            "Administrators, in full-time equivalents",
            Unit::Positions,
            spans,
        )
        .marked(
            vec![Marker {
                label: format!(
                    "{} pupils: under it R.C. 3317.011(F)(3) funds two other administrators \
                     whatever the enrollment. It falls {} districts into the {} band.",
                    grouped(staffing::threshold()),
                    into,
                    SIXTHS[which],
                ),
                value: staffing::threshold(),
                at,
                figure: "project/the-administrator-floors-adm-threshold",
            }],
            Aggregate {
                label: format!(
                    "{} districts employ {} FTE administrators against the {} the four \
                     administrator elements fund at their own {} enrollment: {:.4} employed for \
                     every one funded.",
                    grouped(districts),
                    grouped(employed),
                    grouped(funded),
                    staffing::EMPLOYED_VINTAGE,
                    employed / funded,
                ),
                value: employed / funded,
                figure: "project/administrators-employed-for-every-one-funded",
            },
            floors_the_report_cannot_see(),
        )
    },
}];

/// A band chart and the rows it came out with on this run.
pub struct ComputedBand {
    /// The registry entry.
    pub band: &'static Band,
    /// What [`Band::compute`] returned.
    pub ranges: Ranges,
}

/// Run every band chart in [`BANDS`], in registry order.
#[must_use]
pub fn compute_all_bands() -> Vec<ComputedBand> {
    let inputs = Inputs::shared();
    BANDS
        .iter()
        .map(|band| ComputedBand {
            band,
            ranges: (band.compute)(inputs),
        })
        .collect()
}
