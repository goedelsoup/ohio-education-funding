//! DEFLATE decompression, RFC 1951.
//!
//! An XLSX is a zip archive, and zip members are DEFLATE streams. Reading the department's
//! workbooks therefore requires a decompressor, and this workspace has no dependencies — so
//! here is one. It is a few hundred lines because DEFLATE is a small format: three block
//! types, two Huffman tables, one back-reference rule.
//!
//! The implementation is the canonical-code approach from `puff.c`, with a 9-bit lookup table
//! in front of it. The table matters more than it looks: a 5 MB workbook expands to roughly
//! 40 MB, and bit-at-a-time decoding of that is slow enough in an unoptimised test build to
//! make the end-to-end extraction test unpleasant to run.
//!
//! # What this does not do
//!
//! No zlib or gzip framing — those are wrappers around the same stream and zip members carry
//! neither. No compression. Decompression is all a reader needs.

use std::sync::OnceLock;

/// Longest Huffman code DEFLATE permits.
const MAX_BITS: usize = 15;

/// Width of the direct-lookup table. Codes this long or shorter decode in one indexed read;
/// longer ones fall back to the bit-at-a-time walk. Nine bits covers every code in a fixed
/// block and the large majority in a dynamic one.
const FAST_BITS: u32 = 9;

/// Ceiling on decompressed output, as a defence against a decompression bomb.
///
/// The largest sheet in the department's FY2027 calculator expands to well under 100 MB, so
/// this is generous by an order of magnitude while still bounding a malicious archive.
pub const DEFAULT_LIMIT: usize = 512 * 1024 * 1024;

/// A DEFLATE stream that could not be decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InflateError {
    /// The stream ended in the middle of a symbol, a block header, or a stored run.
    UnexpectedEnd,
    /// Block type 3, which RFC 1951 reserves and no encoder emits.
    ReservedBlockType,
    /// A stored block's length and its one's complement disagree — the stream is corrupt.
    StoredLengthMismatch,
    /// A code-length table assigns more codes than the tree has room for.
    OverSubscribedCode,
    /// A decoded symbol falls outside the range its table defines.
    InvalidSymbol,
    /// A back-reference points further back than the output produced so far.
    DistanceTooFar {
        /// How far back the reference pointed.
        distance: usize,
        /// How many bytes had been produced.
        produced: usize,
    },
    /// Output exceeded the caller's limit. See [`DEFAULT_LIMIT`].
    OutputTooLarge {
        /// The limit that was exceeded.
        limit: usize,
    },
}

impl core::fmt::Display for InflateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnexpectedEnd => write!(f, "compressed stream ended mid-symbol"),
            Self::ReservedBlockType => write!(f, "reserved block type 3"),
            Self::StoredLengthMismatch => write!(f, "stored block length check failed"),
            Self::OverSubscribedCode => write!(f, "over-subscribed Huffman code lengths"),
            Self::InvalidSymbol => write!(f, "symbol outside the table's range"),
            Self::DistanceTooFar { distance, produced } => write!(
                f,
                "back-reference {distance} bytes into {produced} bytes of output"
            ),
            Self::OutputTooLarge { limit } => {
                write!(f, "decompressed output exceeded {limit} bytes")
            }
        }
    }
}

impl std::error::Error for InflateError {}

/// Decompress a raw DEFLATE stream, up to [`DEFAULT_LIMIT`] bytes of output.
///
/// # Errors
///
/// Returns [`InflateError`] if the stream is truncated, malformed, or expands past the limit.
pub fn inflate(data: &[u8]) -> Result<Vec<u8>, InflateError> {
    inflate_with_limit(data, DEFAULT_LIMIT)
}

/// Decompress a raw DEFLATE stream with an explicit output ceiling.
///
/// `expected` is a size hint used only to pre-allocate; a wrong hint costs a reallocation, not
/// correctness.
///
/// # Errors
///
/// Returns [`InflateError`] if the stream is truncated, malformed, or expands past `limit`.
pub fn inflate_with_limit(data: &[u8], limit: usize) -> Result<Vec<u8>, InflateError> {
    let mut reader = BitReader::new(data);
    let mut out = Vec::new();
    loop {
        let last = reader.bits(1)?;
        match reader.bits(2)? {
            0 => stored_block(&mut reader, &mut out, limit)?,
            1 => {
                let (lit, dist) = fixed_tables();
                huffman_block(&mut reader, &mut out, lit, dist, limit)?;
            }
            2 => {
                let (lit, dist) = dynamic_tables(&mut reader)?;
                huffman_block(&mut reader, &mut out, &lit, &dist, limit)?;
            }
            _ => return Err(InflateError::ReservedBlockType),
        }
        if last == 1 {
            return Ok(out);
        }
    }
}

/// Decompress with a pre-allocated output buffer sized to `expected`.
///
/// A zip member declares its uncompressed size, so the caller usually knows the answer before
/// starting.
///
/// # Errors
///
/// Returns [`InflateError`] on a malformed stream, as [`inflate_with_limit`] does.
pub fn inflate_sized(data: &[u8], expected: usize, limit: usize) -> Result<Vec<u8>, InflateError> {
    if expected > limit {
        return Err(InflateError::OutputTooLarge { limit });
    }
    let mut reader = BitReader::new(data);
    let mut out = Vec::with_capacity(expected);
    loop {
        let last = reader.bits(1)?;
        match reader.bits(2)? {
            0 => stored_block(&mut reader, &mut out, limit)?,
            1 => {
                let (lit, dist) = fixed_tables();
                huffman_block(&mut reader, &mut out, lit, dist, limit)?;
            }
            2 => {
                let (lit, dist) = dynamic_tables(&mut reader)?;
                huffman_block(&mut reader, &mut out, &lit, &dist, limit)?;
            }
            _ => return Err(InflateError::ReservedBlockType),
        }
        if last == 1 {
            return Ok(out);
        }
    }
}

// --- bit reading ---------------------------------------------------------------------------

/// DEFLATE packs bits least-significant-first within each byte, and Huffman codes
/// most-significant-first within themselves. That mismatch is why codes are reversed when the
/// lookup table is built rather than when it is read.
struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    buf: u64,
    count: u32,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            buf: 0,
            count: 0,
        }
    }

    #[inline]
    fn fill(&mut self) {
        while self.count <= 56 && self.pos < self.data.len() {
            self.buf |= u64::from(self.data[self.pos]) << self.count;
            self.pos += 1;
            self.count += 8;
        }
    }

    #[inline]
    fn bits(&mut self, n: u32) -> Result<u32, InflateError> {
        if n == 0 {
            return Ok(0);
        }
        if self.count < n {
            self.fill();
            if self.count < n {
                return Err(InflateError::UnexpectedEnd);
            }
        }
        let value = (self.buf & ((1u64 << n) - 1)) as u32;
        self.buf >>= n;
        self.count -= n;
        Ok(value)
    }

    /// Look at the next `n` bits without consuming them, zero-padding past end of input.
    ///
    /// Padding is safe because a table hit on padded bits is checked against [`Self::count`]
    /// before it is accepted; a near-the-end lookup falls through to the slow path, which
    /// reports the truncation properly.
    #[inline]
    fn peek(&mut self, n: u32) -> u32 {
        if self.count < n {
            self.fill();
        }
        (self.buf & ((1u64 << n) - 1)) as u32
    }

    #[inline]
    fn consume(&mut self, n: u32) {
        self.buf >>= n;
        self.count -= n;
    }

    /// Drop back to a byte boundary, returning buffered whole bytes to the input.
    ///
    /// A stored block begins on the next byte boundary; the bits left over in the current
    /// partial byte are discarded, which is what this does.
    fn align_to_byte(&mut self) {
        self.pos -= (self.count / 8) as usize;
        self.buf = 0;
        self.count = 0;
    }
}

// --- Huffman -------------------------------------------------------------------------------

struct Huffman {
    /// How many codes have each length, indexed by length.
    counts: [u16; MAX_BITS + 1],
    /// Symbols in canonical order: by code length, then by symbol.
    symbols: Vec<u16>,
    /// `(length << 12) | symbol` indexed by the next [`FAST_BITS`] bits; zero means "walk".
    fast: Vec<u16>,
}

impl Huffman {
    /// Build a decoding table from a per-symbol code-length list.
    ///
    /// An *incomplete* table — one that leaves codes unassigned — is accepted, because a block
    /// with a single distance code produces one and it is legal. An *over-subscribed* table is
    /// rejected, because no assignment of codes satisfies it.
    fn new(lengths: &[u8]) -> Result<Self, InflateError> {
        let mut counts = [0u16; MAX_BITS + 1];
        for &len in lengths {
            counts[len as usize] += 1;
        }
        counts[0] = 0;

        // Kraft's inequality, walked one code length at a time: doubling the space available
        // per level and spending what this level's codes consume. Going negative means the
        // lengths ask for more codes than a binary tree of that depth can hold.
        let mut left: i32 = 1;
        for &count in &counts[1..=MAX_BITS] {
            left <<= 1;
            left -= i32::from(count);
            if left < 0 {
                return Err(InflateError::OverSubscribedCode);
            }
        }

        let mut offsets = [0usize; MAX_BITS + 2];
        for len in 1..=MAX_BITS {
            offsets[len + 1] = offsets[len] + counts[len] as usize;
        }
        let mut symbols = vec![0u16; offsets[MAX_BITS + 1]];
        let mut cursor = offsets;
        for (symbol, &len) in lengths.iter().enumerate() {
            if len != 0 {
                symbols[cursor[len as usize]] = symbol as u16;
                cursor[len as usize] += 1;
            }
        }

        // Canonical code assignment: the first code of length n is (first of n-1 + count of
        // n-1) << 1. Codes are MSB-first, so they are reversed to index a LSB-first peek.
        let mut fast = vec![0u16; 1 << FAST_BITS];
        let mut code: u32 = 0;
        for len in 1..=MAX_BITS {
            code = (code + u32::from(counts[len - 1])) << 1;
            if len as u32 > FAST_BITS {
                continue;
            }
            let mut next = code;
            for &symbol in &symbols[offsets[len]..offsets[len] + counts[len] as usize] {
                let entry = ((len as u16) << 12) | symbol;
                let mut index = reverse_bits(next, len as u32) as usize;
                while index < (1 << FAST_BITS) {
                    fast[index] = entry;
                    index += 1 << len;
                }
                next += 1;
            }
        }

        Ok(Self {
            counts,
            symbols,
            fast,
        })
    }

    #[inline]
    fn decode(&self, reader: &mut BitReader<'_>) -> Result<u16, InflateError> {
        let entry = self.fast[reader.peek(FAST_BITS) as usize];
        if entry != 0 {
            let len = u32::from(entry >> 12);
            if reader.count >= len {
                reader.consume(len);
                return Ok(entry & 0x0FFF);
            }
        }
        self.decode_walking(reader)
    }

    /// Walk the canonical code one bit at a time. Used for codes longer than [`FAST_BITS`] and
    /// at the very end of the stream, where the peek would run off the input.
    fn decode_walking(&self, reader: &mut BitReader<'_>) -> Result<u16, InflateError> {
        let mut code: i32 = 0;
        let mut first: i32 = 0;
        let mut index: i32 = 0;
        for len in 1..=MAX_BITS {
            code |= reader.bits(1)? as i32;
            let count = i32::from(self.counts[len]);
            if code - count < first {
                return Ok(self.symbols[(index + (code - first)) as usize]);
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }
        Err(InflateError::InvalidSymbol)
    }
}

fn reverse_bits(mut value: u32, n: u32) -> u32 {
    let mut out = 0;
    for _ in 0..n {
        out = (out << 1) | (value & 1);
        value >>= 1;
    }
    out
}

/// The block-1 tables, which RFC 1951 fixes: literals 0-143 are 8 bits, 144-255 are 9,
/// end-of-block and short lengths are 7, the rest 8; every distance code is 5.
fn fixed_tables() -> (&'static Huffman, &'static Huffman) {
    static TABLES: OnceLock<(Huffman, Huffman)> = OnceLock::new();
    let (lit, dist) = TABLES.get_or_init(|| {
        let mut lengths = [8u8; 288];
        lengths[144..256].fill(9);
        lengths[256..280].fill(7);
        let lit = Huffman::new(&lengths).expect("fixed literal lengths are well-formed");
        let dist = Huffman::new(&[5u8; 30]).expect("fixed distance lengths are well-formed");
        (lit, dist)
    });
    (lit, dist)
}

/// Code-length codes are themselves Huffman-coded, and their lengths arrive in this order —
/// chosen so that the rarely-used lengths sit at the end and can be omitted.
const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

fn dynamic_tables(reader: &mut BitReader<'_>) -> Result<(Huffman, Huffman), InflateError> {
    let hlit = reader.bits(5)? as usize + 257;
    let hdist = reader.bits(5)? as usize + 1;
    let hclen = reader.bits(4)? as usize + 4;

    let mut code_lengths = [0u8; 19];
    for &slot in CODE_LENGTH_ORDER.iter().take(hclen) {
        code_lengths[slot] = reader.bits(3)? as u8;
    }
    let code_length_table = Huffman::new(&code_lengths)?;

    let mut lengths = vec![0u8; hlit + hdist];
    let mut i = 0;
    while i < lengths.len() {
        let symbol = code_length_table.decode(reader)?;
        let (value, repeat) = match symbol {
            0..=15 => (symbol as u8, 1),
            16 => {
                if i == 0 {
                    return Err(InflateError::InvalidSymbol);
                }
                (lengths[i - 1], 3 + reader.bits(2)? as usize)
            }
            17 => (0, 3 + reader.bits(3)? as usize),
            18 => (0, 11 + reader.bits(7)? as usize),
            _ => return Err(InflateError::InvalidSymbol),
        };
        if i + repeat > lengths.len() {
            return Err(InflateError::InvalidSymbol);
        }
        lengths[i..i + repeat].fill(value);
        i += repeat;
    }

    let lit = Huffman::new(&lengths[..hlit])?;
    let dist = Huffman::new(&lengths[hlit..])?;
    Ok((lit, dist))
}

// --- blocks --------------------------------------------------------------------------------

fn stored_block(
    reader: &mut BitReader<'_>,
    out: &mut Vec<u8>,
    limit: usize,
) -> Result<(), InflateError> {
    reader.align_to_byte();
    let head = reader.pos;
    if head + 4 > reader.data.len() {
        return Err(InflateError::UnexpectedEnd);
    }
    let len = u16::from_le_bytes([reader.data[head], reader.data[head + 1]]);
    let nlen = u16::from_le_bytes([reader.data[head + 2], reader.data[head + 3]]);
    if len != !nlen {
        return Err(InflateError::StoredLengthMismatch);
    }
    let start = head + 4;
    let end = start + len as usize;
    if end > reader.data.len() {
        return Err(InflateError::UnexpectedEnd);
    }
    if out.len() + len as usize > limit {
        return Err(InflateError::OutputTooLarge { limit });
    }
    out.extend_from_slice(&reader.data[start..end]);
    reader.pos = end;
    Ok(())
}

/// Base length for each length symbol 257..=285.
const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DISTANCE_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

fn huffman_block(
    reader: &mut BitReader<'_>,
    out: &mut Vec<u8>,
    lit: &Huffman,
    dist: &Huffman,
    limit: usize,
) -> Result<(), InflateError> {
    loop {
        let symbol = lit.decode(reader)?;
        match symbol {
            0..=255 => {
                if out.len() >= limit {
                    return Err(InflateError::OutputTooLarge { limit });
                }
                out.push(symbol as u8);
            }
            256 => return Ok(()),
            257..=285 => {
                let index = symbol as usize - 257;
                let length = LENGTH_BASE[index] as usize
                    + reader.bits(u32::from(LENGTH_EXTRA[index]))? as usize;
                let dsym = dist.decode(reader)? as usize;
                if dsym >= DISTANCE_BASE.len() {
                    return Err(InflateError::InvalidSymbol);
                }
                let distance = DISTANCE_BASE[dsym] as usize
                    + reader.bits(u32::from(DISTANCE_EXTRA[dsym]))? as usize;
                if distance > out.len() {
                    return Err(InflateError::DistanceTooFar {
                        distance,
                        produced: out.len(),
                    });
                }
                if out.len() + length > limit {
                    return Err(InflateError::OutputTooLarge { limit });
                }
                // Copied one byte at a time on purpose: a run like `distance = 1, length = 200`
                // reads bytes this same loop is writing, which is how DEFLATE encodes a repeat.
                let start = out.len() - distance;
                for offset in 0..length {
                    let byte = out[start + offset];
                    out.push(byte);
                }
            }
            _ => return Err(InflateError::InvalidSymbol),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Vectors produced once with zlib at raw-DEFLATE settings and committed as constants, so
    // the tests need no compressor of their own.

    /// `Z_FIXED`: block type 1.
    const FIXED: &[u8] = &[
        0xcb, 0x48, 0xcd, 0xc9, 0xc9, 0x57, 0xc8, 0x40, 0x22, 0xcb, 0xf3, 0x8b, 0x72, 0x52, 0x00,
    ];

    /// Level 9 over text plus every byte value: block type 2, and long codes that miss the
    /// fast table.
    const DYNAMIC: &[u8] = &[
        0x2b, 0xc9, 0x48, 0x55, 0x28, 0x2c, 0xcd, 0x4c, 0xce, 0x56, 0x48, 0x2a, 0xca, 0x2f, 0xcf,
        0x53, 0x48, 0xcb, 0xaf, 0x50, 0xc8, 0x2a, 0xcd, 0x2d, 0x28, 0x56, 0xc8, 0x2f, 0x4b, 0x2d,
        0x52, 0x28, 0x01, 0x4a, 0xe7, 0x24, 0x56, 0x55, 0x2a, 0xa4, 0xe4, 0xa7, 0xeb, 0x81, 0x79,
        0xc3, 0x5a, 0x31, 0x03, 0x23, 0x13, 0x33, 0x0b, 0x2b, 0x1b, 0x3b, 0x07, 0x27, 0x17, 0x37,
        0x0f, 0x2f, 0x1f, 0xbf, 0x80, 0xa0, 0x90, 0xb0, 0x88, 0xa8, 0x98, 0xb8, 0x84, 0xa4, 0x94,
        0xb4, 0x8c, 0xac, 0x9c, 0xbc, 0x82, 0xa2, 0x92, 0xb2, 0x8a, 0xaa, 0x9a, 0xba, 0x86, 0xa6,
        0x96, 0xb6, 0x8e, 0xae, 0x9e, 0xbe, 0x81, 0xa1, 0x91, 0xb1, 0x89, 0xa9, 0x99, 0xb9, 0x85,
        0xa5, 0x95, 0xb5, 0x8d, 0xad, 0x9d, 0xbd, 0x83, 0xa3, 0x93, 0xb3, 0x8b, 0xab, 0x9b, 0xbb,
        0x87, 0xa7, 0x97, 0xb7, 0x8f, 0xaf, 0x9f, 0x7f, 0x40, 0x60, 0x50, 0x70, 0x48, 0x68, 0x58,
        0x78, 0x44, 0x64, 0x54, 0x74, 0x4c, 0x6c, 0x5c, 0x7c, 0x42, 0x62, 0x52, 0x72, 0x4a, 0x6a,
        0x5a, 0x7a, 0x46, 0x66, 0x56, 0x76, 0x4e, 0x6e, 0x5e, 0x7e, 0x41, 0x61, 0x51, 0x71, 0x49,
        0x69, 0x59, 0x79, 0x45, 0x65, 0x55, 0x75, 0x4d, 0x6d, 0x5d, 0x7d, 0x43, 0x63, 0x53, 0x73,
        0x4b, 0x6b, 0x5b, 0x7b, 0x47, 0x67, 0x57, 0x77, 0x4f, 0x6f, 0x5f, 0xff, 0x84, 0x89, 0x93,
        0x26, 0x4f, 0x99, 0x3a, 0x6d, 0xfa, 0x8c, 0x99, 0xb3, 0x66, 0xcf, 0x99, 0x3b, 0x6f, 0xfe,
        0x82, 0x85, 0x8b, 0x16, 0x2f, 0x59, 0xba, 0x6c, 0xf9, 0x8a, 0x95, 0xab, 0x56, 0xaf, 0x59,
        0xbb, 0x6e, 0xfd, 0x86, 0x8d, 0x9b, 0x36, 0x6f, 0xd9, 0xba, 0x6d, 0xfb, 0x8e, 0x9d, 0xbb,
        0x76, 0xef, 0xd9, 0xbb, 0x6f, 0xff, 0x81, 0x83, 0x87, 0x0e, 0x1f, 0x39, 0x7a, 0xec, 0xf8,
        0x89, 0x93, 0xa7, 0x4e, 0x9f, 0x39, 0x7b, 0xee, 0xfc, 0x85, 0x8b, 0x97, 0x2e, 0x5f, 0xb9,
        0x7a, 0xed, 0xfa, 0x8d, 0x9b, 0xb7, 0x6e, 0xdf, 0xb9, 0x7b, 0xef, 0xfe, 0x83, 0x87, 0x8f,
        0x1e, 0x3f, 0x79, 0xfa, 0xec, 0xf9, 0x8b, 0x97, 0xaf, 0x5e, 0xbf, 0x79, 0xfb, 0xee, 0xfd,
        0x87, 0x8f, 0x9f, 0x3e, 0x7f, 0xf9, 0xfa, 0xed, 0xfb, 0x8f, 0x9f, 0xbf, 0x7e, 0xff, 0xf9,
        0xfb, 0xef, 0x3f, 0x00,
    ];

    /// Level 0: block type 0, which is longer than its input.
    const STORED: &[u8] = &[
        0x01, 0x23, 0x00, 0xdc, 0xff, 0x73, 0x74, 0x6f, 0x72, 0x65, 0x64, 0x20, 0x62, 0x79, 0x74,
        0x65, 0x73, 0x20, 0x70, 0x61, 0x73, 0x73, 0x20, 0x74, 0x68, 0x72, 0x6f, 0x75, 0x67, 0x68,
        0x20, 0x75, 0x6e, 0x63, 0x68, 0x61, 0x6e, 0x67, 0x65, 0x64,
    ];

    /// `"ab"` 140 times in eight bytes: a distance-2 run longer than the distance, which only
    /// decodes correctly if the copy reads bytes as it writes them.
    const OVERLAP: &[u8] = &[0x4b, 0x4c, 0x4a, 0x1c, 0x85, 0x58, 0x20, 0x00];

    #[test]
    fn decodes_a_fixed_huffman_block() {
        assert_eq!(inflate(FIXED).unwrap(), b"hello hello hello world");
    }

    #[test]
    fn decodes_a_dynamic_huffman_block() {
        let expected: Vec<u8> = {
            let mut v = b"the quick brown fox jumps over the lazy dog. ".repeat(6);
            v.extend((0..=255u8).collect::<Vec<_>>());
            v
        };
        assert_eq!(inflate(DYNAMIC).unwrap(), expected);
    }

    #[test]
    fn decodes_a_stored_block() {
        assert_eq!(
            inflate(STORED).unwrap(),
            b"stored bytes pass through unchanged"
        );
    }

    #[test]
    fn an_overlapping_back_reference_reads_bytes_as_it_writes_them() {
        assert_eq!(inflate(OVERLAP).unwrap(), b"ab".repeat(140));
    }

    #[test]
    fn a_truncated_stream_is_an_error_not_a_short_read() {
        let err = inflate(&DYNAMIC[..DYNAMIC.len() / 2]).unwrap_err();
        assert_eq!(err, InflateError::UnexpectedEnd);
    }

    #[test]
    fn a_stored_block_with_a_broken_length_check_is_rejected() {
        let mut corrupt = STORED.to_vec();
        corrupt[3] ^= 0xFF;
        assert_eq!(
            inflate(&corrupt).unwrap_err(),
            InflateError::StoredLengthMismatch
        );
    }

    #[test]
    fn block_type_three_is_reserved() {
        // Final block, type 3: bits 1, 11.
        assert_eq!(
            inflate(&[0b0000_0111]).unwrap_err(),
            InflateError::ReservedBlockType
        );
    }

    #[test]
    fn an_output_limit_stops_a_decompression_bomb() {
        let err = inflate_with_limit(OVERLAP, 16).unwrap_err();
        assert_eq!(err, InflateError::OutputTooLarge { limit: 16 });
    }

    #[test]
    fn a_size_hint_larger_than_the_limit_is_refused_before_any_work() {
        assert_eq!(
            inflate_sized(FIXED, 1_000, 100).unwrap_err(),
            InflateError::OutputTooLarge { limit: 100 }
        );
    }

    #[test]
    fn an_over_subscribed_code_table_is_rejected() {
        // Three one-bit codes: a binary tree has room for two.
        assert!(matches!(
            Huffman::new(&[1, 1, 1]),
            Err(InflateError::OverSubscribedCode)
        ));
    }

    #[test]
    fn an_incomplete_code_table_is_accepted() {
        // One code of length one leaves half the tree unused. A block with a single distance
        // code produces exactly this, and it is legal.
        assert!(Huffman::new(&[1, 0, 0]).is_ok());
    }

    #[test]
    fn codes_are_reversed_because_deflate_packs_bits_low_first() {
        assert_eq!(reverse_bits(0b110, 3), 0b011);
        assert_eq!(reverse_bits(0b1, 5), 0b10000);
    }

    #[test]
    fn the_size_hinted_path_agrees_with_the_plain_one() {
        let plain = inflate(DYNAMIC).unwrap();
        let hinted = inflate_sized(DYNAMIC, plain.len(), DEFAULT_LIMIT).unwrap();
        assert_eq!(plain, hinted);
    }
}
