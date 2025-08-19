# Realix v0.03 | 19.08.25 [![Status](https://img.shields.io/badge/status-active-brightgreen)](https://github.com/NightFox-YT/Realix) ![License](https://img.shields.io/github/license/NightFox-YT/Realix)
- **Size:** ?? bytes + `AA55` signature
- **Architecture:** x86

## 📌 Description
Realix is a lightweight, simple 16-bit OS designed for x86 architecture, developed from scratch on NASM x86.

✅ This version is officially supported and frequently updated by the author.

## ✨ Features
- ✔️ BIOS-based bootloader
- ✔️ Text output ("Welcome...")
- ✔️ Read from disk | 🆕
- ⏳ FAT12 filesystem
- ❌ No user input
- ❌ No internet support
- ❌ No sounds

## 📦 Hardware Requirements
- **CPU:** x86 (8086+ compatible)
- **RAM:** ≥512 bytes
- **Motherboard:** BIOS-supported

## 📂 File hierarchy
- build/
  - bootix.bin
  - realix.img
- source/
  - bootix.asm
  - disk/
    - lba_to_chs.asm
    - read.asm
  - fat/
    - fat_headers.asm
  - kernel/
    - print.asm
    - print_reg.asm
- LICENSE
- Makefile
- README.md

## 🛠 Build
**Linux:**
  - Use the ready-made solution `Makefile` with command `make`.<br/>

**Windows/MacOS:**
  - Compile source code with `NASM`.
    - `nasm -f bin source/bootix.asm -o build/bootix.bin -i source/kernel -i source/disk -i source/fat`
  - Use the `DD utility`. (MacOS)
    - `dd if=/dev/zero of=build/realix.img bs=512 count=2880`
	  - `newfs_msdos -F 12 -f 2880 build/realix.img`
	  - `dd if=build/bootix.bin of=build/realix.img conv=notrunc`
  - Use `Rufus`. (Windows)

## 🙌 Join Us
**We welcome all contributions!** How to help:
- 🐞 Report bugs
- 💡 Suggest new features
- 📝 Improve documentation
- 🔧 Optimize code