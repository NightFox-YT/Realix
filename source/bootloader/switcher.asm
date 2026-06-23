; © Realix > Switcher CPU modes
; (13.06.26) v0.06
; ================
; ❗️ Зависимости: bootloader/initrix.asm (+kernel16/io)

; Настройка компиляции
bits 16

; Основные константы
%include 'config.asm'


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
    mov si, kernel_filename
    mov cx, KERNEL_LOAD_SEGMENT
    mov bx, KERNEL_LOAD_OFFSET
    mov dl, [boot_drive_num]
    call file_open

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

; Global Descriptor Table
align 4
gdt_start:
    ; Null-дескриптор
    dd 0x0, 0x0

gdt_code:
    ; Дескриптор кода (Смещение 0x08)
    dw 0xFFFF, 0x0, 0x9A00, 0x00CF
    
gdt_data:
    ; Дескриптор данных (Смещение 0x10)
    dw 0xFFFF, 0x0, 0x9200, 0x00CF
gdt_end:
    gdt_descriptor:
        dw gdt_end - gdt_start - 1
        dd gdt_start


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

    ; Вывод 'DONE' напрямую в видеопамять (0xB8000) для проверки
    mov byte [0xB8000], 'D'
    mov byte [0xB8001], 0x0A
    mov byte [0xB8002], 'O'
    mov byte [0xB8003], 0x0A
    mov byte [0xB8004], 'N'
    mov byte [0xB8005], 0x0A
    mov byte [0xB8006], 'E'
    mov byte [0xB8007], 0x0A

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