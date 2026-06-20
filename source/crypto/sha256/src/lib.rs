// Copyright Gleb Obitotsky <https://github.com/oxxx1mif> 2026.
//
// License: GNU General Public License v3
// You can find the license file in the project root.
//
// Implementation version 0.1
// The code was written for Realix.
// 19 june 2026

// Thanks to the authors:
// https://datatracker.ietf.org/doc/html/rfc6234

#![no_std]

mod compress;
mod constants;
mod functions;
mod padding;

pub struct Sha256 {
    state: [u32; 8], // h0 - h7
    buffer: [u8; 64],
    buffer_len: u8,
    bit_len: u64
}

