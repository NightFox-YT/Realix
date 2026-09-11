; © Realix > Command: Registers
; (28.07.26) v0.11
; ================
; ❗️ Зависимости: kernel16-nightly/io: print, print_ctrl, print_reg (print_hex16)
; ❗️ Показывает регистры на входе в команду, т.е. уже с учётом работы CLI

; Смещения регистров в кадре `pusha` относительно bp (Именно такой порядок)
PUSHA_DI equ 0
PUSHA_SI equ 2
PUSHA_BP equ 4
PUSHA_SP equ 6
PUSHA_BX equ 8
PUSHA_DX equ 10
PUSHA_CX equ 12
PUSHA_AX equ 14

; > Команда вывода снимка регистров
cmd_regs:
    ; Снимок общих регистров (bp - указатель на последний элемент снимка)
    pusha
    mov bp, sp

    ; Вывод заголовка
    mov si, title_regs
    call print

    ; Первая строка (Общие регистры): ax, bx, cx, dx
    mov si, label_ax
    mov ax, [bp + PUSHA_AX]
    call print_named_hex16
    mov si, label_bx
    mov ax, [bp + PUSHA_BX]
    call print_named_hex16
    mov si, label_cx
    mov ax, [bp + PUSHA_CX]
    call print_named_hex16
    mov si, label_dx
    mov ax, [bp + PUSHA_DX]
    call print_named_hex16

    call print_new_line

    ; Вторая строка (Индексы и указатели): si, di, bp, sp
    mov si, label_si
    mov ax, [bp + PUSHA_SI]
    call print_named_hex16
    mov si, label_di
    mov ax, [bp + PUSHA_DI]
    call print_named_hex16
    mov si, label_bp
    mov ax, [bp + PUSHA_BP]
    call print_named_hex16
    mov si, label_sp
    mov ax, [bp + PUSHA_SP]
    call print_named_hex16

    call print_new_line

    ; Третья строка (Cегментные регистры): cs, ds, es, ss (Их `pusha` не сохраняет)
    mov si, label_cs
    mov ax, cs
    call print_named_hex16
    mov si, label_ds
    mov ax, ds
    call print_named_hex16
    mov si, label_es
    mov ax, es
    call print_named_hex16
    mov si, label_ss
    mov ax, ss
    call print_named_hex16

    popa
    ret

; > Вывод пары "label" + "value" с отступом в 2 пробела справа
; Параметры:
;  - ds:si: адрес названия регистра (label)
;  - ax: значение регистра (value)
print_named_hex16:
    push ax

    ; Выводим название и значение
    call print
    call print_hex16

    ; Выводим разделитель колонок
    mov al, ' '
    call print_char
    mov al, ' '
    call print_char

    pop ax
    ret

; Названия для вывода регистров
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

; Строки
title_regs: db '- Registers:', ENTER, 0
