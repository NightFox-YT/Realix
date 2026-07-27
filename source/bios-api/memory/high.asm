; © Realix > High memory (Memory Map)
; (27.07.26) v0.1
; ================

; Основные константы
%include 'shared/config.asm'

; Константы карты памяти (! MAX_ENTRIES идёт из Rust)
E820_ENTRY_SIZE  equ 24
E820_MAX_ENTRIES equ 64

; > Получение карты памяти: int 0x15 (E820)
; Параметры:
;  - es:di: адрес сохранения таблицы карты памяти
; Вывод:
;  - bp: количество успешно прочитанных записей
;  - di: указатель на конец таблицы
;  - CF (Carry Flag): 0 (Успех), 1 (Ошибка)
get_memory_map:
    push eax
    push ebx
    push ecx
    push edx

    ; Установка начальных параметров, счётчика записей
    xor ebx, ebx
    xor bp, bp

.next:
    ; Не позволяем BIOS записать больше, чем вмещает `E820Map` в kernel32
    cmp bp, E820_MAX_ENTRIES
    jae .done

    mov eax, 0xe820
    mov edx, 0x0534D4150       ; "SMAP"
    mov ecx, E820_ENTRY_SIZE   ; Запрашиваем 24 байта
    mov [es:di + 20], dword 1  ; Делаем запись валидной для ACPI 3.X (Она сможет перезаписаться)

    ; Установленный CF - "функция не поддерживается" или "конец списка"
    int 0x15
    jc .carry_result

    ; В случае успеха eax должен быть сброшен в "SMAP"
    cmp eax, 0x0534D4150
    jne .fail

    ; Есть ли расширенные атрибуты ACPI 3.X? (BIOS может вернуть 20 или 24 байта)
    cmp ecx, 20
    jb .skipentry

    mov eax, [es:di + 8]  ; Получаем младшие 32 бита длины области памяти
    or eax, [es:di + 12]  ; Проверяем старшие 32 бита на 0
    jz .skipentry         ; Если 64-битная длина равна 0, пропустить запись

    ; Если запись не превышает макс. размер в 24 байта, принимаем
    cmp ecx, E820_ENTRY_SIZE
    jb .accept

    ; Есть атрибуты: очищен ли бит "игнорировать эти данные"?
    test byte [es:di + 20], 1
    jz .skipentry

.accept:
    ; Получена хорошая запись, переход к следующему месту хранения
    inc bp
    add di, E820_ENTRY_SIZE

.skipentry:
    ; Если ebx сбрасывается в 0, конец список достигнут
    test ebx, ebx
    jne .next

.done:
    clc
    jmp .return

.carry_result:
    ; CF при собранных записях означает "конец списка достигнут"
    test bp, bp
    jnz .done

.fail:
    ; Выход по ошибке "Функция не поддерживается"
    stc

.return:
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

    ; Настраиваем сегмент es под `PCINFO`
    xor ax, ax
    mov es, ax

    ; Выводим информацию о кол-ве записей карты памяти
    mov si, str_memory_map
    call print
    mov ax, word [es:PCINFO_ADDR + PCINFO_ENTRIES]
    call print_dec16
    mov si, str_entries
    call print

.done:
    pop es
    pop ax
    pop si
    ret


; > Получение общей длины всех отрезкой памяти по её карте
; Параметры:
;  - es:di: Указатель на `PCINFO`
; Вывод:
;  - ax: Число свободной памяти (МБ), насыщается на 65535 при переполнении
get_usable_memory:
    push ebx
    push ecx
    push edx
    push si

    ; Подготовка параметров
    xor ebx, ebx
    xor edx, edx

    ; Читаем количество записей (Если 0 - Выходим)
    mov cx, [es:di + PCINFO_ENTRIES]
    test cx, cx
    jz .empty

    ; Доп. защита от повреждённого `PCINFO`
    ; (не больше, чем реально бывает записей)
    cmp cx, E820_MAX_ENTRIES
    jbe .count_valid
    mov cx, E820_MAX_ENTRIES

.count_valid:
    ; Адрес первой записи E820
    mov si, di
    add si, PCINFO_MAP

.loop:
    ; Проходим по свободным регионам памяти
    cmp dword [es:si + 16], 1
    jne .skip_entry

    ; Прибавляем 64-битную длину региона к edx:ebx
    add ebx, [es:si + 8]   ; Смещение +8: Младшие 32 бита длины
    adc edx, [es:si + 12]  ; Смещение +12: Старшие 32 бита длины + флаг переноса

.skip_entry:
    ; Переход к следующей записи
    add si, E820_ENTRY_SIZE
    loop .loop

.empty:
    ; Переводим байты (edx:ebx) в Мегабайты (Деление на 2 ** 20)
    shrd ebx, edx, 20  ; Сдвигаем ebx на 20 бит, заполняя верх ebx из edx
    shr edx, 20        ; Сдвигаем edx на 20 бит

    ; Если результат не влезает в ax, насыщаем значение вместо переполнения
    test edx, edx
    jnz .saturate
    cmp ebx, 0xFFFF
    ja .saturate

    mov ax, bx
    jmp .done

.saturate:
    mov ax, 0xFFFF

.done:
    pop si
    pop edx
    pop ecx
    pop ebx
    ret


; > Вывод кол-ва свободной памяти в текстовом режиме
; ❗️ Зависимости: kernel16/print.asm, kernel16/print_reg.asm
show_usable_memory:
    push di
    push es
    push si
    push ax

    ; Считаем и выводим кол-во свободной памяти
    xor ax, ax
    mov es, ax
    mov di, PCINFO_ADDR
    call get_usable_memory

    ; (ax содержит нужное число после `call get_usable_memory`)
    mov si, str_usable_ram
    call print
    call print_dec16
    mov si, str_mb
    call print

.done:
    pop di
    pop ax
    pop si
    pop es
    ret

; Строки
str_memory_map: db 'Memory Map: ', 0
str_entries:    db ' entries', 0
str_usable_ram: db 'Usable RAM: ', 0
str_mb:         db ' MB', 0
