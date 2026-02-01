# 📦 Realix `v0.01` ![Status](https://img.shields.io/badge/status-archived-red) ![License](https://img.shields.io/github/license/NightFox-YT/Realix) ![Architecture](https://img.shields.io/badge/architecture-x86-blue)

⚠️ **This version is archived from `23.08.25`.** No further updates are guaranteed — changes may still be added at the creator's discretion.
Please consider upgrading to a newer version if available.

## 📌 About
Realix is a minimal 16-bit OS designed for x86 architecture, developed from scratch on NASM x86.
- **Size:** 2 bytes + `AA55` signature
- **Release:** `16.08.25`
- **Last update:** `01.02.26`

## ✨ Features
- 🆕 BIOS-based bootloader
- ⏳ VGA text output (80×25)
- ❌ No filesystem
- ❌ No user input
- ❌ No internet support
- ❌ No sounds

## 📦 Hardware Requirements
- **CPU:** x86 (8086+ compatible)
- **RAM:** ≥512 bytes
- **Motherboard:** BIOS-supported

## 📂 File hierarchy
```
.
├── source/
│   └── bootix.asm
├── build/          # Generated on build
├── Makefile
├── LICENSE
└── README.md
```

## 🛠 Build

### Linux
Use the ready-made solution `Makefile`, simply run:
```bash
make
```

### Windows
1. Compile source code with `NASM`:
   ```bash
   nasm -f bin source/bootix.asm -o build/bootix.bin
   ```
2. Write to disk image using **Rufus**.

### macOS
1. Compile source code with `NASM`:
   ```bash
   nasm -f bin source/bootix.asm -o build/bootix.bin
   ```
2. Create a bootable image with `dd`:
   ```bash
   dd if=build/bootix.bin of=build/realix.img bs=512 count=1
   ```

## 🔗 Links
- **Discord:** [discord.gg/zMzpWFgXaH](https://discord.gg/zMzpWFgXaH)

## 🙌 Contributing
Contributions of any kind are welcome:

- 🐞 Report bugs
- 💡 Suggest features
- 🔧 Optimize or refactor code

Feel free to open an issue or reach out via Discord.