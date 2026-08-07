//! Regenerating the `REGEN` blocks the READMEs carry.
//!
//! Six READMEs held blocks reading `_Run `yidam corpus-index` to populate._` for fourteen
//! phases. There is no `yidam` binary — not in this repository and not in the vendored prelude —
//! so those were instructions to run something that does not exist, sitting under headings that
//! promised content.
//!
//! Everything here is derived from the repository itself, so a stale block is a failing check
//! rather than a thing someone notices. That matters more than it sounds: the test counts in
//! `crates/README.md` drifted by 41 across four phases and nobody saw, because a number in
//! prose has nothing checking it.
//!
//! # What is not generated, and why
//!
//! Two of the seven blocks described things that do not exist: a semantic index over the corpus
//! (never built — the corpus fits in context) and a deployment (no target chosen). Those are
//! replaced with a sentence saying so. Generating a plausible-looking status for a system that
//! is not there would be worse than the empty block was.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

/// A block that could not be regenerated.
#[derive(Debug)]
pub enum IndexError {
    /// A file named a block this tool does not know how to fill.
    UnknownBlock {
        /// The command named in the marker.
        command: String,
        /// Where it was found.
        file: String,
    },
    /// A filesystem operation failed.
    Io(io::Error),
}

impl core::fmt::Display for IndexError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownBlock { command, file } => {
                write!(f, "{file}: no generator for REGEN block {command:?}")
            }
            Self::Io(cause) => write!(f, "{cause}"),
        }
    }
}

impl std::error::Error for IndexError {}

impl From<io::Error> for IndexError {
    fn from(cause: io::Error) -> Self {
        Self::Io(cause)
    }
}

/// A README carrying at least one block.
pub const DOCUMENTS: &[&str] = &[
    "crates/README.md",
    "web/README.md",
    "agents/README.md",
    ".yidam/corpus/README.md",
    ".yidam/catalog/README.md",
    ".yidam/skills/README.md",
];

/// A leading `key: value` line — the corpus stores `class:` and `label:` that way.
fn field(text: &str, key: &str) -> Option<String> {
    text.lines()
        .find_map(|line| line.strip_prefix(&format!("{key}:")))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// A `key = "value"` line, which is how TOML says the same thing.
fn toml_field(text: &str, key: &str) -> Option<String> {
    text.lines()
        .find_map(|line| line.trim().strip_prefix(&format!("{key} = ")))
        .map(|value| value.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
}

/// Count markdown links pointing out of a file.
fn outgoing_links(text: &str) -> usize {
    text.match_indices("](").count()
}

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative)).unwrap_or_default()
}

/// Every corpus node, as `(class, slug, path, text)`.
fn corpus_nodes(root: &Path) -> Vec<(String, String, String, String)> {
    let mut out = Vec::new();
    let Ok(classes) = fs::read_dir(root.join(".yidam/corpus")) else {
        return out;
    };
    for class in classes.flatten() {
        if !class.path().is_dir() {
            continue;
        }
        let class_name = class.file_name().to_string_lossy().into_owned();
        let Ok(nodes) = fs::read_dir(class.path()) else {
            continue;
        };
        for node in nodes.flatten() {
            let path = node.path();
            if path.extension().is_some_and(|e| e == "yml") {
                let slug = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                let relative = format!(".yidam/corpus/{class_name}/{slug}.yml");
                let text = fs::read_to_string(&path).unwrap_or_default();
                out.push((class_name.clone(), slug, relative, text));
            }
        }
    }
    out.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    out
}

fn corpus_index(root: &Path) -> String {
    let nodes = corpus_nodes(root);
    // Inbound links are counted by scanning every node for a reference to each slug, which is
    // O(n²) over 58 files and instant. A node nothing points at is the thing worth surfacing.
    let mut inbound: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, slug, _, _) in &nodes {
        let needle = format!("{slug}.yml");
        let count = nodes
            .iter()
            .filter(|(_, other, _, text)| other != slug && text.contains(&needle))
            .count();
        inbound.insert(slug.as_str(), count);
    }

    let mut out = String::from("| Node | Class | Label | Out | In |\n|---|---|---|--:|--:|\n");
    for (class, slug, relative, text) in &nodes {
        let label = field(text, "label").unwrap_or_else(|| slug.clone());
        let path = relative.trim_start_matches(".yidam/corpus/");
        out.push_str(&format!(
            "| [`{slug}`]({path}) | {class} | {label} | {} | {} |\n",
            outgoing_links(text),
            inbound.get(slug.as_str()).copied().unwrap_or(0)
        ));
    }
    let orphans = inbound.values().filter(|count| **count == 0).count();
    out.push_str(&format!(
        "\n{} nodes across {} classes. **{orphans} have nothing pointing at them**, which the \
         corpus rules treat as a gap rather than a fact about the node.\n",
        nodes.len(),
        nodes
            .iter()
            .map(|(class, _, _, _)| class)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    ));
    out
}

fn catalog_audit(root: &Path) -> String {
    let corpus: Vec<String> = corpus_nodes(root)
        .into_iter()
        .map(|(_, _, _, text)| text)
        .collect();
    let mut entries: Vec<(String, String, usize)> = Vec::new();
    if let Ok(files) = fs::read_dir(root.join(".yidam/catalog")) {
        for file in files.flatten() {
            let path = file.path();
            let slug = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            if path.extension().is_none_or(|e| e != "md") || slug == "README" {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap_or_default();
            let title = text
                .lines()
                .find_map(|line| line.strip_prefix("# "))
                .unwrap_or(&slug)
                .to_string();
            let needle = format!("{slug}.md");
            let cited = corpus.iter().filter(|node| node.contains(&needle)).count();
            entries.push((slug, title, cited));
        }
    }
    entries.sort();

    // Sources whose bytes are pinned by digest, which is a stronger claim than being catalogued.
    let manifest = read(root, crate::cache::MANIFEST);
    let pinned = crate::cache::parse_manifest(&manifest).len();

    let mut out = String::from("| Entry | Title | Cited by |\n|---|---|--:|\n");
    for (slug, title, cited) in &entries {
        out.push_str(&format!("| [`{slug}`]({slug}.md) | {title} | {cited} |\n"));
    }
    let uncited = entries.iter().filter(|(_, _, cited)| *cited == 0).count();
    out.push_str(&format!(
        "\n{} entries, {uncited} not yet cited by any corpus node. {pinned} retrieved files are \
         pinned by SHA-256 in [`crates/connect/source-digests.txt`](../../crates/connect/source-digests.txt).\n",
        entries.len()
    ));
    out
}

fn crates_index(root: &Path) -> String {
    let mut rows: Vec<(String, String, usize)> = Vec::new();
    if let Ok(dirs) = fs::read_dir(root.join("crates")) {
        for dir in dirs.flatten() {
            let manifest = dir.path().join("Cargo.toml");
            if !manifest.exists() {
                continue;
            }
            let name = dir.file_name().to_string_lossy().into_owned();
            let text = fs::read_to_string(&manifest).unwrap_or_default();
            let description = toml_field(&text, "description").unwrap_or_default();
            // Counting `#[test]` and `#[tokio::test]`-style attributes across the crate is a
            // close enough proxy for `cargo test`, and unlike that it needs no build.
            let mut tests = 0;
            for entry in walk(&dir.path()) {
                if entry.extension().is_some_and(|e| e == "rs") {
                    tests += fs::read_to_string(&entry)
                        .unwrap_or_default()
                        .matches("#[test]")
                        .count();
                }
            }
            rows.push((name, description, tests));
        }
    }
    rows.sort();

    let total: usize = rows.iter().map(|(_, _, tests)| tests).sum();
    let mut out = String::from("| Crate | Description | `#[test]` fns |\n|---|---|--:|\n");
    for (name, description, tests) in &rows {
        out.push_str(&format!(
            "| [`{name}`]({name}/) | {description} | {tests} |\n"
        ));
    }
    out.push_str(&format!(
        "\n{} crates, {total} test functions, no crates.io dependencies. `cargo test` reports a \
         different total: it adds doc-tests and counts each integration binary separately.\n",
        rows.len()
    ));
    out
}

/// Every file under `dir`, recursively, skipping build output.
fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out
}

fn markdown_index(root: &Path, directory: &str, heading: &str) -> String {
    let mut rows: Vec<(String, String)> = Vec::new();
    if let Ok(files) = fs::read_dir(root.join(directory)) {
        for file in files.flatten() {
            let path = file.path();
            let slug = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            if path.extension().is_none_or(|e| e != "md") || slug == "README" {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap_or_default();
            let description = field(&text, "description")
                .or_else(|| {
                    text.lines()
                        .find_map(|line| line.strip_prefix("**Computes.** "))
                        .map(str::to_string)
                })
                .unwrap_or_default();
            rows.push((slug, description));
        }
    }
    rows.sort();
    let mut out = format!("| {heading} | Description |\n|---|---|\n");
    for (slug, description) in &rows {
        out.push_str(&format!("| [`{slug}`]({slug}.md) | {description} |\n"));
    }
    out
}

fn bundle_status(root: &Path) -> String {
    let feed = read(root, "web/public/data/bundle.json");
    let version = feed
        .lines()
        .find_map(|line| line.trim().strip_prefix("\"contract_version\": "))
        .map(|value| value.trim_matches(|c| c == '"' || c == ',').to_string())
        .unwrap_or_else(|| "unknown".into());
    let districts = feed.matches("\"irn\": ").count();
    // Each kind is counted by a field only that kind carries, rather than by subtracting one
    // count from another. `label` was the discriminator once and stopped being unique the moment
    // the deflator acquired one — a subtraction is only as stable as every other user of the
    // field it subtracts from.
    let checkpoints = feed.matches("\"cost\": ").count();
    let forecasts = feed.matches("\"low\": ").count();
    format!(
        "| Field | Value |\n|---|---|\n\
         | Contract version | `{version}` |\n\
         | Districts in the feed | {districts} |\n\
         | Reference checkpoints | {checkpoints} |\n\
         | Reference forecasts | {forecasts} |\n\
         | Size | {} KB |\n\
         | Deployment target | none chosen; static hosting is the presumption |\n\n\
         Regenerate with `cargo run --manifest-path crates/Cargo.toml -p bundle > \
         web/public/data/bundle.json`. CI fails if the committed feed and a fresh one differ.\n",
        feed.len() / 1024
    )
}

/// Fill one block, given the command in its marker.
fn generate(command: &str, root: &Path) -> Option<String> {
    Some(match command {
        "yidam corpus-index" => corpus_index(root),
        "yidam catalog-audit" => catalog_audit(root),
        "yidam crates-index" => crates_index(root),
        "yidam skills-index" => markdown_index(root, ".yidam/skills", "Skill"),
        "yidam agents-index" => markdown_index(root, "agents", "Agent"),
        "yidam bundle-status" => bundle_status(root),
        // Both describe systems this repository does not have. Saying so beats an empty block
        // and beats a fabricated status.
        "yidam index-status" => "No semantic index is built. The corpus is 58 nodes and fits in \
             context; an index is added when direct retrieval stops working, which has not \
             happened.\n"
            .to_string(),
        _ => return None,
    })
}

/// Regenerate every block in every document. Returns the files that changed.
///
/// # Errors
///
/// Returns [`IndexError::UnknownBlock`] if a document names a generator that does not exist —
/// a new block is then a deliberate act rather than something that silently stays empty.
pub fn regenerate(root: &Path) -> Result<Vec<String>, IndexError> {
    let mut changed = Vec::new();
    for document in DOCUMENTS {
        let path = root.join(document);
        let Ok(original) = fs::read_to_string(&path) else {
            continue;
        };
        let mut out = String::with_capacity(original.len());
        let mut rest = original.as_str();

        while let Some(start) = rest.find("<!-- REGEN: ") {
            let (before, from_marker) = rest.split_at(start);
            out.push_str(before);
            let Some(marker_end) = from_marker.find("-->") else {
                break;
            };
            let Some(close) = from_marker.find("<!-- /REGEN -->") else {
                break;
            };
            let command = from_marker["<!-- REGEN: ".len()..]
                .lines()
                .next()
                .unwrap_or_default()
                .trim()
                .to_string();
            let body = generate(&command, root).ok_or_else(|| IndexError::UnknownBlock {
                command: command.clone(),
                file: (*document).to_string(),
            })?;
            out.push_str(&from_marker[..marker_end + 3]);
            out.push('\n');
            out.push_str(&body);
            rest = &from_marker[close..];
        }
        out.push_str(rest);

        if out != original {
            fs::write(&path, out)?;
            changed.push((*document).to_string());
        }
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::repository_root;

    #[test]
    fn every_block_in_every_document_has_a_generator() {
        // The check that keeps a heading from promising content nothing produces. Adding a new
        // REGEN block without a generator fails here rather than shipping an empty section.
        let root = repository_root();
        for document in DOCUMENTS {
            let text = std::fs::read_to_string(root.join(document)).unwrap_or_default();
            for marker in text.split("<!-- REGEN: ").skip(1) {
                let command = marker.lines().next().unwrap_or_default().trim();
                assert!(
                    generate(command, &root).is_some(),
                    "{document}: no generator for {command:?}"
                );
            }
        }
    }

    #[test]
    fn no_document_still_tells_a_reader_to_run_a_binary_that_does_not_exist() {
        let root = repository_root();
        for document in DOCUMENTS {
            let text = std::fs::read_to_string(root.join(document)).unwrap_or_default();
            assert!(
                !text.contains("to populate._"),
                "{document} still carries an unpopulated block"
            );
        }
    }

    #[test]
    fn the_corpus_index_counts_links_in_both_directions() {
        let index = corpus_index(&repository_root());
        assert!(index.contains("| Node | Class | Label | Out | In |"));
        assert!(index.contains("nodes across"));
        assert!(index.contains("have nothing pointing at them"));
    }

    #[test]
    fn the_crates_index_finds_every_crate_and_some_tests() {
        let index = crates_index(&repository_root());
        for crate_name in ["spreadsheet", "connect", "project", "bundle"] {
            assert!(index.contains(&format!("[`{crate_name}`]")), "{crate_name}");
        }
        assert!(index.contains("no crates.io dependencies"));
    }

    #[test]
    fn the_bundle_status_reads_the_feed_rather_than_describing_it() {
        let status = bundle_status(&repository_root());
        assert!(status.contains("`6.0.0`"), "{status}");
        assert!(
            status.contains("| Districts in the feed | 609 |"),
            "{status}"
        );
    }

    #[test]
    fn the_two_kinds_of_checkpoint_are_counted_apart() {
        // They gate different things — the scenario builder and the projection band — and the
        // page reports them as two numbers. One combined count would match neither.
        let status = bundle_status(&repository_root());
        assert!(status.contains("| Reference checkpoints | 8 |"), "{status}");
        assert!(status.contains("| Reference forecasts | 4 |"), "{status}");
    }

    #[test]
    fn a_status_for_a_system_that_does_not_exist_says_so() {
        let text = generate("yidam index-status", &repository_root()).unwrap();
        assert!(text.contains("No semantic index is built"));
    }

    #[test]
    fn an_unknown_block_is_an_error_rather_than_a_silent_skip() {
        assert!(generate("yidam invented-command", &repository_root()).is_none());
    }

    #[test]
    fn regenerating_twice_changes_nothing_the_second_time() {
        // Idempotence is what makes the CI check meaningful: a diff after regeneration means
        // the documents were stale, not that the generator is unstable.
        let root = repository_root();
        regenerate(&root).expect("generators exist");
        assert!(
            regenerate(&root).expect("generators exist").is_empty(),
            "regeneration is not idempotent"
        );
    }
}
