use std::{
    collections::VecDeque,
    fs::File,
    io::{BufReader, Read},
};

pub enum LzError {}

pub enum LzSymbol {
    Literal(u8),
    Pointer { dist: u16, len: u16 },
}

pub const WINDOW_SIZE: usize = 32768;

pub fn process_lz77(input_stream: &mut BufReader<File>) -> Result<Vec<LzSymbol>, LzError> {
    let sym_strm: Vec<LzSymbol> = Vec::new();
    let window: VecDeque<LzSymbol> = VecDeque::new();
    let mut mid_wind_buf: Vec<u8> = vec![0; WINDOW_SIZE];

    while let Ok(_) = input_stream.read_exact(&mut mid_wind_buf) {}

    return Ok(sym_strm);
}
