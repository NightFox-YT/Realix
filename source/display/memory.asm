; © Realix > Display: Memory
; (15.08.26) v0.12
; ================
; ❗️ Зависимости: bios-api/io/print_*

; > Обёртка вывода кол-ва записей карты памяти (Текстовый режим)
; Параметры:
;  - cx: кол-во записей
show_map_entries_cnt:
    push si
    push ax

    ; Вывод заголовка сообщения
    mov si, str_memory_map
    call print

    ; Вывод значения и постфикса
    mov ax, cx
    call print_dec16
    mov si, str_entries
    call print

.done:
    pop ax
    pop si
    ret


; > Обёртка вывода кол-ва свободной памяти в текстовом режиме
; Параметры:
;  - eax: кол-во свободной памяти
show_usable_memory:
    push si

    ; Вывод заголовка сообщения
    mov si, str_usable_ram
    call print

    ; Вывод значения и постфикса
    call print_dec32
    mov si, str_mb
    call print

.done:
    pop si
    ret

; > Обёртка вывода кол-ва доступной "нижней" памяти (Текстовый режим)
; Параметры:
;  - ax: кол-во доступной "нижней" памяти
show_lower_memory:
    push si

    ; Вывод заголовка сообщения
    mov si, str_low_ram
    call print

    ; Вывод значения и постфикса
    call print_dec16
    mov si, str_kb_w_max
    call print

.done:
    pop si
    ret

; Строки
str_low_ram:    db 'Low RAM: ', 0
str_kb_w_max:   db ' KB / 640 KB', 0
str_usable_ram: db 'Usable RAM: ', 0
str_mb:         db ' MB', 0
str_memory_map: db 'Memory Map: ', 0
str_entries:    db ' entries', 0
