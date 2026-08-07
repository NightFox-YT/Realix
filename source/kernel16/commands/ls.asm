; © Realix > Command: LS
; (27.07.26) v0.1
; ================
; ❗️ Зависимости: bios-api/disk/read.asm, bios-api/fat12/init.asm,
;                 kernel16/io: print, print_ctrl, print_reg

; ❗️ Требуется инициализация FAT12 через `fat12_init`
%include "bios-api/fat12/init.asm"

; Основные константы
%include 'shared/config.asm'

; Раскладка записи корневого каталога FAT12 (32 байта)
; ❗️ Те же смещения используются в bios-api/fat12/file_load.asm
DIRENT_LEN      equ 32    ; Размер одной записи
DIRENT_NAME     equ 0     ; Имя файла (8 байт, формат 8.3)
DIRENT_EXT      equ 8     ; Расширение файла (3 байта)
DIRENT_ATTR     equ 11    ; Атрибуты файла (byte)
DIRENT_FILESIZE equ 28    ; Размер файла (dword)
DIRENT_FREE     equ 0x00  ; Первый байт имени: запись свободна (конец каталога)
DIRENT_DELETED  equ 0xE5  ; Первый байт имени: файл удалён

; Атрибуты записи каталога (Пропускаемые при выводе)
ATTR_VOLUME_ID equ 0x08
ATTR_DIRECTORY equ 0x10
ATTR_LFN       equ 0x0F

; Длины полей имени в формате 8.3
NAME_LEN equ 8
EXT_LEN  equ 3

; > Команда вывода списка файлов в корневом каталоге
cmd_ls:
    push ax
    push bx
    push cx
    push dx
    push si
    push di
    push es

    ; Читаем корневой каталог в память
    xor ax, ax
    mov es, ax
    mov ax, [root_dir_lba]
    mov cx, [root_dir_size]
    mov bx, FAT_BUFFER_ADDR
    call disk_read

    ; Подготовка к выводу информации о файлах
    xor bx, bx               ; Счётчик пройденных записей
    mov di, FAT_BUFFER_ADDR  ; Смещение текущей записи

    mov si, str_ls_header
    call print
    call print_new_line

.loop:
    ; Проверка на конец каталога
    cmp bx, [dir_entries]
    jae .done

    ; Проверка: Запись не пустая (Первый байт имени)
    mov al, [es:di + DIRENT_NAME]
    cmp al, DIRENT_FREE
    je .done             ; Конец каталога
    cmp al, DIRENT_DELETED
    je .next_entry       ; Удалённый файл

    ; Проверка атрибутов (Volume Label, Directory, Long File Name)
    mov al, [es:di + DIRENT_ATTR]
    test al, ATTR_VOLUME_ID
    jnz .next_entry
    test al, ATTR_DIRECTORY
    jnz .next_entry
    test al, ATTR_LFN
    jnz .next_entry

    ; Подготовка к выводу имени файла (формат 8.3)
    mov cx, NAME_LEN
    mov si, di

.print_name:
    ; Читаем имя файла и выводим
    mov al, [es:si]
    cmp al, ' '
    je .check_ext  ; Пробел -> Имя файла уже закончилось
    call print_char

    ; Переходим к след. символу, если считали 8 > к расширению
    inc si
    loop .print_name
    jmp .print_ext

.check_ext:
    ; Пропускаем оставшиеся пробелы в имени
    inc si
    loop .print_name

.print_ext:
    ; Читаем первый символ расширения файла
    mov al, [es:di + DIRENT_EXT]
    cmp al, ' '
    je .print_size  ; Пробел -> Расширения нет

    ; Печатаем точку-разделитель
    mov al, '.'
    call print_char

    ; Подготовка к выводу расширения файла
    mov cx, EXT_LEN
    mov si, di
    add si, DIRENT_EXT

.print_ext_loop:
    ; Читаем расширение файла и выводим
    mov al, [es:si]
    cmp al, ' '
    je .print_size
    call print_char

    ; Переходим к след. символу, если считали 3 > идём дальше
    inc si
    loop .print_ext_loop

.print_size:
    ; Разделитель перед размером файла
    mov si, str_tab
    call print

    ; Считывание 32-битного размера
    mov ax, [es:di + DIRENT_FILESIZE]      ; Младшие 16 бит размера
    mov dx, [es:di + DIRENT_FILESIZE + 2]  ; Старшие 16 бит размера

    ; Если размер не влезает в uint16, выводим как ">64 Kilobytes"
    test dx, dx
    jnz .large_file

    ; Выводим младшие 16 бит размера с префиксом
    call print_dec16

    mov si, str_bytes
    call print
    jmp .next

.large_file:
    mov si, str_large
    call print

.next:
    call print_new_line

.next_entry:
    ; Переход к следующей записи
    add di, DIRENT_LEN
    inc bx
    jmp .loop

.done:
    pop es
    pop di
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret

; Строки
str_ls_header: db 'Files in root directory:', 0
str_tab:       db ' - ', 0
str_bytes:     db ' bytes', 0
str_large:     db '>64 Kilobytes', 0
