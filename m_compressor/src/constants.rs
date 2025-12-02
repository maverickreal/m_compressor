pub const READER_CAPACITY: usize = 1 << 20;

/// [0-255] -> literal (use as is);
/// [256] -> end of block token;
/// [257-285] -> length codes;
/// [286] -> end of stream token
pub const LIT_LEN_ALPHABET_SIZE: usize = 287;
pub const DIST_ALPHABET_SIZE: usize = 30;
pub const END_OF_BLOCK_ID: usize = 256;
pub const END_OF_STREAM_ID: usize = LIT_LEN_ALPHABET_SIZE - 1;

pub const LEN_BASE_CODES: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
pub const LEN_EXTRA_BITS: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
pub const DIST_BASE_CODES: [u16; DIST_ALPHABET_SIZE] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
pub const DIST_EXTRA_BITS: [u8; DIST_ALPHABET_SIZE] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
