//! Just enough DNS to send every name to the setup page.

use alloc::vec::Vec;
use core::net::Ipv4Addr;

/// Bytes of a DNS header.
const HEADER_BYTES: usize = 12;
/// Record type A and class IN.
const TYPE_A: u16 = 1;
const CLASS_IN: u16 = 1;
/// Seconds a phone may keep the answer: short, so it forgets once setup is over.
const TTL_SECONDS: u32 = 10;

/// The answer to a DNS `query`: every name is `address` (type A), and every other type has no
/// records. Returns `None` for anything that is not one standard query with one question.
///
/// ```
/// use core::net::Ipv4Addr;
/// use panel_setup::dns;
///
/// let mut query: Vec<u8> = vec![0x12, 0x34, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0];
/// query.extend_from_slice(b"\x07example\x03com\x00\x00\x01\x00\x01");
/// let answer: Vec<u8> = dns::captive_answer(&query, Ipv4Addr::new(192, 168, 4, 1)).expect("an answer");
/// assert_eq!(&answer[answer.len() - 4..], &[192, 168, 4, 1]);
/// ```
#[must_use]
pub fn captive_answer(query: &[u8], address: Ipv4Addr) -> Option<Vec<u8>> {
    let header: &[u8] = query.get(..HEADER_BYTES)?;
    let is_query: bool = header[2] & 0xf8 == 0;
    let one_question: bool = header[4..6] == [0, 1];
    if !is_query || !one_question {
        return None;
    }
    // The question's name: labels until a zero byte, without compression.
    let mut at: usize = HEADER_BYTES;
    loop {
        let length: usize = usize::from(*query.get(at)?);
        if length == 0 {
            break;
        }
        if length > 63 {
            return None;
        }
        at += 1 + length;
    }
    let question_end: usize = at + 5;
    let question: &[u8] = query.get(HEADER_BYTES..question_end)?;
    let kind: u16 = u16::from_be_bytes([query[at + 1], query[at + 2]]);
    let class: u16 = u16::from_be_bytes([query[at + 3], query[at + 4]]);
    let answers: u8 = u8::from(kind == TYPE_A && class == CLASS_IN);

    let mut answer: Vec<u8> = Vec::with_capacity(question_end + 16);
    answer.extend_from_slice(&header[..2]);
    // A response, authoritative, recursion desired as asked and available; no error.
    answer.extend_from_slice(&[
        0x84 | (header[2] & 0x01),
        0x80,
        0,
        1,
        0,
        answers,
        0,
        0,
        0,
        0,
    ]);
    answer.extend_from_slice(question);
    if answers == 1 {
        // The name as a pointer to the question's.
        answer.extend_from_slice(&[0xc0, HEADER_BYTES as u8]);
        answer.extend_from_slice(&TYPE_A.to_be_bytes());
        answer.extend_from_slice(&CLASS_IN.to_be_bytes());
        answer.extend_from_slice(&TTL_SECONDS.to_be_bytes());
        answer.extend_from_slice(&4_u16.to_be_bytes());
        answer.extend_from_slice(&address.octets());
    }
    Some(answer)
}

#[cfg(test)]
mod test {
    use alloc::vec;

    use super::*;

    const FRAME: Ipv4Addr = Ipv4Addr::new(192, 168, 4, 1);

    fn query(kind: u16) -> Vec<u8> {
        let mut query: Vec<u8> = vec![0xab, 0xcd, 0x01, 0x00, 0, 1, 0, 0, 0, 0, 0, 0];
        for label in ["connectivitycheck", "gstatic", "com"] {
            query.push(label.len() as u8);
            query.extend_from_slice(label.as_bytes());
        }
        query.push(0);
        query.extend_from_slice(&kind.to_be_bytes());
        query.extend_from_slice(&CLASS_IN.to_be_bytes());
        query
    }

    #[test]
    fn answers_every_name_with_the_frame() {
        let asked: Vec<u8> = query(TYPE_A);
        let answer: Vec<u8> = captive_answer(&asked, FRAME).expect("an answer");
        assert_eq!(&answer[..2], &[0xab, 0xcd]);
        assert_eq!(answer[2] & 0x80, 0x80);
        assert_eq!(&answer[6..8], &[0, 1]);
        assert_eq!(&answer[HEADER_BYTES..asked.len()], &asked[HEADER_BYTES..]);
        assert_eq!(&answer[answer.len() - 4..], &[192, 168, 4, 1]);

        let aaaa: Vec<u8> = captive_answer(&query(28), FRAME).expect("an answer");
        assert_eq!(&aaaa[6..8], &[0, 0]);
        assert_eq!(aaaa.len(), asked.len());
    }

    #[test]
    fn ignores_what_it_cannot_read() {
        let mut response: Vec<u8> = query(TYPE_A);
        response[2] |= 0x80;
        assert_eq!(captive_answer(&response, FRAME), None);
        let asked: Vec<u8> = query(TYPE_A);
        assert_eq!(captive_answer(&asked[..asked.len() - 1], FRAME), None);
        assert_eq!(captive_answer(&asked[..5], FRAME), None);
        let mut pointer: Vec<u8> = asked;
        pointer[HEADER_BYTES] = 0xc0;
        assert_eq!(captive_answer(&pointer, FRAME), None);
    }
}
