; © Realix > Kernel16 Shell Commands
; Безопасная исправленная версия
; ===============================

%ifndef KERNEL16_COMMANDS_ASM
%define KERNEL16_COMMANDS_ASM

%include 'kernel16/io/print_hex.asm'


; ============================================================
; ТАБЛИЦА КОМАНД
; ============================================================

align 2

cmd_table:
    dw str_help,     cmd_help
    dw str_cls,      cmd_cls
    dw str_clear,    cmd_cls
    dw str_reboot,   cmd_reboot
    dw str_shutdown, cmd_shutdown
    dw str_meminfo,  cmd_meminfo
    dw str_echo,     cmd_echo
    dw str_calc,     cmd_calc
    dw str_regs,     cmd_regs
    dw str_beep,     cmd_beep
    dw str_time,     cmd_time
    dw str_date,     cmd_date
    dw str_ls,       cmd_ls
    dw str_vga,      cmd_vga
    dw 0, 0

str_help:     db 'help', 0
str_cls:      db 'cls', 0
str_clear:    db 'clear', 0
str_reboot:   db 'reboot', 0
str_shutdown: db 'shutdown', 0
str_meminfo:  db 'meminfo', 0
str_echo:     db 'echo', 0
str_calc:     db 'calc', 0
str_regs:     db 'regs', 0
str_beep:     db 'beep', 0
str_time:     db 'time', 0
str_date:     db 'date', 0
str_ls:       db 'ls', 0
str_vga:      db 'vga', 0


; ============================================================
; ВЫПОЛНЕНИЕ КОМАНДЫ
; ============================================================

execute_cmd:
    push si
    push di
    push ax
    push bx

    call skip_spaces

    cmp byte [si], 0
    je .done

    mov di, cmd_table

.search:
    mov bx, [di]
    test bx, bx
    jz .not_found

    push si

.compare:
    mov al, [si]
    mov ah, [bx]

    test ah, ah
    jz .check_end

    cmp al, ah
    jne .mismatch

    inc si
    inc bx
    jmp .compare

.check_end:
    cmp al, 0
    je .match

    cmp al, ' '
    je .match

.mismatch:
    pop si
    add di, 4
    jmp .search

.match:
    ; Убираем сохранённое исходное значение SI.
    add sp, 2

    mov ax, [di + 2]
    call ax
    jmp .done

.not_found:
    mov si, err_unknown_cmd
    call print

.done:
    pop bx
    pop ax
    pop di
    pop si
    ret


; ============================================================
; HELP
; ============================================================

cmd_help:
    push si

    mov si, msg_help
    call print

    pop si
    ret


; ============================================================
; CLEAR
; ============================================================

cmd_cls:
    push ax

    mov ax, 0x0003
    int 0x10

    pop ax
    ret


; ============================================================
; REBOOT
; ============================================================

cmd_reboot:
    cli
    jmp 0xFFFF:0x0000


; ============================================================
; SHUTDOWN
; ============================================================

cmd_shutdown:
    push ax
    push bx
    push cx
    push si

    ; Проверяем наличие APM.
    mov ax, 0x5300
    xor bx, bx
    int 0x15
    jc .error

    ; Подключение к APM real-mode interface.
    mov ax, 0x5301
    xor bx, bx
    int 0x15
    jc .error

    ; Версия APM 1.2.
    mov ax, 0x530E
    xor bx, bx
    mov cx, 0x0102
    int 0x15
    jc .error

    ; Выключение питания.
    mov ax, 0x5307
    mov bx, 0x0001
    mov cx, 0x0003
    int 0x15

.error:
    mov si, err_shutdown
    call print

    pop si
    pop cx
    pop bx
    pop ax
    ret


; ============================================================
; MEMORY INFO
; ============================================================

cmd_meminfo:
    push si
    push di

    mov di, PCINFO_ADDR

    call show_lower_memory
    call print_new_line

    call show_free_memory
    call print_new_line

    call show_map_entries_cnt
    call print_new_line

    mov si, note_meminfo
    call print

    pop di
    pop si
    ret


; ============================================================
; ECHO
; ============================================================

cmd_echo:
    push si

    call skip_spaces

    cmp byte [si], 0
    je .done

    call print

.done:
    pop si
    ret


; ============================================================
; CALCULATOR
; ============================================================

%include 'kernel16/shell/cmd_calc.asm'


; ============================================================
; REGISTERS
; ============================================================

cmd_regs:
    ; pusha размещает в стеке:
    ; +0  DI
    ; +2  SI
    ; +4  BP
    ; +6  исходный SP
    ; +8  BX
    ; +10 DX
    ; +12 CX
    ; +14 AX
    pusha

    mov bp, sp

    mov si, msg_regs
    call print

    mov si, label_ax
    mov ax, [ss:bp + 14]
    call print_named_hex

    mov si, label_bx
    mov ax, [ss:bp + 8]
    call print_named_hex

    mov si, label_cx
    mov ax, [ss:bp + 12]
    call print_named_hex

    mov si, label_dx
    mov ax, [ss:bp + 10]
    call print_named_hex

    mov si, label_si
    mov ax, [ss:bp + 2]
    call print_named_hex

    mov si, label_di
    mov ax, [ss:bp + 0]
    call print_named_hex

    mov si, label_bp
    mov ax, [ss:bp + 4]
    call print_named_hex

    mov si, label_sp
    mov ax, [ss:bp + 6]
    call print_named_hex

    mov si, label_cs
    mov ax, cs
    call print_named_hex

    mov si, label_ds
    mov ax, ds
    call print_named_hex

    mov si, label_es
    mov ax, es
    call print_named_hex

    mov si, label_ss
    mov ax, ss
    call print_named_hex

    popa
    ret


; Вход:
;  - DS:SI: название регистра
;  - AX: значение
print_named_hex:
    push ax

    call print

    pop ax
    call print_hex
    call print_new_line
    ret


label_ax: db 'AX = 0x', 0
label_bx: db 'BX = 0x', 0
label_cx: db 'CX = 0x', 0
label_dx: db 'DX = 0x', 0
label_si: db 'SI = 0x', 0
label_di: db 'DI = 0x', 0
label_bp: db 'BP = 0x', 0
label_sp: db 'SP = 0x', 0
label_cs: db 'CS = 0x', 0
label_ds: db 'DS = 0x', 0
label_es: db 'ES = 0x', 0
label_ss: db 'SS = 0x', 0

msg_regs:
    db '--- Registers ---', ENTER, 0


; ============================================================
; BEEP
; ============================================================

cmd_beep:
    call print_beep_char
    ret


; ============================================================
; TIME
; ============================================================

cmd_time:
    pusha

    mov ah, 0x02
    int 0x1A
    jc .error

    ; CH = часы, CL = минуты, DH = секунды.
    mov al, ch
    call print_bcd_byte

    mov al, ':'
    call print_char

    mov al, cl
    call print_bcd_byte

    mov al, ':'
    call print_char

    mov al, dh
    call print_bcd_byte

    call print_new_line
    jmp .done

.error:
    mov si, err_time
    call print

.done:
    popa
    ret


; ============================================================
; DATE
; ============================================================

cmd_date:
    pusha

    mov ah, 0x04
    int 0x1A
    jc .error

    ; CH = век, CL = год, DH = месяц, DL = день.
    mov al, dl
    call print_bcd_byte

    mov al, '/'
    call print_char

    mov al, dh
    call print_bcd_byte

    mov al, '/'
    call print_char

    mov al, ch
    call print_bcd_byte

    mov al, cl
    call print_bcd_byte

    call print_new_line
    jmp .done

.error:
    mov si, err_date
    call print

.done:
    popa
    ret


; Вход:
;  - AL: BCD-число
print_bcd_byte:
    push ax
    push dx

    mov dl, al

    ; Старшая BCD-цифра.
    mov al, dl
    shr al, 4
    and al, 0x0F
    add al, '0'
    call print_char

    ; Младшая BCD-цифра.
    mov al, dl
    and al, 0x0F
    add al, '0'
    call print_char

    pop dx
    pop ax
    ret


; ============================================================
; LS — КОРНЕВОЙ КАТАЛОГ FAT12
; ============================================================

cmd_ls:
    pusha
    push es

    ; Корневой каталог читается в 0000:0500.
    xor ax, ax
    mov es, ax

    mov ax, [root_dir_lba]
    mov cx, [root_dir_size]
    mov dl, [boot_drive_num]
    mov bx, 0x0500
    call disk_read

    mov di, 0x0500
    mov bp, [dir_entries]

.next_entry:
    test bp, bp
    jz .done

    ; 00h означает конец используемых записей.
    cmp byte [es:di], 0x00
    je .done

    ; E5h — удалённая запись.
    cmp byte [es:di], 0xE5
    je .skip

    ; 0Fh — Long File Name.
    mov al, [es:di + 11]
    and al, 0x0F
    cmp al, 0x0F
    je .skip

    ; Пропускаем volume label.
    test byte [es:di + 11], 0x08
    jnz .skip

    call print_fat83_name

    ; Для директории показываем маркер.
    test byte [es:di + 11], 0x10
    jz .normal_file

    mov si, str_directory
    call print

.normal_file:
    call print_new_line

.skip:
    add di, 32
    dec bp
    jmp .next_entry

.done:
    pop es
    popa
    ret


; Вход:
;  - ES:DI: FAT directory entry
print_fat83_name:
    push ax
    push bx
    push cx

    ; Базовое имя — 8 байт.
    xor bx, bx

.name_loop:
    cmp bx, 8
    jae .extension

    mov al, [es:di + bx]
    cmp al, ' '
    je .name_next

    call print_char

.name_next:
    inc bx
    jmp .name_loop

.extension:
    ; Проверяем наличие расширения.
    cmp byte [es:di + 8], ' '
    je .done

    mov al, '.'
    call print_char

    mov bx, 8
    mov cx, 3

.ext_loop:
    mov al, [es:di + bx]
    cmp al, ' '
    je .ext_next

    call print_char

.ext_next:
    inc bx
    loop .ext_loop

.done:
    pop cx
    pop bx
    pop ax
    ret


; ============================================================
; VGA DEMO
; ============================================================

cmd_vga:
    pusha

    call vga_video_mode

    mov al, 0x01
    call vga_clear

    mov bx, 50
    mov ax, 50
    mov cx, 150
    mov dx, 150
    mov si, 0x04
    call vga_draw_rect

    xor ah, ah
    int 0x16

    call vga_text_mode

    popa
    ret


; ============================================================
; ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ
; ============================================================

skip_spaces:
.loop:
    cmp byte [si], ' '
    jne .done

    inc si
    jmp .loop

.done:
    ret


; Вход:
;  - AL: символ
print_char:
    push ax
    push bx

    mov ah, 0x0E
    xor bx, bx
    int 0x10

    pop bx
    pop ax
    ret


; ============================================================
; СТРОКИ
; ============================================================

err_unknown_cmd:
    db '[!] Unknown command. Type "help" for list.', 0

err_shutdown:
    db '[!] Shutdown failed or is not supported.', 0

err_time:
    db '[!] Failed to read RTC time.', 0

err_date:
    db '[!] Failed to read RTC date.', 0

note_meminfo:
    db 'Usable memory is calculated from BIOS E820.', 0

str_directory:
    db ' <DIR>', 0

msg_help:
    db 'Commands:', ENTER
    db '  [Base]', ENTER
    db '> help                  - Show this manual', ENTER
    db '> clear / cls           - Clear screen', ENTER
    db '> echo <text>           - Print text', ENTER
    db '> meminfo               - Show memory information', ENTER
    db '> calc <n1> <op> <n2>   - Integer calculator', ENTER
    db '> regs                  - Show register snapshot', ENTER
    db '> beep                  - Play BIOS beep', ENTER
    db '> time                  - Show RTC time', ENTER
    db '> date                  - Show RTC date', ENTER
    db '> ls                    - List FAT12 root directory', ENTER
    db '> vga                   - Run VGA graphics demo', ENTER
    db '  [Power]', ENTER
    db '> reboot                - Reboot computer', ENTER
    db '> shutdown              - Power off via APM', 0

%endif
