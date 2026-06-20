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

#![cfg_attr(not(any(test, feature = "std")), no_std)]
// #![cfg_attr(not(test), no_std)]

//! # SHA-256 (RFC 6234)
//! 
//! Пример: 
//! ```
//! use sha256::Sha256;
//!
//! let mut hasher = Sha256::new();
//! hasher.update(b"hello world");
//! let digest = hasher.finalize();
//! assert_eq!(digest.len(), 32);
//! ```
//!
//! Можно обновлять состояние многократно:
//! ```
//! use sha256::Sha256;
//! 
//! let mut hasher = Sha256::new();
//! hasher.update(b"hello ");
//! hasher.update(b"world");
//! let digest = hasher.finalize();
//! ```
//!
//! После вызова `finalize()` объект остаётся доступным, но все внутренние
//! данные обнулены, и дальнейшие вызовы `update` игнорируются.
//!
//! ## Использование из C (FFI)
//!
//! Экспортируются три функции с C‑ABI:
//!
//! - `sha256_init(ctx)`
//! - `sha256_update(ctx, data, len)`
//! - `sha256_finalize(ctx, out_digest)`
//!
//! Пример на C:
//! ```c
//! #include <stdint.h>
//! #include <string.h>
//!
//! struct Sha256;
//!
//! extern void sha256_init(struct Sha256 *ctx);
//! extern void sha256_update(struct Sha256 *ctx, const uint8_t *data, size_t len);
//! extern void sha256_finalize(struct Sha256 *ctx, uint8_t out[32]);
//!
//! void compute() {
//!     struct Sha256 ctx;
//!     sha256_init(&ctx);
//!     sha256_update(&ctx, (const uint8_t*)"abc", 3);
//!     uint8_t digest[32];
//!     sha256_finalize(&ctx, digest);
//! }
//! ```
//!
//! **Важно:** после `sha256_finalize` память `ctx` полностью обнуляется,
//! но остаётся безопасной для повторного вызова `sha256_init`.
//!
//! - Внутренние буферы и состояние обнуляются при финализации и дропе (`Drop`),
//!   исключая остаточные данные в стеке.
//! - Все промежуточные массивы в `compress_block` очищаются перед возвратом.
//! - FFI-функции не разрушают переданный указатель и проверяют его на null.
//! - Обработка сообщения не содержит условных переходов, зависящих от
//!   секретных данных, что предотвращает утечки по времени.


mod compress;
mod constants;
mod functions;
mod padding;

#[repr(C)]
pub struct Sha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    buffer_len: u8,
    bit_len: u64,
    finalized: bool,
}

impl Sha256 {
    pub const fn new() -> Self {
        Self {
            state: constants::H0,
            buffer: [0u8; 64],
            buffer_len: 0,
            bit_len: 0,
            finalized: false,
        }
    }

    /// Подаёт на вход очередную порцию данных.
    ///
    /// Если после вызова `finalize()` метод вызван снова, он
    /// молча игнорируется
    pub fn update(&mut self, data: &[u8]) {
        if self.finalized {
            return;
        }

        let mut offset = 0;
        while offset < data.len() {
            let remaining = data.len() - offset;
            let free = 64 - self.buffer_len as usize;

            if remaining < free {
                let end = offset + remaining;
                self.buffer[self.buffer_len as usize..self.buffer_len as usize + remaining]
                    .copy_from_slice(&data[offset..end]);
                self.buffer_len += remaining as u8;
                self.bit_len = self.bit_len.wrapping_add((remaining * 8) as u64);
                return;
            }

            let take = free;
            self.buffer[self.buffer_len as usize..]
                .copy_from_slice(&data[offset..offset + take]);
            offset += take;
            self.bit_len = self.bit_len.wrapping_add((take * 8) as u64);
            self.buffer_len = 0;

            compress::compress_block(&mut self.state, &self.buffer);
        }
    }

    /// Завершает вычисление и возвращает 32‑байтовый хеш.
    ///
    /// После вызова внутреннее состояние обнуляется. Повторные вызовы
    /// безопасны, но будут возвращать нулевой хеш.
    pub fn finalize(&mut self) -> [u8; 32] {
        if self.finalized {
            return [0u8; 32];
        }
        self.finalized = true;

        let mut block = self.buffer;
        let buffer_len = self.buffer_len as usize;

        let need_second = padding::pad(&mut block, buffer_len, self.bit_len);
        compress::compress_block(&mut self.state, &block);

        if need_second {
            padding::pad_second_block(&mut block, self.bit_len);
            compress::compress_block(&mut self.state, &block);
        }

        let mut digest = [0u8; 32];
        for (i, word) in self.state.iter().enumerate() {
            digest[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
        }

        block.fill(0);
        self.state.fill(0);
        self.buffer.fill(0);
        self.bit_len = 0;

        digest
    }
}

impl Drop for Sha256 {
    fn drop(&mut self) {
        unsafe {
            core::ptr::write_volatile(&mut self.state, [0u32; 8]);
            core::ptr::write_volatile(&mut self.buffer, [0u8; 64]);
        }
        self.bit_len = 0;
    }
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

// ------------------------------------------------------------------
// FFI для C
// ------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn sha256_init(ctx: *mut Sha256) {
    if !ctx.is_null() {
        unsafe { core::ptr::write(ctx, Sha256::new()); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sha256_update(ctx: *mut Sha256, data: *const u8, len: usize) {
    if !ctx.is_null() && !data.is_null() && len > 0 {
        unsafe {
            let slice = core::slice::from_raw_parts(data, len);
            (*ctx).update(slice);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sha256_finalize(ctx: *mut Sha256, out_digest: *mut u8) {
    if !ctx.is_null() && !out_digest.is_null() {
        unsafe {
            let digest = (*ctx).finalize();
            core::ptr::copy_nonoverlapping(digest.as_ptr(), out_digest, 32);
        }
    }
}

/*
// ------------------------------------------------------------------
// Панический обработчик (только standalone, не для тестов)
// ------------------------------------------------------------------
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
*/

#[cfg(not(any(test, feature = "std")))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }

// ------------------------------------------------------------------
// Тесты
// ------------------------------------------------------------------

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        let mut h = Sha256::new();
        h.update(b"");
        let digest = h.finalize();
        assert_eq!(digest, [
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14,
            0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
            0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c,
            0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
        ]);
    }

    #[test]
    fn abc() {
        let mut h = Sha256::new();
        h.update(b"abc");
        let digest = h.finalize();
        assert_eq!(digest, [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea,
            0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
            0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c,
            0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
        ]);
    }

    #[test]
    fn long_string() {
        let input = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        let mut h = Sha256::new();
        h.update(input);
        let digest = h.finalize();
        assert_eq!(digest, [
            0x24, 0x8d, 0x6a, 0x61, 0xd2, 0x06, 0x38, 0xb8,
            0xe5, 0xc0, 0x26, 0x93, 0x0c, 0x3e, 0x60, 0x39,
            0xa3, 0x3c, 0xe4, 0x59, 0x64, 0xff, 0x21, 0x67,
            0xf6, 0xec, 0xed, 0xd4, 0x19, 0xdb, 0x06, 0xc1,
        ]);
    }

    #[test]
    fn multiple_updates() {
        let mut h = Sha256::new();
        h.update(b"abcdbcdecdefdefgefghfghighijhijk");
        h.update(b"ijkljklmklmnlmnomnopnopq");
        let digest = h.finalize();
        assert_eq!(digest, [
            0x24, 0x8d, 0x6a, 0x61, 0xd2, 0x06, 0x38, 0xb8,
            0xe5, 0xc0, 0x26, 0x93, 0x0c, 0x3e, 0x60, 0x39,
            0xa3, 0x3c, 0xe4, 0x59, 0x64, 0xff, 0x21, 0x67,
            0xf6, 0xec, 0xed, 0xd4, 0x19, 0xdb, 0x06, 0xc1,
        ]);
    }

    #[test]
    fn double_finalize_safe() {
        let mut h = Sha256::new();
        h.update(b"abc");
        let _ = h.finalize();
        let digest2 = h.finalize();
        assert_eq!(digest2, [0u8; 32]);
    }
}
