# Realix v0.01 | 19.07.25
- **Size:** 12 bytes (+2 bytes - signature `AA55`)
- **Support:** ✅ (Active)

## 📌 Description
Realix is a lightweight, simple 16-bit OS designed specifically for x86 architecture, developed from scratch on NASM x86.

✅ This version is officially supported and frequently updated by the author.

## ✨ Features
- ✔️ BIOS-based bootloader | 🆕
- ❌ No text output
- ❌ No user input
- ❌ No networking capability
- ❌ No sounds
- ❌ No filesystem

## 📦 Hardware Requirements
- **CPU:** x86 (8086+)
- **RAM:** 512+ bytes
- **Motherboard:** BIOS-supported

## 📂 File hierarchy
- source/
  - boot.asm
- readme.md
- license
- compile.bat

## 🛠 Build
1. Compile `boot.asm` with NASM to bin file. (Command: nasm -f bin boot.asm -o boot.bin)
2. [VM] Convert `boot.bin` to iso image with UltraISO, PowerISO, etc...
<br/> [Real hardware] Rename `boot.bin` to `Realix.iso`.
3. Done, then you can use it on VM or Real hardware!

Optional: Use the ready-made solution (`compile.bat`), which requires Windows with NASM and QEMU.

## 🙌 Join Us
Help us improve Realix! Pull requests, suggestions, and feedback are welcome.