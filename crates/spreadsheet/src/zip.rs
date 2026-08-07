//! Reading zip archives, which is what an XLSX is.
//!
//! Only the reading half of the format, and only the two storage methods that matter: stored
//! and deflated. Encryption, spanning, and every other zip feature the department does not use
//! are rejected with a named error rather than mis-parsed.
//!
//! # Why the CRC is checked
//!
//! Every member carries a CRC-32 of its uncompressed bytes. Checking it costs one pass over
//! data already in memory and turns a subtle decompression bug into a loud failure. Given that
//! [`crate::inflate`] is hand-written for this workspace, that is worth paying for: the
//! alternative is a wrong number in a fixture that no test would think to question.

use crate::inflate::{self, InflateError};

const EOCD_SIGNATURE: u32 = 0x0605_4b50;
const CENTRAL_SIGNATURE: u32 = 0x0201_4b50;
const LOCAL_SIGNATURE: u32 = 0x0403_4b50;

/// The zip comment field is a 16-bit length, so the record starts within this many bytes of
/// the end.
const MAX_COMMENT: usize = 0xFFFF;

/// An archive that could not be read.
#[derive(Debug, Clone, PartialEq)]
pub enum ZipError {
    /// No end-of-central-directory record. Not a zip, or truncated.
    NotAZip,
    /// The central directory points outside the file.
    Malformed(&'static str),
    /// A storage method other than stored (0) or deflated (8).
    UnsupportedMethod {
        /// The method code found.
        method: u16,
    },
    /// The archive uses Zip64 fields this reader does not implement.
    Zip64Unsupported,
    /// A member is encrypted.
    Encrypted {
        /// The member's name.
        name: String,
    },
    /// A member's DEFLATE stream would not decode.
    Inflate {
        /// The member's name.
        name: String,
        /// What went wrong.
        cause: InflateError,
    },
    /// A member decompressed to bytes that do not match its recorded CRC-32.
    CrcMismatch {
        /// The member's name.
        name: String,
        /// The CRC the archive declares.
        expected: u32,
        /// The CRC of what came out.
        actual: u32,
    },
    /// No member by that name.
    NoSuchEntry {
        /// The name that was asked for.
        name: String,
    },
}

impl core::fmt::Display for ZipError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotAZip => write!(f, "no end-of-central-directory record; not a zip archive"),
            Self::Malformed(what) => write!(f, "malformed archive: {what}"),
            Self::UnsupportedMethod { method } => {
                write!(f, "unsupported compression method {method}")
            }
            Self::Zip64Unsupported => write!(f, "Zip64 archive; not supported"),
            Self::Encrypted { name } => write!(f, "{name} is encrypted"),
            Self::Inflate { name, cause } => write!(f, "{name}: {cause}"),
            Self::CrcMismatch {
                name,
                expected,
                actual,
            } => write!(
                f,
                "{name}: CRC-32 {actual:08x}, archive says {expected:08x}"
            ),
            Self::NoSuchEntry { name } => write!(f, "no entry named {name}"),
        }
    }
}

impl std::error::Error for ZipError {}

/// One member of an archive, located but not yet decompressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Path within the archive, as stored.
    pub name: String,
    method: u16,
    flags: u16,
    crc32: u32,
    compressed_size: u64,
    uncompressed_size: u64,
    local_header_offset: u64,
}

impl Entry {
    /// Size of the member once decompressed, as the archive declares it.
    #[must_use]
    pub const fn uncompressed_size(&self) -> u64 {
        self.uncompressed_size
    }
}

/// An opened zip archive: the whole file in memory plus its parsed directory.
///
/// XLSX workbooks are a few megabytes and every extraction run reads several sheets, so
/// holding the bytes is simpler and faster than seeking.
#[derive(Debug, Clone)]
pub struct Archive {
    data: Vec<u8>,
    entries: Vec<Entry>,
}

fn u16_at(data: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(data.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

fn u64_at(data: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(data.get(at..at + 8)?.try_into().ok()?))
}

impl Archive {
    /// Parse an archive's central directory.
    ///
    /// # Errors
    ///
    /// Returns [`ZipError`] if the file is not a zip, is truncated, or uses Zip64.
    pub fn open(data: Vec<u8>) -> Result<Self, ZipError> {
        let eocd = find_eocd(&data).ok_or(ZipError::NotAZip)?;

        let count = u16_at(&data, eocd + 10).ok_or(ZipError::NotAZip)? as usize;
        let directory_offset = u32_at(&data, eocd + 16).ok_or(ZipError::NotAZip)? as usize;
        if count == 0xFFFF || directory_offset == 0xFFFF_FFFF {
            return Err(ZipError::Zip64Unsupported);
        }

        let mut entries = Vec::with_capacity(count);
        let mut at = directory_offset;
        for _ in 0..count {
            if u32_at(&data, at) != Some(CENTRAL_SIGNATURE) {
                return Err(ZipError::Malformed("central directory entry signature"));
            }
            let flags = u16_at(&data, at + 8).ok_or(ZipError::NotAZip)?;
            let method = u16_at(&data, at + 10).ok_or(ZipError::NotAZip)?;
            let crc32 = u32_at(&data, at + 16).ok_or(ZipError::NotAZip)?;
            let mut compressed = u64::from(u32_at(&data, at + 20).ok_or(ZipError::NotAZip)?);
            let mut uncompressed = u64::from(u32_at(&data, at + 24).ok_or(ZipError::NotAZip)?);
            let name_len = u16_at(&data, at + 28).ok_or(ZipError::NotAZip)? as usize;
            let extra_len = u16_at(&data, at + 30).ok_or(ZipError::NotAZip)? as usize;
            let comment_len = u16_at(&data, at + 32).ok_or(ZipError::NotAZip)? as usize;
            let mut local_offset = u64::from(u32_at(&data, at + 42).ok_or(ZipError::NotAZip)?);

            let name_start = at + 46;
            let name_bytes = data
                .get(name_start..name_start + name_len)
                .ok_or(ZipError::Malformed("entry name runs past end of file"))?;
            let name = String::from_utf8_lossy(name_bytes).into_owned();

            let extra_start = name_start + name_len;
            let extra = data
                .get(extra_start..extra_start + extra_len)
                .ok_or(ZipError::Malformed("extra field runs past end of file"))?;
            read_zip64_extra(extra, &mut uncompressed, &mut compressed, &mut local_offset)?;

            entries.push(Entry {
                name,
                method,
                flags,
                crc32,
                compressed_size: compressed,
                uncompressed_size: uncompressed,
                local_header_offset: local_offset,
            });
            at = extra_start + extra_len + comment_len;
        }

        Ok(Self { data, entries })
    }

    /// Every member, in central-directory order.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Whether the archive has a member with this exact name.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|e| e.name == name)
    }

    /// Decompress a member by name and verify its CRC-32.
    ///
    /// # Errors
    ///
    /// Returns [`ZipError`] if there is no such member, if it is encrypted or stored by an
    /// unsupported method, if its stream will not decode, or if the CRC disagrees.
    pub fn read(&self, name: &str) -> Result<Vec<u8>, ZipError> {
        let entry = self
            .entries
            .iter()
            .find(|e| e.name == name)
            .ok_or_else(|| ZipError::NoSuchEntry {
                name: name.to_string(),
            })?;
        self.read_entry(entry)
    }

    /// Decompress a located member and verify its CRC-32.
    ///
    /// # Errors
    ///
    /// Returns [`ZipError`] as [`Archive::read`] does.
    pub fn read_entry(&self, entry: &Entry) -> Result<Vec<u8>, ZipError> {
        // Bit 0 of the general purpose flags means the member is encrypted. Nothing else in
        // the flags changes how the bytes are read here — bit 3 moves the CRC to a trailing
        // descriptor, but the central directory copy is authoritative and that is what is used.
        if entry.flags & 1 != 0 {
            return Err(ZipError::Encrypted {
                name: entry.name.clone(),
            });
        }
        let head = usize::try_from(entry.local_header_offset)
            .map_err(|_| ZipError::Malformed("local header offset out of range"))?;
        if u32_at(&self.data, head) != Some(LOCAL_SIGNATURE) {
            return Err(ZipError::Malformed("local header signature"));
        }
        // The local header repeats the name and extra length but not reliably the sizes, so
        // only the lengths are read from it — they locate where the data begins.
        let name_len = u16_at(&self.data, head + 26).ok_or(ZipError::NotAZip)? as usize;
        let extra_len = u16_at(&self.data, head + 28).ok_or(ZipError::NotAZip)? as usize;
        let start = head + 30 + name_len + extra_len;
        let end = start
            + usize::try_from(entry.compressed_size)
                .map_err(|_| ZipError::Malformed("compressed size out of range"))?;
        let raw = self
            .data
            .get(start..end)
            .ok_or(ZipError::Malformed("member data runs past end of file"))?;

        let expected = usize::try_from(entry.uncompressed_size)
            .map_err(|_| ZipError::Malformed("uncompressed size out of range"))?;
        let out =
            match entry.method {
                0 => raw.to_vec(),
                8 => inflate::inflate_sized(raw, expected, inflate::DEFAULT_LIMIT).map_err(
                    |cause| ZipError::Inflate {
                        name: entry.name.clone(),
                        cause,
                    },
                )?,
                method => return Err(ZipError::UnsupportedMethod { method }),
            };

        let actual = crc32(&out);
        if actual != entry.crc32 {
            return Err(ZipError::CrcMismatch {
                name: entry.name.clone(),
                expected: entry.crc32,
                actual,
            });
        }
        Ok(out)
    }

    /// Decompress a member and interpret it as UTF-8.
    ///
    /// # Errors
    ///
    /// Returns [`ZipError`] as [`Archive::read`] does; invalid UTF-8 is replaced rather than
    /// rejected, because a stray byte in a district name should not fail an extraction run.
    pub fn read_to_string(&self, name: &str) -> Result<String, ZipError> {
        Ok(String::from_utf8_lossy(&self.read(name)?).into_owned())
    }
}

/// Scan backwards for the end-of-central-directory signature.
///
/// Backwards because the record sits at the very end unless there is an archive comment, and a
/// forward scan could match the signature inside compressed data.
fn find_eocd(data: &[u8]) -> Option<usize> {
    if data.len() < 22 {
        return None;
    }
    let earliest = data.len().saturating_sub(MAX_COMMENT + 22);
    for at in (earliest..=data.len() - 22).rev() {
        if u32_at(data, at) == Some(EOCD_SIGNATURE) {
            let comment_len = u16_at(data, at + 20)? as usize;
            if at + 22 + comment_len == data.len() {
                return Some(at);
            }
        }
    }
    None
}

/// Replace any size or offset the 32-bit fields could not hold with its Zip64 value.
///
/// The extra field is a sequence of `(id, size, payload)` records; the Zip64 record (id 1)
/// carries only the fields that were saturated, in a fixed order.
fn read_zip64_extra(
    extra: &[u8],
    uncompressed: &mut u64,
    compressed: &mut u64,
    local_offset: &mut u64,
) -> Result<(), ZipError> {
    let saturated = *uncompressed == u64::from(u32::MAX)
        || *compressed == u64::from(u32::MAX)
        || *local_offset == u64::from(u32::MAX);
    if !saturated {
        return Ok(());
    }
    let mut at = 0;
    while at + 4 <= extra.len() {
        let id = u16_at(extra, at).ok_or(ZipError::Zip64Unsupported)?;
        let size = u16_at(extra, at + 2).ok_or(ZipError::Zip64Unsupported)? as usize;
        let body = at + 4;
        if id == 0x0001 {
            let mut cursor = body;
            let mut take = |field: &mut u64| -> Result<(), ZipError> {
                if *field == u64::from(u32::MAX) {
                    *field = u64_at(extra, cursor).ok_or(ZipError::Zip64Unsupported)?;
                    cursor += 8;
                }
                Ok(())
            };
            take(uncompressed)?;
            take(compressed)?;
            take(local_offset)?;
            return Ok(());
        }
        at = body + size;
    }
    Err(ZipError::Zip64Unsupported)
}

/// CRC-32 as zip specifies it: the reflected IEEE polynomial, pre- and post-inverted.
#[must_use]
pub fn crc32(data: &[u8]) -> u32 {
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    let table = TABLE.get_or_init(|| {
        let mut table = [0u32; 256];
        for (index, slot) in table.iter_mut().enumerate() {
            let mut c = index as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    0xEDB8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
            *slot = c;
        }
        table
    });
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc = table[((crc ^ u32::from(byte)) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a stored-method archive. No compressor needed, so the tests stay hermetic and the
    /// expected bytes are visible here rather than in a committed binary.
    fn stored_archive(members: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut directory = Vec::new();
        for (name, body) in members {
            let offset = out.len() as u32;
            let crc = crc32(body);
            out.extend_from_slice(&LOCAL_SIGNATURE.to_le_bytes());
            out.extend_from_slice(&[20, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // version, flags, method, time, date
            out.extend_from_slice(&crc.to_le_bytes());
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(body);

            directory.extend_from_slice(&CENTRAL_SIGNATURE.to_le_bytes());
            directory.extend_from_slice(&[20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            directory.extend_from_slice(&crc.to_le_bytes());
            directory.extend_from_slice(&(body.len() as u32).to_le_bytes());
            directory.extend_from_slice(&(body.len() as u32).to_le_bytes());
            directory.extend_from_slice(&(name.len() as u16).to_le_bytes());
            directory.extend_from_slice(&0u16.to_le_bytes()); // extra
            directory.extend_from_slice(&0u16.to_le_bytes()); // comment
            directory.extend_from_slice(&0u16.to_le_bytes()); // disk
            directory.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
            directory.extend_from_slice(&0u32.to_le_bytes()); // external attrs
            directory.extend_from_slice(&offset.to_le_bytes());
            directory.extend_from_slice(name.as_bytes());
        }
        let directory_offset = out.len() as u32;
        let directory_size = directory.len() as u32;
        out.extend_from_slice(&directory);
        out.extend_from_slice(&EOCD_SIGNATURE.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&(members.len() as u16).to_le_bytes());
        out.extend_from_slice(&(members.len() as u16).to_le_bytes());
        out.extend_from_slice(&directory_size.to_le_bytes());
        out.extend_from_slice(&directory_offset.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out
    }

    #[test]
    fn crc32_matches_the_published_check_value() {
        // The IEEE check value: CRC-32 of "123456789".
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn reads_members_by_name() {
        let bytes = stored_archive(&[("a.txt", b"alpha"), ("dir/b.txt", b"beta")]);
        let archive = Archive::open(bytes).unwrap();
        assert_eq!(archive.entries().len(), 2);
        assert_eq!(archive.read("a.txt").unwrap(), b"alpha");
        assert_eq!(archive.read_to_string("dir/b.txt").unwrap(), "beta");
    }

    #[test]
    fn contains_answers_without_decompressing() {
        let archive = Archive::open(stored_archive(&[("a.txt", b"alpha")])).unwrap();
        assert!(archive.contains("a.txt"));
        assert!(!archive.contains("xl/sharedStrings.xml"));
    }

    #[test]
    fn an_unknown_member_names_what_was_asked_for() {
        let archive = Archive::open(stored_archive(&[("a.txt", b"alpha")])).unwrap();
        let err = archive.read("nope").unwrap_err();
        assert_eq!(
            err,
            ZipError::NoSuchEntry {
                name: "nope".into()
            }
        );
    }

    #[test]
    fn a_corrupted_member_fails_the_crc_rather_than_returning_wrong_bytes() {
        let mut bytes = stored_archive(&[("a.txt", b"alpha")]);
        let at = bytes
            .windows(5)
            .position(|w| w == b"alpha")
            .expect("member body present");
        bytes[at] = b'X';
        let archive = Archive::open(bytes).unwrap();
        assert!(matches!(
            archive.read("a.txt").unwrap_err(),
            ZipError::CrcMismatch { .. }
        ));
    }

    #[test]
    fn a_file_with_no_directory_record_is_not_a_zip() {
        assert_eq!(
            Archive::open(b"not a zip file at all, just bytes".to_vec()).unwrap_err(),
            ZipError::NotAZip
        );
        assert_eq!(Archive::open(Vec::new()).unwrap_err(), ZipError::NotAZip);
    }

    #[test]
    fn an_unsupported_storage_method_is_named_not_guessed() {
        let mut bytes = stored_archive(&[("a.txt", b"alpha")]);
        // Method lives at offset 8 of the local header and 10 of the central one; only the
        // central copy is consulted when reading.
        let dir = bytes
            .windows(4)
            .rposition(|w| w == CENTRAL_SIGNATURE.to_le_bytes())
            .unwrap();
        bytes[dir + 10] = 12; // bzip2
        let archive = Archive::open(bytes).unwrap();
        assert_eq!(
            archive.read("a.txt").unwrap_err(),
            ZipError::UnsupportedMethod { method: 12 }
        );
    }

    #[test]
    fn an_encrypted_member_is_refused() {
        let mut bytes = stored_archive(&[("a.txt", b"alpha")]);
        let dir = bytes
            .windows(4)
            .rposition(|w| w == CENTRAL_SIGNATURE.to_le_bytes())
            .unwrap();
        bytes[dir + 8] = 1; // general purpose bit 0
        let archive = Archive::open(bytes).unwrap();
        assert!(matches!(
            archive.read("a.txt").unwrap_err(),
            ZipError::Encrypted { .. }
        ));
    }

    #[test]
    fn an_archive_comment_does_not_hide_the_directory_record() {
        let mut bytes = stored_archive(&[("a.txt", b"alpha")]);
        let len = bytes.len();
        bytes[len - 2..].copy_from_slice(&11u16.to_le_bytes());
        bytes.extend_from_slice(b"a comment!!");
        let archive = Archive::open(bytes).unwrap();
        assert_eq!(archive.read("a.txt").unwrap(), b"alpha");
    }
}
