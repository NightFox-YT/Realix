; © Realix > Command: Sysinfo
; (29.07.26) v0.11
; ================
; ❗️ Зависимости: kernel16/io/print_reg & print

; Основные константы
%include 'shared/config.asm'

; > Команда показа информации:
; 1. Видеорежим, кол-во столбцов, текущий номер страницы
; 2. Кол-во тиков с полуночи
; 3. Номер загрузочного устройства
cmd_sysinfo:
    push ax
    push bx
    push cx
    push dx
    push si
    push es

    ; Функция BIOS: Получение инфы о видеорежиме
    mov ah, 0x0F
    int 0x10

    ; Сохраняем в промежуточный регистр
    mov cx, ax

    ; Вывод номера видеорежима (нижний байт)
    mov si, msg_info_screen_mode
    call print
    movzx ax, cl
    call print_hex16

    ; Вывод кол-ва столбцов (верхний байт)
    mov si, msg_info_screen_cols
    call print
    movzx ax, ch
    call print_dec16

    ; Вывод номера текущей страницы (верхий байт bx)
    mov si, msg_info_screen_page
    call print
    movzx ax, bh
    call print_dec16

    call print_new_line

    ; Функция BIOS: Получение текущее кол-во тиков с полуночи
    xor ah, ah
    int 0x1A

    ; Записываем информацию в eax и выводим информацию о тиках
    mov si, msg_info_ticks
    call print
    movzx eax, cx
    shl eax, 16
    mov ax, dx
    call print_dec32

    call print_new_line

    ; Получение номера загрузочного устройства
    xor ax, ax
    mov es, ax
    movzx ax, byte [es:PCINFO_ADDR + PCINFO_DRIVE]

    ; Выводим номер загрузочного устройства
    mov si, msg_info_boot_drive
    call print
    call print_hex16

    pop es
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret

; Строки
msg_info_screen_mode:  db 'Video mode: ', 0
msg_info_screen_cols:  db ' | Columns: ', 0
msg_info_screen_page:  db ' | Page: ', 0

msg_info_ticks:        db 'Ticks since midnight: ', 0
msg_info_boot_drive:   db 'Boot drive (num): ', 0
