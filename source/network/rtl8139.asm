; © Realix > RTL8139 Network Driver
; Исправленная NASM-совместимая версия
; ===================================

%ifndef RTL8139_ASM
%define RTL8139_ASM


; ============================================================
; PCI
; ============================================================

PCI_CONFIG_ADDRESS  equ 0x0CF8
PCI_CONFIG_DATA     equ 0x0CFC

PCI_ENABLE_BIT      equ 0x80000000
PCI_RTL8139_ID      equ 0x813910EC

PCI_COMMAND_OFFSET  equ 0x04
PCI_BAR0_OFFSET     equ 0x10

PCI_COMMAND_IO      equ 0x0001
PCI_COMMAND_MASTER  equ 0x0004


; ============================================================
; РЕГИСТРЫ RTL8139
; ============================================================

REG_MAC      equ 0x00
REG_MAR      equ 0x08
REG_TSD0     equ 0x10
REG_TSAD0    equ 0x20
REG_RBSTART  equ 0x30
REG_CR       equ 0x37
REG_CAPR     equ 0x38
REG_IMR      equ 0x3C
REG_ISR      equ 0x3E
REG_TCR      equ 0x40
REG_RCR      equ 0x44
REG_CONFIG1  equ 0x52


; ============================================================
; БИТЫ RTL8139
; ============================================================

CR_RST   equ 0x10
CR_RE    equ 0x08
CR_TE    equ 0x04
CR_BUFE  equ 0x01


; ============================================================
; НАСТРОЙКИ
; ============================================================

RTL8139_RESET_TIMEOUT equ 0x0000FFFF


; ============================================================
; ИНИЦИАЛИЗАЦИЯ RTL8139
; ============================================================

; Ищет RTL8139 на PCI bus 0 и выполняет программный сброс.
;
; Выход:
;   CF=0 — RTL8139 найдена и успешно сброшена;
;   CF=1 — карта не найдена, BAR некорректен или сброс завис.
;
; При успехе:
;   [cs:net_io_base] содержит базовый адрес I/O-портов карты.
;
; Регистры вызывающего кода сохраняются.
net_init:
    pushad

    mov word [cs:net_io_base], 0
    mov dword [cs:net_pci_address], 0

    ; Начальный PCI configuration address:
    ; bus      = 0
    ; device   = 0
    ; function = 0
    ; register = 0
    mov ebx, PCI_ENABLE_BIT

.pci_scan:
    ; Читаем Vendor ID и Device ID.
    mov eax, ebx

    mov dx, PCI_CONFIG_ADDRESS
    out dx, eax

    mov dx, PCI_CONFIG_DATA
    in eax, dx

    ; EAX:
    ; младшие 16 бит — Vendor ID;
    ; старшие 16 бит — Device ID.
    cmp eax, PCI_RTL8139_ID
    je .device_found

    ; Переходим к следующему PCI device.
    ; Поле device расположено в битах 11–15.
    add ebx, 0x00000800

    ; Сканируем устройства 0–31 только на bus 0.
    cmp ebx, 0x80010000
    jb .pci_scan

    jmp .failure


; ============================================================
; УСТРОЙСТВО НАЙДЕНО
; ============================================================

.device_found:
    mov [cs:net_pci_address], ebx


; ============================================================
; ВКЛЮЧЕНИЕ PCI I/O SPACE И BUS MASTER
; ============================================================

    ; Выбираем PCI Command/Status register по offset 0x04.
    mov eax, ebx
    or eax, PCI_COMMAND_OFFSET

    mov dx, PCI_CONFIG_ADDRESS
    out dx, eax

    ; Читаем Command/Status DWORD.
    mov dx, PCI_CONFIG_DATA
    in eax, dx

    ; Включаем:
    ; bit 0 — I/O Space;
    ; bit 2 — Bus Master.
    or ax, PCI_COMMAND_IO | PCI_COMMAND_MASTER

    ; Сохраняем изменённое значение.
    mov ecx, eax

    ; Повторно выбираем Command/Status register.
    mov eax, ebx
    or eax, PCI_COMMAND_OFFSET

    mov dx, PCI_CONFIG_ADDRESS
    out dx, eax

    ; Записываем обновлённый Command/Status DWORD.
    mov eax, ecx
    mov dx, PCI_CONFIG_DATA
    out dx, eax


; ============================================================
; ЧТЕНИЕ BAR0
; ============================================================

    ; Выбираем BAR0 по offset 0x10.
    mov eax, ebx
    or eax, PCI_BAR0_OFFSET

    mov dx, PCI_CONFIG_ADDRESS
    out dx, eax

    mov dx, PCI_CONFIG_DATA
    in eax, dx

    ; Для текущего драйвера BAR0 должен быть I/O BAR.
    ; У I/O BAR младший бит установлен.
    test eax, 1
    jz .failure

    ; Очищаем служебные биты BAR.
    and eax, 0xFFFFFFFC

    ; Этот драйвер использует 16-битное пространство I/O-портов.
    test eax, 0xFFFF0000
    jnz .failure

    ; Нулевой адрес недопустим.
    test ax, ax
    jz .failure

    ; Сохраняем базовый I/O-адрес RTL8139.
    mov [cs:net_io_base], ax


; ============================================================
; ВКЛЮЧЕНИЕ ПИТАНИЯ
; ============================================================

    ; CONFIG1 = 0 выводит RTL8139 из режима энергосбережения.
    mov dx, ax
    add dx, REG_CONFIG1

    xor al, al
    out dx, al


; ============================================================
; ПРОГРАММНЫЙ СБРОС
; ============================================================

    mov dx, [cs:net_io_base]
    add dx, REG_CR

    mov al, CR_RST
    out dx, al

    ; Ждём, пока карта очистит бит CR_RST.
    ; Тайм-аут предотвращает бесконечное зависание.
    mov ecx, RTL8139_RESET_TIMEOUT

.wait_reset:
    in al, dx
    test al, CR_RST
    jz .reset_complete

    dec ecx
    jnz .wait_reset

    jmp .failure


; ============================================================
; УСПЕХ
; ============================================================

.reset_complete:
    popad
    clc
    ret


; ============================================================
; ОШИБКА
; ============================================================

.failure:
    mov word [cs:net_io_base], 0
    mov dword [cs:net_pci_address], 0

    popad
    stc
    ret


; ============================================================
; СОСТОЯНИЕ ДРАЙВЕРА
; ============================================================

align 4

net_pci_address:
    dd 0

net_io_base:
    dw 0


%endif
