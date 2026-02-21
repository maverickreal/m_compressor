use std::collections::VecDeque;

use crate::{
    compressor::{CompressError, lz77::LzSymbol},
    constants,
    utils::{bit_writer::BitWriter, package_merge::get_limited_code_lengths},
};

type LitLenCntArr = [usize; constants::LIT_LEN_ALPHABET_SIZE];
type DistCntArr = [usize; constants::DIST_ALPHABET_SIZE];
type CanonicalCodesMapEntry = (u128, u8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_code_index() {
        assert_eq!(get_code_index(3, &constants::LEN_BASE_CODES), 0);
        assert_eq!(get_code_index(4, &constants::LEN_BASE_CODES), 1);
        assert_eq!(get_code_index(258, &constants::LEN_BASE_CODES), 28);
    }

    #[test]
    fn test_get_hm_code_for_lz_ptr() {
        let (dist_idx, len_code) = get_hm_code_for_lz_ptr(&5, &10);
        assert_eq!(dist_idx, 4);
        assert_eq!(len_code, 257 + 7);
    }

    #[test]
    fn test_put_lit_len_dist_freq() {
        let mut lit_len_cnt: LitLenCntArr = [0; constants::LIT_LEN_ALPHABET_SIZE];
        let mut dist_cnt: DistCntArr = [0; constants::DIST_ALPHABET_SIZE];
        let mut lz_symbols: VecDeque<LzSymbol> = VecDeque::new();

        lz_symbols.push_back(LzSymbol::Literal(65));
        lz_symbols.push_back(LzSymbol::Pointer { dist: 5, len: 10 });

        put_lit_len_dist_freq(&mut lit_len_cnt, &mut dist_cnt, &lz_symbols);

        assert_eq!(lit_len_cnt[65], 1);
        assert_eq!(lit_len_cnt[constants::END_OF_BLOCK_ID], 1);
        assert_eq!(dist_cnt[4], 1);
    }

    #[test]
    fn test_generate_canonical_codes_empty() {
        let lengths: Vec<u8> = vec![0; 10];
        let codes = generate_canonical_codes(&lengths);
        assert_eq!(codes.len(), 10);
        assert!(codes.iter().all(|(_, len)| *len == 0));
    }

    #[test]
    fn test_generate_canonical_codes_simple() {
        let lengths = vec![2, 2, 1];
        let codes = generate_canonical_codes(&lengths);

        assert_eq!(codes[0].1, 2);
        assert_eq!(codes[1].1, 2);
        assert_eq!(codes[2].1, 1);
    }

    #[test]
    fn test_generate_canonical_codes_with_zeros() {
        let lengths = vec![0, 2, 0, 3, 0];
        let codes = generate_canonical_codes(&lengths);

        assert_eq!(codes[0].1, 0);
        assert_eq!(codes[1].1, 2);
        assert_eq!(codes[2].1, 0);
        assert_eq!(codes[3].1, 3);
        assert_eq!(codes[4].1, 0);
    }
}

fn get_code_index(val: u16, base_codes: &[u16]) -> u16 {
    for (i, &base) in base_codes.iter().enumerate().rev() {
        if val >= base {
            return i as u16;
        }
    }
    0
}

fn get_hm_code_for_lz_ptr(dist: &u16, len: &u16) -> (u16, u16) {
    let dist_idx = get_code_index(*dist, &constants::DIST_BASE_CODES);
    let len_idx = get_code_index(*len, &constants::LEN_BASE_CODES);

    (dist_idx, 257 + len_idx)
}

/// Populates the frequencies of
/// the current block's alphabet.
fn put_lit_len_dist_freq(
    lit_len_cnt: &mut LitLenCntArr,
    dist_cnt: &mut DistCntArr,
    lz_symbols: &VecDeque<LzSymbol>,
) {
    for symbol in lz_symbols {
        match symbol {
            LzSymbol::Literal(lit) => lit_len_cnt[*lit as usize] += 1,

            LzSymbol::Pointer { dist, len } => {
                let (dist_code, len_code) = get_hm_code_for_lz_ptr(dist, len);
                lit_len_cnt[len_code as usize] += 1;
                dist_cnt[dist_code as usize] += 1;
            }
        }
    }

    // Every block ends with the END_OF_BLOCK_ID (256).
    lit_len_cnt[constants::END_OF_BLOCK_ID] = 1;
}

fn generate_canonical_codes(lengths: &Vec<u8>) -> Vec<CanonicalCodesMapEntry> {
    let mut symbols_with_lengths: Vec<(usize, u8)> = lengths
        .iter()
        .enumerate()
        .filter(|&(_, &len)| len > 0)
        .map(|(i, &len)| (i, len))
        .collect();

    if symbols_with_lengths.is_empty() {
        return vec![(0, 0); lengths.len()];
    }

    symbols_with_lengths.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

    let mut canonical_codes = vec![(0u128, 0u8); lengths.len()];
    let mut current_code = 0u128;
    let mut current_len = symbols_with_lengths[0].1;

    for (symbol, len) in symbols_with_lengths {
        if len > current_len {
            current_code <<= len - current_len;
            current_len = len;
        }

        // Bit-reverse the code because BitWriter writes LSB-first,
        // but Huffman codes must be written MSB-first.
        let mut rev_code = 0u128;
        for i in 0..len {
            if (current_code >> (len - 1 - i)) & 1 == 1 {
                rev_code |= 1 << i;
            }
        }

        canonical_codes[symbol] = (rev_code, len);
        current_code += 1;
    }

    canonical_codes
}

/// Obtains the extra bits-encoded canonical codes
/// corresponding to the LzSymbols, and then writes
/// them to the bit stream using the provided Huffman maps.
fn write_to_stream(
    symbol: &LzSymbol,
    lit_len_canonical_map: &Vec<CanonicalCodesMapEntry>,
    dist_canonical_map: &Vec<CanonicalCodesMapEntry>,
    bit_writer: &mut BitWriter,
) -> Result<(), CompressError> {
    match symbol {
        LzSymbol::Literal(lit) => {
            let (code, code_len) = lit_len_canonical_map[*lit as usize];
            bit_writer
                .write_bits(code, code_len)
                .map_err(|_| CompressError::FileWrite)?;
        }
        LzSymbol::Pointer { dist, len } => {
            let (dist_sym, len_sym) = get_hm_code_for_lz_ptr(dist, len);
            let (len_code, len_code_sz) = lit_len_canonical_map[len_sym as usize];
            let extra_bits_len = constants::LEN_EXTRA_BITS[(len_sym - 257) as usize];

            bit_writer
                .write_bits(len_code, len_code_sz)
                .map_err(|_| CompressError::FileWrite)?;

            if extra_bits_len > 0 {
                let extra_bits =
                    (*len - constants::LEN_BASE_CODES[(len_sym - 257) as usize]) as u128;
                bit_writer
                    .write_bits(extra_bits, extra_bits_len as u8)
                    .map_err(|_| CompressError::FileWrite)?;
            }
            let (dist_code, dist_code_sz) = dist_canonical_map[dist_sym as usize];
            let extra_bits_dist = constants::DIST_EXTRA_BITS[dist_sym as usize];

            bit_writer
                .write_bits(dist_code, dist_code_sz)
                .map_err(|_| CompressError::FileWrite)?;

            if extra_bits_dist > 0 {
                let extra_bits = (*dist - constants::DIST_BASE_CODES[dist_sym as usize]) as u128;
                bit_writer
                    .write_bits(extra_bits, extra_bits_dist)
                    .map_err(|_| CompressError::FileWrite)?;
            }
        }
    }

    Ok(())
}

fn write_header(
    lit_len_canonical_map: &Vec<CanonicalCodesMapEntry>,
    dist_canonical_map: &Vec<CanonicalCodesMapEntry>,
    bit_writer: &mut BitWriter,
) -> Result<(), CompressError> {
    for (_, len) in lit_len_canonical_map {
        bit_writer
            .write_bits(*len as u128, 8)
            .map_err(|_| CompressError::FileWrite)?;
    }

    for (_, len) in dist_canonical_map {
        bit_writer
            .write_bits(*len as u128, 8)
            .map_err(|_| CompressError::FileWrite)?;
    }

    Ok(())
}

/// Produces a bit-stream corresponding to the LzSymbol sequence,
/// and writes it into a file. The first parameter (lz_symbols),
/// represents one block.
///
/// This function implements a semi-dynamic block based 2-pass strategy.
/// Alternatively, there are strategies, such as,
/// full-static, full-dynamic, etc.
pub fn process_huffman(
    lz_symbols: &mut VecDeque<LzSymbol>,
    bit_writer: &mut BitWriter,
    is_last: bool,
) -> Result<(), CompressError> {
    let mut lit_len_cnt: LitLenCntArr = [0; constants::LIT_LEN_ALPHABET_SIZE];
    let mut dist_cnt: DistCntArr = [0; constants::DIST_ALPHABET_SIZE];

    bit_writer
        .write_bits(if is_last { 1 } else { 0 }, 1)
        .map_err(|_| CompressError::FileWrite)?;
    bit_writer
        .write_bits(0b10, 2)
        .map_err(|_| CompressError::FileWrite)?;

    put_lit_len_dist_freq(&mut lit_len_cnt, &mut dist_cnt, lz_symbols);

    let lit_len_lengths = get_limited_code_lengths(&lit_len_cnt, constants::MAX_BIT_LENGTH);
    let dist_lengths = get_limited_code_lengths(&dist_cnt, constants::MAX_BIT_LENGTH);

    let lit_len_canonical_map = generate_canonical_codes(&lit_len_lengths);
    let dist_canonical_map = generate_canonical_codes(&dist_lengths);

    write_header(&lit_len_canonical_map, &dist_canonical_map, bit_writer)?;

    while !lz_symbols.is_empty() {
        let symbol = lz_symbols.pop_front().unwrap();

        write_to_stream(
            &symbol,
            &lit_len_canonical_map,
            &dist_canonical_map,
            bit_writer,
        )?;
    }

    // Every block ends with the END_OF_BLOCK_ID (256) code.
    let (eob_code, eob_code_len) = lit_len_canonical_map[constants::END_OF_BLOCK_ID];
    bit_writer
        .write_bits(eob_code, eob_code_len)
        .map_err(|_| CompressError::FileWrite)?;

    Ok(())
}
