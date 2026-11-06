//! A small zlib compressor: LZ77 with hash chains, coded with deflate's fixed Huffman table.
//!
//! Replaces the `png` crate and its five compression crates. Fixed codes compress the flat
//! areas of a page nearly as well as dynamic ones, and need no tables to be sent.

/// Longest match deflate can express.
const MAX_MATCH: usize = 258;
/// Shortest match worth a back-reference.
const MIN_MATCH: usize = 3;
/// How far back matches may reach.
const WINDOW: usize = 32 * 1024;
/// Candidates tried per position; more compresses better and runs slower.
const CHAIN_LIMIT: usize = 24;
const HASH_BITS: u32 = 15;

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

/// Returns `data` as a zlib stream (RFC 1950) holding one fixed-Huffman deflate block.
pub(crate) fn zlib(data: &[u8]) -> Vec<u8> {
    let mut bits = BitWriter::with_capacity(data.len() / 4 + 64);
    bits.bytes.extend_from_slice(&[0x78, 0x01]);
    bits.write(1, 1); // Final block.
    bits.write(1, 2); // Fixed Huffman codes.
    compress(data, &mut bits);
    write_symbol(&mut bits, 256);
    let mut out: Vec<u8> = bits.finish();
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn compress(data: &[u8], bits: &mut BitWriter) {
    let mut head: Vec<i32> = vec![-1; 1 << HASH_BITS];
    let mut previous: Vec<i32> = vec![-1; WINDOW];
    let mut position: usize = 0;
    let insert = |head: &mut Vec<i32>, previous: &mut Vec<i32>, at: usize| {
        if at + MIN_MATCH <= data.len() {
            let hash: usize = hash(&data[at..at + MIN_MATCH]);
            previous[at % WINDOW] = head[hash];
            head[hash] = i32::try_from(at).unwrap_or(-1);
        }
    };
    while position < data.len() {
        let (length, distance) = longest_match(data, position, &head, &previous);
        if length >= MIN_MATCH {
            write_match(bits, length, distance);
            for at in position..position + length {
                insert(&mut head, &mut previous, at);
            }
            position += length;
        } else {
            write_symbol(bits, u16::from(data[position]));
            insert(&mut head, &mut previous, position);
            position += 1;
        }
    }
}

fn hash(bytes: &[u8]) -> usize {
    let value: u32 = u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16);
    (value.wrapping_mul(0x9e37_79b1) >> (32 - HASH_BITS)) as usize
}

fn longest_match(data: &[u8], position: usize, head: &[i32], previous: &[i32]) -> (usize, usize) {
    if position + MIN_MATCH > data.len() {
        return (0, 0);
    }
    let limit: usize = (data.len() - position).min(MAX_MATCH);
    let mut best: (usize, usize) = (0, 0);
    let mut candidate: i32 = head[hash(&data[position..position + MIN_MATCH])];
    for _ in 0..CHAIN_LIMIT {
        let Ok(start) = usize::try_from(candidate) else {
            break;
        };
        if start >= position || position - start > WINDOW - 1 {
            break;
        }
        let length: usize = data[start..]
            .iter()
            .zip(&data[position..position + limit])
            .take_while(|(left, right)| left == right)
            .count();
        if length > best.0 {
            best = (length, position - start);
            if length == limit {
                break;
            }
        }
        candidate = previous[start % WINDOW];
    }
    best
}

fn write_match(bits: &mut BitWriter, length: usize, distance: usize) {
    let code: usize = LENGTH_BASE
        .iter()
        .rposition(|&base| usize::from(base) <= length)
        .unwrap_or(0);
    write_symbol(bits, 257 + u16::try_from(code).unwrap_or(0));
    bits.write(
        u32::try_from(length - usize::from(LENGTH_BASE[code])).unwrap_or(0),
        LENGTH_EXTRA[code],
    );
    let code: usize = DISTANCE_BASE
        .iter()
        .rposition(|&base| usize::from(base) <= distance)
        .unwrap_or(0);
    bits.write_reversed(u32::try_from(code).unwrap_or(0), 5);
    bits.write(
        u32::try_from(distance - usize::from(DISTANCE_BASE[code])).unwrap_or(0),
        DISTANCE_EXTRA[code],
    );
}

/// Writes a literal/length symbol with the fixed code of RFC 1951 section 3.2.6.
fn write_symbol(bits: &mut BitWriter, symbol: u16) {
    let symbol: u32 = u32::from(symbol);
    match symbol {
        0..=143 => bits.write_reversed(0x30 + symbol, 8),
        144..=255 => bits.write_reversed(0x190 + symbol - 144, 9),
        256..=279 => bits.write_reversed(symbol - 256, 7),
        _ => bits.write_reversed(0xc0 + symbol - 280, 8),
    }
}

fn adler32(data: &[u8]) -> u32 {
    let (mut low, mut high): (u32, u32) = (1, 0);
    for chunk in data.chunks(5552) {
        for &byte in chunk {
            low += u32::from(byte);
            high += low;
        }
        low %= 65521;
        high %= 65521;
    }
    (high << 16) | low
}

/// Packs bit fields least significant bit first, as deflate wants.
struct BitWriter {
    bytes: Vec<u8>,
    pending: u64,
    count: u8,
}

impl BitWriter {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity),
            pending: 0,
            count: 0,
        }
    }

    /// Writes the low `width` bits of `value`, least significant first.
    fn write(&mut self, value: u32, width: u8) {
        if width == 0 {
            return;
        }
        self.pending |= u64::from(value & ((1 << width) - 1)) << self.count;
        self.count += width;
        while self.count >= 8 {
            self.bytes.push((self.pending & 0xff) as u8);
            self.pending >>= 8;
            self.count -= 8;
        }
    }

    /// Writes a Huffman code, most significant bit first.
    fn write_reversed(&mut self, code: u32, width: u8) {
        self.write(code.reverse_bits() >> (32 - u32::from(width)), width);
    }

    fn finish(mut self) -> Vec<u8> {
        if self.count > 0 {
            self.bytes.push((self.pending & 0xff) as u8);
        }
        self.bytes
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// A test-only inflater for fixed-Huffman blocks, to prove the stream decodes.
    fn inflate(stream: &[u8]) -> Vec<u8> {
        let data: &[u8] = &stream[2..stream.len() - 4];
        let mut at: usize = 0;
        let mut bit = || -> u32 {
            let value: u32 = u32::from(data[at / 8] >> (at % 8)) & 1;
            at += 1;
            value
        };
        let read = |width: u8, bit: &mut dyn FnMut() -> u32| -> u32 {
            (0..width).fold(0, |value, index| value | (bit() << index))
        };
        let code = |width: u8, bit: &mut dyn FnMut() -> u32| -> u32 {
            (0..width).fold(0, |value, _| (value << 1) | bit())
        };
        assert_eq!(read(1, &mut bit), 1);
        assert_eq!(read(2, &mut bit), 1);
        let mut out: Vec<u8> = Vec::new();
        loop {
            let mut value: u32 = code(7, &mut bit);
            let symbol: u32 = if value <= 0x17 {
                value + 256
            } else {
                value = (value << 1) | bit();
                if (0x30..=0xbf).contains(&value) {
                    value - 0x30
                } else if (0xc0..=0xc7).contains(&value) {
                    value - 0xc0 + 280
                } else {
                    ((value << 1) | bit()) - 0x190 + 144
                }
            };
            match symbol {
                0..=255 => out.push(symbol as u8),
                256 => break,
                _ => {
                    let index: usize = (symbol - 257) as usize;
                    let length: usize = usize::from(LENGTH_BASE[index])
                        + read(LENGTH_EXTRA[index], &mut bit) as usize;
                    let index: usize = code(5, &mut bit) as usize;
                    let distance: usize = usize::from(DISTANCE_BASE[index])
                        + read(DISTANCE_EXTRA[index], &mut bit) as usize;
                    for _ in 0..length {
                        out.push(out[out.len() - distance]);
                    }
                }
            }
        }
        assert_eq!(stream[stream.len() - 4..], adler32(&out).to_be_bytes());
        out
    }

    #[test]
    fn round_trips_text_runs_and_noise() {
        let mut noise: Vec<u8> = Vec::new();
        let mut state: u32 = 1;
        for _ in 0..100_000 {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            noise.push((state >> 16) as u8);
        }
        let runs: Vec<u8> = (0..200_000).map(|index| (index / 1000) as u8).collect();
        for data in [&b""[..], b"a", b"abcabcabcabcabc", &runs, &noise] {
            assert_eq!(inflate(&zlib(data)), data);
        }
        assert!(
            zlib(&runs).len() < 3_000,
            "runs compress: {}",
            zlib(&runs).len()
        );
    }

    #[test]
    fn adler32_matches_the_rfc_example() {
        assert_eq!(adler32(b"Wikipedia"), 0x11e6_0398);
    }
}
