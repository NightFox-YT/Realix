; © Realix > Switcher CPU modes
; (06.08.26) v0.11
; ================
; ❗️ Не standalone: Подключается из initrix.asm (%include)
; ❗️ Зависимости: kernel16/io: print & print_new_line; bios-api/fat12/file_load

; Настройка компиляции
bits 16

; Основные константы
%include 'shared/config.asm'


boot_switcher:
    mov si, msg_choose_mode
    call print

    ; Выключаем курсор на время выбора режима
    mov ah, 01h
    mov cx, 2607h
    int 0x10

    ; Получаем текущую строку, где будет таймер (в dh)
    mov ah, 03h
    xor bx, bx
    int 0x10
    mov [.timer_cursor_y], dh

    ; Получаем начальное значение тиков (в cx:dx)
    mov ah, 00h
    int 0x1A
    mov [last_tick_low], dx

    ; Вывод сообщения о таймере
    mov si, msg_timer
    call print

.menu_loop:
    ; Проверка нажатия клавиши (из буфера)
    mov ah, 1h
    int 0x16
    jnz .key_pressed

    ; Получаем текущее кол-во тиков и считаем сколько прошло
    mov ah, 00h
    int 0x1A
    mov ax, dx
    sub ax, [last_tick_low]

    ; Если прошла 1 сек. (~18 тиков), обновляем таймер
    cmp ax, 18
    jae .update_timer

    ; Ждём след. прерывание для продолжения
    hlt
    jmp .menu_loop

.update_timer:
    ; Обновляем наши внутренние счётчики
    add word [last_tick_low], 18
    sub byte [remaining_sec], 1

    ; Ставим курсор на место числа
    ; (36 просто посчитано, хардкод о ма гад)
    mov ah, 02h
    mov dh, [.timer_cursor_y]
    mov dl, 36
    int 0x10

    ; Выводим сколько осталось секунд
    movzx ax, byte [remaining_sec]
    call print_byte

    ; Проверяем наш счётчик
    cmp byte [remaining_sec], 0
    je load_kernel32

    jmp .menu_loop

.key_pressed:
    ; Забираем клавишу из буфера
    mov ah, 00h
    int 0x16

    ; Варианты выбора
    cmp al, '1'
    je load_kernel16
    cmp al, '2'
    je load_kernel32

    jmp .menu_loop

.timer_cursor_y: db 0


; > Ветка Real Mode (16 bit)
load_kernel16:
    ; Включаем курсор
    mov ah, 01h
    mov cx, 0607h
    int 0x10

    call print_new_line
    call print_new_line
    mov si, msg_loading_16
    call print

    ; Чтение файла 16-битного ядра с диска
    mov si, kernel16_filename
    mov cx, KERNEL_LOAD_SEGMENT
    mov bx, KERNEL_LOAD_OFFSET
    xor di, di
    call file_load
    jc error_handler

    ; Передача номера диска и собранной структуры данных в ядро
    ; (Читаем переменные Initrix до смены сегмента ds)
    mov dl, [disk_current_drive]
    mov di, PCINFO_ADDR

    ; Настройка сегментов под ядро
    mov ax, KERNEL_LOAD_SEGMENT
    mov ds, ax
    mov es, ax

    jmp KERNEL_LOAD_SEGMENT:KERNEL_LOAD_OFFSET


; > Ветка Protected Mode (32-bit)
load_kernel32:
    ; Включаем курсор
    mov ah, 01h
    mov cx, 0607h
    int 0x10

    call print_new_line
    call print_new_line
    mov si, msg_loading_32
    call print

    ; Чтение файла 32-битного ядра с диска
    mov si, kernel32_filename
    mov cx, KERNEL_LOAD_SEGMENT
    mov bx, KERNEL_LOAD_OFFSET
    xor di, di
    call file_load
    jc error_handler

    ; Динамически вычисляем физический адрес GDT перед загрузкой
    xor eax, eax
    mov ax, ds
    shl eax, 4                     ; Преобразуем ds в линейный адрес (сегмент * 16)
    add eax, gdt_start             ; Прибавляем смещение таблицы GDT
    mov [gdt_descriptor + 2], eax  ; Записываем получившийся адрес в дескриптор таблицы

    ; Динамически вычисляем физический адрес `pmode_entry`
    xor eax, eax
    mov ax, ds
    shl eax, 4                      ; Преобразуем ds в линейный адрес (сегмент * 16)
    add eax, pmode_entry            ; Прибавляем смещение метки `pmode_entry`
    mov [pmode_target_offset], eax  ; Записываем адрес в структуру памяти для перехода

    ; Включаем A20 (С отключением прерываний)
    cli
    in al, 0x92   ; Читаем состояние системного порта 0x92
    and al, 0xFE  ; Сбрасываем 0-й бит "аппаратного сброса", чтобы случайно не перезагрузиться
    or al, 2      ; Устанавливаем 1-ый бит "Fast A20 gate"
    out 0x92, al  ; Отправляем обратно в порт

    ; Загружаем GDT
    lgdt [gdt_descriptor]

    ; Включаем Protected Mode
    mov eax, cr0
    or eax, 0x00000001
    mov cr0, eax

    ; Выполняем 32-битный дальний прыжок через структуру в памяти
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

    ; Остановка CPU (Если ядро Rust вернуло управление)
    cli
    hlt
    jmp $

; Сообщения и строки (16 бит для строковых данных)
bits 16

msg_choose_mode:
    db '[+] Select OS Mode:', ENTER
    db '  [1] 16-bit Real Mode (NASM)', ENTER
    db '  [2] 32-bit Protected Mode (Rust)', ENTER, 0

msg_timer:      db '  (Auto: 32-bit will be selected in 10 seconds)', 0
msg_loading_16: db '[+] Loading 16-bit kernel.', ENTER, 0
msg_loading_32: db '[+] Entering 32-bit Protected Mode.', ENTER, 0

; Переменные
kernel16_filename: db 'KERNEL16BIN'
kernel32_filename: db 'KERNEL32BIN'

remaining_sec:  db 10
last_tick_low:  dw 0