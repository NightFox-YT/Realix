// © Realix > Command: Lspci
// (20.09.26) v0.13
// ================

// Подключение функций
use crate::drivers::pci;

/// Выводит список PCI устройств
pub fn run() {
    pci::scan(pci::PCI_WRITE_INFO_ON_VGA_SCREEN)
}