//! `edfund-connect` — retrieve the department's publications and rebuild the committed
//! fixtures.
//!
//! A thin shell over [`connect`]. Everything worth testing is in the library.

use std::path::PathBuf;
use std::process::ExitCode;

use connect::cache::{self, Digest};
use connect::cpi;
use connect::registry::{self, Status, CONNECTORS};
use connect::{open_workbook, rebuild};

const USAGE: &str = "\
edfund-connect — retrieval and extraction for the Ohio education funding corpus

USAGE:
    edfund-connect <command> [options]

COMMANDS:
    list                  what is retrievable, and how far each connector got
    fetch <source|--all>  download into .cache/sources (only this touches the network)
    rebuild               regenerate the committed fixtures from cached sources
    verify [--write]      check cached sources against the committed digest manifest
    cpi                   check the deflator series against the Bureau's published file
    index [--check]       regenerate the REGEN blocks in the repository's READMEs;
                          --check reports staleness without writing, and exits non-zero
    sheets <source>       list a cached workbook's sheets, for inspecting a new layout
    head <source> <sheet> [n]
                          print the first n rows of a sheet, cell by cell, with column
                          indices — how a column map is worked out when a layout changes

OPTIONS:
    --repo <path>         repository root (default: the one this binary was built in)
    --refresh             on fetch, re-download even if cached
    --check               on index, report rather than write
";

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let root = take_option(&mut args, "--repo").map_or_else(cache::repository_root, PathBuf::from);
    let refresh = take_flag(&mut args, "--refresh");
    let write = take_flag(&mut args, "--write");
    let check = take_flag(&mut args, "--check");

    let result = match args.first().map(String::as_str) {
        Some("list") => list(),
        Some("fetch") => fetch(&root, args.get(1).map(String::as_str), refresh),
        Some("rebuild") => run_rebuild(&root),
        Some("verify") => verify(&root, write),
        Some("cpi") => check_cpi(&root),
        Some("index") => regenerate_index(&root, check),
        Some("sheets") => sheets(&root, args.get(1).map(String::as_str)),
        Some("head") => head(
            &root,
            args.get(1).map(String::as_str),
            args.get(2).map(String::as_str),
            args.get(3).and_then(|n| n.parse().ok()).unwrap_or(6),
        ),
        Some(other) => Err(format!("unknown command {other:?}\n\n{USAGE}")),
        None => {
            print!("{USAGE}");
            return ExitCode::SUCCESS;
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn take_option(args: &mut Vec<String>, name: &str) -> Option<String> {
    let at = args.iter().position(|a| a == name)?;
    let value = args.get(at + 1).cloned();
    args.drain(at..=(at + 1).min(args.len() - 1));
    value
}

fn take_flag(args: &mut Vec<String>, name: &str) -> bool {
    let Some(at) = args.iter().position(|a| a == name) else {
        return false;
    };
    args.remove(at);
    true
}

fn list() -> Result<(), String> {
    println!(
        "{:<16} {:<12} {:<7} PUBLISHER",
        "CONNECTOR", "STATUS", "SOURCES"
    );
    for connector in CONNECTORS {
        println!(
            "{:<16} {:<12} {:<7} {}",
            connector.key,
            connector.status.label(),
            connector.sources.len(),
            connector.publisher
        );
        for source in connector.sources {
            println!(
                "    {:<12} {:?}  {}",
                source.key, source.format, source.filename
            );
        }
        match connector.status {
            Status::Declared { blocked_on } => println!("    blocked on: {blocked_on}"),
            // A connector can feed a fixture and still not reach everything its feeds claim.
            // Printing only the declared case would let `wired` overstate what is served.
            Status::Wired {
                still_blocked: Some(reason),
            } => println!("    still blocked on: {reason}"),
            _ => {}
        }
    }
    Ok(())
}

fn fetch(root: &std::path::Path, which: Option<&str>, refresh: bool) -> Result<(), String> {
    let targets: Vec<_> = match which {
        Some("--all") | None => registry::sources().collect(),
        Some(key) => vec![registry::source(key)
            .ok_or_else(|| format!("no source {key:?}; try `edfund-connect list`"))?],
    };
    for (connector, source) in targets {
        let path = cache::fetch(root, source, refresh).map_err(|e| e.to_string())?;
        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        println!("{:<18} {:>12} bytes  {}", source.key, size, connector.key);
    }
    Ok(())
}

fn run_rebuild(root: &std::path::Path) -> Result<(), String> {
    for rebuilt in rebuild(root).map_err(|e| e.to_string())? {
        match rebuilt {
            connect::Rebuilt::Written { path, rows } => println!("{rows:>5} rows  {path}"),
            connect::Rebuilt::Skipped { path, reason } => {
                println!("skipped  {path}\n         {reason}");
            }
        }
    }
    Ok(())
}

fn verify(root: &std::path::Path, write: bool) -> Result<(), String> {
    let manifest_path = root.join(cache::MANIFEST);
    let recorded =
        cache::parse_manifest(&std::fs::read_to_string(&manifest_path).unwrap_or_default());

    let mut computed: Vec<Digest> = Vec::new();
    let mut failures = 0;
    for (_, source) in registry::sources() {
        match cache::digest_of(root, source) {
            Ok(digest) => {
                let status = match cache::check_against(&recorded, &digest) {
                    Ok(()) => "ok",
                    Err(error) => {
                        failures += 1;
                        eprintln!("  {error}");
                        "CHANGED"
                    }
                };
                println!(
                    "{:<18} {} {:>10}  {}",
                    source.key, digest.sha256, digest.bytes, status
                );
                computed.push(digest);
            }
            // Not cached is not a failure: verification covers what is present, and `fetch`
            // says how to get the rest.
            Err(error) => println!("{:<18} {error}", source.key),
        }
    }

    if write {
        // Keep digests for sources not currently cached, so writing from a partial cache does
        // not silently drop provenance for the rest.
        let mut merged = computed.clone();
        for digest in recorded {
            if !merged.iter().any(|d| d.key == digest.key) {
                merged.push(digest);
            }
        }
        std::fs::write(&manifest_path, cache::render_manifest(&merged))
            .map_err(|e| e.to_string())?;
        println!("wrote {}", manifest_path.display());
        return Ok(());
    }

    if failures > 0 {
        return Err(format!("{failures} source(s) differ from the manifest"));
    }
    Ok(())
}

/// Regenerate every `REGEN` block from the repository itself.
fn regenerate_index(root: &std::path::Path, check: bool) -> Result<(), String> {
    // `--check` reports without writing, and is the form a gate should use. The obvious
    // alternative — regenerate, then `git diff --quiet` — cannot tell a stale block from an
    // uncommitted edit somewhere else in the same file, so it fails on any dirty tree and teaches
    // whoever hits it to ignore the gate. CI never noticed because CI checks out clean.
    let stale = if check {
        connect::index::stale(root).map_err(|e| e.to_string())?
    } else {
        connect::index::regenerate(root).map_err(|e| e.to_string())?
    };
    if stale.is_empty() {
        println!("every block is already current");
        return Ok(());
    }
    if check {
        for document in &stale {
            println!("stale: {document}");
        }
        return Err(format!(
            "{} document(s) carry a stale REGEN block. Regenerate: edfund-connect index",
            stale.len()
        ));
    }
    for document in stale {
        println!("rewrote {document}");
    }
    Ok(())
}

fn check_cpi(root: &std::path::Path) -> Result<(), String> {
    let (_, source) = registry::source("cpi-u-all-items").expect("registered");
    let bytes = cache::read_cached(root, source).map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&bytes);
    let observations = cpi::parse_series(&text, cpi::ALL_ITEMS_NSA, cpi::JUNE);
    println!(
        "{} June observations of {}",
        observations.len(),
        cpi::ALL_ITEMS_NSA
    );

    let checks = cpi::check_committed_series(&observations);
    let mut disagreements = 0;
    for check in &checks {
        let published = check
            .published
            .map_or_else(|| "-".to_string(), |v| format!("{v:.3}"));
        let verdict = if check.agrees() { "agrees" } else { "DIFFERS" };
        if !check.agrees() {
            disagreements += 1;
        }
        println!(
            "  {}  committed {:>8.3}  published {:>8}  {verdict}{}",
            check.fiscal_year,
            check.committed,
            published,
            if check.was_verified {
                ""
            } else {
                "  (was unverified)"
            }
        );
    }
    let promotable = checks
        .iter()
        .filter(|c| !c.was_verified && c.agrees())
        .count();
    println!("{promotable} transcribed point(s) confirmed against the Bureau");
    if disagreements > 0 {
        return Err(format!(
            "{disagreements} point(s) differ from the published series"
        ));
    }
    Ok(())
}

/// Print a sheet's first rows with column indices beside each populated cell.
///
/// This is the tool for the job the column maps in `fixtures.rs` were written by hand from: a
/// department sheet has sixty columns, a three-row banner, and a header on the fourth row, and
/// the only way to map it is to look at it.
fn head(
    root: &std::path::Path,
    key: Option<&str>,
    sheet: Option<&str>,
    rows: usize,
) -> Result<(), String> {
    let key = key.ok_or("usage: edfund-connect head <source> <sheet> [n]")?;
    let sheet = sheet.ok_or("usage: edfund-connect head <source> <sheet> [n]")?;
    let (_, source) = registry::source(key).ok_or_else(|| format!("no source {key:?}"))?;
    let book = open_workbook(root, source).map_err(|e| e.to_string())?;
    for (index, row) in book
        .rows_limited(sheet, rows)
        .map_err(|e| e.to_string())?
        .iter()
        .enumerate()
    {
        println!("--- row {index} ({} cells)", row.len());
        for (column, value) in row.iter().enumerate() {
            if !value.is_empty() {
                println!("  [{column:2}] {value}");
            }
        }
    }
    Ok(())
}

fn sheets(root: &std::path::Path, key: Option<&str>) -> Result<(), String> {
    let key = key.ok_or("usage: edfund-connect sheets <source>")?;
    let (_, source) = registry::source(key).ok_or_else(|| format!("no source {key:?}"))?;
    let book = open_workbook(root, source).map_err(|e| e.to_string())?;
    for name in book.sheet_names() {
        println!("{name}");
    }
    Ok(())
}
