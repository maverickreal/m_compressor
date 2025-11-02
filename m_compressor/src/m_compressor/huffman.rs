use std::{fs::File, io::BufWriter};

use crate::m_compressor::{CompressError, lz77::LzSymbol};

pub fn process_huffman(
    lz_symols: &Vec<LzSymbol>,
    writer: &mut BufWriter<File>,
) -> Result<(), CompressError> {
    Ok(())
}
