//! # Пример
//! ```no_run
//! use secure_memory_rust::{secure_zero_slice_military, SecureZeroMode};
//!
//! let mut key = [0xAA; 32];
//! secure_zero_slice_military(&mut key).unwrap();
//! ```

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

extern crate libc;

mod sys;

use libc::{c_int, c_void};
use core::{ptr, slice};

/// Режимы обнуления
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SecureZeroMode {
    Fast = 1,
    Balanced = 2,
    Military = 3,
    Universal = 4,
}

impl From<SecureZeroMode> for sys::SecureZeroMode {
    fn from(m: SecureZeroMode) -> Self {
        match m {
            SecureZeroMode::Fast => sys::SecureZeroMode::Fast,
            SecureZeroMode::Balanced => sys::SecureZeroMode::Balanced,
            SecureZeroMode::Military => sys::SecureZeroMode::Military,
            SecureZeroMode::Universal => sys::SecureZeroMode::Universal,
        }
    }
}

/// Обнуляет память по указателю и длине.
///
/// # Safety
/// Указатель должен быть валидным и указывать на `len` байт доступной памяти.
#[inline]
pub unsafe fn secure_zero_memory(ptr: *mut u8, len: usize, mode: SecureZeroMode) -> Result<(), i32> {
    let ret = sys::secure_zero_memory(ptr as *mut c_void, len, mode.into());
    if ret == 0 { Ok(()) } else { Err(ret) }
}

/// Обёртки для каждого режима.
#[inline]
pub unsafe fn secure_zero_fast(ptr: *mut u8, len: usize) -> Result<(), i32> {
    secure_zero_memory(ptr, len, SecureZeroMode::Fast)
}
#[inline]
pub unsafe fn secure_zero_balanced(ptr: *mut u8, len: usize) -> Result<(), i32> {
    secure_zero_memory(ptr, len, SecureZeroMode::Balanced)
}
#[inline]
pub unsafe fn secure_zero_military(ptr: *mut u8, len: usize) -> Result<(), i32> {
    secure_zero_memory(ptr, len, SecureZeroMode::Military)
}
#[inline]
pub unsafe fn secure_zero_universal(ptr: *mut u8, len: usize) -> Result<(), i32> {
    secure_zero_memory(ptr, len, SecureZeroMode::Universal)
}

/// Безопасная версия для обнуления изменяемого среза.
pub fn secure_zero_slice<T>(slice: &mut [T], mode: SecureZeroMode) -> Result<(), i32> {
    if slice.is_empty() {
        return Ok(());
    }
    let ptr = slice.as_mut_ptr() as *mut u8;
    let len = slice.len() * core::mem::size_of::<T>();
    unsafe { secure_zero_memory(ptr, len, mode) }
}

#[inline]
pub fn secure_zero_slice_fast<T>(slice: &mut [T]) -> Result<(), i32> {
    secure_zero_slice(slice, SecureZeroMode::Fast)
}
#[inline]
pub fn secure_zero_slice_balanced<T>(slice: &mut [T]) -> Result<(), i32> {
    secure_zero_slice(slice, SecureZeroMode::Balanced)
}
#[inline]
pub fn secure_zero_slice_military<T>(slice: &mut [T]) -> Result<(), i32> {
    secure_zero_slice(slice, SecureZeroMode::Military)
}
#[inline]
pub fn secure_zero_slice_universal<T>(slice: &mut [T]) -> Result<(), i32> {
    secure_zero_slice(slice, SecureZeroMode::Universal)
}

/// Обнуление одного значения по ссылке.
#[inline]
pub fn secure_zero_value<T>(value: &mut T, mode: SecureZeroMode) -> Result<(), i32> {
    let slice = core::slice::from_mut(value);
    secure_zero_slice(slice, mode)
}

// -----------------------------------------------------------------------------
// Отладочные функции
// -----------------------------------------------------------------------------

#[cfg(feature = "debug")]
pub mod debug {
    use core::ffi::CStr;
    use libc::c_void;

    /// Проверяет, что область памяти содержит только нули.
    #[inline]
    pub fn verify_zero(ptr: *const u8, len: usize) -> bool {
        unsafe { crate::sys::secure_verify_zero(ptr as *const c_void, len) == 0 }
    }

    /// Возвращает последнюю ошибку как строку.
    #[inline]
    pub fn last_error_ptr() -> *const libc::c_char {
        unsafe { crate::sys::secure_last_error() }
    }

    #[inline]
    pub fn last_error_str() -> &'static str {
        let ptr = last_error_ptr();
        if ptr.is_null() {
            return "";
        }
        unsafe {
            match CStr::from_ptr(ptr).to_str() {
                Ok(s) => s,
                Err(_) => "Invalid UTF-8 error message",
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Тесты
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_slice() {
        let mut data = [0xAAu8; 1024];
        secure_zero_slice_military(&mut data).unwrap();
        assert!(data.iter().all(|&x| x == 0));
    }
}