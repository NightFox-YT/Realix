; © Realix > DOS: INT 21h API (Подмножество)
; ø Copyright by @Ramix
; (08.09.26) v0.12 [Nightly]
; ================
; ❗️ Зависимости: kernel16-nightly/io/print.asm (print_char), dos/psp.asm
; ❗️ Реализовано подмножество функций DOS API - для простых консольных .COM-программ
;    (Вывод текста, чтение клавиатуры, версия DOS, завершение программы).
;    ❌ НЕ реализовано: файловый ввод-вывод, управление памятью (48h/49h/4Ah), FCB,
;    прерывания (25h/35h) и т.д. - незнакомые функции возвращают CF=1, AX=1
;    ("Invalid function"), как и настоящий DOS, вместо падения/зависания.
; ❗️ Во время выполнения .COM-программы ds/es указывают на DOS_PSP_SEGMENT (не на
;    сегмент ядра!) - для доступа к СВОИМ переменным ядро обязано использовать cs:

; Настройка компиляции
bits 16

; Номера функций DOS API (ah при вызове int 21h)
DOS_FN_TERMINATE       equ 0x00
DOS_FN_READ_CHAR_ECHO  equ 0x01
DOS_FN_PUTCHAR         equ 0x02
DOS_FN_READ_CHAR_RAW   equ 0x08
DOS_FN_PRINT_STRING    equ 0x09
DOS_FN_BUFFERED_INPUT  equ 0x0A
DOS_FN_CHECK_KBD       equ 0x0B
DOS_FN_GET_VERSION     equ 0x30
DOS_FN_TERMINATE_EX    equ 0x4C

; Символ-терминатор строк для DOS_FN_PRINT_STRING
DOS_DOLLAR equ '$'

; Сохранённый указатель стека вызывающего (msexec) - для возврата при завершении программы
dos_saved_ss: dw 0
dos_saved_sp: dw 0

; > Установка векторов int 0x20 и int 0x21 в IVT
dos_int21_init:
    push ax
    push es

    xor ax, ax
    mov es, ax

    mov word [es:0x20 * 4], dos_int20_entry
    mov word [es:0x20 * 4 + 2], cs

    mov word [es:0x21 * 4], dos_int21_entry
    mov word [es:0x21 * 4 + 2], cs

    pop es
    pop ax
    ret

; > Точка входа int 0x20 (Старый способ завершения программы - без кода возврата)
dos_int20_entry:
    jmp dos_terminate

; > Точка входа int 0x21 (Основной диспетчер DOS API)
dos_int21_entry:
    ; ❗️ Не делаем pusha/сохранение всех регистров - каждая функция сама решает,
    ;    что портить, как в настоящем DOS (Вызывающая программа это ожидает)
    cmp ah, DOS_FN_TERMINATE
    je dos_terminate
    cmp ah, DOS_FN_TERMINATE_EX
    je dos_terminate

    cmp ah, DOS_FN_PUTCHAR
    je .fn_putchar
    cmp ah, DOS_FN_PRINT_STRING
    je .fn_print_string
    cmp ah, DOS_FN_READ_CHAR_ECHO
    je .fn_read_char_echo
    cmp ah, DOS_FN_READ_CHAR_RAW
    je .fn_read_char_raw
    cmp ah, DOS_FN_BUFFERED_INPUT
    je .fn_buffered_input
    cmp ah, DOS_FN_CHECK_KBD
    je .fn_check_kbd
    cmp ah, DOS_FN_GET_VERSION
    je .fn_get_version

    ; Неизвестная функция - как в настоящем DOS: CF=1, AX=1
    mov ax, 1
    stc
    iret

; > AH=02h: Вывод символа (dl - символ)
.fn_putchar:
    push ax
    mov al, dl
    call print_char
    pop ax
    clc
    iret

; > AH=09h: Вывод строки, завершённой символом '$' (ds:dx)
.fn_print_string:
    push si
    mov si, dx

.fn_print_string_loop:
    mov al, [si]
    cmp al, DOS_DOLLAR
    je .fn_print_string_done
    call print_char
    inc si
    jmp .fn_print_string_loop

.fn_print_string_done:
    pop si
    clc
    iret

; > AH=01h: Чтение символа с эхом на экран (Результат в al)
.fn_read_char_echo:
    xor ah, ah
    int 0x16
    call print_char
    clc
    iret

; > AH=08h: Чтение символа БЕЗ эха (Результат в al)
.fn_read_char_raw:
    xor ah, ah
    int 0x16
    clc
    iret

; > AH=0Bh: Проверка готовности клавиатуры (al = 0xFF если есть символ, иначе 0x00)
.fn_check_kbd:
    mov ah, 1
    int 0x16
    jz .fn_check_kbd_empty
    mov al, 0xFF
    jmp .fn_check_kbd_done

.fn_check_kbd_empty:
    xor al, al

.fn_check_kbd_done:
    clc
    iret

; > AH=0Ah: Буферизованный ввод строки (ds:dx -> [0]=макс.длина,[1]=факт.длина,[2..]=текст)
.fn_buffered_input:
    push bx
    push cx
    push si

    mov si, dx
    mov bl, [si]        ; bl - максимальное кол-во символов (без CR)
    xor cl, cl           ; cl - счётчик уже введённых символов
    add si, 2             ; si -> начало буфера текста

.fn_buffered_input_loop:
    xor ah, ah
    int 0x16              ; al - введённый символ

    cmp al, 0x0D           ; Enter завершает ввод
    je .fn_buffered_input_done

    cmp al, 0x08            ; Backspace
    jne .fn_buffered_input_store

    test cl, cl
    jz .fn_buffered_input_loop  ; Нечего стирать

    dec cl
    dec si
    call print_char          ; Эхо backspace (Стирание символа терминалом)
    jmp .fn_buffered_input_loop

.fn_buffered_input_store:
    cmp cl, bl
    jae .fn_buffered_input_loop  ; Буфер полон - игнорируем остальные символы

    mov [si], al
    inc si
    inc cl
    call print_char
    jmp .fn_buffered_input_loop

.fn_buffered_input_done:
    call print_char           ; Эхо Enter (Перевод строки на экране)
    mov si, dx
    mov [si + 1], cl           ; offset 1: фактическая длина введённого текста

    pop si
    pop cx
    pop bx
    clc
    iret

; > AH=30h: Версия DOS (al=major, ah=minor, bh=0, bl:cx=0 - серийный номер)
.fn_get_version:
    mov al, 5
    mov ah, 0
    mov bx, 0
    mov cx, 0
    clc
    iret

; > Завершение выполнения .COM-программы (int 0x20 или int 0x21 ah=00h/4Ch)
; ❗️ Не возвращает управление программе - восстанавливает стек msexec и прыгает обратно
dos_terminate:
    cli
    mov ax, cs
    mov ds, ax             ; Восстанавливаем ds для доступа к своим переменным

    mov ax, [dos_saved_ss]
    mov ss, ax
    mov sp, [dos_saved_sp]
    sti

    jmp dos_terminate_return
