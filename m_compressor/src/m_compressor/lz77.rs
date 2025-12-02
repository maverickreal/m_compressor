/// This is a standalone program that implements LZ77 compression.
use crate::{constants, m_compressor::CompressError};

use std::{collections::VecDeque, hash::Hash};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
pub enum LzSymbol {
    Literal(u16),
    Pointer { dist: u16, len: u16 },
}

pub const WINDOW_SIZE: usize = 1 << 15;
pub const MIN_MATCH_SEARCH_SIZE: usize = 3;
pub const MAX_MATCH_SEARCH_SIZE: usize = 258;

/// Gets the next token from the window and buffer.
/// If a match of size at least MIN_MATCH_SEARCH_SIZE isn't found,
/// returns a literal. Otherwise returns a pointer.
fn get_token(window: &VecDeque<u8>, buffer: &VecDeque<u8>) -> LzSymbol {
    // TODO: can/must be efficient
    let mut mx_ind = 0;
    let mut mx_len = 0;

    for i in (0..window.len()).rev() {
        if mx_len == buffer.len() {
            break;
        }

        let mut j = i;

        while j < window.len() && (j - i) < buffer.len() && window[j] == buffer[j - i] {
            j += 1;
        }

        if (j - i) > mx_len {
            mx_ind = i;
            mx_len = j - i;
        }
    }

    if mx_len < MIN_MATCH_SEARCH_SIZE {
        LzSymbol::Literal(buffer[0].into())
    } else {
        LzSymbol::Pointer {
            dist: (window.len() - mx_ind) as u16,
            len: mx_len as u16,
        }
    }
}

/// Returns a sequence of LZ77 symbols
/// corresponding to the input stream.
pub fn process_lz77(
    inp_chunks: &[u8],
    out_chunks: &mut VecDeque<LzSymbol>,
    window: &mut VecDeque<u8>,
) -> Result<(), CompressError> {
    let mut buffer: VecDeque<u8> = VecDeque::new();
    let mut inp_str_ptr: usize = 0;

    // Refills the buffer from the input stream.
    let mut refill_buffer = |buffer: &mut VecDeque<u8>| {
        let req_sz = (MAX_MATCH_SEARCH_SIZE - buffer.len()).min(inp_chunks.len() - inp_str_ptr);

        if req_sz == 0 {
            return;
        }
        buffer.reserve(req_sz);
        buffer.extend(&inp_chunks[inp_str_ptr..inp_str_ptr + req_sz]);
        inp_str_ptr += req_sz;
    };

    refill_buffer(&mut buffer);

    while !buffer.is_empty() {
        let token: LzSymbol = get_token(&window, &buffer);

        let sz = if let LzSymbol::Pointer { dist: _, len } = token {
            len as usize
        } else {
            1
        };

        out_chunks.push_back(token);
        window.extend(buffer.drain(0..sz));

        if window.len() > WINDOW_SIZE {
            window.drain(0..window.len() - WINDOW_SIZE);
        }
        refill_buffer(&mut buffer);
    }

    if !out_chunks.is_empty() {
        let eob_id = constants::END_OF_BLOCK_ID as u16;
        let eob_lz_sym = LzSymbol::Literal(eob_id);

        out_chunks.push_back(eob_lz_sym);
    }

    Ok(())
}
