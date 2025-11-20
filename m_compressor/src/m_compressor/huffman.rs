use std::{cmp::Reverse, collections::{BinaryHeap, VecDeque}};

use crate::{
    constants,
    m_compressor::{CompressError, lz77::LzSymbol},
    utils::{
        bit_writer::BitWriter,
        huffman_tree_node::HuffmanTreeNode,
    },
};

type LitLenCntArr = [usize; 286];
type DistCntArr = [usize; 30];
type HuffmanCode = (u16, u8, u8, u16);

fn get_hm_code_for_lz_symbol(dist: &u16, len: &u16) -> HuffmanCode {
    todo!()
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
                let (dist_code, _, len_code, _) = get_hm_code_for_lz_symbol(dist, len);
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

    put_lit_len_dist_freq(&mut lit_len_cnt, &mut dist_cnt, lz_symbols);
    let lit_len_tree_head_node: HuffmanTreeNode = get_huffman_tree(&lit_len_cnt.to_vec());
    let dist_tree_head_node: HuffmanTreeNode = get_huffman_tree(&dist_cnt.to_vec());

    Ok(())
}
