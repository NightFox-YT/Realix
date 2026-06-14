; © Realix > Free Memory
; (13.06.26) v0.06
; ================

; Основные константы
%include 'config.asm'

; > Получение общей длины всех отрезкой памяти по её карте
; Параметры:
;  - es:di: Указатель на карту памяти (`get_memory_map`)
; Вывод:
;  - ax: Число свободной памяти (КБ)
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
    ; Переводим байты в Килобайты (Деление на 2 ** 10)
    shrd ebx, edx, 10  ; Сдвигаем ebx на 10 бит, заполняя верх из edx
    shr edx, 10        ; Сдвигаем edx на 10 бит

    ; Результат в ebx (КБ)
    mov ax, bx

    pop si
    pop edx
    pop ecx
    pop ebx
    ret


; > Вывод кол-ва свободной памяти в текстовом режиме
; ❗️ Зависимости: kernel16/print.asm, kernel16/print_reg.asm
show_free_memory:
    push si
    push ax
    push es

    ; Считаем и выводим кол-во свободной памяти
    xor ax, ax
    mov es, ax
    mov di, PCINFO_ADDR
    call get_free_memory
    
    ; NOTE: ax содержит нужное число после `call get_free_memory`
    mov si, str_free_ram
    call print
    call print_reg
    mov si, str_kb
    call print

.done:
    pop ax
    pop si
    pop es
    ret

; Строки
str_free_ram:   db 'Free RAM: ', 0
str_kb:         db ' KB', 0