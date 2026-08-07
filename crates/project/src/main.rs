//! `edfund-project` — run a policy scenario against every Ohio school district.
//!
//! A thin shell over [`project`]. Everything worth testing is in the library.

use std::process::ExitCode;

use edfund_core::FiscalYear;
use project::panel::{panel, MINIMUM_STATE_SHARE};
use project::policy::{GuaranteeRule, Policy};
use project::report::{run, Run};
use project::series::{Method, DEFAULT_DAMPING};

const USAGE: &str = "\
edfund-project — what a policy change would do to Ohio school funding

USAGE:
    edfund-project [levers] [options]

LEVERS (default: current law, which reproduces the department's own FY2027 model)
    --guarantee <rule>     as-enacted | removed | rebase:<factor> | phase-out:<remaining>
    --base-cost <scale>    multiplier on aggregate base cost, e.g. 1.031 for a FY2022 refresh
    --min-share <fraction> minimum state share; the FY2027 model uses 0.10
    --phase-in <fraction>  applied to base cost aid
    --phase-in-cat <f>     applied to categorical aid, separately — see below

OPTIONS
    --through <fy>         also forecast to this fiscal year, at projected enrollment
    --method <m>           damped (default) | cagr | linear | flat
    --districts <n>        list the n districts most affected
    --json                 machine-readable output

The two phase-in dials are separate because Ohio's were: in FY2022 the headline phase-in was
16.67% and Disadvantaged Pupil Impact Aid was phased in at 0%, so a district's realized
percentage depended on its funding mix.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    match execute(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    let at = args.iter().position(|a| a == name)?;
    args.get(at + 1).map(String::as_str)
}

fn number(args: &[String], name: &str, default: f64) -> Result<f64, String> {
    match value(args, name) {
        None => Ok(default),
        Some(raw) => raw
            .parse()
            .map_err(|_| format!("{name} wants a number, got {raw:?}")),
    }
}

fn guarantee_rule(raw: &str) -> Result<GuaranteeRule, String> {
    let (kind, argument) = raw.split_once(':').unwrap_or((raw, ""));
    let fraction = || {
        argument
            .parse::<f64>()
            .map_err(|_| format!("{kind} wants a number after the colon, got {argument:?}"))
    };
    match kind {
        "as-enacted" => Ok(GuaranteeRule::AsEnacted),
        "removed" => Ok(GuaranteeRule::Removed),
        "rebase" => Ok(GuaranteeRule::Rebased {
            factor: fraction()?,
        }),
        "phase-out" => Ok(GuaranteeRule::PhasedOut {
            remaining: fraction()?,
        }),
        other => Err(format!(
            "unknown guarantee rule {other:?}; try as-enacted, removed, rebase:0.9, phase-out:0.5"
        )),
    }
}

fn method(raw: &str) -> Result<Method, String> {
    match raw {
        "damped" => Ok(Method::Damped {
            rate: 0.0,
            damping: DEFAULT_DAMPING,
        }),
        "cagr" => Ok(Method::Cagr { rate: 0.0 }),
        "linear" => Ok(Method::LinearTrend {
            slope: 0.0,
            intercept: 0.0,
        }),
        "flat" => Ok(Method::LastObserved),
        other => Err(format!("unknown method {other:?}")),
    }
}

fn execute(args: &[String]) -> Result<(), String> {
    let policy = Policy {
        guarantee: match value(args, "--guarantee") {
            Some(raw) => guarantee_rule(raw)?,
            None => GuaranteeRule::AsEnacted,
        },
        base_cost_scale: number(args, "--base-cost", 1.0)?,
        minimum_state_share: number(args, "--min-share", MINIMUM_STATE_SHARE)?,
        phase_in_base_cost: number(args, "--phase-in", 1.0)?,
        phase_in_categorical: number(args, "--phase-in-cat", 1.0)?,
    };
    let through = match value(args, "--through") {
        Some(raw) => {
            Some(FiscalYear(raw.trim_start_matches("FY").parse().map_err(
                |_| format!("--through wants a fiscal year, got {raw:?}"),
            )?))
        }
        None => None,
    };
    let method = method(value(args, "--method").unwrap_or("damped"))?;
    let listed = number(args, "--districts", 0.0)? as usize;

    let districts = panel();
    let result = run(&districts, &policy, through, method);
    if args.iter().any(|a| a == "--json") {
        println!("{}", json(&result));
    } else {
        print_report(&result, listed);
    }
    Ok(())
}

fn millions(value: f64) -> String {
    format!("{:+.1}M", value / 1_000_000.0)
}

fn print_report(result: &Run, listed: usize) {
    let effect = &result.policy_effect;
    println!("POLICY EFFECT — observed enrollment, exact");
    println!(
        "  {:<34}${:.0}M -> ${:.0}M   {}",
        "total state aid",
        effect.baseline.realized_aid / 1_000_000.0,
        effect.policy.realized_aid / 1_000_000.0,
        millions(effect.cost())
    );
    println!(
        "  {:<34}{} up, {} down, {} unmoved",
        "districts",
        effect.gainers(),
        effect.losers(),
        effect.unmoved()
    );
    println!(
        "  {:<34}{} -> {}",
        "on the guarantee", effect.baseline.on_guarantee, effect.policy.on_guarantee
    );
    println!(
        "  {:<34}${:.0}M -> ${:.0}M",
        "guarantee cost",
        effect.baseline.guarantee / 1_000_000.0,
        effect.policy.guarantee / 1_000_000.0
    );

    if let Some(enrollment) = &result.enrollment_effect {
        println!();
        println!(
            "ENROLLMENT EFFECT — {} at projected enrollment, a forecast",
            enrollment.fiscal_year
        );
        println!(
            "  {:<34}${:.0}M  (${:.0}M to ${:.0}M)",
            "same policy, projected ADM",
            enrollment.realized_aid / 1_000_000.0,
            enrollment.low / 1_000_000.0,
            enrollment.high / 1_000_000.0
        );
        println!(
            "  {:<34}{:.0} ({:+.1}%)",
            "projected ADM",
            enrollment.adm,
            100.0 * (enrollment.adm / effect.policy.adm - 1.0)
        );
        println!(
            "  {:<34}{} -> {}",
            "on the guarantee", effect.policy.on_guarantee, enrollment.on_guarantee
        );
        println!(
            "  {:<34}{}, +/-1 sigma from the {}",
            "method",
            enrollment.method.label(),
            enrollment.prior.source
        );
        println!();
        println!("  The two are reported apart on purpose. Adding them would give the policy");
        println!("  effect the forecast's error while it kept the simulation's look.");
    }

    if listed > 0 {
        let mut sorted: Vec<&project::Outcome> = effect.outcomes.iter().collect();
        sorted.sort_by(|a, b| {
            b.delta_per_pupil()
                .abs()
                .partial_cmp(&a.delta_per_pupil().abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        println!();
        println!("MOST AFFECTED");
        println!("  {:<44}{:>12}{:>12}", "district", "$/pupil", "total");
        for outcome in sorted.iter().take(listed) {
            println!(
                "  {:<44}{:>12.0}{:>12}",
                outcome.name,
                outcome.delta_per_pupil(),
                millions(outcome.delta())
            );
        }
    }
}

fn json(result: &Run) -> String {
    let effect = &result.policy_effect;
    let mut out = String::from("{\n");
    out.push_str(&format!(
        "  \"policy_effect\": {{\"cost\": {:.2}, \"gainers\": {}, \"losers\": {}, \
         \"unmoved\": {}, \"on_guarantee\": {}, \"guarantee\": {:.2}}}",
        effect.cost(),
        effect.gainers(),
        effect.losers(),
        effect.unmoved(),
        effect.policy.on_guarantee,
        effect.policy.guarantee
    ));
    if let Some(enrollment) = &result.enrollment_effect {
        out.push_str(&format!(
            ",\n  \"enrollment_effect\": {{\"fiscal_year\": {}, \"realized_aid\": {:.2}, \
             \"low\": {:.2}, \"high\": {:.2}, \"adm\": {:.2}, \"on_guarantee\": {}, \
             \"method\": \"{}\"}}",
            enrollment.fiscal_year.0,
            enrollment.realized_aid,
            enrollment.low,
            enrollment.high,
            enrollment.adm,
            enrollment.on_guarantee,
            enrollment.method.label()
        ));
    }
    out.push_str("\n}");
    out
}
