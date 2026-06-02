; © Realix > Free Memory
; (02.06.26) v0.05
; ================

; > Получение общей длины всех отрезкой памяти по её карте
; Параметры:
;  - es:di: Указатель на карту памяти (`get_memory_map`)
; Вывод:
;  - ax: Число свободной памяти (МБ)
get_free_memory:
    push ebx
    push ecx
    push edx
    push si

    xor ebx, ebx
    xor edx, edx

    ; Читаем количество записей (Если 0 - Выходим)
    mov cl, [es:di + 3]
    xor ch, ch
    jcxz .empty

    ; Адрес первой записи E820
    mov si, di
    add si, 5

.loop:
    ; Проходим по свободным регионам памяти
    cmp dword [es:si + 16], 1
    jne .skip_entry

    ; Прибавляем 64-битную длину региона к edx:ebx
    add ebx, [es:si + 8]       ; Смещение +8: Младшие 32 бита длины
    adc edx, [es:si + 12]      ; Смещение +12: Старшие 32 бита длины + флаг переноса

.skip_entry:
    ; Переход к следующей записи
    add si, 24
    loop .loop

.empty:
    ; Переводим байты в Мегабайты (Деление на 1 048 576)
    shrd ebx, edx, 20  ; Сдвигаем ebx на 20 бит, заполняя верх из edx
    shr edx, 20        ; Сдвигаем edx на 20 бит

    ; Результат в ebx (МБ | До 64 ГБ поместится в ax)
    mov ax, bx

    pop si
    pop edx
    pop ecx
    pop ebx
    ret