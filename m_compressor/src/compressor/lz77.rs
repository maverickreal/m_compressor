/// This is a standalone program that implements LZ77 compression.
use crate::compressor::CompressError;

use std::{collections::VecDeque, hash::Hash};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
pub enum LzSymbol {
    Literal(u16),
    Pointer { dist: u16, len: u16 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_symbol_creation() {
        let symbol = LzSymbol::Literal(65);
        assert_eq!(symbol, LzSymbol::Literal(65));
    }

    #[test]
    fn test_pointer_symbol_creation() {
        let symbol = LzSymbol::Pointer { dist: 10, len: 5 };
        assert_eq!(symbol, LzSymbol::Pointer { dist: 10, len: 5 });
    }

    #[test]
    fn test_process_lz77_empty_input() {
        let mut lz_symbols: VecDeque<LzSymbol> = VecDeque::new();
        let mut window: VecDeque<u8> = VecDeque::new();
        let input: Vec<u8> = vec![];

        let result = process_lz77(&input, &mut lz_symbols, &mut window);
        assert!(result.is_ok());
        assert!(lz_symbols.is_empty());
    }

    #[test]
    fn test_process_lz77_single_byte() {
        let mut lz_symbols: VecDeque<LzSymbol> = VecDeque::new();
        let mut window: VecDeque<u8> = VecDeque::new();
        let input = vec![0x41];

        let result = process_lz77(&input, &mut lz_symbols, &mut window);
        assert!(result.is_ok());
        assert_eq!(lz_symbols.len(), 1);
        assert_eq!(lz_symbols[0], LzSymbol::Literal(0x41));
    }

    #[test]
    fn test_process_lz77_repeating_pattern() {
        let mut lz_symbols: VecDeque<LzSymbol> = VecDeque::new();
        let mut window: VecDeque<u8> = VecDeque::new();
        let input = b"aaaaabbbbb".to_vec();

        let result = process_lz77(&input, &mut lz_symbols, &mut window);
        assert!(result.is_ok());
        assert!(!lz_symbols.is_empty());
    }

    #[test]
    fn test_process_lz77_window_size_limit() {
        let mut lz_symbols: VecDeque<LzSymbol> = VecDeque::new();
        let mut window: VecDeque<u8> = VecDeque::new();
        let input: Vec<u8> = (0..255).cycle().take(50_000).collect();

        let result = process_lz77(&input, &mut lz_symbols, &mut window);
        assert!(result.is_ok());
        assert!(window.len() <= WINDOW_SIZE);
    }

    #[test]
    fn test_get_token_empty_window() {
        let window: VecDeque<u8> = VecDeque::new();
        let buffer: VecDeque<u8> = vec![0x41, 0x42, 0x43].into();

        let token = get_token(&window, &buffer);
        assert_eq!(token, LzSymbol::Literal(0x41));
    }

    #[test]
    fn test_get_token_no_match() {
        let window: VecDeque<u8> = vec![0x41, 0x42, 0x43].into();
        let buffer: VecDeque<u8> = vec![0x44, 0x45, 0x46].into();

        let token = get_token(&window, &buffer);
        assert_eq!(token, LzSymbol::Literal(0x44));
    }

    #[test]
    fn test_constants() {
        assert_eq!(WINDOW_SIZE, 1 << 15);
        assert_eq!(MIN_MATCH_SEARCH_SIZE, 3);
        assert_eq!(MAX_MATCH_SEARCH_SIZE, 258);
    }
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

    Ok(())
}
