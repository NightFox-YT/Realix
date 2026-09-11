// © Realix > Command: Beep
// (07.09.26) v0.12
// ================

// Подключение функций
use crate::drivers::speaker;

/// Короткий системный гудок через PC speaker
pub fn run() {
    speaker::beep();
}
