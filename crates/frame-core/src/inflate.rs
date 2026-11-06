//! Unpacking the zlib stream `/panel` sends (RFC 1950 and 1951), straight into a page buffer.
//!
//! Small and allocation-free, so the ESP32 can inflate a page into PSRAM as it is, and fast
//! enough there: codes up to [`FAST_BITS`] long are read with one table lookup.

use core::fmt;

/// Codes up to this many bits are decoded with one lookup.
const FAST_BITS: u32 = 9;
const MAX_BITS: usize = 15;
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
const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

/// Why a stream could not be inflated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
#[non_exhaustive]
pub enum InflateError {
    /// The two-byte zlib header is not deflate with a valid check, or asks for a dictionary.
    BadHeader,
    /// The stream ends early.
    Truncated,
    /// A block type, code table or symbol is not valid deflate.
    Corrupt,
    /// The stream unpacks to more bytes than the buffer holds.
    TooLong,
    /// The Adler-32 at the end does not match what was unpacked.
    Checksum,
}

impl fmt::Display for InflateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BadHeader => "not a zlib stream",
            Self::Truncated => "zlib stream ends early",
            Self::Corrupt => "zlib stream is corrupt",
            Self::TooLong => "zlib stream unpacks to more than the buffer holds",
            Self::Checksum => "zlib checksum does not match",
        })
    }
}

impl core::error::Error for InflateError {}

/// Inflates the zlib stream `stream` into `out` and returns how many bytes it filled. The
/// stream's Adler-32 is checked.
///
/// # Errors
///
/// Returns [`InflateError`] when the stream is not valid zlib, is cut short, holds more than
/// `out`, or fails its checksum.
pub fn inflate_zlib(stream: &[u8], out: &mut [u8]) -> Result<usize, InflateError> {
    let [method, flags, ..] = *stream else {
        return Err(InflateError::Truncated);
    };
    let header: u16 = u16::from_be_bytes([method, flags]);
    if method & 0x0f != 8 || method >> 4 > 7 || !header.is_multiple_of(31) || flags & 0x20 != 0 {
        return Err(InflateError::BadHeader);
    }
    let mut bits = Bits::new(&stream[2..]);
    let length: usize = inflate_into(&mut bits, out)?;
    let tail: usize = 2 + bits.byte_position();
    let checksum: &[u8] = stream.get(tail..tail + 4).ok_or(InflateError::Truncated)?;
    if checksum != adler32(&out[..length]).to_be_bytes() {
        return Err(InflateError::Checksum);
    }
    Ok(length)
}

fn inflate_into(bits: &mut Bits<'_>, out: &mut [u8]) -> Result<usize, InflateError> {
    let mut filled: usize = 0;
    loop {
        let last: bool = bits.take(1)? == 1;
        match bits.take(2)? {
            0 => filled = stored(bits, out, filled)?,
            1 => {
                let (literals, distances) = fixed_tables();
                filled = codes(bits, out, filled, &literals, &distances)?;
            }
            2 => {
                let (literals, distances) = dynamic_tables(bits)?;
                filled = codes(bits, out, filled, &literals, &distances)?;
            }
            _ => return Err(InflateError::Corrupt),
        }
        if last {
            return Ok(filled);
        }
    }
}

fn stored(bits: &mut Bits<'_>, out: &mut [u8], filled: usize) -> Result<usize, InflateError> {
    bits.align();
    let length: u32 = bits.take(16)?;
    let inverse: u32 = bits.take(16)?;
    if length != !inverse & 0xffff {
        return Err(InflateError::Corrupt);
    }
    let length: usize = length as usize;
    let end: usize = filled + length;
    let target: &mut [u8] = out.get_mut(filled..end).ok_or(InflateError::TooLong)?;
    bits.copy_bytes(target)?;
    Ok(end)
}

fn codes(
    bits: &mut Bits<'_>,
    out: &mut [u8],
    mut filled: usize,
    literals: &Huffman,
    distances: &Huffman,
) -> Result<usize, InflateError> {
    loop {
        let symbol: usize = literals.decode(bits)?;
        if symbol < 256 {
            *out.get_mut(filled).ok_or(InflateError::TooLong)? = symbol as u8;
            filled += 1;
            continue;
        }
        if symbol == 256 {
            return Ok(filled);
        }
        let index: usize = symbol - 257;
        let (Some(&base), Some(&extra)) = (LENGTH_BASE.get(index), LENGTH_EXTRA.get(index)) else {
            return Err(InflateError::Corrupt);
        };
        let length: usize = usize::from(base) + bits.take(u32::from(extra))? as usize;
        let index: usize = distances.decode(bits)?;
        let (Some(&base), Some(&extra)) = (DISTANCE_BASE.get(index), DISTANCE_EXTRA.get(index))
        else {
            return Err(InflateError::Corrupt);
        };
        let distance: usize = usize::from(base) + bits.take(u32::from(extra))? as usize;
        if distance > filled {
            return Err(InflateError::Corrupt);
        }
        if filled + length > out.len() {
            return Err(InflateError::TooLong);
        }
        if distance >= length {
            out.copy_within(filled - distance..filled - distance + length, filled);
        } else {
            // Overlapping: each byte may be one this copy just wrote.
            for at in filled..filled + length {
                out[at] = out[at - distance];
            }
        }
        filled += length;
    }
}

fn fixed_tables() -> (Huffman, Huffman) {
    let mut lengths: [u8; 320] = [0; 320];
    lengths[..144].fill(8);
    lengths[144..256].fill(9);
    lengths[256..280].fill(7);
    lengths[280..288].fill(8);
    lengths[288..].fill(5);
    // The fixed tables are complete codes, so building them cannot fail.
    (
        Huffman::new(&lengths[..288]).unwrap_or_default(),
        Huffman::new(&lengths[288..]).unwrap_or_default(),
    )
}

fn dynamic_tables(bits: &mut Bits<'_>) -> Result<(Huffman, Huffman), InflateError> {
    let literal_count: usize = bits.take(5)? as usize + 257;
    let distance_count: usize = bits.take(5)? as usize + 1;
    let length_count: usize = bits.take(4)? as usize + 4;
    if literal_count > 286 || distance_count > 30 {
        return Err(InflateError::Corrupt);
    }
    let mut length_lengths: [u8; 19] = [0; 19];
    for &symbol in &CODE_LENGTH_ORDER[..length_count] {
        length_lengths[symbol] = bits.take(3)? as u8;
    }
    let length_code = Huffman::new(&length_lengths)?;
    let total: usize = literal_count + distance_count;
    let mut lengths: [u8; 320] = [0; 320];
    let mut index: usize = 0;
    while index < total {
        let symbol: usize = length_code.decode(bits)?;
        let (value, times): (u8, usize) = match symbol {
            0..=15 => (symbol as u8, 1),
            16 => {
                let previous: u8 = *index
                    .checked_sub(1)
                    .and_then(|before| lengths.get(before))
                    .ok_or(InflateError::Corrupt)?;
                (previous, 3 + bits.take(2)? as usize)
            }
            17 => (0, 3 + bits.take(3)? as usize),
            18 => (0, 11 + bits.take(7)? as usize),
            _ => return Err(InflateError::Corrupt),
        };
        if index + times > total {
            return Err(InflateError::Corrupt);
        }
        lengths[index..index + times].fill(value);
        index += times;
    }
    if lengths[256] == 0 {
        return Err(InflateError::Corrupt);
    }
    Ok((
        Huffman::new(&lengths[..literal_count])?,
        Huffman::new(&lengths[literal_count..total])?,
    ))
}

/// A canonical Huffman code: a lookup table for short codes, counts and symbols for the rest.
struct Huffman {
    /// Indexed by the next [`FAST_BITS`] bits: the code's length shifted up by 9, then its
    /// symbol; 0 when the code is longer.
    fast: [u16; 1 << FAST_BITS],
    counts: [u16; MAX_BITS + 1],
    symbols: [u16; 288],
}

impl Default for Huffman {
    fn default() -> Self {
        Self {
            fast: [0; 1 << FAST_BITS],
            counts: [0; MAX_BITS + 1],
            symbols: [0; 288],
        }
    }
}

impl Huffman {
    fn new(lengths: &[u8]) -> Result<Self, InflateError> {
        let mut table = Self::default();
        for &length in lengths {
            table.counts[usize::from(length)] += 1;
        }
        table.counts[0] = 0;
        // Reject over-subscribed codes; incomplete ones are allowed, as zlib allows them.
        let mut left: i32 = 1;
        for length in 1..=MAX_BITS {
            left = (left << 1) - i32::from(table.counts[length]);
            if left < 0 {
                return Err(InflateError::Corrupt);
            }
        }
        let mut offsets: [u16; MAX_BITS + 2] = [0; MAX_BITS + 2];
        for length in 1..=MAX_BITS {
            offsets[length + 1] = offsets[length] + table.counts[length];
        }
        let mut next_code: [u32; MAX_BITS + 1] = [0; MAX_BITS + 1];
        let mut code: u32 = 0;
        for (next, &count) in next_code.iter_mut().zip(&table.counts).skip(1) {
            *next = code;
            code = (code + u32::from(count)) << 1;
        }
        for (symbol, &length) in lengths.iter().enumerate() {
            if length == 0 {
                continue;
            }
            let length: usize = usize::from(length);
            table.symbols[usize::from(offsets[length])] = symbol as u16;
            offsets[length] += 1;
            let code: u32 = next_code[length];
            next_code[length] += 1;
            if length as u32 <= FAST_BITS {
                // Bits arrive least significant first, so the table is indexed by the code
                // reversed, and every entry that starts with it decodes to it.
                let reversed: usize = (code.reverse_bits() >> (32 - length)) as usize;
                let entry: u16 = ((length as u16) << 9) | symbol as u16;
                let mut index: usize = reversed;
                while index < table.fast.len() {
                    table.fast[index] = entry;
                    index += 1 << length;
                }
            }
        }
        Ok(table)
    }

    fn decode(&self, bits: &mut Bits<'_>) -> Result<usize, InflateError> {
        let (peeked, available) = bits.peek(FAST_BITS);
        let entry: u16 = self.fast[peeked as usize];
        let length: u32 = u32::from(entry >> 9);
        if entry != 0 && length <= available {
            bits.consume(length);
            return Ok(usize::from(entry & 0x1ff));
        }
        // A long code: walk the canonical code a bit at a time.
        let mut code: i32 = 0;
        let mut first: i32 = 0;
        let mut index: i32 = 0;
        for length in 1..=MAX_BITS {
            code |= bits.take(1)? as i32;
            let count: i32 = i32::from(self.counts[length]);
            if code - count < first {
                return Ok(usize::from(self.symbols[(index + code - first) as usize]));
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }
        Err(InflateError::Corrupt)
    }
}

/// Reads bits least significant first.
struct Bits<'a> {
    data: &'a [u8],
    /// The next byte to load.
    next: usize,
    buffer: u64,
    count: u32,
}

impl<'a> Bits<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            next: 0,
            buffer: 0,
            count: 0,
        }
    }

    fn refill(&mut self) {
        while self.count <= 56 {
            let Some(&byte) = self.data.get(self.next) else {
                return;
            };
            self.buffer |= u64::from(byte) << self.count;
            self.next += 1;
            self.count += 8;
        }
    }

    /// The next `width` bits without taking them, and how many of them are real.
    fn peek(&mut self, width: u32) -> (u32, u32) {
        if self.count < width {
            self.refill();
        }
        let mask: u64 = (1 << width) - 1;
        ((self.buffer & mask) as u32, self.count.min(width))
    }

    fn consume(&mut self, width: u32) {
        self.buffer >>= width;
        self.count -= width;
    }

    fn take(&mut self, width: u32) -> Result<u32, InflateError> {
        if width == 0 {
            return Ok(0);
        }
        let (value, available) = self.peek(width);
        if available < width {
            return Err(InflateError::Truncated);
        }
        self.consume(width);
        Ok(value)
    }

    /// Skips to the next byte boundary.
    fn align(&mut self) {
        let skip: u32 = self.count % 8;
        self.consume(skip);
    }

    /// Copies whole bytes out; the reader must be aligned.
    fn copy_bytes(&mut self, target: &mut [u8]) -> Result<(), InflateError> {
        let mut copied: usize = 0;
        while copied < target.len() && self.count >= 8 {
            target[copied] = self.take(8)? as u8;
            copied += 1;
        }
        let rest: usize = target.len() - copied;
        let source: &[u8] = self
            .data
            .get(self.next..self.next + rest)
            .ok_or(InflateError::Truncated)?;
        target[copied..].copy_from_slice(source);
        self.next += rest;
        Ok(())
    }

    /// The byte after the last one read from, once the stream is done.
    fn byte_position(&self) -> usize {
        self.next - (self.count / 8) as usize
    }
}

/// The Adler-32 checksum zlib ends a stream with.
fn adler32(data: &[u8]) -> u32 {
    const MOD: u32 = 65_521;
    let (mut low, mut high): (u32, u32) = (1, 0);
    for chunk in data.chunks(5_552) {
        for &byte in chunk {
            low += u32::from(byte);
            high += low;
        }
        low %= MOD;
        high %= MOD;
    }
    (high << 16) | low
}

#[cfg(test)]
mod test {
    use super::*;

    /// "hello hello hello" from Python's `zlib.compress(..., 9)`: fixed Huffman with a match.
    const HELLO: [u8; 16] = [
        0x78, 0xda, 0xcb, 0x48, 0xcd, 0xc9, 0xc9, 0x57, 0xc8, 0x40, 0x90, 0x00, 0x3a, 0x2e, 0x06,
        0x7d,
    ];

    #[test]
    fn inflates_fixed_and_stored_blocks() {
        let mut out = [0u8; 64];
        // Level 0: one stored block holding "abc".
        let stored: [u8; 14] = [
            0x78, 0x01, 0x01, 0x03, 0x00, 0xfc, 0xff, b'a', b'b', b'c', 0x02, 0x4d, 0x01, 0x27,
        ];
        assert_eq!(inflate_zlib(&stored, &mut out), Ok(3));
        assert_eq!(&out[..3], b"abc");
        assert_eq!(inflate_zlib(&HELLO, &mut out), Ok(17));
        assert_eq!(&out[..17], b"hello hello hello");
    }

    #[test]
    fn rejects_damage() {
        let mut out = [0u8; 64];
        let stored: [u8; 14] = [
            0x78, 0x01, 0x01, 0x03, 0x00, 0xfc, 0xff, b'a', b'b', b'c', 0x02, 0x4d, 0x01, 0x27,
        ];
        let mut bad = stored;
        bad[8] = b'x';
        assert_eq!(inflate_zlib(&bad, &mut out), Err(InflateError::Checksum));
        assert_eq!(
            inflate_zlib(&stored[..12], &mut out),
            Err(InflateError::Truncated)
        );
        assert_eq!(
            inflate_zlib(&[0x78, 0x00], &mut out),
            Err(InflateError::BadHeader)
        );
        assert_eq!(
            inflate_zlib(&stored, &mut out[..2]),
            Err(InflateError::TooLong)
        );
    }

    #[test]
    fn adler32_matches_the_rfc_example() {
        assert_eq!(adler32(b"Wikipedia"), 0x11e6_0398);
    }
}
