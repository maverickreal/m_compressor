use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

mod lz77;

pub struct MCompressor {
    in_file_path: PathBuf,
    out_file_path: PathBuf,
    reader: BufReader<std::fs::File>,
}

pub enum CompressError {
    FileOpenError,
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
        let file: File = File::open(&in_path).map_err(|_| CompressError::FileOpenError)?;
        let mut out_path_os_string = in_path.as_os_str().to_owned();

        out_path_os_string.push(".mc");

        Ok(Self {
            in_file_path: in_path,
            out_file_path: PathBuf::from(out_path_os_string),
            reader: BufReader::with_capacity(lz77::WINDOW_SIZE, file),
        })
    }

    pub fn compress(&self) -> Option<CompressError> {
        let symbols: Vec<u8> = lz77::process_lz77(&self.reader)?;

        return None;
    }
}
