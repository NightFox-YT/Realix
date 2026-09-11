; © Realix > DOS: PSP (Program Segment Prefix)
; ø Copyright by @Ramix
; (08.09.26) v0.12 [Nightly]
; ================
; ❗️ Минимальный, но по назначению корректный PSP для запуска простых .COM-программ.
;    Окружение (offset 0x2C) сознательно не заполняется (Остаётся 0 = "нет окружения") -
;    заполнение потребовало бы отдельного сегмента, начинающегося с offset 0 (Формат DOS),
;    что не вписывается в модель плоского kernel16-бинарника. Простые программы это переживают.

; Настройка компиляции
bits 16

; Сегмент, где грузится PSP + .COM-программа (Совпадает с RLX16_APP_SEGMENT -
; RLX- и DOS-приложения никогда не выполняются одновременно, конфликта адресов нет)
DOS_PSP_SEGMENT equ 0x2000

; Смещение точки входа .COM-программы (Сразу после PSP, стандарт DOS)
DOS_COM_ENTRY_OFFSET equ 0x100

; Предел длины командной строки, копируемой в PSP (Стандартный лимит DOS - 126 байт)
DOS_CMDLINE_MAX equ 126

; > Построение минимального PSP по адресу DOS_PSP_SEGMENT:0x0000
; ❗️ Ожидается es = DOS_PSP_SEGMENT на входе (ds может быть любым - используем cs: для чтения)
; Параметры:
;  - si: смещение (в cs) аргументов командной строки (0 - пустая строка)
dos_build_psp:
    push ax
    push cx
    push di
    push si

    ; Обнуление всего PSP (256 байт = 128 слов)
    xor ax, ax
    mov di, 0
    mov cx, 128
    cld
    rep stosw

    ; offset 0x00: INT 20h (Старый способ завершения программы)
    mov word [es:0x00], 0x20CD

    ; offset 0x02: Сегмент "потолка" выделенной памяти (Условный, +64 КБ от PSP)
    mov ax, DOS_PSP_SEGMENT + 0x1000
    mov [es:0x02], ax

    ; offset 0x05: "CALL 5" точка входа (int 21h; retf) - историческая совместимость
    mov byte [es:0x05], 0xCD
    mov byte [es:0x06], 0x21
    mov byte [es:0x07], 0xCB

    ; offset 0x50: то же самое (Некоторые программы используют более новый оффсет)
    mov byte [es:0x50], 0xCD
    mov byte [es:0x51], 0x21
    mov byte [es:0x52], 0xCB

    ; offset 0x80-0x81+: Командная строка (Длина + текст + CR)
    call dos_copy_cmdline

    pop si
    pop di
    pop cx
    pop ax
    ret

; > Копирование командной строки в PSP (offset 0x80: длина, 0x81: текст, CR в конце)
; Параметры:
;  - si: смещение (в cs) аргументов, 0 - пустая строка
;  - es: DOS_PSP_SEGMENT (уже настроен вызывающим)
dos_copy_cmdline:
    push ax
    push cx
    push di

    xor cx, cx              ; Счётчик скопированных символов
    mov di, 0x81

    test si, si
    jz .done                ; Пустая командная строка - длина 0

.copy_loop:
    mov al, [cs:si]
    test al, al
    jz .done
    cmp cx, DOS_CMDLINE_MAX
    jae .done

    mov [es:di], al
    inc si
    inc di
    inc cx
    jmp .copy_loop

.done:
    ; offset 0x80: длина (без учёта CR)
    mov [es:0x80], cl

    ; Завершающий CR (DOS-конвенция)
    mov byte [es:di], 0x0D

    pop di
    pop cx
    pop ax
    ret
