// Copyright Gleb Obitotsky <https://github.com/oxxx1mif> 2026.
//
// License: GNU General Public License v3
// You can find the license file in the project root.
//
// Implementation version 0.1
// The code was written for Realix.
// 19 june 2026

// [!!!! WARNING] This code cannot yet be used in the kernel or other tasks. It is in an untested and unstable version.

//! Аппаратное сжатие блока SHA‑256 через Intel SHA Extensions (x86_64).
//!
//! # Требования безопасности для ядра
//! Перед вызовом необходимо сохранить XMM‑регистры текущего процесса
//! через `kernel_fpu_begin()` и восстановить через `kernel_fpu_end()`.

use core::arch::x86_64::*;
use core::sync::atomic;

/// Маска для переворота байт в 128-битном векторе.
/// Каждое 32‑битное слово разворачивается независимо.
const BSWAP_MASK: __m128i = unsafe {
    _mm_set_epi8(
        12, 13, 14, 15,  // 3
        8, 9, 10, 11,    // 2
        4, 5, 6, 7,      // 1
        0, 1, 2, 3,      // 0
    )
};

/// Аппаратное сжатие одного 64‑байтного блока.
#[inline(always)]
pub(crate) unsafe fn hw_compress_block(state: &mut [u32; 8], block: &[u8; 64]) {
    // Загружаем начальное состояние (a..h).
    // Инструкции SHA Extensions ожидают порядок слов [A, B, C, D].
    // Мы загружаем в прямом порядке и затем переставляем через _mm_shuffle_epi32(0x1B),
    // чтобы получить правильное расположение байт внутри вектора.
    let mut abcd = _mm_set_epi32(
        state[0] as i32,
        state[1] as i32,
        state[2] as i32,
        state[3] as i32,
    );
    let mut efgh = _mm_set_epi32(
        state[4] as i32,
        state[5] as i32,
        state[6] as i32,
        state[7] as i32,
    );
    abcd = _mm_shuffle_epi32(abcd, 0x1B);
    efgh = _mm_shuffle_epi32(efgh, 0x1B);

    // Загружаем блок сообщения и преобразуем каждое слово в Big‑Endian
    let mut msg0 = _mm_loadu_si128(block.as_ptr() as *const __m128i);
    let mut msg1 = _mm_loadu_si128(block.as_ptr().add(16) as *const __m128i);
    let mut msg2 = _mm_loadu_si128(block.as_ptr().add(32) as *const __m128i);
    let mut msg3 = _mm_loadu_si128(block.as_ptr().add(48) as *const __m128i);

    msg0 = _mm_shuffle_epi8(msg0, BSWAP_MASK);
    msg1 = _mm_shuffle_epi8(msg1, BSWAP_MASK);
    msg2 = _mm_shuffle_epi8(msg2, BSWAP_MASK);
    msg3 = _mm_shuffle_epi8(msg3, BSWAP_MASK);

    // Раундовые константы K,
    // упакованные в векторы в порядке, обратном интуитивному (как требует _mm_set_epi32).
    let k00 = _mm_set_epi32(0xe9b5dba5, 0xb5c0fbcf, 0x71374491, 0x428a2f98);
    let k01 = _mm_set_epi32(0xab1c5ed5, 0x923f82a4, 0x59f111f1, 0x3956c25b);
    let k02 = _mm_set_epi32(0x550c7dc3, 0x243185be, 0x12835b01, 0xd807aa98);
    let k03 = _mm_set_epi32(0xc19bf174, 0x9bdc06a7, 0x80deb1fe, 0x72be5d74);
    let k04 = _mm_set_epi32(0x240ca1cc, 0x0fc19dc6, 0xefbe4786, 0xe49b69c1);
    let k05 = _mm_set_epi32(0x76f988da, 0x5cb0a9dc, 0x4a7484aa, 0x2de92c6f);
    let k06 = _mm_set_epi32(0xbf597fc7, 0xb00327c8, 0xa831c66d, 0x983e5152);
    let k07 = _mm_set_epi32(0x14292967, 0x06ca6351, 0xd5a79147, 0xc6e00bf3);
    let k08 = _mm_set_epi32(0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x27b70a85);
    let k09 = _mm_set_epi32(0x766a0abb, 0x650a7354, 0x92722c85, 0x81c2c92e);
    let k10 = _mm_set_epi32(0xa81a664b, 0xa2bfe8a1, 0xc76c51a3, 0xc24b8b70);
    let k11 = _mm_set_epi32(0xd6990624, 0xd192e819, 0x106aa070, 0xf40e3585);
    let k12 = _mm_set_epi32(0x1e376c08, 0x19a4c116, 0x34b0bcb5, 0x2748774c);
    let k13 = _mm_set_epi32(0x4ed8aa4a, 0x391c0cb3, 0x682e6ff3, 0x5b9cca4f);
    let k14 = _mm_set_epi32(0x78a5636f, 0x748f82ee, 0x8cc70208, 0x84c87814);
    let k15 = _mm_set_epi32(0xa4506ceb, 0x90befffa, 0xc67178f2, 0xbef9a3f7);

    // Макрос для выполнения 4 раундов (обновляет abcd и efgh по очереди)
    macro_rules! rounds4 {
        ($abcd:ident, $efgh:ident, $k:ident, $msg:ident) => {{
            let mut w = _mm_add_epi32($msg, $k);
            // Первые 2 раунда: обновляем abcd, efgh используется как старое состояние
            $abcd = _mm_sha256rnds2_epu32($abcd, $efgh, w);
            w = _mm_shuffle_epi32(w, 0x0E);
            // Вторые 2 раунда: обновляем efgh, используем новый abcd
            $efgh = _mm_sha256rnds2_epu32($efgh, $abcd, w);
        }};
    }

    // Макрос для обновления одного вектора сообщения
    macro_rules! schedule {
        ($v0:ident, $v1:ident, $v2:ident, $v3:ident) => {{
            let mut tmp = _mm_sha256msg1_epu32($v0, $v1);
            // align: конец старого $v3 и начало более нового $v2
            let align = _mm_alignr_epi8($v3, $v2, 4);
            tmp = _mm_add_epi32(tmp, align);
            // msg2 всегда получает обновляемый вектор и самый свежий сгенерированный ($v3)
            $v0 = _mm_sha256msg2_epu32(tmp, $v3);
        }};
    }

    // ----------------------------------------------------------------
    // Раунды 0..15 (используются исходные msg0..msg3)
    // ----------------------------------------------------------------
    rounds4!(abcd, efgh, k00, msg0);
    rounds4!(abcd, efgh, k01, msg1);
    rounds4!(abcd, efgh, k02, msg2);
    rounds4!(abcd, efgh, k03, msg3);

    // ----------------------------------------------------------------
    // Раунды 16..31 (генерируются новые слова, msg0 обновляется первым)
    // ----------------------------------------------------------------
    schedule!(msg0, msg1, msg2, msg3);
    rounds4!(abcd, efgh, k04, msg0);
    schedule!(msg1, msg2, msg3, msg0);
    rounds4!(abcd, efgh, k05, msg1);
    schedule!(msg2, msg3, msg0, msg1);
    rounds4!(abcd, efgh, k06, msg2);
    schedule!(msg3, msg0, msg1, msg2);
    rounds4!(abcd, efgh, k07, msg3);

    // ----------------------------------------------------------------
    // Раунды 32..47
    // ----------------------------------------------------------------
    schedule!(msg0, msg1, msg2, msg3);
    rounds4!(abcd, efgh, k08, msg0);
    schedule!(msg1, msg2, msg3, msg0);
    rounds4!(abcd, efgh, k09, msg1);
    schedule!(msg2, msg3, msg0, msg1);
    rounds4!(abcd, efgh, k10, msg2);
    schedule!(msg3, msg0, msg1, msg2);
    rounds4!(abcd, efgh, k11, msg3);

    // ----------------------------------------------------------------
    // Раунды 48..63
    // ----------------------------------------------------------------
    schedule!(msg0, msg1, msg2, msg3);
    rounds4!(abcd, efgh, k12, msg0);
    schedule!(msg1, msg2, msg3, msg0);
    rounds4!(abcd, efgh, k13, msg1);
    schedule!(msg2, msg3, msg0, msg1);
    rounds4!(abcd, efgh, k14, msg2);
    schedule!(msg3, msg0, msg1, msg2);
    rounds4!(abcd, efgh, k15, msg3);

    // Разворачиваем порядок слов обратно перед сохранением
    abcd = _mm_shuffle_epi32(abcd, 0x1B);
    efgh = _mm_shuffle_epi32(efgh, 0x1B);

    let mut out = [0u32; 8];
    _mm_storeu_si128(&mut out[0] as *mut u32 as *mut __m128i, abcd);
    _mm_storeu_si128(&mut out[4] as *mut u32 as *mut __m128i, efgh);

    // Сложение с начальным состоянием
    for i in 0..8 {
        state[i] = state[i].wrapping_add(out[i]);
    }

    // Гарантированная очистка всех XMM‑регистров
    core::arch::asm!(
        "pxor {v0}, {v0}",
        "pxor {v1}, {v1}",
        "pxor {v2}, {v2}",
        "pxor {v3}, {v3}",
        "pxor {v4}, {v4}",
        "pxor {v5}, {v5}",
        "pxor {v6}, {v6}",
        "pxor {v7}, {v7}",
        v0 = out(reg) _mm_setzero_si128(),
        v1 = out(reg) _mm_setzero_si128(),
        v2 = out(reg) _mm_setzero_si128(),
        v3 = out(reg) _mm_setzero_si128(),
        v4 = out(reg) _mm_setzero_si128(),
        v5 = out(reg) _mm_setzero_si128(),
        v6 = out(reg) _mm_setzero_si128(),
        v7 = out(reg) _mm_setzero_si128(),
        options(nostack, nomem)
    );
    atomic::compiler_fence(atomic::Ordering::SeqCst);
}