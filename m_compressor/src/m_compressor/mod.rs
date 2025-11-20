/* The central module that implements DEFLATE.
 * Orchestrates the following sequence of operations:
 * 1. Pump the input to the LZ77 transformer.
 * 2. Pump the results from above to the Huffman transformer
 */

use crate::{
    m_compressor::lz77::LzSymbol,
    utils::bit_writer::BitWriter,
};
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

mod huffman;
pub(crate) mod lz77;

/// The actual struct deployed by this library.
#[derive(Debug)]
pub struct MCompressor {
    in_file_path: PathBuf,
    out_file_path: PathBuf,
}

/// All errors throughoutthe library are replaced with these errors.
#[derive(Debug)]
pub enum CompressError {
    FileOpenError,
    StreamReadError(String),
    FileWriteError,
}

impl MCompressor {
    pub fn get_out_file_path(&self) -> &Path {
        &self.out_file_path
    }

    pub fn get_in_file_path(&self) -> &Path {
        &self.in_file_path
    }

    pub fn new(in_file_path: impl AsRef<Path>) -> Self {
        let in_path = in_file_path.as_ref().to_path_buf();
        let mut out_path_os_string = in_path.as_os_str().to_owned();

        out_path_os_string.push(".mc");

        Self {
            in_file_path: in_path,
            out_file_path: PathBuf::from(out_path_os_string),
        }
    }

    pub fn compress(&self) -> Result<(), CompressError> {
        let in_file = File::open(&self.in_file_path).map_err(|err| -> CompressError {
            eprintln!("Error: {}", err);
            return CompressError::FileOpenError;
        })?;

        let out_file = File::create(&self.out_file_path).map_err(|err| -> CompressError {
            eprintln!("Error: {}", err);
            return CompressError::FileOpenError;
        })?;

        let mut reader = BufReader::with_capacity(lz77::READER_CAPACITY, in_file);
        let bit_writer = BitWriter::new(out_file);
        let mut lz_symbols: VecDeque<LzSymbol> = VecDeque::new();

        loop {
            let is_eof = reader
                .fill_buf()
                .map_err(|err| {
                    eprintln!("Error: {}", err);
                    return CompressError::StreamReadError(err.to_string());
                })?
                .is_empty();

            if is_eof {
                break;
            }

            lz77::process_lz77(&mut reader, &mut lz_symbols)?;
            huffman::process_huffman(&lz_symbols, &bit_writer)?;
        }
        Ok(())
    }
}
