# 📦 Realix `v0.03` ![Status](https://img.shields.io/badge/status-archived-red) ![License](https://img.shields.io/github/license/NightFox-YT/Realix) ![Architecture](https://img.shields.io/badge/architecture-x86-blue)

⚠️ **This version is archived from `21.03.26`.** No further updates are guaranteed — changes may still be added at the creator's discretion.
Please consider upgrading to a newer version if available.

## 📌 About
Realix is a minimal 16-bit OS designed for x86 architecture, developed from scratch on NASM x86.
- **Size:** 512 bytes (354 bytes of them is code + `AA55` signature)
- **Release:** `19.08.25`

## ✨ Features
- ✔️ BIOS-based bootloader
- ✔️ VGA text output (80×25)
- 🆕 Read from disk
- ⏳ FAT12 filesystem
- ❌ No user input
- ❌ No internet support
- ❌ No sounds

## 📦 Hardware Requirements
- **CPU:** x86 (8086+ compatible)
- **RAM:** 512 bytes
- **Motherboard:** BIOS-supported

## 📂 File hierarchy
```
.
├─ source/
│  ├─ bootix.asm
│  ├─ kernel/
│  │  ├─ print.asm
│  │  └─ print_reg.asm
│  └─ disk/
│  │  ├─ lba_to_chs.asm
│  │  └─ read.asm
├─ build/                # Generated on build
├─ Makefile
├─ LICENSE
└─ README.md
```

## 🛠 Build
### Linux
Use the ready-made solution `Makefile`, simply run: `make`.

### macOS
1. Replace line `mkfs.fat -F 12 -n "Realix" $(BUILD_DIR)/realix.img` with `newfs_msdos -F 12 -f 2880 $(BUILD_DIR)/realix.img` in the `Makefile`.
2. Simply run: `make`

### Windows
1. Compile source code with `NASM`:
   ```bash
   nasm -f bin source/bootix.asm -o build/bootix.bin
   ```
2. Write to disk image using `Rufus`.

## 🔗 Links & 🙌 Contributing
- **Discord:** [discord.gg/zMzpWFgXaH](https://discord.gg/zMzpWFgXaH)

Contributions of any kind are welcome:

- 🐞 Report bugs
- 💡 Suggest features
- 🔧 Optimize or refactor code

Feel free to open an issue or reach out via Discord.