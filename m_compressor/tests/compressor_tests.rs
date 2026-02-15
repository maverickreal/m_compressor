use m_compressor::compressor::MCompressor;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_compression_basic() {
    let input_content = b"abcabcabcabc";
    let in_file_path = PathBuf::from("test_input.txt");
    fs::write(&in_file_path, input_content).unwrap();

    let compressor = MCompressor::new(&in_file_path);
    compressor.compress().expect("Compression failed");

    let out_file_path = compressor.get_out_file_path();
    assert!(out_file_path.exists());

    // Clean up
    let _ = fs::remove_file(&in_file_path);
    let _ = fs::remove_file(out_file_path);
}

#[test]
fn test_compression_empty() {
    let in_file_path = PathBuf::from("test_empty.txt");
    fs::write(&in_file_path, b"").unwrap();

    let compressor = MCompressor::new(&in_file_path);
    compressor.compress().expect("Compression failed");

    let out_file_path = compressor.get_out_file_path();
    assert!(out_file_path.exists());

    let _ = fs::remove_file(&in_file_path);
    let _ = fs::remove_file(out_file_path);
}
