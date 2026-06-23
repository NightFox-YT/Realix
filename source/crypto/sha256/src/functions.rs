// Copyright Gleb Obitotsky <https://github.com/oxxx1mif> 2026.
//
// License: GNU General Public License v3
// You can find the license file in the project root.
//
// Implementation crypto version 0.1
// The code was written for Realix.
// 19 june 2026

// Thanks to the authors:
// https://datatracker.ietf.org/doc/html/rfc6234

//! Логические функции SHA‑256
//!
//! - `ch(x, y, z)`   – Choose.
//! - `maj(x, y, z)`  – Majority.
//! - `big_sigma0(x)` – Σ₀.
//! - `big_sigma1(x)` – Σ₁.
//! - `small_sigma0(x)` – σ₀.
//! - `small_sigma1(x)` – σ₁.


#[inline(always)]
pub(crate) const fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

#[inline(always)]
pub(crate) const fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

#[inline(always)]
pub(crate) const fn big_sigma0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

#[inline(always)]
pub(crate) const fn big_sigma1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

#[inline(always)]
pub(crate) const fn small_sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

#[inline(always)]
pub(crate) const fn small_sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let vals = [0u32, 0xFFFFFFFF, 0x6a09e667, 0xbb67ae85];
        for &x in &vals {
            for &y in &vals {
                for &z in &vals {
                    let _ = ch(x, y, z);
                    let _ = maj(x, y, z);
                }
                let _ = big_sigma0(x);
                let _ = big_sigma1(x);
                let _ = small_sigma0(x);
                let _ = small_sigma1(x);
            }
        }
    }
}