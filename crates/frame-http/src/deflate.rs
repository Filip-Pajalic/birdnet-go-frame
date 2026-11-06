//! A small zlib compressor: LZ77 with hash chains and lazy matching, coded with dynamic Huffman
//! tables (RFC 1951).
//!
//! Replaces the `png` crate and its five compression crates. A 13.3" panel page (960 000 bytes)
//! compresses to about 216 000, within 7 percent of zlib's best level, in under a tenth of a
//! second; panels download fewer bytes, so the radio is on for less time.

/// Longest match deflate can express.
const MAX_MATCH: usize = 258;
/// Shortest match worth a back-reference.
const MIN_MATCH: usize = 3;
/// How far back matches may reach.
const WINDOW: usize = 32 * 1024;
/// Candidates tried per position; more compresses better and runs slower.
const CHAIN_LIMIT: usize = 128;
/// A match this long is taken at once, without looking one byte further for a longer one.
const GOOD_ENOUGH: usize = 128;
const HASH_BITS: u32 = 15;
/// Tokens per block; each block gets Huffman codes fitted to its own content.
const BLOCK_TOKENS: usize = 16 * 1024;
/// Longest Huffman code for literals, lengths and distances, and for the code-length code.
const MAX_CODE: u8 = 15;
const MAX_CODE_LENGTH_CODE: u8 = 7;
const END_OF_BLOCK: usize = 256;

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
/// The order code-length code lengths are sent in.
const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

/// One step of the LZ77 parse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token {
    Literal(u8),
    /// A copy of `length` bytes from `distance` back.
    Match {
        length: u16,
        distance: u16,
    },
}

/// A Huffman symbol and the extra bits after it: their value and how many there are.
type Coded = (usize, (u32, u8));

impl Token {
    /// The literal/length symbol with its extra bits, and the distance symbol with its own.
    fn symbols(self) -> (usize, (u32, u8), Option<Coded>) {
        match self {
            Self::Literal(byte) => (usize::from(byte), (0, 0), None),
            Self::Match { length, distance } => {
                let code: usize = LENGTH_BASE
                    .iter()
                    .rposition(|&base| base <= length)
                    .unwrap_or(0);
                let length_extra = (u32::from(length - LENGTH_BASE[code]), LENGTH_EXTRA[code]);
                let distance_code: usize = DISTANCE_BASE
                    .iter()
                    .rposition(|&base| base <= distance)
                    .unwrap_or(0);
                let distance_extra = (
                    u32::from(distance - DISTANCE_BASE[distance_code]),
                    DISTANCE_EXTRA[distance_code],
                );
                (
                    257 + code,
                    length_extra,
                    Some((distance_code, distance_extra)),
                )
            }
        }
    }
}

/// Returns `data` as a zlib stream (RFC 1950) of dynamic-Huffman deflate blocks, as `/panel`
/// sends pages. [`frame_core::inflate_zlib`] unpacks it.
#[must_use]
pub fn zlib(data: &[u8]) -> Vec<u8> {
    let tokens: Vec<Token> = parse(data);
    let mut bits = BitWriter::with_capacity(data.len() / 4 + 64);
    // Window 32 KiB, default level; the check bits make the header a multiple of 31.
    bits.bytes.extend_from_slice(&[0x78, 0x9c]);
    let blocks: Vec<&[Token]> = if tokens.is_empty() {
        vec![&tokens[..]]
    } else {
        tokens.chunks(BLOCK_TOKENS).collect()
    };
    let last: usize = blocks.len() - 1;
    for (index, block) in blocks.into_iter().enumerate() {
        write_block(&mut bits, block, index == last);
    }
    let mut out: Vec<u8> = bits.finish();
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

/// Finds matches with hash chains. Lazy: when the next byte starts a longer match, the current
/// byte goes out as a literal instead, as zlib does.
fn parse(data: &[u8]) -> Vec<Token> {
    let mut chains = Chains::new();
    let mut tokens: Vec<Token> = Vec::with_capacity(data.len() / 3);
    let mut position: usize = 0;
    while position < data.len() {
        let (length, distance) = chains.longest_match(data, position);
        chains.insert(data, position);
        if length < MIN_MATCH {
            tokens.push(Token::Literal(data[position]));
            position += 1;
            continue;
        }
        if length < GOOD_ENOUGH && position + 1 < data.len() {
            let (next_length, _) = chains.longest_match(data, position + 1);
            if next_length > length {
                tokens.push(Token::Literal(data[position]));
                position += 1;
                continue;
            }
        }
        tokens.push(Token::Match {
            length: u16::try_from(length).unwrap_or(258),
            distance: u16::try_from(distance).unwrap_or(1),
        });
        for at in position + 1..position + length {
            chains.insert(data, at);
        }
        position += length;
    }
    tokens
}

/// Where each three-byte sequence was last seen.
struct Chains {
    head: Vec<i32>,
    previous: Vec<i32>,
}

impl Chains {
    fn new() -> Self {
        Self {
            head: vec![-1; 1 << HASH_BITS],
            previous: vec![-1; WINDOW],
        }
    }

    fn insert(&mut self, data: &[u8], at: usize) {
        if at + MIN_MATCH <= data.len() {
            let hash: usize = hash(&data[at..at + MIN_MATCH]);
            self.previous[at % WINDOW] = self.head[hash];
            self.head[hash] = i32::try_from(at).unwrap_or(-1);
        }
    }

    fn longest_match(&self, data: &[u8], position: usize) -> (usize, usize) {
        if position + MIN_MATCH > data.len() {
            return (0, 0);
        }
        let limit: usize = (data.len() - position).min(MAX_MATCH);
        let mut best: (usize, usize) = (0, 0);
        let mut candidate: i32 = self.head[hash(&data[position..position + MIN_MATCH])];
        for _ in 0..CHAIN_LIMIT {
            let Ok(start) = usize::try_from(candidate) else {
                break;
            };
            if start >= position || position - start > WINDOW - 1 {
                break;
            }
            // Skip candidates that cannot beat the best so far.
            if data[start + best.0.min(limit - 1)] == data[position + best.0.min(limit - 1)] {
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
            }
            candidate = self.previous[start % WINDOW];
        }
        best
    }
}

fn hash(bytes: &[u8]) -> usize {
    let value: u32 = u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16);
    (value.wrapping_mul(0x9e37_79b1) >> (32 - HASH_BITS)) as usize
}

/// Writes one dynamic-Huffman block: its code tables, then its tokens.
fn write_block(bits: &mut BitWriter, tokens: &[Token], last: bool) {
    let mut literal_counts: [u32; 286] = [0; 286];
    let mut distance_counts: [u32; 30] = [0; 30];
    for token in tokens {
        let (symbol, _, distance) = token.symbols();
        literal_counts[symbol] += 1;
        if let Some((code, _)) = distance {
            distance_counts[code] += 1;
        }
    }
    literal_counts[END_OF_BLOCK] = 1;
    let literal_lengths: Vec<u8> = code_lengths(&literal_counts, MAX_CODE);
    let mut distance_lengths: Vec<u8> = code_lengths(&distance_counts, MAX_CODE);
    if distance_lengths.iter().all(|&length| length == 0) {
        // A block without matches still describes one distance code.
        distance_lengths[0] = 1;
    }
    let literal_used: usize = used(&literal_lengths, 257);
    let distance_used: usize = used(&distance_lengths, 1);

    // Both code-length lists are sent as one run-length-coded sequence.
    let joined: Vec<u8> = literal_lengths[..literal_used]
        .iter()
        .chain(&distance_lengths[..distance_used])
        .copied()
        .collect();
    let runs: Vec<(u8, u8)> = run_lengths(&joined);
    let mut length_counts: [u32; 19] = [0; 19];
    for &(symbol, _) in &runs {
        length_counts[usize::from(symbol)] += 1;
    }
    let length_lengths: Vec<u8> = code_lengths(&length_counts, MAX_CODE_LENGTH_CODE);
    let length_used: usize = CODE_LENGTH_ORDER
        .iter()
        .rposition(|&symbol| length_lengths[symbol] != 0)
        .map_or(4, |index| (index + 1).max(4));

    bits.write(u32::from(last), 1);
    bits.write(2, 2);
    bits.write(u32::try_from(literal_used - 257).unwrap_or(0), 5);
    bits.write(u32::try_from(distance_used - 1).unwrap_or(0), 5);
    bits.write(u32::try_from(length_used - 4).unwrap_or(0), 4);
    for &symbol in &CODE_LENGTH_ORDER[..length_used] {
        bits.write(u32::from(length_lengths[symbol]), 3);
    }
    let length_codes: Vec<u32> = canonical_codes(&length_lengths);
    for (symbol, extra) in runs {
        let symbol: usize = usize::from(symbol);
        bits.write_reversed(length_codes[symbol], length_lengths[symbol]);
        match symbol {
            16 => bits.write(u32::from(extra), 2),
            17 => bits.write(u32::from(extra), 3),
            18 => bits.write(u32::from(extra), 7),
            _ => {}
        }
    }

    let literal_codes: Vec<u32> = canonical_codes(&literal_lengths);
    let distance_codes: Vec<u32> = canonical_codes(&distance_lengths);
    for token in tokens {
        let (symbol, (extra, extra_bits), distance) = token.symbols();
        bits.write_reversed(literal_codes[symbol], literal_lengths[symbol]);
        bits.write(extra, extra_bits);
        if let Some((code, (extra, extra_bits))) = distance {
            bits.write_reversed(distance_codes[code], distance_lengths[code]);
            bits.write(extra, extra_bits);
        }
    }
    bits.write_reversed(literal_codes[END_OF_BLOCK], literal_lengths[END_OF_BLOCK]);
}

/// How many entries to send: up to the last non-zero length, at least `least`.
fn used(lengths: &[u8], least: usize) -> usize {
    lengths
        .iter()
        .rposition(|&length| length != 0)
        .map_or(least, |index| (index + 1).max(least))
}

/// Run-length codes a list of code lengths with symbols 16 (repeat the previous 3 to 6 times),
/// 17 (3 to 10 zeros) and 18 (11 to 138 zeros). Returns symbols with their extra-bit values.
fn run_lengths(lengths: &[u8]) -> Vec<(u8, u8)> {
    let mut runs: Vec<(u8, u8)> = Vec::new();
    let mut index: usize = 0;
    while index < lengths.len() {
        let value: u8 = lengths[index];
        let run: usize = lengths[index..]
            .iter()
            .take_while(|&&length| length == value)
            .count();
        let mut left: usize = run;
        if value == 0 {
            while left >= 11 {
                let take: usize = left.min(138);
                runs.push((18, u8::try_from(take - 11).unwrap_or(0)));
                left -= take;
            }
            if left >= 3 {
                runs.push((17, u8::try_from(left - 3).unwrap_or(0)));
                left = 0;
            }
        } else {
            runs.push((value, 0));
            left -= 1;
            while left >= 3 {
                let take: usize = left.min(6);
                runs.push((16, u8::try_from(take - 3).unwrap_or(0)));
                left -= take;
            }
        }
        runs.extend(std::iter::repeat_n((value, 0), left));
        index += run;
    }
    runs
}

/// Returns optimal code lengths for `counts`, none longer than `limit` bits (package-merge).
/// Unused symbols get 0; a lone used symbol gets 1.
fn code_lengths(counts: &[u32], limit: u8) -> Vec<u8> {
    let mut lengths: Vec<u8> = vec![0; counts.len()];
    let mut leaves: Vec<(u64, usize)> = counts
        .iter()
        .enumerate()
        .filter(|(_, count)| **count > 0)
        .map(|(symbol, &count)| (u64::from(count), symbol))
        .collect();
    match leaves.len() {
        0 => return lengths,
        1 => {
            lengths[leaves[0].1] = 1;
            return lengths;
        }
        _ => {}
    }
    leaves.sort_unstable();
    // Each item is a weight and how many times each leaf (by sorted position) is inside it.
    let leaf_items: Vec<(u64, Vec<u16>)> = leaves
        .iter()
        .enumerate()
        .map(|(position, &(weight, _))| {
            let mut inside: Vec<u16> = vec![0; leaves.len()];
            inside[position] = 1;
            (weight, inside)
        })
        .collect();
    let mut items: Vec<(u64, Vec<u16>)> = leaf_items.clone();
    for _ in 1..limit {
        let packages = items.chunks_exact(2).map(|pair| {
            let inside: Vec<u16> = pair[0]
                .1
                .iter()
                .zip(&pair[1].1)
                .map(|(a, b)| a + b)
                .collect();
            (pair[0].0 + pair[1].0, inside)
        });
        let mut merged: Vec<(u64, Vec<u16>)> = leaf_items.clone();
        merged.extend(packages);
        merged.sort_by_key(|item| item.0);
        items = merged;
    }
    for (_, inside) in items.iter().take(2 * leaves.len() - 2) {
        for (position, &times) in inside.iter().enumerate() {
            let symbol: usize = leaves[position].1;
            lengths[symbol] += u8::try_from(times).unwrap_or(0);
        }
    }
    lengths
}

/// Returns the canonical Huffman code for each symbol from its code length (RFC 1951 3.2.2).
fn canonical_codes(lengths: &[u8]) -> Vec<u32> {
    let mut counts: [u32; 16] = [0; 16];
    for &length in lengths {
        counts[usize::from(length)] += 1;
    }
    counts[0] = 0;
    let mut next: [u32; 16] = [0; 16];
    let mut code: u32 = 0;
    for bits in 1..16 {
        code = (code + counts[bits - 1]) << 1;
        next[bits] = code;
    }
    lengths
        .iter()
        .map(|&length| {
            if length == 0 {
                return 0;
            }
            let assigned: u32 = next[usize::from(length)];
            next[usize::from(length)] += 1;
            assigned
        })
        .collect()
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
        if width == 0 {
            return;
        }
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

    /// Inflates with the panel's own inflater, which checks the Adler-32 too.
    fn inflate(stream: &[u8]) -> Vec<u8> {
        let mut out: Vec<u8> = vec![0; 1 << 20];
        let length: usize = frame_core::inflate_zlib(stream, &mut out).expect("inflates");
        out.truncate(length);
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
        // Dither-like: few distinct bytes, short repeats.
        let inks: Vec<u8> = (0..300_000_u32)
            .map(|index| [0x11, 0x12, 0x21, 0x13, 0x51][((index * 7) ^ (index >> 5)) as usize % 5])
            .collect();
        let runs: Vec<u8> = (0..200_000).map(|index| (index / 1000) as u8).collect();
        for data in [&b""[..], b"a", b"abcabcabcabcabc", &runs, &noise, &inks] {
            assert_eq!(inflate(&zlib(data)), data);
        }
        assert!(
            zlib(&runs).len() < 2_000,
            "runs compress: {}",
            zlib(&runs).len()
        );
        assert!(
            zlib(&noise).len() < 101_000,
            "noise barely grows: {}",
            zlib(&noise).len()
        );
    }

    #[test]
    fn code_lengths_are_optimal_and_limited() {
        assert_eq!(code_lengths(&[5, 0, 3, 1, 1], 15), [1, 0, 2, 3, 3]);
        assert_eq!(code_lengths(&[0, 7], 15), [0, 1]);
        // Fibonacci counts would need 20 bits unlimited.
        let fibonacci: Vec<u32> = (0..22)
            .scan((1_u32, 1_u32), |state, _| {
                let value: u32 = state.0;
                *state = (state.1, state.0 + state.1);
                Some(value)
            })
            .collect();
        let lengths: Vec<u8> = code_lengths(&fibonacci, 15);
        assert!(lengths.iter().all(|&length| (1..=15).contains(&length)));
        let kraft: f64 = lengths
            .iter()
            .map(|&length| 0.5_f64.powi(i32::from(length)))
            .sum();
        assert!((kraft - 1.0).abs() < 1e-9, "a complete code: {kraft}");
    }

    #[test]
    fn run_lengths_cover_every_kind() {
        let lengths: Vec<u8> = [vec![8; 9], vec![0; 150], vec![5, 0, 0, 0, 0]].concat();
        let runs = run_lengths(&lengths);
        // One 8, six more as a repeat, and the last two as they are: a repeat needs three.
        // Then 138 zeros, 12 zeros, the 5, and four zeros.
        assert_eq!(
            runs,
            [
                (8, 0),
                (16, 3),
                (8, 0),
                (8, 0),
                (18, 127),
                (18, 1),
                (5, 0),
                (17, 1)
            ]
        );
    }

    #[test]
    fn adler32_matches_the_rfc_example() {
        assert_eq!(adler32(b"Wikipedia"), 0x11e6_0398);
    }
}
