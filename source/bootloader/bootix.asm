; © Realix > Bootix (Stage 1)
; (14.06.26) v0.06
; ================
; ❗️ Зависимости: kernel16/io/print.asm (в режиме PRINT_MINIMAL),
;                 bios-api/disk/read.asm
; ❗️ Свой минимальный обход FAT12 вместо bios-api/fat12,
;   чтобы стадия 1 влезала в 512 байт (Дублирование намеренное).

bits 16
org 0x7C00

%include 'shared/config.asm'

jmp short start
nop

bpb_oem:                 db 'MSWIN4.1'
bpb_bytes_per_sector:    dw 512
bpb_sectors_per_cluster: db 1
bpb_reserved_sectors:    dw 1
bpb_fat_count:           db 2
bpb_dir_entries:         dw 0x0E0
bpb_total_sectors:       dw 2880
bpb_media_type:          db 0xF0
bpb_sectors_per_fat:     dw 9
bpb_sectors_per_track:   dw 18
bpb_heads:               dw 2
bpb_hidden_sectors:      dd 0
bpb_large_sectors:       dd 0

ebr_drive_number: db 0
                  db 0
ebr_signature:    db 29h
ebr_volume_id:    db 52h, 45h, 41h, 4Ch
ebr_volume_label: db 'Realix     '
ebr_system_id:    db 'FAT12   '

start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    mov [ebr_drive_number], dl
    sti
    jmp 0:main

main:
    call disk_init
    mov [bpb_sectors_per_track], cx
    mov [bpb_heads], dh

    xor ah, ah
    mov al, [bpb_fat_count]
    mul word [bpb_sectors_per_fat]
    add ax, [bpb_reserved_sectors]
    push ax
    mov cx, ax

    mov ax, [bpb_dir_entries]
    shl ax, 5
    xor dx, dx
    div word [bpb_bytes_per_sector]
    or dx, dx
    jz .read_root_dir
    inc ax

.read_root_dir:
    add cx, ax
    mov [data_lba], cx
    mov cl, al
    pop ax
    mov bx, FAT_BUFFER_ADDR
    call disk_read
    jc read_error

    xor bx, bx
    mov di, FAT_BUFFER_ADDR

.search_initrix:
    mov si, file_initrix_bin
    mov cx, 11
    push di
    repe cmpsb
    pop di
    je .found_initrix

    add di, 32
    inc bx
    cmp bx, [bpb_dir_entries]
    jb .search_initrix

    mov al, 2
    mov si, err_initrix_not_found
    jmp error_handler

.found_initrix:
    mov ax, [di + 26]
    push ax

    mov ax, [bpb_reserved_sectors]
    mov cl, [bpb_sectors_per_fat]
    mov bx, FAT_BUFFER_ADDR
    call disk_read
    jc read_error

    mov bx, INITRIX_LOAD_SEGMENT
    mov es, bx
    mov bx, INITRIX_LOAD_OFFSET

.load_initrix_loop:
    pop ax
    push ax
    sub ax, 2
    xor ch, ch
    mov cl, [bpb_sectors_per_cluster]
    mul cx
    add ax, [data_lba]
    call disk_read
    jc read_error

    xor ah, ah
    mov al, [bpb_sectors_per_cluster]
    mul word [bpb_bytes_per_sector]
    add bx, ax
    jnc .load_initrix_continue
    mov ax, es
    add ax, 0x1000
    mov es, ax
    xor bx, bx

.load_initrix_continue:
    pop ax
    mov cx, 3
    mul cx
    mov cx, 2
    div cx
    mov si, FAT_BUFFER_ADDR
    add si, ax
    mov ax, [si]
    or dx, dx
    jz .even_cluster
    shr ax, 4
    jmp .next_cluster
.even_cluster:
    and ax, 0x0FFF
.next_cluster:
    cmp ax, CHAIN_END
    jae .read_initrix_finish
    push ax
    jmp .load_initrix_loop

.read_initrix_finish:
    mov dl, [ebr_drive_number]
    mov ax, INITRIX_LOAD_SEGMENT
    mov ds, ax
    mov es, ax
    jmp INITRIX_LOAD_SEGMENT:INITRIX_LOAD_OFFSET


error_handler:
    call print
    push ax
    mov si, err_code_prefix
    call print
    pop ax
    call print_hex8
    mov si, err_code_suffix
    call print
    mov ah, 0
    int 0x16
    jmp 0xFFFF:0

print_hex8:
    push ax
    mov al, ah
    shr al, 4
    call .digit
    pop ax
    and al, 0x0F
    call .digit
    ret
.digit:
    add al, '0'
    cmp al, '9'
    jbe .out
    add al, 7
.out:
    call print_char
    ret

read_error:
    mov al, 1
    jmp error_handler

%define PRINT_MINIMAL
%include 'kernel16/io/print.asm'
%include 'bios-api/disk/read.asm'

err_initrix_not_found: db '[!] No Initrix!', 0
err_code_prefix:       db ' [0x', 0
err_code_suffix:       db ']', 0

file_initrix_bin: db 'INITRIX BIN'
data_lba:         dw 0

times 510-($-$$) db 0
dw 0xAA55