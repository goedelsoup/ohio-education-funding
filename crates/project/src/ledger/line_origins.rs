//! Which act created each of the department's appropriation lines.
//!
//! # What this adds to the appropriation series
//!
//! [`super::appropriations`] answers how much a line was given. This answers how old it is, and
//! the two together say something neither says alone: the department's budget is not designed, it
//! is accreted. The lines in force were created by acts spanning roughly half a century, most of
//! them by legislatures that could not have known what the others would add.
//!
//! # Where the fact comes from
//!
//! The Catalog of Budget Line Items prints a `Legal Basis` for every entry, giving the sections
//! and act that authorise it **this** biennium and, in parentheses, the act that established it
//! originally — `Sections 265.10 … of H.B. 166 of the 133rd G.A. (originally established by
//! H.B. 66 of the 126th G.A.)`. The second clause is the one read here.
//!
//! Not every entry has one. Of the current edition's line items, roughly half name an
//! establishing act and the rest cite only their current authority. Those are reported as unknown
//! rather than guessed at, because a line item's number is reused — `200604` names three
//! different programmes across three funds in this series — so an origin cannot be inferred from
//! an earlier edition carrying the same number.
//!
//! # The other clause, which changes and is the point
//!
//! [`current`] reads the parenthetical and stops, for a stated reason: a line's origin does not
//! change, so reading it eighteen times is eighteen chances for one edition to disagree with
//! another. The **first** clause is the opposite case. It names the Revised Code chapter and the
//! act the line is authorised under *this* biennium, so it changes by design and the change is the
//! information — see [`authority`], which reads every edition for exactly that reason.
//!
//! What it says about Ohio is sharper than it looks. Ohio replaced its school funding formula with
//! a new chapter of the Revised Code in 2009, R.C. 3306, and **that chapter reaches exactly one
//! line of the state budget, in exactly three editions of the Catalog** — GRF 200550, Foundation
//! Funding, in 2009, 2010 and 2011 — and appears nowhere else in eighteen editions. Every other
//! education appropriation kept citing the chapter it had cited before. The Evidence-Based Model's
//! entire footprint in the appropriation record is one line item.
//!
//! # General Assemblies rather than years
//!
//! The Catalog names a General Assembly and never a date. Ohio's are consecutive and biennial, so
//! the mapping is arithmetic — but arithmetic still needs an anchor, and [`convened`] takes its
//! from the acts this corpus already holds with both facts attached. A test checks it against
//! every one of them.

use std::collections::BTreeMap;

/// The committed extract: one row per line item per edition.
const BASIS: &str = include_str!("../../fixtures/catalog-line-item-basis.tsv");

/// The header [`current`] indexes against, promoted out of the reader that used to hold it
/// inline — the only one of these that named its columns nowhere a caller could see.
///
/// Tab-delimited, not comma: a legal basis is a sentence citing sections and session laws.
const BASIS_HEADER: &str = "edition\tfund\tali\tname\tlegal_basis";

/// The columns of [`BASIS_HEADER`], named where they are read.
mod column {
    pub const EDITION: usize = 0;
    pub const FUND: usize = 1;
    pub const ALI: usize = 2;
    pub const NAME: usize = 3;
    pub const LEGAL_BASIS: usize = 4;
}

/// The year a General Assembly convened.
///
/// Ohio numbers its General Assemblies consecutively from the first, each sitting for two years
/// beginning in an odd year, so the 136th convened in 2025 and the mapping is `1753 + 2n`.
///
/// The constant is not a guess. Every act this corpus holds with both a General Assembly and a
/// year satisfies it — H.B. 920 of the 111th in 1975, H.B. 94 of the 124th in 2001, H.B. 66 of
/// the 126th in 2005, H.B. 1 of the 128th in 2009, H.B. 153 of the 129th in 2011, H.B. 110 of the
/// 134th in 2021, H.B. 33 of the 135th in 2023 and H.B. 96 of the 136th in 2025 — and
/// `the_line_origins` checks all of them rather than trusting this note.
#[must_use]
pub fn convened(general_assembly: u16) -> u16 {
    1753 + 2 * general_assembly
}

/// One appropriation line, and where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineOrigin {
    /// The fund it is paid from.
    pub fund: String,
    /// The six-digit line item number.
    pub ali: String,
    /// Its name in this edition.
    pub name: String,
    /// The act that established it, as the Catalog writes it. Empty when it names none.
    pub established_by: String,
    /// That act's General Assembly. `None` when no establishing act is named.
    pub general_assembly: Option<u16>,
    /// Whether the Catalog marks the line discontinued.
    ///
    /// The publisher's own label, and it does **not** distinguish abolition from consolidation —
    /// a line folded into another is discontinued too. See `state-foundation-aid`, where that
    /// distinction is an open question this cannot settle.
    pub discontinued: bool,
}

/// One appropriation line in one edition, and what authorises it that biennium.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineAuthority {
    /// The Catalog edition this reading comes from.
    pub edition: u16,
    /// The fund it is paid from.
    pub fund: String,
    /// The six-digit line item number.
    pub ali: String,
    /// Its name in this edition.
    pub name: String,
    /// Revised Code chapters the basis cites, in the order they appear.
    ///
    /// The Catalog writes them `ORC 3317` in the earlier editions and `R.C. 3317` from 2022, and
    /// both are read. A line can cite none — about half do, naming only session-law sections.
    pub chapters: Vec<String>,
    /// The act named as this biennium's authority, as the Catalog writes it. Empty when none.
    pub act: String,
    /// That act's General Assembly. `None` when no act is named.
    pub general_assembly: Option<u16>,
}

/// Revised Code chapters cited in one legal basis, deduplicated, in order of appearance.
///
/// The Catalog writes the marker `ORC` in the earlier editions and `R.C.` from 2022, and a single
/// marker can introduce several citations — `R.C. 121.086 and 3319.239` is two chapters and one
/// marker, so the run after the marker is read rather than only the first number. A section
/// reference is the same chapter as the section, so each is cut at the point. Only the clause
/// before the origin parenthetical is read, which is the clause that states current authority.
fn chapters(basis: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    // The parenthetical is about origin, not about this biennium's authority, and it carries
    // four-digit years that read as chapters — `established by Controlling Board on February 24,
    // 2025` gave R.C. 2025 until this line was added.
    let mut rest = basis
        .split("(originally established")
        .next()
        .unwrap_or(basis);
    while let Some(at) = rest.find("ORC ").or_else(|| rest.find("R.C. ")) {
        let marker = if rest[at..].starts_with("ORC ") { 4 } else { 5 };
        rest = &rest[at + marker..];
        // Citations run comma- or `and`-separated until something that is not one.
        for token in rest.split([',', ';']) {
            let mut consumed = false;
            for word in token.split_whitespace() {
                let digits: String = word.chars().take_while(char::is_ascii_digit).collect();
                if (3..=4).contains(&digits.len()) {
                    if !out.contains(&digits) {
                        out.push(digits);
                    }
                    consumed = true;
                } else if word != "and" {
                    break;
                }
            }
            if !consumed {
                break;
            }
        }
    }
    out
}

/// Pull `(act, general assembly)` out of the clause naming this biennium's authority.
///
/// The first act named in the basis, which is the current one; the parenthetical that
/// [`established`] reads always comes after it. Ohio writes them `Am. Sub. H.B. 66 of the 126th
/// G.A.`, so the act begins at its bill designator and takes back whatever qualifiers precede it.
fn authorised_by(basis: &str) -> (String, Option<u16>) {
    let head = basis
        .split("(originally established")
        .next()
        .unwrap_or(basis);
    let Some(end) = head.find(" G.A.") else {
        return (String::new(), None);
    };
    let cited = &head[..end + " G.A.".len()];

    let words: Vec<&str> = cited.split_whitespace().collect();
    let bill = words
        .iter()
        .position(|w| w.ends_with("H.B.") || w.ends_with("S.B."));
    let act = match bill {
        Some(mut at) => {
            // `Am.`, `Sub.`, `Amended`, `Substitute` belong to the act's name, `Sections` does not.
            while at > 0 {
                let previous = words[at - 1];
                if matches!(previous, "Am." | "Sub." | "Amended" | "Substitute") {
                    at -= 1;
                } else {
                    break;
                }
            }
            words[at..].join(" ")
        }
        None => String::new(),
    };
    let general_assembly = head[..end]
        .rsplit(' ')
        .next()
        .and_then(|token| {
            token
                .trim_end_matches(|c: char| c.is_ascii_alphabetic())
                .parse::<u16>()
                .ok()
        })
        .filter(|n| (100..200).contains(n));
    (act, general_assembly)
}

/// Every line item in every edition, with the chapter and act that authorise it there.
///
/// Every edition, unlike [`current`], because this is the clause that moves. Ordered by edition
/// and then by line item.
#[must_use]
pub fn authority() -> Vec<LineAuthority> {
    let mut out: Vec<LineAuthority> = edfund_core::csv::delimited(BASIS, BASIS_HEADER, '\t')
        .filter_map(|row| {
            let basis = row.str(column::LEGAL_BASIS);
            let (act, general_assembly) = authorised_by(basis);
            Some(LineAuthority {
                edition: row.str(column::EDITION).parse().ok()?,
                fund: row.str(column::FUND).to_string(),
                ali: row.str(column::ALI).to_string(),
                name: row.str(column::NAME).to_string(),
                chapters: chapters(basis),
                act,
                general_assembly,
            })
        })
        .collect();
    out.sort_by(|a, b| (a.edition, &a.ali).cmp(&(b.edition, &b.ali)));
    out
}

/// Which line items cite one Revised Code chapter, by edition.
///
/// Keyed by edition; the value is that edition's line item numbers, sorted. An edition with no
/// line citing the chapter is absent rather than empty, so the map's keys are the chapter's life
/// in the appropriation record.
#[must_use]
pub fn lines_under(chapter: &str) -> BTreeMap<u16, Vec<String>> {
    let mut out: BTreeMap<u16, Vec<String>> = BTreeMap::new();
    for line in authority() {
        if line.chapters.iter().any(|c| c == chapter) {
            out.entry(line.edition).or_default().push(line.ali);
        }
    }
    for alis in out.values_mut() {
        alis.sort();
        alis.dedup();
    }
    out
}

/// Pull `(act, general assembly)` out of an `originally established by` clause.
fn established(basis: &str) -> (String, Option<u16>) {
    let Some(at) = basis.find("originally established by ") else {
        return (String::new(), None);
    };
    let rest = &basis[at + "originally established by ".len()..];
    let act = rest
        .split(')')
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    // `… of the 126th G.A.` — the number before the ordinal suffix and the abbreviation.
    //
    // The act text keeps its trailing point, because the point belongs to `G.A.` rather than to
    // the sentence. Trimming it as punctuation cost a debugging pass: it left `G.A` behind, the
    // search for `" G.A."` then matched nothing, and the parser reported no establishing act for
    // any of the eighty-eight rows that plainly carry one.
    let ga = act.find(" G.A.").and_then(|end| {
        act[..end]
            .rsplit(' ')
            .next()
            .and_then(|token| {
                token
                    .trim_end_matches(|c: char| c.is_ascii_alphabetic())
                    .parse::<u16>()
                    .ok()
            })
            .filter(|n| (100..200).contains(n))
    });
    (act, ga)
}

/// Every line item in the newest edition, with its origin.
///
/// The newest edition only. Earlier ones are in the fixture and are not read here: a line's origin
/// does not change, so restating it eighteen times would be eighteen chances for one edition's
/// wording to disagree with another's and no way to adjudicate.
#[must_use]
pub fn current() -> Vec<LineOrigin> {
    // The `f.len() < 5` guard this used to carry has moved into the reader, which asserts
    // every row against the header's width rather than skipping the short ones.
    let parsed: Vec<(u16, LineOrigin)> = edfund_core::csv::delimited(BASIS, BASIS_HEADER, '\t')
        .filter_map(|row| {
            let basis = row.str(column::LEGAL_BASIS);
            let (established_by, general_assembly) = established(basis);
            Some((
                row.str(column::EDITION).parse().ok()?,
                LineOrigin {
                    fund: row.str(column::FUND).to_string(),
                    ali: row.str(column::ALI).to_string(),
                    name: row.str(column::NAME).to_string(),
                    established_by,
                    general_assembly,
                    discontinued: basis.to_lowercase().contains("discontinued line item"),
                },
            ))
        })
        .collect();

    let Some(newest) = parsed.iter().map(|(edition, _)| *edition).max() else {
        return Vec::new();
    };
    // Keyed by line item so a fund reformatting cannot produce the same line twice.
    let mut out: BTreeMap<String, LineOrigin> = BTreeMap::new();
    for (edition, origin) in parsed {
        if edition == newest {
            out.insert(origin.ali.clone(), origin);
        }
    }
    out.into_values().collect()
}
