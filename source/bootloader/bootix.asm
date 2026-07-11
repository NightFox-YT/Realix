; © Realix > Bootix
; FAT12 Stage 1 Bootloader
; ========================

bits 16
org 0x7C00


; ============================================================
; FAT12 BIOS PARAMETER BLOCK
; ============================================================

jmp short start
nop

bpb_oem:
    db 'REALIX  '                 ; 8 байт

bpb_bytes_per_sector:
    dw 512

bpb_sectors_per_cluster:
    db 1

bpb_reserved_sectors:
    dw 1

bpb_fat_count:
    db 2

bpb_dir_entries:
    dw 224

bpb_total_sectors:
    dw 2880

bpb_media_type:
    db 0xF0

bpb_sectors_per_fat:
    dw 9

bpb_sectors_per_track:
    dw 18

bpb_heads:
    dw 2

bpb_hidden_sectors:
    dd 0

bpb_large_sectors:
    dd 0


; ============================================================
; EXTENDED BOOT RECORD
; ============================================================

ebr_drive_number:
    db 0

ebr_reserved:
    db 0

ebr_signature:
    db 0x29

ebr_volume_id:
    dd 0x584C4552

ebr_volume_label:
    db 'REALIX     '              ; 11 байт

ebr_system_id:
    db 'FAT12   '                 ; 8 байт


; ============================================================
; КОНСТАНТЫ FLOPPY FAT12
; ============================================================

ROOT_DIR_LBA       equ 19
ROOT_DIR_SECTORS   equ 14
DATA_LBA           equ 33

ROOT_BUFFER        equ 0x0500
FAT_BUFFER         equ 0x0500

INITRIX_SEGMENT    equ 0x07E0
INITRIX_OFFSET     equ 0x0000


; ============================================================
; ТОЧКА ВХОДА
; ============================================================

start:
    cli
    cld

    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00

    ; BIOS передаёт номер загрузочного диска в DL.
    mov [ebr_drive_number], dl

    sti

    ; Читаем корневой каталог FAT12 в 0000:0500.
    mov ax, ROOT_DIR_LBA
    mov cl, ROOT_DIR_SECTORS
    mov bx, ROOT_BUFFER
    call disk_read
    jc boot_error

    ; Ищем INITRIX.BIN в корневом каталоге.
    mov di, ROOT_BUFFER
    mov bp, [bpb_dir_entries]

search_entry:
    test bp, bp
    jz boot_error

    ; 00h означает конец используемых записей каталога.
    cmp byte [es:di], 0x00
    je boot_error

    ; E5h означает удалённую запись.
    cmp byte [es:di], 0xE5
    je next_entry

    ; Пропускаем Long File Name entries.
    mov al, [es:di + 11]
    and al, 0x0F
    cmp al, 0x0F
    je next_entry

    push di

    mov si, initrix_filename
    mov cx, 11
    repe cmpsb

    pop di
    je initrix_found

next_entry:
    add di, 32
    dec bp
    jmp search_entry


; ============================================================
; INITRIX НАЙДЕН
; ============================================================

initrix_found:
    ; Первый кластер файла находится по смещению 26.
    mov ax, [es:di + 26]
    mov [current_cluster], ax

    cmp ax, 2
    jb boot_error

    ; Читаем первую таблицу FAT в 0000:0500.
    xor ax, ax
    mov es, ax

    mov ax, 1
    mov cl, 9
    mov bx, FAT_BUFFER
    call disk_read
    jc boot_error

    ; Настраиваем адрес загрузки Initrix.
    mov ax, INITRIX_SEGMENT
    mov es, ax
    mov bx, INITRIX_OFFSET


; ============================================================
; ЧТЕНИЕ FAT12-ЦЕПОЧКИ
; ============================================================

load_cluster:
    mov ax, [current_cluster]

    ; Допустимые data-кластеры начинаются с 2.
    cmp ax, 2
    jb boot_error

    ; LBA = DATA_LBA + cluster - 2.
    sub ax, 2
    add ax, DATA_LBA

    mov cl, 1
    call disk_read
    jc boot_error

    ; Один кластер равен одному сектору — 512 байт.
    add bx, 512
    jnc destination_ready

    ; При переполнении BX передвигаем ES на 64 КиБ.
    mov ax, es
    add ax, 0x1000
    mov es, ax

destination_ready:
    ; Смещение FAT12-записи:
    ; offset = cluster + cluster / 2.
    mov ax, [current_cluster]
    mov dx, ax

    mov si, ax
    shr si, 1
    add si, ax
    add si, FAT_BUFFER

    ; FAT находится в DS=0 по адресу 0x0500.
    mov ax, [ds:si]

    ; Для нечётного кластера используются старшие 12 бит.
    test dx, 1
    jz even_cluster

    shr ax, 4
    jmp cluster_ready

even_cluster:
    and ax, 0x0FFF

cluster_ready:
    ; 0xFF8–0xFFF — конец FAT12-цепочки.
    cmp ax, 0x0FF8
    jae launch_initrix

    ; 0xFF7 — bad cluster.
    cmp ax, 0x0FF7
    je boot_error

    ; 0xFF0–0xFF6 — зарезервированные значения.
    cmp ax, 0x0FF0
    jae boot_error

    ; 0 и 1 недопустимы для файловой цепочки.
    cmp ax, 2
    jb boot_error

    mov [current_cluster], ax
    jmp load_cluster


; ============================================================
; ЗАПУСК INITRIX
; ============================================================

launch_initrix:
    ; Считываем номер диска до переключения DS.
    mov dl, [ebr_drive_number]

    mov ax, INITRIX_SEGMENT
    mov ds, ax
    mov es, ax

    cld

    jmp INITRIX_SEGMENT:INITRIX_OFFSET


; ============================================================
; BIOS DISK READ
; ============================================================

; Вход:
;   AX    — LBA;
;   CL    — количество секторов;
;   ES:BX — адрес назначения.
;
; Номер диска берётся из ebr_drive_number.
;
; Выход:
;   CF=0 — успех;
;   CF=1 — ошибка.
;
; Примечание:
;   Bootix использует функцию только для запросов, которые не пересекают
;   границу дорожки:
;     root: LBA 19, 14 секторов;
;     FAT:  LBA 1,  9 секторов;
;     файл: по одному сектору.
disk_read:
    pusha
    push es

    mov [read_lba], ax
    mov [read_count], cl
    mov [read_segment], es
    mov [read_offset], bx

    mov di, 3

disk_read_retry:
    mov ax, [read_lba]
    call lba_to_chs

    mov ax, [read_segment]
    mov es, ax
    mov bx, [read_offset]

    mov dl, [ebr_drive_number]

    mov ah, 0x02
    mov al, [read_count]

    stc
    int 0x13
    jnc disk_read_success

    ; Сброс дискового контроллера.
    mov dl, [ebr_drive_number]
    xor ax, ax

    stc
    int 0x13

    dec di
    jnz disk_read_retry

disk_read_failure:
    pop es
    popa

    stc
    ret

disk_read_success:
    pop es
    popa

    clc
    ret


; ============================================================
; LBA → CHS
; ============================================================

; Вход:
;   AX — LBA.
;
; Выход:
;   CH — младшие 8 бит цилиндра;
;   CL — сектор и старшие биты цилиндра;
;   DH — номер головки;
;   DL — номер загрузочного диска.
lba_to_chs:
    xor dx, dx
    div word [bpb_sectors_per_track]

    ; DX = номер сектора внутри дорожки, начиная с 0.
    inc dx
    mov cx, dx

    ; AX = LBA / sectors_per_track.
    xor dx, dx
    div word [bpb_heads]

    ; AX = цилиндр, DX = головка.
    mov dh, dl
    mov ch, al

    ; Старшие два бита цилиндра помещаются в CL[6:7].
    shl ah, 6
    or cl, ah

    mov dl, [ebr_drive_number]
    ret


; ============================================================
; ОШИБКА ЗАГРУЗКИ
; ============================================================

boot_error:
    xor ax, ax
    mov ds, ax

    mov si, error_message
    call print_string

    ; Ожидаем клавишу и перезагружаемся.
    xor ah, ah
    int 0x16

    jmp 0xFFFF:0x0000


; ============================================================
; ВЫВОД СТРОКИ
; ============================================================

; Вход:
;   DS:SI — нуль-терминированная строка.
print_string:
    push ax
    push bx

    mov ah, 0x0E
    xor bx, bx

print_string_next:
    lodsb
    test al, al
    jz print_string_done

    int 0x10
    jmp print_string_next

print_string_done:
    pop bx
    pop ax
    ret


; ============================================================
; ДАННЫЕ
; ============================================================

initrix_filename:
    db 'INITRIX BIN'

error_message:
    db 0x0D, 0x0A
    db '[!] Cannot load INITRIX.BIN'
    db 0

current_cluster:
    dw 0

read_lba:
    dw 0

read_segment:
    dw 0

read_offset:
    dw 0

read_count:
    db 0


; ============================================================
; BIOS BOOT SIGNATURE
; ============================================================

times 510 - ($ - $$) db 0
dw 0xAA55
