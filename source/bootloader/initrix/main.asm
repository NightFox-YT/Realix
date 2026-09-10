; © Realix > Initrix (Stage 2)
; (15.08.26) v0.12
; ================
; ❗️ Загружается Bootix по адресу INITRIX_LOAD_SEGMENT:INITRIX_LOAD_OFFSET

; Настройка компиляции
bits 16
org 0x0

; Основные константы
%include 'shared/config.asm'

; Настройка DiskAPI
%define DISK_INIT bios_disk_init
%define DISK_READ bios_disk_read


; > Основной код
; Параметры:
;  - dl: номер загрузочного диска (Передаётся из Bootix - stage 1)
main:
    ; Вывод сообщения о запуске инициализации
    mov si, msg_init
    call print

    ; Инициализация драйверов (`DISK_INIT` запоминает переданный номер диска)
    call DISK_INIT
    jc disk_init_error
    call fat12_init

    ; Получение и сохранение объёма доступной "нижней" памяти (до 640 КБ)
    call get_lower_memory
    jc lower_memory_error
    mov [low_memory_kb], ax

    ; Сброс доп. сегмента для след. итераций
    xor ax, ax
    mov es, ax

    ;  - Получение карты памяти (*Доп. сегмент)
    mov di, PCINFO_ADDR + PCINFO_MMAP
    call get_memory_map
    jc memory_map_error
    mov [mmap_entries], bp

    ;  - Получение объёма всей доступной памяти (*Доп. сегмент)
    ;  ! Надежда на то, что bp ещё содержит кол-во записей карты памяти
    mov cx, bp
    mov si, PCINFO_ADDR + PCINFO_MMAP
    call get_usable_memory
    mov [memory_mb], eax
    
    ;  - Экспорт собранных данных в PCINFO (*Доп. сегмент)
    mov eax, [memory_mb]
    mov [es:PCINFO_ADDR + PCINFO_ALL_MEM], eax      ; 32: Размер кол-ва всей памяти (МБ)

    mov ax, [low_memory_kb]
    mov [es:PCINFO_ADDR + PCINFO_LOW_MEM], ax       ; 16: Размер кол-ва "нижней" памяти (КБ)
    mov ax, [mmap_entries]                          ; ++
    mov [es:PCINFO_ADDR + PCINFO_MMAP_ENTRIES], ax  ; 16: Кол-во записей в карте памяти

    movzx ax, byte [curr_drive_num]
    mov [es:PCINFO_ADDR + PCINFO_DRIVE], ax         ; 16: Номер загрузочного диска
    mov ax, 0                                       ; ++
    mov word [es:PCINFO_ADDR + PCINFO_VIDEOMODE], 0 ; 16: Номер видеорежима

    ; Вывод заголовка загрузочного экрана
    call clear_screen
    mov si, str_title
    call print
    call print_beep_char

    ; Вывод диагностической информации о памяти
    mov ax, [low_memory_kb]
    call show_lower_memory
    call print_new_line

    mov eax, [memory_mb]
    call show_usable_memory
    call print_new_line

    mov cx, [mmap_entries]
    call show_map_entries_cnt
    call print_new_line
    call print_new_line

    ; Переход в модуль выбора ядра
    jmp boot_switcher


; > Ошибка 5
lower_memory_error:
    mov si, err_get_lower_memory
    jmp error_handler

; > Ошибка 6
memory_map_error:
    mov si, err_get_memory_map
    jmp error_handler

; > Ошибка 7
disk_init_error:
    mov si, err_disk_init
    jmp error_handler

; > Обработчик ошибок
; TODO: Сделать его "мягче", чтобы не любая ошибка влекла перезагрузку
; Параметры:
;  - si: сообщение об ошибке
error_handler:
    ; Вывод сообщения и ожидание нажатия
    call print
    call wait_key

    ; Аппаратный сброс процессора через вектор BIOS
    jmp 0xFFFF:0

; Подключение модулей
%include 'bios-api/disk/read.asm'
%include 'bios-api/io/print.asm'
%include 'bios-api/io/print_reg.asm'
%include 'bios-api/io/screen.asm'
%include 'bios-api/io/cursor.asm'
%include 'bios-api/memory/high.asm'
%include 'bios-api/memory/low.asm'
%include 'bios-api/keyboard.asm'
%include 'bios-api/rtc.asm'
%include 'bios-api/graphics.asm'
%include 'display/memory.asm'
%include 'filesystem/fat12/file_load.asm'
; %include 'drivers/network_rtl8139.asm'
%include 'bootloader/initrix/switcher.asm'

; Сообщения
msg_init:  db '[+] Initializing...', ENTER, 0
str_title:
    db '     Realix ', OS_VERSION, ENTER
    db '(C) NightFox developer', ENTER, ENTER, 0

; Сообщения о предупреждениях и ошибках
msg_warn_no_nic:      db '[*] Network card RTL8139 not found, networking disabled.', ENTER, 0
err_disk_init:        db '[!] E7: Disk init failed!', ENTER, 0
err_get_lower_memory: db '[!] E5: Get lower memory failed (int 12h)!', ENTER, 0
err_get_memory_map:   db '[!] E6: Get memory map failed (int 15h)!', ENTER, 0

; Временная информация о ПК
memory_mb:     dd 0
low_memory_kb: dw 0
mmap_entries:  dw 0