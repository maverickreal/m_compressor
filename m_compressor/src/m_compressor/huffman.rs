use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
};

use crate::{
    constants,
    m_compressor::{CompressError, lz77::LzSymbol},
    utils::{bit_writer::BitWriter, huffman_tree_node::HuffmanTreeNode},
};

type LitLenCntArr = [usize; 286];
type DistCntArr = [usize; 30];
type CanonicalCodesMapEntry = (u128, u8);

fn get_len_dist_code_util(len_dist: u16, codes: &Vec<u16>, ind: usize) -> u16 {
    let mut in_range: bool = codes[ind] == len_dist;
    in_range = in_range || (ind + 1 == codes.len());
    in_range = in_range || (codes[ind + 1] > len_dist);

    return if in_range { codes[ind] } else { 0 };
}

fn get_hm_code_for_lz_ptr(dist: &u16, len: &u16) -> (u16, u16) {
    let mut len_code = 0;
    let mut dist_code = 0;
    let dist_base_codes_vec = constants::DIST_BASE_CODES.to_vec();
    let len_base_codes_vec = constants::LEN_BASE_CODES.to_vec();
    let sz_len = dist_base_codes_vec.len();
    let sz_dist = len_base_codes_vec.len();

    for i in 0..sz_len.max(sz_dist) {
        if len_code > 0 && dist_code > 0 {
            break;
        }

        if i < sz_len {
            len_code = len_code | get_len_dist_code_util(*len, &len_base_codes_vec, i);
        }

        if i < sz_dist {
            dist_code = dist_code | get_len_dist_code_util(*dist, &dist_base_codes_vec, i);
        }
    }

    return (dist_code, len_code);
}

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
    // This signifies the END_OF_BLOCK
    // token in compressed data.
    lit_len_cnt[constants::END_OF_BLOCK_ID] = 1;
}

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
    let Reverse(root) = heap.pop().unwrap();

    return root;
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

/// Produces a bit-stream corresponding to the LzSymbol sequence,
/// and writes it into a file. The first parameter (lz_symbols),
/// represents one block.
///
/// This function implements a semi-dynamic block based 2-pass strategy.
/// Alternatively, there are startehies, such as,
/// full-static, full-dynamic, etc.
pub fn process_huffman(
    lz_symbols: &VecDeque<LzSymbol>,
    bit_writer: &BitWriter,
) -> Result<(), CompressError> {
    let mut lit_len_cnt: LitLenCntArr = [0; 286];
    let mut dist_cnt: DistCntArr = [0; 30];
    let mut lit_len_canonical_map: Vec<CanonicalCodesMapEntry> = Vec::new();
    let mut dist_canonical_map: Vec<CanonicalCodesMapEntry> = Vec::new();

    put_lit_len_dist_freq(&mut lit_len_cnt, &mut dist_cnt, lz_symbols);

    let lit_len_tree_head_node: HuffmanTreeNode = get_huffman_tree(&lit_len_cnt.to_vec());
    let dist_tree_head_node: HuffmanTreeNode = get_huffman_tree(&dist_cnt.to_vec());

    map_canonical_codes_to_lz_symbols(&mut lit_len_canonical_map, lit_len_tree_head_node, 0, 0);
    map_canonical_codes_to_lz_symbols(&mut dist_canonical_map, dist_tree_head_node, 0, 0);

    Ok(())
}
