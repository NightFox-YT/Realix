; © Realix > Kernel16: Panic Handler
; ø Inspired by Atimenka
; (16.08.26) v0.12
; ================
; ❗️ Зависимости: bios-api/io/*, bios-api/vga


; > Установка обработчиков нескольких исключений (0, 3-6, 13)
install_exception_handlers:
    push ax
    push es

    ; Сброс доп. сегмента с сохранением текущего сегмента кода
    xor ax, ax
    mov es, ax
    mov ax, cs

    ; Записываем в IVT BIOS смещения для своих обработчиков и сегмент кода
    mov word [es:0x00 * 4], isr_divide_error
    mov word [es:0x00 * 4 + 2], ax
    mov word [es:0x03 * 4], isr_breakpoint
    mov word [es:0x03 * 4 + 2], ax
    mov word [es:0x04 * 4], isr_overflow
    mov word [es:0x04 * 4 + 2], ax
    mov word [es:0x05 * 4], isr_bounds
    mov word [es:0x05 * 4 + 2], ax
    mov word [es:0x06 * 4], isr_invalid_opcode
    mov word [es:0x06 * 4 + 2], ax
    mov word [es:0x0D * 4], isr_general_protection
    mov word [es:0x0D * 4 + 2], ax

    pop es
    pop ax
    ret


; > Вызов собственного исключения (Тестирование)
debug_panic_manual:
    mov si, exc_manual_str
    jmp exception_common


; > Обработчики некоторых исключений
isr_divide_error:
    mov si, exc_divide_str
    jmp exception_common

isr_breakpoint:
    mov si, exc_breakpoint_str
    jmp exception_common

isr_overflow:
    mov si, exc_overflow_str
    jmp exception_common

isr_bounds:
    mov si, exc_bounds_str
    jmp exception_common

isr_invalid_opcode:
    mov si, exc_invalid_opcode_str
    jmp exception_common

isr_general_protection:
    mov si, exc_gp_str
    jmp exception_common

; > Общий обработчик для исключений (Имитация синего экрана смерти)
; (Стек содержит IP, CS, FLAGS)
exception_common:
    ; Выключение прерываний и обновление фрейма стека
    cli
    push bp
    mov bp, sp
    pusha

    ; Обновляем ds под cs, чтобы спокойно использовать переменные
    push ds
    push cs
    pop ds

    ; Сохраняем сообщение о типе исключения
    mov [.panic_message_ptr], si

    ; Убедимся, что включен текстовый режим и зальём экран одним цветом
    call vga_enable_text_mode
    mov bh, 0x1F               ; Белый текст на синем фоне
    call fill_screen

    ; Вывод заголовка экрана смерти
    mov si, panic_title
    call print

    ; Вывод сообщения о типе исключения
    mov si, panic_reason_str
    call print
    mov si, [.panic_message_ptr]
    call print

    call print_new_line

    ; Вывод адреса на котором произошло исключение CS:IP
    mov si, panic_ptr_str
    call print

    mov ax, [ss:bp + 4]
    call print_hex16
    mov si, colon_sep
    call print
    mov ax, [ss:bp + 2]
    call print_hex16

    call print_new_line
    call print_new_line

    ; Вывод заголовка диагностической информации
    mov si, panic_debug_title
    call print

    ; Вывод диагностической информации
    mov si, panic_flags
    call print
    mov ax, [ss:bp + 6]
    call print_hex16
    call print_new_line

    mov si, panic_ax
    call print
    mov ax, [bp - 2]
    call print_hex16
    call print_new_line

    mov si, panic_bx
    call print
    mov ax, [bp - 8]
    call print_hex16
    call print_new_line

    mov si, panic_cx
    call print
    mov ax, [bp - 4]
    call print_hex16
    call print_new_line

    mov si, panic_dx
    call print
    mov ax, [bp - 6]
    call print_hex16
    call print_new_line

    mov si, panic_si
    call print
    mov ax, [bp - 14]
    call print_hex16
    call print_new_line

    mov si, panic_di
    call print
    mov ax, [bp - 16]
    call print_hex16
    call print_new_line

    ; Выводим подсказку о происходящем ужасе
    mov si, panic_hint
    call print

.halt:
    hlt
    jmp .halt

.panic_message_ptr: dw 0


; Строки экрана смерти
panic_title:
    db '                              [+] Realix here...', ENTER, ENTER
    db '               $$$$$$$\   $$$$$$\  $$\   $$\ $$$$$$\  $$$$$$\  ', ENTER
    db '               $$  __$$\ $$  __$$\ $$$\  $$ |\_$$  _|$$  __$$\ ', ENTER
    db '               $$ |  $$ |$$ /  $$ |$$$$\ $$ |  $$ |  $$ /  \__|', ENTER
    db '               $$$$$$$  |$$$$$$$$ |$$ $$\$$ |  $$ |  $$ |      ', ENTER
    db '               $$  ____/ $$  __$$ |$$ \$$$$ |  $$ |  $$ |      ', ENTER
    db '               $$ |      $$ |  $$ |$$ |\$$$ |  $$ |  $$ |  $$\ ', ENTER
    db '               $$ |      $$ |  $$ |$$ | \$$ |$$$$$$\ \$$$$$$  |', ENTER
    db '               \__|      \__|  \__|\__|  \__|\______| \______/ ', ENTER, ENTER, 0                                                                                 

panic_reason_str:  db 'Reason: ', 0
panic_ptr_str:     db 'Pointer: ', 0
panic_debug_title: db '! Debug info:', ENTER, 0
panic_flags:       db '    FLAGS: 0x', 0
panic_ax:          db '    AX: 0x', 0
panic_bx:          db '    BX: 0x', 0
panic_cx:          db '    CX: 0x', 0
panic_dx:          db '    DX: 0x', 0
panic_si:          db '    SI: 0x', 0
panic_di:          db '    DI: 0x', 0
panic_hint:
    db ENTER, ENTER
    db '          Realix halted to keep the error visible - reset to reboot', 0

; Строки исключений
exc_manual_str:         db 'Manual panic requested', 0
exc_divide_str:         db 'Divide By Zero', 0
exc_breakpoint_str:     db 'Breakpoint', 0
exc_overflow_str:       db 'Overflow', 0
exc_bounds_str:         db 'Bound Range Exceeded', 0
exc_invalid_opcode_str: db 'Invalid Opcode', 0
exc_gp_str:             db 'General Protection Fault', 0

; Вспомогательные строки
colon_sep: db ':', 0