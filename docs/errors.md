# Realix Error Registry

Сквозная нумерация ошибок `E1..EN` по всему проекту.
Не переиспользовать номера; новые коды — только в конец реестра.

## Соглашения

| Возврат | Флаг | `si` содержит |
|---|---|---|
| Успех | `CF=0` | `'[+] <текст>'` или 0 |
| Ошибка | `CF=1` | `'[!] EN: <текст>'` |
| Фатал (Bootix / bios-api) | — | `'[#] <текст>'`, без кода |

Источники правды:
- ASM: `source/shared/errors.asm`, `source/shared/errors_msg.asm`
- Rust: `source/kernel32/src/errors.rs`

## Реестр

| Код | Константа | Сообщение | Где используется |
|-----|-----------|-----------|------------------|
| 0 | `ERR_NONE` | — | Успех |
| E1 | `ERR_FILE_NOT_FOUND` | File not found | `filesystem/fat12/file_load.asm` |
| E2 | `ERR_BAD_CLUSTER` | Bad cluster found | `filesystem/fat12/file_load.asm` |
| E3 | `ERR_GUARD_OVERLAP` | Destination overlaps a protected region | `filesystem/fat12/file_load.asm` |
| E4 | `ERR_BROKEN_CHAIN` | Broken FAT chain | `filesystem/fat12/file_load.asm` |
| E5 | `ERR_LOWER_MEMORY` | Get lower memory failed (int 12h) | `bootloader/initrix/main.asm` |
| E6 | `ERR_MEMORY_MAP` | Get memory map failed (int 15h) | `bootloader/initrix/main.asm` |
| E7 | `ERR_DISK_INIT` | Disk init failed | `bootloader/initrix/main.asm`, `kernel16/main.asm` |
| E8 | `ERR_SHUTDOWN_FAILED` | PC shutdown failed (No APM) | `kernel16/commands/power.asm` |
| E9 | `ERR_RTC_TIME` | Failed to read RTC time | `kernel16/commands/debug/date-time.asm` |
| E10 | `ERR_RTC_DATE` | Failed to read RTC date | `kernel16/commands/debug/date-time.asm` |
| E11 | `ERR_RLX_BAD_NAME` | Invalid filename format for FAT12 | `kernel16/commands/filesystem/*` (резерв) |
| E12 | `ERR_RLX_MAGIC` | Invalid .RLX header magic | `kernel16/commands/filesystem/exec.asm` |
| E13 | `ERR_RLX_MODE` | Not a 16-bit .RLX binary | `kernel16/commands/filesystem/exec.asm` |
| E14 | `ERR_RLX_HEADER` | Corrupted .RLX header | (резерв) |
| E15 | `ERR_NIC_NOT_FOUND` | RTL8139 not found on PCI bus | `drivers/network_rtl8139.asm` (резерв) |
| E16 | `ERR_NIC_RESET_TIMEOUT` | NIC reset timeout | `drivers/network_rtl8139.asm` (резерв) |
| E17 | `ERR_OUT_OF_MEMORY` | Out of memory | `kernel32/memory/frame_allocator.rs` (резерв) |
| E18 | `ERR_MAP_OUT_OF_RANGE` | Address out of range | `kernel32/memory/pmm.rs` (резерв) |
| E19 | `ERR_INVALID_PCINFO` | Invalid PCINFO address | `kernel32/main.rs` |
| E20 | `ERR_INVALID_SYSCALL` | Unknown syscall | `kernel16/syscall.asm` (резерв) |
| E21 | `ERR_INVALID_ARGUMENT` | Invalid argument | команды kernel16/32 |
| E22 | `ERR_PIT_FREQ_RANGE` | PIT frequency out of range | `kernel32/drivers/pit.rs` |
| E23 | `ERR_UNSUPPORTED_VIDEO` | Unsupported video mode | `kernel32/drivers/vga.rs` |
| E24 | `ERR_IO_PORT` | I/O port operation failed | (резерв) |

## Как добавить новую ошибку

1. Добавить `%define ERR_<NAME> <N+1>` в `shared/errors.asm`, увеличить `ERR_COUNT`.
2. Добавить `err_<name>: db '[!] E<N+1>: <текст>', 0` в `shared/errors_msg.asm`.
3. Добавить `dw err_<name>` в `error_msg_table` (строго в конец).
4. Добавить вариант в `Error`-enum в `errors.rs`.
5. Обновить эту таблицу.