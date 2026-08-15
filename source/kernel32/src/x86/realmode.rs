// © Realix > Real Mode: BIOS disk thunk
// (15.08.26) v0.1
// ================
// ❗️ Kernel32 работает в защищённом режиме и не может напрямую вызывать
//    BIOS-прерывания (int 13h) - для дисковых операций эта функция временно
//    выключает Protected Mode (сбрасывает CR0.PE), делает вызов BIOS в Real
//    Mode и возвращается обратно. Работает только т.к. paging не используется
//    (линейный адрес = физический) и kernel32 физически загружен по адресу
//    0x10000 - т.е. полностью в пределах 1 МБ, доступных в Real Mode.
// ❗️ Всё состояние (регистры, флаги, IDTR) сохраняется и восстанавливается;
//    GDT не переключается - используются доп. 16-битные дескрипторы,
//    добавленные в основную GDT ядра (см. x86::gdt::REALMODE_*_SELECTOR).
// ❗️ Прямые ссылки на символы (`[SYMBOL]`) из 16-битного кода не работают:
//    линкер требует, чтобы 16-битное смещение само по себе влезало в 16 бит,
//    а весь код/данные kernel32 линкуются с базы 0x10000 (уже больше 64 КБ).
//    Поэтому единственная область параметров (RmParams) адресуется через
//    регистр esi (32-битная база, вычисленная через `lea` ещё в защищённом
//    режиме) + маленькое константное смещение поля - смещения не требуют
//    релокации, а esi переживает все переходы между режимами (не трогается
//    ни dальними переходами, ни BIOS-вызовами int 13h/int 10h).
// ❗️ Требует, чтобы весь двоичный образ kernel32.bin оставался < 64 КБ -
//    иначе смещения меток внутри 16-битных дескрипторов (база 0x10000,
//    лимит 0xFFFF) перестанут влезать.

use core::mem::offset_of;
use crate::x86::gdt::REALMODE_CODE_SELECTOR;

// Операции для RmParams.operation
pub const OP_READ: u8 = 0;
pub const OP_WRITE: u8 = 1;

// Размер временного стека Real Mode
const RM_STACK_SIZE: usize = 512;

/// Единая область параметров/рабочих переменных Real Mode вызова.
/// Адресуется из 16-битного кода через esi + offset_of!(...) - см. заголовок файла.
#[repr(C, packed)]
struct RmParams {
    // --- Заполняются вызывающей стороной (Rust) перед rm_bios_disk_call ---
    lba: u32,
    sector_count: u8,
    operation: u8, // OP_READ / OP_WRITE
    buffer_segment: u16,
    buffer_offset: u16,
    drive: u8,

    // --- Результат (заполняется самим вызовом) ---
    result: u8, // 0 - успех, 0xFF - ошибка

    // --- Геометрия диска и CHS (рабочие переменные внутри вызова) ---
    spt: u8,
    heads: u8,
    sector: u8,
    head: u8,
    cyl: u16,
    al_save: u8,
    ah_save: u8,
    tries: u8,

    // --- Предвычисленные смещения меток перехода (см. rm_bios_disk_call) ---
    pm16_offset: u16,
    pm16_return_offset: u16,
    realmode_offset: u16,
    pm32_offset: u32,
    stack_top_offset: u16,

    // --- IDTR: сохранённый (защищённого режима) и временный (Real Mode) ---
    saved_idtr_limit: u16,
    saved_idtr_base: u32,
    real_idtr_limit: u16,
    real_idtr_base: u32,

    // --- Сохранённый ESP вызывающей стороны (см. заголовок файла: SP
    // временно перенаправляется на RM_STACK, а верхняя половина ESP при
    // этом не трогается 16-битным `mov sp` - без явного восстановления
    // popfd/popad в конце читали бы не тот стек) ---
    saved_esp: u32,
}

#[no_mangle]
static mut RM_PARAMS: RmParams = RmParams {
    lba: 0, sector_count: 0, operation: OP_READ,
    buffer_segment: 0, buffer_offset: 0, drive: 0,
    result: 0xFF,
    spt: 0, heads: 0, sector: 0, head: 0, cyl: 0,
    al_save: 0, ah_save: 0, tries: 0,
    pm16_offset: 0, pm16_return_offset: 0, realmode_offset: 0,
    pm32_offset: 0, stack_top_offset: 0,
    saved_idtr_limit: 0, saved_idtr_base: 0,
    // IVT-совместимый IDTR: настоящий IVT по 0000:0000, 256 записей x 4 байта
    real_idtr_limit: 0x03FF, real_idtr_base: 0,
    saved_esp: 0,
};

// Временный стек для работы в Real Mode (собственный кадр kernel32)
#[no_mangle]
static mut RM_STACK: [u8; RM_STACK_SIZE] = [0; RM_STACK_SIZE];

unsafe extern "C" {
    /// Единственная точка входа: сохраняет состояние, уходит в Real Mode,
    /// выполняет int 13h (чтение или запись секторов по параметрам RM_PARAMS),
    /// возвращается в Protected Mode. Результат - в RM_PARAMS.result.
    fn rm_bios_disk_call();
}

core::arch::global_asm!(
    ".global rm_bios_disk_call",
    ".section .text",
    ".code32",
    "rm_bios_disk_call:",
    "    pushad",
    "    pushfd",
    "    cli",

    // esi - база RM_PARAMS на всю функцию (пережив ает дальние переходы и
    // int 13h/int 10h - ни то, ни другое esi не трогает)
    "    lea esi, [RM_PARAMS]",

    // Сохраняем ESP вызывающей стороны - 16-битный `mov sp` ниже перенаправит
    // только младшую половину ESP на временный стек Real Mode, старшая
    // половина при этом не тронется, поэтому перед popfd/popad нужно явно
    // восстановить весь ESP, иначе они прочитают не тот участок стека
    "    mov [esi + {off_saved_esp}], esp",

    // Сохраняем текущий (защищённого режима) IDTR
    "    sidt [esi + {off_saved_idtr}]",

    // Предвычисляем смещения меток ещё в безопасном 32-битном контексте:
    // - pm16_offset / pm16_return_offset / realmode_offset / stack_top_offset:
    //   относительно базы 0x10000 (16-битные дескрипторы GDT И сегмент 0x1000
    //   в настоящем Real Mode - та же база) - маскируем в 16 бит
    // - pm32_offset: полный 32-битный адрес (сегмент kernel code, база 0)
    "    lea eax, [pm16_entry]",
    "    and eax, 0xFFFF",
    "    mov [esi + {off_pm16_offset}], ax",

    "    lea eax, [rm_entry]",
    "    and eax, 0xFFFF",
    "    mov [esi + {off_realmode_offset}], ax",

    "    lea eax, [pm16_return]",
    "    and eax, 0xFFFF",
    "    mov [esi + {off_pm16_return_offset}], ax",

    "    lea eax, [pm32_entry]",
    "    mov [esi + {off_pm32_offset}], eax",

    "    lea eax, [RM_STACK]",
    "    add eax, {stack_size}",
    "    and eax, 0xFFFF",
    "    mov [esi + {off_stack_top_offset}], ax",

    // --- Переход A: 32-битный PM -> 16-битный PM (доп. дескриптор GDT) ---
    "    push {rm_code_sel}",
    "    movzx eax, word ptr [esi + {off_pm16_offset}]",
    "    push eax",
    "    retf",

    ".code16",
    "pm16_entry:",
    // ds всё ещё kernel_data_sel (0x10, база 0) - не менялся с самого вызова,
    // поэтому esi-адресация (esi хранит АБСОЛЮТНЫЙ адрес RM_PARAMS) пока
    // корректна без изменений. ВАЖНО: rm_data_sel сюда НЕ грузим - его база
    // 0x10000 совпадает с esi, и адресация задвоилась бы (esi уже абсолютный
    // адрес, а не смещение от чего-либо)
    "    mov bx, [esi + {off_realmode_offset}]",
    "    mov dx, [esi + {off_stack_top_offset}]",

    // Сбрасываем PE в CR0 - выходим из Protected Mode
    "    mov eax, cr0",
    "    and eax, 0xFFFFFFFE",
    "    mov cr0, eax",

    // ss/sp настраиваем ДО перехода ниже, чтобы сам push+retf уже использовал
    // правильный (не мусорный) стек. 0x1000*16 = 0x10000 - совпадает с базой
    // 16-битных дескрипторов GDT, поэтому уже прочитанное смещение верно
    "    mov ax, 0x1000",
    "    mov ss, ax",
    "    mov sp, dx",

    // Дальний переход довершает вход в настоящий Real Mode
    "    push 0x1000",
    "    push bx",
    "    retf",

    "rm_entry:",
    // ds/es пока хранят "устаревший" protected-mode кэш (0x10, база 0) - это
    // ещё валидно для чтения (сегментные регистры не переоцениваются заново
    // сами по себе при переключении PE, только при явной перезагрузке), но
    // раз мы теперь настоящий Real Mode, явно приводим к сегменту 0 - и то,
    // и другое совпадает по факту (база 0), меняем для ясности/на будущее
    "    xor ax, ax",
    "    mov ds, ax",
    "    mov es, ax",

    // IVT-совместимый IDTR (настоящий Real Mode, база 0000:0000).
    // ❗️ Прерывания ВКЛЮЧАЕМ (sti): некоторые BIOS-сервисы (напр. чтение
    // гибкого диска) сами ждут аппаратный IRQ (IRQ6 контроллера дискет) для
    // сигнала завершения - без прерываний вызов может зависнуть навсегда.
    // Это безопасно ТОЛЬКО потому, что bios_disk_call() на стороне Rust уже
    // перепрошил PIC на "родные" смещения BIOS (08h/70h) перед вызовом -
    // иначе IRQ уходил бы не в настоящий BIOS-обработчик, а в мусор
    "    lidt [esi + {off_real_idtr}]",
    "    sti",

    // --- Геометрия диска: int 13h, ah=8h ---
    "    mov dl, [esi + {off_drive}]",
    "    mov ah, 0x08",
    "    push es",
    "    xor di, di",
    "    mov es, di",
    "    int 0x13",
    "    pop es",
    "    jc rm_fail",

    "    and cl, 0x3F",
    "    mov [esi + {off_spt}], cl",
    "    inc dh",
    "    mov [esi + {off_heads}], dh",

    // --- LBA -> CHS ---
    "    mov eax, [esi + {off_lba}]",
    "    xor edx, edx",
    "    movzx ecx, byte ptr [esi + {off_spt}]",
    "    div ecx",
    "    inc dl",
    "    mov [esi + {off_sector}], dl",

    "    xor edx, edx",
    "    movzx ecx, byte ptr [esi + {off_heads}]",
    "    div ecx",
    "    mov [esi + {off_head}], dl",
    "    mov [esi + {off_cyl}], ax",

    // cl = сектор (0-5 бит) | старшие 2 бита цилиндра (6-7 бит)
    "    mov al, byte ptr [esi + {off_cyl} + 1]",
    "    and al, 0x03",
    "    shl al, 6",
    "    mov cl, [esi + {off_sector}]",
    "    or cl, al",
    "    mov ch, byte ptr [esi + {off_cyl}]",
    "    mov dh, [esi + {off_head}]",
    "    mov dl, [esi + {off_drive}]",

    "    mov ax, [esi + {off_buffer_segment}]",
    "    mov es, ax",
    "    mov bx, [esi + {off_buffer_offset}]",
    "    mov al, [esi + {off_sector_count}]",
    "    mov [esi + {off_al_save}], al",

    // Выбор функции BIOS: 02h - чтение, 03h - запись (operation: 0/1)
    "    mov al, [esi + {off_operation}]",
    "    mov ah, 0x02",
    "    test al, al",
    "    jz rm_retry",
    "    mov ah, 0x03",

    "rm_retry:",
    "    mov [esi + {off_ah_save}], ah",
    "    mov byte ptr [esi + {off_tries}], 3",

    "rm_attempt:",
    "    mov ah, [esi + {off_ah_save}]",
    "    mov al, [esi + {off_al_save}]",
    "    stc",
    "    int 0x13",
    "    jnc rm_done",

    "    dec byte ptr [esi + {off_tries}]",
    "    jz rm_fail",

    // Сброс контроллера диска перед повтором (int 13h, ah=0)
    "    xor ah, ah",
    "    mov dl, [esi + {off_drive}]",
    "    int 0x13",
    "    jmp rm_attempt",

    "rm_done:",
    "    mov byte ptr [esi + {off_result}], 0",
    "    jmp rm_return",

    "rm_fail:",
    "    mov byte ptr [esi + {off_result}], 0xFF",

    "rm_return:",
    // --- Возврат в Protected Mode (двумя шагами, симметрично приходу) ---
    "    cli",
    "    mov eax, cr0",
    "    or eax, 1",
    "    mov cr0, eax",

    // Шаг 1: real mode -> тот же 16-битный PM сегмент
    "    mov bx, [esi + {off_pm16_return_offset}]",
    "    push {rm_code_sel}",
    "    push bx",
    "    retf",

    "pm16_return:",
    // ds всё ещё сегмент 0 (не менялся с rm_entry) - для esi-адресации этого
    // достаточно, rm_data_sel сюда намеренно НЕ грузим (см. pm16_entry)

    // Шаг 2: 16-битный PM -> обычный 32-битный PM ядра (полный физический
    // адрес). 32-битные регистры дают 32-битный push автоматически (0x66);
    // retf кодируется вручную байтами (0x66 + 0xCB) для гарантии 32-битного
    // размера операнда возврата - иначе в 16-битном контексте он был бы 16-бит
    "    mov eax, [esi + {off_pm32_offset}]",
    "    mov ecx, {kernel_code_sel}",
    "    push ecx",
    "    push eax",
    "    .byte 0x66, 0xCB",

    ".code32",
    "pm32_entry:",
    "    mov ax, {kernel_data_sel}",
    "    mov ds, ax",
    "    mov es, ax",
    "    mov fs, ax",
    "    mov gs, ax",
    "    mov ss, ax",

    // Восстанавливаем IDTR защищённого режима (PIT/клавиатура снова рабочие)
    "    lidt [esi + {off_saved_idtr}]",

    // Восстанавливаем ESP вызывающей стороны (см. комментарий в начале
    // функции) - без этого popfd/popad читали бы временный стек Real Mode
    // вместо настоящих сохранённых значений
    "    mov esp, [esi + {off_saved_esp}]",

    "    popfd",
    "    popad",
    "    ret",

    rm_code_sel = const REALMODE_CODE_SELECTOR,
    kernel_code_sel = const crate::x86::gdt::KERNEL_CODE_SELECTOR,
    kernel_data_sel = const crate::x86::gdt::KERNEL_DATA_SELECTOR,
    stack_size = const RM_STACK_SIZE,

    off_lba = const offset_of!(RmParams, lba),
    off_sector_count = const offset_of!(RmParams, sector_count),
    off_operation = const offset_of!(RmParams, operation),
    off_buffer_segment = const offset_of!(RmParams, buffer_segment),
    off_buffer_offset = const offset_of!(RmParams, buffer_offset),
    off_drive = const offset_of!(RmParams, drive),
    off_result = const offset_of!(RmParams, result),
    off_spt = const offset_of!(RmParams, spt),
    off_heads = const offset_of!(RmParams, heads),
    off_sector = const offset_of!(RmParams, sector),
    off_head = const offset_of!(RmParams, head),
    off_cyl = const offset_of!(RmParams, cyl),
    off_al_save = const offset_of!(RmParams, al_save),
    off_ah_save = const offset_of!(RmParams, ah_save),
    off_tries = const offset_of!(RmParams, tries),
    off_pm16_offset = const offset_of!(RmParams, pm16_offset),
    off_pm16_return_offset = const offset_of!(RmParams, pm16_return_offset),
    off_realmode_offset = const offset_of!(RmParams, realmode_offset),
    off_pm32_offset = const offset_of!(RmParams, pm32_offset),
    off_stack_top_offset = const offset_of!(RmParams, stack_top_offset),
    off_saved_idtr = const offset_of!(RmParams, saved_idtr_limit),
    off_real_idtr = const offset_of!(RmParams, real_idtr_limit),
    off_saved_esp = const offset_of!(RmParams, saved_esp),
);

/// Чтение секторов диска через BIOS (Real Mode thunk)
/// Параметры:
///  - lba: начальный сектор (абсолютный, 0-based)
///  - count: количество секторов (1-128)
///  - dest: физический адрес назначения (буфер >= count*512 байт, < 1 МБ)
/// Вывод:
///  - true при успехе
pub fn read_sectors(lba: u32, count: u8, dest: u32) -> bool {
    bios_disk_call(OP_READ, lba, count, dest)
}

/// Запись секторов диска через BIOS (Real Mode thunk)
/// Параметры и вывод - см. read_sectors
pub fn write_sectors(lba: u32, count: u8, dest: u32) -> bool {
    bios_disk_call(OP_WRITE, lba, count, dest)
}

// Порты данных (масок) master/slave PIC - см. x86::pic
const PIC1_DATA: u16 = 0x21;
const PIC2_DATA: u16 = 0xA1;

/// Общая точка входа для чтения/записи секторов
fn bios_disk_call(operation: u8, lba: u32, count: u8, dest: u32) -> bool {
    unsafe {
        RM_PARAMS.lba = lba;
        RM_PARAMS.sector_count = count;
        RM_PARAMS.operation = operation;
        RM_PARAMS.buffer_segment = (dest >> 4) as u16;
        RM_PARAMS.buffer_offset = (dest & 0xF) as u16;
        RM_PARAMS.drive = crate::pcinfo().boot_drive_num;

        // PIC сейчас перепрошит на векторы kernel32 (0x20+, см.
        // x86::pic::remap), а BIOS-чтение гибкого диска само ждёт аппаратный
        // IRQ6 (контроллер дискет) для сигнала завершения - на "чужих"
        // векторах он ушёл бы не в BIOS-обработчик, а в мусор, и вызов
        // завис бы навсегда. Временно возвращаем PIC на "родные" смещения
        // BIOS на время вызова, восстанавливая по возврату.
        //
        // ❗️ Снимаем маску ТОЛЬКО с IRQ6 - не со всех IRQ. Полное снятие
        // маски раньше чинило зависание, но приносило новую проблему: IRQ0
        // (таймер) при снятой маске шёл в "родной" BIOS-обработчик 5.5 раз
        // чаще, чем тот ожидает (kernel32 держит PIT на 100 Гц, а BIOS
        // рассчитан на ~18.2 Гц) - это может сбивать внутренний отсчёт
        // таймаута мотора дискеты у BIOS. А IRQ1 (клавиатура) при снятой
        // маске "перехватывался" бы родным BIOS-обработчиком клавиатуры в
        // его собственный буфер вместо очереди kernel32 - клавиши, нажатые
        // во время дисковой операции, терялись бы для shell. С IRQ6 как
        // единственным снятым - нажатия клавиш просто остаются отложенными
        // (8259A защёлкивает IRQ по фронту даже в маске - см. IRR), доходя
        // до kernel32 сразу после восстановления обычной маски по возврату
        //
        // ❗️ Прерывания на время ЭТОЙ перепрошивки отключаем на уровне CPU
        // (не только внутри rm_bios_disk_call): иначе в окне между
        // reinit_offsets(BIOS) и входом в асм-функцию (где ещё активны
        // protected-mode IDT ядра И уже "родные" смещения PIC) залётный IRQ0
        // ушёл бы на вектор 0x08 - а это обработчик Double Fault в IDT
        // kernel32, не таймер. sti обратно включает уже сама asm-функция,
        // но только после того, как загружен временный Real Mode IDTR
        crate::x86::idt::interrupts_disable();

        let saved_mask1 = crate::utils::inb(PIC1_DATA);
        let saved_mask2 = crate::utils::inb(PIC2_DATA);

        crate::x86::pic::reinit_offsets(
            crate::x86::pic::BIOS_PIC1_OFFSET,
            crate::x86::pic::BIOS_PIC2_OFFSET,
        );
        // Маскируем всё, кроме IRQ6 (бит 6) на master; slave не нужен
        crate::utils::outb(PIC1_DATA, !(1u8 << 6));
        crate::utils::outb(PIC2_DATA, 0xFF);

        rm_bios_disk_call();

        crate::x86::pic::reinit_offsets(
            crate::x86::pic::PIC1_OFFSET,
            crate::x86::pic::PIC2_OFFSET,
        );
        crate::utils::outb(PIC1_DATA, saved_mask1);
        crate::utils::outb(PIC2_DATA, saved_mask2);

        crate::x86::idt::interrupts_enable();

        RM_PARAMS.result == 0
    }
}
