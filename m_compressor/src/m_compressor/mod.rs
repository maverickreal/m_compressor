use std::{fs::File, path::Path};

pub struct MCompressor {
    file_path: String,
    compressed_file_path: String,
}

pub enum MCompressError {
    InvalidPath,
    FileNotFound,
}

impl MCompressor {
    fn is_input_file_path_ok(file_path: &String) -> Option<MCompressError> {
        let path: &Path = Path::new(file_path);

        if !path.exists() {
            return Some(MCompressError::InvalidPath);
        }

        if !path.is_file() {
            return Some(MCompressError::FileNotFound);
        }

        return None;
    }

    pub fn get_compressed_file_path(&self) -> &String {
        return &self.compressed_file_path;
    }

    pub fn new(_file_path: String) -> Result<MCompressor, MCompressError> {
        if let Some(err) = Self::is_input_file_path_ok(&_file_path) {
            return Err(err);
        }

        let _compressed_file_path = _file_path.clone() + ".mc";

        let self_obj: MCompressor = MCompressor {
            file_path: _file_path,
            compressed_file_path: _compressed_file_path,
        };

        return Ok(self_obj);
    }
}
