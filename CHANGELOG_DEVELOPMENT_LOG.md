# 🚀 Realix OS — Development Log & Changelog

## 📅 System Architecture & Kernel Enhancements

### 1. ⚙️ Protected Mode Engine & Ring 3 User Mode Isolation
- **`iretw` vs `iretd` Instruction Fix**: Resolved a critical 32-bit Protected Mode kernel crash in `source/kernel32/src/rlx_loader.rs`. Replaced 16-bit `iret` (`66 cf`) with 32-bit `iretd` (`cf`) to ensure complete 32-bit popping of `EIP`, `CS`, `EFLAGS`, `ESP`, and `SS`, preventing `#GP(0x40)` faults during Ring 0 to Ring 3 privilege transitions.
- **TSS & Kernel Stack Isolation**: Separated the Task State Segment (TSS) Ring 0 interrupt stack (`0x00088000`) from the kernel shell execution stack (`0x00090000`) to guarantee zero stack frame corruption during `INT 0x80` system call interrupts.
- **Context Saving & Clean Return (`setjmp`/`longjmp`)**: Implemented `KERNEL_SAVED_ESP`, `KERNEL_SAVED_EBP`, and `KERNEL_SAVED_EIP` register saving in `rlx_loader.rs`. Upon `SYS_EXIT` invocation, `exit_to_kernel()` jumps directly back to the shell execution loop.
- **GDT Expansion**: Expanded GDT size to 16 descriptors in `source/kernel32/src/x86/gdt.rs`.

---

### 2. 🔌 System API Specification (INT 0x80)

| Syscall ID | Name | Parameters | Description |
|------------|------|------------|-------------|
| **1** | `SYS_PRINT_STRING` | `ESI` = string ptr | Prints null-terminated string to VGA buffer |
| **2** | `SYS_PUTCHAR` | `EDX` = char, `EBX` = color | Prints single character with VGA color attributes (`1`=LightBlue, `2`=LightGreen, `7`=LightGray) |
| **3** | `SYS_EXIT` | None | Terminates application and returns to shell |
| **4** | `SYS_READ_KEY` | Return `EAX` = key | Reads ASCII key press from keyboard buffer |
| **5** | `SYS_CLEAR` | None | Clears text screen buffer and resets cursor |
| **6** | `SYS_GET_SYSINFO` | `EDI` = `&RealixSysInfo` | Populates live kernel system info struct (`os_name`, `os_version`, `cpu_vendor` via `CPUID`, `total_ram_mb`, `uptime_sec`) |
| **7** | `SYS_EXEC_16` | `ESI` = payload string ptr | Subsystem bridge syscall for executing 16-bit real mode payloads |

---

### 3. 🌐 Dynamic PATH Environment Subsystem & Command Resolver
- **Environment Management**: Implemented global environment variable `PATH=/bin;/apps;/` in `source/kernel32/src/shell.rs`. Added `set` / `env` / `export` command to view or dynamically alter `PATH`.
- **Dynamic Command Dispatcher (PATH Resolver)**: Built `find_and_run` in `source/kernel32/src/commands/exec.rs`. Executables in `PATH` can be launched directly by typing `snake`, `rlxfetch`, `rlxfetch16`, `exec16`, `app32`, `app16` without typing `exec` or `.rlx`.

---

## 📦 Executables & Applications (.RLX)

### 1. 🐍 `snake32.rlx` (32-bit Ring 3 Snake Game)
- Written in NASM 32-bit Assembly (`source/apps/snake32.asm`).
- Features wall collision detection, apple spawning, score tracking, WASD/Q controls, and System API screen rendering.

### 2. ⚡ `rlxfetch.rlx` (32-bit Native C Port)
- Written in **C11** (`source/apps/rlxfetch.c`) and compiled with **GCC 15** (`gcc -m32 -ffreestanding -nostdlib -fno-pic -fno-pie`).
- Queries live kernel system information via `SYS_GET_SYSINFO` (`INT 0x80`, `EAX = 6`).
- Displays vibrant Blue ASCII Art Logo **R**, live hardware `CPUID` vendor string, live PIT timer uptime, RAM memory, and `PATH`.

### 3. 🖥️ `rlxfetch16.rlx` (16-bit Real Mode Assembly Version)
- Written in NASM 16-bit Real Mode (`source/apps/rlxfetch16.asm`).
- Renders Blue **R** logo via BIOS `INT 0x10`, displays 16-bit kernel state and `PATH`.

### 4. 🌁 `exec16.rlx` (32-bit C Subsystem Bridge Utility)
- Written in C (`source/apps/exec16.c`) and compiled with **GCC 15**.
- Performs `SYS_EXEC_16` (`INT 0x80`, `EAX = 7`) system call to bridge execution to the 16-bit Kernel Subsystem.

### 5. 🌐 Universal `.RLX` Executable Spec v2.0
- Dual-mode header specification (`RLX_MODE_UNIVERSAL` / `'U'`) containing entry offsets for both 16-bit and 32-bit payloads within a single binary.

---

## 🛠️ Build Tools & Windows QEMU

- **`Makefile`**: Updated build rules to compile all `.RLX` binaries, link C apps via `linker32.ld` and `entry32.s`, and bundle them into `realix.img` and `realix.iso`.
- **Native Windows QEMU**: Configured native `qemu-system-x86_64.exe` execution via `run-windows.bat` for smooth 60 FPS display.
- **Git Commit**: Created initial git commit `5bce000` (`feat: Add .RLX 16-bit/32-bit app support, System API (INT 0x80), and Ring 3 isolation`).
