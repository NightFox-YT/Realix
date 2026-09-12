; © Realix > Lower memory
; (15.08.26) v0.12
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
