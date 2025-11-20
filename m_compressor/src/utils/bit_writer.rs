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

        return Ok(());
    }

    /// Cycles through a procedure of writing a number of bits
    /// to the accumulator byte, and fliushing the accumulator
    /// byte to the BufWriter as soon as it has accumulated 8 bits.
    pub fn write_bits(&mut self, value: u128, num_bits: u8) -> io::Result<()> {
        for i in (0..num_bits).rev() {
            let is_bit_set = (value >> i) & 1 == 1;

            if is_bit_set {
                let bit_mask = 1 << (BIT_COUNT_LIMIT - 1 - self.bit_count);
                self.buffer |= bit_mask
            }
            self.bit_count += 1;

            if self.bit_count == BIT_COUNT_LIMIT {
                self.flush_to_writer()?;
            }
        }

        return Ok(());
    }

    /// Flushes the accumulator byte
    /// to the BufWriter and then the latter.
    pub fn flush_all(&mut self) -> io::Result<()> {
        if self.bit_count > 0 {
            self.flush_to_writer()?;
        }
        self.writer.flush()?;

        return Ok(());
    }
}

impl Drop for BitWriter {
    fn drop(&mut self) {
        self.flush_all();
    }
}
