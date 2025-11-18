use std::collections::VecDeque;

use crate::m_compressor::{CompressError, lz77::LzSymbol};

pub fn process_huffman(
    lz_symbols: &VecDeque<LzSymbol>,
    out_buf: &mut Vec<u8>,
) -> Result<(), CompressError> {
    /* Alternatively, we can use a 2-pass strategy.
     * This current implementation uses dynamic block-based startegy.
     */
    Ok(())
}
