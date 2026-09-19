; © Realix > Command: Echo
; (16.08.26) v0.12
; ================
; ❗️ Зависимости: bios-api/io/print, display/memory

; Основные константы
%include 'shared/config.asm'


; > Команда вывода информации о памяти
cmd_meminfo:
    push ax
    push cx
    push si
    push es

    ; Сброс доп. сегмента
    xor ax, ax
    mov es, ax

    ; Сбор информации о памяти и "расстаскивание" для вызова
    call get_lower_memory
    call show_lower_memory
    call print_new_line

    mov cx, [es:PCINFO_ADDR + PCINFO_MMAP_ENTRIES]
    mov si, PCINFO_ADDR + PCINFO_MMAP
    call get_usable_memory
    call show_usable_memory
    call print_new_line

    ; (cx уже передан ранее)
    call show_map_entries_cnt
    call print_new_line
    call print_new_line

    ; Небольшая заметка
    mov si, note_meminfo
    call print

    pop es
    pop si
    pop cx
    pop ax
    ret


; Сообщение
note_meminfo: db 'Note: In Real mode you can access only low RAM.', 0