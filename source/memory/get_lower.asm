; © Realix > Lower memory
; (02.06.26) v0.05
; ================

; > Получение кол-ва свободной "нижней" памяти
; Вывод:
;  - ax: Кол-во свободной памяти (КБ)
get_lower_memory:
    clc
    int 12h
    jc .fail

.end:
    clc
    ret

.fail:
    stc
    ret