//! Сырые FFI-привязки к C-библиотеке.

#![allow(unsafe_code)]

use libc::{c_char, c_int, c_void, size_t};

/// Режимы обнуления
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SecureZeroMode {
    Fast = 1,
    Balanced = 2,
    Military = 3,
    Universal = 4,
}

extern "C" {
    /// Основная функция.
    /// Возвращает 0 при успехе, -1 при ошибке.
    pub fn secure_zero_memory(
        ptr: *mut c_void,
        len: size_t,
        mode: SecureZeroMode,
    ) -> c_int;

    #[cfg(feature = "debug")]
    pub fn secure_verify_zero(ptr: *const c_void, len: size_t) -> c_int;

    #[cfg(feature = "debug")]
    pub fn secure_last_error() -> *const c_char;
}

/// Удобные обёртки для прямого вызова
#[inline]
pub unsafe fn secure_zero_memory_fast(ptr: *mut c_void, len: size_t) -> c_int {
    secure_zero_memory(ptr, len, SecureZeroMode::Fast)
}
#[inline]
pub unsafe fn secure_zero_memory_balanced(ptr: *mut c_void, len: size_t) -> c_int {
    secure_zero_memory(ptr, len, SecureZeroMode::Balanced)
}
#[inline]
pub unsafe fn secure_zero_memory_military(ptr: *mut c_void, len: size_t) -> c_int {
    secure_zero_memory(ptr, len, SecureZeroMode::Military)
}
#[inline]
pub unsafe fn secure_zero_memory_universal(ptr: *mut c_void, len: size_t) -> c_int {
    secure_zero_memory(ptr, len, SecureZeroMode::Universal)
}