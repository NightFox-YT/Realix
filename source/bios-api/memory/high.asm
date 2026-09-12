; © Realix > High memory (Memory Map)
; (15.08.26) v0.12
; ================

; Основные константы
%include 'shared/config.asm'


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


; > Получение общей длины всех отрезкой памяти по её карте
; Параметры:
;  - es:si: Указатель на начало карты памяти
;  - cx: Кол-во записей в карте памяти
; Вывод:
;  - eax: Число свободной памяти (До 2 ^ 32 МБ или до 4096 ТБ)
get_usable_memory:
    push ebx
    push edx
    push cx
    push si

    ; Подготовка параметров
    xor ebx, ebx
    xor edx, edx

    ; Смотрим количество записей (Если 0 - Выходим)
    test cx, cx
    jz .empty

    ; Доп. защита от повреждённого `PCINFO` (Не больше, чем ожидается Rust ядром)
    cmp cx, E820_MAX_ENTRIES
    jbe .loop
    mov cx, E820_MAX_ENTRIES

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

    ; Если результат не влезает (Вдруг >4096 ТБ...), насыщаем значение
    test edx, edx
    jnz .saturate

    mov eax, ebx
    jmp .done

.saturate:
    mov eax, 0xFFFFFFFF

.done:
    pop si
    pop cx
    pop edx
    pop ebx
    ret
