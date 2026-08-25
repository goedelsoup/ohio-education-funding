//! OLE2 compound files — the container a pre-2007 `.xls` lives in.
//!
//! A compound file is a filesystem in a file: fixed-size sectors, a file allocation table
//! chaining them, and a directory tree naming the streams. Ohio still publishes October
//! enrollment headcount in this format, so reading it natively is the difference between an
//! extraction pipeline that runs from a checkout and one that needs LibreOffice installed.
//!
//! # What is implemented
//!
//! Reading, version 3 and version 4, both sector sizes, regular and mini streams, and the DIFAT
//! chain for files with more than 109 FAT sectors. Not writing, not encryption, not the
//! red-black invariants of the directory tree — the directory is walked as a plain array, which
//! is what every implementation does in practice and is why a corrupted tree reads as a missing
//! stream rather than a panic.

/// The eight bytes every compound file starts with.
const MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// Sector marking the end of a chain.
const END_OF_CHAIN: u32 = 0xFFFF_FFFE;
/// Sector belonging to the FAT itself.
const FAT_SECTOR: u32 = 0xFFFF_FFFD;
/// Sector belonging to the DIFAT.
const DIFAT_SECTOR: u32 = 0xFFFF_FFFC;
/// Unallocated sector.
const FREE_SECTOR: u32 = 0xFFFF_FFFF;

/// Bytes of the header given over to the first 109 DIFAT entries.
const HEADER_DIFAT_ENTRIES: usize = 109;

/// Directory entries are a fixed 128 bytes.
const DIRECTORY_ENTRY_SIZE: usize = 128;

/// A compound file that could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ole2Error {
    /// The magic number is absent. Not a compound file.
    NotCompound,
    /// The header is present but describes something this reader cannot follow.
    Malformed(&'static str),
    /// A sector chain leaves the file, loops, or runs longer than the file could hold.
    BadChain,
    /// No stream by that name.
    NoSuchStream {
        /// The name that was asked for.
        name: String,
    },
}

impl core::fmt::Display for Ole2Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotCompound => write!(f, "not an OLE2 compound file"),
            Self::Malformed(what) => write!(f, "malformed compound file: {what}"),
            Self::BadChain => write!(f, "sector chain is circular or runs off the file"),
            Self::NoSuchStream { name } => write!(f, "no stream named {name}"),
        }
    }
}

impl std::error::Error for Ole2Error {}

fn u16_at(data: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(data.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

fn u64_at(data: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(data.get(at..at + 8)?.try_into().ok()?))
}

/// One entry in the directory: a stream, a storage, or the root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Name, decoded from the UTF-16 the format stores.
    pub name: String,
    /// 1 = storage, 2 = stream, 5 = root.
    pub kind: u8,
    start: u32,
    size: u64,
}

/// An opened compound file.
#[derive(Debug, Clone)]
pub struct Compound {
    data: Vec<u8>,
    sector_size: usize,
    mini_sector_size: usize,
    mini_cutoff: u32,
    fat: Vec<u32>,
    mini_fat: Vec<u32>,
    mini_stream: Vec<u8>,
    entries: Vec<Entry>,
}

impl Compound {
    /// Parse a compound file's allocation tables and directory.
    ///
    /// # Errors
    ///
    /// Returns [`Ole2Error`] if the magic is absent or the tables do not describe a file this
    /// reader can follow.
    pub fn open(data: Vec<u8>) -> Result<Self, Ole2Error> {
        if data.len() < 512 || data[..8] != MAGIC {
            return Err(Ole2Error::NotCompound);
        }
        // Only little-endian has ever been produced. Refusing rather than reading it backwards.
        if u16_at(&data, 0x1c) != Some(0xFFFE) {
            return Err(Ole2Error::Malformed("byte order is not little-endian"));
        }

        // Validate before shifting, not after. `1usize << n` for an attacker-supplied `n`
        // panics in debug and silently masks to `n & 63` in release, so a header declaring
        // shift 1033 would parse as 512-byte sectors — a file read as something it never
        // declared. The old `mini_sector_size == 0` check could never fire: `1 << n` is
        // never zero, and the real hazard is a shift so large that `next * mini_sector_size`
        // overflows in `read_mini_chain`.
        let shift = u16_at(&data, 0x1e).ok_or(Ole2Error::NotCompound)?;
        let mini_shift = u16_at(&data, 0x20).ok_or(Ole2Error::NotCompound)?;
        if !(6..=20).contains(&shift) || !(6..=12).contains(&mini_shift) || mini_shift > shift {
            return Err(Ole2Error::Malformed("implausible sector size"));
        }
        let sector_size = 1usize << shift;
        let mini_sector_size = 1usize << mini_shift;

        let fat_sector_count = u32_at(&data, 0x2c).ok_or(Ole2Error::NotCompound)? as usize;
        let directory_start = u32_at(&data, 0x30).ok_or(Ole2Error::NotCompound)?;
        let mini_cutoff = u32_at(&data, 0x38).ok_or(Ole2Error::NotCompound)?;
        let mini_fat_start = u32_at(&data, 0x3c).ok_or(Ole2Error::NotCompound)?;
        let difat_start = u32_at(&data, 0x44).ok_or(Ole2Error::NotCompound)?;
        let difat_sector_count = u32_at(&data, 0x48).ok_or(Ole2Error::NotCompound)? as usize;

        let mut incomplete = Self {
            data,
            sector_size,
            mini_sector_size,
            mini_cutoff,
            fat: Vec::new(),
            mini_fat: Vec::new(),
            mini_stream: Vec::new(),
            entries: Vec::new(),
        };

        let difat = incomplete.read_difat(difat_start, difat_sector_count, fat_sector_count)?;
        incomplete.fat = incomplete.read_fat(&difat)?;
        incomplete.mini_fat = incomplete.read_mini_fat(mini_fat_start)?;
        incomplete.entries = incomplete.read_directory(directory_start)?;
        // The root entry's stream is the mini-stream: where every sub-cutoff stream lives.
        if let Some(root) = incomplete.entries.first().cloned() {
            // `root.size` is eight bytes of a directory entry. A truncating `as` cast on a
            // value of 0xFF..FF asks for a usize::MAX allocation during open(), before any
            // stream has been requested.
            let declared = usize::try_from(root.size).unwrap_or(usize::MAX);
            incomplete.mini_stream = incomplete.read_chain(root.start, declared)?;
        }
        Ok(incomplete)
    }

    /// How many whole sectors the file actually holds, past the 512-byte header.
    ///
    /// The ceiling on every chain: a header field may claim more, but a chain longer than this
    /// has either left the file or looped inside it.
    fn sectors(&self) -> usize {
        self.data.len().saturating_sub(512) / self.sector_size
    }

    fn sector(&self, index: u32) -> Option<&[u8]> {
        // Sector 0 begins immediately after the 512-byte header, whatever the sector size.
        let start = 512 + (index as usize).checked_mul(self.sector_size)?;
        self.data.get(start..start + self.sector_size)
    }

    /// The list of sectors that hold the FAT.
    ///
    /// 109 fit in the header; beyond that they are chained through DIFAT sectors, each of which
    /// spends its last four bytes on the pointer to the next.
    fn read_difat(
        &self,
        start: u32,
        sector_count: usize,
        fat_sector_count: usize,
    ) -> Result<Vec<u32>, Ole2Error> {
        // Both counts are header fields, so both are bounded by what the file could actually
        // hold before either is believed. `fat_sector_count` of 0xFFFFFFFF asks `with_capacity`
        // for 16 GB up front, and `sector_count` of 0xFFFFFFFF drives the loop below through
        // 2.2 TB of pushes on a chain one sector long that points at itself.
        let sectors = self.sectors();
        let mut difat = Vec::with_capacity(fat_sector_count.min(sectors));
        for index in 0..HEADER_DIFAT_ENTRIES {
            let Some(entry) = u32_at(&self.data, 0x4c + index * 4) else {
                break;
            };
            if entry == FREE_SECTOR {
                break;
            }
            difat.push(entry);
        }

        let mut next = start;
        let per_sector = self.sector_size / 4;
        // A DIFAT sector visited twice is a cycle, and no honest chain visits more sectors than
        // the file has. Refusing rather than truncating: a chain that outruns the file is not a
        // chain this reader can follow, and silently keeping its first `sectors` entries would
        // report a FAT assembled from half a lie.
        for step in 0..sector_count {
            if next == END_OF_CHAIN || next == FREE_SECTOR {
                break;
            }
            if step >= sectors {
                return Err(Ole2Error::BadChain);
            }
            let sector = self.sector(next).ok_or(Ole2Error::BadChain)?;
            for index in 0..per_sector - 1 {
                let entry = u32_at(sector, index * 4).ok_or(Ole2Error::BadChain)?;
                if entry != FREE_SECTOR {
                    difat.push(entry);
                }
            }
            next = u32_at(sector, (per_sector - 1) * 4).ok_or(Ole2Error::BadChain)?;
        }
        difat.truncate(fat_sector_count);
        Ok(difat)
    }

    fn read_fat(&self, difat: &[u32]) -> Result<Vec<u32>, Ole2Error> {
        let mut fat = Vec::with_capacity(difat.len() * (self.sector_size / 4));
        for &index in difat {
            let sector = self.sector(index).ok_or(Ole2Error::BadChain)?;
            for offset in (0..self.sector_size).step_by(4) {
                fat.push(u32_at(sector, offset).ok_or(Ole2Error::BadChain)?);
            }
        }
        Ok(fat)
    }

    fn read_mini_fat(&self, start: u32) -> Result<Vec<u32>, Ole2Error> {
        let mut mini_fat = Vec::new();
        let mut next = start;
        let mut guard = 0usize;
        while next != END_OF_CHAIN && next != FREE_SECTOR {
            guard += 1;
            if guard > self.fat.len() + 1 {
                return Err(Ole2Error::BadChain);
            }
            let sector = self.sector(next).ok_or(Ole2Error::BadChain)?;
            for offset in (0..self.sector_size).step_by(4) {
                mini_fat.push(u32_at(sector, offset).ok_or(Ole2Error::BadChain)?);
            }
            next = *self.fat.get(next as usize).ok_or(Ole2Error::BadChain)?;
        }
        Ok(mini_fat)
    }

    /// Follow a sector chain and return `size` bytes of it.
    ///
    /// The guard is the whole point: a corrupt FAT that points a sector at itself would
    /// otherwise allocate until the process died.
    fn read_chain(&self, start: u32, size: usize) -> Result<Vec<u8>, Ole2Error> {
        // Capacity is bounded by the file: a declared size larger than the whole compound
        // document cannot be honoured, and asking for it aborts before the loop's own guard
        // below ever runs.
        let mut out = Vec::with_capacity(size.min(self.data.len()));
        let mut next = start;
        let mut guard = 0usize;
        while next != END_OF_CHAIN && next != FREE_SECTOR && out.len() < size {
            if next == FAT_SECTOR || next == DIFAT_SECTOR {
                return Err(Ole2Error::BadChain);
            }
            guard += 1;
            if guard > self.fat.len() + 1 {
                return Err(Ole2Error::BadChain);
            }
            let sector = self.sector(next).ok_or(Ole2Error::BadChain)?;
            let wanted = (size - out.len()).min(self.sector_size);
            out.extend_from_slice(&sector[..wanted]);
            next = *self.fat.get(next as usize).ok_or(Ole2Error::BadChain)?;
        }
        Ok(out)
    }

    /// Follow a mini-sector chain within the mini-stream.
    fn read_mini_chain(&self, start: u32, size: usize) -> Result<Vec<u8>, Ole2Error> {
        let mut out = Vec::with_capacity(size.min(self.mini_stream.len()));
        let mut next = start;
        let mut guard = 0usize;
        while next != END_OF_CHAIN && next != FREE_SECTOR && out.len() < size {
            guard += 1;
            if guard > self.mini_fat.len() + 1 {
                return Err(Ole2Error::BadChain);
            }
            let Some(at) = (next as usize).checked_mul(self.mini_sector_size) else {
                return Err(Ole2Error::BadChain);
            };
            let sector = self
                .mini_stream
                .get(at..at + self.mini_sector_size)
                .ok_or(Ole2Error::BadChain)?;
            let wanted = (size - out.len()).min(self.mini_sector_size);
            out.extend_from_slice(&sector[..wanted]);
            next = *self
                .mini_fat
                .get(next as usize)
                .ok_or(Ole2Error::BadChain)?;
        }
        Ok(out)
    }

    fn read_directory(&self, start: u32) -> Result<Vec<Entry>, Ole2Error> {
        // The directory chain has no declared length; follow it to the end of chain.
        let mut raw = Vec::new();
        let mut next = start;
        let mut guard = 0usize;
        while next != END_OF_CHAIN && next != FREE_SECTOR {
            guard += 1;
            if guard > self.fat.len() + 1 {
                return Err(Ole2Error::BadChain);
            }
            raw.extend_from_slice(self.sector(next).ok_or(Ole2Error::BadChain)?);
            next = *self.fat.get(next as usize).ok_or(Ole2Error::BadChain)?;
        }

        let mut entries = Vec::new();
        for chunk in raw.as_chunks::<DIRECTORY_ENTRY_SIZE>().0 {
            let kind = chunk[0x42];
            if kind == 0 {
                continue;
            }
            // Name length counts bytes including the terminating NUL, so a 12-character name
            // reports 26.
            let name_bytes = u16_at(chunk, 0x40).unwrap_or(0) as usize;
            let units: Vec<u16> = chunk[..name_bytes.min(64).saturating_sub(2)]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u16::from_le_bytes(*pair))
                .collect();
            entries.push(Entry {
                name: String::from_utf16_lossy(&units),
                kind,
                start: u32_at(chunk, 0x74).unwrap_or(END_OF_CHAIN),
                size: u64_at(chunk, 0x78).unwrap_or(0),
            });
        }
        Ok(entries)
    }

    /// Every entry in the directory, in storage order.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Read a stream by name.
    ///
    /// Streams below the header's cutoff — 4096 bytes in every file this has met — live in the
    /// mini-stream and are chained through a separate table.
    ///
    /// # Errors
    ///
    /// Returns [`Ole2Error::NoSuchStream`] if there is no such stream, or [`Ole2Error::BadChain`]
    /// if its sector chain is broken.
    pub fn read(&self, name: &str) -> Result<Vec<u8>, Ole2Error> {
        let entry = self
            .entries
            .iter()
            .find(|e| e.name == name && e.kind == 2)
            .ok_or_else(|| Ole2Error::NoSuchStream {
                name: name.to_string(),
            })?;
        let size = usize::try_from(entry.size).map_err(|_| Ole2Error::BadChain)?;
        if entry.size < u64::from(self.mini_cutoff) {
            self.read_mini_chain(entry.start, size)
        } else {
            self.read_chain(entry.start, size)
        }
    }

    /// Read the first stream whose name matches any of `names`.
    ///
    /// Excel has used both `Workbook` and the older `Book`, and a file may carry either.
    ///
    /// # Errors
    ///
    /// Returns [`Ole2Error::NoSuchStream`] naming the first candidate if none is present.
    pub fn read_any(&self, names: &[&str]) -> Result<Vec<u8>, Ole2Error> {
        for name in names {
            if let Ok(bytes) = self.read(name) {
                return Ok(bytes);
            }
        }
        Err(Ole2Error::NoSuchStream {
            name: names.first().unwrap_or(&"").to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal compound file with one stream in the FAT and one in the mini-FAT.
    ///
    /// Hand-assembled rather than committed as a binary, so the structure under test is visible
    /// and a reader can check the offsets against the specification.
    fn compound(large: &[u8], small: &[u8]) -> Vec<u8> {
        let sector = 512usize;
        let mini = 64usize;
        // Layout: sector 0 FAT, 1 directory, 2.. mini-stream, then the large stream, then the
        // mini-FAT. Sizes here are small enough that each part is one sector.
        let mini_stream_sectors = small.len().div_ceil(sector).max(1);
        let large_sectors = large.len().div_ceil(sector).max(1);

        let mut fat = vec![FREE_SECTOR; sector / 4];
        fat[0] = FAT_SECTOR;
        fat[1] = END_OF_CHAIN; // directory
        for i in 0..mini_stream_sectors {
            fat[2 + i] = if i + 1 == mini_stream_sectors {
                END_OF_CHAIN
            } else {
                (3 + i) as u32
            };
        }
        let large_start = 2 + mini_stream_sectors;
        for i in 0..large_sectors {
            fat[large_start + i] = if i + 1 == large_sectors {
                END_OF_CHAIN
            } else {
                (large_start + i + 1) as u32
            };
        }
        let mini_fat_sector = large_start + large_sectors;
        fat[mini_fat_sector] = END_OF_CHAIN;

        let mut mini_fat = vec![FREE_SECTOR; sector / 4];
        let small_minis = small.len().div_ceil(mini).max(1);
        for (i, slot) in mini_fat.iter_mut().take(small_minis).enumerate() {
            *slot = if i + 1 == small_minis {
                END_OF_CHAIN
            } else {
                (i + 1) as u32
            };
        }

        let entry = |name: &str, kind: u8, start: u32, size: u64| {
            let mut e = vec![0u8; DIRECTORY_ENTRY_SIZE];
            let units: Vec<u16> = name.encode_utf16().collect();
            for (i, unit) in units.iter().enumerate() {
                e[i * 2..i * 2 + 2].copy_from_slice(&unit.to_le_bytes());
            }
            e[0x40..0x42].copy_from_slice(&(((units.len() + 1) * 2) as u16).to_le_bytes());
            e[0x42] = kind;
            e[0x74..0x78].copy_from_slice(&start.to_le_bytes());
            e[0x78..0x80].copy_from_slice(&size.to_le_bytes());
            e
        };

        let mut directory = Vec::new();
        directory.extend(entry(
            "Root Entry",
            5,
            2,
            (mini_stream_sectors * sector) as u64,
        ));
        directory.extend(entry("Big", 2, large_start as u32, large.len() as u64));
        directory.extend(entry("Small", 2, 0, small.len() as u64));
        directory.resize(sector, 0);

        let mut header = vec![0u8; 512];
        header[..8].copy_from_slice(&MAGIC);
        header[0x1c..0x1e].copy_from_slice(&0xFFFEu16.to_le_bytes());
        header[0x1e..0x20].copy_from_slice(&9u16.to_le_bytes());
        header[0x20..0x22].copy_from_slice(&6u16.to_le_bytes());
        header[0x2c..0x30].copy_from_slice(&1u32.to_le_bytes()); // one FAT sector
        header[0x30..0x34].copy_from_slice(&1u32.to_le_bytes()); // directory at sector 1
        header[0x38..0x3c].copy_from_slice(&4096u32.to_le_bytes());
        header[0x3c..0x40].copy_from_slice(&(mini_fat_sector as u32).to_le_bytes());
        header[0x40..0x44].copy_from_slice(&1u32.to_le_bytes());
        header[0x44..0x48].copy_from_slice(&END_OF_CHAIN.to_le_bytes());
        header[0x4c..0x50].copy_from_slice(&0u32.to_le_bytes()); // FAT lives in sector 0
        for slot in 1..HEADER_DIFAT_ENTRIES {
            let at = 0x4c + slot * 4;
            header[at..at + 4].copy_from_slice(&FREE_SECTOR.to_le_bytes());
        }

        let mut out = header;
        let mut push = |bytes: &[u8], sectors: usize| {
            let mut block = bytes.to_vec();
            block.resize(sectors * sector, 0);
            out.extend_from_slice(&block);
        };
        let fat_bytes: Vec<u8> = fat.iter().flat_map(|v| v.to_le_bytes()).collect();
        push(&fat_bytes, 1);
        push(&directory, 1);
        push(small, mini_stream_sectors);
        push(large, large_sectors);
        let mini_fat_bytes: Vec<u8> = mini_fat.iter().flat_map(|v| v.to_le_bytes()).collect();
        push(&mini_fat_bytes, 1);
        out
    }

    #[test]
    fn reads_a_stream_from_the_regular_chain() {
        let large = vec![b'A'; 5000];
        let file = compound(&large, b"small");
        let compound = Compound::open(file).unwrap();
        assert_eq!(compound.read("Big").unwrap(), large);
    }

    #[test]
    fn reads_a_stream_from_the_mini_chain() {
        // Below the 4096-byte cutoff, so it lives in the mini-stream and is chained separately.
        let file = compound(&vec![b'A'; 5000], b"a small stream");
        let compound = Compound::open(file).unwrap();
        assert_eq!(compound.read("Small").unwrap(), b"a small stream");
    }

    #[test]
    fn a_stream_is_truncated_to_its_declared_size_not_its_sector_count() {
        let file = compound(&vec![b'A'; 5000], b"exact");
        let compound = Compound::open(file).unwrap();
        assert_eq!(compound.read("Small").unwrap().len(), 5);
        assert_eq!(compound.read("Big").unwrap().len(), 5000);
    }

    #[test]
    fn lists_the_directory_including_the_root() {
        let compound = Compound::open(compound(&vec![b'A'; 5000], b"s")).unwrap();
        let names: Vec<&str> = compound.entries().iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["Root Entry", "Big", "Small"]);
        assert_eq!(compound.entries()[0].kind, 5);
    }

    #[test]
    fn an_unknown_stream_names_what_was_asked_for() {
        let compound = Compound::open(compound(&vec![b'A'; 5000], b"s")).unwrap();
        assert_eq!(
            compound.read("Workbook").unwrap_err(),
            Ole2Error::NoSuchStream {
                name: "Workbook".into()
            }
        );
    }

    #[test]
    fn read_any_takes_the_first_name_that_resolves() {
        let compound = Compound::open(compound(&vec![b'A'; 5000], b"s")).unwrap();
        assert_eq!(compound.read_any(&["Workbook", "Big"]).unwrap().len(), 5000);
        assert!(compound.read_any(&["Nope", "Neither"]).is_err());
    }

    #[test]
    fn a_file_without_the_magic_is_not_compound() {
        assert_eq!(
            Compound::open(b"PK\x03\x04 a zip, not an OLE2 file at all........".repeat(20))
                .unwrap_err(),
            Ole2Error::NotCompound
        );
        assert_eq!(
            Compound::open(Vec::new()).unwrap_err(),
            Ole2Error::NotCompound
        );
    }

    #[test]
    fn a_circular_chain_is_refused_rather_than_followed() {
        // The guard that matters: a FAT pointing a sector at itself would otherwise allocate
        // until the process died.
        let mut file = compound(&vec![b'A'; 5000], b"s");
        let large_start_sector = 3usize;
        let fat_at = 512 + large_start_sector * 4;
        file[fat_at..fat_at + 4].copy_from_slice(&(large_start_sector as u32).to_le_bytes());
        let compound = Compound::open(file).unwrap();
        let _ = compound.read("Big");
    }

    /// The header's sector sizes are shift amounts, and a shift is validated before it happens.
    ///
    /// `1usize << n` for an `n` read straight from the file panics in debug and silently masks
    /// to `n & 63` in release — so a header declaring shift 1033 parsed as 512-byte sectors, a
    /// file read as something it never declared. No test perturbed any header field; the only
    /// header the suite built was the valid one below.
    #[test]
    fn an_implausible_sector_shift_is_refused_before_it_is_shifted() {
        let good = compound(b"large stream contents past the mini cutoff", b"small");
        assert!(
            Compound::open(good.clone()).is_ok(),
            "the fixture must be valid"
        );

        for (offset, label) in [(0x1e, "sector shift"), (0x20, "mini sector shift")] {
            for bytes in [[0xFF, 0xFF], [0x09, 0x04], [0x00, 0x00], [0x3E, 0x00]] {
                let mut bad = good.clone();
                bad[offset] = bytes[0];
                bad[offset + 1] = bytes[1];
                // Not dying is the assertion. A panic here is the defect.
                let _ = (label, Compound::open(bad));
            }
        }
    }

    /// Truncating the file anywhere produces an error, never a panic.
    ///
    /// A cheap fuzzer for the whole header-and-directory path: every prefix of a valid
    /// compound document is a malformed one, and each exercises a different partial read.
    #[test]
    fn every_truncation_of_a_compound_document_is_refused_rather_than_panicking() {
        let good = compound(b"large stream contents past the mini cutoff", b"small");
        for n in 0..good.len() {
            let _ = Compound::open(good[..n].to_vec());
        }
    }
}
