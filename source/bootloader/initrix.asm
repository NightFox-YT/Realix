; © Realix > Initrix (Stage 2)
; (27.07.26) v0.1
; ================
; ❗️ Загружается Bootix по адресу INITRIX_LOAD_SEGMENT:0, номер диска в dl

; Настройка компиляции
bits 16
org 0x0

; Основные константы
%include 'shared/config.asm'

main:
    ; Инициализация драйверов 
    ; (`disk_init` запоминает номер диска, переданный из Bootix)
    call disk_init
    jc disk_init_error
    call fat12_init

    mov si, msg_init
    call print

    ; Сброс дополнительного сегмента
    xor ax, ax
    mov es, ax

    ; Получение объёма доступной "нижней" памяти (до 640 КБ)
    call get_lower_memory
    jc lower_memory_error
    mov [low_memory_kb], ax

    ; Получение карты памяти (int 0x15 | E820, *Доп. сегмент)
    mov di, PCINFO_ADDR + PCINFO_MAP
    call get_memory_map
    jc memory_map_error

    ; Экспорт собранных данных (*Доп. сегмент)
    mov ax, [low_memory_kb]
    mov dl, [disk_current_drive]
    mov word [es:PCINFO_ADDR + PCINFO_LOW_MEM], ax  ; Размер "нижней" памяти (КБ)
    mov byte [es:PCINFO_ADDR + PCINFO_DRIVE], dl    ; Номер загрузочного диска
    mov word [es:PCINFO_ADDR + PCINFO_ENTRIES], bp  ; Кол-во записей в карте памяти

    ; Вывод заголовка Initrix
    call cmd_cls
    mov si, str_title
    call print
    call print_beep_char

    ; Вывод диагностической информации о памяти на экран
    call show_lower_memory
    call print_new_line

    call show_usable_memory
    call print_new_line

    call show_map_entries_cnt
    call print_new_line
    call print_new_line

    ; Инициализация сетевой карты (необязательно)
    call net_init
    jnc .nic_ready

    mov si, msg_warn_no_nic
    call print

.nic_ready:
    call print_new_line

    ; Переходим в след. модуль
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
%include 'kernel16/commands/base/cls.asm'
%include 'bios-api/disk/read.asm'
%include 'bios-api/fat12/file_load.asm'
%include 'bios-api/memory/high.asm'
%include 'bios-api/memory/low.asm'
%include 'network/rtl8139.asm'
%include 'bootloader/switcher.asm'

; Сообщения и строки
msg_init:             db '[+] Initializing...', ENTER, 0
err_get_lower_memory: db '[!] E5: Get lower memory failed (int 12h)!', 0
err_get_memory_map:   db '[!] E6: Get memory map failed (int 15h)!', 0

; Предупреждения
msg_warn_no_nic: db '[!] Network card RTL8139 not found, networking disabled.', ENTER, 0

; Сообщения об ошибках
err_disk_init: db '[!] E7: Disk init failed!', ENTER, 0

str_title:
    db '     Realix ', OS_VERSION, ENTER
    db '(C) NightFox developer', ENTER, ENTER, 0

; Данные о ПК и Kernel
low_memory_kb:   dw 0