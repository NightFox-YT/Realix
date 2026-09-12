; © Realix > Switcher CPU modes
; (15.08.26) v0.12
; ================
; ❗️ Зависимости: Подключается напрямую из initrix.asm (%include),
;                 bios-api/io/* (Кроме clear); filesystem/fat12/file_load,
;                 bios-api/rtc, bios-api/keyboard

; Настройка компиляции
bits 16

; Основные константы
%include 'shared/config.asm'


; > Точка входа в модуль
boot_switcher:
    ; Вывод заголовка выбора режима
    mov si, msg_choose_mode
    call print

    ; Выключаем курсор и получаем номер строки, где будет таймер
    call cursor_set_off
    call get_cursor_pos
    mov [.timer_cursor_y], dh

.timer_start:
    ; Получаем начальное значение тиков (в eax)
    call get_ticks_value
    mov [last_tick], eax

    ; Вывод сообщения о запущенном таймере
    mov si, msg_timer
    call print

.timer_loop:
    ; Проверка нажатия клавиши (из буфера)
    call keyboard_poll
    jnz .key_pressed

    ; Получаем текущее кол-во тиков и считаем сколько прошло
    call get_ticks_value
    sub eax, dword [last_tick]

    ; Если прошла 1 сек. (~18 тиков), обновляем таймер
    cmp eax, 18
    jae .update_timer

    ; Ждём след. прерывание для продолжения
    hlt
    jmp .timer_loop

.update_timer:
    ; Обновляем наши внутренние счётчики
    add word [last_tick], 18
    sub byte [remaining_sec], 1

    ; Ставим курсор на место числа
    ; (36 просто посчитано, немного хардкод ;) )
    mov dh, [.timer_cursor_y]
    mov dl, 36
    call set_cursor_pos

    ; Выводим сколько осталось секунд
    mov al, [remaining_sec]
    call print_hex8

    ; Проверяем наш счётчик
    cmp byte [remaining_sec], 0
    je load_kernel32

    jmp .timer_loop

.key_pressed:
    ; Варианты выбора
    cmp al, '1'
    je load_kernel16
    cmp al, '2'
    je load_kernel32
    cmp al, '3'
    je load_kernel32_video

    jmp .timer_loop

.timer_cursor_y: db 0


; > Ветка Real Mode (16 bit)
load_kernel16:
    ; Включаем курсор
    call cursor_set_on

    ; Вывод сообщения о начале загрузки ядра
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
    mov dl, [curr_drive_num]
    mov di, PCINFO_ADDR

    ; Настройка сегментов под ядро
    mov ax, KERNEL_LOAD_SEGMENT
    mov ds, ax
    mov es, ax

    jmp KERNEL_LOAD_SEGMENT:KERNEL_LOAD_OFFSET


; > Ветка Protected Mode с Video Mode (32-bit)
load_kernel32_video:
    ; Включение видеорежима
    call enable_vga_videomode

    ; Обнуление сегмента под `PCINFO` для записи
    push es
    push ax
    xor ax, ax
    mov es, ax

    mov word [es:PCINFO_ADDR + PCINFO_VIDEOMODE], 1

    pop ax
    pop es

    ; Переход к стандартной загрузке
    jmp load_kernel32


; > Ветка Protected Mode (32-bit)
load_kernel32:
    ; Включаем курсор
    call cursor_set_on

    ; Вывод сообщения о начале перехода в Protected Mode
    call print_new_line
    call print_new_line
    mov si, msg_entering_32
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
    db '[?] Select OS Mode:', ENTER
    db '  [1] 16-bit Real Mode (NASM)', ENTER
    db '  [2] 32-bit Protected Mode (Rust)', ENTER
    db '  [3] 32-bit Video Mode (Rust Experiment)', ENTER, 0

msg_timer:       db '  (Auto: 32-bit will be selected in 10 seconds)', 0
msg_loading_16:  db '[+] Loading 16-bit kernel.', ENTER, 0
msg_entering_32: db '[+] Entering 32-bit Protected Mode.', ENTER, 0

; Переменные
kernel16_filename: db 'KERNEL16BIN'
kernel32_filename: db 'KERNEL32BIN'

remaining_sec: db 10
last_tick:     dd 0