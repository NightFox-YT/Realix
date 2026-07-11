; © Realix > BIOS Disk Read
; Исправленная версия
; ===================

%ifndef BIOS_DISK_READ_ASM
%define BIOS_DISK_READ_ASM

%include 'bios-api/disk/init.asm'


; ============================================================
; ЧТЕНИЕ СЕКТОРОВ
; ============================================================

; Вход:
;   AX    — начальный LBA;
;   CX    — количество секторов;
;   DL    — номер BIOS-диска;
;   ES:BX — адрес назначения.
;
; Выход:
;   CF=0 — успешно;
;   CF=1 — ошибка.
;
; Регистры вызывающего кода и ES сохраняются.
disk_read:
    pusha
    push es

    mov [disk_read_lba], ax
    mov [disk_read_count], cx
    mov [disk_read_drive], dl
    mov [disk_read_segment], es
    mov [disk_read_offset], bx

    ; Нулевой запрос считается успешным.
    cmp cx, 0
    je .success

    ; Геометрия должна быть предварительно инициализирована.
    cmp word [disk_spt], 0
    je .failure

    cmp word [disk_heads], 0
    je .failure

.next_sector:
    cmp word [disk_read_count], 0
    je .success

    mov di, 3

.retry:
    mov ax, [disk_read_lba]
    call disk_lba_to_chs
    jc .failure

    mov ax, [disk_read_segment]
    mov es, ax
    mov bx, [disk_read_offset]

    mov dl, [disk_read_drive]
    mov ah, 0x02
    mov al, 1

    ; Некоторые BIOS некорректно оставляют CF неизменным.
    stc
    int 0x13
    jnc .sector_read

    ; Сбрасываем контроллер после ошибки.
    mov dl, [disk_read_drive]
    xor ax, ax
    stc
    int 0x13

    dec di
    jnz .retry

    jmp .failure

.sector_read:
    inc word [disk_read_lba]
    dec word [disk_read_count]

    ; Переход к следующему сектору назначения.
    add word [disk_read_offset], 512
    jnc .next_sector

    ; Переполнение смещения означает переход через 64-КиБ границу.
    add word [disk_read_segment], 0x1000
    jmp .next_sector

.success:
    pop es
    popa
    clc
    ret

.failure:
    pop es
    popa
    stc
    ret


; ============================================================
; LBA → CHS
; ============================================================

; Вход:
;   AX — LBA.
;
; Выход:
;   CH — младшие восемь бит цилиндра;
;   CL[0:5] — номер сектора;
;   CL[6:7] — старшие два бита цилиндра;
;   DH — номер головки;
;   CF=0 — успех;
;   CF=1 — неверная геометрия или цилиндр > 1023.
disk_lba_to_chs:
    push ax
    push bx
    push dx

    cmp word [disk_spt], 0
    je .failure

    cmp word [disk_heads], 0
    je .failure

    ; AX = LBA / sectors_per_track.
    ; DX = LBA % sectors_per_track.
    xor dx, dx
    div word [disk_spt]

    ; Сектора CHS нумеруются от единицы.
    inc dx
    mov cl, dl

    ; AX = cylinder.
    ; DX = head.
    xor dx, dx
    div word [disk_heads]

    ; BIOS CHS поддерживает цилиндры 0–1023.
    cmp ax, 1023
    ja .failure

    mov dh, dl
    mov ch, al

    ; Старшие два бита номера цилиндра.
    mov bl, ah
    and bl, 0x03
    shl bl, 6
    or cl, bl

    mov [disk_chs_head], dh

    pop dx
    mov dh, [disk_chs_head]
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
; СБРОС ДИСКОВОГО КОНТРОЛЛЕРА
; ============================================================

; Вход:
;   DL — номер BIOS-диска.
;
; Выход:
;   CF — результат BIOS.
disk_reset:
    push ax

    xor ax, ax
    stc
    int 0x13

    pop ax
    ret


; ============================================================
; ВРЕМЕННОЕ СОСТОЯНИЕ
; ============================================================

disk_read_lba:
    dw 0

disk_read_count:
    dw 0

disk_read_segment:
    dw 0

disk_read_offset:
    dw 0

disk_read_drive:
    db 0

disk_chs_head:
    db 0

%endif
