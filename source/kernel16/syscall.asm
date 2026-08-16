; © Realix > Kernel16: System API (INT 0x80)
; ø Copyright by Roario0602
; (07.08.26) v0.11
; ================
; ❗️ Зависимости: bios-api/io/print, bios-api/vga, bios-api/keyboard

; Настройка компиляции
bits 16

; Основные константы
%include 'shared/config.asm'

; Флаги работы приложения
app16_running: db 0


; > Инициализация вектора int 0x80 в IVT
syscall16_init:
    push es

    ; Обнуление сегмента es для доступа к IVT
    xor ax, ax
    mov es, ax

    ; Перезапись обработчика 0x80 прерывания
    mov word [es:0x80 * 4], syscall16_entry
    mov word [es:0x80 * 4 + 2], cs

    pop es
    ret


; > Точка входа для int 0x80
syscall16_entry:
    pusha

    ; Чтение параметра прерывания и вызов нужного обработчика
    cmp ah, SYS_PRINT_STRING
    je .sys_print
    cmp ah, SYS_PUTCHAR
    je .sys_putchar
    cmp ah, SYS_EXIT
    je .sys_exit
    cmp ah, SYS_READ_KEY
    je .sys_read_key
    cmp ah, SYS_CLEAR
    je .sys_clear

    jmp .done

.sys_print:
    call print
    jmp .done

.sys_putchar:
    call print_char
    jmp .done

.sys_read_key:
    call wait_key
    jmp .done

.sys_clear:
    call clear_screen
    jmp .done

.sys_exit:
    mov byte [cs:app16_running], 0
    jmp .done

.done:
    popa
    iret
