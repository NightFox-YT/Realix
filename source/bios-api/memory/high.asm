; © Realix > High memory (E820 Memory Map)
; Исправленная версия
; =======================================

%ifndef HIGH_MEMORY_ASM
%define HIGH_MEMORY_ASM

%include 'shared/config.asm'

E820_ENTRY_SIZE   equ 24
E820_MAX_ENTRIES  equ 64


; ============================================================
; ПОЛУЧЕНИЕ КАРТЫ ПАМЯТИ
; ============================================================

; Вход:
;  - ES:DI: буфер карты памяти
;
; Выход:
;  - BP: количество записей
;  - DI: конец данных
;  - CF=0: успех
;  - CF=1: E820 не поддерживается или ошибка первого вызова
get_memory_map:
    push eax
    push ebx
    push ecx
    push edx

    xor ebx, ebx
    xor bp, bp

.next:
    ; Не позволяем BIOS записать больше, чем может принять
    ; структура Rust E820Map.
    cmp bp, E820_MAX_ENTRIES
    jae .success

    mov eax, 0xE820
    mov edx, 0x534D4150
    mov ecx, E820_ENTRY_SIZE

    ; ACPI 3.x extended attributes.
    mov dword [es:di + 20], 1

    int 0x15
    jc .carry_result

    cmp eax, 0x534D4150
    jne .fail

    ; BIOS может вернуть запись размером 20 или 24 байта.
    cmp ecx, 20
    jb .skip

    ; Нулевая длина региона.
    mov eax, [es:di + 8]
    or eax, [es:di + 12]
    jz .skip

    ; Если BIOS вернул extended attributes, проверяем valid bit.
    cmp ecx, 24
    jb .accept

    test byte [es:di + 20], 1
    jz .skip

.accept:
    inc bp
    add di, E820_ENTRY_SIZE

.skip:
    test ebx, ebx
    jne .next

.success:
    clc
    jmp .done

.carry_result:
    ; CF после хотя бы одной записи обычно означает конец списка.
    test bp, bp
    jnz .success

.fail:
    stc

.done:
    pop edx
    pop ecx
    pop ebx
    pop eax
    ret


; ============================================================
; ВЫВОД КОЛИЧЕСТВА ЗАПИСЕЙ
; ============================================================

show_map_entries_cnt:
    push si
    push ax
    push es

    xor ax, ax
    mov es, ax

    mov si, str_memory_map
    call print

    mov ax, [es:PCINFO_ADDR + 3]
    call print_reg

    mov si, str_entries
    call print

    pop es
    pop ax
    pop si
    ret


; ============================================================
; ПОДСЧЁТ USABLE MEMORY
; ============================================================

; Вход:
;  - ES:DI: PCINFO
;
; Выход:
;  - AX: usable memory в мегабайтах, максимум 65535 МБ
get_free_memory:
    push ebx
    push ecx
    push edx
    push si

    xor ebx, ebx
    xor edx, edx

    ; Количество записей хранится как word.
    mov cx, [es:di + 3]
    test cx, cx
    jz .convert

    ; Дополнительная защита от повреждённого PCINFO.
    cmp cx, E820_MAX_ENTRIES
    jbe .count_valid

    mov cx, E820_MAX_ENTRIES

.count_valid:
    mov si, di
    add si, 5

.loop:
    ; Тип 1 — usable memory.
    cmp dword [es:si + 16], 1
    jne .next_entry

    add ebx, [es:si + 8]
    adc edx, [es:si + 12]

.next_entry:
    add si, E820_ENTRY_SIZE
    loop .loop

.convert:
    ; Деление 64-битного EDX:EBX на 2^20.
    shrd ebx, edx, 20
    shr edx, 20

    ; Если результат превышает 65535 МБ, насыщаем значение.
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


; ============================================================
; ВЫВОД USABLE MEMORY
; ============================================================

show_free_memory:
    push es
    push si
    push ax
    push di

    xor ax, ax
    mov es, ax

    mov di, PCINFO_ADDR
    call get_free_memory

    push ax

    mov si, str_free_ram
    call print

    pop ax
    call print_reg

    mov si, str_mb
    call print

    pop di
    pop ax
    pop si
    pop es
    ret


str_memory_map:
    db 'Memory Map: ', 0

str_entries:
    db ' entries', 0

str_free_ram:
    db 'Usable RAM: ', 0

str_mb:
    db ' MB', 0

%endif
