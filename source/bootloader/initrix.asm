; © Realix > Initrix (Stage 2 Bootloader)
; (14.06.26) v0.06
; ================

; Настройка компиляции
bits 16
org 0x0

; Основные константы
%include 'shared/config.asm'

main:
    ; Сохраняем номер диска, переданного из bootix
    mov [boot_drive_num], dl

    ; Инициализация драйверов
    call disk_init
    call fat12_init
    call net_init
    ;jc network_card_error  ; Уберите комментарий, если в ВМ есть это карта
    
    mov si, msg_init
    call print

    ; Получение объёма доступной "нижней" памяти (до 640 КБ, INT 12h)
    call get_lower_memory
    jc lower_memory_error
    mov [low_memory_kb], ax

    ; Получение карты памяти (E820 | INT 15h, с сбросом доп. сегмента)
    xor cx, cx
    mov es, cx
    mov di, PCINFO_ADDR + 5
    call get_memory_map
    jc memory_map_error

    ; Экспорт собранных данных (*Доп. сегмент 0x0)
    mov ax, [low_memory_kb]
    mov dl, [boot_drive_num]
    mov word [es:PCINFO_ADDR], ax      ; Размер "нижней" памяти (КБ)
    mov byte [es:PCINFO_ADDR + 2], dl  ; Номер загрузочного диска
    mov word [es:PCINFO_ADDR + 3], bp  ; Кол-во записей в карте памяти

    call cmd_cls
    mov si, str_title
    call print
    call print_beep_char

    ; Вывод диагностической информации о памяти на экран
    call show_lower_memory
    call print_new_line

    call show_free_memory
    call print_new_line

    call show_map_entries_cnt
    call print_new_line
    call print_new_line

    ; Переходим в след. модуль
    jmp boot_switcher

.halt:
    cli
    hlt
    jmp $

network_card_error:
    mov si, err_network_card_not_found
    jmp error_handler

lower_memory_error:
    mov si, err_get_lower_memory
    jmp error_handler

memory_map_error:
    mov si, err_get_memory_map
    jmp error_handler

; > Обработчик ошибок
; Параметры:
;  - si: сообщение об ошибке
error_handler:
    call print

    ; Ожидание нажатия
    mov ah, 0
    int 0x16

    ; Аппаратный сброс процессора через вектор BIOS
    jmp 0xFFFF:0

; Подключение модулей
%include 'kernel16/io/print.asm'
%include 'kernel16/io/print_ctrl.asm'
%include 'kernel16/io/print_reg.asm'
%include 'kernel16/shell/cmd_cls.asm'
%include 'bios-api/disk/read.asm'
%include 'bios-api/fat12/file_load.asm'
%include 'bios-api/memory/high.asm'
%include 'bios-api/memory/low.asm'
%include 'bios-api/video/vga.asm'
%include 'network/rtl8139.asm'
%include 'bootloader/switcher.asm'

; Сообщения и строки
msg_init: db '[+] Initializing...', ENTER, 0

err_get_memory_map:         db '[!] Get memory map failed (int 15h)!', 0
err_get_lower_memory:       db '[!] Get lower memory failed (int 12h)!', 0
err_network_card_not_found: db '[!] Network card Realtek RTL8139 not found!', 0

str_title:
    db '     Realix v0.07', ENTER,
    db '(C) NightFox developer', ENTER, ENTER, 0

; Данные о ПК и Kernel
boot_drive_num:  db 0
low_memory_kb:   dw 0