/*
 * This module implements a bit level writer.
 * Akin to how BufWriter internally buffers the written items,
 * BitWriter accumulates bits from multiple calls.
 * As soon as the accumulation results to a byte,
 * the byte is reset after being flushed to the BufWriter.
 */

use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

static BIT_COUNT_LIMIT: u8 = 8;
static BUFFER_CAPACITY: usize = 64 * 1024;

#[derive(Debug)]
pub struct BitWriter {
    writer: BufWriter<File>,
    buffer: u8,
    bit_count: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_file(suffix: &str) -> File {
        File::create(format!("test_bit_writer_{}.bin", suffix)).unwrap()
    }

    fn read_test_file(suffix: &str) -> Vec<u8> {
        std::fs::read(format!("test_bit_writer_{}.bin", suffix)).unwrap()
    }

    fn cleanup_test_file(suffix: &str) {
        let _ = std::fs::remove_file(format!("test_bit_writer_{}.bin", suffix));
    }

    #[test]
    fn test_bit_writer_constants() {
        assert_eq!(BIT_COUNT_LIMIT, 8);
        assert_eq!(BUFFER_CAPACITY, 64 * 1024);
    }

    #[test]
    fn test_bit_writer_write_single_bit() {
        let file = create_test_file("single_bit");
        let mut writer = BitWriter::new(file);

        writer.write_bits(1, 1).unwrap();
        writer.flush_all().unwrap();

        let bytes = read_test_file("single_bit");
        assert_eq!(bytes[0], 1);

        cleanup_test_file("single_bit");
    }

    #[test]
    fn test_bit_writer_write_byte() {
        let file = create_test_file("byte");
        let mut writer = BitWriter::new(file);

        writer.write_bits(0b10101010, 8).unwrap();
        writer.flush_all().unwrap();

        let bytes = read_test_file("byte");
        assert_eq!(bytes[0], 0b10101010);

        cleanup_test_file("byte");
    }

    #[test]
    fn test_bit_writer_write_partial_bytes() {
        let file = create_test_file("partial");
        let mut writer = BitWriter::new(file);

        writer.write_bits(0b101, 3).unwrap();
        writer.write_bits(0b01010, 5).unwrap();
        writer.flush_all().unwrap();

        let bytes = read_test_file("partial");
        assert_eq!(bytes[0], 0b01010101);

        cleanup_test_file("partial");
    }

    #[test]
    fn test_bit_writer_write_multiple_bytes() {
        let file = create_test_file("multiple");
        let mut writer = BitWriter::new(file);

        writer.write_bits(0xFF, 8).unwrap();
        writer.write_bits(0x00, 8).unwrap();
        writer.write_bits(0xAA, 8).unwrap();
        writer.flush_all().unwrap();

        let bytes = read_test_file("multiple");
        assert_eq!(bytes, vec![0xFF, 0x00, 0xAA]);

        cleanup_test_file("multiple");
    }

    #[test]
    fn test_bit_writer_write_large_value() {
        let file = create_test_file("large");
        let mut writer = BitWriter::new(file);

        writer.write_bits(0x123456789ABCDEF0, 64).unwrap();
        writer.flush_all().unwrap();

        let bytes = read_test_file("large");
        assert_eq!(bytes, vec![0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12]);

        cleanup_test_file("large");
    }

    #[test]
    fn test_bit_writer_flush_without_full_byte() {
        let file = create_test_file("flush");
        let mut writer = BitWriter::new(file);

        writer.write_bits(0b101, 3).unwrap();
        writer.flush_all().unwrap();

        let bytes = read_test_file("flush");
        assert_eq!(bytes[0], 0b101);

        cleanup_test_file("flush");
    }

    #[test]
    fn test_bit_writer_lsb_first() {
        let file = create_test_file("lsb");
        let mut writer = BitWriter::new(file);

        writer.write_bits(1, 1).unwrap();
        writer.write_bits(1, 1).unwrap();
        writer.write_bits(1, 1).unwrap();
        writer.flush_all().unwrap();

        let bytes = read_test_file("lsb");
        assert_eq!(bytes[0], 0b111);

        cleanup_test_file("lsb");
    }
}

impl BitWriter {
    pub fn new(file: File) -> Self {
        Self {
            writer: BufWriter::with_capacity(BUFFER_CAPACITY, file),
            buffer: 0,
            bit_count: 0,
        }
    }

    /// Resets the accumulated byte,
    /// after transferring its bits to the BufWriter.
    fn flush_to_writer(&mut self) -> io::Result<()> {
        self.writer.write_all(&[self.buffer])?;
        self.buffer = 0;
        self.bit_count = 0;

        Ok(())
    }

    /// Cycles through a procedure of writing a number of bits
    /// to the accumulator byte, and flushing the accumulator
    /// byte to the BufWriter as soon as it has accumulated 8 bits.
    /// Uses LSB-first bit ordering (standard DEFLATE).
    pub fn write_bits(&mut self, value: u128, num_bits: u8) -> io::Result<()> {
        for i in 0..num_bits {
            let is_bit_set = ((value >> i) & 1) == 1;

            if is_bit_set {
                let bit_mask = 1 << self.bit_count;
                self.buffer |= bit_mask
            }
            self.bit_count += 1;

            if self.bit_count == BIT_COUNT_LIMIT {
                self.flush_to_writer()?;
            }
        }

        Ok(())
    }

    /// Flushes the accumulator byte
    /// to the BufWriter and then the latter.
    pub fn flush_all(&mut self) -> io::Result<()> {
        if self.bit_count > 0 {
            self.flush_to_writer()?;
        }
        self.writer.flush()?;

        Ok(())
    }
}

impl Drop for BitWriter {
    fn drop(&mut self) {
        let _ = self.flush_all();
    }
}
