use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

mod lz77;

pub struct MCompressor {
    in_file_path: PathBuf,
    out_file_path: PathBuf,
}

#[derive(Debug)]
pub enum CompressError {
    FileOpenError,
    StreamReadError(String),
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

        let mut reader = BufReader::with_capacity(lz77::READER_CAPACITY, in_file);

        let symbols = lz77::process_lz77(&mut reader)?;
        Ok(())
    }
}
