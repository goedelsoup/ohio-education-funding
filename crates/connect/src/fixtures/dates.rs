//! Two date parsers, for two publishers that date a document differently.
//!
//! Ohio's supreme court prints the decision date inside a parenthetical that ends `.)`; the courts
//! of appeals print `Rendered on <date>` alone on a line. A reader that looks for the bracket
//! first is right about the first and, for the second, finds the next closing parenthesis anywhere
//! in the document — sixty-eight lines of holding, in the first appellate opinion wired here.

/// The month a greenbook was published, from its cover page.
///
/// The cover carries a bare `August 2013` on a line of its own, some way below the analysts' names
/// and above the table of contents. That is the date the **analysis** speaks from, which is months
/// after the act took effect and is not the act's date — a distinction worth keeping in the field
/// rather than in a comment, because a reader who takes it for the effective date is off by most
/// of a fiscal year.
///
/// Empty where no such line is found, on the same ground as [`decided_on`]: a record whose date
/// cannot be read should say nothing rather than guess.
#[must_use]
pub fn published_on(body: &str) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    // Only the cover. Searching the whole document finds a month inside the first sentence that
    // happens to name one — an effective date, a payment schedule — and dates the analysis by it.
    for line in body.lines().take(40) {
        let line = line.trim();
        let Some((month, year)) = line.split_once(' ') else {
            continue;
        };
        if MONTHS.contains(&month) && year.len() == 4 && year.bytes().all(|b| b.is_ascii_digit()) {
            return line.to_string();
        }
    }
    String::new()
}

/// The date an opinion was decided, read off the document.
///
/// The supreme court prints a parenthetical on the first page — `(No. 95-2066--Submitted
/// September 10, 1996--Decided March 24, 1997.)` — with the dash rendering as `--` in the 1997 PDF
/// and as an em dash in the later three. Only the text after `Decided ` is wanted, so the dash
/// never has to be matched.
///
/// **The courts of appeals do not print that parenthetical.** They print `Rendered on March 29,
/// 2024` on its own line instead, and the first opinion of theirs this repository wired came out
/// with an empty date field. Both markers are tried, most specific first.
///
/// Returns an empty string if neither is present, which is the honest answer: a [`super::statute::Record`] whose
/// date could not be read should say nothing rather than guess. It is worth reading at all because
/// the previous extract discarded the one date these documents actually print and wrote an empty
/// field in its place.
#[must_use]
pub fn decided_on(body: &str) -> String {
    const MARKERS: [&str; 2] = ["Decided ", "Rendered on "];
    let Some((start, marker)) = MARKERS
        .iter()
        .filter_map(|m| body.find(m).map(|at| (at, *m)))
        .min()
    else {
        return String::new();
    };
    let rest = &body[start + marker.len()..];
    // `March 24, 1997.)` — the date ends at the sentence stop that closes the parenthetical, or
    // at the end of the line, **whichever comes first**. Preferring `.)` and falling back to the
    // newline is what the supreme court's layout suggests and it is wrong: an opinion that prints
    // the date bare on its own line has no closing parenthesis, so the search runs on to the next
    // one anywhere in the document. The first appellate opinion wired here came out with
    // sixty-eight lines of holding in its date field.
    let end = [rest.find(".)"), rest.find('\n')]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(rest.len());
    rest[..end].trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ohio's two courts date an opinion differently, and one of the two has no closing bracket.
    ///
    /// The supreme court prints the date inside a parenthetical that ends `.)`. The courts of
    /// appeals print `Rendered on <date>` alone on a line. A reader that looks for `.)` first and
    /// only falls back to the line ending is right about the first and finds the *next* closing
    /// parenthesis anywhere in the document for the second — sixty-eight lines of holding, in the
    /// first appellate opinion this repository wired.
    #[test]
    fn a_date_ends_at_whichever_comes_first_of_the_bracket_and_the_line() {
        let supreme = "(No. 95-2066--Submitted September 10, 1996--Decided March 24, 1997.)\n\
                       Constitutional law—Education—Schools (and so on.)";
        assert_eq!(decided_on(supreme), "March 24, 1997");

        let appellate = "Rendered on March 29, 2024\n\nBOGGS, J.\n\
                         {¶ 1} On February 2, 2024, appellees filed the instant motion.\n\
                         The order is not final. (See R.C. 2505.02(B)(4).)";
        assert_eq!(decided_on(appellate), "March 29, 2024");

        // Neither marker present is an empty field rather than a guess.
        assert_eq!(decided_on("IN THE COURT OF APPEALS OF OHIO"), "");
    }
}
