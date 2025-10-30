use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use crate::m_compressor::lz77::LzSymbol;

mod lz77;

pub struct MCompressor {
    in_file_path: PathBuf,
    out_file_path: PathBuf,
    reader: BufReader<std::fs::File>,
}

pub enum CompressError {
    FileOpenError,
    StreamReadError(String),
}

impl MCompressor {
    pub fn get_out_file_path(&self) -> &Path {
        return &self.out_file_path;
    }

    pub fn get_in_file_path(&self) -> &Path {
        return &self.in_file_path;
    }

    pub fn new(in_file_path_str: &str) -> Result<MCompressor, CompressError> {
        let in_path: PathBuf = PathBuf::from(in_file_path_str);

        let file: File = File::open(&in_path).map_err(|err| {
            eprintln!("Error: {}", err);

            return CompressError::FileOpenError;
        })?;
        let mut out_path_os_string = in_path.as_os_str().to_owned();

        out_path_os_string.push(".mc");

        let mut self_obj = Self {
            in_file_path: in_path,
            out_file_path: PathBuf::from(out_path_os_string),
            reader: BufReader::with_capacity(lz77::READER_CAPACITY, file),
        };
        self_obj.compress()?;
        return Ok(self_obj);
    }

    fn compress(&mut self) -> Result<(), CompressError> {
        let symbols: Vec<LzSymbol> = lz77::process_lz77(&mut self.reader)?;
        return Ok(());
    }
}
