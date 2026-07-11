; © Realix > FAT12 File Loader
; Исправленная версия
; ===========================

%ifndef FAT12_FILE_LOAD_ASM
%define FAT12_FILE_LOAD_ASM

%include 'bios-api/fat12/init.asm'


; ============================================================
; КОДЫ ОШИБОК
; ============================================================

FAT12_ERROR_NONE             equ 0
FAT12_ERROR_NOT_FOUND        equ 1
FAT12_ERROR_BAD_CLUSTER      equ 2
FAT12_ERROR_INVALID_CHAIN    equ 3
FAT12_ERROR_DISK             equ 4
FAT12_ERROR_BUFFER_OVERLAP   equ 5
FAT12_ERROR_FILE_TOO_LARGE   equ 6


; ============================================================
; ЗАГРУЗКА ФАЙЛА
; ============================================================

; Вход:
;   DS:SI — FAT 8.3-имя длиной ровно 11 байт;
;   CX    — сегмент назначения;
;   BX    — смещение назначения;
;   DL    — номер BIOS-диска.
;
; Выход:
;   CF=0:
;       DX:AX — размер файла в байтах.
;
;   CF=1:
;       AL — код FAT12_ERROR_*.
;
; Ограничение:
;   один кластер не должен превышать 127 секторов.
file_load:
    push bx
    push cx
    push si
    push di
    push es
    push bp

    mov [fat12_filename], si
    mov [fat12_destination_segment], cx
    mov [fat12_destination_offset], bx
    mov [fat12_drive], dl

    mov byte [fat12_last_error], FAT12_ERROR_NONE

    ; Проверяем пересечение назначения с внутренним FAT/root buffer.
    call fat12_check_destination
    jc .return_error

    ; Читаем root directory в 0000:0500.
    xor ax, ax
    mov es, ax

    mov ax, [root_dir_lba]
    mov cx, [root_dir_size]
    mov dl, [fat12_drive]
    mov bx, 0x0500
    call disk_read
    jc .disk_error

    mov di, 0x0500
    mov bp, [dir_entries]

.search_entry:
    test bp, bp
    jz .not_found

    ; 00h — конец используемых записей.
    cmp byte [es:di], 0x00
    je .not_found

    ; E5h — удалённая запись.
    cmp byte [es:di], 0xE5
    je .next_entry

    ; Пропускаем LFN.
    mov al, [es:di + 11]
    and al, 0x0F
    cmp al, 0x0F
    je .next_entry

    ; Пропускаем каталоги и volume labels.
    test byte [es:di + 11], 0x18
    jnz .next_entry

    push di
    push bp

    mov si, [fat12_filename]
    mov cx, 11
    repe cmpsb

    pop bp
    pop di

    je .found

.next_entry:
    add di, 32
    dec bp
    jmp .search_entry

.not_found:
    mov byte [fat12_last_error], FAT12_ERROR_NOT_FOUND
    jmp .return_error


.found:
    mov ax, [es:di + 26]
    mov [fat12_current_cluster], ax

    ; Размер файла из directory entry.
    mov ax, [es:di + 28]
    mov dx, [es:di + 30]

    mov [fat12_file_size_low], ax
    mov [fat12_file_size_high], dx
    mov [fat12_remaining_low], ax
    mov [fat12_remaining_high], dx

    ; Пустой файл успешно загружен без обращения к FAT.
    or ax, dx
    jz .success

    ; Стартовый кластер непустого файла должен быть >= 2.
    cmp word [fat12_current_cluster], 2
    jb .invalid_chain

    ; Читаем первую FAT в 0000:0500.
    xor ax, ax
    mov es, ax

    mov ax, [reserved_sectors]
    mov cx, [sectors_per_fat]
    mov dl, [fat12_drive]
    mov bx, 0x0500
    call disk_read
    jc .disk_error

    ; Настраиваем destination.
    mov ax, [fat12_destination_segment]
    mov [fat12_active_segment], ax

    mov ax, [fat12_destination_offset]
    mov [fat12_active_offset], ax

    ; Защита от зацикленной FAT-цепочки.
    mov word [fat12_chain_steps], 0


; ============================================================
; ОБХОД FAT12-ЦЕПОЧКИ
; ============================================================

.load_cluster:
    inc word [fat12_chain_steps]

    ; FAT12 floppy 1.44 MB имеет менее 4096 кластеров.
    cmp word [fat12_chain_steps], 4096
    ja .invalid_chain

    mov ax, [fat12_current_cluster]

    cmp ax, 2
    jb .invalid_chain

    cmp ax, 0x0FEF
    ja .invalid_chain

    ; LBA = data_lba + (cluster - 2) * sectors_per_cluster.
    sub ax, 2

    xor cx, cx
    mov cl, [sectors_per_cluster]

    cmp cx, 0
    je .invalid_chain

    cmp cx, 127
    ja .invalid_chain

    mul cx

    or dx, dx
    jnz .file_too_large

    add ax, [data_lba]
    jc .file_too_large

    mov [fat12_cluster_lba], ax

    ; Определяем число секторов, необходимое для остатка файла.
    call fat12_sectors_for_current_cluster
    jc .file_too_large

    mov [fat12_sectors_to_read], cx

    ; Читаем данные кластера.
    mov ax, [fat12_active_segment]
    mov es, ax
    mov bx, [fat12_active_offset]

    mov ax, [fat12_cluster_lba]
    mov dl, [fat12_drive]
    mov cx, [fat12_sectors_to_read]
    call disk_read
    jc .disk_error

    ; Вычисляем фактически потреблённое количество байтов.
    call fat12_consume_cluster_bytes

    ; Остаток равен нулю — загрузка завершена.
    mov ax, [fat12_remaining_low]
    or ax, [fat12_remaining_high]
    jz .success

    ; Переходим к следующему кластеру FAT.
    call fat12_get_next_cluster
    jc .return_error

    mov [fat12_current_cluster], ax
    jmp .load_cluster


; ============================================================
; УСПЕХ И ОШИБКИ
; ============================================================

.success:
    mov ax, [fat12_file_size_low]
    mov dx, [fat12_file_size_high]

    pop bp
    pop es
    pop di
    pop si
    pop cx
    pop bx

    clc
    ret

.disk_error:
    mov byte [fat12_last_error], FAT12_ERROR_DISK
    jmp .return_error

.invalid_chain:
    mov byte [fat12_last_error], FAT12_ERROR_INVALID_CHAIN
    jmp .return_error

.file_too_large:
    mov byte [fat12_last_error], FAT12_ERROR_FILE_TOO_LARGE

.return_error:
    xor ah, ah
    mov al, [fat12_last_error]

    pop bp
    pop es
    pop di
    pop si
    pop cx
    pop bx

    stc
    ret


; ============================================================
; ПРОВЕРКА БУФЕРА НАЗНАЧЕНИЯ
; ============================================================

fat12_check_destination:
    push ax
    push dx

    ; Линейный адрес = segment * 16 + offset.
    mov ax, [fat12_destination_segment]
    mov dx, ax

    shl ax, 4
    shr dx, 12

    add ax, [fat12_destination_offset]
    adc dx, 0

    ; Внутренний буфер занимает область 0000:0500–0000:7BFF.
    test dx, dx
    jnz .safe

    cmp ax, 0x0500
    jb .safe

    cmp ax, 0x7C00
    jae .safe

    mov byte [fat12_last_error], FAT12_ERROR_BUFFER_OVERLAP

    pop dx
    pop ax
    stc
    ret

.safe:
    pop dx
    pop ax
    clc
    ret


; ============================================================
; ЧИСЛО СЕКТОРОВ ДЛЯ ТЕКУЩЕГО КЛАСТЕРА
; ============================================================

; Выход:
;   CX — число секторов;
;   CF=1 — размер не представим.
fat12_sectors_for_current_cluster:
    push ax
    push bx
    push dx

    xor bx, bx
    mov bl, [sectors_per_cluster]

    test bx, bx
    jz .failure

    ; Если remaining_high != 0, нужен полный кластер.
    cmp word [fat12_remaining_high], 0
    jne .full_cluster

    mov ax, [fat12_remaining_low]

    ; sectors = ceil(remaining / bytes_per_sector).
    xor dx, dx
    div word [bytes_per_sector]

    test dx, dx
    jz .rounded

    inc ax

.rounded:
    cmp ax, bx
    jbe .use_calculated

.full_cluster:
    mov cx, bx
    jmp .success

.use_calculated:
    mov cx, ax

.success:
    pop dx
    pop bx
    pop ax
    clc
    ret

.failure:
    pop dx
    pop bx
    pop ax
    stc
    ret


; ============================================================
; ОБНОВЛЕНИЕ ОСТАТКА И АДРЕСА НАЗНАЧЕНИЯ
; ============================================================

fat12_consume_cluster_bytes:
    push ax
    push bx
    push cx
    push dx

    ; DX:AX = sectors_to_read * bytes_per_sector.
    mov ax, [fat12_sectors_to_read]
    mul word [bytes_per_sector]

    ; Если количество прочитанных байтов >= remaining,
    ; остаток становится нулевым.
    cmp dx, [fat12_remaining_high]
    ja .consume_all
    jb .subtract

    cmp ax, [fat12_remaining_low]
    jae .consume_all

.subtract:
    sub [fat12_remaining_low], ax
    sbb [fat12_remaining_high], dx
    jmp .advance_destination

.consume_all:
    mov word [fat12_remaining_low], 0
    mov word [fat12_remaining_high], 0

.advance_destination:
    ; Дисковый драйвер записал DX:AX байтов.
    ; Для текущего API число не превышает 65024 байт.
    add [fat12_active_offset], ax
    jnc .done

    add word [fat12_active_segment], 0x1000

.done:
    pop dx
    pop cx
    pop bx
    pop ax
    ret


; ============================================================
; ПОЛУЧЕНИЕ СЛЕДУЮЩЕГО FAT12-КЛАСТЕРА
; ============================================================

; Выход:
;   AX — следующий кластер;
;   CF=0 — обычный кластер;
;   CF=1 — ошибка либо преждевременный конец цепочки.
fat12_get_next_cluster:
    push bx
    push dx
    push si
    push es

    mov ax, [fat12_current_cluster]
    mov dx, ax

    ; offset = cluster + cluster / 2.
    mov bx, ax
    shr bx, 1
    add bx, ax

    ; Запись должна целиком помещаться в загруженной FAT.
    mov ax, [sectors_per_fat]
    mul word [bytes_per_sector]

    test dx, dx
    jnz .invalid

    dec ax
    cmp bx, ax
    jae .invalid

    xor ax, ax
    mov es, ax

    mov si, 0x0500
    add si, bx
    mov ax, [es:si]

    test word [fat12_current_cluster], 1
    jz .even

    shr ax, 4
    jmp .validate

.even:
    and ax, 0x0FFF

.validate:
    ; Конец цепочки допустим только после полного размера файла.
    cmp ax, 0x0FF8
    jae .invalid

    cmp ax, 0x0FF7
    je .bad_cluster

    cmp ax, 0x0FF0
    jae .invalid

    cmp ax, 2
    jb .invalid

    pop es
    pop si
    pop dx
    pop bx
    clc
    ret

.bad_cluster:
    mov byte [fat12_last_error], FAT12_ERROR_BAD_CLUSTER
    jmp .failure

.invalid:
    mov byte [fat12_last_error], FAT12_ERROR_INVALID_CHAIN

.failure:
    pop es
    pop si
    pop dx
    pop bx
    stc
    ret


; ============================================================
; СОСТОЯНИЕ ЗАГРУЗЧИКА
; ============================================================

fat12_filename:
    dw 0

fat12_destination_segment:
    dw 0

fat12_destination_offset:
    dw 0

fat12_active_segment:
    dw 0

fat12_active_offset:
    dw 0

fat12_current_cluster:
    dw 0

fat12_cluster_lba:
    dw 0

fat12_sectors_to_read:
    dw 0

fat12_chain_steps:
    dw 0

fat12_file_size_low:
    dw 0

fat12_file_size_high:
    dw 0

fat12_remaining_low:
    dw 0

fat12_remaining_high:
    dw 0

fat12_drive:
    db 0

fat12_last_error:
    db FAT12_ERROR_NONE

%endif
