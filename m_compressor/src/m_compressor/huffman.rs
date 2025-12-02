use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
};

use crate::{
    constants,
    m_compressor::{CompressError, lz77::LzSymbol},
    utils::{bit_writer::BitWriter, huffman_tree_node::HuffmanTreeNode},
};

type LitLenCntArr = [usize; constants::LIT_LEN_ALPHABET_SIZE];
type DistCntArr = [usize; constants::DIST_ALPHABET_SIZE];
type CanonicalCodesMapEntry = (u128, u8);

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
    is_last: bool,
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

    if is_last {
        lit_len_cnt[constants::END_OF_STREAM_ID] = 1;
    }
}

/// Creates and returns a tree made of HuffmanTreeNodes.
/// Uses frequencies of the symbols as weights. Higher
/// weighted symbols are assigned relatively shorter codes
/// by building the tree in a fashion which leads to leaves
/// of these symbols being at comparatively smaller paths.
fn get_huffman_tree(arr: &Vec<usize>) -> HuffmanTreeNode {
    let mut heap: BinaryHeap<Reverse<HuffmanTreeNode>> = BinaryHeap::new();

    for i in 0..arr.len() {
        if arr[i] > 0 {
            heap.push(Reverse(HuffmanTreeNode::new_leaf(arr[i], i)));
        }
    }

    while heap.len() > 1 {
        let Reverse(node_1) = heap.pop().unwrap();
        let Reverse(node_2) = heap.pop().unwrap();
        let new_node = HuffmanTreeNode::new_internal(node_1, node_2);

        heap.push(Reverse(new_node));
    }

    if heap.is_empty() {
        return HuffmanTreeNode::new_leaf(0, 0);
    }
    let Reverse(root) = heap.pop().unwrap();

    root
}

fn map_canonical_codes_to_lz_symbols(
    canonical_codes_map: &mut Vec<CanonicalCodesMapEntry>,
    tree_node: HuffmanTreeNode,
    path: u128,
    height: u8,
) {
    if tree_node.is_leaf() {
        canonical_codes_map[tree_node.symbol.unwrap()] = (path, height);
    }

    if tree_node.left.is_some() {
        map_canonical_codes_to_lz_symbols(
            canonical_codes_map,
            *tree_node.left.unwrap(),
            path,
            height + 1,
        );
    }

    if tree_node.right.is_some() {
        let new_path: u128 = path | (1 << height);
        map_canonical_codes_to_lz_symbols(
            canonical_codes_map,
            *tree_node.right.unwrap(),
            new_path,
            height + 1,
        );
    }
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
                let extra_bits = *len as u128 & ((1 << extra_bits_len) - 1);
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
                let extra_bits = *dist as u128 & ((1 << extra_bits_dist) - 1);
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
/// Alternatively, there are startehies, such as,
/// full-static, full-dynamic, etc.
pub fn process_huffman(
    lz_symbols: &mut VecDeque<LzSymbol>,
    bit_writer: &mut BitWriter,
    is_last: bool,
) -> Result<(), CompressError> {
    let mut lit_len_cnt: LitLenCntArr = [0; constants::LIT_LEN_ALPHABET_SIZE];
    let mut dist_cnt: DistCntArr = [0; constants::DIST_ALPHABET_SIZE];
    let mut lit_len_canonical_map: Vec<CanonicalCodesMapEntry> =
        vec![(0, 0); constants::LIT_LEN_ALPHABET_SIZE];
    let mut dist_canonical_map: Vec<CanonicalCodesMapEntry> =
        vec![(0, 0); constants::DIST_ALPHABET_SIZE];

    put_lit_len_dist_freq(&mut lit_len_cnt, &mut dist_cnt, lz_symbols, is_last);

    let lit_len_tree_head_node: HuffmanTreeNode = get_huffman_tree(&lit_len_cnt.to_vec());
    let dist_tree_head_node: HuffmanTreeNode = get_huffman_tree(&dist_cnt.to_vec());

    map_canonical_codes_to_lz_symbols(&mut lit_len_canonical_map, lit_len_tree_head_node, 0, 0);
    map_canonical_codes_to_lz_symbols(&mut dist_canonical_map, dist_tree_head_node, 0, 0);

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

    if is_last {
        let (eos_code, eos_code_len) = lit_len_canonical_map[constants::END_OF_STREAM_ID];

        bit_writer
            .write_bits(eos_code, eos_code_len)
            .map_err(|_| CompressError::FileWrite)?;
    }

    Ok(())
}
