use std::{fs::File, io::BufWriter};

pub struct BitWriter {
    writer: BufWriter<File>,
    accumulator: u64,
    bit_count: u8,
}

impl BitWriter {
    pub fn new(out_file: File) -> Self {
        Self {
            writer: BufWriter::new(out_file),
            accumulator: 0,
            bit_count: 0,
        }
    }

    pub fn write(&mut self, token: u16, bit_cnt: u8) {
        self.accumulator = (self.accumulator << bit_cnt) | (token as u64);
        self.bit_count += bit_cnt;

        while self.bit_count >= 8 {
            let shift = self.bit_count - 8;
            let byte = (self.accumulator >> shift) as u8;
            // TODO: handle error properly
            use std::io::Write;
            let _ = self.writer.write(&[byte]);
            self.bit_count -= 8;
            // Mask out the written bits to keep accumulator clean, though not strictly necessary if we shift correctly next time
            // But it's good practice to keep it clean or just rely on the shift logic.
            // With u64 accumulator, we have plenty of space.
            // Let's just keep the lower bits.
            let mask = (1 << self.bit_count) - 1;
            self.accumulator &= mask;
        }
    }

    pub fn flush(&mut self) {
        if self.bit_count > 0 {
            let shift = 8 - self.bit_count;
            let byte = (self.accumulator << shift) as u8;
            use std::io::Write;
            let _ = self.writer.write(&[byte]);
            self.accumulator = 0;
            self.bit_count = 0;
        }
        use std::io::Write;
        let _ = self.writer.flush();
    }
}
