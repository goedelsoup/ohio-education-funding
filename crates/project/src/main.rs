//! `edfund-project` — run a policy scenario against every Ohio school district.
//!
//! A thin shell over [`project`]. Everything worth testing is in the library.

use std::process::ExitCode;

use edfund_core::FiscalYear;
use project::drafts::{draft, drafts, price, Priced};
use project::panel::{panel, MINIMUM_STATE_SHARE};
use project::policy::{GuaranteeRule, Policy};
use project::report::{run, Run};
use project::series::{Method, DEFAULT_DAMPING};

const USAGE: &str = "\
edfund-project — what a policy change would do to Ohio school funding

USAGE:
    edfund-project [levers] [options]
    edfund-project --draft <slug> [options]

LEVERS (default: current law, which reproduces the department's own FY2027 model)
    --guarantee <rule>     as-enacted | removed | rebase:<factor> | phase-out:<remaining>
    --base-cost <scale>    multiplier on aggregate base cost, e.g. 1.031 for a FY2022 refresh
    --min-share <fraction> minimum state share; the FY2027 model uses 0.10
    --phase-in <fraction>  applied to base cost aid
    --phase-in-cat <f>     applied to categorical aid, separately — see below

DRAFTS
    --draft <slug>         price a draft-legislation node's provisions; --drafts lists them
    --drafts               the drafts this repository holds, and how much of each it can price

OPTIONS
    --through <fy>         also forecast to this fiscal year, at projected enrollment
    --method <m>           damped (default) | cagr | linear | flat
    --districts <n>        list the n districts most affected
    --json                 machine-readable output

The two phase-in dials are separate because Ohio's were: in FY2022 the headline phase-in was
16.67% and Disadvantaged Pupil Impact Aid was phased in at 0%, so a district's realized
percentage depended on its funding mix.

A draft's cost is always printed beside the count of its provisions this model cannot price,
because the provisions that bind to levers are exactly the ones that produce a number — so the
priced subset reported alone reads like the bill and is not.
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
    if args.iter().any(|a| a == "--drafts") {
        return list_drafts();
    }
    if args.iter().any(|a| a == "--draft") {
        // Deliberately not `if let Some(slug) = value(...)`. `value` returns `None` when `--draft`
        // is the last argument, and falling through from there runs the ordinary lever path at
        // current law — printing `+0.0M` under a command whose subject is a bill. That is the
        // "this bill is free" reading `Priced::cost` returns `Option` to prevent, reintroduced at
        // the argument layer by a truncated or shell-mangled slug.
        let slug = value(args, "--draft").ok_or("--draft wants a slug; try --drafts")?;
        // A lever flag beside `--draft` is a contradiction rather than an override: a draft's
        // provisions are its policy, so accepting one would attribute a scenario to a bill that
        // does not contain it. The scenario *page* composes the two and reports the departure in
        // the banner; there is no equivalent in a line of output, so this refuses instead.
        const LEVERS: [&str; 5] = [
            "--guarantee",
            "--base-cost",
            "--min-share",
            "--phase-in",
            "--phase-in-cat",
        ];
        if let Some(flag) = LEVERS.into_iter().find(|f| args.iter().any(|a| a == f)) {
            return Err(format!(
                "{flag} cannot be combined with --draft: a draft's provisions are its policy, and \
                 a lever set beside them would be attributed to the bill"
            ));
        }
        return price_draft(slug, args.iter().any(|a| a == "--json"));
    }
    let policy = Policy {
        guarantee: match value(args, "--guarantee") {
            Some(raw) => GuaranteeRule::parse(raw)?,
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

fn list_drafts() -> Result<(), String> {
    println!("{:<32}{:>12}{:>12}", "draft", "priced", "not priced");
    for (slug, draft) in drafts() {
        println!(
            "{:<32}{:>12}{:>12}",
            slug,
            draft.priced().len(),
            draft.unpriced().len()
        );
    }
    Ok(())
}

/// Price a draft, and print what the figure does not include whether or not anyone asked.
///
/// The unpriced block is unconditional and has no flag to suppress it. A `--brief` that dropped
/// it would recreate exactly the failure [`mod@project::drafts`] exists to prevent, and the first
/// person to want one would be someone quoting the number.
fn price_draft(slug: &str, as_json: bool) -> Result<(), String> {
    let found = draft(slug).ok_or_else(|| {
        let known: Vec<String> = drafts().into_keys().collect();
        format!(
            "no draft {slug:?}; this repository holds {}",
            known.join(", ")
        )
    })?;
    let districts = panel();
    let priced = price(&found, &districts);

    if as_json {
        println!("{}", draft_json(&priced));
        return Ok(());
    }

    println!("DRAFT — {}", priced.slug);
    println!(
        "  {:<34}{} of {} priced, {} not",
        "provisions",
        priced.attribution().len(),
        priced.provisions(),
        priced.unpriced().len()
    );
    println!();
    match priced.cost() {
        Some(_) => print_report(
            &Run {
                policy: found.policy(),
                policy_effect: priced.effect().clone(),
                enrollment_effect: None,
            },
            0,
        ),
        // Not `$0.0M`. A draft whose every provision falls outside the model produces a policy
        // identical to current law, so the arithmetic is zero and the claim is not — "free" and
        // "not priced" are opposite statements, and the second is the true one. See
        // `Priced::cost`.
        None => {
            println!("POLICY EFFECT — none computed");
            println!("  No provision of this draft binds a lever, so there is no figure to give.");
            println!("  That is not a cost of zero. It is the absence of a cost.");
        }
    }

    if let Some(residual) = priced.residual() {
        println!();
        println!("ATTRIBUTION — each provision alone, against current law");
        for entry in priced.attribution() {
            println!(
                "  {:>2}  {:<52}{:>12}",
                entry.ordinal,
                truncate(&entry.title, 52),
                millions(entry.cost)
            );
        }
        println!(
            "      {:<52}{:>12}",
            "interaction residual",
            millions(residual)
        );
        println!("      The parts do not sum to the whole. A district lifted off the guarantee by");
        println!("      one provision cannot be lifted off it again by another.");
    }

    println!();
    if priced.unpriced().is_empty() {
        println!("NOT PRICED — none. Every provision binds a lever, so the figure above is the");
        println!("  whole draft rather than the part of it this model can reach.");
    } else {
        let n = priced.unpriced().len();
        println!(
            "NOT PRICED — {n} provision{}, which {}",
            if n == 1 { "" } else { "s" },
            // The tail changes because there may be no figure to exclude them from. Saying "the
            // figure above does not include" over "POLICY EFFECT — none computed" would invite a
            // reader to go looking for the figure.
            if priced.cost().is_some() {
                "the figure above does not include"
            } else {
                "is the whole of this draft"
            }
        );
        for provision in priced.unpriced() {
            println!("  {:>2}  {}", provision.ordinal, provision.title);
            println!("      {}", provision.note);
        }
    }
    Ok(())
}

fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    text.chars()
        .take(width.saturating_sub(1))
        .collect::<String>()
        + "…"
}

fn draft_json(priced: &Priced) -> String {
    // `null` and not `0` for an unpriced draft, for the reason `Priced::cost` gives: a consumer
    // that read a zero here would publish "this bill is free" from a feed that meant "this bill
    // was not priced".
    let money =
        |value: Option<f64>| value.map_or_else(|| "null".to_string(), |v| format!("{v:.2}"));
    // Escaped for the same reason `crates/bundle` escapes: the fixture is hand-edited, and a quote
    // or a backslash in an authority column would otherwise emit invalid JSON silently.
    let quoted = |raw: &str| {
        raw.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\t', "\\t")
    };
    let mut out = format!(
        "{{\n  \"draft\": \"{}\",\n  \"cost\": {}, \"residual\": {},\n  \
         \"provisions\": {}, \"priced\": {}, \"unpriced\": {},\n  \"attribution\": [",
        quoted(&priced.slug),
        money(priced.cost()),
        money(priced.residual()),
        priced.provisions(),
        priced.attribution().len(),
        priced.unpriced().len()
    );
    for (i, entry) in priced.attribution().iter().enumerate() {
        out.push_str(&format!(
            "\n    {{\"ordinal\": {}, \"cost\": {:.2}}}{}",
            entry.ordinal,
            entry.cost,
            if i + 1 < priced.attribution().len() {
                ","
            } else {
                ""
            }
        ));
    }
    out.push_str("\n  ],\n  \"not_priced\": [");
    for (i, provision) in priced.unpriced().iter().enumerate() {
        out.push_str(&format!(
            "\n    {{\"ordinal\": {}, \"authority\": \"{}\"}}{}",
            provision.ordinal,
            quoted(&provision.authority),
            if i + 1 < priced.unpriced().len() {
                ","
            } else {
                ""
            }
        ));
    }
    out.push_str("\n  ]\n}");
    out
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
