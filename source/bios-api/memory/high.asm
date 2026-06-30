; © Realix > High memory (Memory Map)
; (30.06.26) v0.07
; ================

; Основные константы
%include 'shared/config.asm'

; > Получение карты памяти через прерывание int 0x15 (E820)
; Параметры:
;  - es:di: адрес, куда мы будем сохранять таблицу карт памяти
; Вывод:
;  - bp: количество успешно прочитанных записей
;  - di: указатель на конец таблицы
get_memory_map:
    push eax
    push ebx
    push ecx
    push edx

    ; Установка начальных параметров, счётчика записей, "SMAP"
    xor ebx, ebx
    xor bp, bp
    mov edx, 0x0534D4150

    ; Получение 1-й записи
    mov eax, 0xe820
    mov [es:di + 20], dword 1  ; Делаем запись валидной для ACPI 3.X (Она сможет перезаписаться)
    mov ecx, 24                ; Запрашиваем 24 байта
    int 0x15
    jc .fail                   ; Установленный CF при первом вызове - "функция не поддерживается"
    mov edx, 0x0534D4150       ; Некоторые BIOS, могут затирать этот регистр
    cmp eax, edx               ; В случае успеха eax должен быть сброшен в "SMAP"
    jne .fail

    ; Ebx = 0 означает, что список состоит всего из 1 записи (бесполезно)
    test ebx, ebx
    je .fail

    jmp .jmpin

.loop:
    mov eax, 0xe820            ; Исправляем затирание
    mov [es:di + 20], dword 1  ; Делаем запись валидной для ACPI 3.x (Она сможет перезаписаться)
    mov ecx, 24                ; Запрашиваем 24 байта
    int 0x15

    jc .done                ; Установленный Carry Flag - "конец списка достигнут"
    mov edx, 0x0534D4150    ; Некоторые BIOS, могут затирать этот регистр

.jmpin:
    jcxz .skipentry            ; Пропускаем записи с нулевой длиной
    cmp cl, 20                 ; Есть ли расширенные атрибуты ACPI 3.X?
    jbe short .notext

    test byte [es:di + 20], 1  ; Есть атрибуты: очищен ли бит "игнорировать эти данные"?
    je short .skipentry

.notext:
    mov ecx, [es:di + 8]  ; Получаем младшие 32 бита длины области памяти
    or ecx, [es:di + 12]  ; Проверяем старшие 32 бита на 0
    jz .skipentry         ; Если 64-битная длина равна 0, пропустить запись
    inc bp                ; Получена хорошая запись, переход к следующему месту хранения

    ; Динамическая защита от перезаписи загрузчика по адресу 0x7C00
    cmp di, 0x6FE8
    jae .done
    add di, 24

.skipentry:
    ; Если ebx сбрасывается в 0, конец список достигнут
    test ebx, ebx
    jne short .loop

.done:
    ; Очищаем флаг переноса и выходим
    clc

    pop edx
    pop ecx
    pop ebx
    pop eax
    ret

.fail:
    ; Выход по ошибке "Функция не поддерживается"
    stc

    pop edx
    pop ecx
    pop ebx
    pop eax
    ret


; > Вывод кол-ва записей карты памяти в текстовом режиме
; ❗️ Зависимости: kernel16/print.asm
show_map_entries_cnt:
    push si
    push ax
    push es

    ; Настраиваем сегмент `es` под PCINFO
    xor ax, ax
    mov es, ax

    ; Выводим информацию о кол-ве записей карты памяти
    mov si, str_memory_map
    call print
    mov ax, word [es:PCINFO_ADDR + 3]
    call print_reg
    mov si, str_entries
    call print

.done:
    pop es
    pop ax
    pop si
    ret


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
    ; Переводим байты (edx:ebx) в Мегабайты (Деление на 2 ** 20)
    shrd ebx, edx, 20  ; Сдвигаем ebx на 20 бит, заполняя верх ebx из edx
    shr edx, 20        ; Сдвигаем edx на 20 бит

    ; Результат в ebx (МБ | До 64 ГБ)
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
    mov si, str_mb
    call print

.done:
    pop ax
    pop si
    pop es
    ret

; Строки
str_memory_map: db 'Memory Map: ', 0
str_entries:    db ' entries', 0
str_free_ram:   db 'Free RAM: ', 0
str_mb:         db ' MB', 0