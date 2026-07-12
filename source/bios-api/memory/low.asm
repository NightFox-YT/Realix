; © Realix > Lower memory
; (02.06.26) v0.05
; ================

; Основные константы
%include 'shared/config.asm'

; > Получение кол-ва свободной "нижней" памяти
; Вывод:
;  - ax: Кол-во свободной памяти (КБ)
;  - CF (Carry Flag): 0 — успех, 1 — ошибка
get_lower_memory:
    clc
    int 12h
    ret

; > Вывод кол-ва "нижней" памяти в текстовом режиме
; ❗️ Зависимости: kernel16/print.asm
show_lower_memory:
    push si
    push ax
    push es

    ; Настраиваем сегмент `es` под PCINFO
    xor ax, ax
    mov es, ax

    ; Выводим информацию о кол-ве "нижней" памяти
    mov si, str_low_ram
    call print
    mov ax, word [es:PCINFO_ADDR]
    call print_dec16
    mov si, str_kb_w_max
    call print

.done:
    pop es
    pop ax
    pop si
    ret

; Строки
str_low_ram:  db 'Low RAM: ', 0
str_kb_w_max: db ' KB / 640 KB', 0