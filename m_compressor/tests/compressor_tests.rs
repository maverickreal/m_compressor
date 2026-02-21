use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

const TEST_DATA_DIR: &str = "test_data";

fn setup_test_file(name: &str, content: &[u8]) -> PathBuf {
    let test_dir = PathBuf::from(TEST_DATA_DIR);
    fs::create_dir_all(&test_dir).unwrap();
    let file_path = test_dir.join(name);
    let mut file = File::create(&file_path).unwrap();
    file.write_all(content).unwrap();
    file_path
}

fn cleanup_test_file(path: &PathBuf) {
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    let compressed_path = PathBuf::from(format!("{}.mc", path.display()));
    if compressed_path.exists() {
        let _ = fs::remove_file(&compressed_path);
    }
}

#[test]
fn test_compress_empty_file() {
    let test_file = setup_test_file("empty.txt", b"");
    let compressor = m_compressor::compressor::MCompressor::new(&test_file);

    let result = compressor.compress();
    assert!(result.is_ok(), "Compression of empty file should succeed");

    let compressed_path = compressor.get_out_file_path();
    assert!(
        compressed_path.exists(),
        "Compressed file should exist at {:?}",
        compressed_path
    );

    let metadata = compressed_path.metadata().unwrap();
    assert!(metadata.len() > 0, "Compressed file should not be empty");

    cleanup_test_file(&test_file);
}

#[test]
fn test_compress_single_byte_file() {
    let test_file = setup_test_file("single.txt", b"A");
    let compressor = m_compressor::compressor::MCompressor::new(&test_file);

    let result = compressor.compress();
    assert!(
        result.is_ok(),
        "Compression of single byte file should succeed"
    );

    let compressed_path = compressor.get_out_file_path();
    assert!(compressed_path.exists());

    cleanup_test_file(&test_file);
}

#[test]
fn test_compress_small_text_file() {
    let content = b"Hello, World!";
    let test_file = setup_test_file("small.txt", content);
    let compressor = m_compressor::compressor::MCompressor::new(&test_file);

    let result = compressor.compress();
    assert!(result.is_ok());

    let compressed_path = compressor.get_out_file_path();
    assert!(compressed_path.exists());

    let _compressed_size = compressed_path.metadata().unwrap().len();
    let _original_size = content.len() as u64;

    cleanup_test_file(&test_file);
}

#[test]
fn test_compress_repeating_pattern() {
    let content: Vec<u8> = b"abcde".repeat(1000);
    let test_file = setup_test_file("repeating.bin", &content);
    let compressor = m_compressor::compressor::MCompressor::new(&test_file);

    let result = compressor.compress();
    assert!(result.is_ok());

    let compressed_path = compressor.get_out_file_path();
    assert!(compressed_path.exists());

    let compressed_size = compressed_path.metadata().unwrap().len();
    let original_size = content.len() as u64;
    assert!(
        compressed_size < original_size,
        "Repeating pattern should compress (got {}, original {})",
        compressed_size,
        original_size
    );

    cleanup_test_file(&test_file);
}

#[test]
fn test_compress_large_file() {
    let content: Vec<u8> = (0..=255).cycle().take(100_000).collect();
    let test_file = setup_test_file("large.bin", &content);
    let compressor = m_compressor::compressor::MCompressor::new(&test_file);

    let result = compressor.compress();
    assert!(result.is_ok());

    let compressed_path = compressor.get_out_file_path();
    assert!(compressed_path.exists());

    cleanup_test_file(&test_file);
}

#[test]
fn test_compress_file_with_all_zeros() {
    let content = vec![0u8; 10_000];
    let test_file = setup_test_file("zeros.bin", &content);
    let compressor = m_compressor::compressor::MCompressor::new(&test_file);

    let result = compressor.compress();
    assert!(result.is_ok());

    let compressed_path = compressor.get_out_file_path();
    assert!(compressed_path.exists());

    let compressed_size = compressed_path.metadata().unwrap().len();
    let original_size = content.len() as u64;
    assert!(
        compressed_size < original_size / 5,
        "Zeros should compress very well (got {}, original {})",
        compressed_size,
        original_size
    );

    cleanup_test_file(&test_file);
}

#[test]
fn test_compress_nonexistent_file() {
    let test_file = PathBuf::from(TEST_DATA_DIR).join("nonexistent.txt");
    let compressor = m_compressor::compressor::MCompressor::new(&test_file);

    let result = compressor.compress();
    assert!(
        result.is_err(),
        "Compression of nonexistent file should fail"
    );
}

#[test]
fn test_output_file_naming() {
    let test_file = setup_test_file("naming.txt", b"test");
    let compressor = m_compressor::compressor::MCompressor::new(&test_file);

    let expected_path = PathBuf::from(format!("{}.mc", test_file.display()));
    let actual_path = compressor.get_out_file_path();
    assert_eq!(
        actual_path, expected_path,
        "Output file should have .mc appended"
    );

    cleanup_test_file(&test_file);
}

#[test]
fn test_compress_english_text() {
    let content = b"The quick brown fox jumps over the lazy dog. ";
    let test_file = setup_test_file("english.txt", content);
    let compressor = m_compressor::compressor::MCompressor::new(&test_file);

    let result = compressor.compress();
    assert!(result.is_ok());

    let compressed_path = compressor.get_out_file_path();
    assert!(compressed_path.exists());

    cleanup_test_file(&test_file);
}

#[test]
fn test_compress_binary_data() {
    let content: Vec<u8> = (0..=255).collect();
    let test_file = setup_test_file("binary.dat", &content);
    let compressor = m_compressor::compressor::MCompressor::new(&test_file);

    let result = compressor.compress();
    assert!(result.is_ok());

    let compressed_path = compressor.get_out_file_path();
    assert!(compressed_path.exists());

    cleanup_test_file(&test_file);
}

#[test]
fn test_compress_file_with_special_characters() {
    let content = "Hello\n\tWorld\r\nTest\x00\x01\x02".as_bytes();
    let test_file = setup_test_file("special.bin", content);
    let compressor = m_compressor::compressor::MCompressor::new(&test_file);

    let result = compressor.compress();
    assert!(result.is_ok());

    let compressed_path = compressor.get_out_file_path();
    assert!(compressed_path.exists());

    cleanup_test_file(&test_file);
}

#[test]
fn test_multiple_compressions_same_file() {
    let test_file = setup_test_file("multi.txt", b"test content");
    let compressor1 = m_compressor::compressor::MCompressor::new(&test_file);
    let compressor2 = m_compressor::compressor::MCompressor::new(&test_file);

    let result1 = compressor1.compress();
    let result2 = compressor2.compress();

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let compressed_path1 = compressor1.get_out_file_path();
    let compressed_path2 = compressor2.get_out_file_path();

    let content1 = fs::read(compressed_path1).unwrap();
    let content2 = fs::read(compressed_path2).unwrap();

    assert_eq!(
        content1, content2,
        "Multiple compressions of same file should produce identical output"
    );

    cleanup_test_file(&test_file);
}
