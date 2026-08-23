//! The argument vocabulary of `edfund-project`, kept where it can be tested.
//!
//! # Why an unknown flag has to be an error
//!
//! The binary's levers all default to current law, and its argument reader looked each one up
//! by name — so a flag it did not recognise set nothing and was never noticed. `--min-shares
//! 0.15`, one letter from the real `--min-share`, ran the model at current law and printed
//!
//! ```text
//! total state aid                   $7281M -> $7281M   +0.0M
//! districts                         0 up, 0 down, 609 unmoved
//! ```
//!
//! and exited 0. That output is not a harmless no-op. The corpus names this binary as the
//! reproduction command for `[verified]` scenario figures, so the reading it invites is "the
//! scenario I asked for moves nothing" when what happened is "the scenario I asked for never
//! ran". A wrong answer that announces itself is recoverable; this one looks exactly like a
//! result, and looks *most* like one to the person checking a published figure.
//!
//! It is the same failure [`crate::drafts::Priced::cost`] returns `Option` to prevent — zero
//! and not-computed are opposite claims — reintroduced at the argument layer, and it is why
//! `--draft` with no slug is rejected there rather than falling through to this path.

/// Every argument the tool accepts, and whether it takes a value.
///
/// Exhaustive on purpose: [`check`] refuses anything absent from it, so a flag added to the
/// binary and not added here is rejected rather than silently ignored — the loud direction of
/// the two.
pub const FLAGS: [(&str, bool); 13] = [
    ("-h", false),
    ("--help", false),
    ("--json", false),
    ("--drafts", false),
    ("--draft", true),
    ("--guarantee", true),
    ("--base-cost", true),
    ("--min-share", true),
    ("--phase-in", true),
    ("--phase-in-cat", true),
    ("--through", true),
    ("--method", true),
    ("--districts", true),
];

/// Refuse any argument this tool does not define.
///
/// # Errors
///
/// Returns the offending argument, and a suggestion when a defined flag is close to it.
pub fn check(args: &[String]) -> Result<(), String> {
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        // There are no positional arguments, so a bare word is as much a mistake as an unknown
        // flag and is reported the same way.
        let Some(&(_, takes_value)) = FLAGS.iter().find(|(name, _)| *name == arg) else {
            let near: Vec<&str> = FLAGS
                .iter()
                .map(|(name, _)| *name)
                .filter(|name| resembles(name, arg))
                .collect();
            let hint = if near.is_empty() {
                "; --help lists every flag".to_string()
            } else {
                format!("; did you mean {}?", near.join(" or "))
            };
            return Err(format!("unknown argument {arg:?}{hint}"));
        };
        // A flag's value is skipped rather than inspected, so `--guarantee --json` reaches the
        // parser that wants a rule instead of being misread here as two flags. A trailing flag
        // whose value is missing runs off the end and is caught where the value is read.
        index += 1 + usize::from(takes_value);
    }
    Ok(())
}

/// Whether a defined flag is close enough to a typed one to suggest.
///
/// Deliberately crude. A shared prefix in either direction covers what typos of these flags
/// actually look like — the accidental plural, the truncation, the half-remembered name — and
/// a suggestion that is merely unhelpful costs nothing, because the argument is refused either
/// way.
fn resembles(defined: &str, typed: &str) -> bool {
    let defined = defined.trim_start_matches('-');
    let typed = typed.trim_start_matches('-');
    !typed.is_empty() && (defined.starts_with(typed) || typed.starts_with(defined))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn the_lever_that_ran_current_law_under_another_name_is_refused() {
        let error = check(&args("--min-shares 0.15")).unwrap_err();
        assert!(error.contains("--min-shares"), "{error}");
        assert!(error.contains("--min-share"), "{error}");
    }

    #[test]
    fn every_defined_flag_is_accepted_with_and_without_its_value() {
        for (name, takes_value) in FLAGS {
            let line = if takes_value {
                format!("{name} 1")
            } else {
                name.to_string()
            };
            assert!(check(&args(&line)).is_ok(), "{name} was refused");
        }
    }

    #[test]
    fn a_flags_value_is_never_read_as_a_flag() {
        // `damped` is not a flag and must not be checked as one; nor is a value that happens to
        // look like one, which is how a missing value would otherwise be misdiagnosed.
        assert!(check(&args("--method damped --json")).is_ok());
        assert!(check(&args("--guarantee --json")).is_ok());
        assert!(check(&args("--through FY2030 --districts 3")).is_ok());
    }

    #[test]
    fn a_bare_word_is_refused_as_loudly_as_an_unknown_flag() {
        let error = check(&args("min-share 0.15")).unwrap_err();
        assert!(error.contains("min-share"), "{error}");
        // No positional argument exists to absorb it.
        assert!(check(&args("hb-643-136-introduced")).is_err());
    }

    #[test]
    fn an_argument_resembling_nothing_is_pointed_at_the_help() {
        let error = check(&args("--sector-count")).unwrap_err();
        assert!(error.contains("--help lists every flag"), "{error}");
    }

    #[test]
    fn a_trailing_flag_with_no_value_is_left_to_the_parser_that_wants_it() {
        // Not this function's error to raise: `--draft` alone has a message of its own, and
        // reporting "unknown argument" for a flag that is defined would be wrong.
        assert!(check(&args("--draft")).is_ok());
        assert!(check(&args("--min-share")).is_ok());
    }

    /// The table and the usage text are one list, and neither may quietly outgrow the other.
    #[test]
    fn every_flag_the_usage_text_names_is_in_the_table() {
        let usage = include_str!("main.rs");
        for (name, _) in FLAGS {
            assert!(
                usage.contains(name),
                "{name} is in the table and not the binary"
            );
        }
    }
}
