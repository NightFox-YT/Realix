; © Realix > Shared: Error Registry (messages)
; (19.09.26) v0.13
; ================
; ❗️ Строки и таблица «код -> сообщение». Файл рассчитан на ОДНОКРАТНОЕ
;    подключение из main.asm (или любой точки, где эмиссия данных безопасна).
;
; Зависимости: shared/errors.asm (константы ERR_*), shared/config.asm (ENTER)
;
; Внутренняя нумерация совпадает с ERR_*: индекс таблицы = код - 1.

%ifndef ERRORS_MSG_ASM
%define ERRORS_MSG_ASM

%include 'shared/errors.asm'
%include 'shared/config.asm'

; --- Строки сообщений (все — с нуль-терминатором) --------------------------

err_file_not_found:    db '[!] E1: File not found!', 0
err_bad_cluster:       db '[!] E2: Bad cluster found!', 0
err_guard_overlap:     db '[!] E3: Destination overlaps a protected region!', 0
err_broken_chain:      db '[!] E4: Broken FAT chain!', 0

err_lower_memory:      db '[!] E5: Get lower memory failed (int 12h)!', ENTER, 0
err_memory_map:        db '[!] E6: Get memory map failed (int 15h)!', ENTER, 0
err_disk_init:         db '[!] E7: Disk init failed!', ENTER, 0

err_shutdown_failed:   db '[!] E8: PC shutdown failed! (No APM)', 0

err_rtc_time:          db '[!] E9: Failed to read RTC time.', 0
err_rtc_date:          db '[!] E10: Failed to read RTC date.', 0

err_rlx_bad_name:      db '[!] E11: Invalid filename format for FAT12.', ENTER, 0
err_rlx_magic:         db '[!] E12: Invalid .RLX header magic!', ENTER, 0
err_rlx_mode:          db '[!] E13: Not a 16-bit .RLX binary!', ENTER, 0
err_rlx_header:        db '[!] E14: Corrupted .RLX header!', ENTER, 0

err_nic_not_found:     db '[!] E15: RTL8139 not found on PCI bus!', 0
err_nic_reset_timeout: db '[!] E16: NIC reset timeout!', 0

err_out_of_memory:     db '[!] E17: Out of memory!', 0
err_map_out_of_range:  db '[!] E18: Address out of range!', 0

err_invalid_pcinfo:    db '[!] E19: Invalid PCINFO address!', 0
err_invalid_syscall:   db '[!] E20: Unknown syscall!', 0
err_invalid_argument:  db '[!] E21: Invalid argument!', 0
err_pit_freq_range:    db '[!] E22: PIT frequency out of range!', 0
err_unsupported_video: db '[!] E23: Unsupported video mode!', 0
err_io_port:           db '[!] E24: I/O port operation failed!', 0


; --- Таблица соответствия: код -> смещение строки --------------------------
; Порядок обязан совпадать с нумерацией ERR_* в errors.asm.
; Код 1 лежит по смещению 0 таблицы, код N — по смещению (N-1)*2.

align 2
error_msg_table:
    dw err_file_not_found     ; E1
    dw err_bad_cluster        ; E2
    dw err_guard_overlap      ; E3
    dw err_broken_chain       ; E4
    dw err_lower_memory       ; E5
    dw err_memory_map         ; E6
    dw err_disk_init          ; E7
    dw err_shutdown_failed    ; E8
    dw err_rtc_time           ; E9
    dw err_rtc_date           ; E10
    dw err_rlx_bad_name       ; E11
    dw err_rlx_magic          ; E12
    dw err_rlx_mode           ; E13
    dw err_rlx_header         ; E14
    dw err_nic_not_found      ; E15
    dw err_nic_reset_timeout  ; E16
    dw err_out_of_memory      ; E17
    dw err_map_out_of_range   ; E18
    dw err_invalid_pcinfo     ; E19
    dw err_invalid_syscall    ; E20
    dw err_invalid_argument   ; E21
    dw err_pit_freq_range     ; E22
    dw err_unsupported_video  ; E23
    dw err_io_port            ; E24


; > Получить указатель на строку сообщения по коду ошибки
; Параметры:
;  - ax: код ошибки (ERR_*)
; Вывод:
;  - si: указатель на строку сообщения (0 — если код вне реестра)
;  - CF: 0 (найдено), 1 (не найдено / код вне диапазона)
error_msg:
    test ax, ax
    jz .not_found
    cmp ax, ERR_COUNT
    ja .not_found

    ; ax = (code - 1) * 2 + error_msg_table
    dec ax
    shl ax, 1
    add ax, error_msg_table

    ; si = *(word*)ax
    mov si, ax
    mov si, [si]

    clc
    ret

.not_found:
    xor si, si
    stc
    ret

%endif