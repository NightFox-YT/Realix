# Realix v0.01 | 16.08.25 [![Status](https://img.shields.io/badge/status-active-brightgreen)](https://github.com/NightFox-YT/Realix) ![License](https://img.shields.io/github/license/NightFox-YT/Realix)
- **Size:** 2 bytes (+2 bytes - signature `AA55`)
- **Architecture:** x86

## 📌 Description
Realix is a lightweight, simple 16-bit OS designed for x86 architecture, developed from scratch on NASM x86.

✅ This version is officially supported and frequently updated by the author.

## ✨ Features
- ✔️ BIOS-based bootloader | 🆕
- ⏳ VGA text output (80x25)
- ❌ No filesystem
- ❌ No user input
- ❌ No networking
- ❌ No sounds

## 📦 Hardware Requirements
- **CPU:** x86 (8086+ compatible)
- **RAM:** 512+ bytes
- **Motherboard:** BIOS-supported

## 📂 File hierarchy
- build/
  - bootix.bin
  - realix.img
- source/
  - bootix.asm
- Makefile
- README.md
- LICENSE

## 🛠 Build
**Linux:** Use the ready-made solution `Makefile` with command `make`.
<br/>**Windows/MacOS:**
  - Compile source code with NASM.
    - `nasm -f bin source/bootix.asm -o build/bootix.bin`
  - Use the DD utility. (MacOS)
    - `dd if=build/bootix.bin of=build/realix.img bs=512 count=1`
  - Use Rufus. (Windows)

## 🙌 Join Us
**We welcome all contributions!**
How to help:
- 🐞 Report bugs
- 💡 Suggest new features
- 📝 Improve documentation
- 🔧 Optimize code