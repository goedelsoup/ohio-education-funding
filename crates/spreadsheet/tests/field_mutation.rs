//! Every length, count and offset field in a valid fixture, mutated, one at a time.
//!
//! # Why this file exists and the unit tests do not suffice
//!
//! Issue #67 declared the "malformed length/count/offset field" class closed, and three
//! instances survived it. They survived because the pass was verified against *named
//! instances* — the reference that produced a 7.7 GB row, the root-entry size that asked for
//! `usize::MAX` — each fixed with a guard and a test aimed at that guard. Nothing swept the
//! class, so the next field in it was as unprotected as the first, and one of the three sat
//! two bytes from a field that had been fixed.
//!
//! The sweep below does not name fields. It mutates **every four-byte-aligned window** of a
//! valid document to each of a handful of hostile values and puts the result through the
//! public API. That covers every declared length, count and offset in the format without a
//! table anyone has to remember to extend, and it keeps covering them when a field is added.
//!
//! # What is asserted
//!
//! Two things, and the second is the one the named-instance tests could not state:
//!
//! 1. **No panic.** A malformed document is an `Err`, not an abort. The test harness fails the
//!    test on a panic, so every mutation reaching the end of the loop is the assertion.
//! 2. **No allocation the input did not pay for.** A count field is an *instruction to
//!    allocate*, and the failure mode of the class is not a wrong answer but a dead process:
//!    1 KB of input drove ~2.2 TB of allocation through `read_difat`'s header-declared chain
//!    length. A panic-only sweep passes straight through that on a machine with lazy commit and
//!    dies on the machine that CI runs. So this binary installs an allocator that records the
//!    largest single request, and the sweep asserts a few kilobytes of input never asks for
//!    megabytes.
//!
//! The gauge measures the largest *single* request rather than a total, because that is the
//! quantity a count field controls directly and the one that is stable under a test harness
//! running other work on other threads.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};

use spreadsheet::ole2::Compound;
use spreadsheet::xlsx::Workbook;
use spreadsheet::zip::crc32;

/// The largest single allocation request seen so far, in bytes.
static LARGEST: AtomicUsize = AtomicUsize::new(0);

/// Held for the length of a measurement, so two measuring tests cannot read each other's gauge.
static MEASURING: Mutex<()> = Mutex::new(());

/// An allocator that does nothing but remember the biggest thing it was asked for.
///
/// `spreadsheet` itself is `#![forbid(unsafe_code)]` and stays that way; a global allocator
/// needs `unsafe impl`, so it lives here in the test binary, which is a separate crate and
/// still covers every allocation the library makes while this binary runs it.
struct Watched;

unsafe impl GlobalAlloc for Watched {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LARGEST.fetch_max(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    // Vec growth reallocates, so a table that doubles its way to a gigabyte would otherwise be
    // invisible here: only the first, small `alloc` would be recorded.
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        LARGEST.fetch_max(new_size, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Watched = Watched;

/// A few kilobytes of fixture may not ask for megabytes, whatever its fields declare.
///
/// Generous by three orders of magnitude against the real failures — the survivors asked for
/// gigabytes and terabytes — so that ordinary parse allocations never make this flaky.
const ALLOCATION_CEILING: usize = 8 * 1024 * 1024;

/// Run `body` with the allocation gauge zeroed, then assert what it cost.
fn measured(body: impl FnOnce()) {
    let guard: MutexGuard<'_, ()> = MEASURING.lock().unwrap_or_else(|e| e.into_inner());
    LARGEST.store(0, Ordering::SeqCst);
    body();
    let largest = LARGEST.load(Ordering::SeqCst);
    drop(guard);
    assert!(
        largest <= ALLOCATION_CEILING,
        "a kilobyte-scale fixture drove a single {largest}-byte allocation; a declared count \
         was believed without being bounded by what the file could hold"
    );
}

/// The hostile values. Each is a plausible corruption of a length, a count or an offset.
const POISON: [u32; 6] = [
    0x0000_0000,
    0x0000_0001,
    0x7FFF_FFFF,
    0xFFFF_FFFC, // DIFAT sector marker
    0xFFFF_FFFE, // end of chain
    0xFFFF_FFFF, // free sector, and -1 as a length
];

/// Every four-byte-aligned window of `good`, each set to each poison value in turn.
fn mutations(good: &[u8]) -> impl Iterator<Item = Vec<u8>> + '_ {
    (0..good.len().saturating_sub(3))
        .step_by(4)
        .flat_map(move |at| {
            POISON.iter().map(move |value| {
                let mut bad = good.to_vec();
                bad[at..at + 4].copy_from_slice(&value.to_le_bytes());
                bad
            })
        })
}

// --- OLE2 ------------------------------------------------------------------------------------

/// A minimal but valid compound file: one stream in the FAT, one in the mini-FAT.
///
/// Assembled here rather than committed as a binary so that what the sweep is mutating is
/// visible, and small on purpose — the point of the exercise is that a kilobyte of input cannot
/// buy a gigabyte of work.
fn compound_file() -> Vec<u8> {
    const END_OF_CHAIN: u32 = 0xFFFF_FFFE;
    const FAT_SECTOR: u32 = 0xFFFF_FFFD;
    const FREE_SECTOR: u32 = 0xFFFF_FFFF;
    let sector = 512usize;
    let large = vec![b'A'; 600];
    let small = b"a small stream";

    // Sector 0 the FAT, 1 the directory, 2 the mini-stream, 3 the large stream, 4 the mini-FAT.
    let mut fat = vec![FREE_SECTOR; sector / 4];
    fat[0] = FAT_SECTOR;
    fat[1] = END_OF_CHAIN;
    fat[2] = END_OF_CHAIN;
    fat[3] = 4;
    fat[4] = END_OF_CHAIN;
    fat[5] = END_OF_CHAIN;

    let mut mini_fat = vec![FREE_SECTOR; sector / 4];
    mini_fat[0] = END_OF_CHAIN;

    let entry = |name: &str, kind: u8, start: u32, size: u64| {
        let mut e = vec![0u8; 128];
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
    directory.extend(entry("Root Entry", 5, 2, sector as u64));
    directory.extend(entry("Big", 2, 3, large.len() as u64));
    directory.extend(entry("Small", 2, 0, small.len() as u64));
    directory.resize(sector, 0);

    let mut header = vec![0u8; 512];
    header[..8].copy_from_slice(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
    header[0x1c..0x1e].copy_from_slice(&0xFFFEu16.to_le_bytes());
    header[0x1e..0x20].copy_from_slice(&9u16.to_le_bytes());
    header[0x20..0x22].copy_from_slice(&6u16.to_le_bytes());
    header[0x2c..0x30].copy_from_slice(&1u32.to_le_bytes()); // one FAT sector
    header[0x30..0x34].copy_from_slice(&1u32.to_le_bytes()); // directory at sector 1
    header[0x38..0x3c].copy_from_slice(&4096u32.to_le_bytes()); // mini cutoff
    header[0x3c..0x40].copy_from_slice(&4u32.to_le_bytes()); // mini-FAT at sector 4
    header[0x40..0x44].copy_from_slice(&1u32.to_le_bytes()); // one mini-FAT sector
    header[0x44..0x48].copy_from_slice(&0xFFFF_FFFEu32.to_le_bytes()); // no DIFAT sectors
    header[0x4c..0x50].copy_from_slice(&0u32.to_le_bytes()); // the FAT lives in sector 0
    for slot in 1..109 {
        let at = 0x4c + slot * 4;
        header[at..at + 4].copy_from_slice(&FREE_SECTOR.to_le_bytes());
    }

    let mut out = header;
    let mut push = |bytes: &[u8]| {
        let mut block = bytes.to_vec();
        block.resize(sector, 0);
        out.extend_from_slice(&block);
    };
    push(
        &fat.iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<u8>>(),
    );
    push(&directory);
    push(small);
    push(&large);
    push(
        &mini_fat
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<u8>>(),
    );
    out
}

/// Open a compound document and read everything it says it holds.
///
/// Reading the streams matters: a size field is believed at `read`, not at `open`, so a sweep
/// that only opened would miss half the class.
fn exercise_compound(bytes: Vec<u8>) {
    let Ok(compound) = Compound::open(bytes) else {
        return;
    };
    let names: Vec<String> = compound.entries().iter().map(|e| e.name.clone()).collect();
    for name in names {
        let _ = compound.read(&name);
    }
}

#[test]
fn no_field_of_a_compound_document_buys_more_than_the_file_paid_for() {
    let good = compound_file();
    assert!(
        Compound::open(good.clone()).is_ok(),
        "the fixture must be valid, or the sweep proves nothing"
    );
    measured(|| {
        for bad in mutations(&good) {
            exercise_compound(bad);
        }
    });
}

/// The named survivor, kept beside the sweep that would have found it.
///
/// `read_difat` walked a chain whose length was the header's own `difat_sector_count`, with no
/// bound and no cycle guard: point the chain at a sector that points back at itself and declare
/// 0xFFFFFFFF sectors, and 1 KB of input asks for roughly 2.2 TB.
#[test]
fn a_difat_chain_that_loops_is_refused_rather_than_walked_to_its_declared_length() {
    let mut file = compound_file();
    file[0x44..0x48].copy_from_slice(&0u32.to_le_bytes()); // DIFAT starts at sector 0
    file[0x48..0x4c].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // ...for 4.29 billion sectors
    let last_word = 512 + 512 - 4; // sector 0's next-pointer, aimed back at sector 0
    file[last_word..last_word + 4].copy_from_slice(&0u32.to_le_bytes());
    measured(|| {
        assert!(
            Compound::open(file).is_err(),
            "a DIFAT chain that visits a sector twice is not a chain"
        );
    });
}

// --- XLSX ------------------------------------------------------------------------------------

const RELS: &str = r#"<?xml version="1.0"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Target="worksheets/sheet1.xml"/>
</Relationships>"#;

const BOOK: &str = r#"<?xml version="1.0"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
 xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets><sheet name="Data" sheetId="1" r:id="rId1"/></sheets></workbook>"#;

const SHARED: &str = r#"<?xml version="1.0"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<si><t>IRN</t></si><si><t>District</t></si></sst>"#;

const SHEET: &str = r#"<worksheet><sheetData>
<row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c></row>
<row r="2"><c r="A2"><v>043786</v></c><c r="B2" t="inlineStr"><is><t>Cleveland</t></is></c></row>
</sheetData></worksheet>"#;

/// A stored-method (uncompressed) zip, so that mutating a byte changes a *field* rather than
/// invalidating a DEFLATE stream before any field is read.
fn stored_archive(members: &[(&str, &str)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut directory = Vec::new();
    for (name, body) in members {
        let offset = out.len() as u32;
        let body = body.as_bytes();
        let crc = crc32(body);
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&[20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(body);

        directory.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        directory.extend_from_slice(&[20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        directory.extend_from_slice(&crc.to_le_bytes());
        directory.extend_from_slice(&(body.len() as u32).to_le_bytes());
        directory.extend_from_slice(&(body.len() as u32).to_le_bytes());
        directory.extend_from_slice(&(name.len() as u16).to_le_bytes());
        directory.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        directory.extend_from_slice(&offset.to_le_bytes());
        directory.extend_from_slice(name.as_bytes());
    }
    let directory_offset = out.len() as u32;
    let directory_size = directory.len() as u32;
    out.extend_from_slice(&directory);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(members.len() as u16).to_le_bytes());
    out.extend_from_slice(&(members.len() as u16).to_le_bytes());
    out.extend_from_slice(&directory_size.to_le_bytes());
    out.extend_from_slice(&directory_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

fn workbook_bytes(sheet: &str) -> Vec<u8> {
    stored_archive(&[
        ("xl/_rels/workbook.xml.rels", RELS),
        ("xl/workbook.xml", BOOK),
        ("xl/sharedStrings.xml", SHARED),
        ("xl/worksheets/sheet1.xml", sheet),
    ])
}

fn exercise_workbook(bytes: Vec<u8>) {
    let Ok(book) = Workbook::open(bytes) else {
        return;
    };
    let names: Vec<String> = book.sheet_names().into_iter().map(String::from).collect();
    for name in names {
        let _ = book.rows(&name);
    }
}

#[test]
fn no_field_of_a_workbook_archive_buys_more_than_the_file_paid_for() {
    let good = workbook_bytes(SHEET);
    assert!(
        Workbook::open(good.clone()).is_ok(),
        "the fixture must be valid, or the sweep proves nothing"
    );
    measured(|| {
        for bad in mutations(&good) {
            exercise_workbook(bad);
        }
    });
}

/// The other named survivor, through the public API rather than against the private function.
///
/// A cell with no `r` takes the position after the previous one. Two of them after `XFD` — the
/// last column the format defines — put the implicit counter past the end of a row whose width
/// was clamped to `XFD`, and the write indexed one past the vector. The clamp had a test; the
/// test never reached it, because the reference it corrupted was refused earlier and the row
/// fell back to implicit positions starting at zero.
#[test]
fn a_row_shifted_past_the_last_column_is_dropped_rather_than_written_off_the_end() {
    let sheet = r#"<worksheet><sheetData>
<row r="1"><c r="XFC1"><v>1</v></c><c r="XFD1"><v>2</v></c><c><v>3</v></c><c><v>4</v></c></row>
</sheetData></worksheet>"#;
    let book = Workbook::open(workbook_bytes(sheet)).unwrap();
    let rows = book.rows("Data").unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].len(), 16_384, "a row is never wider than XFD");
    assert_eq!(rows[0][16_382], "1");
    assert_eq!(rows[0][16_383], "2");
}
