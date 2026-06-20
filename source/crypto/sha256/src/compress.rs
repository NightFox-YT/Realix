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

//! Сжатие одного 512‑битного блока
//!
//! `compress_block(state, block)`:
//! - преобразует `block` в 16 big‑endian u32;
//! - расширяет до 64 слов;
//! - выполняет 64 раунда с обновлением рабочих переменных;
//! - добавляет результат к `state`.
//!
//! Все промежуточные данные обнуляются до выхода из функции.

use crate::constants::K;
use crate::functions::*;

pub(crate) fn compress_block(state: &mut [u32; 8], block: &[u8; 64]) {
    let mut w = [0u32; 64];

    // 0..16: прямое преобразование из байтов
    for i in 0..16 {
        let idx = i * 4;
        w[i] = u32::from_be_bytes([block[idx], block[idx+1], block[idx+2], block[idx+3]]);
    }

    // 16..63: расширение
    for i in 16..64 {
        let s0 = small_sigma0(w[i - 15]);
        let s1 = small_sigma1(w[i - 2]);
        w[i] = w[i - 16]
            .wrapping_add(s0)
            .wrapping_add(w[i - 7])
            .wrapping_add(s1);
    }

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    for i in 0..64 {
        let temp1 = h
            .wrapping_add(big_sigma1(e))
            .wrapping_add(ch(e, f, g))
            .wrapping_add(K[i])
            .wrapping_add(w[i]);
        let temp2 = big_sigma0(a).wrapping_add(maj(a, b, c));

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);

    // Немедленное обнуление всех промежуточных слов
    w.fill(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::H0;

    /// После паддинга "abc" (один блок) сжатие должно дать финальный хеш
    #[test]
    fn compress_block_abc() {
        let mut state = H0;
        let mut block = [0u8; 64];
        block[0] = 0x61;
        block[1] = 0x62;
        block[2] = 0x63;
        block[3] = 0x80;
        let len_bytes = (24u64).to_be_bytes();
        block[56..64].copy_from_slice(&len_bytes);

        compress_block(&mut state, &block);

        // SHA-256("abc")
        assert_eq!(state[0], 0xba7816bf);
        assert_eq!(state[1], 0x8f01cfea);
        assert_eq!(state[2], 0x414140de);
        assert_eq!(state[3], 0x5dae2223);
        assert_eq!(state[4], 0xb00361a3);
        assert_eq!(state[5], 0x96177a9c);
        assert_eq!(state[6], 0xb410ff61);
        assert_eq!(state[7], 0xf20015ad);
    }
}