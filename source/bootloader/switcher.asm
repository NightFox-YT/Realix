; © Realix > Switcher CPU modes
; (21.06.26) v0.07
; ================
; ❗️ Зависимости: bootloader/initrix.asm (+kernel16/io)

; Настройка компиляции
bits 16

; Основные константы
%include 'shared/config.asm'


boot_switcher:
    mov si, str_choose_mode
    call print

.wait_key:
    ; Ожидание нажатия
    mov ah, 0x00
    int 0x16

    ; Варианты выбора
    cmp al, '1'
    je .load_kernel16
    cmp al, '2'
    je .load_32bit

    ; Нажали что-то другое - возвращаемся в цикл
    jmp .wait_key

; > Ветка Real Mode (16 bit)
.load_kernel16:
    call print_new_line
    mov si, msg_loading_16
    call print

    ; Чтение файла ядра с диска
    mov si, kernel16_filename
    mov cx, KERNEL_LOAD_SEGMENT
    mov bx, KERNEL_LOAD_OFFSET
    mov dl, [boot_drive_num]
    call file_load

    ; Передача собранной структуры данных в ядро и настройка сегментов
    mov ax, KERNEL_LOAD_SEGMENT
    mov ds, ax
    mov es, ax
    mov di, PCINFO_ADDR

    jmp KERNEL_LOAD_SEGMENT:KERNEL_LOAD_OFFSET


; > Ветка Protected Mode (32-bit)
.load_32bit:
    call print_new_line
    mov si, msg_loading_32
    call print

    ; Чтение файла 32-битного ядра с диска
    mov si, kernel32_filename
    mov cx, KERNEL_LOAD_SEGMENT
    mov bx, KERNEL_LOAD_OFFSET
    mov dl, [boot_drive_num]
    call file_load

    ; Динамически вычисляем физический адрес GDT перед загрузкой
    xor eax, eax
    mov ax, ds
    shl eax, 4                     ; Преобразуем `ds` в линейный адрес (сегмент * 16)
    add eax, gdt_start             ; Прибавляем смещение таблицы GDT
    mov [gdt_descriptor + 2], eax  ; Записываем получившийся адрес в дескриптор таблицы

    ; Динамически вычисляем физический адрес pmode_entry
    xor eax, eax
    mov ax, ds
    shl eax, 4                      ; Преобразуем `ds` в линейный адрес (сегмент * 16)
    add eax, pmode_entry            ; Прибавляем смещение метки `pmode_entry`
    mov [pmode_target_offset], eax  ; Записываем адрес в структуру памяти для дальнего перехода

    ; Включаем A20 (С отключением прерываний)
    cli
    in al, 0x92   ; Читаем состояние системного порта 0x92
    and al, 0xFE  ; Сбрасываем 0-й бит (бит аппаратного сброса), чтобы случайно не перезагрузиться
    or al, 2      ; Устанавливаем во 2-й бит единицу (Fast A20 gate)
    out 0x92, al  ; Отправляем обратно в порт
    
    ; Загружаем GDT
    lgdt [gdt_descriptor]

    ; Включаем Protected Mode
    mov eax, cr0
    or eax, 0x00000001
    mov cr0, eax

    ; Выполняем 32-битный дальний прыжок через структуру в памяти.
    jmp dword far [pmode_target]


; > Структура-указатель для совершения дальнего перехода в 32-битный сегмент кода
pmode_target:
    pmode_target_offset: dd 0     ; Физический адрес pmode_entry (заполняется динамически)
    pmode_target_sel:    dw 0x08  ; Селектор кода в GDT (gdt_code)


; Временный GDT для загрузчика
align 4

; 0: Null дескриптор
gdt_start:
    dd 0x0, 0x0

; (Ring 0) 1: Дескриптор кода (Смещение 0x08)
gdt_code:
    dw 0xFFFF     ; Лимит (Нижние 16 бит)
    dw 0x0000     ; Адрес начала (Нижние 16 бит)
    db 0x00       ; Адрес начала (Средние 8 бит)
    db 10011010b  ; Access Byte
    db 11001111b  ; Flags (4 бита) + Лимит (Старшие 4 бита)
    db 0x00       ; Адрес начала (Старшие 8 бит)

; (Ring 0) Дескриптор данных (Смещение 0x10)
gdt_data:
    dw 0xFFFF     ; Лимит (Нижние 16 бит)
    dw 0x0000     ; Адрес начала (Нижние 16 бит)
    db 0x00       ; Адрес начала (Средние 8 бит)
    db 10010010b  ; Access Byte
    db 11001111b  ; Flags (4 бита) + Лимит (Старшие 4 бита)
    db 0x00       ; Адрес начала (Старшие 8 бит)
    
gdt_end:
    ; Структура-указатель для LGDT
    gdt_descriptor:
        dw gdt_end - gdt_start - 1  ; Лимит (Размер GDT)
        dd gdt_start                ; Адрес начала DGT


; > Точка входа в 32-битный режим
bits 32
pmode_entry:
    ; Настройка 32-битных сегментов данных
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; Настройка стека
    mov ebp, 0x90000
    mov esp, ebp

    ; Передача управления Rust-ядру
    mov ebx, PCINFO_ADDR
    mov eax, KERNEL32_PHYS_ADDR
    jmp eax

    ; Вывод '!' 
    mov byte [0xB8000], '!'
    mov byte [0xB8001], 0x04

    ; Остановка CPU (Ещё нет ядра Rust)
    cli
    hlt
    jmp $


; Сообщения и строки (16 бит для строковых данных)
bits 16

str_choose_mode: 
    db '[+] Select OS Mode:', ENTER
    db '  [1] 16-bit Real Mode', ENTER
    db '  [2] 32-bit Protected Mode (Rust)', ENTER, 0

msg_loading_16: db '[+] Loading 16-bit kernel.', ENTER, 0
msg_loading_32: db '[+] Entering 32-bit Protected Mode.', ENTER, 0

; Переменные
kernel16_filename: db 'KERNEL16BIN'
kernel32_filename: db 'KERNEL32BIN'