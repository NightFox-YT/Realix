// Copyright Gleb Obitotsky <https://github.com/oxxx1mif> 2026.
//
// License: GNU General Public License v3
// You can find the license file in the project root.
//
// Implementation crypto version 0.1
// The code was written for Realix.
// 30 june 2026

// Thanks to the authors:
// https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf

/// Constants IV₀ ... IV₇ used in the initialization of BLAKE3.
pub(crate) const IV: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
    0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

/// Flags for the compression function.
pub(crate) const CHUNK_START: u32          = 1 << 0;
pub(crate) const CHUNK_END: u32            = 1 << 1;
pub(crate) const PARENT: u32               = 1 << 2;
pub(crate) const ROOT: u32                 = 1 << 3;
pub(crate) const KEYED_HASH: u32           = 1 << 4;
pub(crate) const DERIVE_KEY_CONTEXT: u32   = 1 << 5;
pub(crate) const DERIVE_KEY_MATERIAL: u32  = 1 << 6;

/// Sizes in bytes.
pub(crate) const OUT_LEN: usize = 32;
pub(crate) const KEY_LEN: usize = 32;
pub(crate) const BLOCK_LEN: usize = 64;
pub(crate) const CHUNK_LEN: usize = 1024;

/// Cyclic shift constants for the G-function.
pub(crate) const R1: u32 = 16;
pub(crate) const R2: u32 = 12;
pub(crate) const R3: u32 = 8;
pub(crate) const R4: u32 = 7;

/// Permutation to update the message array between rounds.
const MSG_PERMUTATION: [usize; 16] = [
    2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8,
];

#[inline(always)]
pub(crate) fn permute_msg(m: &mut [u32; 16]) {
    let t = *m;
    for i in 0..16 {
        m[i] = t[MSG_PERMUTATION[i]];
    }
}