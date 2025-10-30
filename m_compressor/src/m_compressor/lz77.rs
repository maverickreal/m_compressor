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

fn replenish_containers(
    input_stream: &mut BufReader<File>,
    buffer: &mut Vec<u8>,
    window: &mut VecDeque<u8>,
) -> Result<(), CompressError> {
    let strm_to_que_transfer = |stream_: &mut BufReader<File>,
                                buf_: &mut VecDeque<u8>,
                                exp_sz: usize|
     -> Result<(), CompressError> {
        let int_buf = stream_.fill_buf().map_err(|err| {
            println!("Error: {}", err); // e

            return CompressError::StreamReadError(String::from(
                "An error occurred while reading from the input.",
            ));
        })?;

        let req_sz = int_buf.len().min(exp_sz - buf_.len());

        buf_.reserve(req_sz);
        buf_.extend(&int_buf[0..req_sz]);
        stream_.consume(req_sz);

        return Ok(());
    };

    let mut buf_cpy = VecDeque::from_iter(buffer.drain(..));
    let req_sz = buf_cpy.len().min(WINDOW_SIZE - window.len());
    window.extend(buf_cpy.drain(0..req_sz));

    strm_to_que_transfer(input_stream, window, WINDOW_SIZE)?;
    strm_to_que_transfer(input_stream, &mut buf_cpy, MAX_MATCH_SEARCH_SIZE)?;

    *buffer = Vec::from_iter(buf_cpy);

    return Ok(());
}

pub fn process_lz77(input_stream: &mut BufReader<File>) -> Result<Vec<LzSymbol>, CompressError> {
    let mut sym_strm: Vec<LzSymbol> = Vec::new();
    let mut window: VecDeque<u8> = VecDeque::new();
    let mut buffer: Vec<u8> = Vec::new();

    replenish_containers(input_stream, &mut buffer, &mut window)?;

    while !buffer.is_empty() {
        let token: LzSymbol = get_token(&window, &buffer);

        let sz = if let LzSymbol::Pointer { dist: _, len } = token {
            len as usize
        } else {
            1
        };

        sym_strm.push(token);
        window.drain(0..sz);
        window.extend(buffer.drain(0..sz));
        replenish_containers(input_stream, &mut buffer, &mut window)?;
    }

    return Ok(sym_strm);
}
