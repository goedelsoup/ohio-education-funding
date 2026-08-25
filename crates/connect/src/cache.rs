//! Fetching sources, caching them, and pinning them by digest.
//!
//! # Why this shells out to curl
//!
//! Every source here is served over HTTPS, and HTTPS means TLS. A TLS implementation is not
//! something to hand-write next to a DEFLATE decoder — it is the one place where doing it
//! yourself is worse than not doing it. So retrieval delegates to `curl`, which ships with
//! macOS, Windows, and every Linux distribution this is likely to run on, and which is
//! maintained by people who do that for a living.
//!
//! That is a dependency, and calling it anything else would be dishonest. It is a dependency
//! on a system binary rather than on a package registry, which is the property the workspace
//! actually wants: a checkout from years hence still builds and still computes, and only the
//! *refresh* path needs the outside world.
//!
//! # The digest manifest
//!
//! [`MANIFEST`] is committed. It records, for every source ever fetched, the SHA-256 and byte
//! length of the exact file the fixtures were built from. `verify` recomputes and compares.
//! This matters because the department reposts corrected workbooks at the same URL: without a
//! digest, "regenerated from the FY2027 calculator" names a moving target.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::registry::Source;
use crate::sha256::digest_hex;

/// Environment variable holding a contact address for the agent string.
///
/// The Bureau of Labor Statistics rejects any request whose `User-Agent` does not carry a
/// contact — with a bare 403 and no explanation, which is an afternoon to diagnose. A browser
/// string is refused too, so impersonating one does not help and would be discourteous besides.
///
/// It is read from the environment rather than compiled in because it is the operator's
/// address, not the repository's, and a personal email does not belong in a public source file.
pub const CONTACT_ENV: &str = "EDFUND_CONTACT";

/// Agent string with no contact. Enough for the department's servers; refused by the Bureau's.
pub const ANONYMOUS_AGENT: &str = "ohio-education-funding-corpus/0.1 (research)";

/// The `User-Agent` to send, incorporating [`CONTACT_ENV`] if it is set.
#[must_use]
pub fn user_agent() -> String {
    match std::env::var(CONTACT_ENV) {
        Ok(contact) if !contact.trim().is_empty() => {
            format!("ohio-education-funding-corpus/0.1 ({})", contact.trim())
        }
        _ => ANONYMOUS_AGENT.to_string(),
    }
}

/// Where the committed digest manifest lives, relative to the repository root.
pub const MANIFEST: &str = "crates/connect/source-digests.txt";

/// Where fetched files are kept, relative to the repository root. Not committed.
pub const CACHE: &str = ".cache/sources";

/// Something went wrong retrieving or checking a source.
#[derive(Debug)]
pub enum FetchError {
    /// `curl` is not installed.
    NoCurl,
    /// `curl` ran and failed.
    Transfer {
        /// Which source.
        key: String,
        /// What curl said.
        detail: String,
    },
    /// The file is not in the cache and retrieval was not requested.
    NotCached {
        /// Which source.
        key: String,
        /// Where it was looked for.
        path: PathBuf,
    },
    /// A cached file's digest does not match the committed manifest.
    DigestMismatch {
        /// Which source.
        key: String,
        /// What the manifest records.
        expected: String,
        /// What is on disk.
        actual: String,
    },
    /// A filesystem operation failed.
    Io(io::Error),
}

impl core::fmt::Display for FetchError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoCurl => write!(
                f,
                "curl is not on PATH; it is how this fetches over HTTPS (see crate docs)"
            ),
            Self::Transfer { key, detail } => write!(f, "could not fetch {key}: {detail}"),
            Self::NotCached { key, path } => write!(
                f,
                "{key} is not cached at {}; run `edfund-connect fetch {key}`",
                path.display()
            ),
            Self::DigestMismatch {
                key,
                expected,
                actual,
            } => write!(
                f,
                "{key} has changed: manifest {expected}, on disk {actual}. The publication was \
                 revised — rebuild the fixtures and read the diff before committing it."
            ),
            Self::Io(cause) => write!(f, "{cause}"),
        }
    }
}

impl std::error::Error for FetchError {}

impl From<io::Error> for FetchError {
    fn from(cause: io::Error) -> Self {
        Self::Io(cause)
    }
}

/// The repository root, as compiled in.
///
/// `CARGO_MANIFEST_DIR` is `<repo>/crates/connect`, so the root is two levels up. Baking it in
/// keeps the CLI runnable from anywhere; `--repo` overrides it.
#[must_use]
pub fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap_or(Path::new("."))
        .to_path_buf()
}

/// Where a source is cached under `root`.
#[must_use]
pub fn cached_path(root: &Path, source: &Source) -> PathBuf {
    root.join(CACHE).join(source.filename)
}

/// Return the cached file, downloading it if absent or if `refresh` is set.
///
/// # Errors
///
/// Returns [`FetchError`] if curl is missing, the transfer fails, or the cache directory
/// cannot be created.
pub fn fetch(root: &Path, source: &Source, refresh: bool) -> Result<PathBuf, FetchError> {
    let destination = cached_path(root, source);
    if destination.exists() && !refresh {
        return Ok(destination);
    }
    fs::create_dir_all(destination.parent().unwrap_or(root))?;

    // Download beside the target and rename on success, so an interrupted transfer never
    // leaves a truncated file that later runs would treat as cached.
    let partial = destination.with_extension("partial");
    let output = Command::new("curl")
        .args([
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--max-time",
            "600",
            "--user-agent",
        ])
        .arg(user_agent())
        .args(["--output"])
        .arg(&partial)
        .arg(source.url)
        .output()
        .map_err(|cause| {
            if cause.kind() == io::ErrorKind::NotFound {
                FetchError::NoCurl
            } else {
                FetchError::Io(cause)
            }
        })?;

    if !output.status.success() {
        let _ = fs::remove_file(&partial);
        let mut detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if detail.contains("403") && !user_agent().contains('@') {
            detail.push_str(&format!(
                " — some publishers (the Bureau of Labor Statistics among them) refuse \
                 requests without a contact address. Set {CONTACT_ENV} to an email and retry."
            ));
        }
        return Err(FetchError::Transfer {
            key: source.key.to_string(),
            detail,
        });
    }
    fs::rename(&partial, &destination)?;
    Ok(destination)
}

/// Read a cached source's bytes, without going to the network.
///
/// # Errors
///
/// Returns [`FetchError::NotCached`] if it has not been fetched, naming the command that would.
pub fn read_cached(root: &Path, source: &Source) -> Result<Vec<u8>, FetchError> {
    let path = cached_path(root, source);
    if !path.exists() {
        return Err(FetchError::NotCached {
            key: source.key.to_string(),
            path,
        });
    }
    Ok(fs::read(path)?)
}

/// One line of the digest manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest {
    /// The source key.
    pub key: String,
    /// SHA-256 of the retrieved bytes, hexadecimal.
    pub sha256: String,
    /// Length of the retrieved bytes.
    pub bytes: u64,
}

/// Parse a digest manifest.
///
/// Format is one record per line: `<sha256>  <bytes>  <key>`. Blank lines and `#` comments are
/// ignored, and a malformed line is skipped rather than failing the run — a manifest is a
/// record, not a lock file, and a corrupt line should not stop an extraction.
#[must_use]
pub fn parse_manifest(text: &str) -> Vec<Digest> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let sha256 = fields.next()?.to_string();
            let bytes = fields.next()?.parse().ok()?;
            let key = fields.next()?.to_string();
            Some(Digest { key, sha256, bytes })
        })
        .collect()
}

/// Render a digest manifest.
///
/// Sorted by key so the committed file is stable and a diff shows only what changed.
#[must_use]
pub fn render_manifest(digests: &[Digest]) -> String {
    let mut sorted: Vec<&Digest> = digests.iter().collect();
    sorted.sort_by(|a, b| a.key.cmp(&b.key));
    let mut out = String::from(
        "# SHA-256 and byte length of the exact published files the committed fixtures were\n\
         # built from. Regenerate with `edfund-connect verify --write`. A changed digest means\n\
         # the publication was revised: rebuild, read the diff, and say so in the commit.\n",
    );
    for digest in sorted {
        out.push_str(&format!(
            "{}  {}  {}\n",
            digest.sha256, digest.bytes, digest.key
        ));
    }
    out
}

/// Compute the digest of a cached source.
///
/// # Errors
///
/// Returns [`FetchError::NotCached`] if the source has not been fetched.
pub fn digest_of(root: &Path, source: &Source) -> Result<Digest, FetchError> {
    let bytes = read_cached(root, source)?;
    Ok(Digest {
        key: source.key.to_string(),
        sha256: digest_hex(&bytes),
        bytes: bytes.len() as u64,
    })
}

/// Check a computed digest against the manifest.
///
/// A source the manifest does not mention passes: it has simply never been pinned. A source it
/// does mention must match exactly.
///
/// # Errors
///
/// Returns [`FetchError::DigestMismatch`] if the manifest records a different digest.
pub fn check_against(manifest: &[Digest], computed: &Digest) -> Result<(), FetchError> {
    let Some(recorded) = manifest.iter().find(|d| d.key == computed.key) else {
        return Ok(());
    };
    if recorded.sha256 == computed.sha256 {
        return Ok(());
    }
    Err(FetchError::DigestMismatch {
        key: computed.key.clone(),
        expected: recorded.sha256.clone(),
        actual: computed.sha256.clone(),
    })
}

/// Extract the text layer of a cached PDF.
///
/// # Why this shells out, and why it is a weaker argument than the one for `curl`
///
/// The module header justifies delegating retrieval to `curl` on the grounds that it ships with
/// macOS, Windows and every Linux distribution, so a checkout years hence still builds and only
/// the *refresh* path needs the outside world.
///
/// **`pdftotext` does not ship with macOS or Windows.** The second half of the argument still
/// holds — the extract is committed, so nothing but a refresh needs poppler — but the first half
/// does not, and pretending otherwise would be the kind of quiet overclaim this repository keeps
/// finding in its own notes. A caller without poppler gets [`FetchError::NotCached`]-shaped
/// behaviour: the rebuild reports the fixture skipped and says why, rather than failing.
///
/// That last sentence was itself the quiet overclaim the one before it warns about. It held for
/// eleven of the twelve PDF-backed fixtures; `appropriation-lines.csv` was regenerated **2,083
/// rows short** and reported as *written*, because the greenbooks behind its earliest era were
/// gathered leniently. `connect::greenbook_texts` is what makes the sentence true, and it says
/// so at more length there.
///
/// The alternative is a PDF text extractor in pure std. It is genuinely feasible here —
/// `spreadsheet` already carries a DEFLATE decoder, which is the hard half — and it is a
/// disproportionate amount of surface for one document. If the redbook series is ever wired, that
/// calculation changes and this should be revisited.
///
/// # Errors
///
/// Returns [`FetchError::NotCached`] if the file is absent, or an I/O error carrying `pdftotext`'s
/// own message if it is not installed or cannot read the file.
pub fn pdf_text(root: &Path, source: &Source) -> Result<String, FetchError> {
    let path = cached_path(root, source);
    if !path.exists() {
        return Err(FetchError::NotCached {
            key: source.key.to_string(),
            path,
        });
    }
    let output = Command::new("pdftotext")
        .arg("-layout")
        .arg(&path)
        .arg("-")
        .output()
        .map_err(|cause| {
            FetchError::Io(io::Error::new(
                cause.kind(),
                format!("pdftotext is required to read {}: {cause}", source.key),
            ))
        })?;
    if !output.status.success() {
        return Err(FetchError::Io(io::Error::other(format!(
            "pdftotext failed on {}: {}",
            source.key,
            String::from_utf8_lossy(&output.stderr).trim()
        ))));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(key: &str, sha: &str) -> Digest {
        Digest {
            key: key.into(),
            sha256: sha.into(),
            bytes: 10,
        }
    }

    #[test]
    fn a_manifest_round_trips() {
        let digests = vec![
            digest("cupp-fy24", "aaaa"),
            digest("fy27-calculator", "bbbb"),
        ];
        let parsed = parse_manifest(&render_manifest(&digests));
        assert_eq!(parsed, digests);
    }

    #[test]
    fn the_manifest_is_written_in_key_order() {
        let text = render_manifest(&[digest("zzz", "1"), digest("aaa", "2")]);
        let keys: Vec<String> = parse_manifest(&text).into_iter().map(|d| d.key).collect();
        assert_eq!(keys, ["aaa", "zzz"]);
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let parsed = parse_manifest("# a note\n\n  \nabc  10  key\n");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].key, "key");
    }

    #[test]
    fn a_malformed_line_is_skipped_rather_than_fatal() {
        let parsed = parse_manifest("abc  10  good\nnot-a-record\nxyz  notanumber  bad\n");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].key, "good");
    }

    #[test]
    fn a_source_the_manifest_does_not_pin_passes() {
        assert!(check_against(&[], &digest("new-source", "abc")).is_ok());
    }

    #[test]
    fn a_changed_digest_is_reported_with_both_values() {
        let manifest = vec![digest("cupp-fy24", "old")];
        let err = check_against(&manifest, &digest("cupp-fy24", "new")).unwrap_err();
        let text = err.to_string();
        assert!(text.contains("old") && text.contains("new"), "{text}");
        assert!(
            text.contains("revised"),
            "the message should say what a mismatch means, not just that it happened"
        );
    }

    #[test]
    fn the_repository_root_holds_the_workspace() {
        assert!(repository_root().join("crates/Cargo.toml").exists());
    }
}
