//! The winners-and-losers table for the two levers Ohio argues about.
//!
//! Run with `cargo run --example refresh_incidence -p scenario-delta`. The runner is the result:
//! it is deterministic and dependency-free, so re-running it reproduces these figures exactly and
//! there is no output artifact to drift from the code that made it.
//!
//! Everything printed here is asserted in `tests/who_a_change_reaches.rs`. This exists to make
//! the shape legible, not to establish it.

use project::panel::{panel, DistrictRecord};
use project::policy::{GuaranteeRule, Policy};
use scenario_delta::{across, Axis, Incidence, ScenarioDelta};

/// The statewide computed base cost increase the department's FY2027 model gives for refreshing
/// the classroom teacher salary to FY2024. Verified in `foundation`.
const COMPUTED_INCREASE: f64 = 465.0e6;

fn main() {
    let districts = panel();
    let aggregate: f64 = districts.iter().map(|r| r.aggregate_base_cost).sum();
    let scale = 1.0 + COMPUTED_INCREASE / aggregate;

    report(
        "A BASE COST INCREASE",
        &format!(
            "a uniform {:.3}% rise, sized to the department's $465.0M computed increase",
            (scale - 1.0) * 100.0
        ),
        &ScenarioDelta::between(
            &districts,
            &Policy::current_law(),
            &Policy {
                base_cost_scale: scale,
                ..Policy::current_law()
            },
        ),
    );

    report(
        "REMOVING THE GUARANTEE",
        "every district funded by the formula alone",
        &ScenarioDelta::between(
            &districts,
            &Policy::current_law(),
            &Policy {
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
        ),
    );

    decomposition(&districts, scale);
}

fn report(title: &str, subtitle: &str, delta: &ScenarioDelta) {
    println!("\n{title}");
    println!("{subtitle}\n");

    // The total and its reach arrive together; there is no way to print one without the other.
    let total = delta.total();
    println!(
        "  {:+.1}M statewide, {:+.2} per pupil",
        total.dollars / 1e6,
        total.per_pupil()
    );
    println!(
        "  {} gain, {} lose, {} untouched — of which {} are paid by the guarantee under both",
        total.reach.gainers, total.reach.losers, total.reach.unmoved, total.reach.held_throughout
    );
    if total.reach.lifted_off > 0 {
        println!("  {} lifted off the guarantee", total.reach.lifted_off);
    }

    // Both orderings, because the same policy reads differently through each.
    let ordering = delta.orderings();
    println!("\n  largest movers, by dollars        | by dollars per pupil");
    for rank in 0..5 {
        let dollars = ordering.by_dollars[rank];
        let per_pupil = ordering.by_per_pupil[rank];
        println!(
            "  {:<24.24} {:>9.1}k | {:<24.24} {:>9.2}",
            dollars.name,
            dollars.dollars() / 1e3,
            per_pupil.name,
            per_pupil.per_pupil()
        );
    }

    band(&across(&delta.deltas, &Axis::valuation(), |d| {
        d.valuation_per_pupil
    }));
    band(&across(&delta.deltas, &Axis::state_share(), |d| {
        Some(d.state_share_fraction)
    }));
}

fn band(incidence: &Incidence) {
    println!("\n  incidence across {}", incidence.axis.label);
    for stratum in &incidence.strata {
        println!(
            "    {}  n={:<4} {:>12.4} to {:>12.4}   {:>9.2}/pupil   {:>3} held",
            stratum.label,
            stratum.districts,
            stratum.lower,
            stratum.upper,
            stratum.per_pupil(),
            stratum.held_throughout
        );
    }
    if incidence.unclassified.districts > 0 {
        println!(
            "    unplaced on this axis: {} districts, {:+.0}",
            incidence.unclassified.districts, incidence.unclassified.dollars
        );
    }
    match incidence.gradient() {
        Some(gradient) => println!("    top band / bottom band, per pupil: {gradient:.3}"),
        None => println!("    no gradient: the bottom band receives nothing"),
    }
}

/// Where a computed base cost increase goes on its way to being state aid.
fn decomposition(districts: &[DistrictRecord], scale: f64) {
    let bump = scale - 1.0;
    let law = Policy::current_law();
    let scaled = Policy {
        base_cost_scale: scale,
        ..law
    };
    let unguaranteed = Policy {
        guarantee: GuaranteeRule::Removed,
        ..law
    };

    let computed: f64 = districts.iter().map(|r| r.aggregate_base_cost * bump).sum();
    let full_pass: f64 = districts
        .iter()
        .map(|r| r.base_cost_per_pupil * bump * r.current_year_adm)
        .sum();
    let formula = ScenarioDelta::between(
        districts,
        &unguaranteed,
        &Policy {
            guarantee: GuaranteeRule::Removed,
            ..scaled
        },
    )
    .total()
    .dollars;
    let delivered = ScenarioDelta::between(districts, &law, &scaled)
        .total()
        .dollars;

    println!("\nWHAT REACHES A DISTRICT, OF A $465.0M COMPUTED INCREASE");
    println!("(all figures in millions)\n");
    println!(
        "  computed base cost increase        {:8.1}",
        computed / 1e6
    );
    println!(
        "    less the ADM denominators        {:8.1}   base cost averages three years; the",
        -(computed - full_pass) / 1e6
    );
    println!("                                                state share is paid on this one");
    println!(
        "    less the minimum state share     {:8.1}   138 districts keep a tenth of it",
        -(full_pass - formula) / 1e6
    );
    println!(
        "    less the guarantee               {:8.1}   256 districts keep none of it",
        -(formula - delivered) / 1e6
    );
    println!(
        "  delivered as state aid             {:8.1}   {:.1}% of the computed increase",
        delivered / 1e6,
        delivered / computed * 100.0
    );
}
