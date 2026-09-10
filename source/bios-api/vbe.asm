; © Realix > VBE (VESA BIOS Extensions) - режимы выше 320x200x256
; ================
; ❗️ Работает ТОЛЬКО из Real Mode, ДО перехода в Protected Mode - как и
; обычный enable_vga_videomode (graphics.asm). Никакого рискованного
; возврата в Real Mode во время работы Protected Mode здесь нет и не нужно
; ❗️ Если VBE или линейный фреймбуфер (LFB) недоступны - видеорежим НЕ
; включается (остаётся тем, что было), вызывающий код должен откатиться на
; enable_vga_videomode - см. switcher.asm: load_kernel32_video_hires


; > Попытка включения видеорежима VBE с линейным фреймбуфером (LFB)
; Параметры:
;  - cx: номер режима VBE (напр. 0x100 = 640x400x256 - ровно 2x 320x200,
;    та же пропорция сторон, в отличие от 0x101 = 640x480, 4:3)
; Вывод:
;  - CF (Carry Flag): 0 (Успех - см. vbe_width/height/stride/lfb_addr),
;    1 (Неудача - режим или LFB недоступны, видеорежим НЕ тронут)
enable_vbe_videomode:
    pusha
    push es
    push ds

    mov [.mode_number], cx

    ; ES:DI -> буфер под ModeInfoBlock в НАШЕМ ЖЕ сегменте (проще и
    ; безопаснее, чем выбирать отдельный "свободный" физический адрес)
    mov ax, ds
    mov es, ax
    mov di, vbe_mode_info_block

    ; Function 4F01h: Get SuperVGA Mode Information
    mov cx, [.mode_number]
    mov ax, 0x4F01
    int 0x10
    cmp ax, 0x004F
    jne .fail

    ; ModeAttributes (offset 0): бит0 - режим поддерживается аппаратно,
    ; бит7 - есть линейный фреймбуфер (LFB, VBE 2.0+)
    mov ax, [es:di]
    test ax, 0x0001
    jz .fail
    test ax, 0x0080
    jz .fail

    ; Сохраняем нужные поля СРАЗУ - Set Mode ниже вправе перезаписать буфер
    mov ax, [es:di + 0x12]   ; XResolution
    mov [vbe_width], ax
    mov ax, [es:di + 0x14]   ; YResolution
    mov [vbe_height], ax
    mov ax, [es:di + 0x10]   ; BytesPerScanLine
    mov [vbe_stride], ax
    mov eax, [es:di + 0x28]  ; PhysBasePtr (физический адрес LFB)
    mov [vbe_lfb_addr], eax

    ; Function 4F02h: Set SuperVGA Video Mode (бит14 = использовать LFB)
    mov bx, [.mode_number]
    or bx, 0x4000
    mov ax, 0x4F02
    int 0x10
    cmp ax, 0x004F
    jne .fail

    clc
    jmp .done

.fail:
    stc

.done:
    pop ds
    pop es
    popa
    ret

.mode_number: dw 0


; Результаты последнего успешного enable_vbe_videomode
vbe_width:           dw 0
vbe_height:          dw 0
vbe_stride:          dw 0
vbe_lfb_addr:         dd 0
vbe_mode_info_block: times 256 db 0
