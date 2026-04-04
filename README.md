# 📦 Realix `v0.04` ![Status](https://img.shields.io/badge/status-archived-red) ![License](https://img.shields.io/github/license/NightFox-YT/Realix) ![Architecture](https://img.shields.io/badge/architecture-x86-blue)

⚠️ **This version is archived from `05.04.26`.** No further updates are guaranteed — changes may still be added at the creator's discretion.
Please consider upgrading to a newer version if available.

## 📌 About
Realix is a **minimal 16-bit OS** designed for x86 architecture, developed from scratch on NASM x86.
- **Size:** `593 bytes`
- **Initial release:** `25.08.25`

## ✨ Features
- ✔️ BIOS-based bootloader
- ✔️ VGA text output (80×25)
- ✔️ Read from disk
- 🆕 Read the second-stage bootloader with FAT12
- ⏳ Load files (kernel) with FAT12
- ❌ No user input handling
- ❌ No internet support
- ❌ No sounds

## 📦 Hardware Requirements
- **CPU:** x86 (8086+ compatible)
- **RAM:** 1 MB
- **Motherboard:** BIOS-supported

## 📂 File hierarchy
```
.
├─ source/
│  ├─ bootix.asm
│  ├─ initrix.asm
│  ├─ disk/
│  │  └─ read.asm
│  └─ kernel/
│     ├─ print.asm
│     └─ print_dec.asm
├─ build/               # Generated on build
├─ Makefile
├─ LICENSE
└─ README.md
```

## 🛠 Build
### Linux
Use the ready-made solution `Makefile`. Simply run: `make`.

### macOS
1. Replace line `mkfs.fat -F 12 -n "Realix" $(BUILD_DIR)/realix.img` with `newfs_msdos -F 12 -f 2880 $(BUILD_DIR)/realix.img` in the `Makefile`.
2. Simply run: `make`

### Windows
> ⚠️ Windows builds are no longer supported as of `v0.04`.
> Use Linux or macOS instead. WSL2 is also supported.

## 🔗 Links & Contributing
- **Discord:** [discord.gg/zMzpWFgXaH](https://discord.gg/zMzpWFgXaH)

Contributions of any kind are welcome:

- 🐞 **Report bugs.**
- 💡 **Suggest features** or improvements.
- 🔧 **Optimize or refactor code.**

Feel free to open an issue or reach out via Discord.