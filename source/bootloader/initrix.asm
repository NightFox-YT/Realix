; © Realix > Initrix
; (07.06.26) v0.05
; ================
; ❗️ Бинарный файл должен быть размером <=64 КБ (Ограничение из Bootix)

; Настройка компиляции
bits 16
org 0x0

; Символы
%define ENTER 0x0D, 0x0A

; > Основной код
main:
    ; Сохраняем номер диска, переданного из bootix
    mov [boot_drive], dl

    ; "Инициализация"
    mov si, msg_init
    call print

    ; Получение и сохранение объёма доступной "нижней" памяти (до 640 КБ)
    call get_lower_memory
    jc lower_memory_error
    mov [low_memory_kb], ax

    ; Вызываем чтение карты памяти (С сбросом доп. сегмента на 0x0)
    xor cx, cx
    mov es, cx
    mov di, PCINFO_ADDR + 5
    call get_memory_map
    jc memory_map_error

    ; Сохраняем оставшуюся собранную информацию (*Доп. сегмент 0x0)
    mov ax, [low_memory_kb]
    mov word [es:PCINFO_ADDR], ax      ; Размер нижней памяти (КБ)
    mov byte [es:PCINFO_ADDR + 2], dl  ; Номер загрузочного диска
    mov [es:PCINFO_ADDR + 3], bp       ; Кол-во записей в карте памяти

    ; Очистка экрана
    call clear_screen

    ; Заголовок экрана загрузки (+Короткий звук)
    mov si, str_title
    call print
    call txt_beep

    ; Показ строк с информацией о памяти
    call show_memory_info

    mov si, double_new_line
    call print

    ; Загрузка 16 битного ядра
    call load_kernel16


; > Вывод на экран информации о памяти ПК
; (Часть загрузочного экрана)
show_memory_info:
    push si
    push ax
    push es

    ; Выводим информацию о кол-ве "нижней" памяти
    mov si, str_low_ram
    call print
    mov ax, [low_memory_kb]
    call print_reg
    mov si, str_kb_w_max
    call print

    mov si, new_line
    call print

    ; Считаем и выводим кол-во свободной памяти
    xor cx, cx
    mov es, cx
    mov di, PCINFO_ADDR
    call get_free_memory
    
    mov si, str_free_ram
    call print
    call print_reg  ; ax содержит нужное число после `call get_free_memory`
    mov si, str_mb
    call print

    mov si, new_line
    call print

    ; Выводим информацию о кол-ве записей карты памяти
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

; > Загрузка в память и передача управления 16 битному ядру
; ❗️ Обратный процесс не обратим.
load_kernel16:
    ; "Загрузка ядра..."
    mov si, msg_loading
    call print

    ; Загрузка ядра с диска
    mov si, kernel_filename
    mov cx, KERNEL_LOAD_SEGMENT
    mov bx, KERNEL_LOAD_OFFSET
    mov dl, [boot_drive]
    call file_open

    ; Настройка сегментов и регистров под ядро
    mov ax, KERNEL_LOAD_SEGMENT
    mov ds, ax
    mov es, ax
    mov di, PCINFO_ADDR

    ; Передача управления ядру
    jmp KERNEL_LOAD_SEGMENT:KERNEL_LOAD_OFFSET

; > Остановка CPU
halt:
    cli
    hlt
    jmp $

; > Ошибка получения кол-ва "нижней" памяти
lower_memory_error:
    mov si, err_get_lower_memory
    jmp error_handler

; > Ошибка получения карты памяти
memory_map_error:
    mov si, err_get_memory_map
    jmp error_handler

; > Обработчик ошибок
; Параметры:
;  - si: сообщение об ошибке
error_handler:
    ; Вывод сообщения и ожидание нажатия
    call print
    mov ah, 0
    int 0x16

    ; Переход в вектор сброса BIOS
    jmp 0xFFFF:0

; Подключение модулей
%include 'kernel16/print.asm'
%include 'kernel16/print_reg.asm'
%include 'kernel16/clear_screen.asm'
%include 'disk/read.asm'
%include 'fat12/file_open.asm'
%include 'memory/get_free.asm'
%include 'memory/get_lower.asm'
%include 'memory/get_map.asm'
%include 'drivers/sounds.asm'
%include 'drivers/vga.asm'

; Сообщения
msg_init:             db '[+] Initializing.', ENTER, 0
msg_loading:          db '[+] Loading kernel.', ENTER, 0
err_get_memory_map:   db '[!] Get memory map failed (int 15h)!', 0
err_get_lower_memory: db '[!] Get lower memory failed (int 12h)!', 0

; Строки загрузочного экрана
str_title:
    db '     Realix v0.05', ENTER,
    db '(C) NightFox developer', ENTER, ENTER, 0

str_low_ram:    db 'Low RAM: ', 0
str_kb_w_max:   db ' KB / 640 KB', 0
str_free_ram:   db 'Free RAM: ', 0
str_mb:         db ' MB', 0
str_memory_map: db 'Memory Map: ', 0
str_entries:    db ' entries', 0

; Вспомогательные строки
char_beep:       db 0x07, 0
new_line:        db ENTER, 0
double_new_line: db ENTER, ENTER, 0

; Данные о ПК
boot_drive:    db 0
low_memory_kb: dw 0
PCINFO_ADDR    equ 0x4500

; Переменные и константы Kernel
kernel_filename: db 'KERNEL  BIN'

KERNEL_LOAD_SEGMENT equ 0x1000
KERNEL_LOAD_OFFSET  equ 0x0000