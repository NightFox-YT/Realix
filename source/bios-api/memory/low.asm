; © Realix > Lower memory
; (27.07.26) v0.1
; ================

; > Получение кол-ва доступной "нижней" памяти
; Вывод:
;  - ax: Кол-во доступной "нижней" памяти (КБ)
;  - CF (Carry Flag): 0 (Успех), 1 (Ошибка)
get_lower_memory:
    int 12h

    ; Проверка: Размер доступной "нижней" памяти != 0
    test ax, ax
    jnz .done

.fail:
    stc
    ret

.done:
    clc
    ret

; > Вывод кол-ва доступной "нижней" памяти в текстовом режиме
; ❗️ Зависимости: kernel16/print.asm
show_lower_memory:
    push si
    push ax

    ; Выводим эту информацию о "нижней" памяти
    mov si, str_low_ram
    call print

    call get_lower_memory
    call print_dec16

    mov si, str_kb_w_max
    call print

.done:
    pop ax
    pop si
    ret

; Строки
str_low_ram:  db 'Low RAM: ', 0
str_kb_w_max: db ' KB / 640 KB', 0