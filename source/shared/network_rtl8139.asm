; © Realix > RTL8139 Network Driver
; ø Copyright by @createrman-system + @dintslaych
; (12.07.26) v0.09
; ================
; ❗️ Временно модуль не используется... (Кроме инициализации)

; Порты конфигурации PCI
PCI_CONFIG_ADDRESS  equ 0x0CF8
PCI_CONFIG_DATA     equ 0x0CFC
PCI_ENABLE_BIT      equ 0x80000000  ; Бит "включить конфигурационный цикл"
PCI_RTL8139_ID      equ 0x813910EC  ; Vendor: 0x10EC, Device: 0x8139
PCI_COMMAND_OFFSET  equ 0x04        ; Command/Status регистр в конфиг. пространстве
PCI_BAR0_OFFSET     equ 0x10        ; Base Address Register 0
PCI_COMMAND_IO      equ 0x0001      ; Бит "I/O Space Enable"
PCI_COMMAND_MASTER  equ 0x0004      ; Бит "Bus Master Enable" (нужен для DMA)

; Регистры RTL8139 (смещения от `net_io_base`)
REG_MAC      equ 0x00  ; MAC-адрес (6 байт)
REG_MAR      equ 0x08  ; Multicast регистры (8 байт)
REG_TSD0     equ 0x10  ; Дескриптор статуса передачи (0)
REG_TSAD0    equ 0x20  ; Дескриптор начального адреса передачи (0)
REG_RBSTART  equ 0x30  ; Начальный адрес буфера приёма данных
REG_CR       equ 0x37  ; Регистр команд
REG_CAPR     equ 0x38  ; Текущий адрес чтения пакета
REG_IMR      equ 0x3C  ; Регистр маски прерываний
REG_ISR      equ 0x3E  ; Регистр статуса прерываний
REG_TCR      equ 0x40  ; Регистр конфигурации передачи
REG_RCR      equ 0x44  ; Регистр конфигурации приёма
REG_CONFIG1  equ 0x52  ; Регистр конфигурации 1

; Команды и биты RTL8139
CR_RST   equ 0x10      ; Сброс
CR_RE    equ 0x08      ; Включение приёмника
CR_TE    equ 0x04      ; Включение передатчика
CR_BUFE  equ 0x01      ; Буфер пуст

; Столько итераций ждём снятия CR_RST, прежде чем считать сброс зависшим
RTL8139_RESET_TIMEOUT equ 0x0000FFFF

; > Инициализирует сетевую карту RTL8139 на шине PCI
; Вывод:
;  - Carry Flag (CF): Установлен, если карта не найдена, BAR0 некорректен
;    или сброс не завершился за отведённое время.
net_init:
    pushad

    ; Инициализация параметров
    mov word [cs:net_io_base], 0
    mov dword [cs:net_pci_address], 0

    ; Поиск RTL8139 на PCI (Vendor: 0x10EC, Устройство: 0x8139)
    ; Старт с Bus 0, Dev 0, Func 0
    mov ebx, PCI_ENABLE_BIT

.pci_loop:
    ; Чтение VendorID (low) и DeviceID (high)
    mov eax, ebx
    mov dx, PCI_CONFIG_ADDRESS
    out dx, eax
    mov dx, PCI_CONFIG_DATA
    in eax, dx

    ; Сравниваем полученное название устройства с сетевой картой
    cmp eax, PCI_RTL8139_ID
    je .found

    ; Переход к след. устройству, с условием проверки только первых 32
    add ebx, 0x800
    cmp ebx, 0x80010000
    jb .pci_loop

    ; Карта не найдена
    jmp .fail

.found:
    mov [cs:net_pci_address], ebx

    ; Включаем PCI I/O Space (бит 0) и Bus Master (бит 2) в Command-регистре.
    ; Без Bus Master DMA-передачи дескрипторов TX/RX могут не работать.
    mov eax, ebx
    or eax, PCI_COMMAND_OFFSET
    mov dx, PCI_CONFIG_ADDRESS
    out dx, eax

    mov dx, PCI_CONFIG_DATA
    in eax, dx
    or ax, PCI_COMMAND_IO | PCI_COMMAND_MASTER
    mov ecx, eax

    mov eax, ebx
    or eax, PCI_COMMAND_OFFSET
    mov dx, PCI_CONFIG_ADDRESS
    out dx, eax

    mov eax, ecx
    mov dx, PCI_CONFIG_DATA
    out dx, eax

    ; Получаем базовый адрес порта I/O из BAR0 (Смещение PCI 0x10)
    mov eax, ebx
    or eax, PCI_BAR0_OFFSET
    mov dx, PCI_CONFIG_ADDRESS
    out dx, eax
    mov dx, PCI_CONFIG_DATA
    in eax, dx

    ; У I/O BAR младший бит установлен - иначе это MMIO BAR, нам не подходит
    test eax, 1
    jz .fail

    ; Очистка бита 0 (IO) и бита 1 (reserved)
    and eax, 0xFFFFFFFC

    ; Этот драйвер работает только с 16-битным пространством I/O-портов
    test eax, 0xFFFF0000
    jnz .fail

    ; Нулевой адрес недопустим
    test ax, ax
    jz .fail

    mov [cs:net_io_base], ax

    ; Включение питания (Разблокировка регистров конфигурации)
    mov dx, ax
    add dx, REG_CONFIG1
    mov al, 0x00
    out dx, al

    ; Программный сброс сетевой карты (Software Reset)
    mov dx, [cs:net_io_base]
    add dx, REG_CR
    mov al, CR_RST
    out dx, al

    ; Ждём, пока карта очистит бит CR_RST. Тайм-аут вместо бесконечного
    ; ожидания - иначе зависшая карта повесит загрузку системы навсегда.
    mov ecx, RTL8139_RESET_TIMEOUT

.wait_reset:
    in al, dx
    test al, CR_RST
    jz .reset_complete

    dec ecx
    jnz .wait_reset

    ; Сброс не завершился за отведённое время
    jmp .fail

.reset_complete:
    ; Настройка буфера приема (RX Buffer)
    mov eax, 0x00070000
    mov dx, [cs:net_io_base]
    add dx, REG_RBSTART
    out dx, eax
    mov word [cs:net_rx_ptr], 0

    ; Включение передатчика (TE) и приемника (RE)
    mov dx, [cs:net_io_base]
    add dx, REG_CR
    mov al, CR_RE | CR_TE
    out dx, al

    ; Конфигурация приема (Принимать Broadcast-пакеты + физический MAC)
    mov dx, [cs:net_io_base]
    add dx, REG_RCR
    mov eax, 0x0000000A       ; AB (Accept Broadcast) + AM (Multicast)
    out dx, eax

    ; Сброс CF (Успех)
    popad
    clc
    ret

.fail:
    ; Установка CF (Ошибка) и сброс состояния драйвера
    mov word [cs:net_io_base], 0
    mov dword [cs:net_pci_address], 0

    popad
    stc
    ret


; > Отправка Ethernet-пакета
; Параметры:
;   - ds:si: Указатель на данные пакета
;   - cx:    Размер пакета в байтах
net_send:
    pushad

    ; Определение текущего TX дескриптора (у RTL8139 их всего 4 по 4 байта)
    movzx bx, byte [cs:net_tx_cur]
    shl bx, 2

    ; Установка физического адреса памяти с eax (TSAD)
    mov ax, ds
    movzx eax, ax
    shl eax, 4
    movzx esi, si
    add eax, esi

    mov dx, [cs:net_io_base]
    add dx, REG_TSAD0
    add dx, bx
    out dx, eax

    ; Установка длины пакета и старт передачи (TSD)
    mov dx, [cs:net_io_base]
    add dx, REG_TSD0
    add dx, bx
    movzx eax, cx
    and eax, 0x1FFF   ; Размер пакета (биты 0-12)
    out dx, eax       ; Запись в TSD активирует отправку пакета картой

    ; Инкремент дескриптора для следующей отправки (циклично от 0 до 3)
    inc byte [cs:net_tx_cur]
    and byte [cs:net_tx_cur], 3

    popad
    ret


; > Читает локальный MAC-адрес сетевой карты
; Параметры:
;  - es:di: Указатель на буфер (6 байт) для сохранения MAC-адреса
net_get_mac:
    pushad

    ; Подготовка данных для чтения из микросхемы
    mov dx, [cs:net_io_base]
    add dx, REG_MAC
    mov cx, 6
.loop:
    ; Чтение по байту из dx и запись в es:di
    in al, dx
    stosb
    inc dx
    loop .loop
.done:
    popad
    ret


; Переменные
align 4
net_pci_address:  dd 0  ; PCI configuration address найденной карты (для диагностики)
net_io_base:      dw 0
net_tx_cur:       db 0  ; Текущий дескриптор передачи (0-3)
net_rx_ptr:       dw 0  ; Текущее смещение в буфере приёма
