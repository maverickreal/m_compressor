use crate::m_compressor::CompressError;

use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
pub enum LzSymbol {
    Literal(u8),
    Pointer { dist: u16, len: u16 },
}

pub const WINDOW_SIZE: usize = 1 << 15;
pub const MIN_MATCH_SEARCH_SIZE: usize = 3;
pub const MAX_MATCH_SEARCH_SIZE: usize = 258;
pub const READER_CAPACITY: usize = 1 << 27;

fn get_token(window: &VecDeque<u8>, buffer: &Vec<u8>) -> LzSymbol {
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

    return if mx_len < MIN_MATCH_SEARCH_SIZE {
        LzSymbol::Literal(buffer[0])
    } else {
        LzSymbol::Pointer {
            dist: (window.len() - mx_ind) as u16,
            len: mx_len as u16,
        }
    };
}

fn refill_buffer(
    input_stream: &mut BufReader<File>,
    buffer: &mut Vec<u8>,
) -> Result<(), CompressError> {
    let int_buf = input_stream.fill_buf().map_err(|err| {
        eprintln!("Error: {}", err); // e

        return CompressError::StreamReadError(String::from(
            "An error occurred while reading from the input.",
        ));
    })?;

    let req_sz = int_buf.len().min(MAX_MATCH_SEARCH_SIZE - buffer.len());

    buffer.reserve(req_sz);
    buffer.extend(&int_buf[0..req_sz]);
    input_stream.consume(req_sz);

    return Ok(());
}

pub fn process_lz77(input_stream: &mut BufReader<File>) -> Result<Vec<LzSymbol>, CompressError> {
    let mut sym_strm: Vec<LzSymbol> = Vec::new();
    let mut window: VecDeque<u8> = VecDeque::new();
    let mut buffer: Vec<u8> = Vec::new();

    refill_buffer(input_stream, &mut buffer)?;

    while !buffer.is_empty() {
        let token: LzSymbol = get_token(&window, &buffer);

        let sz = if let LzSymbol::Pointer { dist: _, len } = token {
            len as usize
        } else {
            1
        };

        sym_strm.push(token);
        window.extend(buffer.drain(0..sz));

        if window.len() > WINDOW_SIZE {
            window.drain(0..window.len() - WINDOW_SIZE);
        }
        refill_buffer(input_stream, &mut buffer)?;
    }

    return Ok(sym_strm);
}
