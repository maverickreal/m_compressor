/// The central module that implements DEFLATE.
/// Orchestrates the following sequence of operations:
/// 1. Pump the input to the LZ77 transformer.
/// 2. Pump the results from above to the Huffman transformer
/// 3. Put the results from above in a file.
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::m_compressor::lz77::LzSymbol;

mod huffman;
mod lz77;

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

    /// Transforms LZ77 symbols to Huffman symbols.
    fn lz_to_hm_transformer(
        lz_symbols: &mut VecDeque<LzSymbol>,
        hm_symbols: &mut Vec<u8>,
    ) -> Result<(), CompressError> {
        Ok(())
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
        let mut writer = BufWriter::new(out_file);
        let mut lz_symbols: VecDeque<LzSymbol> = VecDeque::new();
        let mut hm_symbols: Vec<u8> = Vec::new();

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
            self.lz_to_hm_transformer(&mut lz_symbols, &mut hm_symbols);
            huffman::process_huffman(&lz_symbols, &mut hm_symbols)?;

            writer.write_all(&hm_symbols).map_err(|err| {
                eprintln!("Error: {}", err);
                return CompressError::FileWriteError;
            })?;

            hm_symbols.clear();
        }
        Ok(())
    }
}
